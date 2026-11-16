//! Screen data shaping: pure line builders over [`App`]
//! state. Both shells render from these lines — the ratatui
//! panels wrap them in widgets, line mode prints them — so
//! the two modes can never diverge in what they claim.

use serde_json::Value;

use crate::app::{App, Screen};
use crate::facade::ArdorFacade;
use crate::gates::GateVerdict;
use crate::workflows::{self, Mode};
use crate::{TUI_VERSION, truncate_chars};

/// Lines for whichever screen is current.
pub fn screen_lines<F: ArdorFacade>(app: &App<F>) -> Vec<String> {
    match app.screen {
        Screen::Overview => overview_lines(app),
        Screen::Workflows => workflows_lines(app),
        Screen::Safety => safety_lines(app),
        Screen::Log => log_lines(app),
        Screen::Fleet => fleet_lines(app),
    }
}

/// Overview: version, status, gate summary, title list.
pub fn overview_lines<F: ArdorFacade>(app: &App<F>) -> Vec<String> {
    let mut out = vec![format!(
        "ardor-tui {TUI_VERSION} — maker console for ardor (origin_lane=ntranch, content_label=nsfw)"
    )];
    match &app.server_status {
        Some(status) => {
            out.push(format!(
                "server: {} {} — protocol {} — session {}",
                text(status, "server"),
                text(status, "server_version"),
                text(status, "protocol"),
                app.facade.session_id()
            ));
        }
        None => out.push("server: not connected yet (run: status)".to_string()),
    }
    out.push(format!(
        "tools: {}",
        app.tool_count
            .map(|n| n.to_string())
            .unwrap_or_else(|| "?".to_string())
    ));
    match &app.gates {
        Some(gates) => out.push(format!(
            "safety gates ({}): {} via {}",
            gates.project,
            gates.verdict.label(),
            if gates.source_tool.is_empty() {
                "unread"
            } else {
                &gates.source_tool
            }
        )),
        None => out.push("safety gates: not read yet (run: safety)".to_string()),
    }
    out.push(format!("projects ({}):", app.projects.len()));
    for project in &app.projects {
        let marker = if app.selected_project.as_deref() == Some(project.slug.as_str()) {
            "*"
        } else {
            " "
        };
        out.push(format!(
            " {marker} {} — {} ({}, {})",
            project.slug,
            truncate_chars(&project.name, 40),
            project.engine,
            project.rating
        ));
    }
    out.push(format!("mode: {}", app.mode.label()));
    if let Some(staged) = &app.staged {
        out.push(format!("staged: {} (approve/abort)", staged.tool));
    }
    out.push(format!(
        "trigger proposals pending: {}",
        app.proposals.len()
    ));
    if let Some(error) = &app.last_error {
        out.push(format!("last error: {error}"));
    }
    out
}

/// Workflows: the catalogue with effective modes.
pub fn workflows_lines<F: ArdorFacade>(app: &App<F>) -> Vec<String> {
    let mut out =
        vec!["workflows (provider routing: n/a — ardor routes no model calls)".to_string()];
    for (index, def) in workflows::catalogue().iter().enumerate() {
        let marker = if index == app.selected_workflow {
            ">"
        } else {
            " "
        };
        let mode = workflows::effective_mode(def, app.mode);
        out.push(format!(
            "{marker} {:>2} [{}] {} — {} (mode: {}{})",
            index + 1,
            def.group.label(),
            def.title,
            def.tool,
            mode.label(),
            if def.plan_only { ", plan-only" } else { "" }
        ));
    }
    out
}

/// Safety: the gate panel. States come from the server's own
/// results only; the snapshot's provenance is always shown.
pub fn safety_lines<F: ArdorFacade>(app: &App<F>) -> Vec<String> {
    let Some(gates) = &app.gates else {
        return vec!["safety gates: not read yet (run: safety)".to_string()];
    };
    let mut out = vec![format!(
        "safety gates for {}: {}",
        gates.project,
        gates.verdict.label()
    )];
    out.push(format!(
        "source: {} — session {}",
        if gates.source_tool.is_empty() {
            "none (unread)"
        } else {
            &gates.source_tool
        },
        gates.session
    ));
    if let Some(reason) = &gates.unknown_reason {
        out.push(format!("unknown because: {reason}"));
    }
    for check in &gates.checks {
        out.push(format!(
            "  [{}] {} — {}",
            if check.ok { "ok" } else { "FAIL" },
            check.name,
            truncate_chars(&check.detail, 90)
        ));
    }
    if gates.verdict == GateVerdict::Red {
        out.push(format!("failing: {}", gates.failing().join(", ")));
    }
    out
}

