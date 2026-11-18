//! Stage 2: `canon-census 1.0` (ARDOR-PIPELINES.md §2).
//!
//! Canon a title relies on is typed data: group-2 character records,
//! canon rules, declared canon *tables* (one typed row per fact, each
//! with a ruling reference, cells optionally frozen), declared foreign
//! *cast pairs* for differentiation, and the genesis denylist. The
//! census is a pure function of that state: counts against declared
//! totals, orphans in both directions, differentiation numbers per
//! pair, and a sanitization sweep of shippable text. A canon state is
//! *verified* only while a recorded run's input hash equals the
//! current state's hash and that run's verdict is `VERIFIED`.
//!
//! Absolute boundary: nothing here reads, gates, or degrades a safety
//! surface. `safety.json` is not an input to the census at all.

use std::collections::{BTreeMap, BTreeSet};

use serde_json::{Map, Value, json};

use crate::gate::Refusal;
use crate::model::{arr, canonical, find_by_id, fold, is_valid_id, sha256_hex};
use crate::store::ProjectState;

pub const VERSION: &str = "canon-census 1.0";
pub const SUBJECTS: [&str; 2] = ["character", "none"];
pub const CELL_TYPES: [&str; 5] = ["asset", "boolean", "character", "integer", "string"];
pub const MAX_COLUMNS: usize = 64;
pub const MAX_ROWS: usize = 4096;
pub const MAX_PAIR_MEMBERS: usize = 512;
pub const MAX_DESCRIPTOR_FIELDS: usize = 32;
const TEXT_MAX: usize = 4000;
/// Shippable text swept for denylist terms: (document, array key).
/// Safety and systems documents are deliberately not swept.
pub const SWEPT: [(&str, &str); 4] = [
    ("cast", "characters"),
    ("graph", "nodes"),
    ("levels", "levels"),
    ("story", "arcs"),
];

fn nonblank(s: &str, max: usize) -> bool {
    !s.trim().is_empty() && s.len() <= max
}

fn strict<'a>(
    value: &'a Value,
    what: &str,
    allowed: &[&str],
) -> Result<&'a Map<String, Value>, String> {
    let map = value
        .as_object()
        .ok_or_else(|| format!("{what} must be an object"))?;
    if let Some(key) = map.keys().find(|k| !allowed.contains(&k.as_str())) {
        return Err(format!("{what}: unknown field '{key}'"));
    }
    Ok(map)
}

fn text(map: &Map<String, Value>, key: &str, what: &str, max: usize) -> Result<String, String> {
    match map.get(key).and_then(Value::as_str) {
        Some(s) if nonblank(s, max) => Ok(s.to_string()),
        _ => Err(format!(
            "{what}.{key} must be a nonblank string of at most {max} bytes"
        )),
    }
}

fn slug(map: &Map<String, Value>, key: &str, what: &str) -> Result<String, String> {
    match map.get(key).and_then(Value::as_str) {
        Some(s) if is_valid_id(s) => Ok(s.to_string()),
        _ => Err(format!("{what}.{key} must be a lowercase slug")),
    }
}

// ---------- declarations (structural validation: params errors) ----------

