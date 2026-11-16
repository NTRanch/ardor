//! The release-handoff state machine (the broadside verify
//! rule, mirrored for ardor).
//!
//! A handoff may only be **staged** from evidence gathered in
//! the same live session:
//!
//! 1. a playtest run for the same project completed
//!    ([`SessionEvidence::record_playtest`]), and
//! 2. the server's safety gates read **green** for that project
//!    in a snapshot taken in this session
//!    ([`SessionEvidence::record_gates`]).
//!
//! Staging produces a [`StagedHandoff`] — a plan, not a call.
//! Executing it is a separate, explicitly approved step
//! ([`execute`]) and re-checks that the staged evidence still
//! belongs to the facade's current session before the
//! `prepare_handoff` call is made. Server-side, ardor's own
//! gate re-verifies everything (channel, artifacts, accounts,
//! safety audit); this machine exists so the TUI never even
//! asks on stale or missing evidence.

use serde_json::{Value, json};

use crate::facade::{ArdorFacade, FacadeError};
use crate::gates::{GateSnapshot, GateVerdict};
use crate::qa::PlaytestRun;

/// Evidence gathered during one session.
#[derive(Debug, Clone, Default)]
pub struct SessionEvidence {
    playtest: Option<PlaytestRun>,
    gates: Option<GateSnapshot>,
}

impl SessionEvidence {
    /// A fresh, empty evidence set.
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a completed, validated playtest run.
    pub fn record_playtest(&mut self, run: PlaytestRun) {
        self.playtest = Some(run);
    }

    /// Record the latest gate snapshot (green or not).
    pub fn record_gates(&mut self, snapshot: GateSnapshot) {
        self.gates = Some(snapshot);
    }

    /// The recorded playtest run, if any.
    pub fn playtest(&self) -> Option<&PlaytestRun> {
        self.playtest.as_ref()
    }

    /// The recorded gate snapshot, if any.
    pub fn gates(&self) -> Option<&GateSnapshot> {
        self.gates.as_ref()
    }

    /// Drop all evidence (session changed / server restarted).
    pub fn clear(&mut self) {
        self.playtest = None;
        self.gates = None;
    }
}

/// What a handoff would call, once approved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HandoffRequest {
    /// Project slug.
    pub project: String,
    /// Channel id from ardor's registry.
    pub channel: String,
    /// Artifact records (`{name, sha256, kind}`), as the
    /// server's gate expects them.
    pub artifacts: Value,
}

impl HandoffRequest {
    /// The exact tool arguments staging would approve.
    pub fn tool_args(&self) -> Value {
        json!({
            "project": self.project,
            "channel": self.channel,
            "artifacts": self.artifacts,
        })
    }
}

/// A staged (planned, unexecuted) handoff plus the evidence it
/// was staged from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StagedHandoff {
    /// The request to execute on approval.
    pub request: HandoffRequest,
    /// Session the evidence belongs to.
    pub session: String,
    /// Hash of the playtest run that evidences this handoff.
    pub playtest_hash: String,
}

/// Why staging (or executing) a handoff was refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HandoffRefusal {
    /// No completed playtest run for this project this session.
    PlaytestMissing,
    /// No gate snapshot at all this session.
    GatesUnknown,
    /// Gates were read but are not green; carries failing names.
    GatesNotGreen(Vec<String>),
    /// Evidence belongs to a different session than the live one.
    SessionMismatch,
    /// The gate snapshot is for a different project.
    ProjectMismatch,
}

impl std::fmt::Display for HandoffRefusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HandoffRefusal::PlaytestMissing => {
                write!(
                    f,
                    "no completed playtest run for this project in this session"
                )
            }
            HandoffRefusal::GatesUnknown => {
                write!(f, "safety gates not read in this session (state unknown)")
            }
            HandoffRefusal::GatesNotGreen(names) => {
                write!(f, "safety gates not green; failing: {}", names.join(", "))
            }
            HandoffRefusal::SessionMismatch => {
                write!(f, "evidence belongs to a different session")
            }
            HandoffRefusal::ProjectMismatch => {
                write!(f, "gate snapshot is for a different project")
            }
        }
    }
}

/// Stage a handoff from session evidence. Pure: no facade
/// call happens here — staging is the plan-mode step.
pub fn stage(
    evidence: &SessionEvidence,
    current_session: &str,
    request: HandoffRequest,
) -> Result<StagedHandoff, HandoffRefusal> {
    let playtest = evidence
        .playtest()
        .filter(|run| run.project == request.project)
        .ok_or(HandoffRefusal::PlaytestMissing)?;
    let gates = evidence.gates().ok_or(HandoffRefusal::GatesUnknown)?;
    if gates.session != current_session {
        return Err(HandoffRefusal::SessionMismatch);
    }
    if gates.project != request.project {
        return Err(HandoffRefusal::ProjectMismatch);
    }
    match gates.verdict {
        GateVerdict::Green => Ok(StagedHandoff {
            playtest_hash: playtest.hash.clone(),
            session: current_session.to_string(),
            request,
        }),
        GateVerdict::Unknown => Err(HandoffRefusal::GatesUnknown),
        GateVerdict::Red => Err(HandoffRefusal::GatesNotGreen(gates.failing())),
    }
}

