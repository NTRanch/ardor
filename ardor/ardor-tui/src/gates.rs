//! Safety-gate state, parsed — never asserted — from the
//! server's own tool results.
//!
//! Sources: `audit_safety` returns `{project, audit: {ok,
//! checks: [{check, ok, detail}]}}`; `publish_preflight`
//! returns `{checks: [{check, ok, detail}], ..}` with no single
//! `ok`, so the verdict there is the conjunction of its checks.
//! Anything else (missing fields, wrong types, transport
//! failure) yields [`GateVerdict::Unknown`] or a parse error —
//! the panel must never render a green the server did not say.
//!
//! Staleness rule: a snapshot is bound to the session and
//! project that produced it. When a refresh fails, callers
//! replace the snapshot with an `Unknown` one (see
//! [`crate::app::App::refresh_safety`]); last-green is never
//! carried forward as if current.

use serde_json::Value;

/// What the server's gates say, as last successfully read.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GateVerdict {
    /// No trustworthy reading (never read, or the read failed).
    Unknown,
    /// The server reported every check passing.
    Green,
    /// The server reported at least one failing check.
    Red,
}

impl GateVerdict {
    /// Short label for panels.
    pub fn label(self) -> &'static str {
        match self {
            GateVerdict::Unknown => "UNKNOWN",
            GateVerdict::Green => "GREEN",
            GateVerdict::Red => "RED",
        }
    }
}

/// One named check from a server gate result.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GateCheck {
    /// The server's check name (e.g. `safety_surface:stop`).
    pub name: String,
    /// The server's verdict for this check.
    pub ok: bool,
    /// The server's detail text.
    pub detail: String,
}

/// A gate reading bound to one session and one project.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GateSnapshot {
    /// Session that produced this reading.
    pub session: String,
    /// Project the reading is about.
    pub project: String,
    /// Tool the reading came from (`audit_safety`, …).
    pub source_tool: String,
    /// Conjunction verdict.
    pub verdict: GateVerdict,
    /// The server's checks, verbatim.
    pub checks: Vec<GateCheck>,
    /// Why the verdict is Unknown, when it is.
    pub unknown_reason: Option<String>,
}

impl GateSnapshot {
    /// An explicit unknown reading (refresh failed / not read).
    pub fn unknown(session: &str, project: &str, reason: &str) -> Self {
        GateSnapshot {
            session: session.to_string(),
            project: project.to_string(),
            source_tool: String::new(),
            verdict: GateVerdict::Unknown,
            checks: Vec::new(),
            unknown_reason: Some(reason.to_string()),
        }
    }

    /// Parse an `audit_safety` result.
    pub fn from_audit(session: &str, value: &Value) -> Result<Self, String> {
        let project = value
            .get("project")
            .and_then(Value::as_str)
            .ok_or("audit_safety result missing project")?;
        let audit = value
            .get("audit")
            .ok_or("audit_safety result missing audit")?;
        let declared_ok = audit
            .get("ok")
            .and_then(Value::as_bool)
            .ok_or("audit missing ok flag")?;
        let checks = parse_checks(audit)?;
        let all_ok = checks.iter().all(|c| c.ok);
        // The server's own `ok` and the conjunction must agree;
        // a disagreement is malformed, not green.
        if declared_ok != all_ok {
            return Err("audit ok flag disagrees with its checks".to_string());
        }
        Ok(GateSnapshot {
            session: session.to_string(),
            project: project.to_string(),
            source_tool: "audit_safety".to_string(),
            verdict: if all_ok {
                GateVerdict::Green
            } else {
                GateVerdict::Red
            },
            checks,
            unknown_reason: None,
        })
    }

    /// Parse a `publish_preflight` result (checks only; the
    /// verdict is their conjunction).
    pub fn from_preflight(session: &str, project: &str, value: &Value) -> Result<Self, String> {
        let checks = parse_checks(value)?;
        if checks.is_empty() {
            return Err("preflight carried no checks".to_string());
        }
        let all_ok = checks.iter().all(|c| c.ok);
        Ok(GateSnapshot {
            session: session.to_string(),
            project: project.to_string(),
            source_tool: "publish_preflight".to_string(),
            verdict: if all_ok {
                GateVerdict::Green
            } else {
                GateVerdict::Red
            },
            checks,
            unknown_reason: None,
        })
    }