/// Validate a canon table declaration. Structural problems are the
/// tool layer's params errors; frozen-cell law is the gate's.
pub fn validate_table(table: &Value) -> Result<(), String> {
    let t = strict(
        table,
        "table",
        &[
            "id",
            "title",
            "subject",
            "required",
            "declared_rows",
            "columns",
            "rows",
        ],
    )?;
    slug(t, "id", "table")?;
    if t.contains_key("title") {
        text(t, "title", "table", 200)?;
    }
    let subject = t.get("subject").and_then(Value::as_str).unwrap_or("");
    if !SUBJECTS.contains(&subject) {
        return Err("table.subject must be character or none".to_string());
    }
    let required = match t.get("required") {
        None => false,
        Some(Value::Bool(b)) => *b,
        _ => return Err("table.required must be boolean".to_string()),
    };
    if required && subject != "character" {
        return Err("only a character table can be required of every character".to_string());
    }
    match t.get("declared_rows").and_then(Value::as_u64) {
        Some(n) if n as usize <= MAX_ROWS => {}
        _ => {
            return Err(format!(
                "table.declared_rows must be an integer 0..={MAX_ROWS}"
            ));
        }
    }
    let columns = t
        .get("columns")
        .and_then(Value::as_array)
        .filter(|c| !c.is_empty() && c.len() <= MAX_COLUMNS)
        .ok_or_else(|| format!("table.columns must hold 1..={MAX_COLUMNS} columns"))?;
    let mut types = BTreeMap::new();
    for column in columns {
        let c = strict(column, "column", &["name", "type"])?;
        let name = slug(c, "name", "column")?;
        let kind = c.get("type").and_then(Value::as_str).unwrap_or("");
        if !CELL_TYPES.contains(&kind) {
            return Err(format!("column {name}: type must be one of {CELL_TYPES:?}"));
        }
        if types.insert(name.clone(), kind).is_some() {
            return Err(format!("column {name} repeated"));
        }
    }
    let rows = t
        .get("rows")
        .and_then(Value::as_array)
        .filter(|r| r.len() <= MAX_ROWS)
        .ok_or_else(|| format!("table.rows must be an array of at most {MAX_ROWS} rows"))?;
    let mut keys = BTreeSet::new();
    for row in rows {
        let r = strict(row, "row", &["key", "cells", "ruling_ref", "frozen"])?;
        let key = slug(r, "key", "row")?;
        if !keys.insert(key.clone()) {
            return Err(format!("row {key} repeated"));
        }
        text(r, "ruling_ref", "row", TEXT_MAX)?;
        let cells = r
            .get("cells")
            .and_then(Value::as_object)
            .ok_or_else(|| format!("row {key}: cells must be an object"))?;
        if cells.len() != types.len() || cells.keys().any(|k| !types.contains_key(k)) {
            return Err(format!(
                "row {key}: cells must name exactly the declared columns"
            ));
        }
        for (name, value) in cells {
            let ok = match types[name] {
                "boolean" => value.is_boolean(),
                "integer" => value.is_i64(),
                "string" => value.as_str().is_some_and(|s| s.len() <= TEXT_MAX),
                _ => value.as_str().is_some_and(is_valid_id),
            };
            if !ok {
                return Err(format!(
                    "row {key}: cell {name} is not a valid {}",
                    types[name]
                ));
            }
        }
        let frozen = match r.get("frozen") {
            None => Vec::new(),
            Some(Value::Array(f)) => f.clone(),
            _ => {
                return Err(format!(
                    "row {key}: frozen must be an array of column names"
                ));
            }
        };
        let mut seen = BTreeSet::new();
        for f in &frozen {
            match f.as_str() {
                Some(name) if types.contains_key(name) && seen.insert(name) => {}
                _ => {
                    return Err(format!(
                        "row {key}: frozen names an unknown or repeated column"
                    ));
                }
            }
        }
    }
    Ok(())
}

