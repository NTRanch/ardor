# Ardor — To-1.0 Programme Tracker (pipeline completion)

Owner ruling 2026-11-17 (Matt): this file is the live tracker for
completing Ardor as a game-building/rebuilding pipeline. Ardor the
authoring MCP is published (crates.io 1.0.1; ten tool groups, 46
tools at the brief census). What "1.0" means here is the five
production pipelines of `ARDOR-PIPELINES.md` (2026-11-03, D-839)
built and proven: a title carried genesis -> handoff entirely
through Ardor. The runtime half of a playable rebuild is torogao's
road, not this tracker's — Ardor's bar is authoring-through-handoff,
held explicitly.

Every lane reads this file before starting work and updates its
rows as work moves.

## How to work this tracker

- Statuses: `TODO` -> `CLAIMED` (add lane + date) -> `IN-PROGRESS`
  -> `DONE-PENDING-GATE` -> `DONE` (Pax flips this after the gate)
  or `BLOCKED` (add the reason in the row).
- Claim before you build. One lane per task.
- Commit tracker updates with the work they describe, on your task
  branch. NTRanch pushes from primo do not work (read-only SSH
  identity + the insteadOf trap): landings route through Pax via
  the NTRanch connector. Merges to `main` are the Owner's word.
- Gates are Pax's: tests + clippy + fmt by exit code (never through
  a pipe), theodolite where called for. A gate number cites its
  run artifact (D-713).
- Lane law: this is the NTRanch lane. Nothing from here flows into
  qompassai repos unless it is SFW and labelled (bloch lane law).
