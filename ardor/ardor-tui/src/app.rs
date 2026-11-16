//! The console state machine: screens, workflow staging and
//! execution, gate refresh, and the shared execute path behind
//! both the full-screen TUI and line mode (DESIGN §D2 — line
//! mode is not a second implementation).
//!
//! Invariants:
//! - In plan mode a workflow run **stages** the exact tool +
//!   args and makes no facade call; only [`App::approve`]
//!   delivers it. (Proven against [`crate::facade::FakeFacade`]'s
//!   call record in the tests below.)
//! - `prepare_handoff` additionally passes the
//!   [`crate::handoff`] evidence machine at stage time and at
//!   execution time.
//! - A failed gate refresh replaces the snapshot with an
//!   Unknown one: last-green is never presented as current.

use std::collections::VecDeque;

use serde_json::{Value, json};

use crate::facade::ArdorFacade;
use crate::gates::GateSnapshot;
use crate::handoff::{self, HandoffRequest, SessionEvidence, StagedHandoff};
use crate::qa;
use crate::triggers::{EngineAction, Event, Proposal, TriggerEngine};
use crate::workflows::{self, Mode, WorkflowDef};
use crate::{LOG_LINES_MAX, PROGRESS_EVENTS_MAX};

/// The five screens (no Media — see the crate docs).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    /// Version, status, title list.
    Overview,
    /// Authoring / QA / release workflow list.
    Workflows,
    /// Safety-gate panel for the selected project.
    Safety,
    /// Log / Live event view.
    Log,
    /// Fleet panel: trigger rules + audit tail.
    Fleet,
}

impl Screen {
    /// All screens in tab order.
    pub const ALL: [Screen; 5] = [
        Screen::Overview,
        Screen::Workflows,
        Screen::Safety,
        Screen::Log,
        Screen::Fleet,
    ];

    /// Short label.
    pub fn label(self) -> &'static str {
        match self {
            Screen::Overview => "Overview",
            Screen::Workflows => "Workflows",
            Screen::Safety => "Safety gates",
            Screen::Log => "Log/Live",
            Screen::Fleet => "Fleet",
        }
    }
}

/// One project row from `list_projects`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectRow {
    /// Project slug (the tool argument).
    pub slug: String,
    /// Display name.
    pub name: String,
    /// Engine string.
    pub engine: String,
    /// Content rating.
    pub rating: String,
}

/// A staged (planned, unexecuted) call.
#[derive(Debug, Clone)]
pub struct StagedCall {
    /// Workflow id that staged it.
    pub workflow_id: String,
    /// Tool that approval would call.
    pub tool: String,
    /// Exact arguments approval would send.
    pub args: Value,
    /// Handoff evidence binding, when the tool is
    /// `prepare_handoff`.
    pub handoff: Option<StagedHandoff>,
}

/// The console application over one facade session.
pub struct App<F: ArdorFacade> {
    /// The server facade (real stdio client or fake).
    pub facade: F,
    /// Current screen.
    pub screen: Screen,
    /// Requested run mode (plan-only workflows ignore Auto).
    pub mode: Mode,
    /// Last `server/discover` status payload.
    pub server_status: Option<Value>,
    /// Tool count from the server's own list.
    pub tool_count: Option<usize>,
    /// Known projects.
    pub projects: Vec<ProjectRow>,
    /// Selected project slug.
    pub selected_project: Option<String>,
    /// Selected workflow index into the catalogue.
    pub selected_workflow: usize,
    /// Latest gate snapshot (Unknown after a failed refresh).
    pub gates: Option<GateSnapshot>,
    /// Same-session evidence for the handoff machine.
    pub evidence: SessionEvidence,
    /// Bounded Log/Live buffer.
    pub log: VecDeque<String>,
    /// The staged call awaiting approval, if any.
    pub staged: Option<StagedCall>,
    /// Trigger proposals awaiting a human decision.
    pub proposals: Vec<Proposal>,
    /// The trigger engine (rules + audit).
    pub engine: TriggerEngine,
    /// Channel chosen for proposal approval.
    pub handoff_channel: Option<String>,
    /// Set by `quit` / `q`; the shells own the exit.
    pub should_quit: bool,
    /// Last error line, shown on Overview.
    pub last_error: Option<String>,
}