/// Validate a declared foreign cast pair.
pub fn validate_pair(pair: &Value) -> Result<(), String> {
    let p = strict(
        pair,
        "pair",
        &[
            "id",
            "foreign_cast",
            "source_ref",
            "descriptor_fields",
            "members",
            "threshold",
        ],
    )?;
    slug(p, "id", "pair")?;
    text(p, "foreign_cast", "pair", 200)?;
    text(p, "source_ref", "pair", TEXT_MAX)?;
    let fields = p
        .get("descriptor_fields")
        .and_then(Value::as_array)
        .filter(|f| !f.is_empty() && f.len() <= MAX_DESCRIPTOR_FIELDS)
        .ok_or_else(|| {
            format!("pair.descriptor_fields must hold 1..={MAX_DESCRIPTOR_FIELDS} names")
        })?;
    let mut field_set = BTreeSet::new();
    for f in fields {
        match f.as_str() {
            Some(name)
                if is_valid_id(name)
                    && name != "id"
                    && name != "name"
                    && field_set.insert(name) => {}
            _ => {
                return Err(
                    "descriptor fields must be unique slugs other than id and name".to_string(),
                );
            }
        }
    }
    let members = p
        .get("members")
        .and_then(Value::as_array)
        .filter(|m| !m.is_empty() && m.len() <= MAX_PAIR_MEMBERS)
        .ok_or_else(|| format!("pair.members must hold 1..={MAX_PAIR_MEMBERS} members"))?;
    let mut names = BTreeSet::new();
    for member in members {
        let m = strict(member, "member", &["name", "descriptors"])?;
        let name = text(m, "name", "member", 200)?;
        if !names.insert(fold(&name)) {
            return Err(format!("member {name} repeated"));
        }
        let descriptors = match m.get("descriptors") {
            None => Map::new(),
            Some(Value::Object(d)) => d.clone(),
            _ => return Err(format!("member {name}: descriptors must be an object")),
        };
        for (key, value) in &descriptors {
            let ok_value = match value {
                Value::String(s) => s.len() <= TEXT_MAX,
                Value::Array(items) => items
                    .iter()
                    .all(|i| i.as_str().is_some_and(|s| s.len() <= TEXT_MAX)),
                _ => false,
            };
            if !field_set.contains(key.as_str()) || !ok_value {
                return Err(format!(
                    "member {name}: descriptor {key} must be a declared field holding a string or string array"
                ));
            }
        }
    }
    if let Some(threshold) = p.get("threshold") {
        let t = strict(threshold, "threshold", &["permille", "ruling_ref"])?;
        match t.get("permille").and_then(Value::as_u64) {
            Some(n) if n <= 1000 => {}
            _ => return Err("threshold.permille must be an integer 0..=1000".to_string()),
        }
        text(t, "ruling_ref", "threshold", TEXT_MAX)?;
    }
    Ok(())
}

// ---------- gate: frozen cells (semantic refusal, before mutation) ----------

/// Frozen cells hold unless the write carries an amendment with a
/// reason; `apply_table` records the amendment in `table_history`.
pub fn decide_table(
    state: &ProjectState,
    table: &Value,
    amendment: Option<&Value>,
) -> Result<(), Refusal> {
    let id = table["id"].as_str().unwrap_or("");
    let amended = amendment
        .and_then(|a| a.get("reason"))
        .and_then(Value::as_str)
        .is_some_and(|r| nonblank(r, TEXT_MAX));
    let Some(old) = find_by_id(state.doc("canon"), "tables", id) else {
        return Ok(());
    };
    let moved = frozen_moves(old, table);
    if !moved.is_empty() && !amended {
        return Err(Refusal::new(
            "canon_frozen_cell",
            format!(
                "table '{id}': frozen cells {moved:?} would change or unfreeze. \
                 Resubmit with an amendment (reason required) to change canon deliberately."
            ),
        ));
    }
    Ok(())
}

/// `row.column` for every frozen cell of `old` that `new` drops,
/// changes, or unfreezes. Sorted, so history entries are stable.
fn frozen_moves(old: &Value, new: &Value) -> Vec<String> {
    let mut moved = Vec::new();
    for row in arr(old, "rows") {
        let key = row["key"].as_str().unwrap_or("");
        let next = arr(new, "rows").iter().find(|r| r["key"] == key);
        for column in arr(row, "frozen").iter().filter_map(Value::as_str) {
            let held = next.is_some_and(|n| {
                n["cells"][column] == row["cells"][column]
                    && arr(n, "frozen").iter().any(|f| f == column)
            });
            if !held {
                moved.push(format!("{key}.{column}"));
            }
        }
    }
    moved.sort();
    moved
}

