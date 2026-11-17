//! Live-server integration: drives the real ardor binary
//! through the real MCP stdio facade — status, tool list, a
//! created project's safety audit, and a recorded playtest
//! whose payload passes this crate's own QA validation.
//!
//! Gated on `ARDOR_TUI_TEST_SERVER` (path to a built `ardor`
//! binary) because the binary is a sibling workspace artifact,
//! not a dependency of this crate. Without the variable the
//! test is a no-op pass and says so; the gate run sets it.

#![forbid(unsafe_code)]

use ardor_tui::client::McpStdioFacade;
use ardor_tui::facade::ArdorFacade;
use ardor_tui::gates::{GateSnapshot, GateVerdict};
use ardor_tui::qa::parse_playtest;
use serde_json::json;

#[test]
fn live_server_round_trip() {
    let Ok(server) = std::env::var("ARDOR_TUI_TEST_SERVER") else {
        eprintln!("ARDOR_TUI_TEST_SERVER unset; live test skipped");
        return;
    };
    let home = std::env::temp_dir().join(format!("ardor-tui-live-{}", std::process::id()));
    let home_str = home.to_string_lossy().to_string();
    let mut facade = McpStdioFacade::spawn(&server, Some(&home_str)).expect("spawn ardor");

    let status = facade.status().expect("discover");
    assert_eq!(status["server"], "ardor");
    assert_eq!(status["supported_versions"], json!(["2026-07-28"]));

    let tools = facade.tools().expect("tools/list");
    let names: Vec<&str> = tools["tools"]
        .as_array()
        .expect("tools array")
        .iter()
        .filter_map(|t| t["name"].as_str())
        .collect();
    assert_eq!(names.len(), 46, "ardor serves 46 tools: {names:?}");
    assert!(names.contains(&"prepare_handoff"));

    facade
        .call(
            "create_project",
            &json!({
                "name": "TUI Live Game", "slug": "tui-live", "engine": "bevy",
                "rating": "adult",
                "age_gate": "18+ age confirmation on first launch",
                "content_tags": ["romance"]
            }),
        )
        .expect("create_project");

    // A playtest needs a start node (the server refuses with
    // `no_start_node` otherwise — verified live): author one
    // scene pointing at the terminal and make it the start.
    facade
        .call(
            "upsert_scene",
            &json!({"project": "tui-live", "node": {"id": "opening", "next": "END"}}),
        )
        .expect("upsert_scene");
    facade
        .call(
            "update_project_meta",
            &json!({"project": "tui-live", "start_node": "opening"}),
        )
        .expect("update_project_meta");

    let audit = facade
        .call("audit_safety", &json!({"project": "tui-live"}))
        .expect("audit_safety");
    let snap = GateSnapshot::from_audit(facade.session_id(), &audit).expect("parse audit");
    assert_eq!(
        snap.verdict,
        GateVerdict::Green,
        "fresh project audits green"
    );

    let run = facade
        .call("run_playtest", &json!({"project": "tui-live", "seed": 3}))
        .expect("run_playtest");
    let parsed = parse_playtest("tui-live", &run).expect("playtest payload validates");
    assert_eq!(parsed.hash.len(), 64);

    // The handoff state machine must refuse here: gates are
    // green and a playtest completed, but through the App's
    // evidence path — this raw facade check only asserts the
    // server-side gate also refuses an account-less handoff.
    let refused = facade.call(
        "prepare_handoff",
        &json!({"project": "tui-live", "channel": "itch_io", "artifacts": []}),
    );
    assert!(
        refused.is_err(),
        "server gate refuses without account/artifacts"
    );

    drop(facade);
    let _ = std::fs::remove_dir_all(&home);
}
