# tools/moondeep-registry — MoonDeep portrait-lock registry

The durable form of the MoonDeep portrait lock, built per
`TOOLING-PLAN.md` T2 (BUILD-NOW, commissioned 2026-11-27). One JSON
registry + a stdlib-only Python CLI. The registry is the single home
for facts that previously lived as hand-typed canon-pin tables in
every art lane's record: which frame is current for each girl, what
it superseded, and the Owner ruling that put it there.

## Files

- `moondeep-registry` — the CLI (executable, Python 3 stdlib only).
- `registry.json` — the registry of record. **Lock state is versioned
  state**: it lives here, in the repo, and changes to it are commits.
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
(override with `--registry PATH` or `$MOONDEEP_REGISTRY`).

```sh
# Pre-flight before striking on a base: is it still current?
tools/moondeep-registry/moondeep-registry check /path/to/base.png
#   CURRENT <path> (status: ...)            exit 0 — strike away
#   SUPERSEDED-BY <newer path>              exit 1 — use the newer base
#   DEFECTIVE <path>                        exit 1 — gated out (reject/framing)
#   UNKNOWN <path>                          exit 2 — not a frame of record

# Record the Owner's lock for a girl (step 1's closing act):
tools/moondeep-registry/moondeep-registry lock solenne /path/to/final.png \
    --ruling "Owner lock, 2026-11-27" --date 2026-11-27
# Her previous locked/working-set frame is marked superseded by the
# new one; the change is a registry edit — commit it.

# The step-2 (festival) asset manifest: per girl, locked frame if
# any, else the working-set frame, with sha256:
tools/moondeep-registry/moondeep-registry manifest
tools/moondeep-registry/moondeep-registry manifest --format text
```

`check` also matches by content: a byte-identical copy of a
registered frame at a different path resolves to the registered
entry, so lanes can verify staged copies.

## Why

Tonight's failure mode, once, is enough: the judged-picks canvas
carry was struck on Solenne/Vesper bases whose edge cures landed
~20 minutes later, so the carries do not inherit the cures
(edge-cure RECORD, "Canvas-carry inheritance note"). A `check` at
strike time flags a superseded base before the seat time is spent.
The registry is also step 2's asset manifest source: the festival
build consumes `manifest`, diffed — not remembered.