pub fn apply_table(state: &mut ProjectState, table: Value, amendment: Option<Value>) {
    let id = table["id"].as_str().unwrap_or("").to_string();
    let old = find_by_id(state.doc("canon"), "tables", &id).cloned();
    let mut record = table;
    let version = old
        .as_ref()
        .and_then(|o| o["version"].as_i64())
        .unwrap_or(0)
        + 1;
    record["version"] = json!(version);
    let canon = state.doc_mut("canon");
    match crate::model::find_by_id_mut(canon, "tables", &id) {
        Some(slot) => *slot = record.clone(),
        None => crate::model::arr_mut(canon, "tables").push(record.clone()),
    }
    if let Some(old) = old {
        let seq = arr(canon, "table_history").len() + 1;
        let reason = amendment
            .as_ref()
            .and_then(|a| a.get("reason"))
            .and_then(Value::as_str)
            .map(str::to_string);
        crate::model::arr_mut(canon, "table_history").push(json!({
            "seq": seq,
            "table": id,
            "version": version,
            "amended": reason.is_some(),
            "reason": reason,
            "frozen_changes": frozen_moves(&old, &record)
        }));
    }
}

pub fn apply_pair(state: &mut ProjectState, pair: Value) {
    let id = pair["id"].as_str().unwrap_or("").to_string();
    let canon = state.doc_mut("canon");
    match crate::model::find_by_id_mut(canon, "cast_pairs", &id) {
        Some(slot) => *slot = pair,
        None => crate::model::arr_mut(canon, "cast_pairs").push(pair),
    }
}

// ---------- the census (pure) ----------

/// Exactly what the census reads. Any change to it makes a recorded
/// census stale. Asset records contribute only their ids.
fn input(state: &ProjectState) -> Value {
    let mut asset_ids: Vec<&str> = arr(state.doc("assets"), "assets")
        .iter()
        .filter_map(|a| a["id"].as_str())
        .collect();
    asset_ids.sort();
    let denylist = match crate::genesis::denylist(state) {
        Ok(terms) => json!({ "terms": terms }),
        Err(code) => json!({ "unavailable": code }),
    };
    json!({
        "asset_ids": asset_ids,
        "canon": state.doc("canon"),
        "cast": state.doc("cast"),
        "denylist": denylist,
        "graph": state.doc("graph"),
        "levels": state.doc("levels"),
        "story": state.doc("story"),
    })
}

pub fn input_hash(state: &ProjectState) -> String {
    sha256_hex(&canonical(&input(state)))
}

fn tokens(fields: &[&str], source: &Value) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for field in fields {
        match &source[*field] {
            Value::String(s) => {
                out.insert(format!("{field}={}", fold(s)));
            }
            Value::Array(items) => {
                for s in items.iter().filter_map(Value::as_str) {
                    out.insert(format!("{field}={}", fold(s)));
                }
            }
            _ => {}
        }
    }
    out.retain(|t| !t.ends_with('='));
    out
}

/// The stated numeric leg: integer per-mille Jaccard overlap of the
/// folded `field=value` descriptor tokens (0 when both sets are empty).
pub fn overlap_permille(a: &BTreeSet<String>, b: &BTreeSet<String>) -> u64 {
    let union = a.union(b).count() as u64;
    if union == 0 {
        return 0;
    }
    a.intersection(b).count() as u64 * 1000 / union
}

fn differentiation(state: &ProjectState, pair: &Value) -> Value {
    let fields: Vec<&str> = arr(pair, "descriptor_fields")
        .iter()
        .filter_map(Value::as_str)
        .collect();
    let mut collisions = Vec::new();
    let mut overlaps = Vec::new();
    let mut max = 0;
    for home in arr(state.doc("cast"), "characters") {
        let home_id = home["id"].as_str().unwrap_or("");
        let home_name = fold(home["name"].as_str().unwrap_or(""));
        let home_tokens = tokens(&fields, home);
        for member in arr(pair, "members") {
            let name = member["name"].as_str().unwrap_or("");
            if !home_name.is_empty() && home_name == fold(name) {
                collisions.push(json!({"home": home_id, "foreign": name}));
            }
            let permille = overlap_permille(&home_tokens, &tokens(&fields, &member["descriptors"]));
            max = max.max(permille);
            if permille > 0 {
                overlaps.push(json!({"home": home_id, "foreign": name, "permille": permille}));
            }
        }
    }
    let threshold = pair["threshold"]["permille"].as_u64();
    let verdict = if !collisions.is_empty() || threshold.is_some_and(|t| max > t) {
        "collision"
    } else if threshold.is_none() {
        "awaiting_owner_ruling"
    } else {
        "within_threshold"
    };
    json!({
        "pair": pair["id"],
        "foreign_cast": pair["foreign_cast"],
        "name_collisions": collisions,
        "overlaps": overlaps,
        "max_permille": max,
        "threshold_permille": threshold,
        "verdict": verdict
    })
}