impl<F: ArdorFacade> App<F> {
    /// A new app in plan mode on the Overview screen.
    pub fn new(facade: F, engine: TriggerEngine) -> Self {
        App {
            facade,
            screen: Screen::Overview,
            mode: Mode::Plan,
            server_status: None,
            tool_count: None,
            projects: Vec::new(),
            selected_project: None,
            selected_workflow: 0,
            gates: None,
            evidence: SessionEvidence::new(),
            log: VecDeque::new(),
            staged: None,
            proposals: Vec::new(),
            engine,
            handoff_channel: None,
            should_quit: false,
            last_error: None,
        }
    }

    /// Append one line to the bounded log buffer.
    pub fn log_line(&mut self, line: impl Into<String>) {
        if self.log.len() >= LOG_LINES_MAX {
            self.log.pop_front();
        }
        self.log.push_back(line.into());
    }

    /// Refresh status, tool count, and the project list.
    pub fn refresh_overview(&mut self) -> Vec<String> {
        let mut out = Vec::new();
        match self.facade.status() {
            Ok(status) => {
                self.server_status = Some(status);
                out.push("status: connected".to_string());
            }
            Err(e) => return self.fail(format!("status failed: {e}")),
        }
        match self.facade.tools() {
            Ok(tools) => {
                self.tool_count = tools
                    .get("tools")
                    .and_then(Value::as_array)
                    .map(|a| a.len());
            }
            Err(e) => return self.fail(format!("tools/list failed: {e}")),
        }
        match self.facade.call("list_projects", &json!({})) {
            Ok(value) => {
                self.projects = parse_projects(&value);
                if self.selected_project.is_none()
                    && let Some(first) = self.projects.first()
                {
                    self.selected_project = Some(first.slug.clone());
                }
                out.push(format!("projects: {}", self.projects.len()));
            }
            Err(e) => return self.fail(format!("list_projects failed: {e}")),
        }
        self.last_error = None;
        out
    }

    fn fail(&mut self, line: String) -> Vec<String> {
        self.last_error = Some(line.clone());
        self.log_line(line.clone());
        vec![line]
    }

    /// Select the working project by slug.
    pub fn select_project(&mut self, slug: &str) -> Vec<String> {
        if self.projects.iter().any(|p| p.slug == slug) {
            self.selected_project = Some(slug.to_string());
            vec![format!("selected project: {slug}")]
        } else {
            vec![format!("unknown project slug: {slug}")]
        }
    }

    /// Refresh the gate snapshot for the selected project from
    /// the server's `audit_safety`. Any failure — transport or
    /// malformed payload — replaces the snapshot with Unknown.
    pub fn refresh_safety(&mut self) -> Vec<String> {
        let Some(project) = self.selected_project.clone() else {
            return vec!["no project selected".to_string()];
        };
        let session = self.facade.session_id().to_string();
        match self
            .facade
            .call("audit_safety", &json!({"project": project}))
        {
            Ok(value) => match GateSnapshot::from_audit(&session, &value) {
                Ok(snap) => {
                    let line = format!(
                        "gates for {project}: {} ({} checks, via audit_safety)",
                        snap.verdict.label(),
                        snap.checks.len()
                    );
                    self.evidence.record_gates(snap.clone());
                    self.gates = Some(snap);
                    self.log_line(line.clone());
                    vec![line]
                }
                Err(reason) => self.gates_unknown(&project, &format!("malformed audit: {reason}")),
            },
            Err(e) => self.gates_unknown(&project, &format!("audit_safety failed: {e}")),
        }
    }

    fn gates_unknown(&mut self, project: &str, reason: &str) -> Vec<String> {
        let session = self.facade.session_id().to_string();
        let snap = GateSnapshot::unknown(&session, project, reason);
        self.evidence.record_gates(snap.clone());
        self.gates = Some(snap);
        let line = format!("gates for {project}: UNKNOWN ({reason})");
        self.log_line(line.clone());
        vec![line]
    }

