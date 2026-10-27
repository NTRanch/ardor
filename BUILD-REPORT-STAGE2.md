# Ardor — Stage 2 build report (Kotlin companion)

**Commission:** Ardor Stage 2 (D-124/D-125 follow-on), Owner direction 2026-10-27: finish Ardor so it helps the NSFW games/apps NTRanch will publish as Android apps.
**Date:** 2026-10-27. **Tree:** `~/workspace/ntranch-maker-mcp-20261027/ardor-companion/` (sandbox; new sibling tree — nothing under `ardor/` was modified). Build/gate host: primo (staging at `~/.local/share/ardor-companion-build/tree/`), because the sandbox has no JDK.
**Hard limits respected:** no commits, no pushes, no publishes, no repo creation.

## What landed

### 1. `:client` — Kotlin MCP client, modern era only (JVM module)

`ArdorMcpSession` (client/src/main/kotlin/ai/ntranch/ardor/client/): a discover-first session layer over the official Kotlin MCP SDK 0.16.0's `StdioClientTransport` and modern core types — the D-118 pattern, applied to stdio. The SDK's high-level `Client` still performs the legacy `initialize` handshake (its `LATEST_PROTOCOL_VERSION` is 2025-11-25), so the session drives the SDK's transport/types directly: `server/discover` first, per-request `_meta` (protocolVersion `2026-07-28`, clientInfo, clientCapabilities) on every request, `initialize` never sent (asserted in the interop gate from the session's own sent-method log). API: `discover()`, `listTools()`, `callTool(name, args)` returning the SDK's typed results.

### 2. `:qa` — embeddable Android QA library (`ai.ntranch.ardor.qa`)

Pure-Kotlin core (no Android APIs in the logic, so it is fully JVM-testable), reading an exported Ardor project bundle — the server's JSON documents as a directory or a .zip (`DirectoryBundle` / `ZipBundle` / `MapBundle`; `project.json` required, other documents default to empty objects, mirroring the Rust store):

