//! REQ-9 structural projection (D-235): the allowlist filter that
//! sits between an ardor read response and any cross-lane consumer
//! (Seahawk's `canon.context` route, SPEC
//! `seahawk-wiring-20261028` §4). This module is the ardor-side
//! reference implementation: pure, inert, and never invoked by the
//! server's tool dispatch — the route that consumes it is
//! default-off and lives on the consumer side. Building it here
//! keeps the allowlist, the drop counting, and the lane labelling
//! under this crate's own test gate, next to the data it filters.
//!
//! Terms (SPEC §4, D-235): read-only, allowlisted fields only,
//! drops counted, lane-labelled. Only structural scalars cross:
//! identifiers, counts, verdict booleans, and safety-surface
//! presence booleans. Canon *content* — dialogue text, reaction
//! profiles, canon fact bodies, character descriptions, scene and
//! system payloads, and every other free-text field — is dropped,
//! always. Project names are the one free-text exception the SPEC
//! names explicitly (project id *and name* may cross).
//!
//! The projection weakens nothing: it reads a response value and
//! returns a smaller value. It never touches the store, the
//! `decide()` gate, or the U5 safety invariants — safety surfaces
//! cross as presence booleans only, never as configuration.
//!
//! Counting rule: one drop per non-allowlisted field occurrence
//! (a field dropped from each of N array records counts N drops),
//! so the count a consumer logs
//! (`ardor_projected_fields_dropped: n`) reflects how much of the
//! payload the boundary held back.

use serde_json::{Map, Value, json};

use crate::gate::SAFETY_SURFACES;

/// The content label every projected result carries (SPEC §4).
pub const CONTENT_LABEL: &str = "sfw-structural";
/// The origin lane every projected result is labelled with:
/// NTRanch-lane origin is explicit, never laundered (BLOCH §9).
pub const ORIGIN_LANE: &str = "ntranch";

/// One projection outcome: the allowlisted structural subset,
/// wrapped with its lane labels, plus the count of dropped fields.
pub struct Projection {
    pub value: Value,
    pub dropped: usize,
}

/// Accumulates drops while a projection is built.
struct Drops(usize);

impl Drops {
    /// Count every key of `object` except `kept` as dropped.
    fn count_rest(&mut self, object: &Map<String, Value>, kept: &[&str]) {
        for key in object.keys() {
            if !kept.contains(&key.as_str()) {
                self.0 += 1;
            }
        }
    }
}

/// Copy one allowlisted string field; a present field of the
/// wrong type is dropped and counted (never coerced).
fn keep_str(out: &mut Map<String, Value>, obj: &Map<String, Value>, key: &str, drops: &mut Drops) {
    match obj.get(key) {
        Some(Value::String(s)) => {
            out.insert(key.to_string(), Value::String(s.clone()));
        }
        Some(_) => drops.0 += 1,
        None => {}
    }
}

/// Copy one allowlisted boolean field, same discipline as
/// [`keep_str`].
fn keep_bool(out: &mut Map<String, Value>, obj: &Map<String, Value>, key: &str, drops: &mut Drops) {
    match obj.get(key) {
        Some(Value::Bool(b)) => {
            out.insert(key.to_string(), Value::Bool(*b));
        }
        Some(_) => drops.0 += 1,
        None => {}
    }
}

/// Project a record array to its `id` fields (plus any extra
/// allowlisted string fields named in `extra_strs`): every other
/// field of every record is dropped and counted.
fn project_ids(items: &[Value], extra_strs: &[&str], drops: &mut Drops) -> Vec<Value> {
    let mut out = Vec::new();
    for item in items {
        let Some(obj) = item.as_object() else {
            drops.0 += 1;
            continue;
        };
        let mut record = Map::new();
        keep_str(&mut record, obj, "id", drops);
        for key in extra_strs {
            keep_str(&mut record, obj, key, drops);
        }
        let mut kept: Vec<&str> = vec!["id"];
        kept.extend_from_slice(extra_strs);
        drops.count_rest(obj, &kept);
        out.push(Value::Object(record));
    }
    out
}

