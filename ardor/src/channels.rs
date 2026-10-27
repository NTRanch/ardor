//! The channel-target registry (D-126): every English-language
//! adult game/app distribution channel Ardor can prepare a release
//! handoff for. Each record carries platforms, content-policy
//! notes, an automation class (`pipeline` / `prepared_submission`
//! / `curated_pitch`), artifact requirements, its handoff executor
//! (`butler` for itch.io, `slipway` for the Android channels and
//! the self-hosted F-Droid route, `manual` for portal uploads and
//! publisher pitches), and a verification record.
//!
//! Verification was performed live during the build on
//! 2026-10-27 (sources in BUILD-REPORT.md). A channel that failed
//! verification is `flagged`, with the evidence in its note; the
//! gate refuses handoffs to flagged channels rather than letting
//! anyone prepare a package for a dead or unverified target.
//! Requirements recorded here are only what verification actually
//! established — where a route is curated and its package spec is
//! issued during onboarding, the record says so instead of
//! inventing one.
//!
//! Annex (D-128, 2026-10-27): the channel sweep
//! (`ardor-channel-sweep-20261027/SWEEP-REPORT.md`, gated by Pax)
//! added the channels the original D-126 set missed, organised
//! in registry sections: `distribution` (games/apps storefronts,
//! platforms, portals and membership distribution, incl. the
//! annex's games additions), `storefront_rail` (the
//! own-storefront payment processors — a rail, not storefronts),
//! `assets` (adult rig/asset marketplaces), `assets_sfw`
//! (SFW-only asset marketplaces — a deliberately separate
//! section, never presented as adult channels), `services`
//! (adult contracting / service-request channels, tracked
//! alongside sales channels), and `services_sfw` (SFW-only
//! services, noted, not adult recommendations). The gate
//! enforces the sections: release handoffs to services channels
//! are refused (services take commissions, not releases), and
//! adult projects can never hand off to SFW-only channels.
//! Evidence grades from the sweep ([official] / [secondary] /
//! [practice]) are carried in the records' notes; secondary-grade
//! figures are marked as such and unverified legs are recorded
//! as UNVERIFIED, never filled in.
//!
//! Boundaries: Ardor prepares packages and handoff records. Gate
//! definitions stay owned by the publish skills; the named
//! executor performs the release. Steam is out (D-095); Google
//! Play is SFW-lane only and not a target here.
//!
//! Struck pre-build (D-126 addendum 2, 2026-10-27): MiKandi was
//! removed from the target set entirely on Owner-supplied
//! verified evidence — it is defunct (Wikipedia: "a defunct
//! mobile adult software applications store"; last app release
//! 2018). It has no channel record and no credential record here;
//! the strike is recorded in BUILD-REPORT.md.
//!
//! Struck mid-build (D-129, 2026-10-27, Owner ruling): the
//! pixiv family — pixiv, BOOTH and FANBOX (pixiv Requests never
//! had a record) — was removed from the registry entirely,
//! reversing D-126 addendum 3. Scope decision: the sweep's pixiv
//! findings stand as research; the family is not an NTRanch
//! channel. Like MiKandi, the struck family has no channel
//! records and no credential records here; the strike is
//! recorded in BUILD-REPORT-ANNEX.md.

use serde_json::{Value, json};

/// One distribution channel.
pub struct Channel {
    pub id: &'static str,
    pub name: &'static str,
    /// storefront | platform | portal | membership | self_hosted
    /// | presence | publisher | payment_rail | services
    pub category: &'static str,
    /// distribution | storefront_rail | assets | assets_sfw
    /// | services | services_sfw (D-128 annex sections)
    pub section: &'static str,
    /// Other sections this channel is also listed under
    /// (cross-listings; the record itself lives in `section`).
    pub cross_listed_in: &'static [&'static str],
    pub platforms: &'static [&'static str],
    pub content_policy: &'static str,
    /// pipeline | prepared_submission | curated_pitch | prepared_presence
    pub automation_class: &'static str,
    pub artifact_requirements: &'static [&'static str],
    /// butler | slipway | manual
    pub handoff_executor: &'static str,
    pub submission_route: &'static str,
    /// verified | flagged
    pub verification_status: &'static str,
    /// What the 2026-10-27 live check established, or why flagged.
    pub verification_note: &'static str,
}

const CHECKED: &str = "2026-10-27";

/// The registry sections (D-128 annex), alphabetical. The two
/// `*_sfw` sections are deliberately separate: SFW-only channels
/// are never presented as, or confused with, adult channels, and
/// the gate enforces the separation for handoffs.
pub const SECTIONS: [&str; 6] = [
    "assets",
    "assets_sfw",
    "distribution",
    "services",
    "services_sfw",
    "storefront_rail",
];

