//! End-to-end stdio smoke: spawn the real `ardor` binary against
//! a scratch home and drive it like a modern-era MCP client —
//! `server/discover` first, per-request `_meta` on everything
//! after — through a full authoring session (project → cast →
//! graph → dialogue → playtest → credentials → handoff), plus the
//! adversarial battery: legacy `initialize` refusal, framing
//! errors, gate-refused writes leaving state byte-identical,
//! safety surfaces that cannot be progression-gated, consent
//! refusals, and raw secrets refused at the credential gate.
//! Roughly half validation, half adversarial, per estate rule.

use std::collections::BTreeMap;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

use serde_json::{Value, json};

const MODERN: &str = "2026-07-28";
/// A fake credential value used only as adversarial input: the
/// tests assert it is refused and appears in NO state file and NO
/// tool output.
const FAKE_SECRET: &str = "AbCdEfGhIjKlMnOpQrStUvWxYz0123456789abcd";

fn meta(version: &str) -> Value {
    json!({
        "io.modelcontextprotocol/protocolVersion": version,
        "io.modelcontextprotocol/clientInfo": {"name": "smoke", "version": "0"},
        "io.modelcontextprotocol/clientCapabilities": {}
    })
}

struct Client {
    child: Child,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    next_id: u64,
}

impl Client {
    fn spawn(home: &Path) -> Client {
        let mut child = Command::new(env!("CARGO_BIN_EXE_ardor"))
            .arg("--home")
            .arg(home)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn ardor");
        let stdin = child.stdin.take().expect("stdin");
        let stdout = BufReader::new(child.stdout.take().expect("stdout"));
        Client {
            child,
            stdin,
            stdout,
            next_id: 0,
        }
    }

    fn send_raw(&mut self, line: &str) -> Option<Value> {
        self.stdin.write_all(line.as_bytes()).expect("write");
        self.stdin.write_all(b"\n").expect("write nl");
        self.stdin.flush().expect("flush");
        let mut buf = String::new();
        let n = self.stdout.read_line(&mut buf).expect("read");
        if n == 0 {
            None
        } else {
            Some(serde_json::from_str(&buf).expect("response is json"))
        }
    }

    fn call_unstamped(&mut self, method: &str, params: Value) -> Value {
        self.next_id += 1;
        let id = self.next_id;
        let response = self
            .send_raw(
                &json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params})
                    .to_string(),
            )
            .expect("response");
        assert_eq!(response["id"], id, "response id matches request id");
        response
    }

    fn call(&mut self, method: &str, params: Value) -> Value {
        let mut params = params;
        params
            .as_object_mut()
            .expect("params object")
            .insert("_meta".to_string(), meta(MODERN));
        self.call_unstamped(method, params)
    }

    fn discover(&mut self) -> Value {
        let response = self.call("server/discover", json!({}));
        let result = &response["result"];
        assert_eq!(
            result["supportedVersions"],
            json!([MODERN]),
            "discover advertises exactly the modern version: {response}"
        );
        assert_eq!(
            result["_meta"]["io.modelcontextprotocol/serverInfo"]["name"],
            "ardor"
        );
        response
    }

    /// A tool call expected to succeed; returns the parsed payload
    /// and asserts the raw response carries no fake-secret text.
    fn tool(&mut self, name: &str, arguments: Value) -> Value {
        let response = self.tool_raw(name, arguments);
        assert!(
            !response.to_string().contains(FAKE_SECRET),
            "tool output must never carry secret material"
        );
        let text = response["result"]["content"][0]["text"]
            .as_str()
            .expect("tool text content");
        serde_json::from_str(text).expect("tool payload is json")
    }

    /// A tool call expected to fail as a domain (isError) result;
    /// returns the {code, message} payload.
    fn tool_domain_error(&mut self, name: &str, arguments: Value) -> Value {
        let response = self.tool_raw(name, arguments);
        assert_eq!(
            response["result"]["isError"],
            json!(true),
            "expected isError for {name}: {response}"
        );
        let text = response["result"]["content"][0]["text"]
            .as_str()
            .expect("error text content");
        let payload: Value = serde_json::from_str(text).expect("error payload is json");
        payload["error"].clone()
    }

    fn tool_raw(&mut self, name: &str, arguments: Value) -> Value {
        self.call("tools/call", json!({"name": name, "arguments": arguments}))
    }

    fn close(mut self) {
        drop(self.stdin);
        let status = self.child.wait().expect("wait");
        assert!(status.success(), "server exits cleanly on EOF");
    }
}

