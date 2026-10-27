# Ardor companion (Stage 2)

The Kotlin half of Ardor, the NTRanch game/app maker MCP. The
Rust server (Stage 1, `../ardor/`) is the authoring surface; this
companion is the Android-side half: a modern-era MCP client for
driving the server, and an embeddable on-device QA module for
Ardor-authored games and apps.

## Modules

- **`:client`** — a modern-era (2026-07-28) MCP client session
  over stdio, built on the official Kotlin MCP SDK 0.16.0's
  transport and core types (the D-118 pattern: the SDK's
  high-level `Client` still performs the legacy `initialize`
  handshake, so a small session layer drives its machinery
  discover-first, with per-request `_meta`, and never sends
  `initialize`). Drives the real Ardor server: discover,
  tools/list, tool calls.
- **`:qa`** — an Android library that runs Ardor's own checks
  on-device against an exported project bundle (the server's
  JSON documents, as a directory or a .zip):
  - the safety audit, ported line-for-line from the server's
    `audit_safety` (five surfaces present/enabled/ungated +
    adult age gate, same check names and detail strings);
  - deterministic playtest replay: the same SplitMix64 stream
    and canonical-hash contract as the server's
    `generate_playtest`, so an on-device run hash can be compared
    byte-for-byte against the server's recorded run;
  - Android channel readiness evidence for prepared handoffs
    (Nutaku APK, itch APK, self-hosted F-Droid): artifact
    presence + SHA-256 against the handoff record. Evidence for
    slipway — it issues no certification and forks no publish
    gates.
- **`:app`** — a demo app proving the embedding. Two flavors:
  `store` (the shipped shape — the QA module is not even a
  dependency) and `qa` (module + an on-device battery activity,
  separate applicationId, never submitted).

## Toolchain

AGP 8.13.2 · Kotlin plugin 2.4.21 · Gradle 8.14.4 (wrapper) ·
JDK 21 · kotlin-sdk 0.16.0 · compileSdk/targetSdk 36, minSdk 26.
(The Kotlin pin matters: kotlin-sdk 0.16.0 ships Kotlin-2.4
metadata that older plugins cannot read.)

## Fixtures

`fixtures/evidence-game/` is the D-116 evidence project (the
server's JSON documents, produced by the real server), and
`fixtures/server-home/` is a server home containing it plus a
verified channel-account record, for the client interop gate.
The reference run: seed 7, max_steps 32, hash
`14b271c23aef8468e568ee2785041d015be7a27a753fa9d62f0fa4aa59a088f8`.
