//! Stage 4: `keeper-chain 1.0` (ARDOR-PIPELINES.md §4, as
//! amended by Amendment A1, 2026-11-18).
//!
//! Every asset a title ships carries a keeper chain: the
//! ART-BIBLE §1.1 field set generalized to Ardor — its `game`
//! field is the project the chain lives in, and its `kind`
//! vocabulary is Ardor's shipped asset kinds. Chains live in
//! their own typed `chains` document (commission decision (a),
//! 2026-11-18), following the canon.rs pattern: structural
//! validation in this module (the tool layer's params errors),
//! semantic judgement in the gate (`decide_chain`,
//! `decide_activation`), mutation only in `apply_*`.
//!
//! A chain registers in manual-intake form: its session-record
//! field carries the typed state `absent-unverified`. Intake
//! (`apply_intake`) holds a presented session record as data —
//! state `present-unverified` — and activation runs the four
//! legs of §4's Activation against what Ardor holds:
//! (a) the record validates whole against the published
//! `torogao-scene-record/1` schema (see `scene_record`);
//! (b) the recipe's conditioning hash equals the hash Ardor
//! *derives* from the conditioning record it holds — derived,
//! never accepted as transcribed;
//! (c) the chain's final-file hash equals the registered
//! asset's hash (the asset bytes on file, as registered at
//! group-7 intake with provenance);
//! (d) the record's custody map is well-formed; custody
//! refusals Torogao has issued are recorded verbatim in the
//! chain, at activation or whenever they arrive.
//! Pass: the chain is `active`, its session-record field is
//! exactly `{state: verified, format, record_sha256}`, and a
//! §7 conformance run record exists. Any leg failing is a
//! typed refusal and the chain stays in manual-intake form.
//!
//! Commission decisions (Owner commission, 2026-11-18):
//! (a) typed `chains` document, canon.rs pattern — not fields
//! on the asset record; (b) kind vocabulary = the shipped
//! asset kinds; (c) session-record states are
//! absent-unverified → present-unverified → verified;
//! (d) full structural validation of the record spec;
//! (e) dedicated chain verbs (the registry and validator are
//! tools, per the spec's placement section).
//!
//! Honest limits, stated once: Ardor holds hashes, not bytes.
//! Canon-reference and checkpoint hashes are checked for shape
//! and presence here; their byte equality is proven by the
//! specialists that hold the bytes (the chain names them, and
//! the final-file leg is checked against the one hash Ardor
//! does hold twice — the registered asset's). Finish
//! operations are recorded as named, parameterized, ordered
//! data; replaying them is the finishing tools' act, not a
//! claim this validator makes.

use std::collections::BTreeSet;

use serde_json::{Map, Value, json};

use crate::gate::Refusal;
use crate::model::{arr, canonical, find_by_id, is_sha256_hex, is_valid_id, sha256_hex};
use crate::scene_record;
use crate::store::ProjectState;

pub const VERSION: &str = "keeper-chain 1.0";
/// Kind vocabulary: Ardor's shipped asset kinds (decision (b)).
pub const KINDS: [&str; 7] = [
    "sprite", "model3d", "audio", "haptic", "text", "build", "other",
];
pub const VERDICTS: [&str; 3] = ["candidate", "keeper", "rejected"];
pub const STATE_ABSENT: &str = "absent-unverified";
pub const STATE_PRESENT: &str = "present-unverified";
pub const STATE_VERIFIED: &str = "verified";
const MAX_CHAINS: usize = 4096;
const MAX_CANON_REFS: usize = 64;
const MAX_CONDITIONING: usize = 16;
const MAX_FINISH_OPS: usize = 256;
const MAX_CUSTODY_REFUSALS: usize = 256;
const TEXT_MAX: usize = 4000;

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

fn hash(map: &Map<String, Value>, key: &str, what: &str) -> Result<(), String> {
    match map.get(key).and_then(Value::as_str) {
        Some(s) if is_sha256_hex(s) => Ok(()),
        _ => Err(format!(
            "{what}.{key} must be a 64-digit lowercase hex SHA-256"
        )),
    }
}

