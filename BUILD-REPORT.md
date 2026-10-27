# Ardor — Stage 1 build report (Rust server)

**Commission:** D-124/D-125, plus D-126 (channel-target registry), D-126 addenda 2–3 (MiKandi struck; pixiv family added pre-sweep), D-127 (group 10, channel credentials & sessions).
**Date:** 2026-10-27. **Tree:** `~/workspace/ntranch-maker-mcp-20261027/ardor/` (sandbox) + primo mirror at `~/.local/share/ardor-mirror/`.
**Hard limits respected:** no commits, no pushes, no crates.io publish, no registry upload, no GitHub repo creation. Sandbox tree + primo mirror only.
**Stage 2 (Kotlin companion) is NOT built** — pending, per the commission.

Ardor is born on the fleet pattern proven in WS1: rmcp `=3.8.0`, edition 2024, estate pin `nightly-2026-09-25`, handler pinned to protocol version **2026-07-28 only**, and the byte-level era guard on stdio in front of the SDK (legacy `initialize` → typed `-32022` from the guard itself; framing errors `-32700`/`-32600`/`-32601` with the 4 MiB line bound; explicit `shutdown()` of the duplex write half on stdin EOF). The guard is the WS1 implementation adapted, not reinvented (one test frame renamed).

## What landed, per tool group (46 tools)

- **G1 Project scaffolding (4):** `create_project`, `get_project`, `list_projects`, `update_project_meta`. Projects are directories of JSON documents under the server home (`--home`, else `$ARDOR_HOME`, else `$HOME/.local/share/ardor`). Adult-rated projects require a non-empty age gate — enforced at create *and* on meta patches.
- **G2 Cast and canon (6):** `upsert_character`, `get_character`, `list_characters`, `add_canon_rule`, `list_canon`, `check_canon`. Canon `lock` rules freeze fields and `fact` rules pin values; a conflicting write is refused unless it carries an amendment with a reason, and amendments are recorded in the cast history (seq, version, changed fields; character `version` increments). `check_canon` is a dry-run decide() over a proposed write — verdict without mutation.
- **G3 Scenes and systems (7):** `declare_flag`, `upsert_scene`, `upsert_dialogue`, `get_node`, `list_nodes`, `upsert_system`, `list_systems`. Referenced nodes/flags must exist or be declared; branch targets must exist or be `END`.
- **G4 Dialogue & reactions (2 + `upsert_dialogue`):** dialogue is structured game data — participants (bare ids or `{character, role}` objects), lines, actions with integer intensity 0–10, node-level consent state that must be `affirmative` and whose participants must cover every action's actor/target, choices, prerequisites (only declared flags), continuity effects, and animation/voice/sound/haptic cue fields carried through as data. `set_reaction_profile` (fields 0–100, range-gated) and `preview_reaction` (pure deterministic integer formula → score + band guarded/warming/engaged/ardent).
- **G5 Story/plot/levels (5):** `upsert_arc`, `validate_story` (beat references, reachability from start, endings), `upsert_level`, `list_levels`, `generate_playtest`. Levels: minigame placements must reference declared minigame systems; progression gates need an existing target and a declared condition flag.
- **G6 Content & safety systems (3):** `get_safety`, `set_safety_surface`, `audit_safety`. The five surfaces — **stop, safeword, accessibility, aftercare, purge** — are scaffolded enabled and ungated at project creation. `enabled=true` and `progression_gated=false` are *managed invariants*: the gate refuses to disable a surface, and a recursive scan refuses any payload (scene, dialogue, system, level) that progression-gates a safety surface, however deeply nested. `audit_safety` checks every surface present/enabled/ungated plus the adult age gate.
- **G7 Asset routing & provenance (3):** `register_asset`, `list_assets`, `record_asset_handoff`. Records only — SHA-256 must be 64-hex, `provenance.source` is required, `route_to` must name the specialist owner (lumen / torori / ache / neemu / local / unassigned). No specialist domain is re-implemented.
- **G8 Playtest & QA (5):** `run_playtest` (generate + record), `verify_replay` (regenerate from the recorded seed, compare hash), `add_screenshot_spec` (node must exist), `record_defect`, `list_defects`. Playtest generation walks the authored graph from `start_node` with a SplitMix64 seed (scenes follow `next`, dialogue choices are drawn), bounded by a step budget and a per-node visit cap; the run hash is SHA-256 over the canonical (key-sorted) `{seed, start, steps}` — the replay contract.
- **G9 Channel registry & release handoff (4):** `list_channels`, `get_channel`, `prepare_handoff`, `list_handoffs` — over the registry below (D-126 as amended). `prepare_handoff` is a gate op: unknown channel → `unknown_channel`; flagged channel → `channel_flagged`; missing/bad-digest artifacts refused; safety audit must pass; **and** (D-127) a verified channel account must exist → `channel_auth_unverified`. Ardor prepares records; executors (butler / slipway / manual) perform releases. Publish-gate definitions stay owned by the publish skills — nothing is forked.
- **G10 Channel credentials & sessions (7):** `register_channel_account`, `list_channel_accounts`, `get_channel_account`, `record_auth_verification`, `credential_status`, `probe_channel_auth`, `publish_preflight`. **References and status, never values** — enforced in code (below).