/// The registry, alphabetical by id.
pub const CHANNELS: &[Channel] = &[
    Channel {
        id: "affect3d",
        name: "Affect3D Store",
        category: "storefront",
        section: "assets",
        cross_listed_in: &[],
        platforms: &["web"],
        content_policy: "Adult 3D store (operated by Netdifussion Ltd): sells its own product line plus curated third-party 3D animations, comics, image sets and games. Curated — no open vendor signup found; fits finished 3D animation/image-set products more than raw rigs.",
        automation_class: "curated_pitch",
        artifact_requirements: &[
            "finished 3D animation / image-set / comic products per the store's curated onboarding",
            "no public package spec — terms are set during the pitch",
        ],
        handoff_executor: "manual",
        submission_route: "Curated pitch to Affect3D; no open vendor signup found in the 2026-10-27 sweep.",
        verification_status: "verified",
        verification_note: "Operating (sweep 2026-10-27): operator/affiliate listing and store presence [secondary/practice].",
    },
    Channel {
        id: "altstore_pal",
        name: "AltStore PAL",
        category: "platform",
        section: "distribution",
        cross_listed_in: &[],
        platforms: &["ios"],
        content_policy: "EU-only iOS alternative marketplace (DMA): apps are distributed from developer-hosted sources and Apple notarization is a malware scan with no content review, so adult apps are proven possible (the first adult app shipped Feb 2025). Commerce is bring-your-own — AltStore PAL has no built-in storefront payments, so a release here pairs with a storefront_rail processor record.",
        automation_class: "prepared_submission",
        artifact_requirements: &[
            "ios app package prepared for notarization, distributed via a developer-hosted AltStore source",
            "payments are bring-your-own — pair with a storefront_rail processor record; AltStore itself sells nothing",
        ],
        handoff_executor: "manual",
        submission_route: "Open: the developer hosts their own source; the app passes Apple notarization (malware scan only).",
        verification_status: "verified",
        verification_note: "Operating (sweep 2026-10-27): EU alternative marketplace live; adult-app possibility evidenced by Feb 2025 press coverage [secondary].",
    },
    Channel {
        id: "casting_call_club",
        name: "Casting Call Club",
        category: "services",
        section: "services",
        cross_listed_in: &[],
        platforms: &["web"],
        content_policy: "Voice-acting and project-casting platform where NSFW projects and talent operate openly — paid NSFW roles are posted and filled (a flat-rate $470 USD NSFW role was live evidence in the sweep). Adult presence is practice-verified only: the platform's own NSFW policy page was NOT fetched in the sweep, so confirm the live terms before relying on them for a production.",
        automation_class: "prepared_submission",
        artifact_requirements: &[
            "casting post / project listing (role, rate, requirements) — services are contracted, not packaged releases",
        ],
        handoff_executor: "manual",
        submission_route: "Open signup; post casting calls or answer them; payment arranged per project (PayPal invoices in sampled listings).",
        verification_status: "verified",
        verification_note: "Operating (sweep 2026-10-27): NSFW paid roles and 18+ casting calls observed live [practice]; platform policy page not fetched — the caveat is part of this record.",
    },
    Channel {
        id: "ccbill",
        name: "CCBill",
        category: "payment_rail",
        section: "storefront_rail",
        cross_listed_in: &[],
        platforms: &["web"],
        content_policy: "Adult-friendly payment processor — part of the OWN-STOREFRONT RAIL: not a storefront; it bills in its own name for the studio's own storefront, filling the gap left by mainstream processors (Stripe prohibits adult; PayPal bars adult digital goods). Verified as serving digital-goods/software sellers, not only video. Published adult rates ~13-15.5% [secondary-grade]; CCBill's card-network registration runs ~$2,000/yr [secondary-grade] — confirm both against its own rate page at account setup.",
        automation_class: "pipeline",
        artifact_requirements: &[
            "own-storefront release (the studio's own shop sells the package; the rail only processes payment)",
            "merchant application / underwriting package per the processor",
        ],
        handoff_executor: "manual",
        submission_route: "Merchant application and underwriting — an application route, not open signup.",
        verification_status: "verified",
        verification_note: "Operating (sweep 2026-10-27): processor comparison and industry coverage [secondary]; headline rates are secondary-grade.",
    },
    Channel {
        id: "cgtrader",
        name: "CGTrader",
        category: "storefront",
        section: "assets",
        cross_listed_in: &[],
        platforms: &["web"],
        content_policy: "3D-model marketplace whose gated NSFW catalogue is live and selling: adult content is tolerated behind an age-confirmation interstitial, with adult-content flags on listings. This verdict rests on the live catalogue; no official adult-policy page was fetched in the sweep.",
        automation_class: "prepared_submission",
        artifact_requirements: &[
            "3D model packages (FBX/OBJ/STL and native formats), rigged and print models",
            "adult listings carry the platform's adult-content flag",
        ],
        handoff_executor: "manual",
        submission_route: "Open seller signup.",
        verification_status: "verified",
        verification_note: "Operating (sweep 2026-10-27): live gated NSFW catalogue observed [practice]; no official policy page fetched — the caveat is part of this record.",
    },
    Channel {
        id: "commishes",
        name: "Commishes (YCH.Commishes)",
        category: "services",
        section: "services",
        cross_listed_in: &[],
        platforms: &["web"],
        content_policy: "Commission and YCH (your-character-here) auction platform with explicit NSFW trade: live auctions offer SFW/NSFW slots and dedicated NSFW commission slots with explicit pricing tiers. Real auction/slot infrastructure (autobuy, sniping protection, portfolios, reputation pages) but NO platform escrow in sampled listings — payment is PayPal directly between the parties, and the platform fee is UNVERIFIED.",
        automation_class: "prepared_submission",
        artifact_requirements: &[
            "commission slot / auction listing (references, terms, pricing tiers) — services are contracted, not packaged releases",
        ],
        handoff_executor: "manual",
        submission_route: "Open signup; list auctions/slots as a seller or commission as a buyer.",
        verification_status: "verified",
        verification_note: "Operating (sweep 2026-10-27): live NSFW auctions and slots observed [practice]; platform fee not verified.",
    },
    Channel {
        id: "denpasoft",
        name: "Denpasoft",
        category: "storefront",
        section: "distribution",
        cross_listed_in: &[],
        platforms: &["windows"],
        content_policy: "Adult (18+) imprint of Sekai Project; download-only storefront since July 2025 (physical sales moved to Denkai Shoten). Curated catalog of localized visual novels and games.",
        automation_class: "curated_pitch",
        artifact_requirements: &[
            "windows_download_build",
            "package spec issued by the publisher during onboarding (not public)",
        ],
        handoff_executor: "manual",
        submission_route: "Publisher relationship with Sekai Project/Denpasoft; no public self-serve submission portal verified.",
        verification_status: "verified",
        verification_note: "Operating: company record updated 2025-07-21 (download-only transition); adult titles on sale.",
    },
    Channel {
        id: "dlsite",
        name: "DLsite English",
        category: "storefront",
        section: "distribution",
        cross_listed_in: &[],
        platforms: &["windows", "android"],
        content_policy: "Doujin/indie marketplace accepting General, R and Mature (18+) works; Japanese law governs; overseas creators are served through the DLsite English operation (CuriousFactory partner). Note: DLsite banned generative-AI content for manga/CG/video categories in 2023 — check the current per-category rule before submitting AI-assisted work. Registry extension (D-128 annex): DLsite's doujin categories also span software/materials and asset-type works, so this record covers DLsite materials/assets sales as well as games — an extension of this entry, not a separate target.",
        automation_class: "prepared_submission",
        artifact_requirements: &[
            "downloadable_archive (game build as a single archive)",
            "product metadata and sample images per the circle registration flow",
        ],
        handoff_executor: "manual",
        submission_route: "Circle (creator) registration on DLsite; international creator relations handled via DLsite English / CuriousFactory.",
        verification_status: "verified",
        verification_note: "Operating: English storefront actively selling English-language works with sales recorded through 2026-10-27; creator registration open. Materials/assets extension added 2026-10-27 from the D-128 channel sweep (secondary-grade sources; asset-specific depth in that pass was limited).",
    },
    Channel {
        id: "epoch",
        name: "Epoch",
        category: "payment_rail",
        section: "storefront_rail",
        cross_listed_in: &[],
        platforms: &["web"],
        content_policy: "Adult-friendly payment processor — part of the OWN-STOREFRONT RAIL: not a storefront; it bills in its own name for the studio's own storefront. Verified in the sweep's processor comparison as serving digital-goods/software sellers, not only video. Published adult rates ~13-15.5% across this processor class [secondary-grade] — confirm against Epoch's own rate page at account setup.",
        automation_class: "pipeline",
        artifact_requirements: &[
            "own-storefront release (the studio's own shop sells the package; the rail only processes payment)",
            "merchant application / underwriting package per the processor",
        ],
        handoff_executor: "manual",
        submission_route: "Merchant application and underwriting — an application route, not open signup.",
        verification_status: "verified",
        verification_note: "Operating (sweep 2026-10-27): processor comparison coverage [secondary]; headline rates are secondary-grade.",
    },
    Channel {
        id: "erolabs",
        name: "EroLabs",
        category: "platform",
        section: "distribution",
        cross_listed_in: &[],
        platforms: &["windows", "android", "web"],
        content_policy: "Adult game platform (Taiwan-based, English service); free-to-play and premium adult titles, predominantly gacha/RPG, plus single-game sales. Reinstated as a target by D-126.",
        automation_class: "prepared_submission",
        artifact_requirements: &[
            "platform package per EroLabs partner onboarding (PC build and/or Android package)",
            "exact package spec issued by the platform; not public",
        ],
        handoff_executor: "manual",
        submission_route: "Publisher/platform partnership; titles are published through EroLabs as publisher of record for many releases.",
        verification_status: "verified",
        verification_note: "Operating: new releases and live-service events dated through 2026-10-27 on the platform's own news feed.",
    },
    Channel {
        id: "etsy",
        name: "Etsy",
        category: "storefront",
        section: "assets_sfw",
        cross_listed_in: &[],
        platforms: &["web"],
        content_policy: "SFW-only for estate purposes. Since 2024-07-29 Etsy prohibits most sex toys, pornography (incl. vintage), photographic/photo-realistic genitalia and sex acts, and custom sexual content. What survives is a narrow remnant: non-photographic depictions of sex acts WITHOUT visible genitalia, tagged 'mature' with an obscured first thumbnail — too narrow for estate rigs/assets. Recorded for completeness of the SFW map, in the separate SFW-only section.",
        automation_class: "prepared_submission",
        artifact_requirements: &[
            "digital-download listings within the post-2024 remnant only (non-photographic, no visible genitalia, mature-tagged, obscured first thumbnail)",
        ],
        handoff_executor: "manual",
        submission_route: "Open shop signup.",
        verification_status: "verified",
        verification_note: "Operating (sweep 2026-10-27): ban scope from Etsy's own mature-content policy plus press coverage [official + secondary]. SFW-only record — the gate keeps adult projects out.",
    },
    Channel {
        id: "fab",
        name: "Fab (Epic)",
        category: "storefront",
        section: "assets_sfw",
        cross_listed_in: &[],
        platforms: &["web"],
        content_policy: "SFW-only for estate purposes. Epic's marketplace allows 'mature' content behind show/blur/hide controls and parental-control gating, but mature is not explicit: the Epic ecosystem refuses AO-rated products, and mature content cannot be used in UEFN. A mandatory 'Created with AI' seller declaration applies (since May 2025) and buyers can filter AI content out. Recorded in the separate SFW-only section, never as an adult channel.",
        automation_class: "prepared_submission",
        artifact_requirements: &[
            "3D models, environments, VFX, audio and tool packages per Fab seller onboarding (via Epic)",
            "mandatory Created-with-AI declaration where applicable",
        ],
        handoff_executor: "manual",
        submission_route: "Seller onboarding via Epic.",
        verification_status: "verified",
        verification_note: "Operating (sweep 2026-10-27): mature-content posture and the AI declaration from Epic-adjacent and secondary sources [official-adjacent/secondary]. SFW-only record.",
    },
    Channel {
        id: "fakku",
        name: "FAKKU",
        category: "storefront",
        section: "distribution",
        cross_listed_in: &[],
        platforms: &["windows"],
        content_policy: "Largest English-language adult publisher; publishes and distributes adult games since 2016, on its own store and via partner storefronts. Curated catalog.",
        automation_class: "curated_pitch",
        artifact_requirements: &[
            "windows_download_build",
            "package spec issued by the publisher during onboarding (not public)",
        ],
        handoff_executor: "manual",
        submission_route: "Publishing partnership — FAKKU works with game publishers and developers directly; no self-serve submission portal verified.",
        verification_status: "verified",
        verification_note: "Operating: company status active; new game publishing announcements dated October 2026.",
    },
    Channel {
        id: "fansly",
        name: "Fansly",
        category: "membership",
        section: "services",
        cross_listed_in: &[],
        platforms: &["web"],
        content_policy: "Adult platform (purpose-built): subscriptions, PPV, tips and custom content requests — customs are negotiated via direct messaging / PPV rather than an automated order desk. Tracked in the services section for its custom-order flow. Platform takes 20% (creator keeps 80%) across subscriptions/tips/PPV/customs [secondary-grade]; payouts weekly on Mondays, $100 minimum, via ACH (US), wire, crypto (BTC/ETH) or Paxum [secondary-grade] — confirm against Fansly's own pages at account setup (the sweep did not fetch them).",
        automation_class: "prepared_submission",
        artifact_requirements: &[
            "creator profile and priced custom-request offers (DM/PPV) — services and membership content, not packaged releases",
        ],
        handoff_executor: "manual",
        submission_route: "Open creator signup.",
        verification_status: "verified",
        verification_note: "Operating (sweep 2026-10-27) [secondary — multiple creator-economics sources; the platform's own pages were not fetched].",
    },
    Channel {
        id: "fdroid_self_hosted",
        name: "Self-hosted F-Droid repository (NTRanch)",
        category: "self_hosted",
        section: "distribution",
        cross_listed_in: &[],
        platforms: &["android"],
        content_policy: "Estate-operated channel (D-095 route): the studio's own F-Droid repository, so content policy is the studio's own; builds must still pass the fdroid publish gates owned by the publish skills.",
        automation_class: "prepared_submission",
        artifact_requirements: &[
            "signed_apk",
            "fdroiddata metadata recipe per the fdroid-publish skill",
        ],
        handoff_executor: "slipway",
        submission_route: "Studio-operated repository; slipway executes the Android handoff and certification.",
        verification_status: "verified",
        verification_note: "Estate route, not a third-party channel: recorded per D-095/D-126; readiness lives in the publish skills, not here.",
    },
    Channel {
        id: "flipped_normals",
        name: "FlippedNormals",
        category: "storefront",
        section: "assets",
        cross_listed_in: &[],
        platforms: &["web"],
        content_policy: "3D marketplace where NSFW models are openly sold (adult characters; category copy references 'nude, clothed, and NSFW options'). A strict official policy governs products depicting minors (no sexual content, fully clothed, neutral presentation) — adult work listed here must be unambiguously adult. Payout terms were not verified in the sweep.",
        automation_class: "prepared_submission",
        artifact_requirements: &[
            "3D model / Daz content / tutorial packages via the open-a-store route",
            "payout terms UNVERIFIED in the sweep — confirm at store setup",
        ],
        handoff_executor: "manual",
        submission_route: "Open-a-store seller route (site footer).",
        verification_status: "verified",
        verification_note: "Operating (sweep 2026-10-27): live NSFW products observed [practice]; the minors policy is from the platform's own help (updated 2024-08-19) [official].",
    },
    Channel {
        id: "gamcore",
        name: "Gamcore",
        category: "portal",
        section: "distribution",
        cross_listed_in: &[],
        platforms: &["web"],
        content_policy: "Free, advertising-funded browser portal for adult (18+) HTML5 games; curated listings, developer submission is free per the portal's published creator terms; no downloads, games run in-browser.",
        automation_class: "prepared_submission",
        artifact_requirements: &[
            "html5_browser_build (hosted/embeddable)",
            "listing metadata (title, description, thumbnails)",
        ],
        handoff_executor: "manual",
        submission_route: "Developer submission to the portal for consideration; no public self-serve upload dashboard verified — submission is portal-mediated.",
        verification_status: "verified",
        verification_note: "Operating: portal live with current catalog and trust reviews dated October 2026; submission terms from secondary coverage — treat exact revenue terms as unconfirmed.",
    },
    Channel {
        id: "gog",
        name: "GOG",
        category: "storefront",
        section: "distribution",
        cross_listed_in: &[],
        platforms: &["windows", "macos", "linux"],
        content_policy: "General DRM-free storefront with selective, curated adult catalogue: adult-rated titles are accepted case-by-case through curation; some listed adult titles historically shipped censored with off-store restoration patches, others fully uncensored — the per-title arrangement is part of the pitch outcome, never assumed.",
        automation_class: "curated_pitch",
        artifact_requirements: &[
            "drm_free_installer_build (windows; macos/linux where supported)",
            "GOG build and metadata spec issued after curation acceptance",
        ],
        handoff_executor: "manual",
        submission_route: "Developer pitch via GOG's game submission process; curation decides.",
        verification_status: "verified",
        verification_note: "Operating: actively selling adult-tagged titles in 2026 and publicly campaigning on adult-game availability (Freedom to Buy, Aug 2025). Selective — verification confirms the channel exists and takes adult titles, not that any given title will be accepted.",
    },
    Channel {
        id: "indie_gala",
        name: "IndieGala",
        category: "storefront",
        section: "distribution",
        cross_listed_in: &[],
        platforms: &["windows"],
        content_policy: "General storefront with an adult catalogue: NSFW game keys sold behind 18+ gates and some DRM-free adult downloads (e.g. 'Fallen Girls DRM-Free Download'); publisher sales run for adult labels (TinyHat, NSFW18 Games). The usable segment here is the DRM-free/direct-download part — much of the adult catalogue is Steam-key resale, and Steam is tabled (D-126).",
        automation_class: "curated_pitch",
        artifact_requirements: &[
            "drm_free download build for the direct-download segment",
            "pitch / partnership package per IndieGala's curated flow",
        ],
        handoff_executor: "manual",
        submission_route: "Curated / partnership route with the storefront.",
        verification_status: "verified",
        verification_note: "Operating (sweep 2026-10-27): adult catalogue and DRM-free adult downloads observed on the store [practice].",
    },
    Channel {
        id: "itch_io",
        name: "itch.io",
        category: "storefront",
        section: "distribution",
        cross_listed_in: &[],
        platforms: &["windows", "macos", "linux", "web", "android"],
        content_policy: "Open marketplace; adult (NSFW) games are sold with an adult content flag. Since 2025, paid adult games are not indexed in browse/search (payment-partner pressure) — discoverability for paid NSFW is limited; free adult releases and direct links still work. AI-generated content requires disclosure. Asset packs are a first-class catalogue here — Live2D model sets, premade rigged VTuber models, VRM/VRoid avatars, sprite sheets, animation packs — so this record also covers itch.io asset packs as a registry extension (D-128 annex); AI disclosure is strictly enforced for asset packs since Nov 2024.",
        automation_class: "pipeline",
        artifact_requirements: &[
            "per-platform zip archives (desktop builds)",
            "html5 zip with index.html at root for web play (<=1000 files, extracted <=500MB via web upload)",
            "android apk upload for the Android channel",
            "cover image and screenshots; uploads via butler support up to 2GB per file (web upload 1GB)",
            "asset pack archives (Live2D sets, VRM avatars, sprite/animation packs) via the same butler pipeline — extension per the D-128 annex",
        ],
        handoff_executor: "butler",
        submission_route: "Self-serve: creator account, project page, butler push per platform channel.",
        verification_status: "verified",
        verification_note: "Operating: marketplace live; upload limits and adult-indexing posture confirmed from current itch.io documentation coverage (Oct 2026). Asset-pack extension from the D-128 channel sweep: live asset listings observed (Live2D sets, rigged VTuber models, VRM/avatar packs) [practice]; AI-disclosure enforcement for asset packs [secondary].",
    },
    Channel {
        id: "iwantclips",
        name: "IWantClips",
        category: "services",
        section: "services",
        cross_listed_in: &[],
        platforms: &["web"],
        content_policy: "Adult platform (purpose-built) with a formal custom-order system: storefronts expose ORDER CUSTOM / iWantCustomClips with creator-set prices, plus TRIBUTE ME ($10-$10,000) and iWantPhone. Tracked in the services section for custom video/photo content orders (performer content). Creators keep 75% (25% platform) [secondary-grade]; payouts via direct deposit or Paxum [secondary-grade] — confirm at account setup.",
        automation_class: "prepared_submission",
        artifact_requirements: &[
            "custom-order storefront configuration (priced order menu) — services are contracted, not packaged releases",
        ],
        handoff_executor: "manual",
        submission_route: "Open creator signup.",
        verification_status: "verified",
        verification_note: "Operating (sweep 2026-10-27): the custom-order system from storefront walkthroughs and reviews [secondary].",
    },
    Channel {
        id: "jast_usa",
        name: "JAST USA",
        category: "storefront",
        section: "distribution",
        cross_listed_in: &[],
        platforms: &["windows"],
        content_policy: "English publisher/localizer of Japanese visual novels and games (eroge included) since 1996; own web store sells DRM-free downloads. Curated, partnership-based catalog.",
        automation_class: "curated_pitch",
        artifact_requirements: &[
            "windows_download_build (drm-free)",
            "package spec issued by the publisher during onboarding (not public)",
        ],
        handoff_executor: "manual",
        submission_route: "Partnership with the publisher (they work directly with creators and studios); no self-serve submission portal verified.",
        verification_status: "verified",
        verification_note: "Operating: store live with current releases (site crawled Oct 2026); actively publishing partner titles.",
    },
    Channel {
        id: "johren",
        name: "Johren",
        category: "platform",
        section: "distribution",
        cross_listed_in: &[],
        platforms: &["windows", "web", "android"],
        content_policy: "English/Traditional-Chinese adult game platform (founded 2020): premium downloads and browser titles, 18+ versions sold alongside all-ages storefront releases elsewhere; own credits currency.",
        automation_class: "prepared_submission",
        artifact_requirements: &[
            "platform download package per Johren partner spec (windows build; web build for browser titles)",
            "store metadata in English (and Traditional Chinese where applicable)",
        ],
        handoff_executor: "manual",
        submission_route: "Publisher/platform partnership (Shiravune, Co-FUN and others publish 18+ editions here); package spec issued during onboarding.",
        verification_status: "verified",
        verification_note: "Operating: new 18+ download releases dated July 2026; platform traffic current as of September 2026.",
    },
    Channel {
        id: "kagura_games",
        name: "Kagura Games",
        category: "storefront",
        section: "distribution",
        cross_listed_in: &[],
        platforms: &["windows"],
        content_policy: "US publisher (founded 2018) localizing and distributing Japanese adult games; own storefront sells downloads and off-store restoration patches for censored editions sold elsewhere. Curated catalog.",
        automation_class: "curated_pitch",
        artifact_requirements: &[
            "windows_download_build",
            "package spec issued by the publisher during onboarding (not public)",
        ],
        handoff_executor: "manual",
        submission_route: "Publishing partnership; no public self-serve submission portal verified.",
        verification_status: "verified",
        verification_note: "Operating: new published releases dated October 2026 on its own store and partner stores.",
    },
    Channel {
        id: "mangagamer",
        name: "MangaGamer",
        category: "storefront",
        section: "distribution",
        cross_listed_in: &[],
        platforms: &["windows", "macos", "linux"],
        content_policy: "English adult visual-novel publisher/storefront; titles are licensed from partners; full uncut adult editions on its own store, all-ages editions elsewhere. Licensing requires a content rating for the work (per partner guidance).",
        automation_class: "curated_pitch",
        artifact_requirements: &[
            "download_build (windows; macos/linux where the title supports them)",
            "content rating for the work",
            "package spec issued at licensing (not public)",
        ],
        handoff_executor: "manual",
        submission_route: "Licensing partnership — MangaGamer acquires/licenses titles from developers and circles; no self-serve submission portal verified.",
        verification_status: "verified",
        verification_note: "Operating: new licensing and release announcements dated December 2025 and through 2026.",
    },
    Channel {
        id: "manyvids",
        name: "ManyVids",
        category: "services",
        section: "services",
        cross_listed_in: &[],
        platforms: &["web"],
        content_policy: "Adult platform (purpose-built) with an automated custom-video request system: creators set a per-minute rate plus add-on charges; customs are capped at $9,999 per the Pricing Policy. Splits per secondary reviews: clip sales 60% creator; customs, subscriptions and tips 80% creator [secondary-grade]; weekly Wednesday payouts, $50 minimum, via direct deposit/ACH/wire/Paxum [secondary-grade] — confirm against ManyVids' own commission table at account setup.",
        automation_class: "prepared_submission",
        artifact_requirements: &[
            "custom-request configuration (per-minute rate, add-on charges) — services are contracted, not packaged releases",
        ],
        handoff_executor: "manual",
        submission_route: "Open creator signup.",
        verification_status: "verified",
        verification_note: "Operating (sweep 2026-10-27) [secondary — comparison and review sources; the platform's own commission table was not fetched].",
    },
    Channel {
        id: "newgrounds",
        name: "Newgrounds",
        category: "portal",
        section: "distribution",
        cross_listed_in: &[],
        platforms: &["web"],
        content_policy: "Independent creator portal (games/movies/audio/art). Adult games allowed under the A (Adults only 18+) rating with account + age confirmation required to view since 2022; hard bans: photography/video of real naked people, sexual content involving underage characters (no grey areas). Not a sales channel — discovery and community, creators link out to their stores/memberships.",
        automation_class: "prepared_submission",
        artifact_requirements: &[
            "html5_browser_build upload",
            "correct content rating (A for explicit) and descriptors set at submission",
        ],
        handoff_executor: "manual",
        submission_route: "Self-serve creator upload; new submissions pass the portal's judgement process.",
        verification_status: "verified",
        verification_note: "Operating: portal live and independent; game guidelines and A-rating rules current (guidelines mirror crawled Oct 2026).",
    },
    Channel {
        id: "nizima",
        name: "nizima",
        category: "storefront",
        section: "assets_sfw",
        cross_listed_in: &[],
        platforms: &["web"],
        content_policy: "SFW-only in sampled listing terms. Live2D Inc.'s marketplace for illustrations and Live2D data (.moc3-class models) with Live2D preview — the Live2D-native asset channel — but sampled listing terms prohibit use of models for adult-oriented contents, the surface is Japanese-first (English mainly via machine-translated listings), and on-platform PayPal appears available only to residents of Japan: the US-seller payout route is UNVERIFIED and likely blocked on-platform. Recorded in the separate SFW-only section.",
        automation_class: "prepared_submission",
        artifact_requirements: &[
            "Live2D data / illustration listings per nizima's seller flow",
            "US-seller payout route UNVERIFIED — do not rely on on-platform payment; overseas buyers are directed to contact the seller directly",
        ],
        handoff_executor: "manual",
        submission_route: "Seller listings on nizima under a Live2D account.",
        verification_status: "verified",
        verification_note: "Operating (sweep 2026-10-27) [secondary/practice]: marketplace live; payout posture from seller-listing notes — UNVERIFIED for US sellers, recorded rather than filled in. SFW-only record.",
    },
    Channel {
        id: "nizima_requests",
        name: "nizima make-to-order",
        category: "services",
        section: "services_sfw",
        cross_listed_in: &[],
        platforms: &["web"],
        content_policy: "SFW-only. nizima's make-to-order commission route for Live2D data / illustrations with Live2D preview. It inherits the nizima verdict: sampled terms prohibit adult-oriented use of delivered models, and the US-seller payout route is UNVERIFIED and likely blocked on-platform. Noted as an SFW-only service — not an adult recommendation.",
        automation_class: "prepared_submission",
        artifact_requirements: &[
            "make-to-order commission listing (Live2D data / illustration, SFW only)",
        ],
        handoff_executor: "manual",
        submission_route: "Requests via nizima, under the same nizima account identity as the nizima asset record.",
        verification_status: "verified",
        verification_note: "Operating (sweep 2026-10-27): inherits the nizima sources [secondary/practice]; US-seller payout UNVERIFIED. SFW-only record.",
    },
    Channel {
        id: "nutaku",
        name: "Nutaku",
        category: "platform",
        section: "distribution",
        cross_listed_in: &[],
        platforms: &["web", "android", "windows"],
        content_policy: "Adult game portal (launched 2015) for browser and downloadable games; free-to-play and premium; developer program offers hosting, monetization and marketing services.",
        automation_class: "prepared_submission",
        artifact_requirements: &[
            "browser (html5) build integrated with the Nutaku developer portal",
            "signed android apk (developer-signed; signing key must never change after launch), tested in the portal sandbox before production upload",
            "desktop download build where applicable",
        ],
        handoff_executor: "slipway",
        submission_route: "Developer/publisher portal (nutaku.net publishers program); technical documentation in the Nutaku developer docs (APK preparation and publishing flow).",
        verification_status: "verified",
        verification_note: "Operating: developer documentation live with a current APK publishing flow; publisher program active.",
    },
    Channel {
        id: "onlyfans",
        name: "OnlyFans",
        category: "membership",
        section: "distribution",
        cross_listed_in: &["services"],
        platforms: &["web"],
        content_policy: "Adult platform (purpose-built): subscriptions, PPV, tips and custom orders — custom photo sets / video orders are a standard revenue stream, negotiated in DMs rather than through an automated order desk. Membership distribution alongside Patreon and SubscribeStar; cross-listed in the services section for its custom-order flow. Added under D-126's every-English-adult-site instruction as the sweep's registry-gap observation (it was absent from the D-126 set while Patreon/Subscribestar were present). Platform takes 20% (creator keeps 80%) on every transaction incl. custom orders [secondary-grade]; weekly payouts, $20 minimum, bank transfer [secondary-grade] — confirm at account setup.",
        automation_class: "prepared_submission",
        artifact_requirements: &[
            "subscriber posts / PPV delivering works per tier",
            "custom-order offers via DM (the services cross-listing)",
        ],
        handoff_executor: "manual",
        submission_route: "Open creator signup.",
        verification_status: "verified",
        verification_note: "Operating (sweep 2026-10-27) [secondary — creator-economics sources only; the platform's own pages were not fetched].",
    },
    Channel {
        id: "patreon",
        name: "Patreon",
        category: "membership",
        section: "distribution",
        cross_listed_in: &[],
        platforms: &["web", "windows", "android"],
        content_policy: "Membership distribution, not a storefront. Adult/18+ work is permitted only behind the subscription paywall for paying members; public page surfaces must stay clean. Finalised policy (March 2026): characters must appear unmistakably adult; incest, bestiality and sexual deepfakes of real people are banned everywhere. Adult game builds are commonly distributed to members as downloads/posts while public demos live elsewhere.",
        automation_class: "prepared_submission",
        artifact_requirements: &[
            "member-only posts delivering the build (download archive) per tier",
            "public page and previews free of explicit content",
        ],
        handoff_executor: "manual",
        submission_route: "Creator account with Adult/18+ category enabled; distribution is the creator's own page.",
        verification_status: "verified",
        verification_note: "Operating: adult game creators actively distributing member builds in 2026; policy text quoted from the live guidelines in March 2026 coverage.",
    },
    Channel {
        id: "renderhub",
        name: "RenderHub",
        category: "storefront",
        section: "assets_sfw",
        cross_listed_in: &[],
        platforms: &["web"],
        content_policy: "SFW-only going forward. 3D-model marketplace whose own support FAQ states it does not allow NSFW 18+ content, and whose official forum announcement discontinues adult content platform-wide (earlier purges confirmed in forum threads). NSFW listings still lingering at crawl time are being removed — do NOT list adult assets here. Recorded in the separate SFW-only section.",
        automation_class: "prepared_submission",
        artifact_requirements: &[
            "SFW 3D model packages only — adult content is being removed platform-wide",
        ],
        handoff_executor: "manual",
        submission_route: "Vendor storefronts.",
        verification_status: "verified",
        verification_note: "Operating (sweep 2026-10-27): the removal posture is from RenderHub's own FAQ and forum announcement [official]. SFW-only record.",
    },
    Channel {
        id: "renderosity",
        name: "Renderosity",
        category: "storefront",
        section: "assets",
        cross_listed_in: &[],
        platforms: &["web"],
        content_policy: "3D marketplace and gallery with an official Content Maturity Filter — Safe / Mature / Adult ('all types of suggestive and explicit (18+) content') — covering Marketplace and Gallery, so the Adult tier is official, not tolerated-by-neglect. Caveat: gallery/promo-image rules have historically been stricter than product content. Royalties vary by membership tier [secondary].",
        automation_class: "prepared_submission",
        artifact_requirements: &[
            "3D model / content product packages through the vendor review gate",
        ],
        handoff_executor: "manual",
        submission_route: "Vendor submissions through review.",
        verification_status: "verified",
        verification_note: "Operating (sweep 2026-10-27): the Adult tier is from the platform's own newsletter [official-adjacent]; royalty detail [secondary].",
    },
    Channel {
        id: "renderotica",
        name: "Renderotica / CGBytes",
        category: "storefront",
        section: "assets",
        cross_listed_in: &[],
        platforms: &["web"],
        content_policy: "Purpose-built adult 3D-asset marketplace (CGBytes Inc., Canada; operating renderotica.com since 2000): 3D models, textures, and comics/art products of the Daz/Poser class. Payout terms were not verified in the sweep — confirm at vendor onboarding.",
        automation_class: "prepared_submission",
        artifact_requirements: &[
            "3D model / texture / art product packages per vendor onboarding",
            "payout terms UNVERIFIED in the sweep — confirm at vendor onboarding",
        ],
        handoff_executor: "manual",
        submission_route: "Vendor submissions.",
        verification_status: "verified",
        verification_note: "Operating (sweep 2026-10-27): the marketplace is described in a UDRP decision record and competitor profiles [secondary].",
    },
    Channel {
        id: "segpay",
        name: "Segpay",
        category: "payment_rail",
        section: "storefront_rail",
        cross_listed_in: &[],
        platforms: &["web"],
        content_policy: "Adult-friendly payment processor — part of the OWN-STOREFRONT RAIL: not a storefront; it bills in its own name for the studio's own storefront. Verified in the sweep's processor comparison as serving digital-goods/software sellers, not only video. Published adult rates ~13-15.5% across this processor class [secondary-grade] — confirm against Segpay's own rate page at account setup.",
        automation_class: "pipeline",
        artifact_requirements: &[
            "own-storefront release (the studio's own shop sells the package; the rail only processes payment)",
            "merchant application / underwriting package per the processor",
        ],
        handoff_executor: "manual",
        submission_route: "Merchant application and underwriting — an application route, not open signup.",
        verification_status: "verified",
        verification_note: "Operating (sweep 2026-10-27): processor comparison coverage [secondary]; headline rates are secondary-grade.",
    },
    Channel {
        id: "shady_corner",
        name: "Shady Corner Games",
        category: "publisher",
        section: "distribution",
        cross_listed_in: &[],
        platforms: &["windows", "web"],
        content_policy: "Adult publisher that publishes third-party titles (e.g. BoneTown) and also sells direct from its own storefront (shadycorner.com — e.g. Heavy Hearts with a $12.00 buy button). The route is a curated publishing pitch; sales land on its storefront and partner stores.",
        automation_class: "curated_pitch",
        artifact_requirements: &[
            "game build + pitch package per the publisher's onboarding (no public spec)",
        ],
        handoff_executor: "manual",
        submission_route: "Curated pitch to the publisher.",
        verification_status: "verified",
        verification_note: "Operating (sweep 2026-10-27): third-party publishing and direct storefront sales observed [practice]; publisher-page coverage [secondary].",
    },
    Channel {
        id: "sidequest",
        name: "SideQuest",
        category: "platform",
        section: "distribution",
        cross_listed_in: &[],
        platforms: &["android"],
        content_policy: "VR (Quest) sideload marketplace: open developer submissions including paid apps, with a light admin process; adult content is present in its ecosystem (a dedicated NSFW repository in the ecosystem and erotic VR titles sold 'for SideQuest'). CAVEAT carried from the sweep: SideQuest's own content-policy page was NOT fetched — operating status and adult presence rest on secondary/practice sources; confirm the official policy before a first submission.",
        automation_class: "prepared_submission",
        artifact_requirements: &[
            "quest (android) apk package for sideload distribution",
            "store listing metadata per SideQuest's developer flow",
        ],
        handoff_executor: "manual",
        submission_route: "Open developer submissions with a light admin process.",
        verification_status: "verified",
        verification_note: "Operating (sweep 2026-10-27): submissions and paid apps from a SideQuest guide [secondary]; adult presence from ecosystem evidence [practice]. Official content-policy page not fetched — the caveat is part of this record, per the annex.",
    },
    Channel {
        id: "skima",
        name: "SKIMA",
        category: "services",
        section: "services_sfw",
        cross_listed_in: &[],
        platforms: &["web"],
        content_policy: "SFW-only in practice. Japanese commission platform with an English UI, but every sampled listing's standard terms state 'No R18' and bar use violating public order and morals, including R18 use of delivered work — identical boilerplate across unrelated sellers indicates a platform-level template/norm. SKIMA's own official policy page was not separately fetched; the evidence is the uniform listing boilerplate. Noted as an SFW-only service — not an adult recommendation.",
        automation_class: "prepared_submission",
        artifact_requirements: &["illustration commission listings (SFW only)"],
        handoff_executor: "manual",
        submission_route: "Open signup (commission.skima.jp/en).",
        verification_status: "verified",
        verification_note: "Operating (sweep 2026-10-27) [practice — uniform listing terms across sellers]. SFW-only record.",
    },
    Channel {
        id: "subscribestar",
        name: "SubscribeStar (incl. SubscribeStar Adult)",
        category: "membership",
        section: "distribution",
        cross_listed_in: &[],
        platforms: &["web", "windows", "android"],
        content_policy: "Membership distribution, not a storefront; the Adult property explicitly permits adult content and is the standard fallback for adult game creators; flat 5% platform fee. Policy watch: terms were changed in March 2026 to ban several fictional themes, with a rework announced — re-read the live terms before each campaign.",
        automation_class: "prepared_submission",
        artifact_requirements: &[
            "subscriber posts delivering the build (download archive) per tier",
            "tier structure and page copy per the live terms",
        ],
        handoff_executor: "manual",
        submission_route: "Creator (Star) account; approval can take time; distribution is the creator's own page.",
        verification_status: "verified",
        verification_note: "Operating: platform live in 2026 with an active adult creator base; fee and adult posture confirmed in current coverage.",
    },
    Channel {
        id: "superhive",
        name: "Superhive (formerly Blender Market)",
        category: "storefront",
        section: "assets",
        cross_listed_in: &[],
        platforms: &["web"],
        content_policy: "Blender models/rigs marketplace where labelled NSFW rigs are openly sold (e.g. 'Stacy Defaultson NSFW - Advanced Rigged 3D Female Character', ~$15, and a removable-clothing NSFW anime rig) — the best evidenced fit for Blender-rigged parametric characters. POLICY TENSION, recorded per the annex: Superhive's Creator Policy bars content that is 'graphic, obscene, or vulgar' [official] while enforcement evidently permits labelled NSFW character rigs [practice]; explicit sex acts are NOT evidenced. Listings must stay on the evidenced side of that line — nudity/rigs yes, explicit material at-risk.",
        automation_class: "prepared_submission",
        artifact_requirements: &[
            "Blender model/rig packages (incl. rigged characters) via creator storefronts",
            "listings stay on the evidenced side of the policy tension (see content_policy)",
        ],
        handoff_executor: "manual",
        submission_route: "Creator storefronts (open seller route).",
        verification_status: "verified",
        verification_note: "Operating (sweep 2026-10-27): live NSFW rig listings [practice] read against the Creator Policy text [official] — the tension is carried in this record, per the annex.",
    },
    Channel {
        id: "tinyhat",
        name: "TinyHat Studios",
        category: "publisher",
        section: "distribution",
        cross_listed_in: &[],
        platforms: &["windows"],
        content_policy: "Adult publishing label for third-party titles (e.g. Team Nimbus's Cloud Meadow). No own storefront found — the route is a publishing deal; sales land on partner stores (its titles appear in IndieGala publisher sales and on itch.io).",
        automation_class: "curated_pitch",
        artifact_requirements: &[
            "game build + pitch package per the label's onboarding (no public spec)",
        ],
        handoff_executor: "manual",
        submission_route: "Curated pitch (publishing deal).",
        verification_status: "verified",
        verification_note: "Operating (sweep 2026-10-27): third-party publishing evidenced via store/publisher pages [practice/secondary].",
    },
    Channel {
        id: "unity_asset_store",
        name: "Unity Asset Store",
        category: "storefront",
        section: "assets_sfw",
        cross_listed_in: &[],
        platforms: &["web"],
        content_policy: "SFW-only (verified from the official Submission Guidelines): 'Models or images do not contain genitalia. Characters must have sufficient coverage as appropriate'; the sole nude exception is base characters marketed and presented in a neutral pose and rendered in a realistic style for medical or educational purposes. Recorded in the separate SFW-only section.",
        automation_class: "prepared_submission",
        artifact_requirements: &[
            "SFW model/character/tool/VFX packages per the official Submission Guidelines",
            "publisher account with payout profile",
        ],
        handoff_executor: "manual",
        submission_route: "Open publisher accounts.",
        verification_status: "verified",
        verification_note: "Operating (sweep 2026-10-27): the SFW posture is from the official Submission Guidelines and publishing docs [official]. SFW-only record.",
    },
    Channel {
        id: "verotel",
        name: "Verotel",
        category: "payment_rail",
        section: "storefront_rail",
        cross_listed_in: &[],
        platforms: &["web"],
        content_policy: "Adult-friendly payment processor — part of the OWN-STOREFRONT RAIL: not a storefront; it bills in its own name for the studio's own storefront. Verified in the sweep's processor comparison as serving digital-goods/software sellers, not only video. Published adult rates ~13-15.5% across this processor class [secondary-grade] — confirm against Verotel's own rate page at account setup.",
        automation_class: "pipeline",
        artifact_requirements: &[
            "own-storefront release (the studio's own shop sells the package; the rail only processes payment)",
            "merchant application / underwriting package per the processor",
        ],
        handoff_executor: "manual",
        submission_route: "Merchant application and underwriting — an application route, not open signup.",
        verification_status: "verified",
        verification_note: "Operating (sweep 2026-10-27): processor comparison coverage [secondary]; headline rates are secondary-grade.",
    },
    Channel {
        id: "vgen",
        name: "VGen",
        category: "services",
        section: "services",
        cross_listed_in: &[],
        platforms: &["web"],
        content_policy: "Commission platform whose official NSFW Policy reads: 'Adult content is allowed. Porn is not.' Drawn/animated/audio work is in scope under the tag regime (Sexually suggestive 18+, Sexual nudity 18+, Heavy gore/violence 18+; 18+ items hidden from under-18 users; untagged mature content risks removal/suspension); 'porn' in VGen's borrowed definition means photos/video of actual humans in sexual activities — the bound comes from its processors (Stripe + PayPal). Illustration, Live2D art + rigging (a staple category), animation and writing commissions run through per-service request forms with attached ToS; a Shop sells one-of-a-kind/stock products with maturity ratings. Platform fee 5% charged to the artist plus processing (Stripe, US: 2.9% + $0.30; PayPal path = 5% + own PayPal fees) [secondary-grade — confirm at account setup].",
        automation_class: "prepared_submission",
        artifact_requirements: &[
            "service listings with per-service request forms and attached ToS (illustration, Live2D art + rigging, animation, writing) — services are contracted, not packaged releases",
        ],
        handoff_executor: "manual",
        submission_route: "Open artist signup; services listed per category.",
        verification_status: "verified",
        verification_note: "Operating (sweep 2026-10-27): NSFW policy and tag taxonomy from VGen's own help centre [official]; live NSFW commission trade observed [practice]; fee figures [secondary].",
    },
    Channel {
        id: "voice_acting_club",
        name: "Voice Acting Club",
        category: "services",
        section: "services",
        cross_listed_in: &[],
        platforms: &["web"],
        content_policy: "Voice-acting community (Discord-based) with formal NSFW casting infrastructure: the age-restricted #casting-calls-18-up channel hosts NSFW project casting, and its official rules REQUIRE projects featuring explicit sexual content to offer monetary compensation. Also runs #non-vo-services-wanted for hiring artists/writers/composers/audio editors. This is the registry's hiring route for adult-game voice talent. No platform fee — community infrastructure, not an escrow marketplace; payment is arranged between the parties.",
        automation_class: "curated_pitch",
        artifact_requirements: &[
            "casting post in the 18-up channel (project, roles, compensation — compensation is REQUIRED for explicit projects)",
        ],
        handoff_executor: "manual",
        submission_route: "Community posting (curated-pitch class: a post to the community, not a storefront submission).",
        verification_status: "verified",
        verification_note: "Operating (sweep 2026-10-27): channel structure and the compensation rule from the community's own rules page [official].",
    },
    Channel {
        id: "zoom_platform",
        name: "Zoom Platform",
        category: "storefront",
        section: "distribution",
        cross_listed_in: &[],
        platforms: &["windows", "macos", "linux"],
        content_policy: "DRM-free storefront for retro and classic games (operating since 2014), including mature-rated classics. NOT VERIFIED as an adult-games channel: no evidence found that it sells explicit adult (eroge/adults-only) titles or accepts them; it is itself under payment-processor pressure over mature content in retro titles.",
        automation_class: "curated_pitch",
        artifact_requirements: &[
            "unverified — no adult-title package requirements could be established; do not prepare against invented specs",
        ],
        handoff_executor: "manual",
        submission_route: "Publisher partnerships for catalog re-releases; no adult-game submission route verified.",
        verification_status: "flagged",
        verification_note: "FLAGGED 2026-10-27: storefront is operating, but adult-game acceptance could not be verified from any source — its catalogue is retro/general with mature-rated classics, not adults-only titles. Handoffs refused until verification improves.",
    },
];

