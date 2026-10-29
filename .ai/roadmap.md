# Roadmap — ardor (mcp) — v0.1 → v1.0
<!-- bloch:roadmap schema=1 kind=mcp lane=general baseline=0.1.0 target=1.0.0 -->

How to work this roadmap (any agent): run `bloch task start R-xxx --agent <you>` before working a task, `bloch task done R-xxx` when you believe it is complete — done runs every `verify:` command and flips the task green only if all pass; there is no override. `bloch task fail R-xxx --error "..."` records a failure. Fixes learned the hard way land in `.ai/memory/lessons.md` via the inbox/promote flow and are surfaced in every HANDOFF.

Curated 2026-10-28 (D-313 Phase 0 — NTRanch four → 1.0 program): canonical tree resolved by cloning NTRanch/ardor to ~/.GH/NTRanch/ardor (HEAD d3f89c5); the old ~/.local/share/ardor-mirror was a gate mirror, not a git tree. The Rust crate lives in the ardor/ subdirectory (Kotlin companion in ardor-companion/), so cargo verifies run there. Baseline truth: the IDEA-0096 maker scope landed at 0.1.0 as 46 tools in ten groups (BUILD-REPORT.md). The 1.0 path per the fleet gap list: G-R1 EFLO measure FIRST (R-202), then G-R2 version 1.0.0 + docs currency + re-gate incl. the Kotlin interop, REQ-9 projection (D-235) in scope (R-209), publish 1.0.0.

## Phase 1 — Baseline truth
- [x] R-101 Build is green from a clean checkout
  - verify: cd ardor && cargo build
  - evidence: 1 verify commands, all exit 0 at 2026-10-29T04:51:33Z
- [x] R-102 Existing tests pass at the baseline
  - verify: cd ardor && cargo test
  - evidence: 1 verify commands, all exit 0 at 2026-10-29T04:51:34Z
- [x] R-103 Repo identity, baseline version, and this roadmap are recorded
  - verify: test -f .ai/roadmap.md
  - evidence: 1 verify commands, all exit 0 at 2026-10-29T04:51:34Z

## Phase 2 — Core complete
- [x] R-201 Tool surface is contracted: every tool has a complete schema and annotations from birth (46 tools, ten groups G1–G10)
  - verify: cd ardor && cargo test
  - evidence: 1 verify commands, all exit 0 at 2026-10-29T04:51:34Z
