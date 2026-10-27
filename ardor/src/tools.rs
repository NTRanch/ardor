//! Tool handlers: argument validation (params failures, the
//! fleet's `-32602` shape) on the outside, the decide() gate
//! (domain failures as typed `isError` payloads) on the inside.
//! Read tools never touch the gate; every write builds an
//! [`Op`], runs `decide`, then `apply`, then the store's atomic
//! save — in that order, always.

use serde_json::{Map, Value, json};

use crate::accounts::{self, AccountOp};
use crate::channels;
use crate::gate::{self, Op, Refusal};
use crate::model::{arr, find_by_id, is_valid_id};
use crate::playtest;
use crate::store::{ProjectState, Store, StoreError};

/// A tool failure: params failures become protocol errors;
/// domain failures become `isError` content with a stable code.
pub struct ToolFailure {
    pub is_params: bool,
    pub code: String,
    pub message: String,
}

impl ToolFailure {
    pub fn params(message: impl Into<String>) -> ToolFailure {
        ToolFailure {
            is_params: true,
            code: "invalid_params".to_string(),
            message: message.into(),
        }
    }

    pub fn domain(code: impl Into<String>, message: impl Into<String>) -> ToolFailure {
        ToolFailure {
            is_params: false,
            code: code.into(),
            message: message.into(),
        }
    }
}

impl From<Refusal> for ToolFailure {
    fn from(r: Refusal) -> ToolFailure {
        ToolFailure::domain(r.code, r.message)
    }
}

impl From<StoreError> for ToolFailure {
    fn from(e: StoreError) -> ToolFailure {
        ToolFailure::domain(e.code, e.message)
    }
}

type R<T> = Result<T, ToolFailure>;

// ---------- argument helpers ----------

fn strict_object(args: &Value, allowed: &[&str]) -> R<Map<String, Value>> {
    let Some(object) = args.as_object() else {
        return Err(ToolFailure::params("arguments must be a JSON object"));
    };
    for key in object.keys() {
        if !allowed.contains(&key.as_str()) {
            return Err(ToolFailure::params(format!("unknown argument: {key}")));
        }
    }
    Ok(object.clone())
}

fn req_str(map: &Map<String, Value>, key: &str) -> R<String> {
    match map.get(key).and_then(Value::as_str) {
        Some(s) if !s.is_empty() => Ok(s.to_string()),
        _ => Err(ToolFailure::params(format!(
            "missing or empty string argument: {key}"
        ))),
    }
}

fn opt_str(map: &Map<String, Value>, key: &str) -> Option<String> {
    map.get(key).and_then(Value::as_str).map(str::to_string)
}

fn req_id(map: &Map<String, Value>, key: &str) -> R<String> {
    let value = req_str(map, key)?;
    if !is_valid_id(&value) {
        return Err(ToolFailure::params(format!(
            "argument {key} must be a lowercase slug (a-z, 0-9, '_' or '-', max 64 chars): {value}"
        )));
    }
    Ok(value)
}

fn req_enum(map: &Map<String, Value>, key: &str, allowed: &[&str]) -> R<String> {
    let value = req_str(map, key)?;
    if allowed.contains(&value.as_str()) {
        Ok(value)
    } else {
        Err(ToolFailure::params(format!(
            "argument {key} must be one of {}: {value}",
            allowed.join(", ")
        )))
    }
}

fn req_obj(map: &Map<String, Value>, key: &str) -> R<Value> {
    match map.get(key) {
        Some(v) if v.is_object() => Ok(v.clone()),
        _ => Err(ToolFailure::params(format!(
            "missing or non-object argument: {key}"
        ))),
    }
}

fn opt_obj(map: &Map<String, Value>, key: &str) -> Option<Value> {
    map.get(key).filter(|v| v.is_object()).cloned()
}

fn req_payload_id(payload: &Value, what: &str) -> R<String> {
    match payload.get("id").and_then(Value::as_str) {
        Some(id) if is_valid_id(id) => Ok(id.to_string()),
        _ => Err(ToolFailure::params(format!(
            "{what} requires an 'id' that is a lowercase slug"
        ))),
    }
}

fn load_project(store: &Store, map: &Map<String, Value>) -> R<(String, ProjectState)> {
    let slug = req_id(map, "project")?;
    let state = store.load(&slug)?;
    Ok((slug, state))
}

/// The write path, in order: load accounts (the handoff gate
/// consults them), decide, apply, save. A refusal returns before
/// any mutation or save, leaving state byte-identical.
fn gated_write(store: &Store, state: &mut ProjectState, op: Op) -> R<()> {
    let accounts = store.load_accounts()?;
    gate::decide(state, &accounts, &op)?;
    let docs = gate::apply(state, op);
    store.save(state, &docs)?;
    Ok(())
}

fn gated_account_write(store: &Store, op: AccountOp) -> R<Value> {
    let mut accounts = store.load_accounts()?;
    accounts::decide_account(&accounts, &op)?;
    let stored = accounts::apply_account(&mut accounts, op);
    store.save_accounts(&accounts)?;
    Ok(stored)
}

// ---------- group 1: project scaffolding ----------

