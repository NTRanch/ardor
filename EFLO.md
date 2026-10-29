# EFLO record — ardor tools/list payload size

EFLO = the measured token size of the server's live `tools/list`
payload under a real tokenizer. This file is the in-tree record
for roadmap item R-202 / program gap G-R1.

## The measure (1.0.0 surface, 46 tools)

Measured live against the built server (modern-era
`server/discover` → `tools/list` over stdio) on primo,
2026-10-28, with `tiktoken` 0.14.0, `cl100k_base` encoding,
canonical JSON (sorted keys, compact separators) — the neemu
EFLO pattern.

| Measure | Value |
|---|---|
| Tools | 46 |
| Payload | 16,979 bytes |
| **cl100k_base (whole tools array)** | **3,549** |
| cl100k_base (sum of per-tool payloads) | 3,592 |
| Fleet budget (≤) | 2,500 |
| **Overage** | **1,049** |

Per-tool cl100k_base (whole-payload tokenization of each tool
object):

| Tool | Tokens |
|---|---|
| `create_project` | 133 |
| `get_project` | 53 |
| `list_projects` | 38 |
| `update_project_meta` | 138 |
| `upsert_character` | 92 |
| `get_character` | 62 |
| `list_characters` | 45 |
| `add_canon_rule` | 78 |
| `list_canon` | 45 |
| `check_canon` | 86 |
| `declare_flag` | 69 |
| `upsert_scene` | 81 |
| `upsert_dialogue` | 100 |
| `get_node` | 56 |
| `list_nodes` | 63 |
| `upsert_system` | 77 |
| `list_systems` | 42 |
| `set_reaction_profile` | 98 |
| `preview_reaction` | 101 |
| `upsert_arc` | 65 |
| `validate_story` | 57 |
| `upsert_level` | 90 |
| `list_levels` | 40 |
| `generate_playtest` | 89 |
| `get_safety` | 61 |
| `set_safety_surface` | 124 |
| `audit_safety` | 63 |
| `register_asset` | 103 |
| `list_assets` | 45 |
| `record_asset_handoff` | 99 |
| `run_playtest` | 90 |
| `verify_replay` | 74 |
| `add_screenshot_spec` | 71 |
| `record_defect` | 71 |
| `list_defects` | 67 |
| `list_channels` | 60 |
| `get_channel` | 50 |
| `prepare_handoff` | 123 |
| `list_handoffs` | 47 |
| `register_channel_account` | 100 |
| `list_channel_accounts` | 69 |
| `get_channel_account` | 63 |
| `record_auth_verification` | 137 |
| `credential_status` | 84 |
| `probe_channel_auth` | 90 |
| `publish_preflight` | 103 |

## Disposition — WAIVER RECOMMENDED, surface not shrunk

The 46-tool surface is **1,049 tokens over** the fleet's ≤2,500
budget. The surface was **not** shrunk to fit, and no tool
description was trimmed to game the number. Instead, this record
recommends a **D-147-class waiver for ardor at 3,549 tokens**,
for the Owner's ruling, on these grounds:

- The 46 tools in ten groups are the commissioned IDEA-0096
  maker scope, landed whole at 0.1.0; the overage is the scope
  itself, not description bloat (the largest single tool is
  138 tokens; no tool is an outlier).
- Ardor is an authoring-session server: a maker session opens
  it deliberately, works, and closes it. It is not an always-on
  surface riding in every agent context — the condition the
  EFLO budget exists to protect against.
- The known cross-lane consumer (Seahawk's REQ-9 route) calls
  ardor as a *client* under its own projection; ardor's
  tools/list cost does not land in that consumer's own
  tools/list.

**Revisit path (recorded, not built):** if an always-on
consumer appears, or the surface grows past 3,549, the question
returns to the Owner under the D-147 revisit logic, and the
remedy on the table is trigger-scoped group loading (torori's
pattern: the ten groups load on trigger, the baseline stays
lean). Until an Owner ruling lands, this file is the honest
record: **3,549, over budget, waiver recommended — not waived.**