/// Find a channel by id.
pub fn find_channel(id: &str) -> Option<&'static Channel> {
    CHANNELS.iter().find(|c| c.id == id)
}

/// The registry as wire data.
pub fn registry_json() -> Value {
    Value::Array(CHANNELS.iter().map(channel_json).collect())
}

/// One channel as wire data, including its group-10 auth record.
pub fn channel_json(channel: &Channel) -> Value {
    let auth = auth_for(channel.id);
    json!({
        "id": channel.id,
        "name": channel.name,
        "category": channel.category,
        "section": channel.section,
        "cross_listed_in": channel.cross_listed_in,
        "platforms": channel.platforms,
        "content_policy": channel.content_policy,
        "automation_class": channel.automation_class,
        "artifact_requirements": channel.artifact_requirements,
        "handoff_executor": channel.handoff_executor,
        "submission_route": channel.submission_route,
        "verification": {
            "status": channel.verification_status,
            "checked": CHECKED,
            "note": channel.verification_note
        },
        "auth": auth.map(|a| json!({
            "kind": a.kind,
            "note": a.note,
            "probe": a.probe
        }))
    })
}

// ---------- channel auth (group 10, D-127) ----------
//
// The real credential kind for each channel, recorded during the
// same live verification (Stage 1 set) and the D-128 channel
// sweep (annex set). Portal-only channels get `portal_login`
// and an honest note — no token flow is invented for them. The
// probe text describes the check the *holder of the value* (the
// estate store side: primo `pass` or the Secure Vault flow) can
// run; Ardor itself never receives the value, it records the
// attestation result.