fn sweep(
    value: &Value,
    pointer: &str,
    terms: &[(String, String)],
    doc: &str,
    record: &Value,
    hits: &mut Vec<Value>,
) {
    match value {
        Value::String(s) => {
            let folded = fold(s);
            for (term, folded_term) in terms {
                if folded.contains(folded_term.as_str()) {
                    hits.push(
                        json!({"term": term, "doc": doc, "record": record, "pointer": pointer}),
                    );
                }
            }
        }
        Value::Array(items) => {
            for (i, item) in items.iter().enumerate() {
                sweep(item, &format!("{pointer}/{i}"), terms, doc, record, hits);
            }
        }
        Value::Object(map) => {
            for (key, item) in map {
                let escaped = key.replace('~', "~0").replace('/', "~1");
                sweep(
                    item,
                    &format!("{pointer}/{escaped}"),
                    terms,
                    doc,
                    record,
                    hits,
                );
            }
        }
        _ => {}
    }
}

/// The census artifact: a pure function of canon state.
pub fn census(state: &ProjectState) -> Value {
    let canon = state.doc("canon");
    let cast = arr(state.doc("cast"), "characters");
    let character_ids: BTreeSet<&str> = cast.iter().filter_map(|c| c["id"].as_str()).collect();
    let asset_ids: BTreeSet<&str> = arr(state.doc("assets"), "assets")
        .iter()
        .filter_map(|a| a["id"].as_str())
        .collect();
    let mut findings: BTreeMap<&str, Vec<Value>> = BTreeMap::new();

    // Counts per table against declared totals.
    let mut table_counts = Vec::new();
    for table in arr(canon, "tables") {
        let declared = table["declared_rows"].as_u64().unwrap_or(0);
        let found = arr(table, "rows").len() as u64;
        table_counts.push(json!({"table": table["id"], "declared": declared, "found": found}));
        if declared != found {
            findings
                .entry("count_mismatch")
                .or_default()
                .push(json!({"table": table["id"], "declared": declared, "found": found}));
        }
    }

    // Orphans, both directions.
    let mut orphans = Vec::new();
    for rule in arr(canon, "rules") {
        let kind = rule["kind"].as_str().unwrap_or("");
        let subject = rule["character"].as_str().unwrap_or("");
        if (kind == "lock" || kind == "fact") && !character_ids.contains(subject) {
            orphans
                .push(json!({"kind": "rule_subject_missing", "rule": rule["id"], "id": subject}));
        }
    }
    for table in arr(canon, "tables") {
        let types: BTreeMap<&str, &str> = arr(table, "columns")
            .iter()
            .filter_map(|c| Some((c["name"].as_str()?, c["type"].as_str()?)))
            .collect();
        let character_table = table["subject"] == "character";
        let keys: BTreeSet<&str> = arr(table, "rows")
            .iter()
            .filter_map(|r| r["key"].as_str())
            .collect();
        for row in arr(table, "rows") {
            let key = row["key"].as_str().unwrap_or("");
            if character_table && !character_ids.contains(key) {
                orphans
                    .push(json!({"kind": "row_subject_missing", "table": table["id"], "id": key}));
            }
            for (column, kind) in &types {
                let target = row["cells"][*column].as_str().unwrap_or("");
                let missing = match *kind {
                    "character" => !character_ids.contains(target),
                    "asset" => !asset_ids.contains(target),
                    _ => false,
                };
                if missing {
                    orphans.push(json!({"kind": "cell_ref_missing", "table": table["id"], "row": key, "column": column, "id": target}));
                }
            }
        }
        if table["required"] == true {
            for id in &character_ids {
                if !keys.contains(id) {
                    orphans.push(json!({"kind": "record_missing_from_table", "table": table["id"], "id": id}));
                }
            }
        }
    }
    orphans.sort_by_key(canonical);
    if !orphans.is_empty() {
        findings.insert("canon_orphan", orphans.clone());
    }

    // Differentiation per declared pair.
    let mut verdicts = Vec::new();
    for pair in arr(canon, "cast_pairs") {
        let d = differentiation(state, pair);
        match d["verdict"].as_str() {
            Some("collision") => findings
                .entry("differentiation_collision")
                .or_default()
                .push(pair["id"].clone()),
            Some("awaiting_owner_ruling") => findings
                .entry("differentiation_unruled")
                .or_default()
                .push(pair["id"].clone()),
            _ => {}
        }
        verdicts.push(d);
    }

    // Sanitization sweep of shippable text against the genesis denylist.
    let mut hits = Vec::new();
    let denylist = crate::genesis::denylist(state);
    let terms_count = match &denylist {
        Ok(terms) => {
            let folded: Vec<(String, String)> =
                terms.iter().map(|t| (t.clone(), fold(t))).collect();
            for (doc, key) in SWEPT {
                for (i, record) in arr(state.doc(doc), key).iter().enumerate() {
                    sweep(
                        record,
                        &format!("/{key}/{i}"),
                        &folded,
                        doc,
                        &record["id"],
                        &mut hits,
                    );
                }
            }
            json!(terms.len())
        }
        Err(code) => {
            findings
                .entry(code)
                .or_default()
                .push(json!("denylist unavailable: sanitization not swept"));
            Value::Null
        }
    };
    if !hits.is_empty() {
        findings.insert("sanitization_hit", hits.clone());
    }

    let codes: Vec<&str> = findings.keys().copied().collect();
    json!({
        "gate": VERSION,
        "project": state.slug,
        "input_state_hash": input_hash(state),
        "counts": {
            "characters": cast.len(),
            "canon_rules": arr(canon, "rules").len(),
            "tables": table_counts,
            "cast_pairs": arr(canon, "cast_pairs").len(),
            "denylist_terms": terms_count,
            "orphans": orphans.len(),
            "sanitization_hits": hits.len()
        },
        "orphans": orphans,
        "differentiation": verdicts,
        "sanitization": {"swept": SWEPT.iter().map(|(d, k)| format!("{d}.{k}")).collect::<Vec<_>>(), "swept_ok": denylist.is_ok(), "hits": hits},
        "findings": findings,
        "refusal_codes": codes,
        "verdict": if codes.is_empty() { "VERIFIED" } else { "REFUSED" }
    })
}