    /// Run `publish_preflight` for a channel and adopt its
    /// checks as the current gate snapshot.
    pub fn refresh_preflight(&mut self, channel: &str) -> Vec<String> {
        let Some(project) = self.selected_project.clone() else {
            return vec!["no project selected".to_string()];
        };
        let session = self.facade.session_id().to_string();
        let args = json!({"project": project, "channel": channel});
        match self.facade.call("publish_preflight", &args) {
            Ok(value) => match GateSnapshot::from_preflight(&session, &project, &value) {
                Ok(snap) => {
                    let line = format!(
                        "preflight {project} -> {channel}: {} ({} checks)",
                        snap.verdict.label(),
                        snap.checks.len()
                    );
                    self.evidence.record_gates(snap.clone());
                    self.gates = Some(snap);
                    self.log_line(line.clone());
                    vec![line]
                }
                Err(reason) => {
                    self.gates_unknown(&project, &format!("malformed preflight: {reason}"))
                }
            },
            Err(e) => self.gates_unknown(&project, &format!("publish_preflight failed: {e}")),
        }
    }

    /// Stage or execute one workflow. In plan mode (or for a
    /// plan-only workflow) this makes **no** facade call.
    pub fn run_workflow(&mut self, id: &str, extra: &Value) -> Vec<String> {
        let Some(def) = workflows::by_id(id) else {
            return vec![format!("unknown workflow: {id}")];
        };
        let args = match self.build_args(&def, extra) {
            Ok(args) => args,
            Err(line) => return vec![line],
        };
        if workflows::effective_mode(&def, self.mode) == Mode::Plan {
            return self.stage_call(&def, args);
        }
        self.deliver(def.tool, &args)
    }

    fn build_args(&self, def: &WorkflowDef, extra: &Value) -> Result<Value, String> {
        let mut args = json!({});
        if def.needs_project {
            let project = self.selected_project.clone().ok_or("no project selected")?;
            args["project"] = json!(project);
        }
        if !extra.is_null() {
            let obj = extra
                .as_object()
                .ok_or("extra arguments must be a JSON object")?;
            for (key, value) in obj {
                args[key] = value.clone();
            }
        }
        Ok(args)
    }

    fn stage_call(&mut self, def: &WorkflowDef, args: Value) -> Vec<String> {
        if def.tool == "prepare_handoff" {
            return self.stage_handoff(def, args);
        }
        self.staged = Some(StagedCall {
            workflow_id: def.id.to_string(),
            tool: def.tool.to_string(),
            args: args.clone(),
            handoff: None,
        });
        let line = format!("staged (plan): {} {}", def.tool, compact(&args));
        self.log_line(line.clone());
        vec![
            line,
            "approve with: approve   abort with: abort".to_string(),
        ]
    }

    fn stage_handoff(&mut self, def: &WorkflowDef, args: Value) -> Vec<String> {
        let request = HandoffRequest {
            project: args
                .get("project")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
            channel: args
                .get("channel")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
            artifacts: args.get("artifacts").cloned().unwrap_or(json!([])),
        };
        if request.channel.is_empty() {
            return vec!["prepare_handoff needs a channel argument".to_string()];
        }
        let session = self.facade.session_id().to_string();
        match handoff::stage(&self.evidence, &session, request) {
            Ok(staged) => {
                self.staged = Some(StagedCall {
                    workflow_id: def.id.to_string(),
                    tool: def.tool.to_string(),
                    args: staged.request.tool_args(),
                    handoff: Some(staged),
                });
                let line =
                    "staged (plan): prepare_handoff — evidence green in this session".to_string();
                self.log_line(line.clone());
                vec![
                    line,
                    "approve with: approve   abort with: abort".to_string(),
                ]
            }
            Err(refusal) => {
                let line = format!("handoff refused: {refusal}");
                self.log_line(line.clone());
                vec![line]
            }
        }
    }

    /// Approve the staged call: the first moment a plan-mode
    /// workflow touches the facade.
    pub fn approve(&mut self) -> Vec<String> {
        let Some(staged) = self.staged.take() else {
            return vec!["nothing staged".to_string()];
        };
        if let Some(handoff) = &staged.handoff {
            return match handoff::execute(&mut self.facade, handoff) {
                Ok(value) => {
                    let line = format!("prepare_handoff ok: {}", compact(&value));
                    self.log_line(line.clone());
                    vec![line]
                }
                Err(e) => self.fail(format!("prepare_handoff failed: {e}")),
            };
        }
        self.deliver(&staged.tool, &staged.args)
    }

