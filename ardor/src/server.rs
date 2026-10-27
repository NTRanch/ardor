//! The rmcp server: tool surface and handler dispatch.
//!
//! Ardor is born on the fleet's standardized pattern (D-113,
//! proven in WS1): rmcp owns the envelope, the era guard (see
//! `guard.rs`) owns the stdio front door, and this type owns the
//! tools. Domain work lives in `tools.rs` behind the decide()
//! gate in `gate.rs`.
//!
//! Modern-only posture lives in two places: the era guard refuses
//! `initialize` before the SDK sees it, and
//! [`ServerHandler::supported_protocol_versions`] pins this
//! handler to 2026-07-28, so `server/discover` advertises exactly
//! one version and the SDK validates every request's `_meta`
//! against it.
//!
//! Failure taxonomy (fleet-wide): params failures return as
//! protocol errors (`-32602`); domain failures — gate refusals,
//! missing state — return as `isError` content carrying the
//! stable code as compact JSON. Success payloads are pretty JSON
//! text content.

use std::borrow::Cow;
use std::sync::{Arc, Mutex};

use rmcp::model::{
    CallToolRequestParams, CallToolResponse, CallToolResult, ContentBlock, Implementation,
    InitializeResult, ListToolsResult, PaginatedRequestParams, ProtocolVersion, ServerCapabilities,
    Tool,
};
use rmcp::service::RequestContext;
use rmcp::{ErrorData, RoleServer, ServerHandler};
use serde_json::{Value, json};

use crate::store::Store;
use crate::tools;

const INSTRUCTIONS: &str = "Ardor — the NTRanch game/app maker: author adult games and apps as \
    gated project state. Every canonical write passes a decide() gate (canon + safety) before it \
    lands; safety surfaces (stop/safeword, accessibility, aftercare, purge) are never \
    progression-gated. Deterministic playtest generation replays the authored graph under a \
    seed. Release handoffs are prepared records for the channel registry's executors; channel \
    credentials are managed as references and status only — Ardor never holds a credential value.";

fn s() -> Value {
    json!({"type": "string"})
}

fn s_max(max_len: usize) -> Value {
    json!({"type": "string", "maxLength": max_len})
}

fn o() -> Value {
    json!({"type": "object"})
}

fn a() -> Value {
    json!({"type": "array"})
}

fn i64() -> Value {
    json!({"type": "integer", "minimum": 0})
}

fn en(values: &[&str]) -> Value {
    json!({"type": "string", "enum": values})
}

/// Build one schema object from (name, schema) pairs + required.
fn schema(props: &[(&str, Value)], required: &[&str]) -> serde_json::Map<String, Value> {
    let mut map = serde_json::Map::new();
    map.insert("type".to_string(), json!("object"));
    let mut properties = serde_json::Map::new();
    for (name, prop) in props {
        properties.insert((*name).to_string(), prop.clone());
    }
    map.insert("properties".to_string(), Value::Object(properties));
    if !required.is_empty() {
        map.insert(
            "required".to_string(),
            Value::Array(required.iter().map(|r| json!(r)).collect()),
        );
    }
    map.insert("additionalProperties".to_string(), json!(false));
    map
}