fn project_get_project(payload: &Map<String, Value>, drops: &mut Drops) -> Value {
    let mut data = Map::new();
    keep_str(&mut data, payload, "project", drops);
    // meta: the project name alone crosses; engine, rating,
    // age-gate text, and tags are canon-side detail.
    if let Some(meta) = payload.get("meta").and_then(Value::as_object) {
        let mut meta_out = Map::new();
        keep_str(&mut meta_out, meta, "name", drops);
        drops.count_rest(meta, &["name"]);
        data.insert("meta".to_string(), Value::Object(meta_out));
    } else if payload.contains_key("meta") {
        drops.0 += 1;
    }
    // counts: every value is a structural integer by construction.
    if let Some(counts) = payload.get("counts").and_then(Value::as_object) {
        let mut counts_out = Map::new();
        for (key, value) in counts {
            if value.is_u64() {
                counts_out.insert(key.clone(), value.clone());
            } else {
                drops.0 += 1;
            }
        }
        data.insert("counts".to_string(), Value::Object(counts_out));
    } else if payload.contains_key("counts") {
        drops.0 += 1;
    }
    keep_bool(&mut data, payload, "safety_audit_ok", drops);
    drops.count_rest(payload, &["project", "meta", "counts", "safety_audit_ok"]);
    Value::Object(data)
}

fn project_list_projects(payload: &Map<String, Value>, drops: &mut Drops) -> Value {
    let mut data = Map::new();
    if let Some(items) = payload.get("projects").and_then(Value::as_array) {
        let mut out = Vec::new();
        for item in items {
            let Some(obj) = item.as_object() else {
                drops.0 += 1;
                continue;
            };
            // Ardor calls the identifier `slug`; the projection
            // renames it to the SPEC's `id`.
            let mut record = Map::new();
            if let Some(slug) = obj.get("slug").and_then(Value::as_str) {
                record.insert("id".to_string(), Value::String(slug.to_string()));
            } else if obj.contains_key("slug") {
                drops.0 += 1;
            }
            keep_str(&mut record, obj, "name", drops);
            drops.count_rest(obj, &["slug", "name"]);
            out.push(Value::Object(record));
        }
        let count = out.len();
        data.insert("projects".to_string(), Value::Array(out));
        data.insert("count".to_string(), json!(count));
    } else if payload.contains_key("projects") {
        drops.0 += 1;
    }
    drops.count_rest(payload, &["projects", "count"]);
    Value::Object(data)
}

fn project_check_canon(payload: &Map<String, Value>, drops: &mut Drops) -> Value {
    let mut data = Map::new();
    keep_str(&mut data, payload, "op", drops);
    // The verdict crosses as a pass/fail boolean; the refusal
    // code and message are canon-side text and never cross.
    match payload.get("verdict").and_then(Value::as_str) {
        Some(verdict) => {
            data.insert("canon_check_pass".to_string(), json!(verdict == "allowed"));
        }
        None => {
            if payload.contains_key("verdict") {
                drops.0 += 1;
            }
        }
    }
    drops.count_rest(payload, &["op", "verdict"]);
    Value::Object(data)
}

fn project_safety(payload: &Map<String, Value>, drops: &mut Drops) -> Value {
    let mut data = Map::new();
    keep_str(&mut data, payload, "project", drops);
    let mut presence = Map::new();
    for name in SAFETY_SURFACES {
        presence.insert(name.to_string(), Value::Bool(false));
    }
    // get_safety shape: surfaces are records keyed by `id`.
    if let Some(surfaces) = payload.get("surfaces").and_then(Value::as_array) {
        for surface in surfaces {
            if let Some(id) = surface.get("id").and_then(Value::as_str)
                && presence.contains_key(id)
            {
                presence.insert(id.to_string(), Value::Bool(true));
            }
            if let Some(obj) = surface.as_object() {
                // Configuration never crosses; count every field
                // beyond the identifier that established presence.
                drops.count_rest(obj, &["id"]);
            } else {
                drops.0 += 1;
            }
        }
    } else if payload.contains_key("surfaces") {
        drops.0 += 1;
    }
    // audit_safety shape: presence is read off the audit's
    // per-surface checks (a missing surface reports ok=false
    // with a "surface missing" detail — presence means a surface
    // record exists, which the get_safety leg establishes; the
    // audit leg contributes the overall verdict boolean only).
    if let Some(audit) = payload.get("audit").and_then(Value::as_object) {
        keep_bool(&mut data, audit, "ok", drops);
        drops.count_rest(audit, &["ok"]);
        if let Some(checks) = audit.get("checks").and_then(Value::as_array) {
            drops.0 += checks.len();
        }
    } else if payload.contains_key("audit") {
        drops.0 += 1;
    }
    data.insert(
        "safety_surfaces_present".to_string(),
        Value::Object(presence),
    );
    drops.count_rest(payload, &["project", "surfaces", "audit"]);
    Value::Object(data)
}