fn scratch_home(tag: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("ardor-smoke-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    dir
}

/// Every file under the home, bytes included — the byte-identical
/// oracle for gate-refused writes.
fn snapshot_tree(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut out = BTreeMap::new();
    fn walk(dir: &Path, out: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for entry in std::fs::read_dir(dir).expect("read dir").flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, out);
            } else {
                out.insert(path.clone(), std::fs::read(&path).expect("read file"));
            }
        }
    }
    if root.exists() {
        walk(root, &mut out);
    }
    out
}

/// Author the shared mini-project used by several tests: an
/// adult project with an age gate, two characters, one flag, a
/// scene, and a consent-clean dialogue. Returns the playtest hash.
fn author_mini_project(client: &mut Client) -> String {
    let created = client.tool(
        "create_project",
        json!({
            "name": "Smoke Game",
            "slug": "smoke-game",
            "engine": "bevy",
            "rating": "adult",
            "age_gate": "18+ age confirmation on first launch",
            "content_tags": ["romance"]
        }),
    );
    assert_eq!(created["created"], true);
    client.tool(
        "upsert_character",
        json!({"project": "smoke-game", "character": {"id": "mira", "name": "Mira"}}),
    );
    client.tool(
        "upsert_character",
        json!({"project": "smoke-game", "character": {"id": "jun", "name": "Jun"}}),
    );
    client.tool(
        "declare_flag",
        json!({"project": "smoke-game", "flag": {"name": "met_mira", "kind": "story"}}),
    );
    // The gate requires next/choice targets to exist at write
    // time, so author in dependency order: scene (to END) first,
    // then the dialogue, then re-point the scene at the dialogue.
    client.tool(
        "upsert_scene",
        json!({
            "project": "smoke-game",
            "node": {"id": "opening", "next": "END"}
        }),
    );
    client.tool(
        "update_project_meta",
        json!({"project": "smoke-game", "start_node": "opening"}),
    );
    let dialogue = client.tool(
        "upsert_dialogue",
        json!({
            "project": "smoke-game",
            "node": {
                "id": "first_words",
                "participants": ["mira", "jun"],
                "lines": [{"speaker": "mira", "text": "You came."}],
                "consent": {"state": "affirmative", "participants": ["mira", "jun"]},
                "actions": [
                    {"kind": "embrace", "actor": "mira", "target": "jun", "intensity": 4}
                ],
                "choices": [
                    {"label": "Stay", "next": "END"},
                    {"label": "Talk more", "next": "opening"}
                ]
            }
        }),
    );
    assert_eq!(
        dialogue["node"]["id"], "first_words",
        "dialogue landed: {dialogue}"
    );
    let scene = client.tool(
        "upsert_scene",
        json!({
            "project": "smoke-game",
            "node": {"id": "opening", "next": "first_words"}
        }),
    );
    assert_eq!(
        scene["node"]["next"], "first_words",
        "scene re-point landed"
    );
    // The dialogue must really be in the graph (an isError payload
    // would also parse as JSON, so assert the stored node).
    let node = client.tool(
        "get_node",
        json!({"project": "smoke-game", "node": "first_words"}),
    );
    assert_eq!(node["node"]["kind"], "dialogue");
    let run = client.tool(
        "run_playtest",
        json!({"project": "smoke-game", "seed": 42, "max_steps": 64}),
    );
    assert!(
        run["run"]["steps_count"].as_u64().unwrap_or(0) >= 2,
        "the walk must traverse scene -> dialogue, got {run}"
    );
    run["run"]["hash"].as_str().expect("run hash").to_string()
}