/// The tool catalog: hand-written raw schemas (no schemars), one
/// entry per tool in `tools::dispatch`.
fn tool_definitions() -> Vec<Tool> {
    // Each schema needs its own copy of the common project prop.
    let project = || ("project", s_max(64));
    let specs: Vec<(&str, &str, serde_json::Map<String, Value>)> = vec![
        // group 1: project scaffolding
        (
            "create_project",
            "Scaffold a new project (name, engine, rating, age gate) with its safety surfaces seeded enabled and ungated. Adult-rated projects require an age gate — the gate refuses otherwise.",
            schema(
                &[
                    ("name", s_max(200)),
                    ("slug", s_max(64)),
                    ("engine", s_max(80)),
                    ("rating", en(&["adult", "mature", "all_ages"])),
                    ("age_gate", s_max(500)),
                    ("content_tags", a()),
                ],
                &["name"],
            ),
        ),
        (
            "get_project",
            "Project summary: metadata, per-document counts, and whether the safety audit passes.",
            schema(&[project()], &["project"]),
        ),
        (
            "list_projects",
            "Every project in the server home, with name, engine and rating.",
            schema(&[], &[]),
        ),
        (
            "update_project_meta",
            "Patch project metadata (name, engine, rating, age_gate, content_tags, start_node) through the gate: an adult rating still requires an age gate, and start_node must exist.",
            schema(
                &[
                    project(),
                    ("name", s_max(200)),
                    ("engine", s_max(80)),
                    ("rating", en(&["adult", "mature", "all_ages"])),
                    ("age_gate", s_max(500)),
                    ("content_tags", a()),
                    ("start_node", s()),
                ],
                &["project"],
            ),
        ),
        // group 2: cast and canon
        (
            "upsert_character",
            "Create or update a cast member through the canon gate: locked fields and stated facts refuse the write unless an amendment with a reason is attached (amendments are recorded in the cast history).",
            schema(
                &[project(), ("character", o()), ("amendment", o())],
                &["project", "character"],
            ),
        ),
        (
            "get_character",
            "One cast member's full record, including version and reaction profile.",
            schema(
                &[project(), ("character", s_max(64))],
                &["project", "character"],
            ),
        ),
        (
            "list_characters",
            "The full cast plus the amendment history.",
            schema(&[project()], &["project"]),
        ),
        (
            "add_canon_rule",
            "Add a canon rule: kind lock (fields frozen for a character), fact (a field's canonical value), or note. Rules must reference existing characters.",
            schema(&[project(), ("rule", o())], &["project", "rule"]),
        ),
        (
            "list_canon",
            "All canon rules for a project.",
            schema(&[project()], &["project"]),
        ),
        (
            "check_canon",
            "Dry-run a proposed character/scene/dialogue/level write against the gate and report the verdict — allowed, or refused with the gate's code — without writing anything.",
            schema(&[project(), ("proposal", o())], &["project", "proposal"]),
        ),
        // group 3: scenes and systems
        (
            "declare_flag",
            "Declare a named flag (story, relationship, progression or system). Dialogue prerequisites and level gates may only reference declared flags.",
            schema(&[project(), ("flag", o())], &["project", "flag"]),
        ),
        (
            "upsert_scene",
            "Create or update a scene node (next, choices, triggers, cues). Referenced nodes and flags must exist or be declared; safety surfaces may never appear as progression gates.",
            schema(&[project(), ("node", o())], &["project", "node"]),
        ),
        (
            "upsert_dialogue",
            "Create or update a dialogue node: participants, lines, kink-legitimate actions with affirmative consent state and coverage, intensity 0-10, choices, prerequisites, continuity effects, and animation/voice/sound/haptic cues — all gate-checked.",
            schema(&[project(), ("node", o())], &["project", "node"]),
        ),
        (
            "get_node",
            "One graph node's full record.",
            schema(&[project(), ("node", s_max(64))], &["project", "node"]),
        ),
        (
            "list_nodes",
            "Graph nodes, optionally filtered by kind (scene or dialogue).",
            schema(
                &[project(), ("kind", en(&["scene", "dialogue"]))],
                &["project"],
            ),
        ),
        (
            "upsert_system",
            "Create or update a game system (schedule, economy, state_machine, minigame, progression). A system id may not collide with a safety surface.",
            schema(&[project(), ("system", o())], &["project", "system"]),
        ),
        (
            "list_systems",
            "All declared systems.",
            schema(&[project()], &["project"]),
        ),
        // group 4: reaction profiles
        (
            "set_reaction_profile",
            "Set a character's reaction profile: base_receptivity, sensitivity, conditioning, stamina (each 0-100), optional per-action modifiers and notes. Previews and playtests key on this.",
            schema(
                &[project(), ("character", s_max(64)), ("profile", o())],
                &["project", "character", "profile"],
            ),
        ),
        (
            "preview_reaction",
            "Deterministically preview a character's reaction under a context (relationship, arousal, fatigue, scene_history, prior_choices, action): score 0-100 and band (guarded/warming/engaged/ardent).",
            schema(
                &[project(), ("character", s_max(64)), ("context", o())],
                &["project", "character", "context"],
            ),
        ),
        // group 5: story, plot, levels
        (
            "upsert_arc",
            "Create or update a story arc with beats referencing graph nodes; mark ending arcs with ending=true.",
            schema(&[project(), ("arc", o())], &["project", "arc"]),
        ),
        (
            "validate_story",
            "Read-only story audit: beats referencing unknown nodes, nodes unreachable from the start node, declared endings.",
            schema(&[project()], &["project"]),
        ),
        (
            "upsert_level",
            "Create or update a level: node placements, minigame placements (must reference declared minigame systems), and progression gates whose target node exists and whose condition flag is declared. Safety surfaces can never be gate targets.",
            schema(&[project(), ("level", o())], &["project", "level"]),
        ),
        (
            "list_levels",
            "All levels.",
            schema(&[project()], &["project"]),
        ),
        (
            "generate_playtest",
            "Generate a deterministic playthrough of the authored graph under a seed (default seed 1) without recording it. The run hash is the replay contract.",
            schema(
                &[project(), ("seed", i64()), ("max_steps", i64())],
                &["project"],
            ),
        ),
        // group 6: content and safety systems
        (
            "get_safety",
            "The project's safety document: rating, age gate, content tags, all five surfaces, and the audit result.",
            schema(&[project()], &["project"]),
        ),
        (
            "set_safety_surface",
            "Configure one safety surface (stop, safeword, accessibility, aftercare, purge): presentation, wording, behaviour. Enabled and ungated are managed invariants — the gate refuses any attempt to disable a surface or progression-gate it.",
            schema(
                &[
                    project(),
                    (
                        "surface",
                        en(&["stop", "safeword", "accessibility", "aftercare", "purge"]),
                    ),
                    ("config", o()),
                ],
                &["project", "surface", "config"],
            ),
        ),
        (
            "audit_safety",
            "Run the safety audit alone: every surface present, enabled and ungated, plus the age-gate check for adult ratings.",
            schema(&[project()], &["project"]),
        ),
        // group 7: asset routing and provenance
        (
            "register_asset",
            "Register an asset record: id, kind, SHA-256 content hash, provenance (source required), and route_to naming its specialist owner (lumen, torori, ache, neemu, local, unassigned). Records only — Ardor never processes the asset.",
            schema(&[project(), ("asset", o())], &["project", "asset"]),
        ),
        (
            "list_assets",
            "All asset records and asset handoffs.",
            schema(&[project()], &["project"]),
        ),
        (
            "record_asset_handoff",
            "Record that an asset was handed to a specialist (lumen, torori, ache, neemu, slipway) or a registry channel. The asset must be registered first.",
            schema(
                &[project(), ("asset", s_max(64)), ("target", s_max(64))],
                &["project", "asset", "target"],
            ),
        ),
        // group 8: playtest and QA
        (
            "run_playtest",
            "Generate a playthrough under a seed AND record the run (hash, step count, stop reason) in the project's QA document for later replay verification.",
            schema(
                &[project(), ("seed", i64()), ("max_steps", i64())],
                &["project"],
            ),
        ),
        (
            "verify_replay",
            "Regenerate a recorded run from its seed and compare hashes: proof the authored graph still replays byte-identically.",
            schema(&[project(), ("run", s_max(64))], &["project", "run"]),
        ),
        (
            "add_screenshot_spec",
            "Record a screenshot specification (id, node, viewport, notes) for store/listing capture. The node must exist.",
            schema(&[project(), ("spec", o())], &["project", "spec"]),
        ),
        (
            "record_defect",
            "Record a QA defect (id, title, severity low/medium/high/critical, optional node and detail).",
            schema(&[project(), ("defect", o())], &["project", "defect"]),
        ),
        (
            "list_defects",
            "Defect records, optionally filtered by status (open matches records with no status yet).",
            schema(&[project(), ("status", s_max(32))], &["project"]),
        ),
        // group 9: channel registry + release handoff
        (
            "list_channels",
            "The channel-target registry: every English-language adult distribution channel with platforms, content policy, automation class, artifact requirements, handoff executor, auth kind, and live-verification status.",
            schema(&[], &[]),
        ),
        (
            "get_channel",
            "One channel's full registry record, including its credential/auth record.",
            schema(&[("channel", s_max(64))], &["channel"]),
        ),
        (
            "prepare_handoff",
            "Prepare a release handoff record for a channel. The gate refuses when: the channel is unknown or flagged, artifacts are missing or lack SHA-256 digests, no verified channel account exists (group 10), or the safety audit fails. Ardor prepares the record; the channel's executor performs the release.",
            schema(
                &[project(), ("channel", s_max(64)), ("artifacts", a())],
                &["project", "channel", "artifacts"],
            ),
        ),
        (
            "list_handoffs",
            "All prepared release handoffs for a project.",
            schema(&[project()], &["project"]),
        ),
        // group 10: channel credentials and sessions
        (
            "register_channel_account",
            "Register a channel account as a credential REFERENCE: channel, account identity, the channel's real credential kind, and store_ref naming where the value lives (primo pass path or Secure Vault entry). Values are refused — anything shaped like a raw secret, or under a secret-named field, fails the gate.",
            schema(&[("record", o())], &["record"]),
        ),
        (
            "list_channel_accounts",
            "All channel account records (references and status only), optionally filtered by channel. Pass now (ISO timestamp) to evaluate expiry.",
            schema(&[("channel", s_max(64)), ("now", s_max(40))], &[]),
        ),
        (
            "get_channel_account",
            "A channel's registry + auth record together with its account records and statuses.",
            schema(&[("channel", s_max(64)), ("now", s_max(40))], &["channel"]),
        ),
        (
            "record_auth_verification",
            "Record a verification attestation for an account: result (verified/failed), method, verified_at (ISO timestamp), optional detail. The probe itself runs on the value holder's side; Ardor records the outcome, never the value.",
            schema(
                &[
                    ("account_id", s_max(160)),
                    ("result", en(&["verified", "failed"])),
                    ("method", s_max(200)),
                    ("verified_at", s_max(40)),
                    ("detail", s_max(500)),
                ],
                &["account_id", "result", "method", "verified_at"],
            ),
        ),
        (
            "credential_status",
            "Per-channel credential rollup over the whole registry: auth kind, best status (verified/unverified/expired/missing), and the account records. Pass now (ISO timestamp) to evaluate expiry and rotation.",
            schema(&[("channel", s_max(64)), ("now", s_max(40))], &[]),
        ),
        (
            "probe_channel_auth",
            "Describe a channel's verification probe (e.g. itch.io key validation, Patreon token validation) and its current attested status. Ardor holds no value, so the probe executes on the estate-store side; this tool returns the specification, not a value.",
            schema(&[("channel", s_max(64))], &["channel"]),
        ),
        (
            "publish_preflight",
            "The publish preflight for a project + channel: channel known and verified, safety audit green, auth registered, verified, and (when now is supplied) unexpired — each as a named check.",
            schema(
                &[project(), ("channel", s_max(64)), ("now", s_max(40))],
                &["project", "channel"],
            ),
        ),
    ];
    specs
        .into_iter()
        .map(|(name, description, schema)| Tool::new(name, description, schema))
        .collect()
}

