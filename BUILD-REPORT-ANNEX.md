# Ardor — Stage 1.1 build report (registry annex, D-128)

**Commission:** fold the gated channel sweep's verified additions into Ardor's
channel registry (BRIEF.md "Group 9 annex", D-128), following the Stage 1
patterns in BUILD-REPORT.md.
**Mid-build ruling applied:** **D-129 (Owner, 2026-10-27)** — the pixiv family
is STRUCK from the registry, reversing D-126 addendum 3 and cancelling brief
item 2 (the pixiv record refinement). Applied in full; see §3.
**Date:** 2026-10-27. **Tree:** `~/workspace/ntranch-maker-mcp-20261027/ardor/`
(sandbox) + primo mirror at `~/.local/share/ardor-mirror/`.
**Hard limits respected:** no commits, no pushes, no crates.io publish, no
registry upload, no GitHub repo creation. Sandbox tree + primo mirror only.
**Stage 2 (Kotlin companion) is still NOT built** — pending, per the original
commission.

The registry grows from 20 records to **47 channel records + 47 group-10 auth
records**, organised in six sections. All 46 non-flagged records are
`verified`; Zoom Platform remains the registry's only flagged channel.

## 1. What landed, per annex group

### Games/apps additions (section `distribution`)

| Channel | Class / executor | Record summary |
|---|---|---|
| IndieGala (`indie_gala`) | curated_pitch / manual | Adult catalogue incl. DRM-free adult downloads; the record names the DRM-free segment as the usable part and records that much of the catalogue is Steam-key resale (Steam tabled). |
| Shady Corner Games (`shady_corner`) | curated_pitch / manual | New category `publisher`: publishes third-party titles and sells direct from its own storefront. |
| TinyHat Studios (`tinyhat`) | curated_pitch / manual | Publishing label, no own storefront — route is a publishing deal; sales land on partner stores. |
| AltStore PAL (`altstore_pal`) | prepared_submission / manual | EU-only iOS alternative marketplace; notarization = malware scan only, adult apps proven possible; bring-your-own payments (record points at the rail section). |
| SideQuest (`sidequest`) | prepared_submission / manual | Quest VR sideload marketplace, open dev submissions incl. paid apps. **Caveat in the record itself:** SideQuest's own content-policy page was NOT fetched (sweep §4) — confirm policy before a first submission. |
| OnlyFans (`onlyfans`) | prepared_submission / manual | Membership category; added per the sweep's registry-gap observation under D-126's every-English-adult-site instruction. Economics (20% platform) marked **[secondary-grade]**. Cross-listed into `services` (see §4). |
| **Own-storefront rail** (`storefront_rail` section): CCBill, Segpay, Epoch, Verotel | pipeline / manual | New section + new category `payment_rail`. Records state plainly: not storefronts — they bill in their own name for the studio's own storefront. Adult rates ~13–15.5% and CCBill's ~$2,000/yr card-network registration are marked **[secondary-grade]**. |

### Rigs/assets additions — ADULT (section `assets`)

| Channel | Class | Record summary |
|---|---|---|
| Renderotica / CGBytes (`renderotica`) | prepared_submission | Purpose-built adult 3D-asset marketplace (since 2000, Canada). Payout terms recorded as UNVERIFIED (confirm at vendor onboarding). |
| Renderosity (`renderosity`) | prepared_submission | Official Adult maturity tier (Content Maturity Filter) [official-adjacent]; vendor review gate; gallery/promo caveat recorded; royalties tier-dependent [secondary]. |
| CGTrader (`cgtrader`) | prepared_submission | Gated NSFW catalogue live [practice]; record states no official policy page was fetched. |
| FlippedNormals (`flipped_normals`) | prepared_submission | NSFW models openly sold [practice]; strict minors policy [official] noted (adult work must be unambiguously adult); payout terms UNVERIFIED. |
| Superhive (`superhive`) | prepared_submission | Labelled NSFW rigs openly sold [practice] **against** a Creator Policy barring "graphic, obscene, or vulgar" content [official] — the record carries the literal `POLICY TENSION` marker and the rule that listings stay on the evidenced side (nudity/rigs yes, explicit material at-risk, sex acts NOT evidenced). |
| Affect3D Store (`affect3d`) | curated_pitch | Curated; finished 3D animation/image sets more than raw rigs; no open vendor signup. |