fn create_project(store: &mut Store, args: &Value) -> R<Value> {
    let map = strict_object(
        args,
        &[
            "name",
            "slug",
            "engine",
            "rating",
            "age_gate",
            "content_tags",
        ],
    )?;
    let name = req_str(&map, "name")?;
    let slug = match opt_str(&map, "slug") {
        Some(s) => {
            if !is_valid_id(&s) {
                return Err(ToolFailure::params(
                    "slug must be a lowercase slug (a-z, 0-9, '_' or '-')",
                ));
            }
            s
        }
        None => slugify(&name),
    };
    if slug.is_empty() || !is_valid_id(&slug) {
        return Err(ToolFailure::params(
            "could not derive a valid slug from the name; pass slug explicitly",
        ));
    }
    if store.project_exists(&slug) {
        return Err(ToolFailure::domain(
            "project_exists",
            format!("a project named {slug} already exists"),
        ));
    }
    let rating = match opt_str(&map, "rating") {
        Some(r) if r == "adult" || r == "mature" || r == "all_ages" => r,
        Some(r) => {
            return Err(ToolFailure::params(format!(
                "rating must be adult, mature or all_ages: {r}"
            )));
        }
        None => "adult".to_string(),
    };
    let meta = json!({
        "name": name,
        "engine": opt_str(&map, "engine").unwrap_or_else(|| "undecided".to_string()),
        "rating": rating,
        "age_gate": opt_str(&map, "age_gate").unwrap_or_default(),
        "content_tags": map.get("content_tags").cloned().unwrap_or_else(|| json!([])),
        "start_node": null
    });
    let mut state = Store::blank_state(&slug);
    gated_write(store, &mut state, Op::CreateProject { meta })?;
    Ok(json!({
        "project": slug,
        "created": true,
        "meta": state.meta(),
        "safety_surfaces": gate::SAFETY_SURFACES,
        "note": "safety surfaces were scaffolded enabled and ungated; they cannot be disabled or progression-gated"
    }))
}

fn slugify(name: &str) -> String {
    let mut out = String::new();
    for c in name.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        } else if !out.is_empty() && !out.ends_with('-') {
            out.push('-');
        }
    }
    out.trim_matches('-').chars().take(64).collect()
}

fn get_project(store: &mut Store, args: &Value) -> R<Value> {
    let map = strict_object(args, &["project"])?;
    let (slug, state) = load_project(store, &map)?;
    let count = |doc: &str, key: &str| arr(state.doc(doc), key).len();
    Ok(json!({
        "project": slug,
        "meta": state.meta(),
        "counts": {
            "characters": count("cast", "characters"),
            "canon_rules": count("canon", "rules"),
            "flags": count("graph", "flags"),
            "nodes": count("graph", "nodes"),
            "systems": count("systems", "systems"),
            "arcs": count("story", "arcs"),
            "levels": count("levels", "levels"),
            "assets": count("assets", "assets"),
            "playtest_runs": count("qa", "runs"),
            "defects": count("qa", "defects"),
            "handoffs": count("releases", "handoffs")
        },
        "safety_audit_ok": gate::safety_audit(&state)["ok"]
    }))
}

fn list_projects(store: &mut Store, args: &Value) -> R<Value> {
    let _ = strict_object(args, &[])?;
    let mut projects = Vec::new();
    for slug in store.list_projects() {
        if let Ok(state) = store.load(&slug) {
            projects.push(json!({
                "slug": slug,
                "name": state.meta().get("name"),
                "engine": state.meta().get("engine"),
                "rating": state.meta().get("rating")
            }));
        }
    }
    Ok(json!({"projects": projects, "count": projects.len()}))
}

fn update_project_meta(store: &mut Store, args: &Value) -> R<Value> {
    let map = strict_object(
        args,
        &[
            "project",
            "name",
            "engine",
            "rating",
            "age_gate",
            "content_tags",
            "start_node",
        ],
    )?;
    let (_slug, mut state) = load_project(store, &map)?;
    if let Some(r) = opt_str(&map, "rating")
        && !["adult", "mature", "all_ages"].contains(&r.as_str())
    {
        return Err(ToolFailure::params(format!(
            "rating must be adult, mature or all_ages: {r}"
        )));
    }
    let mut patch = Map::new();
    for key in [
        "name",
        "engine",
        "rating",
        "age_gate",
        "content_tags",
        "start_node",
    ] {
        if let Some(v) = map.get(key) {
            patch.insert(key.to_string(), v.clone());
        }
    }
    if patch.is_empty() {
        return Err(ToolFailure::params("nothing to update"));
    }
    gated_write(
        store,
        &mut state,
        Op::UpdateProjectMeta {
            patch: Value::Object(patch),
        },
    )?;
    Ok(json!({"project": state.slug, "meta": state.meta()}))
}

// ---------- group 2: cast and canon ----------

fn upsert_character(store: &mut Store, args: &Value) -> R<Value> {
    let map = strict_object(args, &["project", "character", "amendment"])?;
    let (_slug, mut state) = load_project(store, &map)?;
    let character = req_obj(&map, "character")?;
    req_payload_id(&character, "character")?;
    if character.get("name").and_then(Value::as_str).is_none() {
        return Err(ToolFailure::params("character requires a 'name'"));
    }
    let amendment = opt_obj(&map, "amendment");
    gated_write(
        store,
        &mut state,
        Op::UpsertCharacter {
            character: character.clone(),
            amendment,
        },
    )?;
    let id = character["id"].as_str().unwrap_or("");
    let stored = find_by_id(state.doc("cast"), "characters", id).cloned();
    Ok(json!({"project": state.slug, "character": stored}))
}

fn get_character(store: &mut Store, args: &Value) -> R<Value> {
    let map = strict_object(args, &["project", "character"])?;
    let (_slug, state) = load_project(store, &map)?;
    let id = req_id(&map, "character")?;
    match find_by_id(state.doc("cast"), "characters", &id) {
        Some(c) => Ok(json!({"project": state.slug, "character": c})),
        None => Err(ToolFailure::domain(
            "unknown_character",
            format!("no character '{id}' in this project"),
        )),
    }
}

fn list_characters(store: &mut Store, args: &Value) -> R<Value> {
    let map = strict_object(args, &["project"])?;
    let (_slug, state) = load_project(store, &map)?;
    Ok(json!({
        "project": state.slug,
        "characters": state.doc("cast")["characters"],
        "history": state.doc("cast")["history"]
    }))
}