/// Execute a staged handoff after explicit approval. Refuses
/// (without calling) when the facade's session no longer
/// matches the staged evidence session.
pub fn execute(
    facade: &mut dyn ArdorFacade,
    staged: &StagedHandoff,
) -> Result<Value, ExecuteError> {
    if facade.session_id() != staged.session {
        return Err(ExecuteError::Refused(HandoffRefusal::SessionMismatch));
    }
    facade
        .call("prepare_handoff", &staged.request.tool_args())
        .map_err(ExecuteError::Facade)
}

/// Execution failure: a staging refusal, or the server call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecuteError {
    /// Refused before any call was made.
    Refused(HandoffRefusal),
    /// The server call itself failed.
    Facade(FacadeError),
}

impl std::fmt::Display for ExecuteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecuteError::Refused(r) => write!(f, "refused: {r}"),
            ExecuteError::Facade(e) => write!(f, "{e}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::facade::FakeFacade;
    use crate::gates::GateSnapshot;
    use serde_json::json;

    const HASH: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

    fn run(project: &str) -> PlaytestRun {
        PlaytestRun {
            project: project.to_string(),
            id: "run-1".to_string(),
            seed: 1,
            hash: HASH.to_string(),
            steps_count: 3,
            stop_reason: "terminal".to_string(),
            truncated: false,
            step_nodes: Vec::new(),
        }
    }

    fn green(session: &str, project: &str) -> GateSnapshot {
        GateSnapshot::from_audit(
            session,
            &json!({"project": project, "audit": {"ok": true, "checks": [
                {"check": "safety_surface:stop", "ok": true, "detail": "ok"}
            ]}}),
        )
        .expect("green audit")
    }

    fn request() -> HandoffRequest {
        HandoffRequest {
            project: "demo".to_string(),
            channel: "itch-io".to_string(),
            artifacts: json!([{"name": "build.zip", "sha256": HASH, "kind": "build"}]),
        }
    }

    fn evidence_green() -> SessionEvidence {
        let mut ev = SessionEvidence::new();
        ev.record_playtest(run("demo"));
        ev.record_gates(green("s1", "demo"));
        ev
    }

    #[test]
    fn stage_succeeds_on_full_green_evidence() {
        let staged = stage(&evidence_green(), "s1", request()).expect("stage");
        assert_eq!(staged.playtest_hash, HASH);
        assert_eq!(staged.session, "s1");
    }

    #[test]
    fn stage_refuses_without_playtest() {
        let mut ev = SessionEvidence::new();
        ev.record_gates(green("s1", "demo"));
        assert_eq!(
            stage(&ev, "s1", request()),
            Err(HandoffRefusal::PlaytestMissing)
        );
    }

    #[test]
    fn stage_refuses_without_gates() {
        let mut ev = SessionEvidence::new();
        ev.record_playtest(run("demo"));
        assert_eq!(
            stage(&ev, "s1", request()),
            Err(HandoffRefusal::GatesUnknown)
        );
    }

    #[test]
    fn stage_refuses_on_red_gates() {
        let mut ev = SessionEvidence::new();
        ev.record_playtest(run("demo"));
        let red = GateSnapshot::from_audit(
            "s1",
            &json!({"project": "demo", "audit": {"ok": false, "checks": [
                {"check": "age_gate_present", "ok": false, "detail": "missing"}
            ]}}),
        )
        .expect("red audit");
        ev.record_gates(red);
        assert_eq!(
            stage(&ev, "s1", request()),
            Err(HandoffRefusal::GatesNotGreen(vec![
                "age_gate_present".to_string()
            ]))
        );
    }

    #[test]
    fn stage_refuses_on_stale_session_evidence() {
        let ev = evidence_green();
        assert_eq!(
            stage(&ev, "s2", request()),
            Err(HandoffRefusal::SessionMismatch)
        );
    }

    #[test]
    fn stage_refuses_on_project_mismatch() {
        let mut ev = SessionEvidence::new();
        ev.record_playtest(run("demo"));
        ev.record_gates(green("s1", "other"));
        assert_eq!(
            stage(&ev, "s1", request()),
            Err(HandoffRefusal::ProjectMismatch)
        );
        // A playtest for a different project is no evidence.
        let mut ev2 = SessionEvidence::new();
        ev2.record_playtest(run("other"));
        ev2.record_gates(green("s1", "demo"));
        assert_eq!(
            stage(&ev2, "s1", request()),
            Err(HandoffRefusal::PlaytestMissing)
        );
    }

    #[test]
    fn execute_calls_prepare_handoff_once() {
        let mut ev = SessionEvidence::new();
        ev.record_playtest(run("demo"));
        ev.record_gates(green("fake-session-1", "demo"));
        let staged = stage(&ev, "fake-session-1", request()).expect("stage");
        let mut fake = FakeFacade::new();
        fake.script_call("prepare_handoff", Ok(json!({"handoff": {"id": "h1"}})));
        let out = execute(&mut fake, &staged).expect("execute");
        assert_eq!(out["handoff"]["id"], "h1");
        assert_eq!(fake.calls().len(), 1);
        assert_eq!(fake.calls()[0].0, "prepare_handoff");
        assert_eq!(fake.calls()[0].1["channel"], "itch-io");
    }

    #[test]
    fn execute_refuses_after_session_change_without_calling() {
        let staged = stage(&evidence_green(), "s1", request()).expect("stage");
        let mut fake = FakeFacade::new(); // session fake-session-1 != s1
        let err = execute(&mut fake, &staged).expect_err("session changed");
        assert_eq!(err, ExecuteError::Refused(HandoffRefusal::SessionMismatch));
        assert!(fake.calls().is_empty(), "no call may happen on refusal");
    }
}