### Rigs/assets additions — SFW-ONLY (section `assets_sfw`, separate by construction)

Fab (`fab` — "mature" gated ≠ explicit; mandatory Created-with-AI declaration),
Unity Asset Store (`unity_asset_store` — official guidelines bar genitalia,
narrow medical/educational exception), RenderHub (`renderhub` — adult content
being removed platform-wide per its own announcement; "do NOT list adult
assets here"), nizima (`nizima` — Live2D-native; SFW-only in sampled terms;
US-seller payout UNVERIFIED/likely blocked, recorded not filled in), Etsy
(`etsy` — post-2024 narrow remnant only). Every SFW-only record declares
"SFW-only" in both its policy text and its verification note, and a unit test
enforces that the label is carried in the records, not just the section name.

### Services/contracting additions (section `services`; SFW-only in `services_sfw`)

| Channel | Class | Record summary |
|---|---|---|
| VGen (`vgen`) | prepared_submission | Official NSFW policy quoted ("Adult content is allowed. Porn is not.") with the tag regime and the processor bound; Live2D rigging a staple category; 5% platform fee + processing marked **[secondary-grade]**. |
| Commishes (`commishes`) | prepared_submission | Explicit NSFW auction/slot trade; real auction infrastructure, **no escrow**, PayPal-direct; platform fee UNVERIFIED. |
| Voice Acting Club (`voice_acting_club`) | curated_pitch | Official #casting-calls-18-up; monetary compensation REQUIRED for explicit projects — the hiring route for adult VA. Community infrastructure, no fee. |
| Casting Call Club (`casting_call_club`) | prepared_submission | NSFW paid roles open in practice; record states the platform policy page was NOT fetched — **practice-verified only**. |
| IWantClips (`iwantclips`) | prepared_submission | Formal custom-order system; 75% creator share **[secondary-grade]**. |
| ManyVids (`manyvids`) | prepared_submission | Automated custom requests, $9,999 cap; 80% creator on customs **[secondary-grade]**. |
| Fansly (`fansly`) | prepared_submission | Customs via DM/PPV; 80% creator **[secondary-grade]** (own pages not fetched — stated). |
| SKIMA (`skima`, `services_sfw`) | prepared_submission | SFW-only: uniform "No R18" listing terms; noted, not an adult recommendation. |
| nizima make-to-order (`nizima_requests`, `services_sfw`) | prepared_submission | SFW-only service record sharing the nizima account identity with the asset record. |

### Extensions of existing records (no new targets)

- **itch.io** now also covers **asset packs** (Live2D sets, rigged VTuber
  models, VRM/VRoid avatars, sprite/animation packs) through the same butler
  pipeline, with the rule that **AI disclosure is strictly enforced for asset
  packs since Nov 2024** recorded in both the policy text and the artifact
  requirements.
- **DLsite English** now also covers **materials/assets** sales (doujin
  categories span software/materials and asset-type works), recorded as an
  extension of the existing entry with its secondary grade stated.

### Credential records (group 10)

30 new auth records, one per new channel (47 total, matching the channel
count). Every annex channel authenticates by **`portal_login`** — an honest
account reference with an operator-attestation probe text; no token flow was
invented for any of them. Curated-pitch publishers got login-reference records
only where the sweep supports an account/contact identity (Shady Corner:
partner contact/store account; TinyHat: the record explicitly says the
relationship is a deal and references the partner contact established at
signing). Voice Acting Club's record states it is Discord community
infrastructure whose "account" is the studio's community identity. The two
nizima records share one account identity, mirroring the pattern Stage 1 used
for shared-account families.

## 2. Schema and gate changes (minimal, as briefed)

1. **`Channel` gains two fields:** `section` (one of the six `SECTIONS`) and
   `cross_listed_in` (other sections the channel is also listed under).
   `channel_json` emits both, so the wire record carries its section.
   OnlyFans is the registry's only cross-listed record (record lives in
   `distribution`, cross-listed into `services` for its custom-order flow).
2. **`list_channels`** gains a `sections` count map alongside the existing
   count/verified/flagged keys.