fn add_canon_rule(store: &mut Store, args: &Value) -> R<Value> {
    let map = strict_object(args, &["project", "rule"])?;
    let (_slug, mut state) = load_project(store, &map)?;
    let rule = req_obj(&map, "rule")?;
    req_payload_id(&rule, "canon rule")?;
    let kind = rule.get("kind").and_then(Value::as_str).unwrap_or("");
    if !["lock", "fact", "note"].contains(&kind) {
        return Err(ToolFailure::params(
            "canon rule kind must be lock, fact or note",
        ));
    }
    if kind == "lock" {
        let fields = rule.get("fields").and_then(Value::as_array);
        if fields.is_none() || fields.is_some_and(|f| f.is_empty()) {
            return Err(ToolFailure::params(
                "a lock rule requires a non-empty 'fields' array",
            ));
        }
    }
    if kind == "fact" && (rule.get("field").is_none() || rule.get("value").is_none()) {
        return Err(ToolFailure::params(
            "a fact rule requires 'field' and 'value'",
        ));
    }
    gated_write(store, &mut state, Op::AddCanonRule { rule })?;
    Ok(json!({
        "project": state.slug,
        "rules": state.doc("canon")["rules"]
    }))
}

fn list_canon(store: &mut Store, args: &Value) -> R<Value> {
    let map = strict_object(args, &["project"])?;
    let (_slug, state) = load_project(store, &map)?;
    Ok(json!({"project": state.slug, "rules": state.doc("canon")["rules"]}))
}

fn check_canon(store: &mut Store, args: &Value) -> R<Value> {
    let map = strict_object(args, &["project", "proposal"])?;
    let (_slug, state) = load_project(store, &map)?;
    let proposal = req_obj(&map, "proposal")?;
    let accounts = store.load_accounts()?;
    let op_name = proposal.get("op").and_then(Value::as_str).unwrap_or("");
    let op = match op_name {
        "upsert_character" => Op::UpsertCharacter {
            character: proposal.get("character").cloned().unwrap_or(Value::Null),
            amendment: proposal.get("amendment").cloned(),
        },
        "upsert_scene" => Op::UpsertScene {
            node: proposal.get("node").cloned().unwrap_or(Value::Null),
        },
        "upsert_dialogue" => Op::UpsertDialogue {
            node: proposal.get("node").cloned().unwrap_or(Value::Null),
        },
        "upsert_level" => Op::UpsertLevel {
            level: proposal.get("level").cloned().unwrap_or(Value::Null),
        },
        other => {
            return Err(ToolFailure::params(format!(
                "proposal.op must be upsert_character, upsert_scene, upsert_dialogue or upsert_level: {other}"
            )));
        }
    };
    match gate::decide(&state, &accounts, &op) {
        Ok(()) => Ok(json!({"verdict": "allowed", "op": op_name})),
        Err(r) => Ok(json!({
            "verdict": "refused",
            "op": op_name,
            "code": r.code,
            "message": r.message
        })),
    }
}

// ---------- group 3: scenes and systems ----------

fn declare_flag(store: &mut Store, args: &Value) -> R<Value> {
    let map = strict_object(args, &["project", "flag"])?;
    let (_slug, mut state) = load_project(store, &map)?;
    let flag = req_obj(&map, "flag")?;
    let name = flag.get("name").and_then(Value::as_str).unwrap_or("");
    if !is_valid_id(name) {
        return Err(ToolFailure::params(
            "flag requires a 'name' that is a lowercase slug",
        ));
    }
    let kind = flag.get("kind").and_then(Value::as_str).unwrap_or("");
    if !["story", "relationship", "progression", "system"].contains(&kind) {
        return Err(ToolFailure::params(
            "flag kind must be story, relationship, progression or system",
        ));
    }
    gated_write(store, &mut state, Op::DeclareFlag { flag })?;
    Ok(json!({"project": state.slug, "flags": state.doc("graph")["flags"]}))
}

fn upsert_node(store: &mut Store, args: &Value, dialogue: bool) -> R<Value> {
    let map = strict_object(args, &["project", "node"])?;
    let (_slug, mut state) = load_project(store, &map)?;
    let mut node = req_obj(&map, "node")?;
    req_payload_id(&node, "node")?;
    let want = if dialogue { "dialogue" } else { "scene" };
    if let Some(kind) = node.get("kind").and_then(Value::as_str)
        && kind != want
    {
        return Err(ToolFailure::params(format!(
            "node kind is '{kind}'; use the matching upsert tool for it"
        )));
    }
    node.as_object_mut()
        .expect("object")
        .insert("kind".to_string(), json!(want));
    let op = if dialogue {
        Op::UpsertDialogue { node: node.clone() }
    } else {
        Op::UpsertScene { node: node.clone() }
    };
    gated_write(store, &mut state, op)?;
    let id = node["id"].as_str().unwrap_or("");
    let stored = find_by_id(state.doc("graph"), "nodes", id).cloned();
    Ok(json!({"project": state.slug, "node": stored}))
}

fn get_node(store: &mut Store, args: &Value) -> R<Value> {
    let map = strict_object(args, &["project", "node"])?;
    let (_slug, state) = load_project(store, &map)?;
    let id = req_id(&map, "node")?;
    match find_by_id(state.doc("graph"), "nodes", &id) {
        Some(n) => Ok(json!({"project": state.slug, "node": n})),
        None => Err(ToolFailure::domain(
            "unknown_node",
            format!("no node '{id}' in this project's graph"),
        )),
    }
}

fn list_nodes(store: &mut Store, args: &Value) -> R<Value> {
    let map = strict_object(args, &["project", "kind"])?;
    let (_slug, state) = load_project(store, &map)?;
    let kind = opt_str(&map, "kind");
    let nodes: Vec<Value> = arr(state.doc("graph"), "nodes")
        .iter()
        .filter(|n| kind.is_none() || n.get("kind").and_then(Value::as_str) == kind.as_deref())
        .cloned()
        .collect();
    Ok(json!({"project": state.slug, "nodes": nodes, "count": nodes.len()}))
}