/// Log/Live: the bounded event buffer.
pub fn log_lines<F: ArdorFacade>(app: &App<F>) -> Vec<String> {
    if app.log.is_empty() {
        return vec!["(no events yet)".to_string()];
    }
    app.log.iter().cloned().collect()
}

/// Fleet: trigger rules, proposals, and the audit tail.
pub fn fleet_lines<F: ArdorFacade>(app: &App<F>) -> Vec<String> {
    let mut out = vec![
        "fleet panel — ardor (self) via this MCP session".to_string(),
        "other members: not probed in rung 1 (no fleet bus; triggers fire in-process, DESIGN D4)"
            .to_string(),
        "trigger rules:".to_string(),
    ];
    for rule in app.engine.rules() {
        out.push(format!(
            "  [{}] {} — on {} where {} -> {}.{} (mode {}, max_depth {}, lane={}, label={})",
            if rule.enabled { "on" } else { "off" },
            rule.id,
            rule.on_event,
            if rule.require_gates_green {
                "gates_green"
            } else {
                "always"
            },
            rule.target_mcp,
            rule.target_tool,
            rule.mode.label(),
            rule.max_depth,
            rule.origin_lane,
            rule.content_label
        ));
    }
    out.push(format!("proposals pending: {}", app.proposals.len()));
    for (index, proposal) in app.proposals.iter().enumerate() {
        out.push(format!(
            "  #{index} {} -> {} for {} (hash {}, plan mode)",
            proposal.rule_id,
            proposal.target_tool,
            proposal.project,
            truncate_chars(&proposal.playtest_hash, 16)
        ));
    }
    out.push("trigger audit tail:".to_string());
    for line in app.engine.audit() {
        out.push(format!("  {line}"));
    }
    out
}

fn text(value: &Value, key: &str) -> String {
    value
        .get(key)
        .and_then(Value::as_str)
        .unwrap_or("?")
        .to_string()
}

/// Keep `Mode` referenced for shells that label the header.
pub fn mode_label(mode: Mode) -> &'static str {
    mode.label()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::facade::FakeFacade;
    use crate::triggers::{TriggerEngine, parse_rules};
    use serde_json::json;

    fn app() -> App<FakeFacade> {
        let mut fake = FakeFacade::new();
        fake.script_status(Ok(json!({
            "server": "ardor", "server_version": "1.0.1", "protocol": "2026-07-28"
        })));
        fake.script_tools(Ok(json!({"tools": [{"name": "a"}]})));
        fake.script_call(
            "list_projects",
            Ok(json!({"projects": [
                {"slug": "demo", "name": "Demo", "engine": "renpy", "rating": "adult"}
            ]})),
        );
        let engine =
            TriggerEngine::new(parse_rules(crate::triggers::EMBEDDED_RULES).expect("rules"));
        let mut app = App::new(fake, engine);
        app.refresh_overview();
        app
    }

    #[test]
    fn overview_names_server_project_and_mode() {
        let lines = overview_lines(&app());
        let joined = lines.join("\n");
        assert!(joined.contains("ardor"), "{joined}");
        assert!(joined.contains("demo"), "{joined}");
        assert!(joined.contains("mode: plan"), "{joined}");
    }

    #[test]
    fn fleet_lines_carry_lane_stamps() {
        let lines = fleet_lines(&app());
        let joined = lines.join("\n");
        assert!(joined.contains("lane=ntranch"), "{joined}");
        assert!(joined.contains("label=nsfw"), "{joined}");
        assert!(joined.contains("prepare_handoff"), "{joined}");
    }

    #[test]
    fn safety_lines_before_read_say_not_read() {
        let lines = safety_lines(&app());
        assert!(lines[0].contains("not read yet"), "{lines:?}");
    }

    #[test]
    fn workflows_lines_mark_handoff_plan_only() {
        let lines = workflows_lines(&app());
        let joined = lines.join("\n");
        assert!(joined.contains("plan-only"), "{joined}");
        assert!(joined.contains("provider routing: n/a"), "{joined}");
    }
}
