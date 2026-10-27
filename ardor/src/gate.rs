//! The decide() gate: every canonical write passes through here
//! BEFORE any state changes. `decide` is pure — it reads the
//! current state and the proposed operation and returns either an
//! approval or a typed refusal. `apply` runs only after approval,
//! so a refused operation leaves every state file byte-identical
//! (the test suite proves this by hashing the tree around refused
//! writes).
//!
//! Two families of checks, in order:
//! 1. **Safety (hard law, no override exists):** the five safety
//!    surfaces (stop, safeword, accessibility, aftercare, purge)
//!    must exist, stay enabled, and never be progression-gated —
//!    directly, or indirectly by any payload that pairs a safety
//!    surface with `progression_gated: true`. Adult dialogue with
//!    actions requires recorded affirmative consent covering every
//!    participant. Release handoffs require a passing safety audit
//!    and a verified channel.
//! 2. **Canon:** locked character fields and canon facts cannot be
//!    contradicted by a write unless the write carries an explicit
//!    amendment with a reason; amendments are recorded in the cast
//!    history, so canon changes are versioned, never silent.
//!
//! Release handoffs additionally respect the registry's section
//! discipline (D-128 annex): services-section channels take
//! commissions, not releases (`not_a_release_channel`), and
//! SFW-only asset channels never take an adult project's release
//! (`channel_sfw_only`).
//!
//! Structural problems (missing fields, wrong types, bad enums)
//! are the tool layer's `-32602` params errors, raised before an
//! operation is even built; everything here is a semantic refusal
//! returned as a typed domain error.

use serde_json::{Value, json};

use crate::model::{arr, find_by_id, find_by_id_mut, is_sha256_hex};
use crate::store::ProjectState;

/// The five safety surfaces every project carries from scaffold.
/// None of them may ever be disabled or progression-gated.
pub const SAFETY_SURFACES: [&str; 5] = ["accessibility", "aftercare", "purge", "safeword", "stop"];

/// The reserved branch target meaning "the graph ends here".
pub const TERMINAL: &str = "END";

/// A typed gate refusal: a stable code plus a human message.
#[derive(Debug)]
pub struct Refusal {
    pub code: &'static str,
    pub message: String,
}

impl Refusal {
    pub fn new(code: &'static str, message: impl Into<String>) -> Refusal {
        Refusal {
            code,
            message: message.into(),
        }
    }
}

/// One proposed canonical write. Built by the tool layer after
/// params validation; judged by [`decide`]; executed by [`apply`].
#[derive(Debug)]
pub enum Op {
    AddCanonRule {
        rule: Value,
    },
    AddScreenshotSpec {
        spec: Value,
    },
    CreateProject {
        meta: Value,
    },
    DeclareFlag {
        flag: Value,
    },
    PrepareHandoff {
        channel: String,
        artifacts: Vec<Value>,
    },
    RecordAssetHandoff {
        asset_id: String,
        target: String,
    },
    RecordDefect {
        defect: Value,
    },
    RecordPlaytestRun {
        run: Value,
    },
    RegisterAsset {
        asset: Value,
    },
    SetReactionProfile {
        character_id: String,
        profile: Value,
    },
    SetSafetySurface {
        surface: String,
        config: Value,
    },
    UpdateProjectMeta {
        patch: Value,
    },
    UpsertArc {
        arc: Value,
    },
    UpsertCharacter {
        character: Value,
        amendment: Option<Value>,
    },
    UpsertDialogue {
        node: Value,
    },
    UpsertLevel {
        level: Value,
    },
    UpsertScene {
        node: Value,
    },
    UpsertSystem {
        system: Value,
    },
}