/// How a channel really authenticates its publishers.
pub struct ChannelAuth {
    pub channel: &'static str,
    /// api_key | oauth_token | portal_login | butler_key | signing_key
    pub kind: &'static str,
    pub note: &'static str,
    /// The verification probe available for this channel, or the
    /// honest statement that none exists.
    pub probe: &'static str,
}

/// Auth records, alphabetical by channel id (same order as
/// [`CHANNELS`]).
pub const CHANNEL_AUTH: &[ChannelAuth] = &[
    ChannelAuth {
        channel: "affect3d",
        kind: "portal_login",
        note: "Curated store; the vendor/partner account login is established during the pitch. No developer API or token flow exists to record.",
        probe: "none — portal-only channel; verification is an operator attestation that the partner account login works.",
    },
    ChannelAuth {
        channel: "altstore_pal",
        kind: "portal_login",
        note: "Distribution runs under the studio's Apple Developer account login plus a developer-hosted source the studio operates; AltStore itself issues no publishing credential. Commerce is bring-your-own (pair a storefront_rail record).",
        probe: "none — verification is an operator attestation that the Apple Developer login works and the hosted source serves the app.",
    },
    ChannelAuth {
        channel: "casting_call_club",
        kind: "portal_login",
        note: "Platform account login for posting and answering casting calls. No token flow exists to record.",
        probe: "none — portal-only channel; verification is an operator attestation that the platform account login works.",
    },
    ChannelAuth {
        channel: "ccbill",
        kind: "portal_login",
        note: "Merchant account login issued through application/underwriting. The record is a reference only; merchant credentials live in the estate store like every other channel's.",
        probe: "none — verification is an operator attestation that the merchant account login works.",
    },
    ChannelAuth {
        channel: "cgtrader",
        kind: "portal_login",
        note: "Seller account login. No publishing API key verified.",
        probe: "none — portal-only channel; verification is an operator attestation that the seller account login works.",
    },
    ChannelAuth {
        channel: "commishes",
        kind: "portal_login",
        note: "Platform account login. Payments run PayPal-direct between the parties, so the payment identity is the parties' own — no platform credential beyond the account login exists to record.",
        probe: "none — portal-only channel; verification is an operator attestation that the platform account login works.",
    },
    ChannelAuth {
        channel: "denpasoft",
        kind: "portal_login",
        note: "Publisher-managed storefront; partner access is a store/publisher account login. No developer API or token flow exists to record.",
        probe: "none — portal-only channel; verification is an operator attestation that the partner account login works.",
    },
    ChannelAuth {
        channel: "dlsite",
        kind: "portal_login",
        note: "Circle (creator) account login on DLsite; international creators are served via the DLsite English operation. No public developer API key for publishing.",
        probe: "none — portal-only channel; verification is an operator attestation that the circle account login works.",
    },
    ChannelAuth {
        channel: "epoch",
        kind: "portal_login",
        note: "Merchant account login issued through application/underwriting. The record is a reference only; merchant credentials live in the estate store.",
        probe: "none — verification is an operator attestation that the merchant account login works.",
    },
    ChannelAuth {
        channel: "erolabs",
        kind: "portal_login",
        note: "Partner/publisher platform access by account login through the EroLabs partner flow. No public developer API verified.",
        probe: "none — portal-only channel; verification is an operator attestation that the partner account login works.",
    },
    ChannelAuth {
        channel: "etsy",
        kind: "portal_login",
        note: "Shop account login. No developer API or token flow for publishing exists to record.",
        probe: "none — portal-only channel; verification is an operator attestation that the shop account login works.",
    },
    ChannelAuth {
        channel: "fab",
        kind: "portal_login",
        note: "Epic seller account login (Epic Developer portal). No separate Fab token flow for publishing exists to record.",
        probe: "none — portal-only channel; verification is an operator attestation that the Epic seller account login works.",
    },
    ChannelAuth {
        channel: "fakku",
        kind: "portal_login",
        note: "Publishing partnership; partner/store account login. No developer API or token flow exists to record.",
        probe: "none — portal-only channel; verification is an operator attestation that the partner account login works.",
    },
    ChannelAuth {
        channel: "fansly",
        kind: "portal_login",
        note: "Creator account login. No developer API or token flow verified.",
        probe: "none — portal-only channel; verification is an operator attestation that the creator account login works.",
    },
    ChannelAuth {
        channel: "fdroid_self_hosted",
        kind: "signing_key",
        note: "Estate-operated repository: the credential is the repository signing key plus server access, held in the estate store; slipway executes. Recorded as a reference only, like every other channel.",
        probe: "attestation by the executor: a test repository index signature verifies against the recorded key fingerprint (the fingerprint is a reference, not key material).",
    },
    ChannelAuth {
        channel: "flipped_normals",
        kind: "portal_login",
        note: "Store account login (open-a-store route). No developer API or token flow verified.",
        probe: "none — portal-only channel; verification is an operator attestation that the store account login works.",
    },
    ChannelAuth {
        channel: "gamcore",
        kind: "portal_login",
        note: "Portal-mediated submission; developer contact/account with the portal. No developer API verified.",
        probe: "none — portal-only channel; verification is an operator attestation that the portal account/contact route works.",
    },
    ChannelAuth {
        channel: "gog",
        kind: "portal_login",
        note: "GOG partner/developer account, created through the submission and curation process; builds move through the partner pipeline under that login. No public API key for publishing.",
        probe: "none — portal-only channel; verification is an operator attestation that the partner account login works.",
    },
    ChannelAuth {
        channel: "indie_gala",
        kind: "portal_login",
        note: "Publisher/partner account established through the curated partnership route. No self-serve developer API verified.",
        probe: "none — portal-only channel; verification is an operator attestation that the partner account login works.",
    },
    ChannelAuth {
        channel: "itch_io",
        kind: "butler_key",
        note: "itch.io API key from the account's API-key settings; the same key authenticates butler (butler login / BUTLER_API_KEY) and the itch.io API. This is the registry's model case for a real key-based flow.",
        probe: "key validation: GET https://itch.io/api/1/<key>/me returns the owning account (200) for a live key. Executed by the holder of the value (estate store side); Ardor records the attested result.",
    },
    ChannelAuth {
        channel: "iwantclips",
        kind: "portal_login",
        note: "Creator storefront account login. No developer API or token flow verified.",
        probe: "none — portal-only channel; verification is an operator attestation that the creator account login works.",
    },
    ChannelAuth {
        channel: "jast_usa",
        kind: "portal_login",
        note: "Publishing partnership; partner/store account login. No developer API or token flow exists to record.",
        probe: "none — portal-only channel; verification is an operator attestation that the partner account login works.",
    },
    ChannelAuth {
        channel: "johren",
        kind: "portal_login",
        note: "Platform partner access by account login; titles are onboarded through publisher partners. No public developer API verified.",
        probe: "none — portal-only channel; verification is an operator attestation that the partner account login works.",
    },
    ChannelAuth {
        channel: "kagura_games",
        kind: "portal_login",
        note: "Publishing partnership; partner/store account login. No developer API or token flow exists to record.",
        probe: "none — portal-only channel; verification is an operator attestation that the partner account login works.",
    },
    ChannelAuth {
        channel: "mangagamer",
        kind: "portal_login",
        note: "Licensing partnership; partner account login. No developer API or token flow exists to record.",
        probe: "none — portal-only channel; verification is an operator attestation that the partner account login works.",
    },
    ChannelAuth {
        channel: "manyvids",
        kind: "portal_login",
        note: "Creator account login. No developer API or token flow verified.",
        probe: "none — portal-only channel; verification is an operator attestation that the creator account login works.",
    },
    ChannelAuth {
        channel: "newgrounds",
        kind: "portal_login",
        note: "Creator account login for uploads. (Newgrounds also issues per-app API credentials for in-game services such as medals/scoreboards; those are game-service credentials, not publishing auth, and are recorded the same reference-only way if used.)",
        probe: "none for publishing — verification is an operator attestation that the creator account login works.",
    },
    ChannelAuth {
        channel: "nizima",
        kind: "portal_login",
        note: "The nizima account login (Live2D) operates both this asset-marketplace record and the nizima_requests service record — one account identity spans both. On-platform payout posture for US sellers is UNVERIFIED (see the channel record).",
        probe: "none — portal-only channel; verification is an operator attestation that the nizima account login works.",
    },
    ChannelAuth {
        channel: "nizima_requests",
        kind: "portal_login",
        note: "The same nizima account identity as the nizima asset record — one login spans both records; register the account against either channel id with the same identity.",
        probe: "none — portal-only channel; verification is an operator attestation that the nizima account login works.",
    },
    ChannelAuth {
        channel: "nutaku",
        kind: "portal_login",
        note: "Developer/publisher portal account login; the portal issues per-game integration credentials (game keys for the platform SDK) during onboarding — recorded per game as references when they exist. No public self-serve API key flow for publishing itself.",
        probe: "none — portal-only channel; verification is an operator attestation that the developer portal login works.",
    },
    ChannelAuth {
        channel: "onlyfans",
        kind: "portal_login",
        note: "Creator account login. No developer API or token flow verified.",
        probe: "none — portal-only channel; verification is an operator attestation that the creator account login works.",
    },
    ChannelAuth {
        channel: "patreon",
        kind: "oauth_token",
        note: "Patreon API v2: the creator portal issues a Creator Access Token (OAuth2 bearer) for the creator's own account, with registered-client OAuth for anything broader. Tokens are values: only the reference lives here.",
        probe: "token validation: GET https://www.patreon.com/api/oauth2/v2/identity with the bearer token returns the creator identity for a live token. Executed by the holder of the value; Ardor records the attested result.",
    },
    ChannelAuth {
        channel: "renderhub",
        kind: "portal_login",
        note: "Vendor account login. No developer API or token flow verified.",
        probe: "none — portal-only channel; verification is an operator attestation that the vendor account login works.",
    },
    ChannelAuth {
        channel: "renderosity",
        kind: "portal_login",
        note: "Vendor account login (submissions pass a review gate). No developer API or token flow verified.",
        probe: "none — portal-only channel; verification is an operator attestation that the vendor account login works.",
    },
    ChannelAuth {
        channel: "renderotica",
        kind: "portal_login",
        note: "Vendor account login. No developer API or token flow verified.",
        probe: "none — portal-only channel; verification is an operator attestation that the vendor account login works.",
    },
    ChannelAuth {
        channel: "segpay",
        kind: "portal_login",
        note: "Merchant account login issued through application/underwriting. The record is a reference only; merchant credentials live in the estate store.",
        probe: "none — verification is an operator attestation that the merchant account login works.",
    },
    ChannelAuth {
        channel: "shady_corner",
        kind: "portal_login",
        note: "Publisher relationship; partner contact / store account login. No developer API or token flow exists to record.",
        probe: "none — portal-only channel; verification is an operator attestation that the partner account login works.",
    },
    ChannelAuth {
        channel: "sidequest",
        kind: "portal_login",
        note: "Developer account login for submissions. No publishing API key verified. (The channel's own content policy is itself unverified — see the channel record.)",
        probe: "none — portal-only channel; verification is an operator attestation that the developer account login works.",
    },
    ChannelAuth {
        channel: "skima",
        kind: "portal_login",
        note: "Platform account login. No developer API or token flow verified.",
        probe: "none — portal-only channel; verification is an operator attestation that the platform account login works.",
    },
    ChannelAuth {
        channel: "subscribestar",
        kind: "portal_login",
        note: "Creator (Star) account login. No public developer API or token flow verified.",
        probe: "none — portal-only channel; verification is an operator attestation that the creator account login works.",
    },
    ChannelAuth {
        channel: "superhive",
        kind: "portal_login",
        note: "Creator (storefront) account login. No developer API or token flow verified.",
        probe: "none — portal-only channel; verification is an operator attestation that the creator account login works.",
    },
    ChannelAuth {
        channel: "tinyhat",
        kind: "portal_login",
        note: "Publishing label with no own storefront: the relationship is a publishing deal, so this record references the partner contact/login established at signing. No developer API or token flow exists to record.",
        probe: "none — verification is an operator attestation that the partner contact route works.",
    },
    ChannelAuth {
        channel: "unity_asset_store",
        kind: "portal_login",
        note: "Publisher account login (with payout profile). No separate token flow for publishing exists to record.",
        probe: "none — portal-only channel; verification is an operator attestation that the publisher account login works.",
    },
    ChannelAuth {
        channel: "verotel",
        kind: "portal_login",
        note: "Merchant account login issued through application/underwriting. The record is a reference only; merchant credentials live in the estate store.",
        probe: "none — verification is an operator attestation that the merchant account login works.",
    },
    ChannelAuth {
        channel: "vgen",
        kind: "portal_login",
        note: "Artist account login; payouts route via Stripe/PayPal under the account, but commission operations authenticate by the account login. No token flow exists to record.",
        probe: "none — portal-only channel; verification is an operator attestation that the artist account login works.",
    },
    ChannelAuth {
        channel: "voice_acting_club",
        kind: "portal_login",
        note: "Community infrastructure (Discord): the 'account' is the studio's community identity used to post casting calls — there is no marketplace account, escrow or token flow, and none is invented here.",
        probe: "none — verification is an operator attestation that the community identity can post in #casting-calls-18-up.",
    },
    ChannelAuth {
        channel: "zoom_platform",
        kind: "portal_login",
        note: "Publisher partnership account login. Channel is flagged (adult acceptance unverified), so account registration is refused until the flag clears.",
        probe: "none — portal-only channel; and the channel itself is flagged.",
    },
];

