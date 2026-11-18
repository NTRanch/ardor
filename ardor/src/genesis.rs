//! Stage 1: versioned genesis records and pure census/decision logic.
//! No research is performed here: evidence and Owner rulings are inputs.
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::gate::Refusal;
use crate::model::{arr, canonical, sha256_hex};
use crate::store::ProjectState;

pub const ITEMS: [&str; 7] = [
    "platforms",
    "connectivity",
    "release_model",
    "art_audio_pipeline",
    "voice",
    "language",
    "accessibility",
];
pub const SEED_PATHS: [&str; 5] = [
    "LICENSE",
    "README.md",
    "design/DESIGN.md",
    "tests/README.md",
    "DECISIONS.md",
];
pub const VERSION: &str = "ardor-genesis-sheet 1.0";

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Ruling {
    pub item: String,
    pub state: DefaultState,
    pub value: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum DefaultState {
    Confirmed,
    ReRuled,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Evidence {
    pub url: String,
    pub grade: Grade,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Grade {
    Primary,
    Secondary,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Leg {
    pub id: String,
    pub finding: String,
    pub verified: bool,
    pub evidence: Vec<Evidence>,
    pub owed_steps: Vec<String>,
    pub outcome: Verdict,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub enum Verdict {
    #[serde(rename = "CLEAR")]
    Clear,
    #[serde(rename = "CLEAR WITH NOTES")]
    Notes,
    #[serde(rename = "COLLISION")]
    Collision,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Clearance {
    pub candidate: String,
    pub verdict: Verdict,
    pub legs: Vec<Leg>,
    pub notes: Vec<String>,
}

/// Proposals only. No value is an Owner ruling; all start inherited.
pub fn profile() -> Value {
    serde_json::from_str(include_str!("../schemas/ntranch-defaults-1.0.json"))
        .expect("embedded profile is valid JSON")
}

/// A self-contained fragment for MCP catalog discovery.
pub fn schema_definition(name: &str) -> Value {
    let schema: Value = serde_json::from_str(include_str!(
        "../schemas/ardor-genesis-sheet-1.0.schema.json"
    ))
    .expect("embedded schema is valid JSON");
    inline_schema(&schema["$defs"][name], &schema["$defs"])
}

// MCP fragments are nested under inputSchema.properties, so local refs must
// be expanded rather than accidentally resolving against the tool root.
fn inline_schema(value: &Value, defs: &Value) -> Value {
    match value {
        Value::Object(map) => {
            if let Some(reference) = map.get("$ref").and_then(Value::as_str) {
                return inline_schema(
                    &defs[reference
                        .strip_prefix("#/$defs/")
                        .expect("local schema ref")],
                    defs,
                );
            }
            Value::Object(
                map.iter()
                    .map(|(key, value)| (key.clone(), inline_schema(value, defs)))
                    .collect(),
            )
        }
        Value::Array(values) => {
            Value::Array(values.iter().map(|v| inline_schema(v, defs)).collect())
        }
        _ => value.clone(),
    }
}

pub fn initialize(state: &mut ProjectState) {
    let profile = profile();
    let defaults: Vec<Value> = ITEMS
        .iter()
        .map(|id| {
            json!({
                "id":id, "state":"inherited", "value":profile["items"][id], "ruling_ref":null
            })
        })
        .collect();
    let files = json!({
        "LICENSE":"License selection requires an Owner ruling. No license grant is made by this scaffold.\n",
        "README.md":format!("# {}\n\nWorking title; adoption pending clearance.\n", state.slug),
        "design/DESIGN.md":"# Design\n\nRecord ruled requirements here.\n",
        "tests/README.md":"# Tests\n\nRecord validation and adversarial gates here.\n",
        "DECISIONS.md":"# Decisions\n\nAppend-only rulings are held in genesis.json ledger.\n"
    });
    let manifest: Vec<Value> = files.as_object().expect("seed object").iter().map(|(path, content)| {
        json!({"path":path,"sha256":sha256_hex(content.as_str().expect("seed text")),"content":content})
    }).collect();
    let targets = state.meta()["channel_targets"]
        .as_array()
        .cloned()
        .unwrap_or_default();
    let candidate = state.meta()["name"].clone();
    *state.doc_mut("genesis") = json!({
        "schema":VERSION,"profile":profile,"defaults":defaults,"candidate":candidate,"initial_candidate":candidate,
        "channel_targets":targets,"clearance":null,"adoption":null,
        "denylist":[],"seed_manifest":manifest,"ledger":[],"runs":[]
    });
}

fn nonblank(s: &str) -> bool {
    !s.trim().is_empty() && s.len() <= 4000
}
fn refusal(code: &'static str, message: impl Into<String>) -> Refusal {
    Refusal::new(code, message)
}

pub fn decide_rulings(
    state: &ProjectState,
    rulings: &[Ruling],
    reference: &str,
) -> Result<(), Refusal> {
    if state.doc("genesis")["schema"] != VERSION {
        return Err(refusal(
            "genesis_missing",
            "legacy project has no genesis profile",
        ));
    }
    if !runs_match(state) {
        return Err(refusal(
            "genesis_run_mismatch",
            "historical conformance record does not match its artifact or predecessor",
        ));
    }
    if !ledger_matches(state.doc("genesis")) {
        return Err(refusal(
            "genesis_ledger_mismatch",
            "current state does not match ledger",
        ));
    }
    if !nonblank(reference) || rulings.is_empty() || rulings.len() > ITEMS.len() {
        return Err(refusal(
            "invalid_default_ruling",
            "provide 1-7 items and a nonblank ruling_ref",
        ));
    }
    let mut seen = std::collections::BTreeSet::new();
    for ruling in rulings {
        if !ITEMS.contains(&ruling.item.as_str())
            || !seen.insert(&ruling.item)
            || !nonblank(&ruling.value)
        {
            return Err(refusal(
                "invalid_default_ruling",
                format!("invalid or repeated item: {}", ruling.item),
            ));
        }
        if ruling.state == DefaultState::Confirmed
            && state.doc("genesis")["profile"]["items"][&ruling.item] != ruling.value
        {
            return Err(refusal(
                "default_value_changed",
                format!("{} requires re-ruled", ruling.item),
            ));
        }
        if ruling.item == "accessibility"
            && ruling.value != state.doc("genesis")["profile"]["items"]["accessibility"]
        {
            return Err(refusal(
                "safety_surface_immutable",
                "accessibility must remain enabled and ungated",
            ));
        }
    }
    Ok(())
}

/// Exact leg coverage includes every intended group-9 channel. Evidence grades
/// are attestations, not a claim that Ardor visited or verified any URL.
pub fn validate_clearance(state: &ProjectState, sheet: &Clearance) -> Result<(), Refusal> {
    let g = state.doc("genesis");
    if g["schema"] != VERSION {
        return Err(refusal("genesis_missing", "no genesis profile"));
    }
    if !nonblank(&sheet.candidate) {
        return Err(refusal(
            "clearance_candidate_mismatch",
            "candidate must be nonblank",
        ));
    }
    let mut required: std::collections::BTreeSet<String> = [
        "package:crates.io",
        "namespace:github",
        "namespace:mcp",
        "storefront:itch_io",
        "trademark",
        "domain",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();
    for target in arr(g, "channel_targets") {
        required.insert(format!("storefront:{}", target.as_str().unwrap_or("")));
    }
    let mut seen = std::collections::BTreeSet::new();
    let mut collision = false;
    let mut notes = !sheet.notes.is_empty();
    if sheet.notes.iter().any(|s| !nonblank(s)) {
        return Err(refusal("invalid_clearance", "blank note"));
    }
    for leg in &sheet.legs {
        if !required.contains(&leg.id)
            || !seen.insert(leg.id.clone())
            || !nonblank(&leg.finding)
            || leg.evidence.is_empty()
            || leg.evidence.len() > 32
        {
            return Err(refusal(
                "invalid_clearance",
                format!("invalid, repeated, or evidence-free leg: {}", leg.id),
            ));
        }
        if leg.evidence.iter().any(|e| {
            !(e.url.starts_with("https://") || e.url.starts_with("http://"))
                || e.url.len() > 4000
                || e.url
                    .split("://")
                    .nth(1)
                    .is_none_or(|s| s.is_empty() || s.contains('@'))
        }) {
            return Err(refusal(
                "invalid_clearance",
                format!("invalid evidence URL: {}", leg.id),
            ));
        }
        let primary = leg.evidence.iter().any(|e| e.grade == Grade::Primary);
        if (!primary || !leg.verified) && leg.owed_steps.is_empty()
            || leg.owed_steps.iter().any(|s| !nonblank(s))
            || leg.verified && !primary
        {
            return Err(refusal(
                "clearance_evidence_unverified",
                format!(
                    "{}: secondary/unverified evidence requires owed steps and cannot be verified",
                    leg.id
                ),
            ));
        }
        collision |= leg.outcome == Verdict::Collision;
        notes |= leg.outcome == Verdict::Notes || !leg.owed_steps.is_empty() || !leg.verified;
    }
    if seen != required {
        return Err(refusal(
            "name_conditional",
            format!(
                "missing legs: {:?}",
                required.difference(&seen).collect::<Vec<_>>()
            ),
        ));
    }
    let expected = if collision {
        Verdict::Collision
    } else if notes {
        Verdict::Notes
    } else {
        Verdict::Clear
    };
    if sheet.verdict != expected {
        return Err(refusal(
            "clearance_verdict_mismatch",
            "verdict disagrees with legs, notes, or owed steps",
        ));
    }
    Ok(())
}

pub fn decide_clearance(
    state: &ProjectState,
    sheet: &Clearance,
    reference: &str,
    adopt: bool,
) -> Result<(), Refusal> {
    if !runs_match(state) {
        return Err(refusal(
            "genesis_run_mismatch",
            "historical conformance record does not match its artifact or predecessor",
        ));
    }
    if !ledger_matches(state.doc("genesis")) {
        return Err(refusal(
            "genesis_ledger_mismatch",
            "current state does not match ledger",
        ));
    }
    validate_clearance(state, sheet)?;
    if !nonblank(reference) {
        return Err(refusal(
            "ruling_ref_required",
            "record a ruling reference (including explicit acceptance of notes when adopting)",
        ));
    }
    if adopt && sheet.verdict == Verdict::Collision {
        return Err(refusal(
            "clearance_collision",
            format!(
                "candidate collides: {:?}",
                sheet
                    .legs
                    .iter()
                    .filter(|leg| leg.outcome == Verdict::Collision)
                    .map(|leg| &leg.id)
                    .collect::<Vec<_>>()
            ),
        ));
    }
    Ok(())
}

pub fn apply_rulings(state: &mut ProjectState, rulings: Vec<Ruling>, reference: String) {
    let before = sha256_hex(&canonical(state.doc("genesis")));
    for ruling in &rulings {
        let entry = crate::model::arr_mut(state.doc_mut("genesis"), "defaults")
            .iter_mut()
            .find(|i| i["id"] == ruling.item)
            .expect("profile item");
        *entry = json!({"id":ruling.item,"state":ruling.state,"value":ruling.value,"ruling_ref":reference});
    }
    append(state, "defaults", json!(rulings), reference, before);
}

pub fn apply_clearance(state: &mut ProjectState, sheet: Clearance, reference: String, adopt: bool) {
    let before = sha256_hex(&canonical(state.doc("genesis")));
    let g = state.doc_mut("genesis");
    g["candidate"] = json!(sheet.candidate);
    g["clearance"] = json!(sheet);
    g["adoption"] = if adopt {
        json!({"candidate":sheet.candidate,"ruling_ref":reference,"clearance_hash":sha256_hex(&canonical(&json!(sheet)))})
    } else {
        Value::Null
    };
    append(
        state,
        "clearance",
        json!({"sheet":sheet,"adopt":adopt}),
        reference,
        before,
    );
}

/// Bounds for the Stage 2 denylist (studio names, real persons, real
/// projects). Terms are compared case- and whitespace-folded.
pub const DENYLIST_MAX_TERMS: usize = 1024;

fn validate_denylist(terms: &[String]) -> Result<(), Refusal> {
    if terms.len() > DENYLIST_MAX_TERMS {
        return Err(refusal(
            "invalid_denylist",
            format!("at most {DENYLIST_MAX_TERMS} terms"),
        ));
    }
    let mut seen = std::collections::BTreeSet::new();
    for term in terms {
        if !nonblank(term) || !seen.insert(crate::model::fold(term)) {
            return Err(refusal(
                "invalid_denylist",
                format!("blank, oversized or repeated term: {term:?}"),
            ));
        }
    }
    Ok(())
}

/// Stage 2: the denylist is genesis state, replaced whole by an
/// Owner-referenced ledger entry (never edited in place).
pub fn decide_denylist(
    state: &ProjectState,
    terms: &[String],
    reference: &str,
) -> Result<(), Refusal> {
    if state.doc("genesis")["schema"] != VERSION {
        return Err(refusal(
            "genesis_missing",
            "legacy project has no genesis profile",
        ));
    }
    if !runs_match(state) {
        return Err(refusal(
            "genesis_run_mismatch",
            "historical conformance record does not match its artifact or predecessor",
        ));
    }
    if !ledger_matches(state.doc("genesis")) {
        return Err(refusal(
            "genesis_ledger_mismatch",
            "current state does not match ledger",
        ));
    }
    if !nonblank(reference) {
        return Err(refusal(
            "ruling_ref_required",
            "a denylist change records its ruling reference",
        ));
    }
    validate_denylist(terms)
}

pub fn apply_denylist(state: &mut ProjectState, terms: Vec<String>, reference: String) {
    let before = sha256_hex(&canonical(state.doc("genesis")));
    state.doc_mut("genesis")["denylist"] = json!(terms);
    append(
        state,
        "denylist",
        json!({ "terms": terms }),
        reference,
        before,
    );
}

/// The denylist the canon census sweeps with, only when genesis state
/// is present and re-derives from its ledger; otherwise the typed code.
pub fn denylist(state: &ProjectState) -> Result<Vec<String>, &'static str> {
    let g = state.doc("genesis");
    if g["schema"] != VERSION {
        return Err("genesis_missing");
    }
    if !ledger_matches(g) {
        return Err("genesis_ledger_mismatch");
    }
    // An empty list is a valid ruling; never having ruled is not.
    if !arr(g, "ledger").iter().any(|e| e["kind"] == "denylist") {
        return Err("denylist_unruled");
    }
    Ok(arr(g, "denylist")
        .iter()
        .filter_map(Value::as_str)
        .map(str::to_string)
        .collect())
}

fn append(
    state: &mut ProjectState,
    kind: &str,
    payload: Value,
    reference: String,
    input_hash: String,
) {
    let g = state.doc_mut("genesis");
    let seq = arr(g, "ledger").len() + 1;
    crate::model::arr_mut(g, "ledger").push(
        json!({"id":format!("D-{seq}"),"kind":kind,"ruling_ref":reference,"payload":payload}),
    );
    let sheet = sheet(state);
    let census = sheet_census(state);
    let run = json!({"pipeline":"genesis","pipeline_version":"1.0","gate":VERSION,"input_state_hash":input_hash,
        "output_artifact_hashes":{"sheet":sha256_hex(&canonical(&sheet))},
        "verdict":if census["complete"] == true {"PASS"} else {"REVIEW"},
        "findings":census["findings"],"ruling_refs":arr(state.doc("genesis"),"ledger").iter().map(|e| e["ruling_ref"].clone()).collect::<Vec<_>>(),
        "artifact":sheet});
    crate::model::arr_mut(state.doc_mut("genesis"), "runs").push(run);
}

pub fn sheet(state: &ProjectState) -> Value {
    let mut sheet = state.doc("genesis").clone();
    if let Some(obj) = sheet.as_object_mut() {
        obj.remove("runs");
    }
    sheet
}

// Re-derive current state from the append-only ledger before trusting rulings.
fn ledger_matches(g: &Value) -> bool {
    if g["profile"] != profile() {
        return false;
    }
    let mut defaults: Vec<Value> = ITEMS.iter().map(|id| json!({"id":id,"state":"inherited","value":g["profile"]["items"][id],"ruling_ref":null})).collect();
    let mut clearance = Value::Null;
    let mut adoption = Value::Null;
    let mut candidate = g["initial_candidate"].clone();
    let mut denylist = json!([]);
    for (index, entry) in arr(g, "ledger").iter().enumerate() {
        let Some(reference) = entry["ruling_ref"].as_str().filter(|s| nonblank(s)) else {
            return false;
        };
        if entry["id"] != format!("D-{}", index + 1) {
            return false;
        }
        match entry["kind"].as_str() {
            Some("defaults") => {
                let Ok(rulings) = serde_json::from_value::<Vec<Ruling>>(entry["payload"].clone())
                else {
                    return false;
                };
                let mut seen = std::collections::BTreeSet::new();
                if rulings.is_empty() || rulings.len() > ITEMS.len() {
                    return false;
                }
                for ruling in rulings {
                    let Some(slot) = defaults.iter_mut().find(|i| i["id"] == ruling.item) else {
                        return false;
                    };
                    if !seen.insert(ruling.item.clone())
                        || !nonblank(&ruling.value)
                        || (ruling.state == DefaultState::Confirmed
                            || ruling.item == "accessibility")
                            && g["profile"]["items"][&ruling.item] != ruling.value
                    {
                        return false;
                    }
                    *slot = json!({"id":ruling.item,"state":ruling.state,"value":ruling.value,"ruling_ref":reference});
                }
            }
            Some("clearance") => {
                let Ok(sheet) =
                    serde_json::from_value::<Clearance>(entry["payload"]["sheet"].clone())
                else {
                    return false;
                };
                let Some(adopt) = entry["payload"]["adopt"].as_bool() else {
                    return false;
                };
                if adopt && sheet.verdict == Verdict::Collision {
                    return false;
                }
                candidate = json!(sheet.candidate);
                clearance = json!(sheet);
                adoption = if adopt {
                    json!({"candidate":candidate,"ruling_ref":reference,"clearance_hash":sha256_hex(&canonical(&clearance))})
                } else {
                    Value::Null
                };
            }
            Some("denylist") => {
                let Ok(terms) =
                    serde_json::from_value::<Vec<String>>(entry["payload"]["terms"].clone())
                else {
                    return false;
                };
                if validate_denylist(&terms).is_err() {
                    return false;
                }
                denylist = json!(terms);
            }
            _ => return false,
        }
    }
    g["defaults"] == json!(defaults)
        && g["clearance"] == clearance
        && g["adoption"] == adoption
        && g["candidate"] == candidate
        && g["denylist"] == denylist
}

fn sheet_census(state: &ProjectState) -> Value {
    let g = state.doc("genesis");
    let mut findings = Vec::new();
    if g["schema"] != VERSION {
        findings.push(json!({"code":"genesis_missing"}));
    }
    if !ledger_matches(g) {
        findings.push(json!({"code":"genesis_ledger_mismatch"}));
    }
    for id in ITEMS {
        let entries: Vec<_> = arr(g, "defaults")
            .iter()
            .filter(|i| i["id"] == id)
            .collect();
        if entries.len() != 1
            || entries.iter().any(|i| {
                !matches!(i["state"].as_str(), Some("confirmed" | "re-ruled"))
                    || i["ruling_ref"].as_str().is_none_or(|s| !nonblank(s))
                    || i["value"].as_str().is_none_or(|s| !nonblank(s))
            })
        {
            findings.push(json!({"code":"defaults_unconfirmed","item":id}));
        }
    }
    match serde_json::from_value::<Clearance>(g["clearance"].clone()) {
        Ok(clearance) => {
            if let Err(e) = validate_clearance(state, &clearance) {
                findings.push(json!({"code":e.code,"detail":e.message}));
            }
            if clearance.verdict == Verdict::Collision {
                findings.push(json!({"code":"clearance_collision"}));
            }
            if g["adoption"]["ruling_ref"]
                .as_str()
                .is_none_or(|s| !nonblank(s))
                || g["adoption"]["clearance_hash"] != sha256_hex(&canonical(&g["clearance"]))
            {
                findings.push(json!({"code":"name_conditional"}));
            }
        }
        Err(_) => findings.push(json!({"code":"name_conditional"})),
    }
    if arr(g, "seed_manifest").len() != SEED_PATHS.len()
        || SEED_PATHS.iter().any(|path| {
            arr(g, "seed_manifest")
                .iter()
                .filter(|f| f["path"] == *path)
                .count()
                != 1
        })
        || arr(g, "seed_manifest").iter().any(|f| {
            f["content"]
                .as_str()
                .is_none_or(|s| f["sha256"] != sha256_hex(s))
        })
    {
        findings.push(json!({"code":"seed_manifest_mismatch"}));
    }
    json!({"gate":VERSION,"complete":findings.is_empty(),"findings":findings})
}

// Validate each run against its exact predecessor, not just its output hash.
fn runs_match(state: &ProjectState) -> bool {
    let g = state.doc("genesis");
    let mut prior = sheet(state);
    prior["defaults"] = json!(ITEMS.iter().map(|id| json!({"id":id,"state":"inherited","value":g["profile"]["items"][id],"ruling_ref":null})).collect::<Vec<_>>());
    prior["candidate"] = g["initial_candidate"].clone();
    prior["clearance"] = Value::Null;
    prior["adoption"] = Value::Null;
    prior["denylist"] = json!([]);
    prior["ledger"] = json!([]);
    prior["runs"] = json!([]);
    let mut previous_runs = Vec::new();
    for (index, run) in arr(g, "runs").iter().enumerate() {
        let artifact = &run["artifact"];
        let mut at_run = state.clone();
        *at_run.doc_mut("genesis") = artifact.clone();
        let verdict = if sheet_census(&at_run)["complete"] == true {
            "PASS"
        } else {
            "REVIEW"
        };
        if run["pipeline"] != "genesis"
            || run["pipeline_version"] != "1.0"
            || run["gate"] != VERSION
            || run["input_state_hash"] != sha256_hex(&canonical(&prior))
            || run["output_artifact_hashes"]["sheet"] != sha256_hex(&canonical(artifact))
            || run["verdict"] != verdict
            || run["findings"] != sheet_census(&at_run)["findings"]
            || arr(g, "ledger")
                .get(..index + 1)
                .is_none_or(|prefix| arr(artifact, "ledger") != prefix)
            || run["ruling_refs"]
                != json!(
                    arr(artifact, "ledger")
                        .iter()
                        .map(|e| e["ruling_ref"].clone())
                        .collect::<Vec<_>>()
                )
        {
            return false;
        }
        previous_runs.push(run.clone());
        prior = artifact.clone();
        prior["runs"] = json!(previous_runs);
    }
    true
}

pub fn census(state: &ProjectState) -> Value {
    let mut census = sheet_census(state);
    if !runs_match(state) {
        census["complete"] = json!(false);
        census["findings"]
            .as_array_mut()
            .expect("findings array")
            .push(json!({"code":"genesis_run_mismatch"}));
    }
    if arr(state.doc("genesis"), "runs").len() != arr(state.doc("genesis"), "ledger").len()
        || arr(state.doc("genesis"), "runs")
            .last()
            .is_none_or(|r| r["artifact"] != sheet(state))
    {
        census["complete"] = json!(false);
        census["findings"]
            .as_array_mut()
            .expect("findings array")
            .push(json!({"code":"genesis_run_missing_or_stale"}));
    }
    census
}

pub fn get_state(state: &ProjectState) -> Value {
    let sheet = sheet(state);
    let census = census(state);
    json!({"project":state.slug,"sheet":sheet,"sheet_hash":sha256_hex(&canonical(&sheet)),"census":census,"runs":state.doc("genesis")["runs"]})
}
