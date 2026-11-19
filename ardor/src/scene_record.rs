//! `torogao-scene-record/1`: full structural validation of the
//! session record's JSON wire form (ARDOR-PIPELINES.md §4,
//! Amendment A1). The format is published by
//! SCENE-RECORD-SPEC.md in the torogao-lift workdir and by the
//! landed torogao-gencond code (D-838, landed on Torogao main
//! 2026-11-04 under D-927), whose serde definitions are the
//! evidence for the wire spellings enforced here — snake_case
//! enum variants, externally tagged states, unknown fields
//! refused. Where the spec document and that code disagree, the
//! code's gate tests are the evidence (the spec says so itself).
//!
//! Ardor links no Torogao code (the D-693/D-835 direction):
//! records are consumed downstream as hashed data and validated
//! against the published format, never against a Torogao build.
//! Validation is total and bounded: every rule in the published
//! spec has a check here, and a failure names the rule that
//! failed. Nothing is truncated or clamped.

use serde_json::{Map, Value};

use crate::model::{canonical, is_sha256_hex, sha256_hex};

/// The format token a verified chain's session-record field
/// carries (ARDOR-PIPELINES.md §4, Activation).
pub const FORMAT: &str = "torogao-scene-record/1";
/// The only schema version this build reads.
pub const SCHEMA_VERSION: u64 = 1;
/// The rig-model format a populated `puppet2d` block references.
pub const RIG_FORMAT: &str = "lumen-rig-model";
/// The rig-model format version this build references.
pub const RIG_FORMAT_VERSION: u64 = 1;
/// The four medium kinds, in record order; each appears once.
pub const KINDS: [&str; 4] = ["puppet2d", "armature3d", "sprite_cels", "audio_events"];

// Bounds, from the published spec §7 and the gencond limits.
const CUSTODY_ENTRIES_MAX: usize = 512;
const PROVENANCE_ENTRIES_MAX: usize = 256;
const FROZEN_TRACKS_MAX: usize = 64;
const KEYFORMS_MAX: usize = 256;
const PALETTE_SNAPS_MAX: usize = 64;
const PALETTE_PAIRS_MAX: usize = 256;
const NODE_BYTES_MAX: usize = 256;
const TEXT_BYTES_MAX: usize = 1024;
const SOURCE_ID_BYTES_MAX: usize = 64;
const MODEL_NAME_BYTES_MAX: usize = 256;

/// SHA-256 of a conditioning record's canonical JSON — the
/// derivation a session emitter uses to fill the recipe's
/// `conditioning_sha256` mechanically (spec §2). Ardor derives
/// it from the conditioning record it holds; a presented hash
/// is never accepted as transcribed.
pub fn conditioning_sha256(conditioning: &Value) -> String {
    sha256_hex(&canonical(conditioning))
}

/// SHA-256 of a session record's canonical JSON: the digest a
/// chain's session-record field holds the record by.
pub fn record_sha256(record: &Value) -> String {
    sha256_hex(&canonical(record))
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

fn strict_map<'a>(
    map: &'a Map<String, Value>,
    what: &str,
    allowed: &[&str],
) -> Result<&'a Map<String, Value>, String> {
    if let Some(key) = map.keys().find(|k| !allowed.contains(&k.as_str())) {
        return Err(format!("{what}: unknown field '{key}'"));
    }
    Ok(map)
}

fn req<'a>(map: &'a Map<String, Value>, key: &str, what: &str) -> Result<&'a Value, String> {
    map.get(key)
        .ok_or_else(|| format!("{what}.{key} is required"))
}

