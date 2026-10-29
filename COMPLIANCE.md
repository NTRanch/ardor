# Compliance — Ardor 1.0.0

The compliance record for the shipped surface, current as of
the 1.0.0 close-out (2026-10-28). Surface of record: **46 tools
in ten groups** (G1–G10, IDEA-0096), unchanged from 0.1.0; the
1.0.0 delta is the REQ-9 projection module (no new tool), the
EFLO record, and this document set.

## Protocol posture

- **Modern MCP era only (2026-07-28)**, rmcp 3.8.0 (fleet pin,
  D-113). Sessions open with `server/discover`; legacy
  `initialize` contact is refused at the stdio front door with
  the typed `-32022` error naming 2026-07-28. Stateless /
  discover-first; state rides on explicit arguments.
- MCP Inspector `--cli --protocol-era modern --method
  tools/list --strict` exited 0 at the 0.1.0 gate; the surface
  is byte-identical at 1.0.0 (no tool definition changed), and
  the full test battery was re-run on this tree.

## Enforcement

- **Every canonical write passes the pure `decide()` gate**
  (canon + safety) before any mutation; a refused write leaves
  every state file byte-identical (test-proven). Params
  failures are protocol errors (`-32602`); domain failures are
  `isError` results carrying a stable typed code.
- **U5 — safety systems are managed invariants.** The five
  surfaces (stop, safeword, accessibility, aftercare, purge)
  are scaffolded **enabled and ungated** in every project; the
  gate refuses any attempt to disable one or to
  progression-gate one, directly or by payload smuggling
  (recursive scan). Release handoffs require a passing safety
  audit, including the adult age-gate check. 1.0.0 changed none
  of this; the REQ-9 projection reads safety state as presence
  booleans only and cannot configure a surface.
- **Credentials are references and status, never values**
  (group G10). No tool stores a raw secret in project state,
  returns one in output, or writes one to a log; secret-shaped
  payloads are refused by scan (adversarially tested with a
  seeded fake secret that must appear in no state file and no
  tool output).

## EFLO

`tools/list` measures **3,549 tokens (cl100k_base)** — 1,049
over the ≤2,500 fleet budget. Full measure, per-tool table,
and disposition in `EFLO.md`: **a D-147-class waiver is
recommended on the record; the surface was not shrunk and the
number is not waived until the Owner rules.** Revisit path:
trigger-scoped group loading if an always-on consumer appears
or the surface grows.

## REQ-9 projection (D-235)

`docs/REQ-9-PROJECTION.md` + `ardor/src/projection.rs`: the
lane-boundary projection for Seahawk's default-off,
read-class context route. Allowlisted structural scalars only;
canon content is dropped and the drops counted; every result
is labelled `content_label = "sfw-structural"`,
`origin_lane = "ntranch"`. The module is inert (never invoked
by tool dispatch) and grants no write-back.

## Lane and publication

- **NTRanch lane only.** D-123 sweep: publishable metadata,
  docs, and `server.json` carry zero markers of the SFW-lane
  estate (re-run at 1.0.0; the sweep covers the files this
  close-out ships).
- crates.io: package `ardor`, published under the NTRanch
  account (owner id 445961). Registry name (case-sensitive):
  **`io.github.NTRanch/ardor`**.
- Ardor prepares release records and handoffs; it never
  performs releases — slipway and the channel executors own
  publish gates (group G9 discipline).
- Kotlin companion: modern-era client + on-device QA library;
  Android-facing pieces are feature-gated and compiled out of
  store artifacts unless explicitly enabled (companion README).
  Interop re-run on the 1.0.0 tree: see
  `.ai/memory/evidence/kotlin-interop-1.0.0.md`.