    /// Names of the failing checks (empty unless Red).
    pub fn failing(&self) -> Vec<String> {
        self.checks
            .iter()
            .filter(|c| !c.ok)
            .map(|c| c.name.clone())
            .collect()
    }

    /// True when this reading came from `session` for `project`.
    pub fn is_current_for(&self, session: &str, project: &str) -> bool {
        self.session == session && self.project == project && self.verdict != GateVerdict::Unknown
    }
}

fn parse_checks(container: &Value) -> Result<Vec<GateCheck>, String> {
    let raw = container
        .get("checks")
        .and_then(Value::as_array)
        .ok_or("gate result missing checks array")?;
    let mut checks = Vec::with_capacity(raw.len());
    for entry in raw {
        let name = entry
            .get("check")
            .and_then(Value::as_str)
            .ok_or("gate check missing name")?;
        let ok = entry
            .get("ok")
            .and_then(Value::as_bool)
            .ok_or("gate check missing ok flag")?;
        let detail = entry
            .get("detail")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        checks.push(GateCheck {
            name: name.to_string(),
            ok,
            detail,
        });
    }
    Ok(checks)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn green_audit() -> Value {
        json!({
            "project": "demo",
            "audit": {"ok": true, "checks": [
                {"check": "safety_surface:stop", "ok": true, "detail": "present, enabled, ungated"},
                {"check": "age_gate_present", "ok": true, "detail": "age-gate pattern recorded"}
            ]}
        })
    }

    #[test]
    fn audit_green_parses() {
        let snap = GateSnapshot::from_audit("s1", &green_audit()).expect("parse");
        assert_eq!(snap.verdict, GateVerdict::Green);
        assert_eq!(snap.checks.len(), 2);
        assert!(snap.is_current_for("s1", "demo"));
        assert!(!snap.is_current_for("s2", "demo"));
    }

    #[test]
    fn audit_red_names_failing() {
        let mut value = green_audit();
        value["audit"]["ok"] = json!(false);
        value["audit"]["checks"][0]["ok"] = json!(false);
        let snap = GateSnapshot::from_audit("s1", &value).expect("parse");
        assert_eq!(snap.verdict, GateVerdict::Red);
        assert_eq!(snap.failing(), vec!["safety_surface:stop".to_string()]);
    }

    #[test]
    fn audit_ok_disagreeing_with_checks_is_malformed() {
        let mut value = green_audit();
        value["audit"]["checks"][1]["ok"] = json!(false);
        assert!(GateSnapshot::from_audit("s1", &value).is_err());
    }

    #[test]
    fn malformed_gate_results_are_refused() {
        assert!(GateSnapshot::from_audit("s1", &json!({})).is_err());
        assert!(GateSnapshot::from_audit("s1", &json!({"project": "x"})).is_err());
        assert!(
            GateSnapshot::from_audit("s1", &json!({"project": "x", "audit": {"ok": true}}))
                .is_err()
        );
        assert!(GateSnapshot::from_preflight("s1", "x", &json!({"checks": []})).is_err());
    }

    #[test]
    fn preflight_conjunction() {
        let value = json!({"checks": [
            {"check": "channel_known", "ok": true, "detail": "itch"},
            {"check": "safety_audit", "ok": false, "detail": "see audit_safety"}
        ]});
        let snap = GateSnapshot::from_preflight("s1", "demo", &value).expect("parse");
        assert_eq!(snap.verdict, GateVerdict::Red);
        assert_eq!(snap.failing(), vec!["safety_audit".to_string()]);
    }

    #[test]
    fn unknown_snapshot_is_never_current() {
        let snap = GateSnapshot::unknown("s1", "demo", "server dead");
        assert_eq!(snap.verdict, GateVerdict::Unknown);
        assert!(!snap.is_current_for("s1", "demo"));
    }
}