fn project_defects(payload: &Map<String, Value>, drops: &mut Drops) -> Value {
    let mut data = Map::new();
    keep_str(&mut data, payload, "project", drops);
    if let Some(items) = payload.get("defects").and_then(Value::as_array) {
        // SPEC: defect counts and statuses cross; titles and
        // bodies do not. Ids ride along as identifiers.
        let projected = project_ids(items, &["status"], drops);
        let count = projected.len();
        data.insert("defects".to_string(), Value::Array(projected));
        data.insert("count".to_string(), json!(count));
    } else if payload.contains_key("defects") {
        drops.0 += 1;
    }
    drops.count_rest(payload, &["project", "defects", "count"]);
    Value::Object(data)
}

/// Project one ardor read-tool payload to its REQ-9 structural
/// subset. Deny-by-default: a tool with no named projection (any
/// write tool, any unknown name) projects to an empty structural
/// object with every top-level field counted as dropped — a
/// consumer that asks for a projection it should not have gets
/// labels and a drop count, never content.
pub fn project(tool: &str, payload: &Value) -> Projection {
    let mut drops = Drops(0);
    let Some(obj) = payload.as_object() else {
        return Projection {
            value: envelope(tool, Value::Object(Map::new())),
            dropped: 1,
        };
    };
    let data = match tool {
        "get_project" => project_get_project(obj, &mut drops),
        "list_projects" => project_list_projects(obj, &mut drops),
        "check_canon" => project_check_canon(obj, &mut drops),
        "list_characters" | "get_character" => {
            let mut data = Map::new();
            keep_str(&mut data, obj, "project", &mut drops);
            for key in ["characters", "character"] {
                match obj.get(key) {
                    Some(Value::Array(items)) => {
                        let projected = project_ids(items, &[], &mut drops);
                        data.insert("characters".to_string(), Value::Array(projected));
                    }
                    Some(single @ Value::Object(_)) => {
                        let projected = project_ids(std::slice::from_ref(single), &[], &mut drops);
                        data.insert("characters".to_string(), Value::Array(projected));
                    }
                    Some(_) => drops.0 += 1,
                    None => {}
                }
            }
            // Cast history is canon content; it never crosses.
            drops.count_rest(obj, &["project", "characters", "character"]);
            Value::Object(data)
        }
        "list_canon" => {
            let mut data = Map::new();
            keep_str(&mut data, obj, "project", &mut drops);
            if let Some(items) = obj.get("rules").and_then(Value::as_array) {
                // Rule identifiers and kinds cross; fact bodies
                // (field/value/fields payloads) never do.
                let projected = project_ids(items, &["kind"], &mut drops);
                data.insert("rules".to_string(), Value::Array(projected));
            } else if obj.contains_key("rules") {
                drops.0 += 1;
            }
            drops.count_rest(obj, &["project", "rules"]);
            Value::Object(data)
        }
        "get_safety" | "audit_safety" => project_safety(obj, &mut drops),
        "list_defects" => project_defects(obj, &mut drops),
        "add_screenshot_spec" => {
            // There is no list tool for screenshot specs; the QA
            // document reaches a consumer through this write's
            // echo, so its projection is named explicitly: spec
            // ids cross, node bindings and framing text do not.
            let mut data = Map::new();
            keep_str(&mut data, obj, "project", &mut drops);
            if let Some(items) = obj.get("screenshot_specs").and_then(Value::as_array) {
                let projected = project_ids(items, &[], &mut drops);
                data.insert("screenshot_specs".to_string(), Value::Array(projected));
            } else if obj.contains_key("screenshot_specs") {
                drops.0 += 1;
            }
            drops.count_rest(obj, &["project", "screenshot_specs"]);
            Value::Object(data)
        }
        _ => {
            drops.0 += obj.len();
            Value::Object(Map::new())
        }
    };
    Projection {
        value: envelope(tool, data),
        dropped: drops.0,
    }
}