fn upsert_system(store: &mut Store, args: &Value) -> R<Value> {
    let map = strict_object(args, &["project", "system"])?;
    let (_slug, mut state) = load_project(store, &map)?;
    let system = req_obj(&map, "system")?;
    req_payload_id(&system, "system")?;
    let kind = system.get("kind").and_then(Value::as_str).unwrap_or("");
    if ![
        "schedule",
        "economy",
        "state_machine",
        "minigame",
        "progression",
    ]
    .contains(&kind)
    {
        return Err(ToolFailure::params(
            "system kind must be schedule, economy, state_machine, minigame or progression",
        ));
    }
    gated_write(
        store,
        &mut state,
        Op::UpsertSystem {
            system: system.clone(),
        },
    )?;
    let id = system["id"].as_str().unwrap_or("");
    let stored = find_by_id(state.doc("systems"), "systems", id).cloned();
    Ok(json!({"project": state.slug, "system": stored}))
}

fn list_systems(store: &mut Store, args: &Value) -> R<Value> {
    let map = strict_object(args, &["project"])?;
    let (_slug, state) = load_project(store, &map)?;
    Ok(json!({
        "project": state.slug,
        "systems": state.doc("systems")["systems"]
    }))
}

// ---------- group 4: reaction profiles (dialogue lives in group 3) ----------

fn set_reaction_profile(store: &mut Store, args: &Value) -> R<Value> {
    let map = strict_object(args, &["project", "character", "profile"])?;
    let (_slug, mut state) = load_project(store, &map)?;
    let character_id = req_id(&map, "character")?;
    let profile = req_obj(&map, "profile")?;
    gated_write(
        store,
        &mut state,
        Op::SetReactionProfile {
            character_id: character_id.clone(),
            profile,
        },
    )?;
    let stored = find_by_id(state.doc("cast"), "characters", &character_id).cloned();
    Ok(json!({"project": state.slug, "character": stored}))
}

fn preview_reaction(store: &mut Store, args: &Value) -> R<Value> {
    let map = strict_object(args, &["project", "character", "context"])?;
    let (_slug, state) = load_project(store, &map)?;
    let id = req_id(&map, "character")?;
    let context = req_obj(&map, "context")?;
    let Some(character) = find_by_id(state.doc("cast"), "characters", &id) else {
        return Err(ToolFailure::domain(
            "unknown_character",
            format!("no character '{id}' in this project"),
        ));
    };
    let Some(profile) = character.get("reaction_profile").filter(|p| p.is_object()) else {
        return Err(ToolFailure::domain(
            "no_reaction_profile",
            format!("character '{id}' has no reaction profile; set one with set_reaction_profile"),
        ));
    };
    let preview = playtest::preview_reaction(profile, &context);
    Ok(json!({
        "project": state.slug,
        "character": id,
        "context": context,
        "preview": preview
    }))
}

// ---------- group 5: story, plot, levels ----------

fn upsert_arc(store: &mut Store, args: &Value) -> R<Value> {
    let map = strict_object(args, &["project", "arc"])?;
    let (_slug, mut state) = load_project(store, &map)?;
    let arc = req_obj(&map, "arc")?;
    req_payload_id(&arc, "arc")?;
    gated_write(store, &mut state, Op::UpsertArc { arc: arc.clone() })?;
    let id = arc["id"].as_str().unwrap_or("");
    let stored = find_by_id(state.doc("story"), "arcs", id).cloned();
    Ok(json!({"project": state.slug, "arc": stored}))
}

fn validate_story(store: &mut Store, args: &Value) -> R<Value> {
    let map = strict_object(args, &["project"])?;
    let (_slug, state) = load_project(store, &map)?;
    let graph = state.doc("graph");
    let mut issues: Vec<Value> = Vec::new();
    // Beats must reference real nodes.
    for arc in arr(state.doc("story"), "arcs") {
        for beat in arr(arc, "beats") {
            if let Some(node) = beat.get("node").and_then(Value::as_str)
                && find_by_id(graph, "nodes", node).is_none()
            {
                issues.push(json!({
                    "kind": "beat_references_unknown_node",
                    "arc": arc.get("id"),
                    "beat": beat.get("id"),
                    "node": node
                }));
            }
        }
    }
    // Reachability from the start node.
    let start = state
        .meta()
        .get("start_node")
        .and_then(Value::as_str)
        .unwrap_or("");
    let mut reachable: Vec<String> = Vec::new();
    if start.is_empty() {
        issues.push(json!({"kind": "no_start_node", "detail": "project has no start node"}));
    } else {
        let mut stack = vec![start.to_string()];
        while let Some(id) = stack.pop() {
            if reachable.contains(&id) || id == gate::TERMINAL {
                continue;
            }
            reachable.push(id.clone());
            if let Some(node) = find_by_id(graph, "nodes", &id) {
                if let Some(next) = node.get("next").and_then(Value::as_str) {
                    stack.push(next.to_string());
                }
                for choice in arr(node, "choices") {
                    if let Some(next) = choice.get("next").and_then(Value::as_str) {
                        stack.push(next.to_string());
                    }
                }
            }
        }
    }
    let unreachable: Vec<String> = arr(graph, "nodes")
        .iter()
        .filter_map(|n| n.get("id").and_then(Value::as_str))
        .filter(|id| !reachable.iter().any(|r| r == id))
        .map(str::to_string)
        .collect();
    for id in &unreachable {
        issues.push(json!({"kind": "unreachable_node", "node": id}));
    }
    // Endings: arcs flagged as endings + terminal-reaching nodes.
    let endings: Vec<Value> = arr(state.doc("story"), "arcs")
        .iter()
        .filter(|a| a.get("ending").and_then(Value::as_bool) == Some(true))
        .map(|a| json!({"arc": a.get("id"), "title": a.get("title")}))
        .collect();
    Ok(json!({
        "project": state.slug,
        "ok": issues.is_empty(),
        "issues": issues,
        "reachable_nodes": reachable.len(),
        "unreachable_nodes": unreachable,
        "endings": endings
    }))
}