#[test]
fn full_client_round_trip() {
    let home = scratch_home("roundtrip");
    let mut client = Client::spawn(&home);
    client.discover();

    // The catalog is the full ten-group surface.
    let list = client.call("tools/list", json!({}));
    let names: Vec<&str> = list["result"]["tools"]
        .as_array()
        .expect("tools array")
        .iter()
        .filter_map(|t| t["name"].as_str())
        .collect();
    assert_eq!(names.len(), 46, "46 tools across the ten groups: {names:?}");
    for expected in [
        "create_project",
        "upsert_dialogue",
        "prepare_handoff",
        "register_channel_account",
        "publish_preflight",
        "list_channels",
    ] {
        assert!(names.contains(&expected), "missing tool {expected}");
    }

    let hash = author_mini_project(&mut client);

    // The same seed replays identically in-process and verifies.
    let again = client.tool(
        "generate_playtest",
        json!({"project": "smoke-game", "seed": 42, "max_steps": 64}),
    );
    assert_eq!(again["hash"], hash);
    let replay = client.tool(
        "verify_replay",
        json!({"project": "smoke-game", "run": "run-1"}),
    );
    assert_eq!(replay["match"], true);

    // The registry answers with the D-126 target set as amended
    // by the D-128 sweep annex and the D-129 strike: 47 records
    // — 46 verified at build time, 1 flagged (Zoom Platform).
    // The pixiv family was struck by D-129 and MiKandi by D-126
    // addendum 2; struck channels have no records at all.
    let channels = client.tool("list_channels", json!({}));
    assert_eq!(channels["count"], 47);
    assert_eq!(channels["verified"], 46);
    assert_eq!(channels["flagged"], 1);

    // Group 10 happy path: register the itch reference, attest
    // the probe, preflight, then the handoff gate opens.
    let account = client.tool(
        "register_channel_account",
        json!({
            "record": {
                "channel": "itch_io",
                "account": "ntranch",
                "credential_kind": "butler_key",
                "store_ref": {"store": "pass", "path": "ntranch/itch/butler-key"},
                "scopes": ["publish"]
            }
        }),
    );
    assert_eq!(account["account"]["status"], "unverified");
    // Handoff before verification: refused by the group-10 check.
    let err = client.tool_domain_error(
        "prepare_handoff",
        json!({
            "project": "smoke-game",
            "channel": "itch_io",
            "artifacts": [{"name": "game.zip", "sha256": "a".repeat(64)}]
        }),
    );
    assert_eq!(err["code"], "channel_auth_unverified");
    client.tool(
        "record_auth_verification",
        json!({
            "account_id": "itch_io:ntranch",
            "result": "verified",
            "method": "itch.io API key validation (GET /api/1/<key>/me) run by the value holder",
            "verified_at": "2026-10-27T15:00:00Z"
        }),
    );
    let preflight = client.tool(
        "publish_preflight",
        json!({"project": "smoke-game", "channel": "itch_io", "now": "2026-10-27T16:00:00Z"}),
    );
    assert_eq!(preflight["ok"], true, "preflight: {preflight}");
    let handoff = client.tool(
        "prepare_handoff",
        json!({
            "project": "smoke-game",
            "channel": "itch_io",
            "artifacts": [{"name": "game.zip", "sha256": "a".repeat(64), "kind": "build"}]
        }),
    );
    assert_eq!(handoff["handoff"]["executor"], "butler");
    assert_eq!(handoff["handoff"]["status"], "prepared");

    client.close();

    // State persists across a restart, hash included.
    let mut client = Client::spawn(&home);
    client.discover();
    let replay = client.tool(
        "verify_replay",
        json!({"project": "smoke-game", "run": "run-1"}),
    );
    assert_eq!(replay["match"], true);
    assert_eq!(replay["recorded_hash"], hash);
    client.close();
    let _ = std::fs::remove_dir_all(&home);
}

