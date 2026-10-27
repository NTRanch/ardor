# Ardor

**Works of passion for passionate people.**

Ardor is NTRanch's game/app maker MCP server: one authoring surface for
building adult (18+) games and apps end to end — project scaffolding,
cast and canon, scenes and systems, kink-legitimate dialogue as
structured data, story and levels, content-safety systems, asset
routing, deterministic playtest QA, release handoffs to 47 verified
distribution/service channels, and channel credential *references*
(never values).

- MCP Registry name: `mcp-name: io.github.NTRanch/ardor`
- Source: https://github.com/NTRanch/ardor
- License: Apache-2.0

## Protocol era: modern only (2026-07-28)

Ardor is built on [rmcp](https://crates.io/crates/rmcp) 3.8 and serves
the modern MCP era only. Sessions open with `server/discover`; there is
no legacy `initialize` handshake. Legacy contact is refused at the
stdio front door with the typed `-32022` unsupported-version error
naming `2026-07-28` as the one supported version. State rides on
explicit arguments, never on protocol-session affinity.

## The ten tool groups (46 tools)

1. **Project scaffolding** — create/inspect projects with the studio
   conventions in place. Adult-rated projects require a non-empty age
   gate at creation and on every metadata patch.
2. **Cast and canon** — character identity records, canon `lock`/`fact`
   rules, and versioned cast changes. Conflicting writes are refused
   unless they carry a recorded amendment.
3. **Scenes and systems** — dialogue/scene graphs, flags, schedules,
   progression DAGs, economies, state machines, minigame definitions.
   References must resolve or the write is refused.
4. **Dialogue and reactions** — adult dialogue as structured game
   data: sex actions, character reactions, consent and safety state,
   participant roles, prerequisites, intensity, continuity effects,
   branching, and animation/voice/sound/haptic cues. Per-character
   reaction profiles drive a deterministic preview.
5. **Story, plot, and levels** — arcs, beats, quests, character and
   relationship progression, branching endings, encounters, and level
   composition with triggers and progression gates, plus deterministic
   playtest generation from the authored graph.
6. **Content and safety systems** — stop, safeword, accessibility,
   aftercare, and Purge surfaces are scaffolded **enabled and
   ungated**, and that is a managed invariant: the write gate refuses
   to disable a surface or to progression-gate one, however deeply
   nested the payload. Includes content tags, rating metadata, and
   age-gate patterns.
7. **Asset routing and provenance** — intake, hash, provenance, and
   routing records. Specialist work stays with its owner (lumen for
   sprite-to-3D, torori for audio, ache for haptics, neemu for
   screened corpus work); Ardor records handoffs, it does not
   re-implement domains.
8. **Playtest and QA** — scripted playthrough definitions,
   deterministic replay/hash verification (SplitMix64 walk, SHA-256
   over the canonical run record), screenshot specifications, and
   defect records tied to authored project state.
9. **Build and release handoff** — a channel-target registry of 47
   English-language adult distribution, asset, and service channels,
   each verified live at build time and organised in six sections
   (distribution, own-storefront payment rail, adult assets, SFW-only
   assets, services, SFW-only services). `prepare_handoff` is gated:
   the safety audit must pass and a verified channel account must
   exist. Ardor prepares records; executors (butler for itch.io,
   slipway for Android channels, manual elsewhere) perform releases.
10. **Channel credentials and sessions** — credential **references
    and status, never values**. Records hold the channel, account
    identity, credential kind, where the value lives (a `pass` path
    or a Secure Vault entry), scopes, expiry, and verification
    attestations. Secret-shaped payloads are refused by scan
    (`secret_value_refused`); invented token flows for portal-only
    channels are refused by kind (`auth_kind_mismatch`); publish
    preflight refuses handoffs whose channel auth is missing or
    unverified.

## The write gate

Every canonical write runs through a pure `decide()` gate before any
mutation: canon locks, consent coverage, declared-reference checks,
and the safety invariants are enforced there, and a refused write
leaves the on-disk project state byte-identical (proven by the
integration suite). Refusals are typed errors with stable codes.

## Build and run

Requires the estate Rust pin (see `rust-toolchain.toml`,
nightly-2026-09-25).

```sh
cargo build --release
./target/release/ardor [--home <dir>]
```

The server speaks MCP over stdio. Its home directory holds one
directory per authored project plus the studio-level channel-account
document. The home defaults to `$ARDOR_HOME`, else
`$HOME/.local/share/ardor`.

## Companion

The Kotlin companion (`ardor-companion/` in the source repository)
provides a modern-era MCP client for host-side drivers and an
embeddable Android QA library that re-runs the safety audit, age-gate
check, deterministic replay, and channel artifact readiness on-device.
Its replay hash matches the server's byte-for-byte. The QA module is
debug/QA-only and is excluded from release artifacts by construction.

## Boundaries

Ardor is an authoring and project-state server. It is not a runtime
or orchestrator, not a live session host, and not a publish
certifier: release gates stay owned by the publish skills and their
executors.