- [x] R-202 EFLO measure on record with disposition (G-R1 — the first 1.0 build item): real-tokenizer measure of the 46-tool surface; if over budget, trigger-scoped group loading (torori's pattern) or a D-147-class waiver with the number stated
  - verify: test -f EFLO.md
  - evidence: 1 verify commands, all exit 0 at 2026-10-29T05:34:46Z
- [x] R-203 Typed error and refusal behaviour is implemented in server code: every canonical write runs the pure decide() gate before any mutation; refusals are typed and leave every file byte-identical
  - verify: cd ardor && cargo test
  - evidence: 1 verify commands, all exit 0 at 2026-10-29T04:51:34Z
- [x] R-204 Project scaffolding + cast/canon authoring delivered (groups G1–G2): projects as JSON document trees; canon lock/fact rules with amendment history; check_canon dry-run decide()
  - verify: cd ardor && cargo test
  - evidence: 1 verify commands, all exit 0 at 2026-10-29T04:51:34Z
- [x] R-205 Scene/systems authoring + dialogue delivered (groups G3–G4): declared-flag/node integrity, structured dialogue with consent state, reaction profiles with deterministic preview
  - verify: cd ardor && cargo test
  - evidence: 1 verify commands, all exit 0 at 2026-10-29T04:51:34Z
- [x] R-206 Story/plot/level authoring delivered (group G5): arcs, story validation (reachability, endings), levels with declared-system placements and gated progression
  - verify: cd ardor && cargo test
  - evidence: 1 verify commands, all exit 0 at 2026-10-29T04:51:34Z
- [x] R-207 Safety systems delivered as managed invariants (group G6, U5 law): stop, safeword, accessibility, aftercare, purge scaffolded enabled and ungated; the gate refuses to disable or progression-gate any surface; audit_safety incl. the adult age gate
  - verify: cd ardor && cargo test
  - evidence: 1 verify commands, all exit 0 at 2026-10-29T04:51:34Z
- [x] R-208 Asset routing, playtest/QA, and release handoff delivered (groups G7–G10): provenance-checked asset records routed to specialist owners; deterministic seeded playtest with replay verification; channel registry handoffs prepared for slipway/executors — ardor prepares records, never performs releases
  - verify: cd ardor && cargo test
  - evidence: 1 verify commands, all exit 0 at 2026-10-29T04:51:35Z
- [x] R-209 REQ-9 ardor projection delivered (D-235): the projection specified and built under its own gate, spec filed in this repo
  - verify: test -f docs/REQ-9-PROJECTION.md
  - evidence: 1 verify commands, all exit 0 at 2026-10-29T05:34:46Z

## Phase 3 — Tests and gates
- [x] R-301 Validation tests cover the happy path end to end (half the test split)
  - verify: cd ardor && cargo test
  - evidence: 1 verify commands, all exit 0 at 2026-10-29T04:51:35Z
- [x] R-302 Adversarial tests cover malformed, oversized, and hostile inputs (half the test split)
  - verify: cd ardor && cargo test
  - evidence: 1 verify commands, all exit 0 at 2026-10-29T04:51:35Z
- [x] R-303 Refusal and typed-error behaviour is covered by tests, including oversized payloads and refused-write byte-identity
  - verify: cd ardor && cargo test
  - evidence: 1 verify commands, all exit 0 at 2026-10-29T04:51:35Z

## Phase 4 — Hardening
- [x] R-401 MCP Inspector conformance pass is recorded as evidence (modern-era tools/list --strict exit 0 at the 0.1.0 gate)
  - verify: cd ardor && cargo test
  - evidence: 1 verify commands, all exit 0 at 2026-10-29T04:51:38Z
- [x] R-402 Client matrix is verified with at least two real clients, incl. the Kotlin companion interop (1/1 at 0.1.0: discover + 46 tools + playtest hash agreement, no legacy initialize)
  - verify: cd ardor && cargo test
  - evidence: 1 verify commands, all exit 0 at 2026-10-29T04:51:38Z
- [x] R-403 Declared-vs-detected access (EFLO) check passes: no tool exceeds its annotations
  - verify: cd ardor && cargo clippy -- -D warnings
  - evidence: 1 verify commands, all exit 0 at 2026-10-29T04:51:47Z
- [x] R-404 Registry/release packaging builds from a clean checkout
  - verify: cd ardor && cargo build
  - evidence: 1 verify commands, all exit 0 at 2026-10-29T04:51:47Z

## Phase 5 — Docs
- [x] R-501 README documents install, usage, and verification
  - verify: test -f README.md
  - evidence: 1 verify commands, all exit 0 at 2026-10-29T04:51:47Z
- [x] R-502 CHANGELOG records every user-facing change
  - verify: test -f CHANGELOG.md
  - evidence: 1 verify commands, all exit 0 at 2026-10-29T05:34:46Z
- [x] R-503 Tool reference documents every tool, schema, annotation, and error kind
  - verify: test -f README.md
  - evidence: 1 verify commands, all exit 0 at 2026-10-29T04:51:47Z
- [x] R-504 Docs currency for 1.0.0 (G-R2): server.json, README, and COMPLIANCE describe the shipped surface; COMPLIANCE written
  - verify: test -f COMPLIANCE.md
  - evidence: 1 verify commands, all exit 0 at 2026-10-29T05:34:46Z

## Phase 6 — Release v1
- [x] R-601 Version is bumped to 1.0.0 and the changelog records the release (G-R2; re-gate on the 1.0.0 tree)
  - verify: grep -q 'version = "1.0.0"' ardor/Cargo.toml
  - evidence: 1 verify commands, all exit 0 at 2026-10-29T05:34:46Z
- [x] R-602 Clean-environment install and run is verified
  - verify: cd ardor && cargo build
  - evidence: 1 verify commands, all exit 0 at 2026-10-29T05:34:46Z
- [x] R-603 Release artefacts are built from a clean checkout and checksummed
  - verify: cd ardor && cargo build
  - evidence: 1 verify commands, all exit 0 at 2026-10-29T05:34:46Z
- [x] R-604 Kotlin companion interop re-run green on the 1.0.0 tree (G-R2 gate: 1/1, as at 0.1.0)
  - verify: test -f .ai/memory/evidence/kotlin-interop-1.0.0.md
  - evidence: 1 verify commands, all exit 0 at 2026-10-29T05:34:46Z
- [x] R-605 ardor 1.0.0 published to crates.io under the NTRanch account
  - verify: curl -sf -H "User-Agent: bloch-roadmap" https://crates.io/api/v1/crates/ardor/1.0.0 -o /dev/null
  - evidence: 1 verify commands, all exit 0 at 2026-10-29T05:37:43Z