// ---------- declarations (structural validation: params errors) ----------

/// Validate a keeper-chain record for registration. Structural
/// problems are the tool layer's params errors; existence and
/// hash-equality law is the gate's.
pub fn validate_chain(chain: &Value) -> Result<(), String> {
    let c = strict(
        chain,
        "chain",
        &[
            "id",
            "asset",
            "kind",
            "record_id",
            "canon_refs",
            "prompt_pos",
            "prompt_neg",
            "model",
            "conditioning",
            "seed",
            "finish_ops",
            "sha256",
            "verdict",
            "custody_refusals",
        ],
    )?;
    slug(c, "id", "chain")?;
    slug(c, "asset", "chain")?;
    let kind = c.get("kind").and_then(Value::as_str).unwrap_or("");
    if !KINDS.contains(&kind) {
        return Err(format!("chain.kind must be one of {KINDS:?}"));
    }
    match c.get("record_id") {
        None | Some(Value::Null) => {}
        Some(Value::String(s)) if is_valid_id(s) => {}
        _ => return Err("chain.record_id must be a lowercase slug or null".to_string()),
    }
    let canon_refs = c
        .get("canon_refs")
        .and_then(Value::as_array)
        .filter(|r| r.len() <= MAX_CANON_REFS)
        .ok_or_else(|| format!("chain.canon_refs must hold at most {MAX_CANON_REFS} references"))?;
    let mut ref_ids = BTreeSet::new();
    for reference in canon_refs {
        let r = strict(reference, "canon reference", &["id", "sha256"])?;
        let id = slug(r, "id", "canon reference")?;
        if !ref_ids.insert(id.clone()) {
            return Err(format!("canon reference {id} repeated"));
        }
        hash(r, "sha256", "canon reference")?;
    }
    text(c, "prompt_pos", "chain", TEXT_MAX)?;
    match c.get("prompt_neg").and_then(Value::as_str) {
        Some(s) if s.len() <= TEXT_MAX => {}
        _ => {
            return Err(format!(
                "chain.prompt_neg must be a string of at most {TEXT_MAX} bytes"
            ));
        }
    }
    let model = strict(
        c.get("model").ok_or("chain.model is required")?,
        "model",
        &["checkpoint", "sha256", "sampler", "steps", "cfg", "size"],
    )?;
    text(model, "checkpoint", "model", 1024)?;
    hash(model, "sha256", "model")?;
    text(model, "sampler", "model", 200)?;
    match model.get("steps").and_then(Value::as_u64) {
        Some(n) if (1..=1_000_000).contains(&n) => {}
        _ => return Err("model.steps must be an integer 1..=1000000".to_string()),
    }
    match model.get("cfg").and_then(Value::as_f64) {
        Some(n) if n.is_finite() && (0.0..=100.0).contains(&n) => {}
        _ => return Err("model.cfg must be a finite number 0..=100".to_string()),
    }
    text(model, "size", "model", 64)?;
    let conditioning = c
        .get("conditioning")
        .and_then(Value::as_array)
        .filter(|r| r.len() <= MAX_CONDITIONING)
        .ok_or_else(|| {
            format!("chain.conditioning must hold at most {MAX_CONDITIONING} entries")
        })?;
    for entry in conditioning {
        let e = strict(entry, "conditioning entry", &["model", "weight"])?;
        text(e, "model", "conditioning entry", 1024)?;
        match e.get("weight").and_then(Value::as_f64) {
            Some(w) if w.is_finite() && (0.0..=16.0).contains(&w) => {}
            _ => {
                return Err("conditioning entry.weight must be a finite number 0..=16".to_string());
            }
        }
    }
    if c.get("seed").and_then(Value::as_u64).is_none() {
        return Err("chain.seed must be a non-negative integer".to_string());
    }
    let finish_ops = c
        .get("finish_ops")
        .and_then(Value::as_array)
        .filter(|r| r.len() <= MAX_FINISH_OPS)
        .ok_or_else(|| format!("chain.finish_ops must hold at most {MAX_FINISH_OPS} operations"))?;
    for op in finish_ops {
        let o = strict(op, "finish op", &["name", "params", "region"])?;
        text(o, "name", "finish op", 200)?;
        match o.get("params") {
            None => {}
            Some(Value::Object(_)) => {}
            _ => return Err("finish op.params must be an object".to_string()),
        }
        match o.get("region") {
            None => {}
            Some(Value::String(s)) if s.len() <= 1024 => {}
            _ => return Err("finish op.region must be a string of at most 1024 bytes".to_string()),
        }
    }
    hash(c, "sha256", "chain")?;
    let verdict = strict(
        c.get("verdict").ok_or("chain.verdict is required")?,
        "verdict",
        &["state", "ruling_ref"],
    )?;
    let state = verdict.get("state").and_then(Value::as_str).unwrap_or("");
    if !VERDICTS.contains(&state) {
        return Err(format!("verdict.state must be one of {VERDICTS:?}"));
    }
    text(verdict, "ruling_ref", "verdict", TEXT_MAX)?;
    match c.get("custody_refusals") {
        None => {}
        Some(Value::Array(items)) if items.len() <= MAX_CUSTODY_REFUSALS => {
            for item in items {
                match item.as_str() {
                    Some(s) if nonblank(s, TEXT_MAX) => {}
                    _ => {
                        return Err(
                            "chain.custody_refusals entries must be nonblank strings".to_string()
                        );
                    }
                }
            }
        }
        _ => {
            return Err(format!(
                "chain.custody_refusals must hold at most {MAX_CUSTODY_REFUSALS} entries"
            ));
        }
    }
    Ok(())
}

