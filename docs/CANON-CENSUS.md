# Canon Census — Stage 2 (canon-census 1.0)

origin_lane: ntranch
content_label: nsfw

Implements ARDOR-PIPELINES.md §2 (Canon as Data) and the §7
conformance shape. Gate of record: `canon-census 1.0`.

## What was added

- **Denylist as genesis state** (Stage 1 extension). Tool
  `record_denylist` replaces the whole list (studio names, real
  persons, real projects) as one genesis ledger entry of kind
  `denylist`, Owner ruling reference required, at most 1024 terms,
  no blank or fold-duplicate terms. The genesis ledger replay and
  run re-derivation cover the new kind, and the genesis-sheet
  schema admits it. The census can only sweep with a denylist that
  re-derives from its ledger; an unruled denylist is the finding
  `denylist_unruled`, never an empty sweep.
- **Canon tables as data** (group 2). Tool `declare_canon_table`:
  typed columns (`asset`, `boolean`, `character`, `integer`,
  `string`), one row per fact with a ruling reference, a declared
  row total, an optional `required` flag (every character must
  appear), and per-cell `frozen` marks. Changing or unfreezing a
  frozen cell is refused (`canon_frozen_cell`) unless the write
  carries an amendment with a reason; every redeclaration bumps
  the table version and appends a `table_history` entry recording
  any frozen changes.
- **Cast pairs as data** (group 2). Tool `declare_cast_pair`: a
  foreign cast's members with declared descriptor fields, a source
  reference, and optionally the Owner's overlap threshold in
  per-mille with its ruling reference.
- **The census** (group 2, beside `check_canon`). Tools
  `run_canon_census` (records a §7 run) and `get_canon_census`
  (read-only status). The census is a pure function of canon
  state — group-2 records and rules, declared tables and pairs,
  the genesis denylist, asset ids, and the shippable text of
  cast/graph/levels/story — and reports:
  - counts per table against declared totals (`count_mismatch`);
  - orphans in both directions: rule subjects, row keys, and
    typed cell references that name no record, and records a
    required table omits (`canon_orphan`);
  - differentiation per pair: folded name equality is a collision
    outright; descriptor overlap is the stated numeric leg —
    integer per-mille Jaccard over folded `field=value` tokens —
    reported for the Owner's ruling, or judged against a recorded
    threshold (`differentiation_collision`, `differentiation_unruled`);
  - a sanitization sweep of shippable text against the denylist,
    hits listed by document, record, and JSON pointer
    (`sanitization_hit`). Safety documents are never swept,
    read, or gated: the census does not touch group 6.

## Verified means

A project state's canon is *verified* only while the latest
recorded run's input hash equals the current canon state's hash,
the run re-derives exactly, and its verdict is `VERIFIED`.
`canon::require_verified` is the check downstream gates call:
`canon_unverified` (no run, or a run whose findings carry no more
specific code), `census_stale` (any canon write after the run),
`census_run_mismatch` (a recorded run that does not match its
artifact hash, sequence, or current state — REVIEW under D-713),
or the run's own `canon_orphan` / `differentiation_collision` /
`sanitization_hit`.

## Evidence

- `ardor/tests/canon.rs`: 7 validation + 7 adversarial tests.
- `ardor/fixtures/canon-census-1.0/`: the fixture census run of
  record (VERIFIED) with its run record and reproduction steps.
  The live server home holds 0 projects, so the fixture is the
  census run of record for this gate; batches over a live canon
  remain Pax-gated proposals (tracker ST2-2).