fn envelope(tool: &str, data: Value) -> Value {
    json!({
        "content_label": CONTENT_LABEL,
        "origin_lane": ORIGIN_LANE,
        "tool": tool,
        "data": data,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const MARKER: &str = "SEEDED-CANON-MARKER";

    fn assert_no_marker(p: &Projection) {
        assert!(
            !p.value.to_string().contains(MARKER),
            "projected value must never carry canon content: {}",
            p.value
        );
    }

    // ----- validation: allowlisted structure crosses intact -----

    #[test]
    fn project_keeps_ids_counts_and_verdicts() {
        let payload = json!({
            "project": "evidence-game",
            "meta": {"name": "Evidence Game", "engine": "bevy", "rating": "adult"},
            "counts": {"characters": 2, "canon_rules": 3},
            "safety_audit_ok": true
        });
        let p = project("get_project", &payload);
        assert_eq!(p.value["content_label"], "sfw-structural");
        assert_eq!(p.value["origin_lane"], "ntranch");
        assert_eq!(p.value["data"]["project"], "evidence-game");
        assert_eq!(p.value["data"]["meta"]["name"], "Evidence Game");
        assert_eq!(p.value["data"]["counts"]["characters"], 2);
        assert_eq!(p.value["data"]["safety_audit_ok"], true);
        // engine + rating were held back.
        assert_eq!(p.dropped, 2);
    }

    #[test]
    fn canon_verdict_crosses_as_boolean() {
        let allowed = project(
            "check_canon",
            &json!({"verdict": "allowed", "op": "upsert_character"}),
        );
        assert_eq!(allowed.value["data"]["canon_check_pass"], true);
        assert_eq!(allowed.dropped, 0);
        let refused = project(
            "check_canon",
            &json!({"verdict": "refused", "op": "upsert_level", "code": "canon_lock", "message": MARKER}),
        );
        assert_eq!(refused.value["data"]["canon_check_pass"], false);
        assert_eq!(refused.dropped, 2);
        assert_no_marker(&refused);
    }

    #[test]
    fn safety_crosses_as_presence_only() {
        let payload = json!({
            "project": "evidence-game",
            "rating": "adult",
            "age_gate": MARKER,
            "surfaces": [
                {"id": "stop", "enabled": true, "config": {"word": MARKER}},
                {"id": "safeword", "enabled": true, "config": {}}
            ],
            "audit": {"ok": false, "checks": [{"check": "safety_surface:stop", "ok": true, "detail": MARKER}]}
        });
        let p = project("get_safety", &payload);
        let presence = &p.value["data"]["safety_surfaces_present"];
        assert_eq!(presence["stop"], true);
        assert_eq!(presence["safeword"], true);
        assert_eq!(presence["purge"], false);
        assert_eq!(p.value["data"]["ok"], false);
        assert!(p.dropped > 0);
        assert_no_marker(&p);
    }

    // ----- adversarial: canon content never crosses -----

    #[test]
    fn character_descriptions_and_history_never_cross() {
        let payload = json!({
            "project": "evidence-game",
            "characters": [
                {"id": "opal", "name": MARKER, "description": MARKER, "appearance": MARKER}
            ],
            "history": [{"note": MARKER}]
        });
        let p = project("list_characters", &payload);
        assert_eq!(p.value["data"]["characters"], json!([{"id": "opal"}]));
        // name, description, appearance per record + history.
        assert_eq!(p.dropped, 4);
        assert_no_marker(&p);
    }

    #[test]
    fn canon_fact_bodies_never_cross() {
        let payload = json!({
            "project": "evidence-game",
            "rules": [
                {"id": "lock-1", "kind": "lock", "fields": ["hair"]},
                {"id": "fact-1", "kind": "fact", "field": "hair", "value": MARKER}
            ]
        });
        let p = project("list_canon", &payload);
        assert_eq!(
            p.value["data"]["rules"],
            json!([{"id": "lock-1", "kind": "lock"}, {"id": "fact-1", "kind": "fact"}])
        );
        assert_eq!(p.dropped, 3);
        assert_no_marker(&p);
    }

    #[test]
    fn defect_titles_never_cross() {
        let payload = json!({
            "project": "evidence-game",
            "defects": [{"id": "d-1", "status": "open", "title": MARKER, "body": MARKER}],
            "count": 1
        });
        let p = project("list_defects", &payload);
        assert_eq!(
            p.value["data"]["defects"],
            json!([{"id": "d-1", "status": "open"}])
        );
        assert_eq!(p.dropped, 2);
        assert_no_marker(&p);
    }

    #[test]
    fn unlisted_tools_project_to_nothing() {
        // A dialogue write payload — the class of content the
        // boundary exists to hold back — yields labels and a
        // drop count only.
        let payload = json!({"project": "evidence-game", "node": {"id": "n1", "text": MARKER}});
        let p = project("upsert_dialogue", &payload);
        assert_eq!(p.value["data"], json!({}));
        assert_eq!(p.dropped, 2);
        assert_no_marker(&p);
        let unknown = project("no_such_tool", &payload);
        assert_eq!(unknown.value["data"], json!({}));
        assert_no_marker(&unknown);
    }

    #[test]
    fn non_object_payloads_project_to_nothing() {
        let p = project("get_project", &json!([MARKER]));
        assert_eq!(p.value["data"], json!({}));
        assert_eq!(p.dropped, 1);
        assert_no_marker(&p);
    }
}
