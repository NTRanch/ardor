//! ardor-tui — thin bin: parse args, build the real facade
//! (an MCP stdio client spawning the ardor server), run.
//!
//! Usage: `ardor-tui [--server <path-to-ardor>] [--home <dir>]
//! [--triggers <path>] [--fake] [--line]`
//!
//! Full-screen ratatui runs where a TTY exists; otherwise (or
//! with `--line`) the same app runs in line mode over
//! stdin/stdout. `--fake` drives the in-memory demo facade —
//! for smoke and demos only; it is not the server.

#![forbid(unsafe_code)]

use std::io::{BufRead, IsTerminal, Write};
use std::process::ExitCode;
use std::time::Duration;

use ardor_tui::app::{App, Screen};
use ardor_tui::client::McpStdioFacade;
use ardor_tui::facade::{ArdorFacade, FakeFacade};
use ardor_tui::line::execute_line;
use ardor_tui::triggers::{TriggerEngine, parse_rules};
use ardor_tui::workflows::{self, Mode};
use serde_json::json;

struct Args {
    server: String,
    home: Option<String>,
    triggers: Option<String>,
    fake: bool,
    line: bool,
}

fn parse_args(args: &[String]) -> Result<Args, String> {
    let mut out = Args {
        server: std::env::var("ARDOR_BIN").unwrap_or_else(|_| "ardor".to_string()),
        home: None,
        triggers: None,
        fake: false,
        line: false,
    };
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--server" => {
                i += 1;
                out.server = args.get(i).ok_or("--server needs a path")?.clone();
            }
            "--home" => {
                i += 1;
                out.home = Some(args.get(i).ok_or("--home needs a dir")?.clone());
            }
            "--triggers" => {
                i += 1;
                out.triggers = Some(args.get(i).ok_or("--triggers needs a path")?.clone());
            }
            "--fake" => out.fake = true,
            "--line" => out.line = true,
            other => return Err(format!("unknown argument: {other}")),
        }
        i += 1;
    }
    Ok(out)
}

fn load_engine(path: Option<&str>) -> Result<TriggerEngine, String> {
    let text = match path {
        Some(path) => std::fs::read_to_string(path)
            .map_err(|e| format!("cannot read triggers file {path}: {e}"))?,
        None => ardor_tui::triggers::EMBEDDED_RULES.to_string(),
    };
    Ok(TriggerEngine::new(parse_rules(&text)?))
}

/// A demo facade for `--fake`: scripted, finite, clearly not
/// the server. Enough responses for a status/safety/playtest
/// walk in line mode or the TUI.
fn demo_fake() -> FakeFacade {
    const HASH: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    let mut fake = FakeFacade::new();
    for _ in 0..8 {
        fake.script_status(Ok(json!({
            "server": "ardor (FAKE)", "server_version": "1.0.1", "protocol": "2026-07-28"
        })));
        fake.script_tools(Ok(
            json!({"tools": [{"name": "list_projects"}, {"name": "audit_safety"}]}),
        ));
        fake.script_call(
            "list_projects",
            Ok(json!({"projects": [
                {"slug": "demo", "name": "Demo Game", "engine": "renpy", "rating": "adult"}
            ], "count": 1})),
        );
        fake.script_call(
            "audit_safety",
            Ok(json!({"project": "demo", "audit": {"ok": true, "checks": [
                {"check": "safety_surface:stop", "ok": true, "detail": "present, enabled, ungated"},
                {"check": "age_gate_present", "ok": true, "detail": "age-gate pattern recorded"}
            ]}})),
        );
        fake.script_call(
            "run_playtest",
            Ok(json!({"project": "demo", "run": {
                "id": "run-1", "seed": 1, "steps_count": 4, "truncated": false,
                "stop_reason": "terminal", "hash": HASH
            }})),
        );
        fake.script_call(
            "prepare_handoff",
            Ok(json!({"project": "demo", "handoff": {"id": "h-demo"}})),
        );
    }
    fake
}

fn run_line_mode<F: ArdorFacade>(app: &mut App<F>) -> ExitCode {
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        for out in execute_line(app, &line) {
            let _ = writeln!(stdout, "{out}");
        }
        let _ = stdout.flush();
        if app.should_quit {
            break;
        }
    }
    ExitCode::SUCCESS
}