    /// Abort the staged call without executing it.
    pub fn abort(&mut self) -> Vec<String> {
        match self.staged.take() {
            Some(staged) => {
                let line = format!("aborted staged call: {}", staged.tool);
                self.log_line(line.clone());
                vec![line]
            }
            None => vec!["nothing staged".to_string()],
        }
    }

    /// Approve one trigger proposal by index: stages the
    /// handoff (still plan mode — execution needs `approve`).
    pub fn approve_proposal(&mut self, index: usize) -> Vec<String> {
        if index >= self.proposals.len() {
            return vec![format!("no proposal #{index}")];
        }
        let proposal = self.proposals.remove(index);
        let Some(channel) = self.handoff_channel.clone() else {
            return vec![
                "set a channel first: channel <id> (proposal kept? no — re-run the playtest)"
                    .to_string(),
            ];
        };
        let def = workflows::by_id("prepare-handoff").expect("catalogue carries handoff");
        let args = json!({
            "project": proposal.project,
            "channel": channel,
            "artifacts": [],
        });
        let mut out = vec![format!(
            "proposal {} accepted into staging (lane={} label={})",
            proposal.rule_id, proposal.origin_lane, proposal.content_label
        )];
        out.extend(self.stage_call(&def, args));
        out
    }

    /// Deliver one tool call and post-process its result:
    /// playtest runs are validated, recorded as evidence, and
    /// fed to the trigger engine; everything else is logged.
    pub fn deliver(&mut self, tool: &str, args: &Value) -> Vec<String> {
        self.log_line(format!("call {tool} {}", compact(args)));
        match self.facade.call(tool, args) {
            Ok(value) => {
                if tool == "run_playtest" || tool == "generate_playtest" {
                    return self.on_playtest_result(tool, args, &value);
                }
                let line = format!("{tool} ok: {}", compact(&value));
                self.log_line(line.clone());
                vec![line]
            }
            Err(e) => self.fail(format!("{tool} failed: {e}")),
        }
    }

    fn on_playtest_result(&mut self, _tool: &str, args: &Value, value: &Value) -> Vec<String> {
        let project = args
            .get("project")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let run = match qa::parse_playtest(&project, value) {
            Ok(run) => run,
            Err(reason) => {
                // Malformed QA results are refused: no evidence,
                // no trigger, no completion claim.
                return self.fail(format!("malformed playtest result refused: {reason}"));
            }
        };
        let mut out = playtest_progress_lines(&run);
        for line in &out {
            self.log_line(line.clone());
        }
        self.evidence.record_playtest(run.clone());
        let gates = self.evidence.gates().cloned();
        let event = Event::PlaytestCompleted { run, depth: 0 };
        for action in self.engine.on_event(&event, gates.as_ref()) {
            match action {
                EngineAction::Proposed(proposal) => {
                    out.push(format!(
                        "trigger proposal: {} -> {} (plan mode; approve-proposal to stage)",
                        proposal.rule_id, proposal.target_tool
                    ));
                    self.proposals.push(proposal);
                }
                EngineAction::Dropped(msg) | EngineAction::Refused(msg) => out.push(msg),
            }
        }
        out
    }
}

/// Parse the `list_projects` payload into rows.
pub fn parse_projects(value: &Value) -> Vec<ProjectRow> {
    let mut rows = Vec::new();
    if let Some(list) = value.get("projects").and_then(Value::as_array) {
        for entry in list {
            let text = |key: &str| {
                entry
                    .get(key)
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string()
            };
            rows.push(ProjectRow {
                slug: text("slug"),
                name: text("name"),
                engine: text("engine"),
                rating: text("rating"),
            });
        }
    }
    rows
}

/// Progress lines for one playtest run: one line per step at a
/// stride chosen so the total stays within
/// [`PROGRESS_EVENTS_MAX`], then the completion line carrying
/// the replay hash.
pub fn playtest_progress_lines(run: &qa::PlaytestRun) -> Vec<String> {
    let mut out = Vec::new();
    let total = run.step_nodes.len();
    if total > 0 {
        let stride = total.div_ceil(PROGRESS_EVENTS_MAX).max(1);
        for (i, node) in run.step_nodes.iter().enumerate() {
            if i % stride == 0 {
                out.push(format!("  step {i}: {node}"));
            }
        }
    }
    out.push(format!(
        "playtest completed: steps={} stop={} hash={}",
        run.steps_count, run.stop_reason, run.hash
    ));
    out
}

