//! Line mode: the non-TTY shell (DESIGN §D2). Every command
//! drives the same [`App`] execute path as the full-screen
//! TUI — staging, approval, gate refresh, and trigger
//! evaluation are literally the same methods — so line mode is
//! scriptable and testable without a terminal.

use serde_json::{Value, json};

use crate::app::{App, Screen};
use crate::facade::ArdorFacade;
use crate::screens;
use crate::workflows::Mode;
use crate::{ARGS_BYTES_MAX, INPUT_BYTES_MAX};

/// The help text, one command per line.
pub const HELP: &[&str] = &[
    "commands:",
    "  status                     refresh server status, tools, projects",
    "  tools                      list the server's tools",
    "  projects                   list projects",
    "  select <slug>              select the working project",
    "  safety                     refresh safety gates (audit_safety)",
    "  preflight <channel>        publish preflight for a channel",
    "  workflows                  list workflows",
    "  run <id> [json-args]       stage (plan) or execute (auto) a workflow",
    "  mode plan|auto             set the requested run mode",
    "  playtest [seed]            run-playtest workflow for the selected project",
    "  channel <id>               set the channel for proposal approval",
    "  handoff <channel> [json]   stage a release handoff (plan-only, gated)",
    "  staged                     show the staged call",
    "  approve                    execute the staged call",
    "  abort                      discard the staged call",
    "  proposals                  list trigger proposals",
    "  approve-proposal <n>       stage proposal n as a handoff",
    "  triggers                   fleet panel (rules + audit)",
    "  trigger on|off <rule-id>   enable/disable a trigger rule",
    "  log                        show the log buffer",
    "  screen <name>              show a screen (overview|workflows|safety|log|fleet)",
    "  quit                       exit",
];

/// Execute one input line; returns the lines to print.
pub fn execute_line<F: ArdorFacade>(app: &mut App<F>, line: &str) -> Vec<String> {
    let line = line.trim();
    if line.is_empty() {
        return Vec::new();
    }
    if line.len() > INPUT_BYTES_MAX {
        return vec![format!("input over {INPUT_BYTES_MAX} bytes; refused")];
    }
    let head = line.split_whitespace().next().unwrap_or("");
    let rest = line[head.len()..].trim();
    match head {
        "help" => HELP.iter().map(|s| s.to_string()).collect(),
        "status" => app.refresh_overview(),
        "tools" => tools_lines(app),
        "projects" => screens::overview_lines(app)
            .into_iter()
            .filter(|l| l.contains("—") || l.starts_with("projects"))
            .collect(),
        "select" => app.select_project(rest),
        "safety" => app.refresh_safety(),
        "preflight" => {
            if rest.is_empty() {
                vec!["usage: preflight <channel>".to_string()]
            } else {
                app.refresh_preflight(rest)
            }
        }
        "workflows" => screens::workflows_lines(app),
        "run" => run_command(app, rest),
        "mode" => match rest {
            "plan" => {
                app.mode = Mode::Plan;
                vec!["mode: plan".to_string()]
            }
            "auto" => {
                app.mode = Mode::Auto;
                vec!["mode: auto (plan-only workflows still stage)".to_string()]
            }
            _ => vec!["usage: mode plan|auto".to_string()],
        },
        "playtest" => {
            let extra = if rest.is_empty() {
                json!({})
            } else {
                match rest.parse::<u64>() {
                    Ok(seed) => json!({"seed": seed}),
                    Err(_) => return vec!["usage: playtest [seed]".to_string()],
                }
            };
            app.run_workflow("run-playtest", &extra)
        }
        "channel" => {
            if rest.is_empty() {
                vec!["usage: channel <id>".to_string()]
            } else {
                app.handoff_channel = Some(rest.to_string());
                vec![format!("handoff channel: {rest}")]
            }
        }
        "handoff" => {
            let (channel, extra) = split_json(rest);
            if channel.is_empty() {
                return vec!["usage: handoff <channel> [json-artifacts]".to_string()];
            }
            let mut args = json!({"channel": channel});
            if let Some(obj) = extra {
                args["artifacts"] = obj;
            }
            app.run_workflow("prepare-handoff", &args)
        }
        "staged" => match &app.staged {
            Some(staged) => vec![format!(
                "staged: {} {}",
                staged.tool,
                crate::app::compact(&staged.args)
            )],
            None => vec!["nothing staged".to_string()],
        },
        "approve" => app.approve(),
        "abort" => app.abort(),
        "proposals" => screens::fleet_lines(app)
            .into_iter()
            .filter(|l| l.contains("proposals") || l.starts_with("  #"))
            .collect(),
        "approve-proposal" => match rest.parse::<usize>() {
            Ok(index) => app.approve_proposal(index),
            Err(_) => vec!["usage: approve-proposal <n>".to_string()],
        },
        "triggers" | "fleet" => screens::fleet_lines(app),
        "trigger" => trigger_command(app, rest),
        "log" => screens::log_lines(app),
        "screen" => screen_command(app, rest),
        "quit" | "exit" => {
            app.should_quit = true;
            vec!["bye".to_string()]
        }
        other => vec![format!("unknown command: {other} (try: help)")],
    }
}