## The heart: the decide() gate

Every canonical write builds an `Op`, runs the pure `decide(state, accounts, op)`, and only on approval runs `apply` + the store's write-temp-then-rename save (per changed document only). A refusal returns before any mutation or save. The integration suite proves the property end-to-end: a battery of refused writes (safety gating, surface disable, missing consent, undeclared flag, flagged/struck channel handoffs) leaves **every file in the server home byte-identical** (full-tree byte comparison, before vs after). Refusals are typed (`isError` content `{"error":{"code","message"}}` with stable codes); params failures are protocol `-32602` with the fleet's exact messages; unknown tools are `-32602`.

**Group 10 enforcement detail.** Account records live in a studio-level `accounts.json` (channel, account identity, credential kind, `store_ref` naming a primo `pass` path or a Secure Vault entry, scopes, expiry, verification attestation). Three independent refusals guard the no-values rule: (1) a **secret-material scan** rejects payloads carrying secret-named fields (`api_key`, `token`, `secret`, …) or secret-shaped strings (known prefixes, JWT shape, long mixed-class runs — checked per whitespace token too, so a value pasted into a prose field is still caught) → `secret_value_refused`; (2) the registered credential kind must equal the channel's *real* kind → `auth_kind_mismatch` (no invented token flows for portal-only channels); (3) flagged channels refuse account registration → `channel_flagged`. Probes are specifications, not executions: Ardor holds no value, so e.g. itch.io key validation runs on the value holder's side and the outcome is recorded as an attestation via `record_auth_verification`. `publish_preflight` reports every check by name (channel known/verified, safety audit, auth registered/verified/unexpired).

## Gates — numbers

| Gate | Result |
|---|---|
| `cargo test` (estate pin nightly-2026-09-25) | **29 passed, 0 failed** — 24 unit + 5 integration (`tests/mcp_smoke.rs`), incl. byte-identical-state, secret-refusal, legacy-refusal, and deterministic-hash batteries |
| `cargo clippy --all-targets -- -D warnings` | **clean** (exit 0) |
| `cargo fmt --check` | **clean** |
| MCP Inspector, `--protocol-era modern --method tools/list` | **exit 0** |
| MCP Inspector, modern `tools/list --strict` | **exit 0** (no schema errors) |
| MCP Inspector, modern `tools/call list_channels` | **exit 0**, real registry payload |
| MCP Inspector, `--protocol-era legacy --method initialize` | **exit 1**, stderr `{"error":{"code":"error","message":"Unsupported protocol version"}}` — the typed refusal, as designed |
| Diver gate (primo mirror, headless nvim, live config) | **PASS — 11/11 Rust files, rust-analyzer attached, total diagnostics 0** (`~/.local/share/ardor-mirror/diver-gate-results.txt`) |
| D-123 marker sweep | Case-insensitive sweep for the SFW-lane namespace markers over the whole crate tree (excluding `target/`): **0 hits** |

Test balance is ≈50/50 validation/adversarial per estate rule: round-trip + determinism + registry shape on one side; legacy/framing refusal, gate-refusal byte-identity, consent/safety refusals, and secret-material refusal on the other.

## Channel registry (D-126 as amended) — verification record

20 records. Verified live during the build on 2026-10-27 (web sources; per-channel notes are in `src/channels.rs`). Flagged channels stay in the registry with their evidence; the gate refuses handoffs (and account registrations) to them.

