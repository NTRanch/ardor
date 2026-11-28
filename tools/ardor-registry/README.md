# tools/ardor-registry — MoonDeep portrait-lock registry

The durable form of the MoonDeep portrait lock, built per
`TOOLING-PLAN.md` T2 (BUILD-NOW, commissioned 2026-11-27). One JSON
registry + a stdlib-only Python CLI. The registry is the single home
for facts that previously lived as hand-typed canon-pin tables in
every art lane's record: which frame is current for each girl, what
it superseded, and the Owner ruling that put it there.

## Files

- `ardor-registry` — the CLI (executable, Python 3 stdlib only).
- `registry.json` — the registry of record. **Lock state is versioned
  state**: it lives here, in the repo, and changes to it are commits.
- `moondeep/` — the same registry migrated into the ardor MCP
  portrait tools (see "MCP form" below): the verbatim ardor store
  directory the migration produced, loadable by pointing an ardor
  server home at its parent.
- `README.md` — this file.

## Registry schema (`moondeep-portrait-registry/1`)

`registry.json` holds `girls` (the seven) and `entries`, one per
frame of record:

| field | meaning |
|---|---|
| `id`, `path` | absolute path of the frame where it lives (the art trees under `~/workspace/moondeep-*`) |
| `girl`, `set` | owner girl; producing set/lane (`set1`, `torogao`, `canvas-text`, `canvas-carry-c1`, `canvas-carry-judged`, `canvas-fresh`, `heightfix`, `edge-cure`, the regen strike sets, …) |
| `sha256` | recomputed from the file bytes at seed/registration time — never copied from a record |
| `status` | `working-set` · `candidate` · `superseded` · `defective` · `locked` |
| `supersedes`, `superseded_by` | the cure/replacement chain, by path |
| `ruling` | `{date, quote}` — the Owner ruling that set the status, where one did |
| `note` | provenance, named defects, inheritance caveats |

Seeded 2026-11-27 with 139 entries covering all of that day's sets.
The three full-colour restrike lanes were still in flight at seed
time; their delivered frames are registered when they land (add by
hand in this schema, or via `lock` when the Owner picks finals).

## Usage

Run from anywhere; the CLI finds `registry.json` beside itself
(override with `--registry PATH` or `$ARDOR_REGISTRY`).

```sh
# Pre-flight before striking on a base: is it still current?
tools/ardor-registry/ardor-registry check /path/to/base.png
#   CURRENT <path> (status: ...)            exit 0 — strike away
#   SUPERSEDED-BY <newer path>              exit 1 — use the newer base
#   DEFECTIVE <path>                        exit 1 — gated out (reject/framing)
#   UNKNOWN <path>                          exit 2 — not a frame of record

# Record the Owner's lock for a girl (step 1's closing act):
tools/ardor-registry/ardor-registry lock solenne /path/to/final.png \
    --ruling "Owner lock, 2026-11-27" --date 2026-11-27
# Her previous locked/working-set frame is marked superseded by the
# new one; the change is a registry edit — commit it.

# The step-2 (festival) asset manifest: per girl, locked frame if
# any, else the working-set frame, with sha256:
tools/ardor-registry/ardor-registry manifest
tools/ardor-registry/ardor-registry manifest --format text
```

`check` also matches by content: a byte-identical copy of a
registered frame at a different path resolves to the registered
entry, so lanes can verify staged copies.

## MCP form — the portrait-lock registry as ardor tools (M-1)

As of 2026-11-27 the registry mechanism also lives in the ardor
MCP server, generalized per game (mapping:
`moondeep-lessons-to-mcp-20261127/MAPPING.md` §M-1). Five tools,
named in the live surface's attested shapes — writes verb-first,
reads exclusively `get_<singular>` / `list_<plural>` (no `*_show`
verb exists anywhere on the 61-tool surface; census in LANE-A of
the mapping folder):

| tool | act |
|---|---|
| `register_portrait` | register a frame: the server recomputes the sha256 pin from the frame's bytes on disk (never caller-supplied); status `candidate` / `working-set` / `defective`; optional `supersedes`, `ruling`, `note` |
| `lock_portrait` | the Owner's lock: **requires** a ruling `{date, quote}` — the gate refuses a lock without one (`ruling_required`); supersedes the character's standing locked/working-set frames |
| `get_portrait_lock` | one character's current frame + full entry list, or — given a `path` — the CLI `check` verdicts (`current` / `superseded` / `defective` / `unknown`), resolving byte-identical copies by content hash |
| `list_portrait_locks` | the manifest: per character the current frame (locked first, else working-set), status counts, set vocabulary |
| `list_portrait_history` | one character's entries in registration order |

Design points:

- **Per-game by project, schema neutral.** The store document is
  `portrait-registry/1`; the game is the ardor project slug, and
  the cast list, set vocabulary, and frame root paths are whatever
  the project's registrations say — no per-game schema fork. The
  source schema name (`moondeep-portrait-registry/1`) is kept
  above as provenance for the CLI file.
- **Built on the keeper-chain surface, not beside it.** Every
  registered portrait is born as an asset record through the same
  gate checks as `register_asset` (kind `sprite`, lane-stamped),
  and the portraits document references it by id; keeper chains
  remain the provenance half, joined by asset id. Lock state is
  the distinct mechanism the chains do not carry.
- **Pins are re-verified on every read.** Each entry view carries
  `sha_state` — `verified` / `mismatch` / `missing` / `unreadable`
  — recomputed live from the frame's bytes, so a tampered pin or
  a tampered frame is caught by any read.
- **Supersession normalization.** Superseding a frame rewrites its
  status to `superseded` *unless it is `defective`* — a defect is
  the stronger fact and is never laundered into a mere
  replacement; the `superseded_by` link still records the
  replacement. This reproduces the source registry exactly,
  including Tamsin's defective canvas-fresh frame and its
  heightfix cure. Chains cannot fork (a frame is replaced exactly
  once) or cycle (self-supersession refused; edges point only
  backwards at registration).
- **Lock state is still versioned state.** The MCP store lives in
  the server home; `moondeep/` here is the migrated dataset
  committed as the record, as `registry.json` is for the CLI.

Migration: `registry.json`'s 139 entries were registered through
the tools in topological order (a frame only after the frames it
supersedes; the three recorded-`superseded` entries registered as
candidates and flipped by their successors' registrations, as the
mechanism requires). Verification through the tools:
**139/139 sha pins verified**, statuses, supersession links, and
rulings identical to the source; a planted wrong-sha control copy
was flagged as exactly one `mismatch`. Driver and full record:
night lane B, 2026-11-27 (`night-20261127/migrate_portraits.py`,
`night-20261127/lane-B-registry.md` in the estate workspace).

## Why

Tonight's failure mode, once, is enough: the judged-picks canvas
carry was struck on Solenne/Vesper bases whose edge cures landed
~20 minutes later, so the carries do not inherit the cures
(edge-cure RECORD, "Canvas-carry inheritance note"). A `check` at
strike time flags a superseded base before the seat time is spent.
The registry is also step 2's asset manifest source: the festival
build consumes `manifest`, diffed — not remembered.
