# canon-census 1.0 — fixture evidence

origin_lane: ntranch
content_label: nsfw (the fixture text itself is all-ages synthetic)

Committed output of one `canon-census 1.0` run over a synthetic
fixture canon, recorded 2026-11-18 (Stage 2, Pax takeover of the
rate-limited lane). The live Ardor server home holds 0 projects
(finding F-2 in the lane's STATUS.md), so this fixture is the
census run of record for the Stage 2 gate; classification batches
over a live canon remain proposals for Pax to gate (ST2-2).

## The fixture

Built through the public tool surface only, by
`ardor/examples/canon_census_fixture.rs` (shared verbatim with
`ardor/tests/canon.rs`):

- project `census-fixture` (all-ages, genesis seeded);
- denylist ruled as genesis state: "Studio Prime",
  "Jane Realperson", "Project Realname" (ruling FIXTURE-DENYLIST-1);
- characters `wren` and `sorrel` (descriptor fields hair, style);
- canon rules: a lock on wren.hair, a fact sorrel.hair = "red";
- required table `signature-traits`: 2 declared / 2 found rows,
  wren's trait cell frozen (ruling FIXTURE-CANON-1);
- cast pair `foreign-cast-a`: Mina Voss shares exactly half of
  wren's descriptor tokens; Petra Ilves shares none; Owner
  threshold 500 per-mille (ruling FIXTURE-DIFF-1).

## The run

- verdict: **VERIFIED**
- input state hash:
  `9ba39f3dd8b6ec49674ae01702540897895081f6bcc7f66eb332655b234e8224`
- census artifact hash (canonical form):
  `e13e8079eb2536f3b0590cb9db063a13184cacdc79fc64f966c625adc0a83017`
- counts: characters 2, canon rules 2, tables 1 (2/2 rows),
  cast pairs 1, denylist terms 3, orphans 0, sanitization hits 0
- differentiation: max overlap 500 per-mille, at threshold,
  verdict `within_threshold`, no name collisions
- files: `census-artifact.json` (sha256
  `71df66e3d5d2e5128590f86efb1053db7907c08dca20304d79244b3e92dee430`),
  `run-record.json` (sha256
  `dd9665bee7dcafc2d61176731294458ff73e5df7f6f6b15f2ebf4aa715f1bc8d`)

## Reproduce

```sh
cargo run --example canon_census_fixture -- <empty-home-dir> <out-dir>
```

regenerates both files byte-identically (asserted continuously by
the test `validation_fixture_artifact_matches_committed_evidence`).
