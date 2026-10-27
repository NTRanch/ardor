# Ardor — NTRanch NSFW game/app maker MCP

Commissioned by the Owner, 2026-10-27 (decision D-124). Name ruled by the Owner, 2026-10-27 (decision D-125): **Ardor** — "these games and apps are works of passion for passionate people." Register: IDEA-0096.

## Queue position

Front of the MCP from-birth build queue. The WS1 eigenstyle pilot completes first because it proves the fleet's rmcp pattern; Ardor is the next build slot, ahead of the torori and tackle starts, the WS2 batch migrations, and slipway's update treatment.

## Purpose

One NTRanch-lane maker surface for building the planned NSFW games and apps end to end: Strobe, Torogao, MoonDeep, Hands-On, and the hypnosis-remake family. It is an authoring and project-state server, not a runtime/orchestration replacement for phlow and not a session host replacement for torogao-session-host.

## Why a server

It holds authored project state across sessions and enforces canon and safety decisions through a decide()-class gate before canonical writes. That state plus enforcement meets the estate's server graduation rule; thin one-shot capabilities remain skills.

## Technology contract

- Rust server on rmcp, MCP modern era only (2026-07-28), stateless/discover-first.
- Kotlin MCP SDK companion for Android-side authoring, preview, and playtest bridges, following the D-118 pattern.
- Android-facing pieces are feature-gated and compiled out of release/store artifacts unless explicitly enabled.
- Pure Rust core where it owns game data and project state; edition 2024 and the estate nightly pin apply to Rust game code.

## Tool groups

1. **Project scaffolding** — create and inspect NTRanch game/app projects with the estate conventions, engine shell, metadata, and test layout already in place.
2. **Cast and canon** — character identity records, appearance/rig references, canon checks, and versioned changes to cast data.
3. **Scenes and systems authoring** — dialogue/scene graphs, schedules, flags, progression DAGs, economies, state machines, and minigame definitions. This group absorbs gap W-1 (game-systems authoring verbs) from IDEA-0095.
4. **Kink-legitimate dialogue and narrative creation** — author adult dialogue as structured game data, not euphemism-only prose: sex actions, character reactions, consent and safety state, participant roles, prerequisites, intensity, continuity effects, branching choices, and cues for animation, voice, sound, and haptics. Reaction profiles are per character and can depend on relationship, arousal/sensitivity/fatigue/conditioning state, prior choices, and scene history.
5. **Story, plot, and level making** — create and validate story arcs, beats, quests/objectives, character arcs, relationship progression, branching endings, scenes, encounters, and levels. Level tools cover layout/scene composition, triggers, schedules, progression gates, minigame placement, and deterministic playtest generation from the authored graph.
6. **Content and safety systems** — first-class scaffolding and checks for stop/safeword, accessibility, aftercare, and Purge surfaces; these are never progression-gated. Includes content tags, rating metadata, and age-gate patterns.
7. **Asset routing and provenance** — intake, hash, provenance, and routing records. Specialist work stays with its owner: lumen for sprite-to-3D, torori for audio, ache for haptics, neemu for screened corpus work.
8. **Playtest and QA** — scripted playthrough definitions, deterministic replay/hash checks, screenshot specifications, and defect records tied to authored project state.
9. **Build and release handoff** — prepare channel artifacts and hand them to slipway for Play/F-Droid certification. Ardor never forks slipway's publish gates or definitions. Per D-126 (Owner, 2026-10-27), this group also carries a **channel-target registry for every English-language adult game/app distribution channel**: itch.io (pipeline via butler; desktop/web/Android APK), Nutaku (portal package; web + Android APK), DLsite English (portal package), FAKKU, JAST USA, Denpasoft, MangaGamer, Kagura Games (curated storefront packages), Johren (platform package), GOG (selective; package), Zoom Platform (package), EroLabs (reinstated by D-126 over D-095's supersession), Newgrounds and Gamcore (web-game portals), Patreon + Subscribestar (membership distribution, a distinct category from storefronts). The pixiv family (pixiv proper, BOOTH, FANBOX, pixiv Requests) was added by D-126 addendum 3 and then STRUCK by D-129 (Owner, 2026-10-27): do not build or keep target or credential records for it — the strike is recorded in the registry like MiKandi's. Each target records: platforms, content-policy notes, automation class (pipeline / prepared-submission / curated-pitch), artifact requirements, and its handoff executor (butler for itch; slipway for the Android channels — Nutaku APK, itch APK, self-hosted F-Droid). MiKandi was in the original D-126 set and was STRUCK the same day (D-126 addendum 2): it is defunct (Wikipedia, verified 2026-10-27) — do not build a target or credential record for it. Every channel must be verified live at build time (still operating, English-language, accepts adult games/apps, submission route, accepted artifact formats); a channel that fails verification is flagged with evidence, never given invented requirements. Steam stays out (general storefront, ruled out in D-095); Google Play remains SFW-lane only (adult content prohibited platform-wide).