// ---------- gate (semantic refusals, before mutation) ----------

fn chain_not_found(id: &str) -> Refusal {
    Refusal::new(
        "unknown_chain",
        format!("no keeper chain named '{id}' in this project"),
    )
}

pub fn find_chain<'a>(state: &'a ProjectState, id: &str) -> Option<&'a Value> {
    find_by_id(state.doc("chains"), "chains", id)
}

/// Registration judgement: the asset the chain extends must be
/// registered, of the same kind, and the record it emits from
/// must exist when one is named.
pub fn decide_chain(state: &ProjectState, chain: &Value) -> Result<(), Refusal> {
    let id = chain["id"].as_str().unwrap_or("");
    if find_chain(state, id).is_none() && arr(state.doc("chains"), "chains").len() >= MAX_CHAINS {
        return Err(Refusal::new(
            "chain_bound_exceeded",
            format!("a project holds at most {MAX_CHAINS} keeper chains"),
        ));
    }
    let asset_id = chain["asset"].as_str().unwrap_or("");
    let Some(asset) = find_by_id(state.doc("assets"), "assets", asset_id) else {
        return Err(Refusal::new(
            "unknown_asset",
            format!("keeper chain references unknown asset '{asset_id}'; register the asset first"),
        ));
    };
    if asset["kind"] != chain["kind"] {
        return Err(Refusal::new(
            "chain_kind_mismatch",
            format!(
                "chain kind '{}' does not match asset '{asset_id}' kind '{}'",
                chain["kind"].as_str().unwrap_or(""),
                asset["kind"].as_str().unwrap_or("")
            ),
        ));
    }
    if let Some(record_id) = chain.get("record_id").and_then(Value::as_str)
        && find_by_id(state.doc("cast"), "characters", record_id).is_none()
    {
        return Err(Refusal::new(
            "unknown_character",
            format!("chain record_id references unknown character '{record_id}'"),
        ));
    }
    Ok(())
}