fn upsert_level(store: &mut Store, args: &Value) -> R<Value> {
    let map = strict_object(args, &["project", "level"])?;
    let (_slug, mut state) = load_project(store, &map)?;
    let level = req_obj(&map, "level")?;
    req_payload_id(&level, "level")?;
    gated_write(
        store,
        &mut state,
        Op::UpsertLevel {
            level: level.clone(),
        },
    )?;
    let id = level["id"].as_str().unwrap_or("");
    let stored = find_by_id(state.doc("levels"), "levels", id).cloned();
    Ok(json!({"project": state.slug, "level": stored}))
}

fn list_levels(store: &mut Store, args: &Value) -> R<Value> {
    let map = strict_object(args, &["project"])?;
    let (_slug, state) = load_project(store, &map)?;
    Ok(json!({
        "project": state.slug,
        "levels": state.doc("levels")["levels"]
    }))
}

fn generate_playtest_tool(store: &mut Store, args: &Value) -> R<Value> {
    let map = strict_object(args, &["project", "seed", "max_steps"])?;
    let (_slug, state) = load_project(store, &map)?;
    if state
        .meta()
        .get("start_node")
        .and_then(Value::as_str)
        .is_none()
    {
        return Err(ToolFailure::domain(
            "no_start_node",
            "the project has no start node; author a scene and set start_node first",
        ));
    }
    let seed = map.get("seed").and_then(Value::as_u64).unwrap_or(1);
    let max_steps = map
        .get("max_steps")
        .and_then(Value::as_u64)
        .map_or(crate::model::PLAYTEST_DEFAULT_STEPS, |n| n as usize);
    Ok(playtest::generate_playtest(&state, seed, max_steps))
}

// ---------- group 6: content and safety systems ----------

fn get_safety(store: &mut Store, args: &Value) -> R<Value> {
    let map = strict_object(args, &["project"])?;
    let (_slug, state) = load_project(store, &map)?;
    Ok(json!({
        "project": state.slug,
        "rating": state.meta().get("rating"),
        "age_gate": state.meta().get("age_gate"),
        "content_tags": state.meta().get("content_tags"),
        "surfaces": state.doc("safety")["surfaces"],
        "audit": gate::safety_audit(&state)
    }))
}

fn set_safety_surface(store: &mut Store, args: &Value) -> R<Value> {
    let map = strict_object(args, &["project", "surface", "config"])?;
    let (_slug, mut state) = load_project(store, &map)?;
    let surface = req_enum(&map, "surface", &gate::SAFETY_SURFACES)?;
    let config = req_obj(&map, "config")?;
    gated_write(store, &mut state, Op::SetSafetySurface { surface, config })?;
    Ok(json!({
        "project": state.slug,
        "surfaces": state.doc("safety")["surfaces"],
        "audit": gate::safety_audit(&state)
    }))
}

fn audit_safety(store: &mut Store, args: &Value) -> R<Value> {
    let map = strict_object(args, &["project"])?;
    let (_slug, state) = load_project(store, &map)?;
    Ok(json!({
        "project": state.slug,
        "audit": gate::safety_audit(&state)
    }))
}

// ---------- group 7: asset routing and provenance ----------

fn register_asset(store: &mut Store, args: &Value) -> R<Value> {
    let map = strict_object(args, &["project", "asset"])?;
    let (_slug, mut state) = load_project(store, &map)?;
    let asset = req_obj(&map, "asset")?;
    req_payload_id(&asset, "asset")?;
    let kind = asset.get("kind").and_then(Value::as_str).unwrap_or("");
    if ![
        "sprite", "model3d", "audio", "haptic", "text", "build", "other",
    ]
    .contains(&kind)
    {
        return Err(ToolFailure::params(
            "asset kind must be sprite, model3d, audio, haptic, text, build or other",
        ));
    }
    let route = asset.get("route_to").and_then(Value::as_str).unwrap_or("");
    if !["lumen", "torori", "ache", "neemu", "local", "unassigned"].contains(&route) {
        return Err(ToolFailure::params(
            "asset route_to must name its specialist owner: lumen, torori, ache, neemu, local or unassigned",
        ));
    }
    gated_write(
        store,
        &mut state,
        Op::RegisterAsset {
            asset: asset.clone(),
        },
    )?;
    let id = asset["id"].as_str().unwrap_or("");
    let stored = find_by_id(state.doc("assets"), "assets", id).cloned();
    Ok(json!({"project": state.slug, "asset": stored}))
}

fn list_assets(store: &mut Store, args: &Value) -> R<Value> {
    let map = strict_object(args, &["project"])?;
    let (_slug, state) = load_project(store, &map)?;
    Ok(json!({
        "project": state.slug,
        "assets": state.doc("assets")["assets"],
        "handoffs": state.doc("assets")["handoffs"]
    }))
}

fn record_asset_handoff(store: &mut Store, args: &Value) -> R<Value> {
    let map = strict_object(args, &["project", "asset", "target"])?;
    let (_slug, mut state) = load_project(store, &map)?;
    let asset_id = req_id(&map, "asset")?;
    let target = req_str(&map, "target")?;
    let specialists = ["lumen", "torori", "ache", "neemu", "slipway"];
    if !specialists.contains(&target.as_str()) && channels::find_channel(&target).is_none() {
        return Err(ToolFailure::params(format!(
            "handoff target must be a specialist ({}) or a registry channel id: {target}",
            specialists.join(", ")
        )));
    }
    gated_write(
        store,
        &mut state,
        Op::RecordAssetHandoff {
            asset_id: asset_id.clone(),
            target,
        },
    )?;
    Ok(json!({
        "project": state.slug,
        "handoffs": state.doc("assets")["handoffs"]
    }))
}

