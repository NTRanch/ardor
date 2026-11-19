# Keeper chains (`keeper-chain 1.0`)

Stage 4 of the Ardor pipelines programme (ARDOR-PIPELINES.md §4,
as amended by Amendment A1, Owner ruling 2026-11-18). Ardor holds
the keeper chain of record for every shipped asset: the canon
references the asset was generated against, the exact prompt
pair, the model checkpoint and its hash, the conditioning, the
seed, the finish operations in order, the final-file SHA-256, and
the verdict with its ruling reference. A chain whose provenance
cannot be re-derived from the record is refused, never narrated.

## The record

Chains live in a typed `chains` document in the project store
(commission decision (a): a document of their own, following the
`canon.rs` pattern — not extension fields on the asset record).
The field set generalizes ART-BIBLE §1.1, written there for
images, to every shipped asset kind (decision (b): `sprite`,
`model3d`, `audio`, `haptic`, `text`, `build`, `other` — the
group-7 vocabulary): `game` becomes the containing project,
`record_id` names the cast character the asset belongs to where
one applies, and `finish_ops` entries carry an optional `region`
(the ART-BIBLE's per-step region note, generalized past images).
Registration input never carries the server-managed fields
(`version`, `state`, `session_record`, the held record and
conditioning); unknown fields are refused at the tool layer.

## The session-record field

Every chain carries the typed session-record state (decision
(c)): `absent-unverified` at manual intake, `present-unverified`
once a `torogao-scene-record/1` record has been intaken, and
exactly `{state: verified, format: "torogao-scene-record/1",
record_sha256}` once activated — the Amendment A1 shape. The
field is never absent and never free-form: a chain that cannot
show a session record says so in its type.

## Activation (Amendment A1: Ardor-side)

A chain activates when Ardor's own `keeper-chain 1.0` validator
passes its four legs over what Ardor holds — no Torogao code is
linked, spawned, or consulted, and no Ardor stage waits on a
Torogao build:

- **(a)** the held session record validates *whole* against the
  published `torogao-scene-record/1` schema (decision (d): full
  structural validation in `src/scene_record.rs` — the four
  medium blocks exactly once, recipe honesty: a byte-identity
  rerender claim requires a pinned execution context, a
  best-effort claim requires its reason; provenance
  epsilon/tolerance discipline; frozen-track and palette-snap
  bounds);
- **(b)** the recipe's `conditioning_sha256` equals the SHA-256
  Ardor *derives* from the conditioning record it holds
  (canonical JSON, sorted keys) — a transcribed hash is never
  compared against itself;
- **(c)** the chain's final-file SHA-256 equals the registered
  asset's SHA-256 (Ardor holds hashes, not asset bytes; the
  asset's hash was itself refused at registration without
  provenance);
- **(d)** the record's custody map is well-formed (unique node
  paths, typed open/frozen states); custody refusals Torogao has
  issued are recorded verbatim in the chain and no Ardor verb
  overrides a custody freeze.

Pass: the chain is `active`, its session-record field reads
verified, and a §7 conformance run record exists. Any leg
failing is a typed refusal (`record_schema_invalid`,
`chain_hash_mismatch`, `chain_incomplete`,
`chain_already_active`) and the chain stays in manual-intake
form; refused writes leave the tree byte-identical.

Re-registering an active chain against a different final-file
hash, or intaking a new session record over an active chain,
returns it to manual-intake form — verification is earned
against specific bytes and does not carry over (the census
staleness discipline, applied to chains).

## Verbs (group 7, decision (e))

`register_chain`, `list_chains`, `get_chain`,
`run_chain_validation`, `intake_session_record`,
`activate_chain`, `record_custody_refusal` — 54 → 61 tools.
Validation and activation runs are recorded in the chains
document in the §7 shape (`pipeline: "keeper-chain"`,
`pipeline_version: "1.0"`, input/output artifact hashes,
verdict, refusal codes, ruling references).

## Honest limits

Canon-reference and checkpoint hashes are shape-checked, not
byte-checked: Ardor holds neither the canon images nor the
checkpoint bytes. Finish operations are recorded, not replayed —
replay belongs to the finishing tools. Leg (c) binds the chain
to the asset *record*; byte-level custody of the asset file
itself is Torogao's downstream integration surface, non-gating
per Amendment A1.