/// The server handler: one store behind a mutex; calls serialize,
/// and the blocking dispatch runs on a blocking thread so file I/O
/// cannot stall the runtime.
pub struct ArdorServer {
    store: Arc<Mutex<Store>>,
}

impl ArdorServer {
    pub fn new(store: Store) -> Self {
        ArdorServer {
            store: Arc::new(Mutex::new(store)),
        }
    }
}

impl ServerHandler for ArdorServer {
    fn get_info(&self) -> InitializeResult {
        let mut server_info = Implementation::default();
        server_info.name = "ardor".to_string();
        server_info.version = env!("CARGO_PKG_VERSION").to_string();
        let mut info = InitializeResult::default();
        info.protocol_version = ProtocolVersion::V_2026_07_28;
        info.capabilities = ServerCapabilities::builder().enable_tools().build();
        info.server_info = server_info;
        info.instructions = Some(INSTRUCTIONS.to_string());
        info
    }

    fn supported_protocol_versions(&self) -> Cow<'static, [ProtocolVersion]> {
        Cow::Borrowed(&[ProtocolVersion::V_2026_07_28])
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, ErrorData> {
        Ok(ListToolsResult {
            tools: tool_definitions(),
            ..Default::default()
        })
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        _context: RequestContext<RoleServer>,
    ) -> Result<CallToolResponse, ErrorData> {
        let name = request.name.to_string();
        let arguments = Value::Object(request.arguments.unwrap_or_default());
        let store = Arc::clone(&self.store);
        let outcome = tokio::task::spawn_blocking(move || {
            let mut guard = store.lock().expect("store mutex poisoned");
            tools::dispatch(&mut guard, &name, &arguments)
        })
        .await;
        match outcome {
            Ok(Ok(payload)) => Ok(CallToolResult::success(vec![ContentBlock::text(
                serde_json::to_string_pretty(&payload).unwrap_or_default(),
            )])
            .into()),
            Ok(Err(failure)) if failure.is_params => {
                Err(ErrorData::invalid_params(failure.message, None))
            }
            Ok(Err(failure)) => Ok(CallToolResult::error(vec![ContentBlock::text(
                serde_json::to_string(&json!({
                    "error": {"code": failure.code, "message": failure.message}
                }))
                .unwrap_or_default(),
            )])
            .into()),
            Err(join) => Err(ErrorData::internal_error(
                format!("tool task failed: {join}"),
                None,
            )),
        }
    }
}