#[test]
fn legacy_contact_is_refused_and_the_server_survives() {
    let home = scratch_home("refusal");
    let mut client = Client::spawn(&home);

    for version in ["2025-06-18", "2025-03-26", "2024-11-05"] {
        let response = client.call_unstamped(
            "initialize",
            json!({"protocolVersion": version, "capabilities": {}, "clientInfo": {"name": "legacy", "version": "0"}}),
        );
        assert_eq!(response["error"]["code"], -32022, "initialize {version}");
        assert_eq!(response["error"]["message"], "Unsupported protocol version");
        assert_eq!(response["error"]["data"]["requested"], version);
        assert_eq!(response["error"]["data"]["supported"], json!([MODERN]));
    }
    // Even a modern-version initialize is refused: the modern era
    // has no initialize handshake at all.
    let response = client.call_unstamped(
        "initialize",
        json!({"protocolVersion": MODERN, "capabilities": {}, "clientInfo": {"name": "x", "version": "0"}}),
    );
    assert_eq!(response["error"]["code"], -32022);

    // No _meta at all: refused by the era machinery, not served.
    let response = client.call_unstamped("tools/list", json!({}));
    assert_eq!(response["error"]["code"], -32602);
    assert!(
        response["error"]["message"]
            .as_str()
            .unwrap_or("")
            .contains("_meta"),
        "metadata refusal names the missing field: {response}"
    );

    // Legacy-stamped _meta: refused -32022.
    let mut params = json!({});
    params
        .as_object_mut()
        .expect("obj")
        .insert("_meta".to_string(), meta("2025-06-18"));
    let response = client.call_unstamped("tools/list", params);
    assert_eq!(response["error"]["code"], -32022);

    // After every refusal the same process still serves modern work.
    client.discover();
    let projects = client.tool("list_projects", json!({}));
    assert_eq!(projects["count"], 0);

    client.close();
    let _ = std::fs::remove_dir_all(&home);
}

#[test]
fn adversarial_frames_and_bounds() {
    let home = scratch_home("adversarial");
    let mut client = Client::spawn(&home);

    let response = client.send_raw("{not json").expect("parse error response");
    assert_eq!(response["error"]["code"], -32700);
    let response = client.send_raw("[1,2,3]").expect("response");
    assert_eq!(response["error"]["code"], -32600);
    let response = client
        .send_raw("{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"bogus\"}")
        .expect("response");
    assert_eq!(response["error"]["code"], -32601);

    // Oversized line: -32700, remainder drained, server alive.
    let big = "x".repeat(4 * 1024 * 1024 + 16);
    let response = client.send_raw(&big).expect("oversize response");
    assert_eq!(response["error"]["code"], -32700);
    assert_eq!(
        response["error"]["message"],
        "request line exceeds size bound"
    );

    client.discover();

    // Unknown tool / unknown argument: baseline params errors.
    let response = client.tool_raw("nuke_project", json!({}));
    assert_eq!(response["error"]["code"], -32602);
    assert_eq!(response["error"]["message"], "unknown tool: nuke_project");
    let response = client.tool_raw("list_projects", json!({"surprise": true}));
    assert_eq!(response["error"]["code"], -32602);
    assert_eq!(response["error"]["message"], "unknown argument: surprise");

    client.close();
    let _ = std::fs::remove_dir_all(&home);
}