fn run_tui<F: ArdorFacade>(app: &mut App<F>) -> ExitCode {
    use crossterm::event::{Event, KeyEventKind};
    use crossterm::terminal::{
        EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
    };
    use crossterm::{ExecutableCommand, execute};

    let mut stdout = std::io::stdout();
    if enable_raw_mode().is_err() || stdout.execute(EnterAlternateScreen).is_err() {
        eprintln!("ardor-tui: cannot enter raw terminal mode; use --line");
        return ExitCode::FAILURE;
    }
    let backend = ratatui::backend::CrosstermBackend::new(stdout);
    let mut terminal = match ratatui::Terminal::new(backend) {
        Ok(terminal) => terminal,
        Err(_) => {
            let _ = disable_raw_mode();
            return ExitCode::FAILURE;
        }
    };
    let _ = app.refresh_overview();
    while !app.should_quit {
        let _ = terminal.draw(|frame| ardor_tui::panels::render(frame, app));
        if !matches!(crossterm::event::poll(Duration::from_millis(100)), Ok(true)) {
            continue;
        }
        let Ok(Event::Key(key)) = crossterm::event::read() else {
            continue;
        };
        if key.kind != KeyEventKind::Press {
            continue;
        }
        handle_key(app, key.code);
    }
    let _ = terminal.show_cursor();
    let _ = disable_raw_mode();
    let _ = execute!(std::io::stdout(), LeaveAlternateScreen);
    ExitCode::SUCCESS
}

fn handle_key<F: ArdorFacade>(app: &mut App<F>, code: crossterm::event::KeyCode) {
    use crossterm::event::KeyCode;
    let out: Vec<String> = match code {
        KeyCode::Char('q') => {
            app.should_quit = true;
            Vec::new()
        }
        KeyCode::Char(c @ '1'..='5') => {
            app.screen = Screen::ALL[usize::from(c as u8 - b'1')];
            Vec::new()
        }
        KeyCode::Tab => {
            let index = Screen::ALL
                .iter()
                .position(|s| *s == app.screen)
                .unwrap_or(0);
            app.screen = Screen::ALL[(index + 1) % Screen::ALL.len()];
            Vec::new()
        }
        KeyCode::Char('j') | KeyCode::Down => {
            let count = workflows::catalogue().len();
            app.selected_workflow = (app.selected_workflow + 1) % count;
            Vec::new()
        }
        KeyCode::Char('k') | KeyCode::Up => {
            let count = workflows::catalogue().len();
            app.selected_workflow = (app.selected_workflow + count - 1) % count;
            Vec::new()
        }
        KeyCode::Enter if app.screen == Screen::Workflows => {
            let def = &workflows::catalogue()[app.selected_workflow];
            let out = app.run_workflow(def.id, &json!({}));
            app.screen = Screen::Log;
            out
        }
        KeyCode::Char('a') => app.approve(),
        KeyCode::Char('x') => app.abort(),
        KeyCode::Char('m') => {
            app.mode = match app.mode {
                Mode::Plan => Mode::Auto,
                Mode::Auto => Mode::Plan,
            };
            vec![format!("mode: {}", app.mode.label())]
        }
        KeyCode::Char('p') => cycle_project(app),
        KeyCode::Char('r') => match app.screen {
            Screen::Safety => app.refresh_safety(),
            _ => app.refresh_overview(),
        },
        _ => Vec::new(),
    };
    for line in out {
        app.log_line(line);
    }
}

fn cycle_project<F: ArdorFacade>(app: &mut App<F>) -> Vec<String> {
    if app.projects.is_empty() {
        return vec!["no projects".to_string()];
    }
    let current = app
        .projects
        .iter()
        .position(|p| Some(&p.slug) == app.selected_project.as_ref())
        .unwrap_or(0);
    let next = &app.projects[(current + 1) % app.projects.len()];
    app.select_project(&next.slug.clone())
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let args = match parse_args(&args) {
        Ok(args) => args,
        Err(e) => {
            eprintln!("ardor-tui: {e}");
            return ExitCode::from(2);
        }
    };
    let engine = match load_engine(args.triggers.as_deref()) {
        Ok(engine) => engine,
        Err(e) => {
            eprintln!("ardor-tui: triggers: {e}");
            return ExitCode::from(2);
        }
    };
    let use_line = args.line || !std::io::stdout().is_terminal();
    if args.fake {
        let mut app = App::new(demo_fake(), engine);
        return if use_line {
            run_line_mode(&mut app)
        } else {
            run_tui(&mut app)
        };
    }
    let facade = match McpStdioFacade::spawn(&args.server, args.home.as_deref()) {
        Ok(facade) => facade,
        Err(e) => {
            eprintln!("ardor-tui: cannot start ardor server: {e}");
            return ExitCode::FAILURE;
        }
    };
    let mut app = App::new(facade, engine);
    if use_line {
        run_line_mode(&mut app)
    } else {
        run_tui(&mut app)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_args_accepts_forms() {
        let args = parse_args(&["--line".to_string()]).expect("parse");
        assert!(args.line);
        let args = parse_args(&[
            "--server".to_string(),
            "/x/ardor".to_string(),
            "--fake".to_string(),
        ])
        .expect("parse");
        assert_eq!(args.server, "/x/ardor");
        assert!(args.fake);
        assert!(parse_args(&["--bogus".to_string()]).is_err());
        assert!(parse_args(&["--server".to_string()]).is_err());
    }
}