3. **Handoff gate (`gate.rs`) section discipline:** `prepare_handoff` now
   refuses services-section channels with the typed code
   **`not_a_release_channel`** (services take commissions, not releases), and
   refuses SFW-only asset channels for adult-rated projects with
   **`channel_sfw_only`**. Prepared handoff records now also record the
   channel's `section`.
4. **`publish_preflight` (accounts.rs)** gains a **`release_target`** check
   mirroring the gate, so an author sees the section verdict before
   attempting a handoff (e.g. VGen preflight: every check green except
   `release_target`).
5. Category vocabulary extended by two values: `publisher` (Shady Corner,
   TinyHat) and `payment_rail` (the four processors).

## 3. The D-129 strike (Owner ruling, applied mid-build)

The pixiv family records built in Stage 1 — channel records for **pixiv,
BOOTH, FANBOX** and their three group-10 auth records (the shared pixiv
account) — are **removed entirely**, exactly the way the MiKandi strike is
recorded: no channel record, no credential record; the strike is recorded in
prose (the `channels.rs` module header and this report) with the Owner's
reason: *"Struck by Owner ruling D-129, 2026-10-27; scope decision — the
sweep's pixiv findings stand as research, the family is not an NTRanch
channel."* Consequences, all verified below:

- Brief item 2 (pixiv record refinement — FANBOX payout verification, pixiv
  Requests facts, BOOTH flag update) is **cancelled, not performed**. The
  annex's BOOTH-assets line and pixiv Requests cross-listing produced no
  records. (The annex's "UNVERIFIED items" line still mentions BOOTH payout;
  it is moot — there is no BOOTH record for it to block.)
- Tests that asserted the pixiv records exist (the Stage 1
  `pixiv_family_shape_follows_the_ruling` unit test; the smoke test's booth
  cases) now assert the **struck state**: `find_channel`/`auth_for` return
  none for `pixiv`/`booth`/`fanbox`; a booth handoff and a booth account
  registration both refuse with `unknown_channel` (in the byte-identical
  battery); the Stage 1 registry counts moved 20/16/4 → 47/46/1.
- Live wire evidence: the D-116 annex session (§5) ends with a BOOTH handoff
  refused `unknown_channel` against the built binary.

## 4. Tests

Suite total: **29 → 36 tests** (unit 24 → 30, integration 5 → 6), all green.

- `channels.rs` unit tests 5 → 11: D-126 set as amended; all 30 annex ids
  present (total exactly 47); flagged set is exactly `[zoom_platform]`;
  **struck-state test** (MiKandi + pixiv family have no channel/auth
  records); section partition (counts 23/4/6/5/7/2, rail all `payment_rail`,
  exact SFW and services sets); SFW-only records declare "SFW-only" in their
  own text; OnlyFans cross-listing is unique and correct; extension notes
  present (itch asset packs, DLsite materials, and no `dlsite_materials`
  record); annex caveats carried in the records (SideQuest NOT-fetched,
  Superhive POLICY TENSION, CCC practice-verified-only, Commishes
  UNVERIFIED, [secondary-grade] markings); auth completeness incl. annex
  `portal_login` kinds; executor rules incl. the rail.
- New integration test `annex_registry_end_to_end`: section counts over the
  wire; struck family absent over the wire (`get_channel fanbox` →
  `unknown_channel`); VGen register → verify → preflight not-ok on
  `release_target` → handoff refused `not_a_release_channel`; Renderotica
  register → verify → preflight all-green → handoff **prepared** (executor
  manual, section `assets`); CCBill rail handoff **prepared** (section
  `storefront_rail`); credential rollup count 47.
- Existing batteries extended, still green: gate-refusal byte-identity now
  also covers pixiv (struck), VGen (services) and Fab (SFW-only) handoffs;
  the credential battery's flagged-registration case moved from booth to
  Zoom Platform, with booth re-asserted as `unknown_channel`.
- Balance of the new tests is ≈50/50 validation/adversarial per estate rule:
  happy paths (section shape, cross-listing, two prepared handoffs,
  preflight green) against refusals (struck/services/SFW handoffs,
  byte-identity, caveat-carriage assertions).

## 5. D-116 utilization evidence (annex session)