| Channel | Category / class / executor | Auth kind (G10) | Status |
|---|---|---|---|
| itch.io | storefront / pipeline / butler | **butler_key** (API key; probe: `GET /api/1/<key>/me`) | verified |
| Nutaku | platform / prepared_submission / slipway | portal_login | verified |
| DLsite English | storefront / prepared_submission / manual | portal_login | verified |
| FAKKU | storefront / curated_pitch / manual | portal_login | verified |
| JAST USA | storefront / curated_pitch / manual | portal_login | verified |
| Denpasoft | storefront / curated_pitch / manual | portal_login | verified |
| MangaGamer | storefront / curated_pitch / manual | portal_login | verified |
| Kagura Games | storefront / curated_pitch / manual | portal_login | verified |
| Johren | platform / prepared_submission / manual | portal_login | verified |
| GOG | storefront / curated_pitch (selective) / manual | portal_login | verified |
| EroLabs | platform / prepared_submission / manual | portal_login | verified (reinstated by D-126) |
| Newgrounds | portal / prepared_submission / manual | portal_login | verified |
| Gamcore | portal / prepared_submission / manual | portal_login | verified, with caution (no public self-serve dashboard or revenue terms confirmed) |
| Self-hosted F-Droid (NTRanch) | self_hosted / pipeline / slipway | signing_key (estate-held reference) | verified (estate route, D-095) |
| Patreon | membership / prepared_submission / manual | **oauth_token** (Patreon API v2 creator token; probe: `/api/oauth2/v2/identity`) | verified |
| SubscribeStar | membership / prepared_submission / manual | portal_login | verified |
| Zoom Platform | storefront / prepared_submission / manual | portal_login | **FLAGGED** — operating DRM-free store, but acceptance of explicit adults-only titles unverified |
| pixiv | presence / prepared_presence / manual | portal_login (shared pixiv account) | **FLAGGED — UNVERIFIED-pending-sweep** (D-126 addendum 3) |
| BOOTH | storefront / prepared_submission / manual | portal_login (shared pixiv account) | **FLAGGED — UNVERIFIED-pending-sweep** (payout route not established) |
| FANBOX | membership / prepared_submission / manual | portal_login (shared pixiv account) | **FLAGGED — UNVERIFIED-pending-sweep** (payout route not established) |
| ~~MiKandi~~ | — | — | **STRUCK pre-build** (D-126 addendum 2) — removed on Owner-supplied verified evidence: defunct (Wikipedia: "a defunct mobile adult software applications store"; last app release 2018). No channel record and no credential record were built. This row records the live-verification rule working; it is not a build verification failure. |

Steam is out (D-095); Google Play is SFW-lane only and not a target. The pixiv family is modelled as the ruling specifies: pixiv proper sells nothing (gallery/funnel presence routing to the arms), BOOTH is the one-off sales arm, FANBOX the membership arm, and one shared pixiv account identity spans all three credential records. Payout/onboarding routes for the family are marked **UNVERIFIED-pending-sweep** in the records themselves — never invented; the pending sweep's findings will refine them (handoffs stay refused until then).

## D-116 utilization evidence

**Named consumer workflows:** Strobe, Torogao, MoonDeep, and Hands-On authoring, and the hypnosis-remake program (Modification App first) — all author games as Ardor project state and release through the registry's executors.

**Recorded real invocation:** `invocation-transcript.txt` (same directory as this report) captures one full session against the built binary: discover → scaffold an adult project → two characters → flag → scene/dialogue authoring (with consent-clean action data) → reaction profile + preview → `run_playtest` (seed 7, hash `14b271c23aef8468e568ee2785041d015be7a27a753fa9d62f0fa4aa59a088f8`) → `verify_replay` match → itch.io account reference registered → verification attested → `publish_preflight` all-green → `prepare_handoff` prepared (executor: butler) → and a final `set_safety_surface` disable attempt refused `safety_surface_mandatory` on the record. Server exited 0 on EOF. The integration suite additionally replays the same arc in-process and across a process restart.

## Premise corrections (stated as corrections)

1. **Participant and prerequisite shapes.** The gate as first written accepted only object-shaped dialogue participants (`{"character": id}`) and only bare-string prerequisites. Both are wrong as sole shapes — authoring uses bare ids *and* `{character, role}` objects, and string *and* `{"flag": …}` prerequisites. Found by the smoke suite the hard way: a dialogue write was being silently refused while the round-trip test still passed, because an `isError` payload also parses as JSON. The gate now accepts both shapes in both places, and the tests assert *stored state* (node present, walk traverses ≥2 nodes) rather than mere payload parseability.
2. **MiKandi.** The original D-126 target set included MiKandi; D-126 addendum 2 struck it on verified evidence before any record shipped (see the struck row above). My in-progress flagged record for it was deleted, not shipped.
3. **Inspector era default.** The MCP Inspector CLI negotiates the *legacy* era by default; modern runs require `--protocol-era modern`. The default-mode run doubles as the legacy-refusal evidence above.

## Scope choices, stubs, deferred

- Per the commission's tie-break, depth went to the decide()-gate write path, groups 1–6, and group 10's enforcement; groups 7–9 are deliberately records-level, matching their brief boundaries (records and handoffs only — no lumen/torori/ache/neemu domains, no slipway gate forks).
- **Kotlin companion: Stage 2, not built** (pending, per the commission).
- **Pixiv family records** await the separate channel sweep (English accessibility, seller onboarding, US-seller payout) — flagged in-registry until it lands.
- **Credential probes** execute on the value holder's side by design; Ardor records attestations. No tool in the server can receive, store, or emit a credential value — the adversarial tests assert this against state files and tool outputs.
- Reaction preview is a deterministic integer model (documented formula in `src/playtest.rs`), a first pass for authoring feedback — not a simulation of record.
- No commits, pushes, publishes, or repo creation were performed; the primo tree is a read-only mirror for the Diver gate.