/// The activation judgement: all four legs of §4's Activation,
/// in order, over state Ardor already holds. Pure — a refusal
/// here leaves every file byte-identical.
pub fn decide_activation(state: &ProjectState, chain_id: &str) -> Result<(), Refusal> {
    let Some(chain) = find_chain(state, chain_id) else {
        return Err(chain_not_found(chain_id));
    };
    match chain["session_record"]["state"].as_str() {
        Some(STATE_VERIFIED) => {
            return Err(Refusal::new(
                "chain_already_active",
                format!("chain '{chain_id}' is already active"),
            ));
        }
        Some(STATE_PRESENT) => {}
        _ => {
            return Err(Refusal::new(
                "chain_incomplete",
                format!("chain '{chain_id}' holds no session record; intake one before activation"),
            ));
        }
    }
    let record = &chain["held_record"];
    if !record.is_object() {
        return Err(Refusal::new(
            "chain_incomplete",
            format!("chain '{chain_id}' carries a present-unverified state but no held record"),
        ));
    }
    // Leg (a): the record validates whole against the schema.
    if let Err(detail) = scene_record::validate_record(record) {
        return Err(Refusal::new(
            "record_schema_invalid",
            format!("activation leg (a), record schema: {detail}"),
        ));
    }
    // Leg (d): the custody map, named separately per the spec.
    if let Err(detail) = scene_record::validate_custody(&record["custody"]) {
        return Err(Refusal::new(
            "record_schema_invalid",
            format!("activation leg (d), custody map: {detail}"),
        ));
    }
    // Leg (b): the conditioning hash is derived, never transcribed.
    let derived = scene_record::conditioning_sha256(&chain["held_conditioning"]);
    let presented = record["recipe"]["conditioning_sha256"]
        .as_str()
        .unwrap_or("");
    if derived != presented {
        return Err(Refusal::new(
            "chain_hash_mismatch",
            format!(
                "activation leg (b): the record presents conditioning hash {presented}, \
                 but the hash derived from the conditioning record on file is {derived}"
            ),
        ));
    }
    // Leg (c): the chain's final-file hash equals the registered
    // asset's hash — the bytes on file, as registered.
    let asset_id = chain["asset"].as_str().unwrap_or("");
    let asset = find_by_id(state.doc("assets"), "assets", asset_id);
    let asset_hash = asset.and_then(|a| a["sha256"].as_str()).unwrap_or("");
    if chain["sha256"].as_str().unwrap_or("") != asset_hash {
        return Err(Refusal::new(
            "chain_hash_mismatch",
            format!(
                "activation leg (c): chain final-file hash {} does not equal \
                 registered asset '{asset_id}' hash {asset_hash}",
                chain["sha256"].as_str().unwrap_or("")
            ),
        ));
    }
    Ok(())
}

/// Intake and custody-refusal judgement: the chain must exist.
pub fn decide_existing(state: &ProjectState, chain_id: &str) -> Result<(), Refusal> {
    if find_chain(state, chain_id).is_none() {
        return Err(chain_not_found(chain_id));
    }
    Ok(())
}

// ---------- apply (mutation, after approval) ----------

pub fn apply_chain(state: &mut ProjectState, chain: Value) {
    let id = chain["id"].as_str().unwrap_or("").to_string();
    let old = find_chain(state, &id).cloned();
    let version = old
        .as_ref()
        .and_then(|o| o["version"].as_i64())
        .unwrap_or(0)
        + 1;
    let mut record = chain;
    let sha_changed = old
        .as_ref()
        .is_some_and(|o| o["sha256"] != record["sha256"]);
    let obj = record.as_object_mut().expect("chain is an object");
    obj.entry("record_id".to_string()).or_insert(Value::Null);
    obj.entry("custody_refusals".to_string())
        .or_insert_with(|| json!([]));
    obj.insert("version".to_string(), json!(version));
    let mut reset = false;
    match &old {
        Some(old) => {
            let held = old["held_record"].clone();
            obj.insert("held_record".to_string(), held.clone());
            obj.insert(
                "held_conditioning".to_string(),
                old["held_conditioning"].clone(),
            );
            let was_active = old["state"].as_str() == Some("active");
            if was_active && sha_changed {
                // The verified final-file leg named other bytes:
                // activation falls back to manual intake, the
                // held record stays for re-activation.
                reset = true;
                obj.insert("state".to_string(), json!("manual-intake"));
                let session = if held.is_object() {
                    json!({
                        "state": STATE_PRESENT,
                        "format": scene_record::FORMAT,
                        "record_sha256": scene_record::record_sha256(&held)
                    })
                } else {
                    json!({"state": STATE_ABSENT})
                };
                obj.insert("session_record".to_string(), session);
            } else {
                obj.insert("state".to_string(), old["state"].clone());
                obj.insert("session_record".to_string(), old["session_record"].clone());
            }
        }
        None => {
            obj.insert("state".to_string(), json!("manual-intake"));
            obj.insert("session_record".to_string(), json!({"state": STATE_ABSENT}));
            obj.insert("held_record".to_string(), Value::Null);
            obj.insert("held_conditioning".to_string(), Value::Null);
        }
    }
    let doc = state.doc_mut("chains");
    match crate::model::find_by_id_mut(doc, "chains", &id) {
        Some(slot) => *slot = record,
        None => crate::model::arr_mut(doc, "chains").push(record),
    }
    if old.is_some() {
        let seq = arr(state.doc("chains"), "chain_history").len() + 1;
        crate::model::arr_mut(state.doc_mut("chains"), "chain_history").push(json!({
            "seq": seq,
            "chain": id,
            "version": version,
            "activation_reset": reset
        }));
    }
}