/// The auth record for a channel.
pub fn auth_for(channel_id: &str) -> Option<&'static ChannelAuth> {
    CHANNEL_AUTH.iter().find(|a| a.channel == channel_id)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ids() -> Vec<&'static str> {
        CHANNELS.iter().map(|c| c.id).collect()
    }

    fn in_section(section: &str) -> Vec<&'static str> {
        CHANNELS
            .iter()
            .filter(|c| c.section == section)
            .map(|c| c.id)
            .collect()
    }

    #[test]
    fn registry_covers_the_d126_target_set() {
        // The D-126 set as amended: the pixiv family (booth /
        // fanbox / pixiv) is NOT expected here any more — it was
        // struck by D-129 (see the struck-state test below).
        let ids = ids();
        for expected in [
            "denpasoft",
            "dlsite",
            "erolabs",
            "fakku",
            "fdroid_self_hosted",
            "gamcore",
            "gog",
            "itch_io",
            "jast_usa",
            "johren",
            "kagura_games",
            "mangagamer",
            "newgrounds",
            "nutaku",
            "patreon",
            "subscribestar",
            "zoom_platform",
        ] {
            assert!(ids.contains(&expected), "missing channel {expected}");
        }
        // Steam stays out (D-095); Play is SFW-lane only.
        assert!(!ids.contains(&"steam"));
        assert!(!ids.contains(&"google_play"));
    }

    #[test]
    fn annex_additions_are_all_present() {
        // Every addition in the BRIEF's group-9 annex (D-128),
        // minus the items D-129 struck from it (BOOTH assets and
        // the pixiv Requests cross-listing never became records).
        let ids = ids();
        for expected in [
            // games/apps additions
            "indie_gala",
            "shady_corner",
            "tinyhat",
            "altstore_pal",
            "sidequest",
            "onlyfans",
            // own-storefront rail
            "ccbill",
            "segpay",
            "epoch",
            "verotel",
            // adult rig/asset marketplaces
            "renderotica",
            "renderosity",
            "cgtrader",
            "flipped_normals",
            "superhive",
            "affect3d",
            // SFW-only asset marketplaces
            "fab",
            "unity_asset_store",
            "renderhub",
            "nizima",
            "etsy",
            // services / contracting
            "vgen",
            "commishes",
            "voice_acting_club",
            "casting_call_club",
            "iwantclips",
            "manyvids",
            "fansly",
            // SFW-only services
            "skima",
            "nizima_requests",
        ] {
            assert!(ids.contains(&expected), "missing annex channel {expected}");
        }
        assert_eq!(ids.len(), 47, "17 original + 30 annex records");
    }

    #[test]
    fn flagged_channels_are_exactly_the_failed_verifications() {
        // Only Zoom Platform remains flagged: operating, but its
        // acceptance of explicit adults-only titles could not be
        // verified. The pixiv family is not here — it was struck
        // (D-129), not flagged, and has no records at all.
        let flagged: Vec<&str> = CHANNELS
            .iter()
            .filter(|c| c.verification_status == "flagged")
            .map(|c| c.id)
            .collect();
        assert_eq!(flagged, vec!["zoom_platform"]);
    }

    #[test]
    fn struck_channels_have_no_records() {
        // MiKandi (D-126 addendum 2, defunct) and the pixiv
        // family — pixiv, BOOTH, FANBOX (D-129, Owner ruling
        // reversing D-126 addendum 3) — are struck the same way:
        // no channel record, no credential record; the strikes
        // live in the build reports, not the registry.
        for struck in ["mikandi", "pixiv", "booth", "fanbox"] {
            assert!(
                find_channel(struck).is_none(),
                "{struck} must have no channel record"
            );
            assert!(
                auth_for(struck).is_none(),
                "{struck} must have no auth record"
            );
        }
    }

    #[test]
    fn sections_partition_the_registry() {
        for channel in CHANNELS {
            assert!(
                SECTIONS.contains(&channel.section),
                "{} has unknown section {}",
                channel.id,
                channel.section
            );
            for other in channel.cross_listed_in {
                assert!(
                    SECTIONS.contains(other),
                    "{} cross-lists into unknown section {other}",
                    channel.id
                );
            }
        }
        assert_eq!(in_section("distribution").len(), 23);
        assert_eq!(in_section("storefront_rail").len(), 4);
        assert_eq!(in_section("assets").len(), 6);
        assert_eq!(in_section("assets_sfw").len(), 5);
        assert_eq!(in_section("services").len(), 7);
        assert_eq!(in_section("services_sfw").len(), 2);
        // The rail is its own category of records, all processors.
        for id in in_section("storefront_rail") {
            assert_eq!(find_channel(id).unwrap().category, "payment_rail");
        }
        // The SFW-only sections hold exactly the annex's SFW sets.
        assert_eq!(
            in_section("assets_sfw"),
            vec!["etsy", "fab", "nizima", "renderhub", "unity_asset_store"]
        );
        assert_eq!(in_section("services_sfw"), vec!["nizima_requests", "skima"]);
        // The services section holds exactly the annex's adult
        // service channels (OnlyFans reaches it by cross-listing).
        assert_eq!(
            in_section("services"),
            vec![
                "casting_call_club",
                "commishes",
                "fansly",
                "iwantclips",
                "manyvids",
                "vgen",
                "voice_acting_club"
            ]
        );
    }

    #[test]
    fn sfw_only_records_say_so() {
        // A record in an SFW-only section must declare itself
        // SFW-only in its own policy text — the separation is
        // carried by the records, not just by the section label.
        for channel in CHANNELS.iter().filter(|c| c.section.ends_with("_sfw")) {
            assert!(
                channel.content_policy.contains("SFW-only"),
                "{} must declare SFW-only in its policy text",
                channel.id
            );
            assert!(
                channel.verification_note.contains("SFW-only"),
                "{} must repeat SFW-only in its verification note",
                channel.id
            );
        }
        // And no adult-section record borrows the label.
        for channel in CHANNELS.iter().filter(|c| !c.section.ends_with("_sfw")) {
            assert!(
                !channel.content_policy.starts_with("SFW-only"),
                "{} is not an SFW channel",
                channel.id
            );
        }
    }

    #[test]
    fn onlyfans_is_cross_listed_into_services() {
        // One record, in distribution (it is a membership
        // channel), cross-listed into services for its
        // custom-order flow. It is the only cross-listed record.
        let onlyfans = find_channel("onlyfans").unwrap();
        assert_eq!(onlyfans.section, "distribution");
        assert_eq!(onlyfans.category, "membership");
        assert_eq!(onlyfans.cross_listed_in, &["services"]);
        let cross_listed: Vec<&str> = CHANNELS
            .iter()
            .filter(|c| !c.cross_listed_in.is_empty())
            .map(|c| c.id)
            .collect();
        assert_eq!(cross_listed, vec!["onlyfans"]);
    }

    #[test]
    fn extension_notes_are_recorded() {
        // itch.io asset packs (annex A-6): the existing record
        // carries the extension, incl. the AI-disclosure rule for
        // asset packs.
        let itch = find_channel("itch_io").unwrap();
        assert!(itch.content_policy.contains("Asset packs"));
        assert!(
            itch.artifact_requirements
                .join(" ")
                .contains("asset pack archives")
        );
        // DLsite materials (annex): extension of the existing
        // entry, not a new target.
        let dlsite = find_channel("dlsite").unwrap();
        assert!(dlsite.content_policy.contains("materials/assets"));
        assert!(find_channel("dlsite_materials").is_none());
    }

    #[test]
    fn annex_caveats_are_carried_in_the_records() {
        // SideQuest: the official content-policy page was not
        // fetched — the caveat is in the record, per the annex.
        let sidequest = find_channel("sidequest").unwrap();
        assert!(sidequest.content_policy.contains("NOT fetched"));
        assert!(sidequest.verification_note.contains("not fetched"));
        // Superhive: the policy tension is recorded, not smoothed.
        let superhive = find_channel("superhive").unwrap();
        assert!(superhive.content_policy.contains("POLICY TENSION"));
        assert!(superhive.content_policy.contains("NOT evidenced"));
        // Casting Call Club: practice-verified only.
        let ccc = find_channel("casting_call_club").unwrap();
        assert!(ccc.content_policy.contains("practice-verified only"));
        // Commishes: no escrow, fee unverified.
        let commishes = find_channel("commishes").unwrap();
        assert!(commishes.content_policy.contains("UNVERIFIED"));
        // Secondary-grade figures are marked as such wherever
        // the records carry them.
        for id in ["ccbill", "epoch", "segpay", "verotel", "fansly", "onlyfans"] {
            let channel = find_channel(id).unwrap();
            assert!(
                channel.content_policy.contains("[secondary-grade]"),
                "{id} must mark its secondary-grade figures"
            );
        }
    }

    #[test]
    fn every_channel_has_an_auth_record() {
        for channel in CHANNELS {
            let auth =
                auth_for(channel.id).unwrap_or_else(|| panic!("no auth record for {}", channel.id));
            assert!(!auth.kind.is_empty());
            assert!(!auth.probe.is_empty());
        }
        assert_eq!(CHANNEL_AUTH.len(), CHANNELS.len());
        // The two real token/key flows in the registry.
        assert_eq!(auth_for("itch_io").unwrap().kind, "butler_key");
        assert_eq!(auth_for("patreon").unwrap().kind, "oauth_token");
        // Every annex channel authenticates by honest account
        // reference — no invented token flows.
        for id in [
            "vgen",
            "ccbill",
            "renderotica",
            "fab",
            "skima",
            "voice_acting_club",
        ] {
            assert_eq!(auth_for(id).unwrap().kind, "portal_login", "{id}");
        }
    }

    #[test]
    fn executors_follow_the_brief() {
        assert_eq!(find_channel("itch_io").unwrap().handoff_executor, "butler");
        assert_eq!(find_channel("nutaku").unwrap().handoff_executor, "slipway");
        assert_eq!(
            find_channel("fdroid_self_hosted").unwrap().handoff_executor,
            "slipway"
        );
        // Membership channels are a distinct category, not storefronts.
        for id in ["patreon", "subscribestar", "onlyfans"] {
            assert_eq!(find_channel(id).unwrap().category, "membership");
        }
        // The own-storefront rail is executed by hand (the studio
        // builds its own storefront on the rail).
        for id in in_section("storefront_rail") {
            assert_eq!(find_channel(id).unwrap().handoff_executor, "manual");
        }
    }
}