/// Which documents an approved operation changes (for the store's
/// atomic save). Alphabetical within each op.
fn changed_docs(op: &Op) -> Vec<&'static str> {
    match op {
        Op::AddCanonRule { .. } => vec!["canon"],
        Op::AddScreenshotSpec { .. } | Op::RecordDefect { .. } | Op::RecordPlaytestRun { .. } => {
            vec!["qa"]
        }
        Op::CreateProject { .. } => vec![
            "assets", "canon", "cast", "graph", "levels", "project", "qa", "releases", "safety",
            "story", "systems",
        ],
        Op::DeclareFlag { .. } | Op::UpsertDialogue { .. } | Op::UpsertScene { .. } => {
            vec!["graph"]
        }
        Op::PrepareHandoff { .. } => vec!["releases"],
        Op::RecordAssetHandoff { .. } | Op::RegisterAsset { .. } => vec!["assets"],
        Op::SetReactionProfile { .. } | Op::UpsertCharacter { .. } => vec!["cast"],
        Op::SetSafetySurface { .. } => vec!["safety"],
        Op::UpdateProjectMeta { .. } => vec!["project"],
        Op::UpsertArc { .. } => vec!["story"],
        Op::UpsertLevel { .. } => vec!["levels"],
        Op::UpsertSystem { .. } => vec!["systems"],
    }
}

// ---------- shared lookups ----------

fn character_exists(state: &ProjectState, id: &str) -> bool {
    find_by_id(state.doc("cast"), "characters", id).is_some()
}

fn node_exists(state: &ProjectState, id: &str) -> bool {
    id == TERMINAL || find_by_id(state.doc("graph"), "nodes", id).is_some()
}

fn flag_declared(state: &ProjectState, name: &str) -> bool {
    arr(state.doc("graph"), "flags")
        .iter()
        .any(|f| f.get("name").and_then(Value::as_str) == Some(name))
}

fn str_field<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str)
}

/// Recursive scan: any object that pairs a reference to a safety
/// surface (under `surface`, `surface_id`, `target`, or `gates`)
/// with `progression_gated: true` is an attempt to gate a safety
/// surface, wherever it hides in a payload.
fn scan_safety_gating(value: &Value) -> Option<String> {
    match value {
        Value::Object(map) => {
            if map.get("progression_gated").and_then(Value::as_bool) == Some(true) {
                for key in ["gates", "surface", "surface_id", "target"] {
                    if let Some(name) = map.get(key).and_then(Value::as_str)
                        && SAFETY_SURFACES.contains(&name)
                    {
                        return Some(name.to_string());
                    }
                }
            }
            map.values().find_map(scan_safety_gating)
        }
        Value::Array(items) => items.iter().find_map(scan_safety_gating),
        _ => None,
    }
}

fn refuse_safety_gating(surface: &str) -> Refusal {
    Refusal::new(
        "safety_progression_gate_forbidden",
        format!(
            "safety surface '{surface}' may never be progression-gated; \
             stop/safeword, accessibility, aftercare and purge stay available from the first minute"
        ),
    )
}