#[test]
fn gate_refusals_leave_state_byte_identical() {
    let home = scratch_home("gateproof");
    let mut client = Client::spawn(&home);
    client.discover();
    author_mini_project(&mut client);
    let before = snapshot_tree(&home);

    // Safety surface gated behind progress — refused at the gate.
    let err = client.tool_domain_error(
        "upsert_scene",
        json!({
            "project": "smoke-game",
            "node": {"id": "trap", "gates": [{"surface": "safeword", "progression_gated": true}]}
        }),
    );
    assert_eq!(err["code"], "safety_progression_gate_forbidden");
    // Disabling a safety surface — refused.
    let err = client.tool_domain_error(
        "set_safety_surface",
        json!({
            "project": "smoke-game",
            "surface": "safeword",
            "config": {"enabled": false}
        }),
    );
    assert_eq!(err["code"], "safety_surface_mandatory");
    // Dialogue action without affirmative consent — refused.
    let err = client.tool_domain_error(
        "upsert_dialogue",
        json!({
            "project": "smoke-game",
            "node": {
                "id": "bad_dialogue",
                "participants": ["mira", "jun"],
                "actions": [{"kind": "embrace", "actor": "mira", "target": "jun", "intensity": 3}]
            }
        }),
    );
    assert_eq!(err["code"], "consent_state_required");
    // Undeclared flag prerequisite — refused.
    let err = client.tool_domain_error(
        "upsert_dialogue",
        json!({
            "project": "smoke-game",
            "node": {
                "id": "bad_dialogue2",
                "participants": ["mira"],
                "prerequisites": [{"flag": "never_declared"}]
            }
        }),
    );
    assert_eq!(err["code"], "undeclared_flag");
    // Handoff to a flagged channel — refused.
    let err = client.tool_domain_error(
        "prepare_handoff",
        json!({
            "project": "smoke-game",
            "channel": "zoom_platform",
            "artifacts": [{"name": "g.zip", "sha256": "b".repeat(64)}]
        }),
    );
    assert_eq!(err["code"], "channel_flagged");
    // Handoff to the struck channel: it has no record at all.
    let err = client.tool_domain_error(
        "prepare_handoff",
        json!({
            "project": "smoke-game",
            "channel": "mikandi",
            "artifacts": [{"name": "g.zip", "sha256": "b".repeat(64)}]
        }),
    );
    assert_eq!(err["code"], "unknown_channel");
    // The pixiv family is struck the same way (D-129): no record.
    let err = client.tool_domain_error(
        "prepare_handoff",
        json!({
            "project": "smoke-game",
            "channel": "booth",
            "artifacts": [{"name": "g.zip", "sha256": "b".repeat(64)}]
        }),
    );
    assert_eq!(err["code"], "unknown_channel");
    // A services channel takes commissions, not releases.
    let err = client.tool_domain_error(
        "prepare_handoff",
        json!({
            "project": "smoke-game",
            "channel": "vgen",
            "artifacts": [{"name": "g.zip", "sha256": "b".repeat(64)}]
        }),
    );
    assert_eq!(err["code"], "not_a_release_channel");
    // An SFW-only channel never takes an adult project's release.
    let err = client.tool_domain_error(
        "prepare_handoff",
        json!({
            "project": "smoke-game",
            "channel": "fab",
            "artifacts": [{"name": "g.zip", "sha256": "b".repeat(64)}]
        }),
    );
    assert_eq!(err["code"], "channel_sfw_only");

    // Nothing above touched a single byte on disk.
    let after = snapshot_tree(&home);
    assert_eq!(before, after, "refused writes leave state byte-identical");
    client.close();
    let _ = std::fs::remove_dir_all(&home);
}