fn tools_lines<F: ArdorFacade>(app: &mut App<F>) -> Vec<String> {
    match app.facade.tools() {
        Ok(value) => {
            let mut out = Vec::new();
            if let Some(tools) = value.get("tools").and_then(Value::as_array) {
                out.push(format!("tools ({}):", tools.len()));
                for tool in tools {
                    if let Some(name) = tool.get("name").and_then(Value::as_str) {
                        out.push(format!("  {name}"));
                    }
                }
            } else {
                out.push("tools/list returned no tools array".to_string());
            }
            out
        }
        Err(e) => vec![format!("tools/list failed: {e}")],
    }
}

/// Split `rest` into its first word and an optional trailing
/// JSON value (object or array), bounded by
/// [`ARGS_BYTES_MAX`].
fn split_json(rest: &str) -> (String, Option<Value>) {
    let first = rest.split_whitespace().next().unwrap_or("").to_string();
    let tail = rest[first.len()..].trim();
    if tail.is_empty() || tail.len() > ARGS_BYTES_MAX {
        return (first, None);
    }
    (first, serde_json::from_str(tail).ok())
}

fn run_command<F: ArdorFacade>(app: &mut App<F>, rest: &str) -> Vec<String> {
    let (id, extra) = split_json(rest);
    if id.is_empty() {
        return vec!["usage: run <workflow-id> [json-args]".to_string()];
    }
    app.run_workflow(&id, &extra.unwrap_or(Value::Null))
}

fn trigger_command<F: ArdorFacade>(app: &mut App<F>, rest: &str) -> Vec<String> {
    let mut parts = rest.split_whitespace();
    let (Some(switch), Some(id)) = (parts.next(), parts.next()) else {
        return vec!["usage: trigger on|off <rule-id>".to_string()];
    };
    let enabled = match switch {
        "on" => true,
        "off" => false,
        _ => return vec!["usage: trigger on|off <rule-id>".to_string()],
    };
    if app.engine.set_enabled(id, enabled) {
        vec![format!("rule {id}: {}", if enabled { "on" } else { "off" })]
    } else {
        vec![format!("unknown rule: {id}")]
    }
}

fn screen_command<F: ArdorFacade>(app: &mut App<F>, rest: &str) -> Vec<String> {
    let screen = match rest {
        "overview" => Screen::Overview,
        "workflows" => Screen::Workflows,
        "safety" => Screen::Safety,
        "log" => Screen::Log,
        "fleet" => Screen::Fleet,
        _ => return vec!["usage: screen overview|workflows|safety|log|fleet".to_string()],
    };
    app.screen = screen;
    screens::screen_lines(app)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::facade::FakeFacade;
    use crate::triggers::{TriggerEngine, parse_rules};
    use serde_json::json;

    const HASH: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

    fn app() -> App<FakeFacade> {
        let mut fake = FakeFacade::new();
        fake.script_status(Ok(json!({
            "server": "ardor", "server_version": "1.0.1", "protocol": "2026-07-28"
        })));
        fake.script_tools(Ok(json!({"tools": [{"name": "list_projects"}]})));
        fake.script_call(
            "list_projects",
            Ok(json!({"projects": [
                {"slug": "demo", "name": "Demo", "engine": "renpy", "rating": "adult"}
            ]})),
        );
        let engine =
            TriggerEngine::new(parse_rules(crate::triggers::EMBEDDED_RULES).expect("rules"));
        App::new(fake, engine)
    }

    #[test]
    fn line_mode_answers_status_from_a_pipe() {
        let mut app = app();
        let out = execute_line(&mut app, "status");
        assert!(out.iter().any(|l| l.contains("connected")), "{out:?}");
        assert_eq!(app.selected_project.as_deref(), Some("demo"));
    }

    #[test]
    fn line_mode_plan_then_approve_calls_once() {
        let mut app = app();
        let _ = execute_line(&mut app, "status");
        app.facade.script_call(
            "run_playtest",
            Ok(json!({"project": "demo", "run": {
                "id": "run-1", "seed": 1, "steps_count": 1, "truncated": false,
                "stop_reason": "terminal", "hash": HASH
            }})),
        );
        let out = execute_line(&mut app, "playtest");
        assert!(out[0].contains("staged"), "{out:?}");
        assert!(
            app.facade.calls().iter().all(|(t, _)| t != "run_playtest"),
            "line mode shares the plan-mode proof"
        );
        let out = execute_line(&mut app, "approve");
        assert!(
            out.iter().any(|l| l.contains("playtest completed")),
            "{out:?}"
        );
    }

    #[test]
    fn line_mode_rejects_oversized_input_and_unknown_commands() {
        let mut app = app();
        let big = "x".repeat(INPUT_BYTES_MAX + 1);
        let out = execute_line(&mut app, &big);
        assert!(out[0].contains("refused"), "{out:?}");
        let out = execute_line(&mut app, "frobnicate");
        assert!(out[0].contains("unknown command"), "{out:?}");
        let out = execute_line(&mut app, "");
        assert!(out.is_empty());
    }

    #[test]
    fn line_mode_quit_sets_flag() {
        let mut app = app();
        let _ = execute_line(&mut app, "quit");
        assert!(app.should_quit);
    }
}