pub fn apply_intake(state: &mut ProjectState, chain_id: &str, record: Value, conditioning: Value) {
    let doc = state.doc_mut("chains");
    let Some(chain) = crate::model::find_by_id_mut(doc, "chains", chain_id) else {
        return;
    };
    let obj = chain.as_object_mut().expect("chain is an object");
    obj.insert(
        "session_record".to_string(),
        json!({
            "state": STATE_PRESENT,
            "format": scene_record::FORMAT,
            "record_sha256": scene_record::record_sha256(&record)
        }),
    );
    obj.insert("held_record".to_string(), record);
    obj.insert("held_conditioning".to_string(), conditioning);
    if obj["state"].as_str() == Some("active") {
        obj.insert("state".to_string(), json!("manual-intake"));
    }
}

pub fn apply_custody_refusal(state: &mut ProjectState, chain_id: &str, refusal: String) {
    let doc = state.doc_mut("chains");
    let Some(chain) = crate::model::find_by_id_mut(doc, "chains", chain_id) else {
        return;
    };
    crate::model::arr_mut(chain, "custody_refusals").push(Value::String(refusal));
}

fn input_hash(chain: &Value, record: &Value) -> String {
    sha256_hex(&canonical(&json!({"chain": chain, "record": record})))
}

/// The §7 run record for an activation of the chain as it
/// stands *before* `apply_activation` mutates it.
fn activation_run(state: &ProjectState, chain: &Value) -> Value {
    let record = &chain["held_record"];
    let record_hash = scene_record::record_sha256(record);
    json!({
        "seq": arr(state.doc("chains"), "runs").len() + 1,
        "pipeline": "keeper-chain",
        "pipeline_version": "1.0",
        "gate": VERSION,
        "kind": "activation",
        "chain": chain["id"],
        "input_state_hash": input_hash(chain, record),
        "output_artifact_hashes": {"session_record": record_hash},
        "verdict": "ACTIVATED",
        "refusal_codes": [],
        "ruling_refs": [chain["verdict"]["ruling_ref"]],
        "artifact": {
            "chain": chain["id"],
            "record_sha256": record_hash,
            "conditioning_sha256": scene_record::conditioning_sha256(&chain["held_conditioning"])
        }
    })
}

pub fn apply_activation(state: &mut ProjectState, chain_id: &str, custody_refusals: Vec<String>) {
    let chain = find_chain(state, chain_id)
        .cloned()
        .expect("gate checked the chain exists");
    let mut run = activation_run(state, &chain);
    let record_hash = scene_record::record_sha256(&chain["held_record"]);
    let doc = state.doc_mut("chains");
    let slot = crate::model::find_by_id_mut(doc, "chains", chain_id)
        .expect("gate checked the chain exists");
    for refusal in custody_refusals {
        crate::model::arr_mut(slot, "custody_refusals").push(Value::String(refusal));
    }
    let obj = slot.as_object_mut().expect("chain is an object");
    obj.insert("state".to_string(), json!("active"));
    obj.insert(
        "session_record".to_string(),
        json!({
            "state": STATE_VERIFIED,
            "format": scene_record::FORMAT,
            "record_sha256": record_hash
        }),
    );
    let after = slot.clone();
    run["output_artifact_hashes"]["chain"] = json!(sha256_hex(&canonical(&after)));
    crate::model::arr_mut(state.doc_mut("chains"), "runs").push(run);
}