#[test]
fn credential_values_are_refused_and_never_stored() {
    let home = scratch_home("secrets");
    let mut client = Client::spawn(&home);
    client.discover();

    // A raw value in the reference field: refused as secret-shaped.
    let err = client.tool_domain_error(
        "register_channel_account",
        json!({
            "record": {
                "channel": "itch_io",
                "account": "ntranch",
                "credential_kind": "butler_key",
                "store_ref": {"store": "pass", "path": FAKE_SECRET}
            }
        }),
    );
    assert_eq!(err["code"], "secret_value_refused");
    // A secret-named field anywhere in the payload: refused.
    let err = client.tool_domain_error(
        "register_channel_account",
        json!({
            "record": {
                "channel": "itch_io",
                "account": "ntranch",
                "credential_kind": "butler_key",
                "store_ref": {"store": "pass", "path": "ntranch/itch/key"},
                "api_key": FAKE_SECRET
            }
        }),
    );
    assert_eq!(err["code"], "secret_value_refused");
    // Invented auth flow for a portal-only channel: refused.
    let err = client.tool_domain_error(
        "register_channel_account",
        json!({
            "record": {
                "channel": "fakku",
                "account": "ntranch",
                "credential_kind": "api_key",
                "store_ref": {"store": "pass", "path": "ntranch/fakku/key"}
            }
        }),
    );
    assert_eq!(err["code"], "auth_kind_mismatch");
    // Accounts for flagged channels are refused too.
    let err = client.tool_domain_error(
        "register_channel_account",
        json!({
            "record": {
                "channel": "zoom_platform",
                "account": "ntranch",
                "credential_kind": "portal_login",
                "store_ref": {"store": "pass", "path": "ntranch/zoom/login"}
            }
        }),
    );
    assert_eq!(err["code"], "channel_flagged");
    // Accounts for struck channels are refused: the pixiv family
    // has no records at all (D-129), so it is unknown, not flagged.
    let err = client.tool_domain_error(
        "register_channel_account",
        json!({
            "record": {
                "channel": "booth",
                "account": "ntranch-pixiv",
                "credential_kind": "portal_login",
                "store_ref": {"store": "pass", "path": "ntranch/pixiv/login"}
            }
        }),
    );
    assert_eq!(err["code"], "unknown_channel");

    // No accounts file may exist (nothing was ever written), and
    // no file under the home may contain the fake secret.
    assert!(!home.join("accounts.json").exists());
    for (path, bytes) in snapshot_tree(&home) {
        assert!(
            !String::from_utf8_lossy(&bytes).contains(FAKE_SECRET),
            "secret material found in {}",
            path.display()
        );
    }

    // The happy path: a reference for a verified portal channel
    // registers and reports its unverified status.
    let ok = client.tool(
        "register_channel_account",
        json!({
            "record": {
                "channel": "fakku",
                "account": "ntranch",
                "credential_kind": "portal_login",
                "store_ref": {"store": "secure_vault", "entry": "fakku-partner-login"}
            }
        }),
    );
    assert_eq!(ok["account"]["channel"], "fakku");
    let status = client.tool("credential_status", json!({"channel": "fakku"}));
    assert_eq!(status["channels"][0]["status"], "unverified");

    client.close();
    let _ = std::fs::remove_dir_all(&home);
}

