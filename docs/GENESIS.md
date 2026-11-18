# Stage 1: Genesis

A new project has seven visible defaults, all `inherited`. Reading its state
never confirms them. The Owner confirms the block or re-rules individual
items; the recorded reference is an attestation, not authentication of the
Owner. Agents must not invent rulings.

The versioned profile is `ardor/schemas/ntranch-defaults-1.0.json`. Its values
are commissioning proposals, deliberately marked `proposal`; no sibling
project's platform, pricing, connectivity, voice or language answer is adopted.
An Owner profile ruling remains open. Projects can record their own rulings
now. Accessibility's enabled/ungated invariant cannot be re-ruled away.

The structural contract is `ardor/schemas/ardor-genesis-sheet-1.0.schema.json`.
The gate of record is **ardor-genesis-sheet 1.0**, checked by the Rust census.
The sheet holds the profile, seven defaults, candidate/adoption, immutable
channel targets, denylist (initially empty), seed manifest, and decision ledger.
The stored `genesis.json` adds conformance runs outside the sheet to avoid
self-hashing. It is canonical key-sorted JSON, saved by temp-and-rename.

## Group 1 extensions

- `get_genesis_state({project})`: read sheet/hash, census findings and runs.
  Incomplete or missing legacy state is REVIEW, never silently confirmed.
- `record_default_ruling({project,rulings,ruling_ref})`: `rulings` is 1–7 unique
  `{item,state,value}` objects. `state` is `confirmed` (exact profile value) or
  `re-ruled` (a replacement). One reference can confirm the whole block. Each
  accepted call appends one D-numbered ledger event and one conformance run.
- `record_clearance_sheet({project,sheet,ruling_ref,adopt?})`: record the skill's
  research. `adopt` defaults false. True attests an Owner adoption ruling,
  including explicit acceptance of every note when present. COLLISION always
  refuses adoption. A new candidate supersedes the previous pick; previous
  sheets and rulings remain in the ledger. Every new sheet invalidates prior
  adoption unless adopted with its own ruling.

`create_project` additionally accepts `channel_targets`, an array of unique
registry ids. Required clearance legs are `package:crates.io`,
`namespace:github`, `namespace:mcp`, `storefront:itch_io`, `trademark`, `domain`,
and `storefront:<id>` for each additional intended target. An inapplicable leg
needs an evidence-backed scope finding; it cannot simply disappear.

Each leg contains `id`, `finding`, `verified`, `evidence` (nonempty
`{url,grade}` array, grade `primary`/`secondary`), `owed_steps`, and `outcome`.
The overall `verdict` and leg `outcome` use `CLEAR`, `CLEAR WITH NOTES`, or
`COLLISION`. The sheet also contains `candidate`, `legs`, and `notes`.
Secondary-only evidence cannot be marked verified, and requires named owed
steps. Any unverified leg requires owed steps. Any notes or owed steps force
CLEAR WITH NOTES unless a collision forces COLLISION. Acceptance retains the
notes and owed steps; it does not manufacture verification.

The research procedure lives outside this repository at
`~/workspace/custom-skills/ardor-name-clearance/SKILL.md`. No server tool does
network research. Evidence grades and ruling references are caller
attestations; Ardor validates consistency and retains evidence, not truth.

## Census and provenance

Completion requires every default ruled, a consistent complete clearance sheet,
Owner adoption referencing that exact sheet hash, five hash-pinned seed files,
and a current conformance artifact. The census re-derives defaults, candidate,
clearance and adoption from the ledger. Historical run artifact hashes are
checked, and reads reject a missing/stale current run as REVIEW.

Runs store the pipeline/version, gate id/version, exact prior genesis state
hash, output sheet hash and snapshot, verdict/findings and ruling references.
Snapshots retain every prior ruling and seed content, allowing later review
without mutable external research pages as the sole witness. A failed write
records no run because refusal must leave the entire store byte-identical.

`repo-seed/` contains LICENSE (explicit license-selection placeholder, not a
license grant), README, design skeleton, test-layout skeleton and ledger index.
The manifest pins their UTF-8 bytes. Genesis reads and writes check the materialized files
against those hashes; other groups, including safety, remain accessible. Edits are reported as a mismatch, never silently
re-baselined. These are immutable seed artifacts; a working repository can be
copied from them and evolved separately. No git repository is initialized.

Metadata name changes require adoption of that exact candidate. Existing
project data is not retroactively migrated. Stages 2–5 consume this ledger and
add their downstream gates in their own commissions; this stage does not
implement canon census, keeper chains, content sealing or handoff packets.
Safety operations remain available throughout incomplete genesis.

## Stable refusal codes

`genesis_missing`, `genesis_ledger_mismatch`, `invalid_default_ruling`,
`default_value_changed`, `safety_surface_immutable`, `ruling_ref_required`,
`invalid_clearance`, `clearance_evidence_unverified`,
`clearance_verdict_mismatch`, `name_conditional`, `clearance_collision`,
`seed_manifest_mismatch`. Census-only findings additionally include
`defaults_unconfirmed`, `genesis_run_mismatch`,
`genesis_run_missing_or_stale`. Missing leg and default findings identify the
responsible ids. Malformed shapes and unknown arguments are params errors.