10. **Channel credentials and sessions** (D-127) — auth, API key, token, and login management for every channel in the group-9 registry. Hard rule: Ardor manages credential **references and status, never values** — no tool stores a raw secret in project state, returns one in its output, or writes one to a log or report. A channel-account record holds the channel, the account identity, the credential kind (API key / OAuth token / portal login / butler key), where the value lives (a primo `pass` path or a Secure Vault entry — the estate's two existing stores), scopes, expiry, and last-verified time. Tools register and inspect channel accounts, report credential status, run a per-channel verification probe where the channel supports one (e.g. validating the itch.io key against its API), track expiry and rotation, and run a publish **preflight** that refuses a handoff when the target channel's auth is missing or unverified. Channels with no API get honest login-reference records, never invented token flows. Values are acquired only through the existing secure flows (pass on primo, Secure Vault in chat); Ardor requests that a reference be created and never receives the value itself.

## Boundaries

- phlow remains the agent runtime and workflow orchestrator.
- tackle remains the editor broker.
- torogao-session-host remains the live session host.
- slipway remains the Android/publish-certification surface.
- Specialist MCPs keep their domains; Ardor calls or records handoffs rather than re-implementing them.
- NTRanch lane only. Registry publication, when its standardized build is conformance-green, follows D-123 under an NTRanch namespace and the marker gate.

## Name

Ruled by the Owner (D-125): **Ardor** — "these games and apps are works of passion for passionate people." The crates.io name returned 404 (unclaimed) on 2026-10-27; the NTRanch GitHub namespace is checked at repo creation.

Earlier candidates, retained as history: **boudoir** (initial working title) and **afterdark** were also unclaimed at the same check. **Liaison**, **tryst**, and **velvet** were already claimed on crates.io.

## Group 9 annex — channel sweep additions (gated by Pax 2026-10-27, D-128)

Source: `~/workspace/ardor-channel-sweep-20261027/SWEEP-REPORT.md` (verified live 2026-10-27; evidence grades per entry in the report). Add target records for each; credential records per group 10 where an account exists. Unverified legs are recorded as UNVERIFIED, never filled in.

**Games/apps additions:** IndieGala (curated-pitch; the DRM-free segment is the usable part — much of its adult catalogue is Steam-key resale and Steam is tabled), Shady Corner Games (curated-pitch; publisher with own storefront), TinyHat Studios (curated-pitch; publishing label, no own storefront), AltStore PAL (prepared-submission; EU-only iOS alternative marketplace, bring-your-own payments, adult apps proven possible via notarization being a malware scan only), SideQuest (prepared-submission; Quest VR sideload marketplace; official content-policy page NOT fetched — record carries that caveat), OnlyFans (prepared-submission; membership + custom orders — added under D-126's every-English-adult-site instruction; flagged to the Owner as the sweep's registry-gap judgment call), and a distinct OWN-STOREFRONT RAIL category: direct sale via adult-friendly processors CCBill / Segpay / Epoch / Verotel (pipeline class; published adult rates ~13–15.5% secondary-grade; CCBill card-network registration ~$2,000/yr secondary-grade).

**Rigs/assets additions — ADULT:** Renderotica/CGBytes (prepared-submission; purpose-built adult 3D-asset marketplace), Renderosity (prepared-submission; official Adult maturity tier; vendor review gate), CGTrader (prepared-submission; gated NSFW catalogue live), FlippedNormals (prepared-submission), Superhive (formerly Blender Market; prepared-submission; labelled NSFW rigs openly sold BUT its Creator Policy bars "graphic, obscene, or vulgar" content — record the tension; listings stay on the evidenced side), itch.io asset packs (pipeline via butler; extension of the existing itch entry; AI disclosure enforced for asset packs since Nov 2024), Affect3D Store (curated-pitch; finished 3D animation/image sets more than raw rigs), DLsite materials (extension of the existing DLsite entry, not a new target).

**Rigs/assets additions — SFW-ONLY (separate registry section, never presented as adult channels):** Fab (Epic; "mature" gated ≠ explicit; mandatory Created-with-AI declaration), Unity Asset Store (official guidelines bar genitalia; narrow medical/educational exception), RenderHub (adult content being removed platform-wide), nizima (Live2D-native; SFW-only in sampled terms; US-seller payout unverified/likely blocked), Etsy (post-2024 narrow remnant only).

**Services/contracting additions (new registry section — Ardor tracks service channels alongside sales channels):** VGen (prepared-submission; adult allowed bounded — "Adult content is allowed. Porn is not," drawn/animated/audio work in scope under the tag regime; 5% platform fee secondary-grade; Live2D rigging a staple category), Commishes/YCH.Commishes (prepared-submission; explicit NSFW auction/slot trade; PayPal-direct, no escrow; fee unverified), Voice Acting Club (curated-pitch/community; official #casting-calls-18-up — monetary compensation REQUIRED for explicit projects; the hiring route for adult VA), Casting Call Club (prepared-submission; NSFW roles open in practice; platform policy page not fetched — practice-verified only), IWantClips (prepared-submission; formal custom-order system; 75% creator secondary-grade), ManyVids (prepared-submission; automated custom requests; 80% creator on customs secondary-grade), Fansly (prepared-submission; customs via DM/PPV; 80% creator secondary-grade), OnlyFans (cross-listed from games additions). SFW-only services noted, not adult recommendations: SKIMA (uniform "No R18" listing terms), nizima make-to-order.

**Confirmed exclusions (do NOT create records; the report's §3/§6/§8e carry the evidence):** Gumroad, Game Jolt, Ko-fi (+ Ko-fi Commissions), Fiverr, Upwork, Aptoide developer channel (all POLICY-PROHIBITS with official texts fetched); Kimochi and Artists&Clients (DEFUNCT — the latter also breach-reported); Erogames and Winged Cloud (not sales channels); J-List and Sekai Project store (overlaps of JAST USA / Denpasoft, already registered); Critical Bliss (no storefront; Steam route tabled); Ci-en (fails the English criterion on available evidence); Toyhouse (no payment rail); Sketchfab Store (defunct, folded into Fab); ArtStation Marketplace for explicit assets (ToS §24 conflicts with practice); TurboSquid for adult (UNVERIFIED); FurAffinity/Inkbunny (informal only). UNVERIFIED items carried open: BOOTH US-seller payout route (blocks confident reliance on BOOTH records), Skeb official terms (site returned 429; practice suggests tolerance, not a policy), QooApp developer sales route, SideQuest official policy.
