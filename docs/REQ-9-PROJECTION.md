# REQ-9 — Ardor structural projection

**Status:** built (ardor side) at 1.0.0 — `ardor/src/projection.rs`,
gated by this crate's test battery.
**Authority:** Owner decision **D-235** (2026-10-28): Seahawk may
receive ardor's structural projection under the SPEC's terms.
**Spec of record:** `seahawk-wiring-20261028/SPEC.md` §4 ("The
ardor lane boundary") and `CLOSEOUT-FILINGS.md` REQ-9 ("Ardor
structural-projection route (lane-gated)").
**Roadmap:** R-209.

## What REQ-9 is

Seahawk (the SFW-lane broker) brokers engine sessions. An
NTRanch-lane engine session sometimes needs project/canon
*context* from ardor (NTRanch lane) — "does this scene's canon
check pass", "which screenshot specs exist" — without canon
*content* ever crossing the lane boundary. REQ-9 is that
crossing, fenced: a default-off, config-enabled, read-only
route whose every response passes through an allowlist
**structural projection** before it can reach a result or a log.

## Terms (SPEC §4, as allowed by D-235)

1. **Default off.** The route exists only when the consumer's
   deployment config carries an ardor entry. The shipped
   example config carries none. Enabling it is a deployment act
   in an NTRanch-lane lab config, never a default.
2. **Read-class lookups only.** The consumer may call
   `get_project`, `list_projects`, `check_canon`,
   `list_characters` (identity fields only), screenshot-spec
   QA metadata, and `list_defects` (counts/status). It **never**
   calls an ardor write — ardor's `decide()` gate is not the
   consumer's to invoke.
3. **Structural projection.** Responses pass through the
   allowlist below before crossing. Every dropped field is
   counted (`ardor_projected_fields_dropped: n` in the
   consumer's op log) so the boundary's operation is auditable.
4. **Nothing ardor-origin is persisted in the consumer's tree.**
   The consumer's log records that a lookup happened, the
   projection counts, and the structural scalars returned — in
   an NTRanch-lane deployment's log only.
5. **Direction of travel.** Structural metadata flows
   ardor → consumer for the duration of a brokered session.
   Canon content stays in ardor and is read by the calling
   agent through its own ardor session, never relayed.

Lane law (BLOCH §9): sharing is by *content label*, not origin.
Every projected result is labelled
`content_label = "sfw-structural"` with
`origin_lane = "ntranch"` explicit — NTRanch origin is stated,
never laundered, and the projection is what earns the
structural label: it is constructed from allowlisted scalars
only, deny-by-default.

## The allowlist

| Source | Crosses | Never crosses |
|---|---|---|
| `get_project` | project id, project name, all `counts`, `safety_audit_ok` | engine, rating, age-gate text, content tags |
| `list_projects` | project id (slug), project name | engine, rating |
| `check_canon` | op name, `canon_check_pass` boolean (from the verdict) | refusal code, refusal message |
| `list_characters` / `get_character` | project id, character ids | names, descriptions, appearance, cast history — everything else |
| `list_canon` | project id, rule ids, rule kinds | fact bodies (`field`/`value`), lock `fields` payloads |
| `get_safety` / `audit_safety` | project id, per-surface **presence booleans** for the five U5 surfaces, audit `ok` | surface configuration, age-gate text, check details |
| screenshot specs (`add_screenshot_spec` echo — no list tool exists) | project id, spec ids | node bindings, framing text |
| `list_defects` | project id, defect ids, defect statuses, count | titles, bodies |

Project names are the one free-text exception, named explicitly
by SPEC §4 ("project id and name"). Any tool with no named
projection — every write tool, every unknown name — projects to
an **empty** structural object with all fields counted dropped.

**Counting rule:** one drop per non-allowlisted field
occurrence (a field dropped from each of N array records counts
N drops).

## The ardor-side implementation

`ardor/src/projection.rs` is the reference implementation of
the projection itself, built and gated in this repo:

- `project(tool, payload) -> Projection { value, dropped }` —
  pure, no I/O, no store access, no mutation.
- The result `value` is the envelope
  `{content_label, origin_lane, tool, data}`.
- **Inert by design:** the module is never invoked by the
  server's tool dispatch and adds **no MCP tool** — the surface
  stays at 46 tools. It weakens nothing: the U5 safety
  invariants, the `decide()` gate, and the credential
  references-never-values rule are untouched (safety surfaces
  cross as presence booleans only, never as configuration).
- Tests (in-module, validation + adversarial halves): the
  allowlisted structure crosses intact; seeded canon markers in
  dialogue-class payloads, character descriptions, canon fact
  bodies, defect titles, and safety configuration appear in
  **no** projected value; unlisted tools and non-object
  payloads project to nothing with drops counted.

## What remains consumer-side (not claimed here)

The Seahawk-side wiring — the default-off `[[tooling]]` config
entry, the `canon.context` tool registered only when
configured, the stdio client route, and the op-log drop
counter — is Seahawk's build under its own brief and gate. This
document and the ardor-side module are the ardor half of
REQ-9: the projection specified, built, and proven against the
real payload shapes ardor emits. D-235 grants no write-back, no
content transfer beyond the allowlist, and no change to
Seahawk's `decide()` charter; nothing here exceeds it.
