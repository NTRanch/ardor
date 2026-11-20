# MODEL ASSIGNMENT — local models for the D-1563 proving ground

> BLOCH: origin_lane=ntranch, content_label=nsfw.
> Compiled 2026-11-20 by Owner commission ("do the census for all
> the nsfw games to assign models to them; if making copies of
> models to use would help, do so"). Companion to
> `docs/REMAKE-TRACKER.md`. The inventory below is a live probe of
> the rose store (primo, 2026-11-20, `rose list` + `/api/tags` +
> `rose show`), not a record copied from elsewhere. Assignments
> name only models that exist in that probe.

## 1. Inventory of record (rose store, primo, 2026-11-20)

Serving: `rose-serve.service` (loopback :11434), A6000-first per
estate policy. GPU reality: the A6000 (46 GiB) holds one 30B-class
model plus one mid model, or several small ones; lanes serialize
on the GPU by policy regardless — assignments are about fit, not
concurrency.

**Coder seat class**
- `hf-qwen3-coder-30b` — 30.5B MoE, Q4_K_M, context 262144 (per
  `rose show` on its derived variant). The heavy build coder.
- `hf-devstral-24b-tools-64k` — 23.6B, Q4_K_M, tools-tuned, 64k.
  The harness-grade coder: build work that is mostly tool calls.
- `opal` — devstral variant already in the store.
- `qwen2.5-coder:7b`, `hf-qwen2.5-coder-3b`, `-1.5b`, `-0.5b` —
  small coders for bounded edits and clerk work.

**General / evidence seat class**
- `clara` — qwen3moe 30.5B variant. Long-document evidence
  reading and spec synthesis.
- `hf-qwen36-27b`, `sylvia`, `heretic-bench` — qwen35 26.9B line.
- `hf-gpt-oss-20b-tools-64k` — 20.9B MoE, tools-tuned, 64k. The
  tool-driven census worker (fathom probes, file censuses).
- `freya` — nemotron_h_moe 31.6B variant.
- `hf-gemma-4-12b` — 11.9B. The translation seat (strongest
  multilingual model in the store; see §5 gaps).
- `mira` — glimmer-30b **vision** variant. Asset/image census.
- `hf-glimmer-30b`, `hf-glimmer-30b-vision` — base line of mira.

**Harness / tools seat class (the "-tools-64k" line)**
- `hf-granite-4.2-8b-tools-64k` — 8.8B. Small harness clerk:
  bounded, well-tooled sub-tasks, audits, inventories.
- `hf-devstral-24b-tools-64k` (also coder class) — the
  harness-engineer candidate seat (see §4).
- `hf-gpt-oss-20b-tools-64k` (also evidence class).

**Restricted**
- `hf-phi-4-tools` / `tessa` — **never the engineer seat, and
  never unsupervised tool work**: on record, this model's
  semantically correct tool calls are emitted as fenced JSON
  text that rose's parser does not promote to `tool_calls`.
  Drafting-only, supervised, until the parser-tolerance fix
  lands and is re-probed.

**Clerk class (triage, routing, small checks)**
- `hf-granite-4.2-3b`, `hf-dory-4b`, `hf-gemma-3-1b-it`,
  `hf-llama-3.2-1b`, `hf-granite-4.0-1b`, `hf-lfm2.5-1.2b`,
  `hf-qwen3.5-0.8b`, `hf-qwen3-0.6b`.

## 2. Assignment rules

A title's seat follows its **next rung's work shape**, and is
re-ruled when the rung changes:

- **Build rungs** (Rust implementation, R4–R9 and equivalents):
  coder class — `hf-qwen3-coder-30b` primary, devstral-tools
  support; titles with an imminent lane get a derived variant
  (§4) as primary instead.
- **Consolidation / parity / audit rungs** (tool-driven gates):
  `hf-devstral-24b-tools-64k` primary, qwen3-coder support.
- **Extraction / census / spec rungs** (long-evidence reading):
  `clara` primary, `hf-gpt-oss-20b-tools-64k` support for the
  tool-driven census mechanics.
- **Translation rungs** (R2): `hf-gemma-4-12b` joins as support.
- **Asset-heavy censuses**: `mira` joins as support (evidence
  assembly only — verdicts stay with Pax/Owner).
- **Systems-only titles**: clerk class only; no content work
  exists to assign.

## 3. Per-title assignment

Primary / support, by the tracker's state of record. "Variant"
means the derived copy in §4 is the primary.

| Title | Wave | Next-rung shape | Primary | Support |
|---|---|---|---|---|
| Magic Coin | W1 | Consolidation + parity walk (PILOT) | pg-magic-coin-build (variant) | hf-devstral-24b-tools-64k |
| Modification App | W1 | Consolidation → R1 re-verification | hf-devstral-24b-tools-64k | hf-qwen3-coder-30b |
| Milk Factory | W1 | Consolidation → R9 gates | hf-devstral-24b-tools-64k | hf-qwen3-coder-30b |
| Time Stop | W1 | Consolidation → R9 | hf-devstral-24b-tools-64k | hf-qwen3-coder-30b |
| Where She Asks to Fall | W1 | S5 consolidation + gates | hf-devstral-24b-tools-64k | hf-qwen3-coder-30b |
| Crescendo | W2 | First from-scratch build | pg-crescendo-build (variant) | hf-devstral-24b-tools-64k |
| Hypnosis Card | W2 | R1 values consumption → build | pg-hypnosis-card-build (variant) | hf-devstral-24b-tools-64k |
| handfast | W2 | Platform completion audit | hf-devstral-24b-tools-64k | hf-granite-4.2-8b-tools-64k |
| withheld | W2 | Audit then complete | hf-devstral-24b-tools-64k | hf-granite-4.2-8b-tools-64k |
| blushline | W3 | Jam-family rebuild (reference shape) | pg-jam-family-build (variant) | hf-devstral-24b-tools-64k |
| by-heart | W3 | Jam-family rebuild | pg-jam-family-build (variant) | hf-devstral-24b-tools-64k |
| creamery | W3 | Jam-family rebuild | pg-jam-family-build (variant) | hf-devstral-24b-tools-64k |
| exposure | W3 | Jam-family rebuild | pg-jam-family-build (variant) | hf-devstral-24b-tools-64k |
| idle-hands | W3 | Jam-family rebuild | pg-jam-family-build (variant) | hf-devstral-24b-tools-64k |
| milkmaid | W3 | Jam-family rebuild | pg-jam-family-build (variant) | hf-devstral-24b-tools-64k |
| standard-issue | W3 | Jam-family rebuild | pg-jam-family-build (variant) | hf-devstral-24b-tools-64k |
| trancefall | W3 | Jam-family rebuild | pg-jam-family-build (variant) | hf-devstral-24b-tools-64k |
| quartermaster | W3 | Jam-family rebuild (LÖVE/Lua source) | pg-jam-family-build (variant) | hf-devstral-24b-tools-64k |
| redlight | W3 | Jam-family rebuild | pg-jam-family-build (variant) | hf-devstral-24b-tools-64k |
| second-wave | W3 | Jam-family rebuild | pg-jam-family-build (variant) | hf-devstral-24b-tools-64k |
| desperado | W4 | Rust rebuild (largest Godot title) | hf-qwen3-coder-30b | clara |
| roseford | after W2 | Original build (design-stage) | clara | hf-gemma-4-12b |
| Aria Succubism | W4 | Audio-sync spike → build | hf-devstral-24b-tools-64k | hf-granite-4.2-8b-tools-64k |
| CIRCLEMATE | W4 | S2c expressions note (study) | clara | hf-gpt-oss-20b-tools-64k |
| Hypno_Rhythm | W4 | Audio-sync spike → build | hf-devstral-24b-tools-64k | hf-granite-4.2-8b-tools-64k |
| HypnoClicker | W4 | R1 verification → build | hf-devstral-24b-tools-64k | hf-qwen3-coder-30b |
| Maeve's Academy | W4 | Lane D stages from vocabulary | clara | hf-gpt-oss-20b-tools-64k |
| Unholy Maiden | unassigned | D-1587 source-evidence rung | pg-unholy-maiden-extract (variant) | hf-gpt-oss-20b-tools-64k |
| Teaching Feeling | — | **Systems-only permanently** | hf-granite-4.2-3b (clerk, systems docs only) | — |
| Hypnotic Eyes pair | W4 | Extraction rungs | clara | hf-gpt-oss-20b-tools-64k |
| IHSA | W4 | Extraction (rung position unresolved, §8 C2) | clara | hf-gpt-oss-20b-tools-64k |
| Undercity Doll Shop | W4 | S-rung re-walk → extraction | clara | hf-gpt-oss-20b-tools-64k |
| Complicated Natsuki | W4 | S-rung re-walk → extraction | clara | hf-gpt-oss-20b-tools-64k |
| Ren'Py group remainder | W4 | Dialogue-tree extraction | clara | hf-gpt-oss-20b-tools-64k |
| RPG Maker group remainder | W4 | MV/MZ extraction | clara | hf-gpt-oss-20b-tools-64k |
| Moo-Moo Manor | W4 | Spec drafting from measured coverage | clara | hf-gpt-oss-20b-tools-64k |
| Monster Girl Assault | W4 | Extraction (GameMaker route unexercised) | clara | hf-gpt-oss-20b-tools-64k |
| HYPERDEEP | W4 | Extraction (Unreal pak route unexercised) | clara | hf-gpt-oss-20b-tools-64k |
| Detailed Cowgirl Simulator | unassigned | D-1587/D-1588 rung → spec | clara | hf-gpt-oss-20b-tools-64k |
| Puppet Master | W4 | Extraction rungs | clara | hf-gpt-oss-20b-tools-64k |
| Hypnosis App | W4 | Extraction rungs | clara | hf-gpt-oss-20b-tools-64k |
| SoldGirl Town | W4 | MonoBehaviour census → spec | clara | hf-gpt-oss-20b-tools-64k |
| Mari's Story | W4 | Intake census | clara | hf-gpt-oss-20b-tools-64k |
| NTR Idol | W4 | Pak census | clara | hf-gpt-oss-20b-tools-64k |
| Lewd Avatar Simulator | unassigned | Intake stages 3–5; **systems-only until cast verification** | clara | mira (asset census) |
| hypno_booth | unassigned | D-1587 GDScript decompile labour | hf-devstral-24b-tools-64k | clara |
| White Knight Discipline | unassigned | Full intake | clara | hf-gpt-oss-20b-tools-64k |
| Vesper Isles | unassigned | Freeze gate → R2 → build | hf-devstral-24b-tools-64k | hf-gemma-4-12b (R2) |
| SiNiSistar 2 | unassigned | D-1587 study evidence → spec | clara | mira (asset census) |
| Hachina | unassigned | R2 manifest → R3 → data-layer build | hf-devstral-24b-tools-64k | hf-gemma-4-12b (R2) |
| StopKing | unassigned | R2 → R3 → build | hf-devstral-24b-tools-64k | hf-gemma-4-12b (R2) |
| Suspended Sex Simulator | unassigned | D-1587 + serialized-values → freeze | clara | hf-gpt-oss-20b-tools-64k |
| Elise and the Spellbinding Marionette | unassigned | **Cast verification first**; systems-only until it passes | clara (census support only; no build assignment) | mira (evidence assembly) |
| Clergy n Cocktail | unassigned | D-1587 + serialized-values → freeze | clara | hf-gpt-oss-20b-tools-64k |
| The Censor | unassigned | R2 → R3 → build (spec frozen) | hf-devstral-24b-tools-64k | hf-gemma-4-12b (R2) |
| Iris the Blood Servant | unassigned | Intake commission | clara | hf-gpt-oss-20b-tools-64k |
| Agent Mirai | unassigned | Intake (asar census) | clara | hf-gpt-oss-20b-tools-64k |
| Goblin Burrow | unassigned | Intake (xp3 census) | clara | hf-gpt-oss-20b-tools-64k |
| Mindsweeper | unassigned | Planning products → audit | clara | hf-gpt-oss-20b-tools-64k |
| Hypnotic-Family Planning S | unassigned | **Cast verification (named unknown) first** | clara (census support only; no build assignment) | mira (evidence assembly) |
| Underground Idol X | unassigned | Planning (SWF — new fathom family) | clara | hf-gpt-oss-20b-tools-64k |
| Dohna Dohna | unassigned | Planning (System4 census) | clara | hf-gpt-oss-20b-tools-64k |
| UnHolY DisAsTeR | unassigned | Planning (Fusion CCN) | clara | hf-gpt-oss-20b-tools-64k |
| Popolife! | unassigned | Planning (Cocos/JSB index) | clara | hf-gpt-oss-20b-tools-64k |
| Kamimachi Site | unassigned | **SOURCE-BLOCKED** | — (no assignment until a source exists) | — |

Part II seats (tracker §6): the pilot's worker is
`pg-tackle-worker`; the graduation rung's engineer seat is
`pg-slipway-engineer`; all other Part II enhancement work runs
`hf-devstral-24b-tools-64k` as worker under the current engineer
seat. Phaedrus (research seat) is a tool, not a model — the
evidence-class models above serve its queries.

## 4. Copies made (derived variants, 2026-11-20)

Per the Owner's conditional ("if making copies of models to use
would help, do so"): copies help exactly where a lane is imminent
and the title's canon must be in the worker's context from the
first token — a derived variant bakes the title's spec pointer,
pipeline law, and lane discipline into the system prompt, so no
session burns context re-deriving them and no lane can quietly
run under the wrong rules. Copies are `rose create` derivations:
they share the base model's blobs (a new manifest + a small
system layer), so each costs kilobytes of marginal disk, verified
at creation. Distant-wave titles get their variant at lane
dispatch, from the same recipe — making 65 copies today would be
inventory theatre.

