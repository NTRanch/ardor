# Changelog — Ardor

## 1.0.0 — 2026-10-28

The 1.0 close-out of the 0.1.0 maker surface. No tool was added,
removed, or re-shaped: the surface stays at **46 tools in ten
groups**, and every 0.1.0 gate was re-run on this tree.

### Added

- **REQ-9 structural projection (D-235)** — `ardor/src/projection.rs`:
  the allowlist projection specified in
  `docs/REQ-9-PROJECTION.md` (identifiers, counts, verdict and
  presence booleans cross; canon content never does; drops
  counted; results labelled `sfw-structural` / origin
  `ntranch`). Pure and inert — not wired into tool dispatch, no
  new MCP tool.
- **EFLO record** — `EFLO.md`: the live 46-tool `tools/list`
  measured at **3,549 tokens (cl100k_base)** against the ≤2,500
  fleet budget, with a D-147-class waiver recommendation stated
  on the record (surface not shrunk; see the file for grounds
  and the revisit path).
- **COMPLIANCE.md** — the compliance record for the shipped
  surface: era posture, the `decide()` gate, U5 safety
  invariants, credential references-never-values, EFLO
  disposition, and the publication/registry state.

### Changed

- Version 0.1.0 → **1.0.0** (`ardor/Cargo.toml`, lockfile,
  `server.json`, README currency).
- Crate restructured lib + bin (`ardor/src/lib.rs`): the
  projection and the gate/store machinery are a library surface;
  the `ardor` binary is a thin shell over it. No behaviour
  change; the test battery count is unchanged in total.
- Kotlin companion interop re-run green on the 1.0.0 tree
  (evidence: `.ai/memory/evidence/kotlin-interop-1.0.0.md`).

## 0.1.0 — 2026-10-27

The initial landed release (IDEA-0096, D-124/D-125): a
born-modern rmcp server (modern MCP era only, 2026-07-28) with
46 tools in ten groups — project scaffolding; cast and canon;
scenes and systems; kink-legitimate dialogue as structured
data; story, plot, and levels; content and safety systems
(U5: stop, safeword, accessibility, aftercare, purge —
scaffolded enabled and ungated, never disable-able, never
progression-gated); asset routing and provenance; playtest and
QA with deterministic replay; build and release handoff over a
47-channel verified registry; and channel credentials managed
as references and status, never values. Every canonical write
passes the pure `decide()` gate before any mutation. Kotlin
MCP SDK companion (modern-era client + on-device QA library)
with interop 1/1. MCP Inspector strict `tools/list` exit 0.