// ---------- group 8: playtest and QA ----------

fn run_playtest(store: &mut Store, args: &Value) -> R<Value> {
    let map = strict_object(args, &["project", "seed", "max_steps"])?;
    let (_slug, mut state) = load_project(store, &map)?;
    let seed = map.get("seed").and_then(Value::as_u64).unwrap_or(1);
    let max_steps = map
        .get("max_steps")
        .and_then(Value::as_u64)
        .map_or(crate::model::PLAYTEST_DEFAULT_STEPS, |n| n as usize);
    let generated = playtest::generate_playtest(&state, seed, max_steps);
    let seq = arr(state.doc("qa"), "runs").len() + 1;
    let run = json!({
        "id": format!("run-{seq}"),
        "seed": seed,
        "max_steps": max_steps,
        "start": generated["start"],
        "steps_count": generated["steps_count"],
        "truncated": generated["truncated"],
        "stop_reason": generated["stop_reason"],
        "hash": generated["hash"]
    });
    gated_write(
        store,
        &mut state,
        Op::RecordPlaytestRun { run: run.clone() },
    )?;
    Ok(json!({"project": state.slug, "run": run}))
}

fn verify_replay(store: &mut Store, args: &Value) -> R<Value> {
    let map = strict_object(args, &["project", "run"])?;
    let (_slug, state) = load_project(store, &map)?;
    let run_id = req_str(&map, "run")?;
    let Some(run) = arr(state.doc("qa"), "runs")
        .iter()
        .find(|r| r.get("id").and_then(Value::as_str) == Some(run_id.as_str()))
    else {
        return Err(ToolFailure::domain(
            "unknown_run",
            format!("no recorded playtest run '{run_id}'"),
        ));
    };
    let seed = run.get("seed").and_then(Value::as_u64).unwrap_or(1);
    let max_steps = run
        .get("max_steps")
        .and_then(Value::as_u64)
        .map_or(crate::model::PLAYTEST_DEFAULT_STEPS, |n| n as usize);
    let regenerated = playtest::generate_playtest(&state, seed, max_steps);
    let recorded_hash = run.get("hash").cloned().unwrap_or(Value::Null);
    let current_hash = regenerated["hash"].clone();
    Ok(json!({
        "project": state.slug,
        "run": run_id,
        "match": recorded_hash == current_hash,
        "recorded_hash": recorded_hash,
        "current_hash": current_hash,
        "note": if recorded_hash == current_hash {
            "the authored graph still replays byte-identically"
        } else {
            "the graph changed since the run was recorded (or the run was recorded against other state)"
        }
    }))
}

fn add_screenshot_spec(store: &mut Store, args: &Value) -> R<Value> {
    let map = strict_object(args, &["project", "spec"])?;
    let (_slug, mut state) = load_project(store, &map)?;
    let spec = req_obj(&map, "spec")?;
    req_payload_id(&spec, "screenshot spec")?;
    if spec.get("node").and_then(Value::as_str).is_none() {
        return Err(ToolFailure::params("a screenshot spec requires a 'node'"));
    }
    gated_write(store, &mut state, Op::AddScreenshotSpec { spec })?;
    Ok(json!({
        "project": state.slug,
        "screenshot_specs": state.doc("qa")["screenshot_specs"]
    }))
}

fn record_defect(store: &mut Store, args: &Value) -> R<Value> {
    let map = strict_object(args, &["project", "defect"])?;
    let (_slug, mut state) = load_project(store, &map)?;
    let defect = req_obj(&map, "defect")?;
    req_payload_id(&defect, "defect")?;
    if defect.get("title").and_then(Value::as_str).is_none() {
        return Err(ToolFailure::params("a defect requires a 'title'"));
    }
    let severity = defect.get("severity").and_then(Value::as_str).unwrap_or("");
    if !["low", "medium", "high", "critical"].contains(&severity) {
        return Err(ToolFailure::params(
            "defect severity must be low, medium, high or critical",
        ));
    }
    gated_write(store, &mut state, Op::RecordDefect { defect })?;
    Ok(json!({
        "project": state.slug,
        "defects": state.doc("qa")["defects"]
    }))
}

fn list_defects(store: &mut Store, args: &Value) -> R<Value> {
    let map = strict_object(args, &["project", "status"])?;
    let (_slug, state) = load_project(store, &map)?;
    let status = opt_str(&map, "status");
    let defects: Vec<Value> = arr(state.doc("qa"), "defects")
        .iter()
        .filter(|d| {
            status.is_none()
                || d.get("status").and_then(Value::as_str) == status.as_deref()
                || (status.as_deref() == Some("open") && d.get("status").is_none())
        })
        .cloned()
        .collect();
    Ok(json!({"project": state.slug, "defects": defects, "count": defects.len()}))
}

// ---------- group 9: channel registry + release handoff ----------

fn list_channels(_store: &mut Store, args: &Value) -> R<Value> {
    let _ = strict_object(args, &[])?;
    let registry = channels::registry_json();
    let verified = channels::CHANNELS
        .iter()
        .filter(|c| c.verification_status == "verified")
        .count();
    let mut sections = serde_json::Map::new();
    for name in channels::SECTIONS {
        let count = channels::CHANNELS
            .iter()
            .filter(|c| c.section == name)
            .count();
        sections.insert(name.to_string(), json!(count));
    }
    Ok(json!({
        "channels": registry,
        "count": channels::CHANNELS.len(),
        "verified": verified,
        "flagged": channels::CHANNELS.len() - verified,
        "sections": sections,
        "note": "flagged channels failed live verification at build time; the gate refuses handoffs to them. Sections (D-128 annex): assets_sfw / services_sfw are SFW-only channels, never adult channels; services channels take commissions, not releases"
    }))
}