/// Every Owner ruling the census relied on: table rows, pair
/// thresholds, and the genesis ledger's denylist entries.
fn ruling_refs(state: &ProjectState) -> Vec<Value> {
    let mut refs = BTreeSet::new();
    for table in arr(state.doc("canon"), "tables") {
        for row in arr(table, "rows") {
            if let Some(r) = row["ruling_ref"].as_str() {
                refs.insert(r.to_string());
            }
        }
    }
    for pair in arr(state.doc("canon"), "cast_pairs") {
        if let Some(r) = pair["threshold"]["ruling_ref"].as_str() {
            refs.insert(r.to_string());
        }
    }
    for entry in arr(state.doc("genesis"), "ledger") {
        if entry["kind"] == "denylist"
            && let Some(r) = entry["ruling_ref"].as_str()
        {
            refs.insert(r.to_string());
        }
    }
    refs.into_iter().map(Value::String).collect()
}

/// A §7 run record for the census of the current state.
pub fn run_record(state: &ProjectState) -> Value {
    let artifact = census(state);
    json!({
        "seq": arr(state.doc("census"), "runs").len() + 1,
        "pipeline": "canon",
        "pipeline_version": "1.0",
        "gate": VERSION,
        "input_state_hash": artifact["input_state_hash"],
        "output_artifact_hashes": {"census": sha256_hex(&canonical(&artifact))},
        "verdict": artifact["verdict"],
        "refusal_codes": artifact["refusal_codes"],
        "ruling_refs": ruling_refs(state),
        "artifact": artifact
    })
}