/// A source id: `[a-z0-9-]`, starting with a letter or digit,
/// at most 64 bytes (the gencond `check_source_id` rule).
fn is_source_id(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= SOURCE_ID_BYTES_MAX
        && s.bytes()
            .next()
            .is_some_and(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
        && s.bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
}

fn hash_field(map: &Map<String, Value>, key: &str, what: &str) -> Result<(), String> {
    match map.get(key).and_then(Value::as_str) {
        Some(s) if is_sha256_hex(s) => Ok(()),
        _ => Err(format!(
            "{what}.{key} must be a 64-digit lowercase hex SHA-256"
        )),
    }
}

fn finite(map: &Map<String, Value>, key: &str, what: &str) -> Result<f64, String> {
    match map.get(key).and_then(Value::as_f64) {
        Some(n) if n.is_finite() => Ok(n),
        _ => Err(format!("{what}.{key} must be a finite number")),
    }
}

/// One custody node path (spec §4): `/`-joined segments, each
/// `[a-z0-9-]` starting with a letter or digit, no empty
/// segments, at most 256 bytes whole.
fn check_node_id(id: &str) -> Result<(), String> {
    let valid = !id.is_empty()
        && id.len() <= NODE_BYTES_MAX
        && id.split('/').all(|segment| {
            !segment.is_empty()
                && segment
                    .bytes()
                    .next()
                    .is_some_and(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
                && segment
                    .bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        });
    if valid {
        Ok(())
    } else {
        Err(format!(
            "node id '{id}' is not a valid node path ([a-z0-9-] segments joined by '/')"
        ))
    }
}

/// The custody map's well-formedness (spec §4): bounds, node
/// discipline, one entry per node, frozen entries carrying
/// their author id. This is activation leg (d), and it also
/// runs as part of whole-record validation.
pub fn validate_custody(custody: &Value) -> Result<(), String> {
    let map = strict(custody, "custody", &["entries"])?;
    let entries = req(map, "entries", "custody")?
        .as_array()
        .ok_or_else(|| "custody.entries must be an array".to_string())?;
    if entries.len() > CUSTODY_ENTRIES_MAX {
        return Err("custody map exceeds the entry bound (512)".to_string());
    }
    let mut nodes: Vec<&str> = Vec::new();
    for entry in entries {
        let e = strict(entry, "custody entry", &["node", "state"])?;
        let node = req(e, "node", "custody entry")?
            .as_str()
            .ok_or_else(|| "custody entry.node must be a string".to_string())?;
        check_node_id(node).map_err(|e| format!("custody {e}"))?;
        if nodes.contains(&node) {
            return Err(format!("custody node '{node}' is defined twice"));
        }
        nodes.push(node);
        match req(e, "state", "custody entry")? {
            Value::String(s) if s == "open" => {}
            Value::Object(state) => {
                let frozen = strict_map(state, "custody state", &["frozen"])?;
                let f = strict(
                    req(frozen, "frozen", "custody state")?,
                    "custody frozen",
                    &["by", "reason"],
                )?;
                let by = req(f, "by", "custody frozen")?
                    .as_str()
                    .ok_or_else(|| "custody frozen.by must be a string".to_string())?;
                if !is_source_id(by) {
                    return Err(format!(
                        "custody entry '{node}' was frozen by '{by}', which is not a valid author id"
                    ));
                }
                match req(f, "reason", "custody frozen")? {
                    Value::String(r) if r.len() <= TEXT_BYTES_MAX => {}
                    _ => {
                        return Err(
                            "custody frozen.reason must be a string of at most 1024 bytes"
                                .to_string(),
                        );
                    }
                }
            }
            _ => {
                return Err(format!(
                    "custody entry '{node}': state must be \"open\" or a frozen object"
                ));
            }
        }
    }
    Ok(())
}

fn validate_recipe(recipe: &Value) -> Result<(), String> {
    let r = strict(
        recipe,
        "recipe",
        &[
            "conditioning_sha256",
            "graph_sha256",
            "seed",
            "backend",
            "execution_context",
            "rerender",
        ],
    )?;
    hash_field(r, "conditioning_sha256", "recipe")?;
    hash_field(r, "graph_sha256", "recipe")?;
    if req(r, "seed", "recipe")?.as_i64().is_none() {
        return Err("recipe.seed must be an integer".to_string());
    }
    match req(r, "backend", "recipe")?.as_str() {
        Some(b)
            if !b.is_empty()
                && b.len() <= MODEL_NAME_BYTES_MAX
                && !b.chars().any(char::is_control) => {}
        _ => {
            return Err(
                "recipe.backend must be a non-empty model name without control characters"
                    .to_string(),
            );
        }
    }
    let context = match req(r, "execution_context", "recipe")?.as_str() {
        Some(c) if c.len() <= TEXT_BYTES_MAX => c,
        _ => {
            return Err(
                "recipe.execution_context must be a string of at most 1024 bytes".to_string(),
            );
        }
    };
    // The honesty contract (spec §2): byte identity requires a
    // pinned context on record; best-effort requires its reason.
    match req(r, "rerender", "recipe")? {
        Value::String(s) if s == "pinned_context_byte_identical" => {
            if context.is_empty() {
                return Err(
                    "a byte-identical re-render claim requires the pinned execution \
                     context to be recorded; an unpinned run is best_effort with its reason"
                        .to_string(),
                );
            }
        }
        Value::Object(guarantee) => {
            let g = strict_map(guarantee, "recipe.rerender", &["best_effort"])?;
            let effort = strict(
                req(g, "best_effort", "recipe.rerender")?,
                "recipe.rerender.best_effort",
                &["reason"],
            )?;
            match req(effort, "reason", "recipe.rerender.best_effort")?.as_str() {
                Some(reason) if !reason.is_empty() && reason.len() <= TEXT_BYTES_MAX => {}
                _ => {
                    return Err(
                        "a best-effort re-render guarantee must carry its reason".to_string()
                    );
                }
            }
        }
        _ => {
            return Err(
                "recipe.rerender must be \"pinned_context_byte_identical\" or a best_effort object"
                    .to_string(),
            );
        }
    }
    Ok(())
}

fn validate_extensions(extensions: &Value) -> Result<(), String> {
    let blocks = extensions
        .as_array()
        .ok_or_else(|| "extensions must be an array".to_string())?;
    if blocks.len() != KINDS.len() {
        return Err(
            "a scene record carries exactly the four medium-extension blocks; \
             absence is ambiguous, so every block is present and states its state"
                .to_string(),
        );
    }
    let mut kinds: Vec<&str> = Vec::new();
    for block in blocks {
        let b = strict(block, "extension", &["kind", "state"])?;
        let kind = req(b, "kind", "extension")?
            .as_str()
            .ok_or_else(|| "extension.kind must be a string".to_string())?;
        if !KINDS.contains(&kind) {
            return Err(format!(
                "medium extension kind '{kind}' is not known to this build"
            ));
        }
        kinds.push(kind);
        match req(b, "state", "extension")? {
            Value::String(s) if s == "not_yet_populated" => {}
            Value::Object(state) => {
                let tagged = strict_map(state, "extension state", &["puppet2d"])?;
                if kind != "puppet2d" {
                    return Err("a puppet2d reference sits in a block of another kind".to_string());
                }
                let reference = strict(
                    req(tagged, "puppet2d", "extension state")?,
                    "puppet2d reference",
                    &["format", "format_version", "model_sha256"],
                )?;
                match req(reference, "format", "puppet2d reference")?.as_str() {
                    Some(f) if f == RIG_FORMAT => {}
                    Some(f) => {
                        return Err(format!(
                            "puppet2d block references format '{f}'; the puppet lane \
                             references {RIG_FORMAT} (D-835: lumen's format, consumed downstream)"
                        ));
                    }
                    None => {
                        return Err("puppet2d reference.format must be a string".to_string());
                    }
                }
                match req(reference, "format_version", "puppet2d reference")?.as_u64() {
                    Some(v) if v == RIG_FORMAT_VERSION => {}
                    Some(v) => {
                        return Err(format!(
                            "puppet2d block references {RIG_FORMAT} version {v}; \
                             this build references version {RIG_FORMAT_VERSION}"
                        ));
                    }
                    None => {
                        return Err(
                            "puppet2d reference.format_version must be an integer".to_string()
                        );
                    }
                }
                hash_field(reference, "model_sha256", "puppet2d reference")?;
            }
            _ => {
                return Err(format!(
                    "extension '{kind}': state must be \"not_yet_populated\" or a puppet2d reference"
                ));
            }
        }
    }
    for kind in KINDS {
        let count = kinds.iter().filter(|k| **k == kind).count();
        if count != 1 {
            return Err(format!(
                "medium extension '{kind}' appears {count} times; each kind appears exactly once"
            ));
        }
    }
    Ok(())
}

fn validate_provenance(provenance: &Value) -> Result<(), String> {
    let entries = provenance
        .as_array()
        .ok_or_else(|| "provenance must be an array".to_string())?;
    if entries.len() > PROVENANCE_ENTRIES_MAX {
        return Err("scene record exceeds the provenance bound (256)".to_string());
    }
    for entry in entries {
        let e = strict(
            entry,
            "provenance entry",
            &[
                "op",
                "source_sha256",
                "frozen_sha256",
                "epsilon",
                "epsilon_origin",
                "tolerance",
            ],
        )?;
        let op = req(e, "op", "provenance entry")?
            .as_str()
            .ok_or_else(|| "provenance entry.op must be a string".to_string())?;
        if !["freeze_track", "freeze_palette"].contains(&op) {
            return Err(format!("freeze operation '{op}' is not known"));
        }
        hash_field(e, "source_sha256", "provenance entry")?;
        hash_field(e, "frozen_sha256", "provenance entry")?;
        let present = |key: &str| e.get(key).is_some_and(|v| !v.is_null());
        // Exactly one contract-value family per op (spec §5).
        match op {
            "freeze_track" => {
                match e.get("epsilon") {
                    Some(Value::Number(n))
                        if n.as_f64().is_some_and(|v| v.is_finite() && v >= 0.0) => {}
                    _ => {
                        return Err(
                            "a track freeze's provenance carries its epsilon (finite, non-negative)"
                                .to_string(),
                        );
                    }
                }
                match e.get("epsilon_origin").and_then(Value::as_str) {
                    Some(o) if ["caller", "track_noise"].contains(&o) => {}
                    _ => {
                        return Err("a track freeze's provenance carries its epsilon's origin \
                             (caller or track_noise)"
                            .to_string());
                    }
                }
                if present("tolerance") {
                    return Err("a track freeze's provenance carries no tolerance".to_string());
                }
            }
            _ => {
                match e.get("tolerance").and_then(Value::as_u64) {
                    Some(t) if t <= 255 => {}
                    _ => {
                        return Err(
                            "a palette freeze's provenance carries its tolerance (0..=255)"
                                .to_string(),
                        );
                    }
                }
                if present("epsilon") || present("epsilon_origin") {
                    return Err("a palette freeze's provenance carries no epsilon".to_string());
                }
            }
        }
    }
    Ok(())
}

fn validate_frozen_tracks(tracks: &Value) -> Result<(), String> {
    let items = tracks
        .as_array()
        .ok_or_else(|| "frozen_tracks must be an array".to_string())?;
    if items.len() > FROZEN_TRACKS_MAX {
        return Err("scene record exceeds the frozen-track bound (64)".to_string());
    }
    let mut parameters: Vec<&str> = Vec::new();
    for track in items {
        let t = strict(
            track,
            "frozen track",
            &["parameter", "keyforms", "max_deviation"],
        )?;
        let parameter = req(t, "parameter", "frozen track")?
            .as_str()
            .ok_or_else(|| "frozen track.parameter must be a string".to_string())?;
        if !is_source_id(parameter) {
            return Err(format!(
                "frozen track parameter '{parameter}' is not a valid id"
            ));
        }
        if parameters.contains(&parameter) {
            return Err(format!(
                "parameter '{parameter}' is frozen twice in one scene"
            ));
        }
        parameters.push(parameter);
        let keyforms = req(t, "keyforms", "frozen track")?
            .as_array()
            .ok_or_else(|| "frozen track.keyforms must be an array".to_string())?;
        if keyforms.len() < 2 {
            return Err("a frozen track needs at least two keyforms".to_string());
        }
        if keyforms.len() > KEYFORMS_MAX {
            return Err("frozen track exceeds the keyform bound (256)".to_string());
        }
        let mut previous: Option<f64> = None;
        for key in keyforms {
            let k = strict(
                key,
                "keyform",
                &["time", "value", "tangent_in", "tangent_out"],
            )?;
            let time = finite(k, "time", "keyform")?;
            finite(k, "value", "keyform")?;
            finite(k, "tangent_in", "keyform")?;
            finite(k, "tangent_out", "keyform")?;
            if let Some(before) = previous
                && time <= before
            {
                return Err("keyform times must be strictly increasing".to_string());
            }
            previous = Some(time);
        }
        match finite(t, "max_deviation", "frozen track") {
            Ok(d) if d >= 0.0 => {}
            _ => {
                return Err(
                    "a frozen track's max_deviation must be finite and non-negative".to_string(),
                );
            }
        }
    }
    Ok(())
}

/// A `#rrggbb` colour in canonical (lowercase) form — the only
/// form a stored snap carries (the gencond canonical-colour
/// rule: either case parses, the canonical form is what lands).
fn is_canonical_colour(s: &str) -> bool {
    let digits = s.strip_prefix('#').unwrap_or("");
    digits.len() == 6
        && s.len() == 7
        && digits
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

fn validate_palette_snaps(snaps: &Value) -> Result<(), String> {
    let items = snaps
        .as_array()
        .ok_or_else(|| "palette_snaps must be an array".to_string())?;
    if items.len() > PALETTE_SNAPS_MAX {
        return Err("scene record exceeds the palette-snap bound (64)".to_string());
    }
    let mut palettes: Vec<&str> = Vec::new();
    for snap in items {
        let s = strict(
            snap,
            "palette snap",
            &["palette_sha256", "tolerance", "pairs"],
        )?;
        hash_field(s, "palette_sha256", "palette snap")?;
        let palette = s["palette_sha256"].as_str().unwrap_or("");
        if palettes.contains(&palette) {
            return Err(
                "one palette is snapped twice in one scene; a re-snap replaces, it does not stack"
                    .to_string(),
            );
        }
        palettes.push(palette);
        match req(s, "tolerance", "palette snap")?.as_u64() {
            Some(t) if t <= 255 => {}
            _ => return Err("palette tolerance must lie in [0, 255]".to_string()),
        }
        let pairs = req(s, "pairs", "palette snap")?
            .as_array()
            .ok_or_else(|| "palette snap.pairs must be an array".to_string())?;
        if pairs.len() > PALETTE_PAIRS_MAX {
            return Err("palette snap exceeds the colour bound (256)".to_string());
        }
        let mut sources: Vec<&str> = Vec::new();
        for pair in pairs {
            let p = strict(pair, "palette pair", &["source", "snapped"])?;
            for key in ["source", "snapped"] {
                match req(p, key, "palette pair")?.as_str() {
                    Some(c) if is_canonical_colour(c) => {}
                    _ => {
                        return Err(
                            "palette snap colours must be in canonical #rrggbb form".to_string()
                        );
                    }
                }
            }
            let source = p["source"].as_str().unwrap_or("");
            if sources.contains(&source) {
                return Err(format!("palette snap carries colour '{source}' twice"));
            }
            sources.push(source);
        }
    }
    Ok(())
}

/// Validate a whole session record against the published
/// schema: version, id, recipe honesty, the four-block
/// completeness law, custody, and every frozen artifact it
/// carries. This is activation leg (a).
pub fn validate_record(record: &Value) -> Result<(), String> {
    let r = strict(
        record,
        "scene record",
        &[
            "schema",
            "id",
            "recipe",
            "extensions",
            "custody",
            "provenance",
            "frozen_tracks",
            "palette_snaps",
        ],
    )?;
    match req(r, "schema", "scene record")?.as_u64() {
        Some(v) if v == SCHEMA_VERSION => {}
        Some(v) => {
            return Err(format!(
                "scene record schema {v} is not supported (this build reads {SCHEMA_VERSION})"
            ));
        }
        None => return Err("scene record.schema must be an integer".to_string()),
    }
    match req(r, "id", "scene record")?.as_str() {
        Some(id) if is_source_id(id) => {}
        _ => {
            return Err(
                "scene record.id must be a valid id ([a-z0-9-], bounded length)".to_string(),
            );
        }
    }
    validate_recipe(req(r, "recipe", "scene record")?)?;
    validate_extensions(req(r, "extensions", "scene record")?)?;
    validate_custody(req(r, "custody", "scene record")?)?;
    validate_provenance(req(r, "provenance", "scene record")?)?;
    validate_frozen_tracks(req(r, "frozen_tracks", "scene record")?)?;
    validate_palette_snaps(req(r, "palette_snaps", "scene record")?)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn valid_record() -> Value {
        json!({
            "schema": 1,
            "id": "scene-one",
            "recipe": {
                "conditioning_sha256": "a".repeat(64),
                "graph_sha256": "b".repeat(64),
                "seed": 7,
                "backend": "seam-c",
                "execution_context": "",
                "rerender": {"best_effort": {"reason": "backend context unpinned"}}
            },
            "extensions": [
                {"kind": "puppet2d", "state": {"puppet2d": {"format": "lumen-rig-model", "format_version": 1, "model_sha256": "c".repeat(64)}}},
                {"kind": "armature3d", "state": "not_yet_populated"},
                {"kind": "sprite_cels", "state": "not_yet_populated"},
                {"kind": "audio_events", "state": "not_yet_populated"}
            ],
            "custody": {"entries": [{"node": "hero/hand", "state": {"frozen": {"by": "owner", "reason": "hand edit"}}}]},
            "provenance": [{"op": "freeze_palette", "source_sha256": "d".repeat(64), "frozen_sha256": "e".repeat(64), "tolerance": 0}],
            "frozen_tracks": [],
            "palette_snaps": []
        })
    }

    #[test]
    fn valid_record_passes() {
        validate_record(&valid_record()).expect("valid record");
    }

    #[test]
    fn duplicated_medium_block_refused() {
        let mut record = valid_record();
        record["extensions"][1] = json!({"kind": "puppet2d", "state": "not_yet_populated"});
        let err = validate_record(&record).unwrap_err();
        assert!(err.contains("appears 2 times"), "{err}");
    }

    #[test]
    fn unpinned_byte_identity_refused() {
        let mut record = valid_record();
        record["recipe"]["rerender"] = json!("pinned_context_byte_identical");
        let err = validate_record(&record).unwrap_err();
        assert!(err.contains("byte-identical"), "{err}");
    }

    #[test]
    fn malformed_custody_refused() {
        let mut record = valid_record();
        record["custody"]["entries"][0]["node"] = json!("Hero Hand");
        let err = validate_custody(&record["custody"]).unwrap_err();
        assert!(err.contains("not a valid node path"), "{err}");
    }

    #[test]
    fn conditioning_hash_is_derived_from_canonical_json() {
        let conditioning = json!({"b": 1, "a": {"y": [1, 2], "x": "s"}});
        assert_eq!(
            conditioning_sha256(&conditioning),
            sha256_hex("{\"a\":{\"x\":\"s\",\"y\":[1,2]},\"b\":1}")
        );
    }
}