fn get_channel(_store: &mut Store, args: &Value) -> R<Value> {
    let map = strict_object(args, &["channel"])?;
    let id = req_id(&map, "channel")?;
    match channels::find_channel(&id) {
        Some(c) => Ok(channels::channel_json(c)),
        None => Err(ToolFailure::domain(
            "unknown_channel",
            format!("'{id}' is not in the channel-target registry"),
        )),
    }
}

fn prepare_handoff(store: &mut Store, args: &Value) -> R<Value> {
    let map = strict_object(args, &["project", "channel", "artifacts"])?;
    let (_slug, mut state) = load_project(store, &map)?;
    let channel = req_id(&map, "channel")?;
    let Some(artifacts) = map.get("artifacts").and_then(Value::as_array) else {
        return Err(ToolFailure::params(
            "artifacts must be an array of {name, sha256, kind}",
        ));
    };
    for artifact in artifacts {
        if !artifact.is_object()
            || artifact.get("name").and_then(Value::as_str).is_none()
            || artifact.get("sha256").and_then(Value::as_str).is_none()
        {
            return Err(ToolFailure::params(
                "each artifact requires name and sha256 (the gate verifies the digest)",
            ));
        }
    }
    gated_write(
        store,
        &mut state,
        Op::PrepareHandoff {
            channel,
            artifacts: artifacts.clone(),
        },
    )?;
    let handoffs = state.doc("releases")["handoffs"].clone();
    let latest = arr(state.doc("releases"), "handoffs").last().cloned();
    Ok(json!({
        "project": state.slug,
        "handoff": latest,
        "handoffs": handoffs
    }))
}

fn list_handoffs(store: &mut Store, args: &Value) -> R<Value> {
    let map = strict_object(args, &["project"])?;
    let (_slug, state) = load_project(store, &map)?;
    Ok(json!({
        "project": state.slug,
        "handoffs": state.doc("releases")["handoffs"]
    }))
}

// ---------- group 10: channel credentials and sessions ----------

fn register_channel_account(store: &mut Store, args: &Value) -> R<Value> {
    let map = strict_object(args, &["record"])?;
    let record = req_obj(&map, "record")?;
    for key in ["channel", "account", "credential_kind", "store_ref"] {
        if record.get(key).is_none() {
            return Err(ToolFailure::params(format!(
                "account record requires '{key}' (channel, account identity, credential kind, store reference)"
            )));
        }
    }
    let stored = gated_account_write(store, AccountOp::RegisterAccount { record })?;
    let accounts = store.load_accounts()?;
    let _ = &accounts;
    Ok(json!({
        "account": accounts::account_status(&stored, None),
        "note": "reference registered; the value itself never entered Ardor — it lives in the named estate store"
    }))
}

fn list_channel_accounts(store: &mut Store, args: &Value) -> R<Value> {
    let map = strict_object(args, &["channel", "now"])?;
    let accounts = store.load_accounts()?;
    let channel = opt_str(&map, "channel");
    let now = opt_str(&map, "now");
    let statuses: Vec<Value> = arr(&accounts, "accounts")
        .iter()
        .filter(|a| {
            channel.is_none() || a.get("channel").and_then(Value::as_str) == channel.as_deref()
        })
        .map(|a| accounts::account_status(a, now.as_deref()))
        .collect();
    Ok(json!({"accounts": statuses, "count": statuses.len()}))
}

fn get_channel_account(store: &mut Store, args: &Value) -> R<Value> {
    let map = strict_object(args, &["channel", "now"])?;
    let channel_id = req_id(&map, "channel")?;
    let now = opt_str(&map, "now");
    let Some(channel) = channels::find_channel(&channel_id) else {
        return Err(ToolFailure::domain(
            "unknown_channel",
            format!("'{channel_id}' is not in the channel-target registry"),
        ));
    };
    let accounts = store.load_accounts()?;
    let statuses: Vec<Value> = arr(&accounts, "accounts")
        .iter()
        .filter(|a| a.get("channel").and_then(Value::as_str) == Some(channel_id.as_str()))
        .map(|a| accounts::account_status(a, now.as_deref()))
        .collect();
    Ok(json!({
        "channel": channels::channel_json(channel),
        "accounts": statuses,
        "count": statuses.len()
    }))
}

fn record_auth_verification(store: &mut Store, args: &Value) -> R<Value> {
    let map = strict_object(
        args,
        &["account_id", "result", "method", "verified_at", "detail"],
    )?;
    let account_id = req_str(&map, "account_id")?;
    let result = req_enum(&map, "result", &["verified", "failed"])?;
    let method = req_str(&map, "method")?;
    let verified_at = req_str(&map, "verified_at")?;
    let detail = opt_str(&map, "detail");
    let stored = gated_account_write(
        store,
        AccountOp::RecordVerification {
            account_id,
            result,
            method,
            verified_at,
            detail,
        },
    )?;
    Ok(json!({
        "account": accounts::account_status(&stored, None),
        "note": "attestation recorded; the probe ran on the value holder's side — Ardor never saw the credential"
    }))
}