// ---------- the validator (pure) ----------

/// The completeness check the gate of record names: every
/// mandatory field present (registration enforced that), the
/// final-file hash equal to the registered asset's, and the
/// verdict read for keeper state. A pure function of state.
pub fn validation(state: &ProjectState, chain: &Value) -> Value {
    let mut findings: Vec<Value> = Vec::new();
    let asset_id = chain["asset"].as_str().unwrap_or("");
    match find_by_id(state.doc("assets"), "assets", asset_id) {
        None => findings.push(json!({
            "code": "unknown_asset",
            "message": format!("chain references unknown asset '{asset_id}'")
        })),
        Some(asset) => {
            if asset["sha256"] != chain["sha256"] {
                findings.push(json!({
                    "code": "chain_hash_mismatch",
                    "message": format!(
                        "chain final-file hash {} does not equal registered asset hash {}",
                        chain["sha256"].as_str().unwrap_or(""),
                        asset["sha256"].as_str().unwrap_or("")
                    )
                }));
            }
        }
    }
    let codes: Vec<&str> = findings.iter().filter_map(|f| f["code"].as_str()).collect();
    let complete = findings.is_empty();
    json!({
        "gate": VERSION,
        "project": state.slug,
        "chain": chain["id"],
        "complete": complete,
        "keeper": complete && chain["verdict"]["state"] == "keeper",
        "verdict_state": chain["verdict"]["state"],
        "chain_state": chain["state"],
        "session_record": chain["session_record"],
        "findings": findings,
        "refusal_codes": codes,
        "verdict": if complete { "VALID" } else { "REFUSED" }
    })
}

/// A §7 run record for validating the chain as it stands.
pub fn validation_run(state: &ProjectState, chain: &Value) -> Value {
    let artifact = validation(state, chain);
    json!({
        "seq": arr(state.doc("chains"), "runs").len() + 1,
        "pipeline": "keeper-chain",
        "pipeline_version": "1.0",
        "gate": VERSION,
        "kind": "validation",
        "chain": chain["id"],
        "input_state_hash": sha256_hex(&canonical(chain)),
        "output_artifact_hashes": {"validation": sha256_hex(&canonical(&artifact))},
        "verdict": artifact["verdict"],
        "refusal_codes": artifact["refusal_codes"],
        "ruling_refs": [chain["verdict"]["ruling_ref"]],
        "artifact": artifact
    })
}

pub fn apply_validation_run(state: &mut ProjectState, chain_id: &str) {
    let chain = find_chain(state, chain_id)
        .cloned()
        .expect("gate checked the chain exists");
    let run = validation_run(state, &chain);
    crate::model::arr_mut(state.doc_mut("chains"), "runs").push(run);
}

/// Read-only view: one chain with its run records.
pub fn get_state(state: &ProjectState, chain_id: &str) -> Option<Value> {
    let chain = find_chain(state, chain_id)?;
    let runs: Vec<Value> = arr(state.doc("chains"), "runs")
        .iter()
        .filter(|r| r["chain"] == chain["id"])
        .cloned()
        .collect();
    Some(json!({
        "project": state.slug,
        "gate": VERSION,
        "chain": chain,
        "validation": validation(state, chain),
        "runs": runs
    }))
}

/// Read-only view: every chain and the run count.
pub fn list_state(state: &ProjectState) -> Value {
    json!({
        "project": state.slug,
        "gate": VERSION,
        "chains": state.doc("chains")["chains"],
        "runs_recorded": arr(state.doc("chains"), "runs").len()
    })
}