Created and verified in `rose list` (2026-11-20):

| Variant | Base | Serves |
|---|---|---|
| `pg-crescendo-build` | hf-qwen3-coder-30b | Crescendo build lane (W2 first build) |
| `pg-magic-coin-build` | hf-qwen3-coder-30b | Magic Coin consolidation (W1 pilot) |
| `pg-hypnosis-card-build` | hf-qwen3-coder-30b | Hypnosis Card build lane (W2) |
| `pg-jam-family-build` | hf-qwen3-coder-30b | All eleven W3 jam-family rebuilds |
| `pg-unholy-maiden-extract` | clara | Unholy Maiden D-1587 rung |
| `pg-tackle-worker` | hf-devstral-24b-tools-64k | Proving-ground pilot (tackle adoption) |
| `pg-slipway-engineer` | hf-devstral-24b-tools-64k | Graduation rung engineer seat (slipway) |

Modelfiles of record: `~/workspace/model-census-20261120/modelfiles/`
(hatch workspace) — the recipe is FROM + a low temperature +
a SYSTEM block carrying the title's canon and the standing rules
(Rust/Tiger Style, English-only, evidence discipline, theodolite
gates, friction log). One correction the creation run surfaced:
the first variant's `rose show` reports context length 262144
for the qwen3moe base — the build coder is not context-poor, and
§1 records it.

## 5. Gaps and honest limits

- **No dedicated translation model.** R2 seats ride
  `hf-gemma-4-12b` as the store's strongest multilingual model;
  if a translation rung gates poorly, the gap is named here in
  advance: a purpose-chosen JA→EN model is an acquisition
  decision for the Owner, not a silent substitution.
- **The phi-4 parser restriction** (§1) removes one mid-size
  tools model from engineer candidacy until the rose
  parser-tolerance fix lands; today's Qompass rollup surfaced
  NVIDIA's `frontend-crates` work as a direct reference for that
  fix.
- **Context ceilings**: the "-tools-64k" line is 64k by design;
  extraction seats (clara line) carry the long-document work.
  Any rung whose evidence exceeds its seat's context is a
  friction-log entry and a re-seating, not a truncation.
- **Copies are not concurrency.** Derived variants share the
  base weights and the one A6000; the proving ground's lanes
  still serialize under the heavy-compute policy. What copies
  buy is canon-in-context and per-title reproducibility (the
  variant name + digest pins exactly which worker a lane ran).
- **Assignment is per-rung, not per-title forever.** When a
  title's rung changes shape (extraction → build), its seat
  changes per §2 and this table is amended in the same commit
  series as the lane brief that fires the rung.