/// Every prerequisite named anywhere in a payload must be a
/// declared flag.
fn check_prerequisites(state: &ProjectState, value: &Value) -> Result<(), Refusal> {
    fn walk(state: &ProjectState, value: &Value) -> Result<(), Refusal> {
        match value {
            Value::Object(map) => {
                if let Some(prereqs) = map.get("prerequisites").and_then(Value::as_array) {
                    // Entries are bare flag names or objects
                    // carrying {"flag": name, ...}.
                    for entry in prereqs {
                        let Some(flag) = entry.as_str().or_else(|| str_field(entry, "flag")) else {
                            continue;
                        };
                        if !flag_declared(state, flag) {
                            return Err(Refusal::new(
                                "undeclared_flag",
                                format!(
                                    "prerequisite flag '{flag}' is not declared in this project"
                                ),
                            ));
                        }
                    }
                }
                for v in map.values() {
                    walk(state, v)?;
                }
                Ok(())
            }
            Value::Array(items) => {
                for v in items {
                    walk(state, v)?;
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }
    walk(state, value)
}

// ---------- the safety audit ----------

/// The audit every release handoff must pass: all five safety
/// surfaces present, enabled, and ungated; an adult-rated project
/// carries an age-gate pattern. Returned as data so tools can show
/// exactly which check failed.
pub fn safety_audit(state: &ProjectState) -> Value {
    let mut checks = Vec::new();
    let surfaces = arr(state.doc("safety"), "surfaces");
    for name in SAFETY_SURFACES {
        let found = surfaces.iter().find(|s| str_field(s, "id") == Some(name));
        let (ok, detail) = match found {
            None => (false, "surface missing".to_string()),
            Some(s) => {
                let enabled = s.get("enabled").and_then(Value::as_bool) == Some(true);
                let gated = s.get("progression_gated").and_then(Value::as_bool) == Some(true);
                if !enabled {
                    (false, "surface disabled".to_string())
                } else if gated {
                    (false, "surface progression-gated".to_string())
                } else {
                    (true, "present, enabled, ungated".to_string())
                }
            }
        };
        checks.push(json!({"check": format!("safety_surface:{name}"), "ok": ok, "detail": detail}));
    }
    let rating = str_field(state.meta(), "rating").unwrap_or("adult");
    if rating == "adult" {
        let age_gate = str_field(state.meta(), "age_gate").unwrap_or("");
        checks.push(json!({
            "check": "age_gate_present",
            "ok": !age_gate.is_empty(),
            "detail": if age_gate.is_empty() { "adult-rated project has no age-gate pattern" } else { "age-gate pattern recorded" }
        }));
    }
    let ok = checks.iter().all(|c| c["ok"].as_bool() == Some(true));
    json!({"ok": ok, "checks": checks})
}

// ---------- decide ----------

/// Judge one operation against the current state. Pure: no
/// mutation, no I/O. `state` is the project the op targets; for
/// `CreateProject` it is the blank state and the caller has
/// already established the slug is free. `accounts` is the
/// studio-level channel-account document (group 10), consulted by
/// the release-handoff check.
pub fn decide(state: &ProjectState, accounts: &Value, op: &Op) -> Result<(), Refusal> {
    match op {
        Op::CreateProject { meta } => {
            if str_field(meta, "rating") == Some("adult")
                && str_field(meta, "age_gate").unwrap_or("").is_empty()
            {
                return Err(Refusal::new(
                    "age_gate_required",
                    "an adult-rated project must declare an age-gate pattern at scaffold time",
                ));
            }
            Ok(())
        }
        Op::UpdateProjectMeta { patch } => {
            let rating = str_field(patch, "rating")
                .or_else(|| str_field(state.meta(), "rating"))
                .unwrap_or("adult");
            let age_gate = str_field(patch, "age_gate")
                .or_else(|| str_field(state.meta(), "age_gate"))
                .unwrap_or("");
            if rating == "adult" && age_gate.is_empty() {
                return Err(Refusal::new(
                    "age_gate_required",
                    "an adult-rated project must keep an age-gate pattern",
                ));
            }
            if let Some(start) = str_field(patch, "start_node")
                && !node_exists(state, start)
            {
                return Err(Refusal::new(
                    "unknown_node",
                    format!("start_node '{start}' is not a node in this project's graph"),
                ));
            }
            Ok(())
        }
        Op::UpsertCharacter {
            character,
            amendment,
        } => check_character_canon(state, character, amendment.as_ref()),
        Op::AddCanonRule { rule } => {
            let kind = str_field(rule, "kind").unwrap_or("");
            if kind == "lock" || kind == "fact" {
                let character = str_field(rule, "character").unwrap_or("");
                if !character_exists(state, character) {
                    return Err(Refusal::new(
                        "unknown_character",
                        format!("canon rule references unknown character '{character}'"),
                    ));
                }
            }
            Ok(())
        }
        Op::DeclareFlag { .. } => Ok(()),
        Op::UpsertScene { node } | Op::UpsertDialogue { node } => {
            check_node(state, node, matches!(op, Op::UpsertDialogue { .. }))
        }
        Op::UpsertSystem { system } => {
            let id = str_field(system, "id").unwrap_or("");
            if SAFETY_SURFACES.contains(&id) {
                return Err(Refusal::new(
                    "safety_surface_id_reserved",
                    format!(
                        "'{id}' is a safety surface; author it through the safety tools, never as a system"
                    ),
                ));
            }
            if let Some(surface) = scan_safety_gating(system) {
                return Err(refuse_safety_gating(&surface));
            }
            check_prerequisites(state, system)
        }
        Op::SetReactionProfile {
            character_id,
            profile,
        } => {
            if !character_exists(state, character_id) {
                return Err(Refusal::new(
                    "unknown_character",
                    format!("reaction profile for unknown character '{character_id}'"),
                ));
            }
            for key in ["base_receptivity", "conditioning", "sensitivity", "stamina"] {
                if let Some(v) = profile.get(key).and_then(Value::as_i64)
                    && !(0..=100).contains(&v)
                {
                    return Err(Refusal::new(
                        "profile_value_out_of_range",
                        format!("reaction profile field '{key}' must be within 0..=100, got {v}"),
                    ));
                }
            }
            Ok(())
        }
        Op::UpsertArc { .. } => Ok(()),
        Op::UpsertLevel { level } => {
            if let Some(surface) = scan_safety_gating(level) {
                return Err(refuse_safety_gating(&surface));
            }
            for gate in arr(level, "progression_gates") {
                if let Some(target) = str_field(gate, "gates")
                    && SAFETY_SURFACES.contains(&target)
                {
                    return Err(refuse_safety_gating(target));
                }
                if let Some(flag) = str_field(gate, "condition_flag")
                    && !flag_declared(state, flag)
                {
                    return Err(Refusal::new(
                        "undeclared_flag",
                        format!("progression gate condition flag '{flag}' is not declared"),
                    ));
                }
            }
            for placement in arr(level, "minigame_placements") {
                let system_id = str_field(placement, "system").unwrap_or("");
                let is_minigame = find_by_id(state.doc("systems"), "systems", system_id)
                    .is_some_and(|s| str_field(s, "kind") == Some("minigame"));
                if !is_minigame {
                    return Err(Refusal::new(
                        "unknown_minigame",
                        format!(
                            "minigame placement references '{system_id}', which is not a declared minigame system"
                        ),
                    ));
                }
            }
            for trigger in arr(level, "triggers") {
                let node = str_field(trigger, "node").unwrap_or("");
                if !node_exists(state, node) {
                    return Err(Refusal::new(
                        "unknown_node",
                        format!("level trigger references unknown node '{node}'"),
                    ));
                }
            }
            Ok(())
        }
        Op::SetSafetySurface { surface, config } => {
            if config.get("progression_gated").and_then(Value::as_bool) == Some(true) {
                return Err(refuse_safety_gating(surface));
            }
            if config.get("enabled").and_then(Value::as_bool) == Some(false) {
                return Err(Refusal::new(
                    "safety_surface_mandatory",
                    format!("safety surface '{surface}' is mandatory and cannot be disabled"),
                ));
            }
            Ok(())
        }
        Op::RegisterAsset { asset } => {
            let hash = str_field(asset, "sha256").unwrap_or("");
            if !is_sha256_hex(hash) {
                return Err(Refusal::new(
                    "invalid_content_hash",
                    "asset sha256 must be a 64-character lowercase hex digest",
                ));
            }
            let source = asset
                .get("provenance")
                .and_then(|p| str_field(p, "source"))
                .unwrap_or("");
            if source.is_empty() {
                return Err(Refusal::new(
                    "provenance_required",
                    "an asset record must name its provenance source; unprovenanced assets do not enter a project",
                ));
            }
            Ok(())
        }
        Op::RecordAssetHandoff { asset_id, .. } => {
            if find_by_id(state.doc("assets"), "assets", asset_id).is_none() {
                return Err(Refusal::new(
                    "unknown_asset",
                    format!("asset handoff references unknown asset '{asset_id}'"),
                ));
            }
            Ok(())
        }
        Op::AddScreenshotSpec { spec } => {
            let node = str_field(spec, "node").unwrap_or("");
            if !node_exists(state, node) {
                return Err(Refusal::new(
                    "unknown_node",
                    format!("screenshot spec references unknown node '{node}'"),
                ));
            }
            Ok(())
        }
        Op::RecordDefect { .. } => Ok(()),
        Op::RecordPlaytestRun { .. } => {
            if str_field(state.meta(), "start_node").is_none() {
                return Err(Refusal::new(
                    "no_start_node",
                    "the project has no start node; author a scene and set start_node first",
                ));
            }
            Ok(())
        }
        Op::PrepareHandoff { channel, artifacts } => {
            let Some(target) = crate::channels::find_channel(channel) else {
                return Err(Refusal::new(
                    "unknown_channel",
                    format!("'{channel}' is not in the channel-target registry"),
                ));
            };
            if target.verification_status != "verified" {
                return Err(Refusal::new(
                    "channel_flagged",
                    format!(
                        "channel '{}' is flagged ({}): {}",
                        target.id, target.verification_status, target.verification_note
                    ),
                ));
            }
            // Section discipline (D-128 annex): services channels
            // take commissions and contracts, not releases — a
            // release handoff to one is a category error. SFW-only
            // asset channels live in a separate registry section
            // and never take an adult project's release.
            if target.section == "services" || target.section == "services_sfw" {
                return Err(Refusal::new(
                    "not_a_release_channel",
                    format!(
                        "channel '{}' is a services channel (commissions/contracts), not a release target; \
                         it is tracked for commissioning and hiring, and Ardor prepares no release handoff for it",
                        target.id
                    ),
                ));
            }
            if target.section == "assets_sfw"
                && str_field(state.meta(), "rating").unwrap_or("adult") == "adult"
            {
                return Err(Refusal::new(
                    "channel_sfw_only",
                    format!(
                        "channel '{}' is SFW-only and this project is adult-rated; \
                         SFW-only channels live in a separate registry section and never take adult releases",
                        target.id
                    ),
                ));
            }
            if artifacts.is_empty() {
                return Err(Refusal::new(
                    "handoff_needs_artifacts",
                    "a release handoff must name at least one artifact (name, sha256, kind)",
                ));
            }
            for artifact in artifacts {
                let hash = str_field(artifact, "sha256").unwrap_or("");
                if !is_sha256_hex(hash) {
                    return Err(Refusal::new(
                        "invalid_content_hash",
                        "every handoff artifact must carry its SHA-256 digest",
                    ));
                }
            }
            // Group 10 (D-127): no handoff without the channel's
            // auth on record and verified — references and status,
            // never values.
            if crate::accounts::verified_account_for(accounts, channel).is_none() {
                return Err(Refusal::new(
                    "channel_auth_unverified",
                    format!(
                        "no verified channel account for '{channel}': register the credential \
                         reference (group 10) and record a successful verification before \
                         preparing a handoff — run publish_preflight to see the full checklist"
                    ),
                ));
            }
            let audit = safety_audit(state);
            if audit["ok"].as_bool() != Some(true) {
                let failed: Vec<String> = arr(&audit, "checks")
                    .iter()
                    .filter(|c| c["ok"].as_bool() != Some(true))
                    .map(|c| c["check"].as_str().unwrap_or("?").to_string())
                    .collect();
                return Err(Refusal::new(
                    "handoff_blocked_safety",
                    format!(
                        "safety audit fails ({}) — fix the safety surfaces before preparing a release handoff",
                        failed.join(", ")
                    ),
                ));
            }
            Ok(())
        }
    }
}

/// Canon checks for a character insert/update: canon facts must
/// hold, locked fields must not move — unless the write carries
/// an amendment with a reason, which `apply` records in history.
fn check_character_canon(
    state: &ProjectState,
    character: &Value,
    amendment: Option<&Value>,
) -> Result<(), Refusal> {
    let id = str_field(character, "id").unwrap_or("");
    let existing = find_by_id(state.doc("cast"), "characters", id);
    let amended = amendment
        .and_then(|a| str_field(a, "reason"))
        .is_some_and(|r| !r.trim().is_empty());
    for rule in arr(state.doc("canon"), "rules") {
        if str_field(rule, "character") != Some(id) {
            continue;
        }
        match str_field(rule, "kind") {
            Some("fact") => {
                let field = str_field(rule, "field").unwrap_or("");
                if let (Some(proposed), Some(fact)) = (character.get(field), rule.get("value"))
                    && proposed != fact
                    && !amended
                {
                    return Err(Refusal::new(
                        "canon_contradiction",
                        format!(
                            "canon fact: {id}.{field} is {fact}; the write proposes {proposed}. \
                             Resubmit with an amendment (reason required) to change canon deliberately."
                        ),
                    ));
                }
            }
            Some("lock") => {
                if let Some(old) = existing {
                    for field in arr(rule, "fields").iter().filter_map(Value::as_str) {
                        if character.get(field) != old.get(field) && !amended {
                            return Err(Refusal::new(
                                "canon_locked_field",
                                format!(
                                    "field '{field}' of character '{id}' is canon-locked. \
                                     Resubmit with an amendment (reason required) to change it deliberately."
                                ),
                            ));
                        }
                    }
                }
            }
            _ => {}
        }
    }
    Ok(())
}

/// Graph node checks, shared by scenes and dialogue; dialogue adds
/// the consent and action rules.
fn check_node(state: &ProjectState, node: &Value, dialogue: bool) -> Result<(), Refusal> {
    if let Some(surface) = scan_safety_gating(node) {
        return Err(refuse_safety_gating(&surface));
    }
    check_prerequisites(state, node)?;
    // Participants must be cast members. A participant entry is
    // either a bare character id or an object carrying
    // {"character": id, "role": ...} — both shapes are accepted.
    let mut participants: Vec<String> = Vec::new();
    for p in arr(node, "participants") {
        let character = p
            .as_str()
            .or_else(|| str_field(p, "character"))
            .unwrap_or("");
        if !character_exists(state, character) {
            return Err(Refusal::new(
                "unknown_character",
                format!("node references unknown character '{character}'"),
            ));
        }
        participants.push(character.to_string());
    }
    if let Some(next) = str_field(node, "next")
        && !node_exists(state, next)
    {
        return Err(Refusal::new(
            "unknown_branch_target",
            format!("node 'next' points at unknown node '{next}' (or use \"{TERMINAL}\")"),
        ));
    }
    if !dialogue {
        return Ok(());
    }
    // --- adult dialogue as structured data ---
    let actions = arr(node, "actions");
    for action in actions {
        let intensity = action.get("intensity").and_then(Value::as_i64);
        match intensity {
            Some(v) if (0..=10).contains(&v) => {}
            Some(v) => {
                return Err(Refusal::new(
                    "intensity_out_of_range",
                    format!("action intensity must be within 0..=10, got {v}"),
                ));
            }
            None => {
                return Err(Refusal::new(
                    "intensity_out_of_range",
                    "every action must carry an integer intensity within 0..=10",
                ));
            }
        }
        for role in ["actor", "target"] {
            let who = str_field(action, role).unwrap_or("");
            if !participants.iter().any(|p| p == who) {
                return Err(Refusal::new(
                    "action_participant_missing",
                    format!(
                        "action {role} '{who}' is not a declared participant of this dialogue node"
                    ),
                ));
            }
        }
    }
    if !actions.is_empty() {
        let Some(consent) = node.get("consent").filter(|c| c.is_object()) else {
            return Err(Refusal::new(
                "consent_state_required",
                "a dialogue node with actions must record its consent state",
            ));
        };
        if str_field(consent, "state") != Some("affirmative") {
            return Err(Refusal::new(
                "consent_not_affirmative",
                "actions require consent.state \"affirmative\" recorded on the node",
            ));
        }
        let acknowledged: Vec<&str> = arr(consent, "participants")
            .iter()
            .filter_map(Value::as_str)
            .collect();
        for action in actions {
            for role in ["actor", "target"] {
                let who = str_field(action, role).unwrap_or("");
                if !acknowledged.contains(&who) {
                    return Err(Refusal::new(
                        "consent_coverage_incomplete",
                        format!("consent.participants does not cover action {role} '{who}'"),
                    ));
                }
            }
        }
    }
    for choice in arr(node, "choices") {
        let next = str_field(choice, "next").unwrap_or("");
        if !node_exists(state, next) {
            return Err(Refusal::new(
                "unknown_branch_target",
                format!("choice points at unknown node '{next}' (or use \"{TERMINAL}\")"),
            ));
        }
    }
    Ok(())
}

// ---------- apply ----------

/// Execute an approved operation, mutating the state. Returns the
/// documents to persist. Callers MUST have run [`decide`] first.
pub fn apply(state: &mut ProjectState, op: Op) -> Vec<&'static str> {
    let docs = changed_docs(&op);
    match op {
        Op::CreateProject { meta } => {
            let mut project = meta;
            project
                .as_object_mut()
                .expect("meta is an object")
                .insert("slug".to_string(), json!(state.slug));
            *state.doc_mut("project") = project;
            let surfaces: Vec<Value> = SAFETY_SURFACES
                .iter()
                .map(|id| {
                    json!({
                        "id": id,
                        "enabled": true,
                        "progression_gated": false,
                        "config": {}
                    })
                })
                .collect();
            *state.doc_mut("safety") = json!({"surfaces": surfaces});
        }
        Op::UpdateProjectMeta { patch } => {
            let meta = state.doc_mut("project").as_object_mut().expect("object");
            if let Some(patch_obj) = patch.as_object() {
                for (key, value) in patch_obj {
                    meta.insert(key.clone(), value.clone());
                }
            }
        }
        Op::UpsertCharacter {
            character,
            amendment,
        } => {
            let id = str_field(&character, "id").unwrap_or("").to_string();
            let cast = state.doc_mut("cast");
            let existing = find_by_id(cast, "characters", &id).cloned();
            let mut record = character;
            let (version, changed_fields) = match &existing {
                Some(old) => {
                    let old_version = old.get("version").and_then(Value::as_i64).unwrap_or(1);
                    let mut changed = Vec::new();
                    if let Some(obj) = record.as_object() {
                        for (key, value) in obj {
                            if old.get(key) != Some(value) {
                                changed.push(key.clone());
                            }
                        }
                    }
                    (old_version + 1, changed)
                }
                None => (1, Vec::new()),
            };
            record
                .as_object_mut()
                .expect("character is an object")
                .insert("version".to_string(), json!(version));
            match find_by_id_mut(cast, "characters", &id) {
                Some(slot) => *slot = record,
                None => crate::model::arr_mut(cast, "characters").push(record),
            }
            if existing.is_some() {
                let seq = arr(cast, "history").len() + 1;
                let reason = amendment
                    .as_ref()
                    .and_then(|a| str_field(a, "reason"))
                    .map(str::to_string);
                crate::model::arr_mut(cast, "history").push(json!({
                    "seq": seq,
                    "character": id,
                    "version": version,
                    "amended": reason.is_some(),
                    "reason": reason,
                    "changed_fields": changed_fields
                }));
            }
        }
        Op::AddCanonRule { rule } => {
            crate::model::arr_mut(state.doc_mut("canon"), "rules").push(rule);
        }
        Op::DeclareFlag { flag } => {
            let name = str_field(&flag, "name").unwrap_or("").to_string();
            let graph = state.doc_mut("graph");
            match arr_mut_find_flag(graph, &name) {
                Some(slot) => *slot = flag,
                None => crate::model::arr_mut(graph, "flags").push(flag),
            }
        }
        Op::UpsertScene { node } | Op::UpsertDialogue { node } => {
            let id = str_field(&node, "id").unwrap_or("").to_string();
            let graph = state.doc_mut("graph");
            match crate::model::find_by_id_mut(graph, "nodes", &id) {
                Some(slot) => *slot = node,
                None => crate::model::arr_mut(graph, "nodes").push(node),
            }
        }
        Op::UpsertSystem { system } => {
            let id = str_field(&system, "id").unwrap_or("").to_string();
            let doc = state.doc_mut("systems");
            match crate::model::find_by_id_mut(doc, "systems", &id) {
                Some(slot) => *slot = system,
                None => crate::model::arr_mut(doc, "systems").push(system),
            }
        }
        Op::SetReactionProfile {
            character_id,
            profile,
        } => {
            let cast = state.doc_mut("cast");
            if let Some(character) = crate::model::find_by_id_mut(cast, "characters", &character_id)
            {
                character
                    .as_object_mut()
                    .expect("character is an object")
                    .insert("reaction_profile".to_string(), profile);
            }
        }
        Op::UpsertArc { arc } => {
            let id = str_field(&arc, "id").unwrap_or("").to_string();
            let doc = state.doc_mut("story");
            match crate::model::find_by_id_mut(doc, "arcs", &id) {
                Some(slot) => *slot = arc,
                None => crate::model::arr_mut(doc, "arcs").push(arc),
            }
        }
        Op::UpsertLevel { level } => {
            let id = str_field(&level, "id").unwrap_or("").to_string();
            let doc = state.doc_mut("levels");
            match crate::model::find_by_id_mut(doc, "levels", &id) {
                Some(slot) => *slot = level,
                None => crate::model::arr_mut(doc, "levels").push(level),
            }
        }
        Op::SetSafetySurface { surface, config } => {
            let doc = state.doc_mut("safety");
            if let Some(slot) = crate::model::arr_mut(doc, "surfaces")
                .iter_mut()
                .find(|s| str_field(s, "id") == Some(surface.as_str()))
            {
                let obj = slot.as_object_mut().expect("surface is an object");
                if let Some(cfg) = config.as_object() {
                    for (key, value) in cfg {
                        if key != "enabled" && key != "progression_gated" {
                            obj.insert(key.clone(), value.clone());
                        }
                    }
                }
                // Managed fields are invariants, reasserted here as
                // well as at the gate: always enabled, never gated.
                obj.insert("enabled".to_string(), json!(true));
                obj.insert("progression_gated".to_string(), json!(false));
            }
        }
        Op::RegisterAsset { asset } => {
            let id = str_field(&asset, "id").unwrap_or("").to_string();
            let doc = state.doc_mut("assets");
            match crate::model::find_by_id_mut(doc, "assets", &id) {
                Some(slot) => *slot = asset,
                None => crate::model::arr_mut(doc, "assets").push(asset),
            }
        }
        Op::RecordAssetHandoff { asset_id, target } => {
            let doc = state.doc_mut("assets");
            let seq = arr(doc, "handoffs").len() + 1;
            crate::model::arr_mut(doc, "handoffs").push(json!({
                "seq": seq,
                "asset": asset_id,
                "target": target,
                "status": "recorded"
            }));
        }
        Op::AddScreenshotSpec { spec } => {
            crate::model::arr_mut(state.doc_mut("qa"), "screenshot_specs").push(spec);
        }
        Op::RecordDefect { defect } => {
            let doc = state.doc_mut("qa");
            let seq = arr(doc, "defects").len() + 1;
            let mut defect = defect;
            defect
                .as_object_mut()
                .expect("defect is an object")
                .insert("seq".to_string(), json!(seq));
            crate::model::arr_mut(doc, "defects").push(defect);
        }
        Op::RecordPlaytestRun { run } => {
            crate::model::arr_mut(state.doc_mut("qa"), "runs").push(run);
        }
        Op::PrepareHandoff { channel, artifacts } => {
            let target = crate::channels::find_channel(&channel).expect("gate checked channel");
            let doc = state.doc_mut("releases");
            let seq = arr(doc, "handoffs").len() + 1;
            crate::model::arr_mut(doc, "handoffs").push(json!({
                "seq": seq,
                "id": format!("handoff-{seq}"),
                "channel": target.id,
                "channel_name": target.name,
                "section": target.section,
                "executor": target.handoff_executor,
                "automation_class": target.automation_class,
                "artifacts": artifacts,
                "status": "prepared",
                "note": "prepared by Ardor; execution belongs to the named executor and the publish skills' gates"
            }));
        }
    }
    docs
}

fn arr_mut_find_flag<'a>(graph: &'a mut Value, name: &str) -> Option<&'a mut Value> {
    crate::model::arr_mut(graph, "flags")
        .iter_mut()
        .find(|f| str_field(f, "name") == Some(name))
}