/// Compact one-line JSON for logs and staged-call display.
pub fn compact(value: &Value) -> String {
    crate::truncate_chars(&value.to_string(), 300)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::facade::{FacadeError, FakeFacade};
    use crate::gates::GateVerdict;
    use crate::triggers::parse_rules;

    const HASH: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

    fn green_audit() -> Value {
        json!({"project": "demo", "audit": {"ok": true, "checks": [
            {"check": "safety_surface:stop", "ok": true, "detail": "present, enabled, ungated"},
            {"check": "age_gate_present", "ok": true, "detail": "recorded"}
        ]}})
    }

    fn recorded_run() -> Value {
        json!({"project": "demo", "run": {
            "id": "run-1", "seed": 1, "steps_count": 4, "truncated": false,
            "stop_reason": "terminal", "hash": HASH
        }})
    }

    fn app_with_fake() -> App<FakeFacade> {
        let mut fake = FakeFacade::new();
        fake.script_status(Ok(json!({
            "server": "ardor", "server_version": "1.0.1",
            "supported_versions": ["2026-07-28"], "protocol": "2026-07-28"
        })));
        fake.script_tools(Ok(json!({"tools": [{"name": "a"}, {"name": "b"}]})));
        fake.script_call(
            "list_projects",
            Ok(json!({"projects": [
                {"slug": "demo", "name": "Demo", "engine": "renpy", "rating": "adult"}
            ], "count": 1})),
        );
        let engine =
            TriggerEngine::new(parse_rules(crate::triggers::EMBEDDED_RULES).expect("rules"));
        let mut app = App::new(fake, engine);
        app.refresh_overview();
        app
    }

    #[test]
    fn overview_refresh_selects_first_project() {
        let app = app_with_fake();
        assert_eq!(app.selected_project.as_deref(), Some("demo"));
        assert_eq!(app.tool_count, Some(2));
        assert_eq!(app.projects.len(), 1);
    }

    #[test]
    fn plan_mode_makes_no_call_until_approval() {
        let mut app = app_with_fake();
        app.facade.script_call("run_playtest", Ok(recorded_run()));
        let out = app.run_workflow("run-playtest", &json!({}));
        assert!(out[0].contains("staged"), "{out:?}");
        assert!(
            app.facade.calls().iter().all(|(t, _)| t != "run_playtest"),
            "plan mode must not call: {:?}",
            app.facade.calls()
        );
        assert!(app.staged.is_some());
        let out = app.approve();
        assert!(
            out.iter().any(|l| l.contains("playtest completed")),
            "{out:?}"
        );
        let calls: Vec<&str> = app.facade.calls().iter().map(|(t, _)| t.as_str()).collect();
        assert_eq!(calls, vec!["list_projects", "run_playtest"]);
        assert!(app.evidence.playtest().is_some());
    }

    #[test]
    fn approval_then_abort_makes_no_call() {
        let mut app = app_with_fake();
        let _ = app.run_workflow("run-playtest", &json!({}));
        let out = app.abort();
        assert!(out[0].contains("aborted"));
        assert!(app.staged.is_none());
        let out = app.approve();
        assert_eq!(out, vec!["nothing staged".to_string()]);
        assert!(
            app.facade.calls().iter().all(|(t, _)| t != "run_playtest"),
            "abort must leave the server untouched"
        );
    }

    #[test]
    fn auto_mode_executes_reads_immediately() {
        let mut app = app_with_fake();
        app.mode = Mode::Auto;
        app.facade
            .script_call("validate_story", Ok(json!({"ok": true})));
        let out = app.run_workflow("validate-story", &json!({}));
        assert!(out[0].contains("validate_story ok"), "{out:?}");
        assert!(app.staged.is_none());
    }

    #[test]
    fn handoff_without_green_gates_is_refused_at_staging() {
        let mut app = app_with_fake();
        // Playtest evidence exists, gates were never read.
        app.facade.script_call("run_playtest", Ok(recorded_run()));
        app.mode = Mode::Auto;
        let _ = app.run_workflow("run-playtest", &json!({}));
        assert!(app.evidence.playtest().is_some());
        app.mode = Mode::Plan;
        let out = app.run_workflow(
            "prepare-handoff",
            &json!({"channel": "itch-io", "artifacts": []}),
        );
        assert!(out[0].contains("handoff refused"), "{out:?}");
        assert!(app.staged.is_none());
        assert!(
            app.facade
                .calls()
                .iter()
                .all(|(t, _)| t != "prepare_handoff"),
            "a refused handoff must never reach the server"
        );
    }

    #[test]
    fn handoff_full_path_stage_then_approve() {
        let mut app = app_with_fake();
        app.facade.script_call("audit_safety", Ok(green_audit()));
        app.facade.script_call("run_playtest", Ok(recorded_run()));
        app.facade
            .script_call("prepare_handoff", Ok(json!({"handoff": {"id": "h1"}})));
        let _ = app.refresh_safety();
        assert_eq!(
            app.gates.as_ref().map(|g| g.verdict),
            Some(GateVerdict::Green)
        );
        app.mode = Mode::Auto;
        let _ = app.run_workflow("run-playtest", &json!({}));
        // The trigger fired a plan-mode proposal on green gates.
        assert_eq!(app.proposals.len(), 1);
        app.mode = Mode::Plan;
        let out = app.run_workflow(
            "prepare-handoff",
            &json!({"channel": "itch-io", "artifacts": [
                {"name": "b.zip", "sha256": HASH, "kind": "build"}
            ]}),
        );
        assert!(out[0].contains("staged"), "{out:?}");
        let out = app.approve();
        assert!(out[0].contains("prepare_handoff ok"), "{out:?}");
        assert!(
            app.facade
                .calls()
                .iter()
                .any(|(t, _)| t == "prepare_handoff")
        );
    }

    #[test]
    fn gate_staleness_server_death_turns_unknown_not_last_green() {
        let mut app = app_with_fake();
        app.facade.script_call("audit_safety", Ok(green_audit()));
        let _ = app.refresh_safety();
        assert_eq!(
            app.gates.as_ref().map(|g| g.verdict),
            Some(GateVerdict::Green)
        );
        app.facade.set_dead(true);
        let out = app.refresh_safety();
        assert!(out[0].contains("UNKNOWN"), "{out:?}");
        assert_eq!(
            app.gates.as_ref().map(|g| g.verdict),
            Some(GateVerdict::Unknown),
            "a dead server must read unknown, never last-green"
        );
        // And staging a handoff now refuses.
        let out = app.run_workflow(
            "prepare-handoff",
            &json!({"channel": "itch-io", "artifacts": []}),
        );
        assert!(out[0].contains("handoff refused"), "{out:?}");
    }

    #[test]
    fn malformed_playtest_result_is_refused_and_fires_nothing() {
        let mut app = app_with_fake();
        app.facade.script_call("audit_safety", Ok(green_audit()));
        let _ = app.refresh_safety();
        app.facade.script_call(
            "run_playtest",
            Ok(json!({"project": "demo", "run": {
                "id": "run-9", "seed": 1, "steps_count": 4,
                "stop_reason": "terminal", "hash": "not-a-hash"
            }})),
        );
        app.mode = Mode::Auto;
        let out = app.run_workflow("run-playtest", &json!({}));
        assert!(
            out[0].contains("malformed playtest result refused"),
            "{out:?}"
        );
        assert!(app.evidence.playtest().is_none());
        assert!(app.proposals.is_empty(), "malformed runs never trigger");
    }

    #[test]
    fn oversized_and_wrong_type_args_are_refused() {
        let mut app = app_with_fake();
        let out = app.run_workflow("validate-story", &json!([1, 2, 3]));
        assert!(out[0].contains("JSON object"), "{out:?}");
        let out = app.run_workflow("no-such-workflow", &json!({}));
        assert!(out[0].contains("unknown workflow"));
    }

    #[test]
    fn facade_tool_error_surfaces_typed() {
        let mut app = app_with_fake();
        app.mode = Mode::Auto;
        app.facade.script_call(
            "validate_story",
            Err(FacadeError::Tool {
                code: "not_found".to_string(),
                message: "no project".to_string(),
            }),
        );
        let out = app.run_workflow("validate-story", &json!({}));
        assert!(out[0].contains("not_found"), "{out:?}");
    }
}
