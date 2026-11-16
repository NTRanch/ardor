//! Playtest result parsing and validation.
//!
//! Two server shapes exist: `generate_playtest` returns the run
//! inline (`{seed, start, steps: [...], steps_count, truncated,
//! stop_reason, hash}`) and `run_playtest` returns it recorded
//! (`{project, run: {id, seed, steps_count, truncated,
//! stop_reason, hash}}`, no step list). Both are validated
//! before anything downstream — especially the trigger engine —
//! may treat a run as completed: the hash must be a 64-digit
//! SHA-256 hex string, the stop reason must be one the server
//! can actually produce, and a present step list must match
//! its declared count. Malformed results are refused, never
//! repaired.

use serde_json::Value;

/// Stop reasons ardor's playtest walk can produce
/// (`ardor::playtest`).
pub const STOP_REASONS: [&str; 4] = ["terminal", "step_budget", "missing_node", "loop_guard"];

/// One validated playtest run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaytestRun {
    /// Project the run belongs to (from the call, when the
    /// payload does not carry it).
    pub project: String,
    /// Run id (`run_playtest` only; empty for generated runs).
    pub id: String,
    /// Seed the run was generated under.
    pub seed: u64,
    /// SHA-256 replay hash, 64 lowercase hex chars.
    pub hash: String,
    /// Number of steps walked.
    pub steps_count: usize,
    /// Why the walk stopped (one of [`STOP_REASONS`]).
    pub stop_reason: String,
    /// True when the walk stopped for a non-terminal reason.
    pub truncated: bool,
    /// Node ids in walk order, when the payload carried steps.
    pub step_nodes: Vec<String>,
}

/// Validate a playtest payload (`generate_playtest` or
/// `run_playtest` shape) for `project`.
pub fn parse_playtest(project: &str, value: &Value) -> Result<PlaytestRun, String> {
    let run = value.get("run").unwrap_or(value);
    let hash = run
        .get("hash")
        .and_then(Value::as_str)
        .ok_or("playtest result missing hash")?;
    if hash.len() != 64 || !hash.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(format!(
            "playtest hash is not a SHA-256 hex digest: {hash:.40}"
        ));
    }
    let stop_reason = run
        .get("stop_reason")
        .and_then(Value::as_str)
        .ok_or("playtest result missing stop_reason")?;
    if !STOP_REASONS.contains(&stop_reason) {
        return Err(format!("unknown stop_reason: {stop_reason}"));
    }
    let steps_count = run
        .get("steps_count")
        .and_then(Value::as_u64)
        .ok_or("playtest result missing steps_count")?;
    let steps_count = usize::try_from(steps_count).map_err(|_| "steps_count overflow")?;
    let mut step_nodes = Vec::new();
    if let Some(steps) = run.get("steps") {
        let steps = steps.as_array().ok_or("playtest steps is not an array")?;
        if steps.len() != steps_count {
            return Err(format!(
                "steps_count {steps_count} disagrees with {} steps present",
                steps.len()
            ));
        }
        for step in steps {
            let node = step
                .get("node")
                .and_then(Value::as_str)
                .ok_or("playtest step missing node")?;
            step_nodes.push(node.to_string());
        }
    }
    Ok(PlaytestRun {
        project: project.to_string(),
        id: run
            .get("id")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string(),
        seed: run.get("seed").and_then(Value::as_u64).unwrap_or(0),
        hash: hash.to_string(),
        steps_count,
        stop_reason: stop_reason.to_string(),
        truncated: run
            .get("truncated")
            .and_then(Value::as_bool)
            .unwrap_or(false),
        step_nodes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const HASH: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

    fn generated() -> Value {
        json!({
            "seed": 7, "start": "intro",
            "steps": [
                {"i": 0, "node": "intro", "kind": "scene", "choice": null, "next": "hall"},
                {"i": 1, "node": "hall", "kind": "scene", "choice": null, "next": null}
            ],
            "steps_count": 2, "truncated": false, "stop_reason": "terminal", "hash": HASH
        })
    }

    #[test]
    fn generated_shape_parses() {
        let run = parse_playtest("demo", &generated()).expect("parse");
        assert_eq!(run.steps_count, 2);
        assert_eq!(run.step_nodes, vec!["intro", "hall"]);
        assert_eq!(run.hash, HASH);
    }

    #[test]
    fn recorded_shape_parses() {
        let value = json!({"project": "demo", "run": {
            "id": "run-3", "seed": 1, "steps_count": 5, "truncated": true,
            "stop_reason": "step_budget", "hash": HASH
        }});
        let run = parse_playtest("demo", &value).expect("parse");
        assert_eq!(run.id, "run-3");
        assert!(run.truncated);
        assert!(run.step_nodes.is_empty());
    }

    // Adversarial: every malformed shape is refused.

    #[test]
    fn short_hash_refused() {
        let mut value = generated();
        value["hash"] = json!("abcd");
        assert!(parse_playtest("demo", &value).is_err());
    }

    #[test]
    fn non_hex_hash_refused() {
        let mut value = generated();
        value["hash"] = json!("z".repeat(64));
        assert!(parse_playtest("demo", &value).is_err());
    }

    #[test]
    fn unknown_stop_reason_refused() {
        let mut value = generated();
        value["stop_reason"] = json!("exploded");
        assert!(parse_playtest("demo", &value).is_err());
    }

    #[test]
    fn count_mismatch_refused() {
        let mut value = generated();
        value["steps_count"] = json!(99);
        assert!(parse_playtest("demo", &value).is_err());
    }

    #[test]
    fn missing_fields_refused() {
        assert!(parse_playtest("demo", &json!({})).is_err());
        assert!(parse_playtest("demo", &json!({"run": null})).is_err());
        let mut value = generated();
        value["steps"][0] = json!({"i": 0});
        assert!(parse_playtest("demo", &value).is_err());
    }
}
