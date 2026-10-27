//! Deterministic playtest generation and reaction preview.
//!
//! Playtest generation walks the authored graph from the
//! project's start node. Branch choices are drawn from a
//! SplitMix64 stream seeded by the caller, so the same graph +
//! same seed always yields the same step list and the same
//! SHA-256 hash — that stability is what `verify_replay` checks.
//! The walk is bounded (step budget + per-node visit cap) and
//! records exactly why it stopped.
//!
//! Reaction preview is a pure function of a character's reaction
//! profile plus a context (relationship, arousal, fatigue,
//! conditioning, prior choices, scene history). It uses integer
//! arithmetic only, so previews are reproducible and testable.

use serde_json::{Value, json};

use crate::gate::TERMINAL;
use crate::model::{
    PLAYTEST_MAX_STEPS, PLAYTEST_NODE_VISIT_CAP, SplitMix64, arr, canonical, find_by_id, sha256_hex,
};
use crate::store::ProjectState;

fn str_field<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str)
}

/// Generate one deterministic playthrough of the project's graph.
/// Returns `{seed, start, steps, steps_count, truncated, stop_reason, hash}`.
pub fn generate_playtest(state: &ProjectState, seed: u64, max_steps: usize) -> Value {
    let max_steps = max_steps.clamp(1, PLAYTEST_MAX_STEPS);
    let graph = state.doc("graph");
    let start = str_field(state.meta(), "start_node")
        .unwrap_or("")
        .to_string();
    let mut rng = SplitMix64::new(seed);
    let mut steps: Vec<Value> = Vec::new();
    let mut visits: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut current = start.clone();
    // Assigned on every exit path out of the walk loop below.
    let stop_reason: &str;
    loop {
        if steps.len() >= max_steps {
            stop_reason = "step_budget";
            break;
        }
        if current == TERMINAL || current.is_empty() {
            stop_reason = "terminal";
            break;
        }
        let Some(node) = find_by_id(graph, "nodes", &current) else {
            stop_reason = "missing_node";
            break;
        };
        let count = visits.entry(current.clone()).or_insert(0);
        *count += 1;
        if *count > PLAYTEST_NODE_VISIT_CAP {
            stop_reason = "loop_guard";
            break;
        }
        let kind = str_field(node, "kind").unwrap_or("scene");
        let step_index = steps.len();
        // Branch selection: a scene follows `next`; a dialogue
        // with choices draws one deterministically.
        let (next, choice) = if kind == "dialogue" {
            let choices = arr(node, "choices");
            if choices.is_empty() {
                (str_field(node, "next").map(str::to_string), None)
            } else {
                let pick = rng.below(choices.len());
                (
                    str_field(&choices[pick], "next").map(str::to_string),
                    Some(json!({
                        "index": pick,
                        "label": str_field(&choices[pick], "label").unwrap_or("")
                    })),
                )
            }
        } else {
            (str_field(node, "next").map(str::to_string), None)
        };
        steps.push(json!({
            "i": step_index,
            "node": current,
            "kind": kind,
            "choice": choice,
            "next": next
        }));
        match next {
            Some(n) => current = n,
            None => {
                stop_reason = "terminal";
                break;
            }
        }
    }
    let truncated = stop_reason != "terminal";
    let hash = sha256_hex(&canonical(&json!({
        "seed": seed,
        "start": start,
        "steps": steps
    })));
    json!({
        "seed": seed,
        "start": start,
        "steps": steps,
        "steps_count": steps.len(),
        "truncated": truncated,
        "stop_reason": stop_reason,
        "hash": hash
    })
}