pub fn apply_run(state: &mut ProjectState) {
    let run = run_record(state);
    crate::model::arr_mut(state.doc_mut("census"), "runs").push(run);
}

fn run_intact(index: usize, run: &Value) -> bool {
    run["seq"] == json!(index + 1)
        && run["pipeline"] == "canon"
        && run["pipeline_version"] == "1.0"
        && run["gate"] == VERSION
        && run["output_artifact_hashes"]["census"] == sha256_hex(&canonical(&run["artifact"]))
        && run["input_state_hash"] == run["artifact"]["input_state_hash"]
        && run["verdict"] == run["artifact"]["verdict"]
        && run["refusal_codes"] == run["artifact"]["refusal_codes"]
}

/// Whether canon is verified *now*. Downstream gates (Stages 3-5)
/// call this; it never consults or affects safety state.
pub fn require_verified(state: &ProjectState) -> Result<(), Refusal> {
    let runs = arr(state.doc("census"), "runs");
    let Some(latest) = runs.last() else {
        return Err(Refusal::new(
            "canon_unverified",
            "no canon-census 1.0 run is recorded",
        ));
    };
    if runs.iter().enumerate().any(|(i, r)| !run_intact(i, r)) {
        return Err(Refusal::new(
            "census_run_mismatch",
            "a recorded census run does not match its artifact hash or sequence (REVIEW)",
        ));
    }
    let current = input_hash(state);
    if latest["input_state_hash"] != json!(current) {
        return Err(Refusal::new(
            "census_stale",
            format!(
                "canon state {current} differs from census run {} input {}",
                latest["seq"], latest["input_state_hash"]
            ),
        ));
    }
    if latest["artifact"] != census(state) {
        return Err(Refusal::new(
            "census_run_mismatch",
            "the recorded census does not re-derive from current state (REVIEW)",
        ));
    }
    if latest["verdict"] != "VERIFIED" {
        let codes: Vec<&str> = arr(latest, "refusal_codes")
            .iter()
            .filter_map(Value::as_str)
            .collect();
        // The spec's specific codes first; any other finding (count
        // mismatch, unruled pair or denylist) leaves canon unverified.
        let code = [
            "canon_orphan",
            "differentiation_collision",
            "sanitization_hit",
        ]
        .into_iter()
        .find(|c| codes.contains(c))
        .unwrap_or("canon_unverified");
        return Err(Refusal::new(
            code,
            format!(
                "census run {} refused verification: {codes:?}",
                latest["seq"]
            ),
        ));
    }
    Ok(())
}

/// Read-only view: the census of current state, and verification.
pub fn get_state(state: &ProjectState) -> Value {
    let runs = arr(state.doc("census"), "runs");
    let status = match require_verified(state) {
        Ok(()) => json!({"verified": true, "code": null, "message": null}),
        Err(r) => json!({"verified": false, "code": r.code, "message": r.message}),
    };
    json!({
        "project": state.slug,
        "gate": VERSION,
        "census": census(state),
        "status": status,
        "runs_recorded": runs.len(),
        "latest_run": runs.last().map(|r| json!({
            "seq": r["seq"],
            "input_state_hash": r["input_state_hash"],
            "output_artifact_hashes": r["output_artifact_hashes"],
            "verdict": r["verdict"],
            "refusal_codes": r["refusal_codes"]
        }))
    })
}