#[test]
fn annex_registry_end_to_end() {
    let home = scratch_home("annex");
    let mut client = Client::spawn(&home);
    client.discover();
    author_mini_project(&mut client);

    // Section shape over the wire (D-128 annex).
    let channels = client.tool("list_channels", json!({}));
    assert_eq!(channels["sections"]["distribution"], 23);
    assert_eq!(channels["sections"]["storefront_rail"], 4);
    assert_eq!(channels["sections"]["assets"], 6);
    assert_eq!(channels["sections"]["assets_sfw"], 5);
    assert_eq!(channels["sections"]["services"], 7);
    assert_eq!(channels["sections"]["services_sfw"], 2);

    // The struck family is absent over the wire too (D-129).
    let err = client.tool_domain_error("get_channel", json!({"channel": "fanbox"}));
    assert_eq!(err["code"], "unknown_channel");
    // Sections and cross-listings are part of the wire record.
    let fab = client.tool("get_channel", json!({"channel": "fab"}));
    assert_eq!(fab["section"], "assets_sfw");
    let onlyfans = client.tool("get_channel", json!({"channel": "onlyfans"}));
    assert_eq!(onlyfans["section"], "distribution");
    assert_eq!(onlyfans["cross_listed_in"], json!(["services"]));

    // Services channel (VGen): the account reference registers
    // and verifies, preflight is honest that it is not a release
    // target, and the handoff gate refuses by section.
    let account = client.tool(
        "register_channel_account",
        json!({
            "record": {
                "channel": "vgen",
                "account": "ntranch-studio",
                "credential_kind": "portal_login",
                "store_ref": {"store": "secure_vault", "entry": "vgen-studio-login"},
                "scopes": ["commissions"]
            }
        }),
    );
    assert_eq!(account["account"]["channel"], "vgen");
    client.tool(
        "record_auth_verification",
        json!({
            "account_id": "vgen:ntranch-studio",
            "result": "verified",
            "method": "operator attestation: VGen artist account login works",
            "verified_at": "2026-10-27T17:00:00Z"
        }),
    );
    let preflight = client.tool(
        "publish_preflight",
        json!({"project": "smoke-game", "channel": "vgen", "now": "2026-10-27T18:00:00Z"}),
    );
    assert_eq!(preflight["ok"], false, "preflight: {preflight}");
    let release_check = preflight["checks"]
        .as_array()
        .expect("checks array")
        .iter()
        .find(|c| c["check"] == "release_target")
        .expect("release_target check present")
        .clone();
    assert_eq!(release_check["ok"], false);
    let err = client.tool_domain_error(
        "prepare_handoff",
        json!({
            "project": "smoke-game",
            "channel": "vgen",
            "artifacts": [{"name": "rig.zip", "sha256": "c".repeat(64), "kind": "asset"}]
        }),
    );
    assert_eq!(err["code"], "not_a_release_channel");

    // Adult asset channel (Renderotica): register -> verify ->
    // preflight green -> handoff prepared, executor manual.
    client.tool(
        "register_channel_account",
        json!({
            "record": {
                "channel": "renderotica",
                "account": "ntranch",
                "credential_kind": "portal_login",
                "store_ref": {"store": "pass", "path": "ntranch/renderotica/vendor-login"},
                "scopes": ["publish"]
            }
        }),
    );
    client.tool(
        "record_auth_verification",
        json!({
            "account_id": "renderotica:ntranch",
            "result": "verified",
            "method": "operator attestation: Renderotica vendor account login works",
            "verified_at": "2026-10-27T17:05:00Z"
        }),
    );
    let preflight = client.tool(
        "publish_preflight",
        json!({"project": "smoke-game", "channel": "renderotica", "now": "2026-10-27T18:00:00Z"}),
    );
    assert_eq!(preflight["ok"], true, "preflight: {preflight}");
    let handoff = client.tool(
        "prepare_handoff",
        json!({
            "project": "smoke-game",
            "channel": "renderotica",
            "artifacts": [{"name": "asset-pack.zip", "sha256": "d".repeat(64), "kind": "asset"}]
        }),
    );
    assert_eq!(handoff["handoff"]["status"], "prepared");
    assert_eq!(handoff["handoff"]["executor"], "manual");
    assert_eq!(handoff["handoff"]["section"], "assets");

    // Own-storefront rail (CCBill): a handoff rides the rail —
    // the record names the processor, the studio's own storefront
    // does the selling.
    client.tool(
        "register_channel_account",
        json!({
            "record": {
                "channel": "ccbill",
                "account": "ntranch-merchant",
                "credential_kind": "portal_login",
                "store_ref": {"store": "pass", "path": "ntranch/ccbill/merchant-login"},
                "scopes": ["billing"]
            }
        }),
    );
    client.tool(
        "record_auth_verification",
        json!({
            "account_id": "ccbill:ntranch-merchant",
            "result": "verified",
            "method": "operator attestation: CCBill merchant account login works",
            "verified_at": "2026-10-27T17:10:00Z"
        }),
    );
    let handoff = client.tool(
        "prepare_handoff",
        json!({
            "project": "smoke-game",
            "channel": "ccbill",
            "artifacts": [{"name": "game.zip", "sha256": "e".repeat(64), "kind": "build"}]
        }),
    );
    assert_eq!(handoff["handoff"]["status"], "prepared");
    assert_eq!(handoff["handoff"]["section"], "storefront_rail");

    // The credential rollup covers the whole annex registry.
    let status = client.tool("credential_status", json!({}));
    assert_eq!(status["count"], 47);

    client.close();
    let _ = std::fs::remove_dir_all(&home);
}