/// Preview how a character reacts under a context, from their
/// reaction profile. Pure and deterministic. The context fields
/// (all 0..=100 unless noted): `relationship`, `arousal`,
/// `sensitivity_state` (defaults to the profile's sensitivity),
/// `fatigue`, `conditioning` (defaults to the profile's),
/// `prior_choices` (count), `scene_history` (count).
pub fn preview_reaction(profile: &Value, context: &Value) -> Value {
    let get =
        |v: &Value, key: &str, default: i64| v.get(key).and_then(Value::as_i64).unwrap_or(default);
    let base = get(profile, "base_receptivity", 50);
    let sensitivity = get(
        context,
        "sensitivity_state",
        get(profile, "sensitivity", 50),
    );
    let conditioning = get(context, "conditioning", get(profile, "conditioning", 0));
    let relationship = get(context, "relationship", 50);
    let arousal = get(context, "arousal", 0);
    let fatigue = get(context, "fatigue", 0);
    let prior_choices = get(context, "prior_choices", 0);
    let scene_history = get(context, "scene_history", 0);

    // Integer formula, documented so authors can reason about it:
    // receptivity is the base; relationship and arousal lift it;
    // sensitivity and conditioning modulate; fatigue drags;
    // familiarity (history, prior choices) settles it slightly.
    let score = base + relationship / 2 + arousal / 2 + sensitivity / 4 + conditioning / 5
        - fatigue / 3
        + (scene_history.min(10)) / 2
        + (prior_choices.min(10)) / 4;
    let score = score.clamp(0, 100);
    let band = match score {
        0..=24 => "guarded",
        25..=49 => "warming",
        50..=74 => "engaged",
        _ => "ardent",
    };
    // Per-action modifiers from the profile, if the context names
    // an action kind: a flat integer delta recorded by the author.
    let action_kind = str_field(context, "action_kind").unwrap_or("");
    let modifier = if action_kind.is_empty() {
        0
    } else {
        profile
            .get("action_modifiers")
            .and_then(|m| m.get(action_kind))
            .and_then(Value::as_i64)
            .unwrap_or(0)
    };
    let adjusted = (score + modifier).clamp(0, 100);
    json!({
        "score": adjusted,
        "base_score": score,
        "band": match adjusted {
            0..=24 => "guarded",
            25..=49 => "warming",
            50..=74 => "engaged",
            _ => "ardent",
        },
        "profile_band": band,
        "action_kind": if action_kind.is_empty() { Value::Null } else { json!(action_kind) },
        "action_modifier": modifier,
        "factors": {
            "base_receptivity": base,
            "relationship": relationship,
            "arousal": arousal,
            "sensitivity": sensitivity,
            "conditioning": conditioning,
            "fatigue": fatigue,
            "prior_choices": prior_choices,
            "scene_history": scene_history
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::Store;
    use serde_json::json;
    use std::fs;

    fn fixture_state() -> ProjectState {
        let root = std::env::temp_dir().join(format!("ardor-playtest-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let store = Store::open(&root).unwrap();
        let mut state = Store::blank_state("demo");
        state
            .doc_mut("project")
            .as_object_mut()
            .unwrap()
            .insert("start_node".to_string(), json!("intro"));
        let nodes = json!([
            {"id": "intro", "kind": "scene", "next": "talk"},
            {"id": "talk", "kind": "dialogue", "choices": [
                {"label": "stay", "next": "deep"},
                {"label": "leave", "next": "END"}
            ]},
            {"id": "deep", "kind": "scene", "next": "END"}
        ]);
        state
            .doc_mut("graph")
            .as_object_mut()
            .unwrap()
            .insert("nodes".to_string(), nodes);
        let _ = store;
        let _ = fs::remove_dir_all(&root);
        state
    }

    #[test]
    fn generation_is_deterministic_and_hashed() {
        let state = fixture_state();
        let a = generate_playtest(&state, 7, 64);
        let b = generate_playtest(&state, 7, 64);
        assert_eq!(a, b, "same seed must produce the identical run");
        assert_eq!(a["steps"][0]["node"], "intro");
        assert!(a["hash"].as_str().unwrap().len() == 64);
        let c = generate_playtest(&state, 8, 64);
        // A different seed may still collide on a tiny graph, but
        // the recorded seed is part of the hashed content.
        assert_ne!(a["hash"], c["hash"]);
    }

    #[test]
    fn loop_guard_bounds_the_walk() {
        let mut state = fixture_state();
        state.doc_mut("graph").as_object_mut().unwrap().insert(
            "nodes".to_string(),
            json!([
                {"id": "intro", "kind": "scene", "next": "intro"}
            ]),
        );
        let run = generate_playtest(&state, 1, 512);
        assert_eq!(run["stop_reason"], "loop_guard");
        assert_eq!(run["truncated"], true);
    }

    #[test]
    fn reaction_preview_is_pure_and_banded() {
        let profile = json!({"base_receptivity": 40, "sensitivity": 60, "conditioning": 10});
        let context = json!({"relationship": 80, "arousal": 50, "fatigue": 30});
        let a = preview_reaction(&profile, &context);
        let b = preview_reaction(&profile, &context);
        assert_eq!(a, b);
        // 40 + 40 + 25 + 15 + 2 - 10 = 112 -> clamped 100 -> ardent.
        assert_eq!(a["score"], 100);
        assert_eq!(a["band"], "ardent");
        let low = preview_reaction(
            &json!({"base_receptivity": 5, "sensitivity": 0, "conditioning": 0}),
            &json!({"relationship": 0, "arousal": 0, "fatigue": 90}),
        );
        assert_eq!(low["band"], "guarded");
    }
}