- **Safety audit on-device** (`SafetyAudit`): a line-for-line port of the Rust `safety_audit` (gate.rs is the spec of record) — all five surfaces present/enabled/ungated, check names (`safety_surface:<name>`) and detail strings byte-identical to the server's, plus the adult age-gate check, which is also exposed as its own reported item (`SafetyAudit.ageGateCheck`, and a separate line in the runner's report).
- **Deterministic playtest replay on-device** (`PlaytestReplay` + `SplitMix64` + canonical JSON in `Json.kt`): the same graph walk, the same SplitMix64 stream, and SHA-256 over the canonical `{seed, start, steps}` — the canonical writer reproduces the Rust `canonical()` byte-for-byte (sorted keys, serde_json string escaping). The seed is carried as an unsigned 64-bit value end-to-end so the full server seed range hashes exactly.
- **Android channel readiness** (`ChannelReadiness`): for each prepared handoff in `releases.json` targeting a registry Android channel (nutaku, itch_io, fdroid_self_hosted — the channels whose registry platform list includes Android), it checks the handoff carries an `.apk` artifact (all three channels' recorded artifact requirements demand a signed/Android APK) and that every recorded artifact is present on the device with bytes hashing to the recorded SHA-256. This prepares evidence for slipway; it issues no certification and forks no publish gate.
- **`QaRunner`** composes the battery into one structured report (JSON + PASS/FAIL summary lines), replaying the latest server-recorded run from `qa.json` when the bundle carries one, so the device hash is compared against the server's own record.

### 3. `:app` — demo app proving the embedding

Flavors on a `channel` dimension: **`store`** (default; the QA library is not a dependency at all) and **`qa`** (`applicationIdSuffix ".qa"`, separate app id, never submitted) with `QaActivity` — extracts a demo bundle + artifact from assets and renders the battery's PASS/FAIL lines (machine-readable via `uiautomator dump`, as in D-118). The demo bundle is the evidence-game project with a Nutaku handoff whose `demo.apk` digest matches the shipped placeholder bytes, so every readiness check can genuinely pass on-device.

### Toolchain

Per the D-118 forced pins: **AGP 8.13.2 · Kotlin plugin 2.4.21 · Gradle 8.14.4 (wrapper) · JDK 21** (/usr/lib/jvm/java-21-openjdk on primo) · kotlin-sdk **0.16.0** · kotlinx-coroutines 1.11.0 · compileSdk/targetSdk 36, minSdk 26, jvmTarget JVM_17. One machine note: the wrapper had to be generated in a scratch directory — AGP 8.13.2 cannot even configure under primo's installed Gradle 9.8.2 (it uses a Gradle internal API removed in 9.6; the error names the fix), after which `./gradlew` runs everything under 8.14.4 as pinned. Unlike ONTrack, no `android.builtInKotlin`/`newDsl` flags were needed (those are AGP 9.x concerns).

## Gates — numbers

| Gate | Result |
|---|---|
| Gradle build (`./gradlew build`, all modules, lint included) | **BUILD SUCCESSFUL** (274 tasks) |
| `:qa` JVM unit tests | **22/22 passed, 0 skipped** — ReplayTest 8, AuditReadinessTest 14; ≈50/50 validation/adversarial (see below) |
| **Cross-implementation hash gate** | **PASS, byte-identical** — see evidence below |
| `:client` interop test (Kotlin client ↔ real Rust server binary, stdio) | **1/1 passed** — discover versions `["2026-07-28"]`, tools/list = 46 tools, `get_project`, `run_playtest` (seed 7 → hash `14b271c2…`), `publish_preflight` itch_io all-checks-ok; sent methods `[server/discover, tools/list, tools/call ×3]` — no `initialize`. Transcript: `ardor-companion/interop-transcript.txt` |
| Emulator battery (primo, AVD `ontrack-pixel`, API 36 x86_64, swiftshader; audio off) | **PASS 10/10 lines** via `uiautomator dump`: 5 safety surfaces + age gate (audit), age gate as its own item, `replay_recorded_hash — 14b271c2…`, channel readiness nutaku + artifact digest match; final line `ARDOR-QA DONE 10/10 PASS`. Emulator shut down afterwards |
| Release exclusion (artifact level) | **PASS** — store-release APK scanned across all 14 entries (raw + UTF-16LE): companion markers `ai.ntranch.ardor.qa` / `ai/ntranch/ardor/qa`, `PlaytestReplay`, `SafetyAudit`, `ChannelReadiness`, `SplitMix64`, `QaRunner`, `ProjectBundle`, `QaActivity`, `ARDOR-QA`, `demo-bundle` → **0 hits**. (The only `ntranch/ardor` hits, 2, are the demo app's own `ai.ntranch.ardor.demo` classes — the app itself, present by design.) Positive control: the qa-debug APK scores 375 hits on the same marker set, so the scan demonstrably detects the companion when present |
| D-123 marker sweep (companion tree, case-insensitive) | **0 hits** |

**Test balance (:qa, ≈50/50).** Validation: fixture audit passes with the server's exact check shape; age gate as own item; fixture replay = server hash; canonical byte shape pinned against a literal; replay determinism; SplitMix64 vectors (independently computed in Python); zip bundle = directory bundle; readiness pass for all three Android channels; runner green on a satisfiable bundle. Adversarial: wrong seed changes the hash; tampered graph changes the hash; progression-gated surface refused; missing surface refused; disabled surface refused; stripped age gate refused; tampered artifact bytes fail the digest; missing artifact fails; Android handoff without an `.apk` fails; the fixture's own unsatisfiable itch handoff makes readiness — and only readiness — fail in the runner.

## Cross-implementation hash evidence

Path taken: **reconstructed the D-116 project**, as preferred. The 31 request frames in Stage 1's `invocation-transcript.txt` were replayed verbatim against the server binary (a copy staged for the gates); the server's `run_playtest` (seed 7, max_steps 32) reproduced `14b271c23aef8468e568ee2785041d015be7a27a753fa9d62f0fa4aa59a088f8` and `verify_replay` agreed. That server home became the fixture (`fixtures/server-home/`, plus `fixtures/evidence-game/`). Then, three independent Kotlin computations matched that hash byte-for-byte:

1. `:qa` JVM test — Kotlin replay over the fixture → `14b271c2…` (plus the canonical string pinned to its exact literal form);
2. `:client` interop — the Kotlin client drove the server itself to the same hash in-session;
3. On-device — the emulator battery's replay line reports the same hash from the Android runtime.

## Honest limits / stubs

- **The client module is JVM, not Android.** A stdio client needs a server process to spawn; Android apps do not host the Rust server binary. The client's role is the host-side driver (authoring tools, CI, gate scripts) — exactly the role the D-118 QA script played — while the on-device half of the companion is the `:qa` module. The session layer itself is platform-neutral Kotlin except for `java.util.Process` stream wiring.
- **Emulator, not hardware:** the on-device battery ran on primo's x86_64 API-36 emulator (the D-118 AVD). No physical-device run.
- **Unsigned release artifact:** `assembleStoreRelease` runs without a signing config, so the exclusion proof is over the unsigned APK. Signing adds a signature block, not code; the flavor scoping (no `:qa` dependency in `store`) is the mechanism, and the scan verifies it.
- **Readiness channel set is a snapshot:** the three Android channel ids are taken from the server's registry as of this build (channels.rs). If the registry grows an Android channel, this set needs the same one-line update; deriving it from an exported registry document is future work, noted rather than built.
- **JSON parser scope:** bundle documents with integers outside the signed-64 range would fail to parse; the server's own documents never produce those (its hashed content is strings + u64 seeds, and the seed path here is unsigned-exact).
- The interop binary was the sandbox-built server (from the annex-updated `ardor/` tree, copied — never edited — for the gates) running natively on primo; its discover response and hash behaviour matched the Stage 1 record.