- Lane key: PAX, CLAUDE (Opus lane), ASTRA (Codex lane),
  AZOTH (Grok; SFW-shaped research only — no adult content, per his
  standing boundary), LOCAL (rose/granite local models; bounded
  batches, proposals only, Pax-gated), OWNER (Matt's act).

## Stage 0 — Reachable shipped source (VERIFIED 2026-11-17, Pax)

| ID | Task | Lane | Status | Notes |
|---|---|---|---|---|
| ST0 | Prove the shipped source is reachable and identify its home | PAX | DONE | Evidence (primo, 2026-11-17): origin/main `8e2db90` carries `8c1b9ed` "ardor 1.0.0 published" — the 1.0.0 shipped tree IS on GitHub main (the 2026-11-03 spec's unreachable finding is superseded). The 1.0.1 release commit `b2b0495` (ardor/Cargo.toml version 1.0.1, 101 files) lives on the pushed branches `pax/agent-card-20261116` (tip `f6d1fdd`, this tracker's base) and `pax/ardor-tui-20261116`; it is NOT merged to main. Remaining act: merging the 1.0.1 lineage to main is the Owner's word (the DFR-1 posture). Pipeline stages build on the branch lineage meanwhile. |

## Stage 1 — Genesis (S)

| ID | Task | Lane | Status | Notes |
|---|---|---|---|---|
| ST1-1 | Defaults-profile record + genesis-sheet schema and census | ASTRA | DONE | ASTRA 2026-11-17: versioned proposal profile, schema/census, seed hashes and conformance runs. See docs/GENESIS.md; Owner profile values await ruling. |
| ST1-2 | Group-1 extension pair: `record_default_ruling`, `get_genesis_state`; ledger as project state | ASTRA | DONE | ASTRA 2026-11-17: extension pair and append-only ledger; typed refusals and deterministic history. 120 workspace tests (base 106); workdir REPORT.md carries gate artifacts. |
| ST1-3 | Clearance scan ships as a skill in the same commission (checklist + evidence grading), sheet recorded by tool | ASTRA | DONE | ASTRA 2026-11-17: ~/workspace/custom-skills/ardor-name-clearance/SKILL.md; record_clearance_sheet intake. Skill validates; trust recorder missing (reported). |
| GATE-S1 | Stage 1 gate | PAX | TODO | Unlocks enforcement for Stages 2 and 4. |

## Stage 2 — Canon census (M)

| ID | Task | Lane | Status | Notes |
|---|---|---|---|---|
| ST2-1 | `canon-census 1.0`: counts, orphans, differentiation, sanitization, staleness — extending group 2 beside `check_canon` | CLAUDE + PAX (takeover) | DONE-PENDING-GATE | Denylist as genesis state (ledger kind `denylist`), canon tables + cast pairs as group-2 data, census + §7 run records + staleness in `ardor/src/canon.rs`; tools 49 → 54. Gates (takeover, 2026-11-18): tests 134/0, clippy -D warnings rc=0, fmt rc=0, theodolite PASS control_count 2. See docs/CANON-CENSUS.md and the lane REPORT.md. |
| ST2-2 | Census classification batches over the live canon (proposals only; Pax gates every write) | CLAUDE + PAX (takeover) | DONE-PENDING-GATE | Live canon is empty (server home: 0 projects, lane finding F-2), so the batch of record is the census run over the committed fixture canon: VERIFIED, artifacts in `ardor/fixtures/canon-census-1.0/`. Batches over a live canon remain proposals for Pax to gate when a live canon exists. |
| GATE-S2 | Stage 2 gate | PAX | TODO | |

## Stage 3 — Content census, sealing, contract packs (M-L)

| ID | Task | Lane | Status | Notes |
|---|---|---|---|---|
| ST3-1 | `content-census 1.0` + unit gate states and sealing | CLAUDE | TODO | The design-risk stage: sealing semantics. |
| ST3-2 | Pack definitions and application across groups 5/8; contract marks in run/defect records | CLAUDE | TODO | |
| ST3-3 | `pack_touches_safety` refusal | CLAUDE | TODO | Refusal behaviour is gated adversarially (half the tests). |
| GATE-S3 | Stage 3 gate | PAX | TODO | |

## Stage 4 — Keeper chains (M, then S)

| ID | Task | Lane | Status | Notes |
|---|---|---|---|---|
| ST4-1 | `keeper-chain 1.0` schema + validator as a group-7 extension, manual-intake form (session-record field carrying `absent-unverified`) | ASTRA | TODO | Registry half; waits on Stages 1-2 only. |
| ST4-2 | Activation: session-record intake as the normal route, custody refusals recorded verbatim, manual intake as fallback | ASTRA | BLOCKED | Waits on the D-838 torogao lift builds (session records, custody maps, freeze verbs), themselves stacked on the gencond branch landing — the Owner's call, separate programme. |
| GATE-S4 | Stage 4 gate (registry half) | PAX | TODO | |

## Stage 5 — Handoff packet (S-M)

| ID | Task | Lane | Status | Notes |
|---|---|---|---|---|
| ST5-1 | `handoff-packet 1.0` + extended preflight check list | PAX + ASTRA | TODO | Mostly assembly: every check it names exists as a gate by the time this runs. |
| ST5-2 | Double-bake evidence wiring | ASTRA | TODO | |
| GATE-S5 | Stage 5 gate | PAX | TODO | Waits on Stages 2-4. |

## The proof

| ID | Task | Lane | Status | Notes |
|---|---|---|---|---|
| PROOF-1 | One NTR rebuild title carried genesis -> handoff entirely through Ardor, conformance record per pipeline (spec S7) | PAX + ASTRA + CLAUDE | TODO | Candidate: Magic Coin (its rebuild is furthest along). Fathom supplies disassembly-to-source study evidence (D-1587) — that is fathom's lane, consumed here, not rebuilt here. Playability additionally requires torogao's runtime; this proof's bar is the handoff packet, stated plainly. |

## Owner gates on the critical path

- The DFR-1 word: merge the 1.0.1 lineage to `main` (nothing blocks
  on it except main's freshness; stages build on branches).
- The gencond branch landing (unblocks ST4-2 via the D-838 lift
  programme).