Appended to `invocation-transcript.txt` in the same directory as this report
(13 frames against the updated binary, home `/tmp/ardor-annex-home`):
discover → scaffold adult project → `get_channel vgen` → VGen account
reference registered → verification attested → `publish_preflight` vgen
**ok:false** (only `release_target` fails) → `prepare_handoff` vgen refused
**`not_a_release_channel`** → Renderotica account registered + verified →
preflight **all-green** → handoff **prepared** → Fab handoff refused
**`channel_sfw_only`** → BOOTH handoff refused **`unknown_channel`** (the
D-129 strike, live). Server exited 0 on EOF.

## 6. Gates — numbers

| Gate | Result |
|---|---|
| `cargo test` (estate pin nightly-2026-09-25) | **36 passed, 0 failed** — 30 unit + 6 integration (`tests/mcp_smoke.rs`) |
| `cargo clippy --all-targets -- -D warnings` | **clean** (exit 0) |
| `cargo fmt --check` | **clean** |
| MCP Inspector, `--protocol-era modern --method tools/list` | **exit 0** |
| MCP Inspector, modern `tools/list --strict` | **exit 0** (no schema errors) |
| MCP Inspector, modern `tools/call list_channels` | **exit 0** — count 47, verified 46, flagged 1; sections {distribution 23, storefront_rail 4, assets 6, assets_sfw 5, services 7, services_sfw 2}; pixiv family absent; OnlyFans cross-listing present |
| MCP Inspector, `--protocol-era legacy --method initialize` | **exit 1**, stderr `{"error":{"code":"error","message":"Unsupported protocol version"}}` — the typed refusal, as designed |
| Diver gate (primo mirror, headless nvim, live config) | **PASS — 11/11 Rust files, rust-analyzer attached, total diagnostics 0** (`~/.local/share/ardor-mirror/diver-gate-results.txt`; mirror refreshed from this tree, `src/channels.rs` md5-identical both sides) |
| D-123 marker sweep | Case-insensitive sweep for the SFW-lane namespace markers over the whole crate tree (excluding `target/`): **0 hits** |

## 7. Judgment calls and unverified items, stated plainly

1. **SideQuest, Superhive, Casting Call Club are `verified`, not `flagged`,**
   despite their unfetched-policy / policy-tension / practice-only evidence.
   Rationale: the annex lists them as additions (Gamcore set the Stage 1
   precedent — verified with the caution carried in the record), and their
   caveats live **inside** the records, where a unit test asserts their
   presence. Flagging them would refuse all handoffs, which the sweep's
   recommendations do not support. If Pax prefers the stricter reading, the
   change is one field per record plus the flagged-set test.
2. **AltStore PAL's executor is `manual`, not `slipway`.** The brief names
   slipway for the Android channels; AltStore PAL is iOS, and the sweep's
   executor fit reads "slipway-class / manual". I recorded `manual` and the
   bring-your-own-payments pairing with the rail instead of silently
   extending slipway's remit to iOS.
3. **Services handoffs are refused by design** (`not_a_release_channel`)
   rather than prepared: `prepare_handoff` is a *release* handoff, and the
   annex frames services as a section Ardor "tracks alongside" sales
   channels. The preflight `release_target` check keeps the verdict visible
   instead of surprising. This is a modelling decision Pax can reverse with
   one gate branch.
4. **nizima is two records** (`nizima` assets_sfw + `nizima_requests`
   services_sfw) sharing one account identity, because the annex lists the
   marketplace under SFW assets and the make-to-order route under SFW
   services.
5. **Unverified legs remain unverified, in the records:** BOOTH payout is
   moot post-strike; SideQuest's official policy, nizima's US-seller
   payout, Renderotica's and FlippedNormals' payout terms, and Commishes'
   platform fee are recorded as UNVERIFIED/not-fetched — never filled in.
   Secondary-grade fee figures (rail rates, OnlyFans/Fansly/IWantClips/
   ManyVids splits, VGen fees) are marked [secondary-grade] in the records
   and asserted by test.
6. **BUILD-REPORT.md (Stage 1) is now historical** — its registry table
   shows the pre-annex 20 records and the pixiv family as flagged. It was
   not edited; this report supersedes that table (Stage 1's own struck-row
   convention records history in reports rather than rewriting them).
7. No new source files were added, so the Diver gate's 11-file list is
   unchanged; the gate script on primo was reused as-is.