fn credential_status(store: &mut Store, args: &Value) -> R<Value> {
    let map = strict_object(args, &["channel", "now"])?;
    let accounts = store.load_accounts()?;
    let channel = opt_str(&map, "channel");
    let now = opt_str(&map, "now");
    // Per-channel rollup over the whole registry, so missing auth
    // is visible, not just present auth.
    let mut rows = Vec::new();
    for c in channels::CHANNELS {
        if let Some(want) = &channel
            && c.id != want
        {
            continue;
        }
        let records: Vec<Value> = arr(&accounts, "accounts")
            .iter()
            .filter(|a| a.get("channel").and_then(Value::as_str) == Some(c.id))
            .map(|a| accounts::account_status(a, now.as_deref()))
            .collect();
        let best = if records
            .iter()
            .any(|r| r["status"].as_str() == Some("verified"))
        {
            "verified"
        } else if records.is_empty() {
            "missing"
        } else if records
            .iter()
            .any(|r| r["status"].as_str() == Some("expired"))
        {
            "expired"
        } else {
            "unverified"
        };
        rows.push(json!({
            "channel": c.id,
            "auth_kind": channels::auth_for(c.id).map(|a| a.kind),
            "status": best,
            "accounts": records
        }));
    }
    Ok(json!({"channels": rows, "count": rows.len()}))
}

fn probe_channel_auth(store: &mut Store, args: &Value) -> R<Value> {
    let map = strict_object(args, &["channel"])?;
    let channel_id = req_id(&map, "channel")?;
    let Some(channel) = channels::find_channel(&channel_id) else {
        return Err(ToolFailure::domain(
            "unknown_channel",
            format!("'{channel_id}' is not in the channel-target registry"),
        ));
    };
    let auth = channels::auth_for(&channel_id);
    let accounts = store.load_accounts()?;
    let statuses: Vec<Value> = arr(&accounts, "accounts")
        .iter()
        .filter(|a| a.get("channel").and_then(Value::as_str) == Some(channel_id.as_str()))
        .map(|a| accounts::account_status(a, None))
        .collect();
    Ok(json!({
        "channel": channel.id,
        "credential_kind": auth.map(|a| a.kind),
        "probe": auth.map(|a| a.probe),
        "auth_note": auth.map(|a| a.note),
        "accounts": statuses,
        "how_it_works": "Ardor holds no credential value, so the probe runs on the value holder's side (primo pass / Secure Vault flow); record its outcome with record_auth_verification"
    }))
}

fn publish_preflight(store: &mut Store, args: &Value) -> R<Value> {
    let map = strict_object(args, &["project", "channel", "now"])?;
    let (_slug, state) = load_project(store, &map)?;
    let channel = req_id(&map, "channel")?;
    let now = opt_str(&map, "now");
    let accounts = store.load_accounts()?;
    Ok(accounts::publish_preflight(
        &state,
        &accounts,
        &channel,
        now.as_deref(),
    ))
}

// ---------- dispatch ----------

/// Route one tool call. Unknown tools are params errors, as in
/// the fleet pattern.
pub fn dispatch(store: &mut Store, name: &str, arguments: &Value) -> R<Value> {
    match name {
        // group 10: channel credentials and sessions
        "credential_status" => credential_status(store, arguments),
        "get_channel_account" => get_channel_account(store, arguments),
        "list_channel_accounts" => list_channel_accounts(store, arguments),
        "probe_channel_auth" => probe_channel_auth(store, arguments),
        "publish_preflight" => publish_preflight(store, arguments),
        "record_auth_verification" => record_auth_verification(store, arguments),
        "register_channel_account" => register_channel_account(store, arguments),
        // group 9: channel registry + release handoff
        "get_channel" => get_channel(store, arguments),
        "list_channels" => list_channels(store, arguments),
        "list_handoffs" => list_handoffs(store, arguments),
        "prepare_handoff" => prepare_handoff(store, arguments),
        // group 8: playtest and QA
        "add_screenshot_spec" => add_screenshot_spec(store, arguments),
        "list_defects" => list_defects(store, arguments),
        "record_defect" => record_defect(store, arguments),
        "run_playtest" => run_playtest(store, arguments),
        "verify_replay" => verify_replay(store, arguments),
        // group 7: asset routing and provenance
        "list_assets" => list_assets(store, arguments),
        "record_asset_handoff" => record_asset_handoff(store, arguments),
        "register_asset" => register_asset(store, arguments),
        // group 6: content and safety systems
        "audit_safety" => audit_safety(store, arguments),
        "get_safety" => get_safety(store, arguments),
        "set_safety_surface" => set_safety_surface(store, arguments),
        // group 5: story, plot, levels
        "generate_playtest" => generate_playtest_tool(store, arguments),
        "list_levels" => list_levels(store, arguments),
        "upsert_arc" => upsert_arc(store, arguments),
        "upsert_level" => upsert_level(store, arguments),
        "validate_story" => validate_story(store, arguments),
        // group 4: dialogue + reaction profiles (upsert_dialogue is group 3's verb)
        "preview_reaction" => preview_reaction(store, arguments),
        "set_reaction_profile" => set_reaction_profile(store, arguments),
        // group 3: scenes and systems
        "declare_flag" => declare_flag(store, arguments),
        "get_node" => get_node(store, arguments),
        "list_nodes" => list_nodes(store, arguments),
        "list_systems" => list_systems(store, arguments),
        "upsert_dialogue" => upsert_node(store, arguments, true),
        "upsert_scene" => upsert_node(store, arguments, false),
        "upsert_system" => upsert_system(store, arguments),
        // group 2: cast and canon
        "add_canon_rule" => add_canon_rule(store, arguments),
        "check_canon" => check_canon(store, arguments),
        "get_character" => get_character(store, arguments),
        "list_canon" => list_canon(store, arguments),
        "list_characters" => list_characters(store, arguments),
        "upsert_character" => upsert_character(store, arguments),
        // group 1: project scaffolding
        "create_project" => create_project(store, arguments),
        "get_project" => get_project(store, arguments),
        "list_projects" => list_projects(store, arguments),
        "update_project_meta" => update_project_meta(store, arguments),
        "" => Err(ToolFailure::params("tools/call requires a tool name")),
        other => Err(ToolFailure::params(format!("unknown tool: {other}"))),
    }
}
