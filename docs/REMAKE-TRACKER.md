# REMAKE TRACKER — D-1563 NSFW games rebuild programme

> BLOCH: origin_lane=ntranch, content_label=nsfw.
> Compiled 2026-11-20 from two sources only:
> `~/workspace/ntr-games-rebuild-20261116/CENSUS.md` (compiled
> 2026-11-16 with appended deltas through 2026-11-20) and
> `~/workspace/ntr-games-rebuild-20261116/PIPELINE.md` (D-1563,
> with amendments through 2026-11-18). Every state line below
> traces to one of those two documents. Where they are thin or
> mark something UNKNOWN, this tracker says so in those words.
> Where they contradict each other, the contradiction is listed
> in the closing section, not resolved here. This draft is
> written for the ardor repo's docs as the programme's remake
> tracker; it is not itself a ruling on any title.

## 1. Purpose — as the Owner stated it, 2026-11-20

The programme takes **every NSFW title in the census** from
wherever its records actually stand to a **finished clean-room
rebuild**: native Linux + native Android, Rust, English-only
shipped text, same features / same play as the title's oracle
(PIPELINE §3 defines "same" per population).

Execution is by **local models** — rose-served, driven through
phlow — using the **estate MCP fleet** as the tool layer (SFW
and NSFW MCPs alike, as each title's workflow requires). The
Owner's stated intent is dual: the rebuilds get done, **and**
the programme is run explicitly as a **proving ground for
local-model and MCP validation**. Per the standing sequencing
ruling (Owner, 2026-11-18), once the tooling the rebuilds stand
on is done, the clean-room builds proceed **in census wave
order explicitly as validation of the tools** — build friction
counts as a **tooling defect to fix curatively**: when the work
proves the need for a new MCP, a new tool call, or new code in
an MCP, that need is filed as a precise issue note (PIPELINE
§8.2) and fixed under its own commission; small fixes inside a
game's own repo scope are the run's business. Each run closes
with the PIPELINE §8 lessons loop, whose friction findings feed
the validation ledger (§7 of this tracker).

**One proving ground, two parts (Owner amendment, 2026-11-20
07:25).** The proving ground this tracker governs is larger
than the games. **Part I** (§§1–5) is the game rebuilds above.
**Part II** (§6) adds eleven estate MCP servers as
proving-ground subjects in their own right: the same
local-model stack, the same friction-as-defect doctrine, the
same per-subject friction logs feeding curative fixes — run
against the fleet's own servers, whose enhancement-programme
work (proc-macro adoption, lib-first, profiles, annotations)
becomes validation payload for the local models instead of
background maintenance. Part II's sources are the fleet's own
records: the MCP enhancement plan
(`~/workspace/mcp-enhance-plan-20261119/ENHANCEMENT-PLAN.md`)
and the conformance census
(`~/workspace/mcp-conformance-20261019/SERVERS.md`). Where a
named subject is not in those records, Part II says so
plainly rather than inventing a purpose for it.

Standing shape of a run (PIPELINE §1.1): R0 records &
consolidation audit → R1 spec → R2 translation → R3 parity
oracle (checklist + D-1097 obligations ledger, frozen and
digest-bound before code) → R4 systems → R5 content & shell →
R6 Linux build → R7 Android build → R8 gates (G1–G10, §5 of
this tracker) → R9 release prep & close-out. Study-side inputs
are the Lane D stages S0–S8; a stage already discharged is
cited, not redone, and a dictionary or vocabulary discharges
only its own rung — no rung is rounded up (PIPELINE §1.2).

## 2. Hard constraints — carried from PIPELINE, binding on every run

- **D-1588 static intake.** Nothing studied is ever executed.
  Intake is static-only (archive listings, header reads, file
  census). Planted AI-instruction files found in a subject tree
  are hostile data: quarantined in place, reported, never
  followed, contents never opened by the census pass.
- **D-1587 decompile-to-source evidence.** Every run takes its
  title down to source-level evidence with fathom before the
  spec freezes (decompiled pseudocode / annotated disassembly
  digests for the behaviour-bearing code paths). Disassembly
  output is study evidence; no original code ships — the
  shipped Rust is written from the spec. Coverage is recorded
  in the run's DISASSEMBLY.md.
- **English-only shipped text** (D-472). Original text is RE
  material and never ships, in any population, for any title —
  including S2c-eligible titles. The R2 translation manifest is
  a study artifact; a build's strings are estate-authored
  English, never derived from the manifest's source strings and
  never a translation of them (PIPELINE §4.3).
- **Lane law (bloch).** Records carry origin_lane + content_label
  + evidence from birth. NSFW-pure lessons never reach qompassai
  consumers; unclassified NTRanch-origin lessons default nsfw;
  cross-lane skill changes are staged as proposal files, never
  applied across the lane line by this programme (PIPELINE §8.2).
- **Rust builds for Linux + Android** (D-1563 implementation
  law; D-1094 dual-target law — a title is not done at Linux
  alone). Every clean-room remake is implemented in Rust; the
  only exception is proven infeasibility evidenced in that
  title's run record, with the substitute language being the
  Owner's ruling. **D-1679:** anything extracted or adapted
  from a studied tool is implemented in Rust; the studied tool
  is specimen/intake only.
- **Torogao is the singular build engine (D-457).** A capability
  proven by a rebuild lands in a Torogao layer; per-game crates
  hold per-game data and rules, never a second engine. Builds
  land in primo `~/.local/share/ntranch/remakes/`, hash-verified.
- **S2c law, as amended (Owner, 2026-11-17).** The eligibility
  verification standard is the title's identity on adult/NSFW
  game sites: every title in the RE tree is adult-coded and
  verified as adult by that identity, and a title excluded
  solely for an unestablished cast is an eligible content
  source on the same terms as Aria Succubism and CIRCLEMATE.
  Eligibility gates what may be *studied* as content, never
  what may ship: shipped text and assets remain estate-authored
  for every title, and clean-room discipline is untouched.
- **Per-push boundary.** Nothing in a run authorizes a push;
  commits and pushes follow the standing per-push confirmation
  rule. **No Docker / Podman** anywhere in the build path.
  Primo heavy-compute policy binds every run (one heavy job at
  a time; build scripts trap-release the slot).
- **Numbers discipline (D-713).** Every number in a spec, a
  checklist, a gate report, or a close-out cites its run
  artifact; an unverifiable number routes to review.

### 2.1 Consolidated restrictions list — title-level hard stops and scope limits

Each item below also appears in the affected title's section.

1. **Teaching Feeling — sexual content CLOSED permanently.**
   Sylvie is depicted in the work itself as a child; a
   listing-level or nominal adult designation does not change
   the depiction. Scene/dialogue extraction, content map,
   translation inventory of sexual content, and parity
   re-authoring of sexual scenes are closed permanently — no
   lane, local model, or Pax takeover may perform them, and no
   Owner ruling reopens this (PIPELINE amendment 2026-11-18
   01:05). RULED (Owner, 2026-11-18 01:03): the title stays in
   the programme at **systems-only scope permanently**. (The
   earlier 2026-11-17 lift of the D-1583 safeguard is superseded
   for this title's sexual content by the carve-out.)
2. **Elise and the Spellbinding Marionette — systems-only until
   depicted-cast verification passes.** Depicted-cast ages are
   UNVERIFIED (SPEC ELI-U-1); the title stands at systems-only
   scope until Pax-gated depicted-cast verification at the
   MV/MZ rung, and permanently if that check fails.
3. **Hypnotic-Family Planning S — depicted-cast verification is
   a named unknown (hard).** The listing's tag set includes
   "Loli". Depicted-cast verification is an explicit named
   unknown for the planning/intake lane (Elise precedent): if
   verification fails, this title is systems-only permanently.
   The unknown governs scope regardless of S2c eligibility.
4. **Lewd Avatar Simulator — systems-only until cast
   verification passes.** D-1588 audit found 24
   planted-instruction files (7 distinct hashes), quarantined
   in place, nothing executed. Depicted-cast clearance and
   binary cheat screening are carried as named UNKNOWNs
   (Pax disposition 2026-11-19).
5. **The Censor — two NPC slots carry no stated age** (recorded
   UNKNOWN; U-10 CLOSED by ruling 2026-11-19: StudentsCouple +
   ChurchSmallNun are rebuilt as estate-authored adults). The
   oracle is the cheat-patched v3.1.5 build; its three touched
   sites are carried as UNKNOWN (U-1/U-2).
6. **StopKing — content-level work gated behind REVIEW-grade
   identity** until U-4 (listing identity, Pax/Owner-side)
   closes. The data layer is buildable; U-4..U-8 carried open
   at the R1 freeze.
7. **Hachina — behaviour-layer build gated behind U-4/U-5/U-6**
   (D-1587 decompile over the manager classes). The proven
   data layer is buildable. Seven UNKNOWNs carried as named
   opens at the R1 freeze (U-4, U-5, U-6, U-8, U-9, U-10, U-12;
   U-12 is an Owner ruling, incl. HAC-D1).
8. **Milk Factory — roster reduced to 1–4 of 18 by Owner
   ruling; expansion data-only.** A run does not expand content
   scope on its own motion.
9. **S5 content passes are Owner-gated everywhere**
   (PIPELINE §7 gap 9). A run's brief names its content scope
   as the Owner ruled it; expansion is a new ruling.
10. **Aria Succubism and Hypno_Rhythm — S4/R4 gated on the
    audio-sync spike's verdict.** The spike has never run (it
    failed to run on primo saturation, not on merit); a wave
    brief for either title names the spike as its first task.
11. **Kamimachi Site — SOURCE-BLOCKED (2026-11-20).** Every
    known download link is dead; no archive, listing, hash, or
    engine identification exists. A fresh source (a live mirror
    or the Owner's own copy) unblocks this title.
12. **hypno_booth — Owner's rebuild-spec transformations are
    binding on the spec freeze (D-1656):** (1) the subject
    character is an adult woman, not the original's man;
    (2) the conditioning theme is lactation / hucow /
    corruption conditioning, replacing the original's
    feminization/sissy theme. Study of the original is at the
    systems level; the rebuild's content is estate-authored
    under the new theme.
13. **Vesper Isles — the source README itself asks that the
    game not be re-uploaded and its data not extracted/reused.**
    The census records that the clean-room law already binds
    the programme to exactly that: systems and structure are
    studied, no data ships.
14. **Community-repack provenance, recorded not cleared:**
    Goblin Burrow is the community "AIO unified edition"
    repack, not DLsite's original packaging; Dohna Dohna is a
    community "eng Cracked Patch 1.02" repack (full game tree),
    and its separate uncensor patch was NOT acquired. Intake
    censuses verify contents against the version.
15. **Worked / translation-prepared copies, recorded:** StopKing
    (BepInEx + XUnity.AutoTranslator + ReiPatcher; its
    Assembly-CSharp of record is a patched assembly, deltas vs
    pristine UNKNOWN), White Knight Discipline (BepInEx install;
    a root save.dat shows play), SiNiSistar 2 (worked, modded
    copy — study only, nothing ships; the Mod/ overlay's
    producer is UNKNOWN), Lewd Avatar Simulator (fan
    translation/cleanup toolkit in-tree; who produced this
    variant is UNKNOWN), Agent Mirai (extracted tree carries
    saves dated 2026-11-16 — the copy has been played).
    Translation caches are study evidence only, never
    shippable text.
16. **Not rebuild items (recorded so no wave re-litigates
    them):** Hands-On (done-by-construction; the reference
    implementation), MoonDeep and Strobe (in-development
    originals, their own programmes), the niflheim repo
    (pointer; the game lives in the qompass programme),
    versemend (**excluded**: qompass/SFW lane despite sitting
    in the NTRanch directory on disk), desperado-midnight
    (census-grade only; not a git repo on disk — repo-ifying
    it is an Owner-side housekeeping item, not done by any run).

## 3. How to read this tracker

- **Populations** (CENSUS §§1–4; PIPELINE §1.2). A —
  originals with estate specs (parity = the title's own spec +
  conformance vectors). B — study targets with S4/S5 lineage
  (parity = systems/play parity with estate-authored content).
  C — study targets at census/spec grade (parity as B; a run
  executes everything from the first undischarged Lane D stage
  forward, then R0–R9).
- **Spec status** means the R1/S3 spec of record: *frozen*
  (Pax gate taken on the Owner's ruling), *draft* (assembled,
  unfrozen), or *none* (no spec exists yet). A vocabulary or
  data dictionary is not a spec; the tracker names it as its
  own rung.
- **Standing MCP set for every run:** rose/phlow is the
  local-model runtime executing the rungs; fathom carries the
  study-side routes (census, extraction, D-1587 disassembly,
  D-1588 audit support); torogao is the build engine and host;
  theodolite is the code-intelligence layer every build lane
  validates through (below); bloch receives the run's lessons
  candidates at R9; ardor is the NSFW authoring surface for
  estate-authored content passes; lumen serves media
  conversion where a title's workflow touches sprites/video.
  Per-title sections list these only where a title exercises
  them in a title-specific way, and name fathom routes exactly
  where the census names them.
- **phaedrus — the research seat (Owner amendment,
  2026-11-20).** When a rebuild task hits an unsolved
  technique, format, or engine question — an unexercised
  extraction route, an undocumented container format, a
  decompile fidelity question — phaedrus researches how to
  solve it (prior art, format documentation, technique
  evidence) and the findings feed the build lane and the
  friction log. Research is a rung input, not a substitute for
  one: a phaedrus finding informs a spec or a tool commission;
  it never discharges a measurement rung by itself. Per-title
  sections name phaedrus wherever research is a named input to
  that title's next rung.
- **theodolite — four named services (Owner amendment,
  2026-11-20).** (1) **LSP**: diagnostics, completions, and
  code actions through the bridge, for the file's language.
  (2) **Formatter routing**: the lane's files are formatted
  through the routed formatter for their type. (3) **DAP
  debugging**: debug-adapter sessions for build-lane code
  (trajectory mismatches, loader refusals, fixture failures).
  (4) **SCIP code indexing**: the indexed code graph the lanes
  navigate by (cross-references, definitions) in large
  workspaces and worktree consolidations. The standing rule:
  code is validated through theodolite's LSP bridge for the
  file's type, and **zero LSP errors before a file is called
  done** — gate G4 is the run-level form of that rule, planted-
  error control included. Where a per-title section lists
  theodolite, it names which of the four duties that title's
  rungs exercise.
- **Local-model role, standing:** local models perform the
  rungs' labour — intake reading support, spec assembly from
  evidence, R2 manifest production (study renderings only),
  R4/R5 implementation under the gate set, and the friction
  log — with Pax gating each stage. Model failure modes
  observed in a rung are validation-ledger material (§6), not
  run folklore.
- **Gates, standing:** G1–G10 (PIPELINE §6) apply to every
  run, plus the Owner's eye/ear verdict pack under D-221 where
  a run's slice is seen or heard; per-title additions are named
  in the title's section. Gate numbers from earlier programmes
  are history; a run cites only its own artifacts.

## 4. Summary table

Wave key: W1–W4 per CENSUS §6 / PIPELINE §1.3. "Unassigned"
means the census records no wave; by §6 such titles rebuild in
QUEUE order as their extraction rungs land.

| Title | Engine | State of record | Spec | Next rung | Wave |
|---|---|---|---|---|---|
| Magic Coin | Unity Mono (original) | Pop B; S4 crate + S5 shell landed; D-1097 ledger exists (pilot) | Spec-grade (MEASURED) | R0 consolidation + fresh full gate run + parity walk (PILOT) | W1 |
| Modification App | Godot 4.7.1 (corrected by GDRE measurement) | Pop B; S4 + S5 landed; 4 REVIEW items open | Spec-grade | R0 consolidation, then R1 re-verification → R9 | W1 |
| Milk Factory | Unity Mono (original) | Pop B; S4 + S5 landed; fixture hash equal Linux + Android host line | Spec-grade | R0 consolidation → R9 (fresh gates) | W1 |
| Time Stop | Unity Mono (original) | Pop B; S4 + S5 landed; first title to PASS the D-1097 behaviour stage (10 discharged / 0 failed / 2 open-with-reason) | Spec-grade | R0 consolidation → R9; levels/music/UI-UX axes stay OPEN-WITH-REASON until the R1-rung payloads are consumed | W1 |
| Where She Asks to Fall | Estate-original design (original is an S2c-excluded no-download prototype) | Pop B mechanics / Pop A design; S5 binding landed (49/49) | Estate design of record | Consolidate S5 state + D-1563 gates | W1 |
| Crescendo | Godot + JS core port (implementation of record) | Pop A; conformance vectors exist; staged brief written 2026-11-16 | Own book/spec (arousal-engine/1) | R0 → R1 reconciliation → R9; first from-scratch rebuild | W2 |
| Hypnosis Card | Unity Mono (original pair, 1.02 ↔ 2) | Pop B at S4 stage 1 only; R1 serialized values were OPEN at census compile (see §8) | Spec repo (measured specification) | Consume R1-rung values as data, then R1 → R9 | W2, after the R1 rung |
| handfast | Native Rust game (implementation of record) | Pop A; full game + playthrough tests + web shell | Own book/spec | Platform completion audit (Linux native + Android binding) | W2 |
| withheld | Rust crate + web shell | Pop A | Own book/spec | Audit then complete | W2 |
| blushline | JS kernel + web (implementation of record) | Pop A; conformance vectors exist | Own book/spec | Rust rebuild vs conformance vectors; family reference shape, first of the jam family | W3 (see contradiction C1) |
| by-heart | Web/JS family shape | Pop A | Own book/spec | Rust rebuild | W3 |
| creamery | Web/JS family shape (.mjs + .zig fragments) | Pop A | Own book/spec | Rust rebuild | W3 |
| exposure | Web/JS family shape | Pop A | Own book/spec | Rust rebuild | W3 |
| idle-hands | Web/JS family shape | Pop A | Own book/spec | Rust rebuild | W3 |
| milkmaid | Web/JS family shape | Pop A | Own book/spec | Rust rebuild | W3 |
| standard-issue | Web/JS family shape (.mjs + .cpp fragments) | Pop A | Own book/spec | Rust rebuild | W3 |
| trancefall | Web/JS family shape (.mjs + .zig fragments) | Pop A | Own book/spec | Rust rebuild | W3 |
| quartermaster | LÖVE/Lua family shape | Pop A | Own book/spec | Rust rebuild | W3 |
| redlight | Unity/C# jam entry + web tooling | Pop A | Own book/spec | Rust rebuild | W3 |
| second-wave | Web build + Unity marker | Pop A | Own book/spec | Rust rebuild | W3 |
| desperado | Godot original (471 files) | Pop A; largest Godot title | Own book/spec | Rust rebuild | W4 |
| roseford | Design-stage original (Cargo.toml + docs) | Pop A variant: build, not rebuild; design fed by the Unholy Maiden study (premise/systems shapes only, D-1123) | Design docs | Its own commission shape (build) | After W2; listed in W4 per PIPELINE §1.3 |
| Aria Succubism | Not stated in the census sections | Pop C; spec + S2c expressions pilot; content-eligible | Spec (S3) | Audio-sync spike first, then S4/R4 → R9 | W4 (QUEUE order) |
| CIRCLEMATE | Not stated in the census sections | Pop C; intake + animation note; content-eligible; expressions note not yet produced | Below spec (intake + animation note) | S2c expressions note as a study commission ahead of the rebuild | W4 (QUEUE order) |
| Hypno_Rhythm | Not stated in the census sections (spec repo exists) | Pop C; spec MEASURED | Spec MEASURED | Audio-sync spike, then S4 → R9 | W4 (QUEUE order) |
| HypnoClicker | Godot (GDRE dynamics spec) | Pop C; full GDRE dynamics spec | Spec | R1 verification → R2 → R3 → R4–R9 | W4 (QUEUE order) |
| Maeve's Academy | Not stated in the census sections | Pop C; state vocabulary, 604 labels | Vocabulary only (discharges only its own rung) | Remaining Lane D stages from the vocabulary forward → R0–R9 | W4 (QUEUE order) |
| Unholy Maiden | RPG Maker family (data JSONs + plugin JS; engine class from the census record) | Pop C; data dictionary discharged; dynamics spec assembled 2026-11-18 | Draft (unfrozen) | D-1587 source-evidence rung (plugin JS + event command lists), then Pax freeze gate | Unassigned (QUEUE family) |
| Teaching Feeling | Not stated in the census (no census entry; record is PIPELINE's) | Systems-only permanently (hard stop §2.1.1) | Not stated in the sources | Systems-only remake is the ceiling; content scope closed | Not stated in the sources |
| Hypnotic Eyes pair | Not stated in the census sections | Pop C; distinct-verdict only | None | Extraction rungs in QUEUE order | W4 (QUEUE order) |
| IHSA | Unity IL2CPP class (per PIPELINE §7: QUEUE #10/#13/#22/#26 class) | Pop C; waits on the IL2CPP/Ghidra-class rung per §4 (see contradiction C2) | None | R1 does not start until the rung position is resolved at wave commissioning | W4 |
| Undercity Doll Shop | Unity IL2CPP class | Pop C; the IL2CPP/native rung (D-1202) is proven on this title per census §7 | None | Re-walk of S-rung assignment before its wave commissions (census §7) | W4 |
| Complicated Natsuki | Unity IL2CPP class | Pop C; serialized values recovered by bundle-native rung 2 (17,535/17,670 exact, D-1222) | None | Re-walk of S-rung assignment before its wave commissions (census §7) | W4 |
| Ren'Py group remainder | Ren'Py | Pop C; dialogue trees exist for 10 of 12 | None per title | Extraction rungs in QUEUE order (fathom census_dialogue_tree family) | W4 (QUEUE order) |
| RPG Maker group remainder | RPG Maker MV/MZ family | Pop C | None per title | Extraction rungs in QUEUE order (MV/MZ rung family) | W4 (QUEUE order) |
| Moo-Moo Manor | Godot (GDRE coverage measured) | Pop C; GDRE coverage measured, spec unwritten | None | Spec drafting from measured coverage → R1 → R9 | W4 (QUEUE order) |
| Monster Girl Assault | GameMaker | Pop C; sizes only; GameMaker decompile route unexercised | None | Extraction rung as a commissioned phase, or the run does not begin | W4 (QUEUE order) |
| HYPERDEEP | Unreal (pak probe) | Pop C; pak probe only; Unreal pak route beyond census unexercised | None | Extraction rung as a commissioned phase, or the run does not begin | W4 (QUEUE order) |
| Detailed Cowgirl Simulator | Unreal IoStore (pak/ucas) | Pop C; intake census landed 2026-11-16 (genuine RJ01419389 build); D-1587/D-1588 rung commissioned 2026-11-16 | None | D-1587/D-1588 rung → spec → R0–R9 | Unassigned |
| Puppet Master | SRPG Studio (probe) | Pop C; probe verdict only | None | Extraction rungs in QUEUE order | W4 (QUEUE order) |
| Hypnosis App | Native (inferred only) | Pop C; inferred only | None | Extraction rungs in QUEUE order | W4 (QUEUE order) |
| SoldGirl Town | Unity Mono class (MonoBehaviour census) | Pop C; MonoBehaviour census was uncommissioned per §4; unblocked by fathom's BQ-2 tools on main per §7 | None | MonoBehaviour census → spec → R0–R9 | W4 (QUEUE order) |
| Mari's Story | Not stated in the census sections | Pop C; intake census next | None | Intake census | W4 (QUEUE order) |
| NTR Idol | Not stated in the census sections (pak census) | Pop C; pak census next | None | Pak census | W4 (QUEUE order) |
| Lewd Avatar Simulator | Unity 6000.4.2f1, IL2CPP | Intake stages 1–2 done at census grade; D-1588 audit done (24 planted files quarantined); intake resumes at stages 3–5 per the 2026-11-19 correction | None frozen | Intake stages 3–5 products → spec freeze (Pax gate) → R0–R9; systems-only until cast verification passes | Unassigned |
| hypno_booth (0.8.0, Linux) | Godot 4.4.1 stable, GDScript (compiled .gdc only) | First census landed (D-1656); PCK unencrypted, 515 files | None | D-1587 rung (GDScript decompile labour via GDRE) + D-1588 audit → spec freeze (with the D-1656 transformations binding) → R0–R9 | Unassigned |
| White Knight Discipline | Unity Mono | First-pass fingerprint only (D-1673); full intake not yet run | None | Full intake (file inventory, assembly map, D-1588 audit, animation-system inventory per D-1674) | Unassigned |
| Vesper Isles | Godot 4.7.2 stable (single-file export, embedded PCK format 4) | Phase-(1) intake COMPLETE; R1 spec drafted from GDRE-recovered source (139/139 scripts); all 9 intake UNKNOWNs closed | Draft (unfrozen) | Pax freeze gate (incl. the SPEC §4 content-scope ruling) → R2 → R3 → R4–R9 | Unassigned |
| SiNiSistar 2 | Unity 2022.3.62f2, IL2CPP | Static intake COMPLETE (D-1680 phase 1); worked/modded copy | None (spec not yet drafted) | D-1587 fathom study-evidence → spec → Pax freeze gate → R0–R9 | Unassigned |
| Hachina | Unity 2020.1.8f1, Mono (both builds) | Full intake done; serialized-values + native-shape rungs run (275,441/275,442 exact class results) | **Frozen 2026-11-18** (7 UNKNOWNs carried as named opens) | R2 manifest → R3 → data-layer R4 (behaviour layer gated behind U-4/U-5/U-6) | Unassigned |
| StopKing | Unity 2021.3.6f1, Mono | Intake COMPLETE through the D-1587 decompile stage | **Frozen 2026-11-18** | R2 manifest → R3 → R4–R9 (content-level work behind U-4) | Unassigned |
| Suspended Sex Simulator vol02 v1.1.1 | Unity 2021.3.17f1, Mono | Census + engine DONE (lane C intake); archive-only copy | Draft (lane workdir, not frozen) | D-1587 disassembly + serialized-values rung → freeze → R0–R9 | Unassigned |
| Elise and the Spellbinding Marionette | RPG Maker MV (NW.js) | Census + engine DONE (lane C intake); archive-only copy | Draft (lane workdir, not frozen) | MV/MZ study rung — depicted-cast verification is its first step; systems-only until it passes | Unassigned |
| Clergy n Cocktail | Unity 2022.3.62f1, Mono | Census + engine DONE (lane C intake); archive-only copy | Draft (lane workdir, not frozen) | D-1587 disassembly + serialized-values rung → freeze → R0–R9 | Unassigned |
| The Censor | Unity 2022.3.0f1, Mono | Intake + D-1588 audit + D-1587 decompile DONE | **Frozen 2026-11-19** | R2 → R3 → R4–R9; opens U-1, U-2, U-3, U-5, U-7(a)/(b), U-8, U-9, U-11, U-12 carried | Unassigned |
| Iris the Blood Servant | Unity 2022.3.62f2, Mono | Registered 2026-11-19; static archive listing only | None | Intake commission (D-1588 audit + D-1587) → spec freeze → R0–R9 | Unassigned |
| Agent Mirai and the Submission Machines | TyranoBuilder (TyranoScript) in an Electron shell | Registered 2026-11-19; static listings only | None | Intake commission (D-1588 audit → static asar census) → spec freeze → R0–R9 | Unassigned |
| Goblin Burrow | Kirikiri (xp3) | Registered 2026-11-19; static archive listing only; community repack | None | Intake commission (D-1588 audit → xp3 static census) → spec freeze → R0–R9 | Unassigned |
| Mindsweeper | Not stated in the registration (engine ID was the planning lane's first job; the appended planning record's rung order includes an NScripter script-evidence rung) | Registered 2026-11-19; static archive listing only | None | Planning lane products → D-1588 audit → spec freeze → R0–R9 | Unassigned |
| Hypnotic-Family Planning S | TyranoBuilder in an Electron/NW.js shell | Registered 2026-11-19; static archive listing only | None | Planning lane → depicted-cast verification (named unknown) → D-1588 audit → spec freeze → R0–R9 | Unassigned |
| Underground Idol X | Adobe AIR / Flash (SWF) | Registered 2026-11-19; static archive listing only | None | Planning lane (fathom SWF coverage vs gaps — a new fathom family) → D-1588 audit → spec freeze → R0–R9 | Unassigned |
| Dohna Dohna | Alicesoft System4 family | Registered 2026-11-19; static archive listing only; community repack | None | Planning lane (System4 AIN/AFA census) → D-1588 audit → spec freeze → R0–R9 | Unassigned |
| UnHolY DisAsTeR | Clickteam Fusion | Registered 2026-11-19; static archive listings only | None | Planning lane (Fusion CCN events; exe-vs-.ccn confirmation) → D-1588 audit → spec freeze → R0–R9 | Unassigned |
| Popolife! | Cocos2d-x | Registered 2026-11-19; static archive listing only | None | Planning lane (Cocos project/JSB index, conditional) → D-1588 audit → spec freeze → R0–R9 | Unassigned |
| Kamimachi Site | UNKNOWN (no archive acquired) | Registered 2026-11-19; **SOURCE-BLOCKED 2026-11-20** | None | Source acquisition — everything else waits | Unassigned |

Titles present in the RE tree that §4 / the Lane D QUEUE do not
carry (census §7) are listed in §5.66; they have no queue
registration, no rung, and no wave in the sources.

## 5. Per-title sections

### 5.1 Magic Coin — Population B, Wave 1 (PILOT)

- **State of record.** Original: Unity Mono, JA, DLsite
  RJ318915. S4 crate in the magic-coin repo @ 4d34fd7
  (torogao-coin, 811 LOC + 264 test LOC, RON dynamics,
  deterministic trajectory hash). S5 landed in the torogao
  worktree coin-s5 @ 4ea0c5e: shell + coin-android + torogao-app
  host binding; APK exists in the worktree's gates/. It is the
  only Population B title besides Time Stop with a D-1097
  obligations ledger, and the S5 pattern's exemplar.
- **Remaining rungs (PIPELINE order).** R0 consolidation audit
  (the standalone repo is one stage behind the worktree state;
  the run record names which tree is the source of truth per
  crate) → R1 re-verification → R2 in study mode → R3 (ledger
  exists; checklist walked in full) → R4/R5 completion and
  hardening of the landed crates, not rewrites → R6 → R7 →
  R8 **fresh** end-to-end gate run (landed gate numbers are
  history, not evidence) → R9.
- **MCP workflow.** fathom: study-side records already
  discharged; no new route named for this title. torogao:
  engine + app host. theodolite: LSP (G4 planted-error gate),
  formatter routing, DAP (trajectory-hash mismatch debugging),
  SCIP (navigation across the repo + worktree consolidation).
  ardor: content pass surface if the run's ruled scope touches
  content. bloch: R9 lessons candidates.
- **Local-model role.** Consolidation labour, spec
  re-verification reading, checklist walk execution, gate-run
  operation, friction log.
- **Gates.** G1–G10 in full, fresh; the parity walk (G5) and
  ledger evaluation (G6) are the pilot's named centre.
- **Unknowns / hard stops.** Magic Coin's clip timing/curves
  are explicitly still open (PIPELINE §7 gap 1: S2b animation
  notes missing; the checklist derives only from notes that
  exist and carries the animation residue as a carried item —
  the rebuild does not invent timing). S5 content scope stands
  as ruled; expansion is Owner-gated.

### 5.2 Modification App — Population B, Wave 1

- **State of record.** Original: Godot 4.7.1 (engine corrected
  by GDRE measurement). S4 crate torogao-modapp in the
  modification-app repo. S5 landed in modapp-s5 @ b04e430
  (20/0 + 1/0), plus the Setup minigame. **4 REVIEW items open.**
- **Remaining rungs.** R0 consolidation → R1 re-verification
  (the 4 REVIEW items are carried, not resolved by fiat) →
  R2 study mode → R3 (ledger to be authored — only Coin and
  Time Stop have one) → R4/R5 completion → R6 → R7 → R8 fresh
  gates → R9.
- **MCP workflow.** fathom: the GDRE route is this title's
  study-side route of record (it produced the engine
  correction). torogao: engine + host. theodolite: LSP,
  formatter routing, SCIP (consolidation navigation).
  bloch: R9 candidates.
- **Local-model role.** Consolidation, REVIEW-item evidence
  work (a REVIEW value is promoted only by naming the
  measuring route and re-grading with its artifact), ledger
  authoring at R3, build completion.
- **Gates.** G1–G10 fresh; G6 cannot pass until the R3 ledger
  exists.
- **Unknowns / hard stops.** The 4 open REVIEW items (the
  census does not enumerate them; they live in the title's
  records). S5 content scope Owner-gated.

### 5.3 Milk Factory — Population B, Wave 1

- **State of record.** Original: Unity Mono, JA. S4 crate
  torogao-milkfactory in the milk-factory repo. S5 landed in
  milkfactory-remake @ a603292 (28/0). Cross-platform fixture
  hash `56a658fbff35ed7f` was equal on the Linux S4 smoke, the
  Linux shell smoke, and the Android host line — the G8
  exemplar of record.
- **Remaining rungs.** R0 consolidation → R1 re-verification →
  R2 study mode → R3 (ledger to be authored) → R4/R5
  completion → R6 → R7 → R8 fresh gates (G8 must re-produce
  fixture equality on this run's own artifacts) → R9.
- **MCP workflow.** fathom: study-side routes of record
  (Unity Mono extraction family). torogao: engine + host.
  theodolite: LSP, formatter routing, DAP (fixture-equality
  failure debugging), SCIP. bloch: R9 candidates.
- **Local-model role.** Consolidation, ledger authoring,
  build completion, gate operation.
- **Gates.** G1–G10 fresh; G8 named explicitly for this title.
- **Unknowns / hard stops.** **Roster reduced to 1–4 of 18 by
  Owner ruling; expansion is data-only** and a run does not
  expand content scope on its own motion (restrictions list
  §2.1.8).

### 5.4 Time Stop — Population B, Wave 1

- **State of record.** Original: Unity Mono, JA. S4 crate
  torogao-timestop in the time-stop repo. S5 landed in
  timestop-s5 @ b3e77e6 (28/0 + 1/0). **First title to PASS
  the D-1097 behaviour stage** (10 discharged / 0 failed /
  2 open-with-reason).
- **Remaining rungs.** R0 consolidation → R1 re-verification →
  R2 study mode → R3 (ledger exists; the 2 open-with-reason
  obligations carry) → R4/R5 completion → R6 → R7 → R8 fresh
  gates → R9.
- **MCP workflow.** fathom: study-side routes of record; the
  serialized-values (AssetRipper-class) rung produced this
  title's payloads per census §7 (see contradiction C3).
  torogao: engine + host. theodolite: LSP, formatter routing,
  DAP, SCIP. bloch: R9 candidates.
- **Local-model role.** Consolidation, open-obligation
  evidence work, build completion, gate operation.
- **Gates.** G1–G10 fresh; G6 walks the existing ledger.
- **Unknowns / hard stops.** The title's levels/music/UI-UX
  axes stay OPEN-WITH-REASON in its ledger until the R1-rung
  values are consumed (PIPELINE §7 gap 5). S5 content scope
  Owner-gated.

### 5.5 Where She Asks to Fall — Population B mechanics / estate-original design, Wave 1

- **State of record.** Estate-original design: the original is
  an S2c-excluded no-download prototype; nothing was
  extractable from it, by law and by fact (Lane D census §5
  row 27, per PIPELINE §1.2). A Rust crate exists in its repo
  (7 .rs); the S5 binding landed in the torogao worktree
  fall-s5-20261109 @ 3be1a32 (49/49). It follows Population B's
  consolidation row in the census §3 table.
- **Remaining rungs.** R0 consolidation (S5 state) → R1 in the
  Population A reduced form (spec recovery/reconciliation of
  the estate design) → R2 as the English audit → R3 (oracle
  from the title's own design + records) → R4–R9 in full with
  D-1563 gates.
- **MCP workflow.** No fathom study route applies (nothing to
  extract). torogao: engine + host. theodolite: LSP, formatter
  routing, SCIP. ardor: estate-authored content surface.
  bloch: R9 candidates.
- **Local-model role.** Consolidation, spec reconciliation,
  build completion, gate operation.
- **Gates.** G1–G10 fresh.
- **Unknowns / hard stops.** None recorded in the sources
  beyond the standing content-scope rule.

### 5.6 Hypnosis Card — Population B at S4 stage 1 only, Wave 2 (after the R1 rung)

- **State of record.** Original pair: Hypnosis Card 1.02 ↔ 2,
  Unity Mono, JA. S4 crate torogao-hypnocard @ b0d1d28 (28/0),
  keyed-by-name effects, living in the torogao lineage with no
  standalone repo. S4 stage 1 only: the R1 serialized values
  were OPEN at census compile (AssetRipper-class rung
  uncommissioned per CENSUS §3/§4 and PIPELINE §7 gap 5;
  census §7 records the rung as landed (D-1219) and producing
  this title's payloads — contradiction C3, unresolved here).
- **Remaining rungs.** Consume the R1-rung values as data only
  (the crate is built so closing R1 is data-only: keyed-by-name
  config makes the original's renumber trap inexpressible) →
  R1 (its values were open at census) → R2 study mode → R3
  (ledger to be authored) → R4/R5 → R6 → R7 → R8 → R9.
- **MCP workflow.** fathom: the serialized-values /
  bundle-native reader family is this title's route of record.
  torogao: engine + host (the crate already lives in its
  lineage). theodolite: LSP, formatter routing, DAP (loader /
  keyed-name data debugging), SCIP. bloch: R9 candidates.
- **Local-model role.** Data intake and validation against
  the keyed-by-name schema, spec completion, build, gates.
- **Gates.** G1–G10; per-title addition: the loader's
  typed-UNKNOWN refusal behaviour (out-of-table index is an
  error, never an extrapolation).
- **Unknowns / hard stops.** R1 values' state is exactly the
  C3 contradiction — the wave brief must re-walk it before
  commissioning (census §7's own instruction for this class).

### 5.7 Crescendo — Population A, Wave 2 (first from-scratch rebuild)

- **State of record.** Implementation of record: Godot
  (project.godot, main.tscn, crescendo_core.gd 551 lines) + a
  JS core port (web/core.mjs 421 lines) + conformance vectors.
  It is a Godot port of the canon arousal-engine/1 spec (the
  engine's Rust spec lives in the torogao lineage; the
  GDScript port exists to test spec completeness). Its staged
  brief (BRIEF-crescendo.md) was written 2026-11-16 beside the
  pipeline, and its PORT-BACK's divergence list (D-1…D-6) is
  the shape spec findings take.
- **Remaining rungs.** R0 records audit → R1 reduced form:
  spec recovery and reconciliation across its three witnesses
  (code, book, vectors), divergences filed as numbered spec
  findings → R2 as the English audit → R3 (oracle from the
  book/spec + vectors; ledger to be authored) → R4–R9 in
  full. The Rust rebuild is an engine-true shell against the
  conformance vectors.
- **MCP workflow.** torogao: the arousal-engine/1 Rust spec's
  home lineage and the build host. theodolite: LSP, formatter
  routing, DAP (vector-trace mismatch debugging), SCIP.
  phaedrus: research seat if a witness divergence turns on an
  unsolved spec-completeness question. bloch: R9 candidates.
- **Local-model role.** Witness reconciliation, divergence
  drafting, Rust implementation against vectors, gate
  operation.
- **Gates.** G1–G10 plus the per-title addition the census
  names by design: the conformance-vector exactness run
  (integer state traces; final records canonicalised for key
  order — the B-1 lesson: dictionary key order is not an
  engine invariant).
- **Unknowns / hard stops.** Divergences that change canon
  numbers need the Owner; findings that merely name what the
  spec left unstated are recorded and carried (PIPELINE §2.2).

### 5.8 handfast — Population A, Wave 2

- **State of record.** Native Rust: 10 .rs, a full game
  (game/levels/save/rng/fixed) + playthrough tests + web
  shell. Branch pax/phorge-plan-20261110.
- **Remaining rungs.** R0 → R1 reduced form → R2 English
  audit → R3 → R4–R9, entered as a **platform completion
  audit** (Linux native + Android binding) rather than a
  from-scratch build.
- **MCP workflow.** torogao host binding for the Android
  line. theodolite: LSP, formatter routing, SCIP. bloch: R9.
- **Local-model role.** Audit execution, Android binding
  completion, gates.
- **Gates.** G1–G10, with G10 (APK build proof) the audit's
  centre.
- **Unknowns / hard stops.** None recorded beyond standing law.

### 5.9 withheld — Population A, Wave 2

- **State of record.** Rust marker (Cargo.toml) + JS tooling,
  50 files; a Rust crate + web shell.
- **Remaining rungs.** R0 → R1 reduced form → R2 audit → R3 →
  R4–R9: audit then complete.
- **MCP workflow.** torogao host. theodolite: LSP, formatter
  routing, SCIP. bloch: R9.
- **Local-model role.** Audit, completion build, gates.
- **Gates.** G1–G10.
- **Unknowns / hard stops.** None recorded beyond standing law.

### 5.10 blushline — Population A, Wave 3 (family first; see contradiction C1)

- **State of record.** JS kernel (core/kernel.mjs,
  web/game.mjs) + conformance vectors. The kernel is the
  JS/Lua/web jam family's reference shape. (Wave conflict:
  CENSUS §1's table says "wave 2"; CENSUS §6 and PIPELINE
  §1.3 put blushline first in wave 3 — contradiction C1.)
- **Remaining rungs.** R0 → R1 reduced form → R2 audit → R3
  (oracle from book/spec + vectors) → R4–R9: Rust rebuild
  against the conformance vectors.
- **MCP workflow.** torogao host. theodolite: LSP, formatter
  routing, DAP (vector-trace debugging), SCIP. bloch: R9.
- **Local-model role.** Kernel-to-Rust port under vector
  parity, gates.
- **Gates.** G1–G10 + conformance-vector exactness.
- **Unknowns / hard stops.** Wave assignment is C1; the wave
  brief must state which order it follows and why.

### 5.11 by-heart — Population A, Wave 3

- **State of record.** .mjs tooling + docs, 54 files; Web/JS
  family shape.
- **Remaining rungs.** R0 → R1 reduced → R2 audit → R3 →
  R4–R9: Rust rebuild.
- **MCP workflow.** torogao host. theodolite: LSP, formatter
  routing. bloch: R9.
- **Local-model role.** Port and build, gates.
- **Gates.** G1–G10 (+ vector exactness where vectors exist).
- **Unknowns / hard stops.** None recorded beyond standing law.

### 5.12 creamery — Population A, Wave 3

- **State of record.** .mjs + .zig fragments; Web/JS family
  shape.
- **Remaining rungs.** R0 → R1 reduced → R2 audit → R3 →
  R4–R9: Rust rebuild.
- **MCP workflow.** torogao host. theodolite: LSP, formatter
  routing. bloch: R9.
- **Local-model role.** Port and build, gates.
- **Gates.** G1–G10.
- **Unknowns / hard stops.** None recorded beyond standing law.

### 5.13 exposure — Population A, Wave 3

- **State of record.** .mjs tooling + docs; Web/JS family shape.
- **Remaining rungs.** R0 → R1 reduced → R2 audit → R3 →
  R4–R9: Rust rebuild.
- **MCP workflow.** torogao host. theodolite: LSP, formatter
  routing. bloch: R9.
- **Local-model role.** Port and build, gates.
- **Gates.** G1–G10.
- **Unknowns / hard stops.** None recorded beyond standing law.

### 5.14 idle-hands — Population A, Wave 3

- **State of record.** .mjs tooling + docs; Web/JS family shape.
- **Remaining rungs.** R0 → R1 reduced → R2 audit → R3 →
  R4–R9: Rust rebuild.
- **MCP workflow.** torogao host. theodolite: LSP, formatter
  routing. bloch: R9.
- **Local-model role.** Port and build, gates.
- **Gates.** G1–G10.
- **Unknowns / hard stops.** None recorded beyond standing law.

### 5.15 milkmaid — Population A, Wave 3

- **State of record.** .mjs tooling + docs; Web/JS family shape.
- **Remaining rungs.** R0 → R1 reduced → R2 audit → R3 →
  R4–R9: Rust rebuild.
- **MCP workflow.** torogao host. theodolite: LSP, formatter
  routing. bloch: R9.
- **Local-model role.** Port and build, gates.
- **Gates.** G1–G10.
- **Unknowns / hard stops.** None recorded beyond standing law.

### 5.16 standard-issue — Population A, Wave 3

- **State of record.** .mjs + .cpp fragments; Web/JS family
  shape.
- **Remaining rungs.** R0 → R1 reduced → R2 audit → R3 →
  R4–R9: Rust rebuild.
- **MCP workflow.** torogao host. theodolite: LSP, formatter
  routing. bloch: R9.
- **Local-model role.** Port and build, gates.
- **Gates.** G1–G10.
- **Unknowns / hard stops.** None recorded beyond standing law.

### 5.17 trancefall — Population A, Wave 3

- **State of record.** .mjs + .zig fragments; Web/JS family
  shape.
- **Remaining rungs.** R0 → R1 reduced → R2 audit → R3 →
  R4–R9: Rust rebuild.
- **MCP workflow.** torogao host. theodolite: LSP, formatter
  routing. bloch: R9.
- **Local-model role.** Port and build, gates.
- **Gates.** G1–G10.
- **Unknowns / hard stops.** None recorded beyond standing law.

### 5.18 quartermaster — Population A, Wave 3

- **State of record.** .lua 19 + .js 11; LÖVE/Lua family shape.
- **Remaining rungs.** R0 → R1 reduced → R2 audit → R3 →
  R4–R9: Rust rebuild.
- **MCP workflow.** torogao host. theodolite: LSP (incl. the
  Lua reference tree's route), formatter routing. bloch: R9.
- **Local-model role.** Port and build, gates.
- **Gates.** G1–G10.
- **Unknowns / hard stops.** None recorded beyond standing law.

### 5.19 redlight — Population A, Wave 3

- **State of record.** Unity markers (.cs, .csproj) + web
  tooling; a Unity/C# jam entry.
- **Remaining rungs.** R0 → R1 reduced → R2 audit → R3 →
  R4–R9: Rust rebuild.
- **MCP workflow.** torogao host. theodolite: LSP, formatter
  routing. bloch: R9.
- **Local-model role.** Port and build, gates.
- **Gates.** G1–G10.
- **Unknowns / hard stops.** None recorded beyond standing law.

### 5.20 second-wave — Population A, Wave 3

- **State of record.** Web build (html/js/css) + a Unity
  marker.
- **Remaining rungs.** R0 → R1 reduced → R2 audit → R3 →
  R4–R9: Rust rebuild.
- **MCP workflow.** torogao host. theodolite: LSP, formatter
  routing. bloch: R9.
- **Local-model role.** Port and build, gates.
- **Gates.** G1–G10.
- **Unknowns / hard stops.** None recorded beyond standing law.

### 5.21 desperado — Population A, Wave 4

- **State of record.** Godot original, 471 files, 36 MB,
  branch pax/art-stage1-20261109, dirty 1 (not ours — hands
  off, per the census). A space-western arena-duels title
  (Game Off 2026); its GAME-PAGE carries no 18+ gate, as the
  census records. It is the largest Godot title in the
  population.
- **Remaining rungs.** R0 (records audit — branch and dirty
  state verified, never assumed; the dirty file is hands-off)
  → R1 reduced → R2 audit → R3 → R4–R9: Rust rebuild.
- **MCP workflow.** torogao host. theodolite: LSP, formatter
  routing, DAP, SCIP (the population's largest tree). bloch:
  R9.
- **Local-model role.** Port at scale, gates.
- **Gates.** G1–G10.
- **Unknowns / hard stops.** The dirty-1 tree state is a
  hands-off flag for R0, recorded in the census.

### 5.22 roseford — Population A variant (build, not rebuild), after Wave 2 / Wave 4

- **State of record.** Cargo.toml + docs (23 .md);
  design-stage original, a dungeon crawler. Its design was
  fed by the Unholy Maiden study (premise/systems shapes
  only, D-1123).
- **Remaining rungs.** Not a D-1563 rebuild: it takes its own
  commission shape, queued after wave 2 (CENSUS §1) and
  listed in wave 4 by PIPELINE §1.3. Its build consumes the
  Unholy Maiden study products as they land (§5.28).
- **MCP workflow.** torogao engine. theodolite: LSP,
  formatter routing, SCIP. ardor: content authoring surface.
  phaedrus: research seat for dungeon-crawler technique
  questions the design docs leave open. bloch: close-out
  candidates under its own commission.
- **Local-model role.** Build labour under its own commission.
- **Gates.** Its own commission's gate set; the D-1563 G-set
  applies where that commission adopts it — the sources do
  not state that it does.
- **Unknowns / hard stops.** Commission shape is the Owner's;
  nothing in the sources scopes it further.

### 5.23 Aria Succubism — Population C, Wave 4 (QUEUE order)

- **State of record.** Spec + the S2c expressions pilot — the
  only title for which S2c was executed (PIPELINE §7 gap 2).
  **Content-eligible** (one of the two originally eligible
  titles). Engine: not stated in the census sections.
- **Remaining rungs.** The **audio-sync spike first** (its
  verdict gates S4/R4; PIPELINE §7 gap 4) → R1 verification →
  R2 → R3 → R4–R9.
- **MCP workflow.** fathom: study-side routes of record.
  phaedrus: research seat for the audio-sync technique
  question the spike must answer. torogao: engine + host.
  theodolite: LSP, formatter routing (build rungs). ardor:
  content surface (eligible title; §4.3 study scope).
  bloch: R9.
- **Local-model role.** Spike execution support, spec
  verification, build, gates.
- **Gates.** The spike verdict is this title's named
  pre-gate; then G1–G10.
- **Unknowns / hard stops.** S4 is spike-first on the same
  knowledge as Hypno_Rhythm (§5.25); the spike needs quiet
  measurement conditions.

### 5.24 CIRCLEMATE — Population C, Wave 4 (QUEUE order)

- **State of record.** Intake + animation note.
  **Content-eligible** (the second originally eligible
  title). Its S2c expressions note has **not yet been
  produced** (PIPELINE §7 gap 2). Engine: not stated in the
  census sections.
- **Remaining rungs.** The expressions note is a **study
  commission ahead of the rebuild, not a rebuild subtask**
  → then the remaining Lane D stages → R1 → R2 → R3 →
  R4–R9. No run claims S2c parity for this title except from
  the note once it exists.
- **MCP workflow.** fathom: intake/extraction routes of
  record. torogao: engine + host. theodolite: LSP, formatter
  routing (build rungs). ardor: content surface. bloch: R9.
- **Local-model role.** Study commission labour, spec
  assembly, build, gates.
- **Gates.** G1–G10.
- **Unknowns / hard stops.** Expressions note outstanding —
  recorded as a gap, not a delay.

### 5.25 Hypno_Rhythm — Population C, Wave 4 (QUEUE order)

- **State of record.** Spec MEASURED. A spec repo exists
  ("measured specification"; CENSUS §1's hypno-rhythm repo
  row follows this study-target row). S4 is gated on the
  never-run audio-sync spike.
- **Remaining rungs.** The **audio-sync spike** (first task
  of any wave brief for this title) → R1 verification → R2 →
  R3 → R4–R9.
- **MCP workflow.** fathom: study-side routes of record.
  phaedrus: research seat for the spike's audio-sync
  technique question. torogao: engine + host. theodolite:
  LSP, formatter routing. bloch: R9.
- **Local-model role.** Spike support, build, gates.
- **Gates.** Spike verdict pre-gate; then G1–G10.
- **Unknowns / hard stops.** The spike failed to run on primo
  saturation, not on merit (PIPELINE §7 gap 4) — quiet
  measurement conditions are a brief requirement.

### 5.26 HypnoClicker — Population C, Wave 4 (QUEUE order)

- **State of record.** Full GDRE dynamics spec.
- **Remaining rungs.** R1 verification → R2 → R3 → R4–R9
  (the census's own statement for spec-grade titles).
- **MCP workflow.** fathom: the GDRE route is the study
  route of record. torogao: engine + host. theodolite: LSP,
  formatter routing, DAP. bloch: R9.
- **Local-model role.** Spec verification, build, gates.
- **Gates.** G1–G10.
- **Unknowns / hard stops.** None recorded beyond standing law.

### 5.27 Maeve's Academy — Population C, Wave 4 (QUEUE order)

- **State of record.** State vocabulary, 604 labels. Under
  PIPELINE §1.2 a vocabulary discharges only its own rung.
- **Remaining rungs.** Everything from the first
  undischarged Lane D stage forward (extraction rungs in
  QUEUE order) → R1 → R2 → R3 → R4–R9.
- **MCP workflow.** fathom: extraction routes as
  commissioned. torogao: engine + host. theodolite: LSP,
  formatter routing (build rungs). bloch: R9.
- **Local-model role.** Extraction labour, spec assembly,
  build, gates.
- **Gates.** G1–G10.
- **Unknowns / hard stops.** The census records no engine,
  no spec, and no per-rung detail for this title beyond the
  vocabulary — the tracker says so in those words.

### 5.28 Unholy Maiden — Population C, QUEUE family (wave unassigned)

- **State of record.** RPG Maker family (data JSONs + plugin
  JS, per the census record). Two rungs stand: the **data
  dictionary is discharged** (Lane D QUEUE #21) and a
  **dynamics spec was assembled 2026-11-18** at primo
  `~/.local/share/pax/rebuild-lane-a-20261118/unholy-maiden/SPEC.md`
  — a **draft, unfrozen**, with an 8-item UNKNOWN register.
  The census's rung correction governs: §4's "spec-grade or
  better" grouping is superseded for this title; a dictionary
  discharges only its own rung, and no S4/S5 lineage exists.
  Base tree re-homed to
  `originals/Unholy_Maiden/Unholy Maiden v1.04/` (mod payload
  data JSONs 405: 391 numbered maps + MapInfos + 13 core
  tables; base data JSONs 345; ground truth re-verified
  2026-11-18); the mod tree stands at
  `originals/unholy-maiden-seed-of-profanity/Seed Of Profanity [Final]/`.
  Its study feeds Roseford's design (§5.22).
- **Remaining rungs.** The **D-1587 source-evidence rung**
  over the behaviour-bearing code (plugin JS + event command
  lists) — the stage has been built (2026-11-18) and has not
  run on this title → Pax spec-freeze gate → R2 → R3 →
  R4–R9. (A dedicated extraction lane was briefed and staged
  2026-11-20 on the Owner's commission; it queues in the
  staged block.)
- **MCP workflow.** fathom: the D-1587 rung and the MV/MZ
  family routes. phaedrus: research seat if a plugin/event
  format question blocks the rung. torogao: engine + host.
  theodolite: LSP, formatter routing (build rungs). bloch:
  R9.
- **Local-model role.** Source-evidence extraction labour,
  UNKNOWN-register closure work, spec revision, build, gates.
- **Gates.** Freeze gate (Pax, on the evidence) before R2;
  then G1–G10.
- **Unknowns / hard stops.** The draft's 8-item UNKNOWN
  register stands; **harassment logic is UNKNOWN** (named in
  the spec). Hard-stop screening: no trigger at record level
  (adult cast of record; SPEC.md §8 states basis + limits).

### 5.29 Teaching Feeling — systems-only permanently (hard stop)

- **State of record.** The census carries **no per-title
  entry** for this title in §§1–4; its programme record is
  PIPELINE's: the 2026-11-17 ruling lifted the D-1583
  safeguard (abstract non-sexual systems only) and re-scoped
  the title to full content parity, and the **2026-11-18
  01:05 amendment then closed its sexual content
  permanently** (restrictions list §2.1.1). RULED (Owner,
  2026-11-18 01:03): the title stays in the programme at
  systems-only scope permanently; the item is closed.
- **Remaining rungs.** A systems-only remake is this title's
  ceiling: the run executes the systems rungs (spec of
  systems, R3 oracle over systems claims, R4 systems, R5
  shell with estate-authored non-sexual content, R6–R9)
  and **no** scene/dialogue extraction, content map,
  translation inventory of sexual content, or parity
  re-authoring of sexual scenes — by any lane, local model,
  or takeover.
- **MCP workflow.** fathom: systems-level study routes only.
  torogao: engine + host. theodolite: LSP, formatter routing
  (build rungs). bloch: R9 (systems lessons only).
- **Local-model role.** Systems build labour within the
  ceiling.
- **Gates.** G1–G10 over the systems scope; the scope ceiling
  itself is the title's first gate and is not Owner-reopenable.
- **Unknowns / hard stops.** The hard stop itself (§2.1.1).
  Engine, rung detail, and wave are not stated in the two
  source documents — the tracker says so in those words.

### 5.30 Hypnotic Eyes pair — Population C, Wave 4 (QUEUE order)

- **State of record.** Distinct-verdict only (a probe-grade
  row). The census does not name the pair's two titles
  individually in the sections of record.
- **Remaining rungs.** Extraction rungs in QUEUE order →
  spec → R0–R9.
- **MCP workflow.** fathom: extraction routes as
  commissioned. torogao host at build rungs. theodolite:
  LSP, formatter routing (build rungs). bloch: R9.
- **Local-model role.** Extraction, spec, build, gates.
- **Gates.** G1–G10.
- **Unknowns / hard stops.** Everything past the
  distinct-verdict is unrecorded in the sources.

### 5.31 IHSA — Population C, Wave 4

- **State of record.** One of the IHSA-class titles (PIPELINE
  §7 names the class as QUEUE #10/#13/#22/#26) that §4
  records as waiting on the uncommissioned IL2CPP/Ghidra-class
  rung. Census §7 records that rung as commissioned (D-1202)
  and proven on Undercity Doll Shop — contradiction C2.
- **Remaining rungs.** Per PIPELINE §7 gap 3, IHSA-class
  titles do not start R1; they sit in wave 4 "as their rungs
  land", and commissioning the rung is its own Owner
  decision. Per census §7, the rung exists. The wave brief
  must re-walk the S-rung assignment before commissioning
  (census §7's instruction) → then extraction → spec →
  R0–R9.
- **MCP workflow.** fathom: the IL2CPP native rung (D-1202)
  and the metadata slice. phaedrus: research seat for the
  rung-status question itself (which record governs) and any
  native-format question the rung raises. torogao host at
  build rungs. theodolite: LSP, formatter routing (build
  rungs). bloch: R9.
- **Local-model role.** Extraction, spec, build, gates.
- **Gates.** G1–G10.
- **Unknowns / hard stops.** The C2 contradiction is this
  title's named blocker.

### 5.32 Undercity Doll Shop — Population C, Wave 4

- **State of record.** IL2CPP class. Census §7: the
  IL2CPP/Ghidra-class native rung §4 waits on was
  commissioned (D-1202) and is **proven on this title**.
- **Remaining rungs.** Re-walk of the S-rung assignment
  (census §7) → remaining extraction → spec → R0–R9.
- **MCP workflow.** fathom: the native rung (D-1202), proven
  here — this title is the route's evidence base. torogao
  host at build rungs. theodolite: LSP, formatter routing
  (build rungs). bloch: R9.
- **Local-model role.** Extraction, spec, build, gates.
- **Gates.** G1–G10.
- **Unknowns / hard stops.** §4's "wait" framing is superseded
  by §7's record; the re-walk is still owed (gap G4).

### 5.33 Complicated Natsuki — Population C, Wave 4

- **State of record.** IL2CPP class. Census §7: its
  serialized values were recovered by the bundle-native
  rung 2 (**17,535/17,670 exact, D-1222**).
- **Remaining rungs.** Re-walk of the S-rung assignment
  (census §7) → remaining extraction → spec → R0–R9.
- **MCP workflow.** fathom: bundle-native reader routes
  (rung 2 of record). torogao host at build rungs.
  theodolite: LSP, formatter routing (build rungs). bloch:
  R9.
- **Local-model role.** Extraction, spec, build, gates.
- **Gates.** G1–G10.
- **Unknowns / hard stops.** The 135 non-exact values of the
  rung-2 result are carried in that rung's record; the
  census does not grade them further here.

### 5.34 Ren'Py group remainder — Population C, Wave 4 (QUEUE order)

- **State of record.** A group row: dialogue trees exist for
  **10 of 12** Ren'Py titles. The census sections of record
  do not name the 12, nor which 2 lack trees (gap G2).
- **Remaining rungs.** Per title: extraction rungs in QUEUE
  order (dialogue-tree completion for the remainder) → spec
  → R0–R9.
- **MCP workflow.** fathom: `census_dialogue_tree` is the
  route family of record (it counts .ks/.rpy-class text; the
  seven-games planning record notes it does not resolve
  cross-file targets or KAG semantics — a friction-log
  candidate). torogao host at build rungs. theodolite: LSP,
  formatter routing (build rungs). bloch: R9.
- **Local-model role.** Extraction, spec, build, gates.
- **Gates.** G1–G10 per title.
- **Unknowns / hard stops.** Group-level record only; each
  title's own rung state is established at its intake.

### 5.35 RPG Maker group remainder — Population C, Wave 4 (QUEUE order)

- **State of record.** A group row: the RPG Maker group
  remainder, no per-title rung detail in the census sections
  of record.
- **Remaining rungs.** Per title: the MV/MZ rung family
  (decrypt + data/event extraction) → D-1587 evidence →
  spec → R0–R9.
- **MCP workflow.** fathom: the MV/MZ decrypt rung and
  data routes. phaedrus: research seat for per-title format
  variants the rung has not met. torogao host at build
  rungs. theodolite: LSP, formatter routing (build rungs).
  bloch: R9.
- **Local-model role.** Extraction, spec, build, gates.
- **Gates.** G1–G10 per title; depicted-cast verification
  gates content scope wherever a title's cast is unverified
  (the Elise precedent, §5.51).
- **Unknowns / hard stops.** Group-level record only.

### 5.36 Moo-Moo Manor — Population C, Wave 4 (QUEUE order)

- **State of record.** GDRE coverage measured; spec
  unwritten.
- **Remaining rungs.** Spec drafting from the measured
  coverage → R1 → R2 → R3 → R4–R9.
- **MCP workflow.** fathom: the GDRE route (coverage of
  record). torogao host. theodolite: LSP, formatter routing.
  bloch: R9.
- **Local-model role.** Spec drafting from evidence, build,
  gates.
- **Gates.** G1–G10.
- **Unknowns / hard stops.** None recorded beyond the
  unwritten spec itself.

### 5.37 Monster Girl Assault — Population C, Wave 4 (QUEUE order)

- **State of record.** GameMaker; sizes only. The GameMaker
  decompile route is unexercised (PIPELINE §7).
- **Remaining rungs.** The run begins with the extraction
  rung as a commissioned phase, or does not begin → spec →
  R0–R9.
- **MCP workflow.** phaedrus: research seat — the GameMaker
  decompile route is an unsolved route question (prior art,
  format documentation) and its findings are the commissioned
  phase's first input. fathom: the extraction rung once
  commissioned. torogao host at build rungs. theodolite:
  LSP, formatter routing (build rungs). bloch: R9.
- **Local-model role.** Extraction (once the route exists),
  spec, build, gates.
- **Gates.** G1–G10.
- **Unknowns / hard stops.** Route nonexistence is the named
  blocker; no run commissions it implicitly.

### 5.38 HYPERDEEP — Population C, Wave 4 (QUEUE order)

- **State of record.** Unreal pak probe only. The Unreal pak
  route beyond census is unexercised (PIPELINE §7).
- **Remaining rungs.** The run begins with the extraction
  rung as a commissioned phase, or does not begin → spec →
  R0–R9.
- **MCP workflow.** phaedrus: research seat for the Unreal
  pak route question. fathom: pak census routes of record;
  the beyond-census rung once commissioned. torogao host at
  build rungs. theodolite: LSP, formatter routing (build
  rungs). bloch: R9.
- **Local-model role.** Extraction, spec, build, gates.
- **Gates.** G1–G10.
- **Unknowns / hard stops.** Route depth is the named
  blocker.

### 5.39 Detailed Cowgirl Simulator — Population C, wave unassigned

- **State of record.** Unreal IoStore (pak/ucas). Intake
  census landed 2026-11-16: the genuine **RJ01419389** build,
  loose in the RE tree. **Not** the RJ01580537 archive set
  filed under its name earlier the same day — that set's
  payload is Yandere Kamikakushi (census correction of
  record in the run's INTAKE.md). D-1587/D-1588 rung
  commissioned 2026-11-16.
- **Remaining rungs.** The commissioned D-1587/D-1588 rung →
  spec → freeze → R0–R9.
- **MCP workflow.** fathom: IoStore rung routes. phaedrus:
  research seat for IoStore format questions the rung
  raises. torogao host at build rungs. theodolite: LSP,
  formatter routing (build rungs). bloch: R9.
- **Local-model role.** Rung labour, spec, build, gates.
- **Gates.** G1–G10.
- **Unknowns / hard stops.** The same-day misfiling is the
  title's provenance caution of record (contradiction C7).

### 5.40 Puppet Master — Population C, Wave 4 (QUEUE order)

- **State of record.** SRPG Studio probe — a probe verdict
  only, which discharges nothing beyond S1 (PIPELINE §1.2).
- **Remaining rungs.** Extraction rungs in QUEUE order →
  spec → R0–R9.
- **MCP workflow.** fathom: extraction routes as
  commissioned; phaedrus: research seat if the SRPG Studio
  format question blocks a rung. torogao host at build
  rungs. theodolite: LSP, formatter routing (build rungs).
  bloch: R9.
- **Local-model role.** Extraction, spec, build, gates.
- **Gates.** G1–G10.
- **Unknowns / hard stops.** Probe-grade record only.

### 5.41 Hypnosis App — Population C, Wave 4 (QUEUE order)

- **State of record.** Native; inferred only.
- **Remaining rungs.** Extraction rungs in QUEUE order (the
  native PE triage family is the planning record's first
  rung for native titles) → spec → R0–R9.
- **MCP workflow.** fathom: native triage routes.
  phaedrus: research seat for engine identification if
  triage leaves it inferred. torogao host at build rungs.
  theodolite: LSP, formatter routing (build rungs). bloch:
  R9.
- **Local-model role.** Extraction, spec, build, gates.
- **Gates.** G1–G10.
- **Unknowns / hard stops.** Even the engine attribution is
  inference-grade in the record.

### 5.42 SoldGirl Town — Population C, Wave 4 (QUEUE order)

- **State of record.** MonoBehaviour census uncommissioned
  per census §4; census §7 records it as **unblocked by
  fathom's BQ-2 tools on main**.
- **Remaining rungs.** MonoBehaviour census (bundle-native /
  serialized-values family) → spec → R0–R9.
- **MCP workflow.** fathom: the BQ-2 tools named in §7.
  torogao host at build rungs. theodolite: LSP, formatter
  routing (build rungs). bloch: R9.
- **Local-model role.** Census labour, spec, build, gates.
- **Gates.** G1–G10.
- **Unknowns / hard stops.** The §4/§7 staleness pair is
  recorded as gap G4's family; the census itself supplies
  the newer record.

### 5.43 Mari's Story — Population C, Wave 4 (QUEUE order)

- **State of record.** Intake census next — nothing further
  is recorded in the sources: no engine, no rung, no spec.
- **Remaining rungs.** Intake census (D-1588 discipline) →
  extraction → spec → R0–R9.
- **MCP workflow.** fathom: census routes. torogao host at
  build rungs. theodolite: LSP, formatter routing (build
  rungs). bloch: R9.
- **Local-model role.** Intake, extraction, spec, build,
  gates.
- **Gates.** G1–G10.
- **Unknowns / hard stops.** The record is one line; the
  tracker says so in those words.

### 5.44 NTR Idol — Population C, Wave 4 (QUEUE order)

- **State of record.** Pak census next — nothing further is
  recorded in the sources.
- **Remaining rungs.** Pak census → extraction → spec →
  R0–R9.
- **MCP workflow.** fathom: pak census routes; phaedrus:
  research seat if the pak format is an unmet variant.
  torogao host at build rungs. theodolite: LSP, formatter
  routing (build rungs). bloch: R9.
- **Local-model role.** Census, extraction, spec, build,
  gates.
- **Gates.** G1–G10.
- **Unknowns / hard stops.** One-line record, as §5.43.

### 5.45 Lewd Avatar Simulator — appended registration (D-1652; corrected 2026-11-19), wave unassigned

- **State of record.** Unity **6000.4.2f1, IL2CPP**
  (metadata 39). Registration (2026-11-16, D-1652) carried
  119 files / 5,051,421,182 B; the intake lane's hashed
  census **supersedes** it: **118 files / 3,790,852,142 B**
  (the differing path and the 1,260,569,040 B delta — the
  sibling distribution RAR — identified by the resumed lane;
  contradiction C4 is the stale registration figure). The
  copy is a **worked tree**: a fan translation/cleanup
  toolkit ships in-tree (`tools/`, a `translator/` project);
  who produced this variant is UNKNOWN. Content ships under
  a **protected layer**: 10 `.pab` files (header magic
  MPAB1 — a custom protected bundle format, NOT UnityFS) +
  4 `.pscn` scene files (MPSC1); no estate reader exists for
  MPAB1/MPSC1 and the decryption/format route is UNPROVEN.
  Intake: stages 1–2 done; **D-1588 audit found 24
  planted-instruction files (7 distinct hashes), quarantined
  in place**, nothing executed; per the 2026-11-19 census
  correction, the intake RESUMES at stages 3–5 with the
  quarantined files excluded. The
  Owner's stated value: its avatar construction and
  animation systems are study material for the cross-medium
  animation pipeline and the lumen sprite-to-3D factory, as
  well as a rebuild candidate (VRM pipeline, runtime FBX
  import, MagicaClothV2 + spring-bone secondary motion,
  OpenXR embodiment — all named in the §8 census entry;
  fingerprint-negative: no Live2D, no Spine).
- **Remaining rungs.** Spec freeze (Pax gate) on the
  SPEC-DRAFT → R2 → R3 → R4–R9, **systems-only until cast
  verification passes**; the MPAB1/MPSC1 format route is the
  D-1587 rung's open problem.
- **MCP workflow.** fathom: bundle-native reader,
  serialized-values and native-shape rungs (the Hachina /
  StopKing intake class applies directly), D-1587
  disassembly. phaedrus: research seat — MPAB1/MPSC1 is an
  undocumented format question, a named research input to
  the D-1587 rung. lumen: the sprite-to-3D factory study
  line this title feeds. torogao: engine + host.
  theodolite: LSP, formatter routing (build rungs). bloch:
  R9.
- **Local-model role.** Intake labour under quarantine
  discipline, spec work, systems build, gates.
- **Gates.** G1–G10; pre-gates: D-1588 quarantine discipline
  and the cast-verification gate on content scope.
- **Unknowns / hard stops.** **Systems-only until cast
  verification passes** (§2.1.4); depicted-cast clearance and
  binary cheat screening are named UNKNOWNs; MPAB1/MPSC1
  decryption route UNPROVEN; `.pscn` payload encryption
  UNKNOWN; the worked tree's producer UNKNOWN.

### 5.46 hypno_booth (0.8.0, Linux) — appended registration (D-1656), wave unassigned

- **State of record.** **Godot 4.4.1 stable, GDScript** —
  a single ELF with an embedded PCK (format 2, **not
  encrypted**, 515 files), scripts compiled-only (15 `.gdc`,
  zero `.gd` source). Archive-only copy; the second archive
  is a byte-identical re-pack, not a variant. Provenance:
  no worked-tree flags, and the copy's source channel is
  UNKNOWN (a single-file export cannot show working files by
  construction). Character × state variant matrix (four
  character sets, each plain + `hb-` variant), a word/flash
  presentation system, session parameter controls, and a
  gallery surface are censused at filename/structure grade.
- **Remaining rungs.** D-1587 rung (GDRE route; `.gdc`
  decompile labour) + D-1588 audit → spec freeze — with the
  **D-1656 transformations binding on the freeze**
  (restrictions §2.1.12) → R2 → R3 → R4–R9.
- **MCP workflow.** fathom: the GDRE route (the B-07 estate
  wrap of GDRE stands as a route-quality gap). phaedrus:
  research seat for `.gdc` decompile fidelity on this title.
  torogao: engine + host; the word/flash system sits on
  torogao's hypno-technique line. theodolite: LSP, formatter
  routing (build rungs). bloch: R9.
- **Local-model role.** Decompile labour, timing/cadence
  measurement support, spec, build, gates.
- **Gates.** G1–G10; the freeze gate carries the D-1656
  transformations as binding inputs.
- **Unknowns / hard stops.** UNKNOWNs of record: the
  original's language (72 `.txt` files unread); the
  project/display name; whether Windows/Android builds of
  v0.8.0 exist in the tree; the word/flash timing and
  cadence values (D-1587 measurements, not yet taken);
  `.gdc` decompile fidelity (unproven per-title); the copy's
  source channel. S2c line in §9 says NOT content-eligible
  with no supersession note (contradiction C5).

### 5.47 White Knight Discipline — appended registration (D-1673/D-1674), wave unassigned

- **State of record.** **Unity Mono** (White.exe + White_Data
  + MonoBleedingEdge; Assembly-CSharp.dll). First-pass
  fingerprint only (2026-11-17): the copy carries a **BepInEx**
  install (XUnity.AutoTranslator + ResourceRedirector +
  DumbRendererDemosaic) — a translation-prepared copy — and a
  root `save.dat` showing play. The Owner's stated study
  value is its BDSM/corruption mechanics as design input for
  the Torogao heroines and the Strobe cast; D-1674 adds a
  fidelity requirement: the intake also inventories its
  character-animation system as a named studied reference for
  the cross-medium animation pipeline (first-pass evidence:
  NOT Live2D, NOT Spine — the animation lives inside the
  Unity serialized assets; the AssetRipper rung (D-1653)
  names it exactly during intake).
- **Remaining rungs.** Full intake (file inventory, assembly
  map with game assemblies separated from BepInEx/plugin
  assemblies, D-1588 static security audit, animation-system
  inventory) → D-1587 → spec → freeze → R0–R9. The census
  records the intake as queueing with RE-lane staffing.
- **MCP workflow.** fathom: census + the AssetRipper rung
  (D-1653) for the animation inventory. torogao: the study's
  consumer (heroine mechanics/animation reference) and the
  build host. theodolite: LSP, formatter routing (build
  rungs). bloch: R9.
- **Local-model role.** Intake labour, spec, build, gates.
- **Gates.** G1–G10; the D-1674 animation inventory is a
  named intake deliverable.
- **Unknowns / hard stops.** Full intake not yet run — every
  value past the first-pass fingerprint is unrecorded.
  Translation cache is study evidence only.

### 5.48 Vesper Isles — appended registration (D-1675) + intake/R1 delta, wave unassigned

- **State of record.** **Godot 4.7.2.stable.official**,
  single-file export, embedded PCK format 4 (not encrypted),
  704 files. Circle: Honey Gear (DLsite RG01083108).
  Phase-(1) deep static intake COMPLETE (2026-11-18);
  D-1588 audit complete (no AI-instruction files; a dormant
  env-var-gated capture/test harness ships in the binary —
  recorded, not inherited). The **R1 spec stands in DRAFT**
  at the lane workdir (SPEC.md, sha256 `10c578fe…` as
  recorded in the census), drafted from
  the **GDRE-recovered source: 139/139 scripts**, and **all
  9 intake UNKNOWNs (U-1..U-9) are CLOSED** with source
  citations — tuning numbers, the production-chain graph,
  the four JSON files' field-level schemas, the cast model
  (32 names + numeral recycling, 8 look palettes, one VRM),
  per-language string counts. Systems at code grade: 27
  buildings / 7 stations, 14 research nodes, 27 quests,
  contracts, the Edging Governor, chamber, 13 outfits, body
  development, achievements, gallery, 5-language i18n, audio
  (160 indexed streams; 570 voice clips from 16 takes), VRM
  presentation (57 face morphs, 20 spring-bone springs).
  The census's own two audio counts differ by stage: ~163
  ogg streams at intake, 160 indexed streams in the R1
  record (gap G7).
- **Remaining rungs.** **Pax freeze gate** — including the
  SPEC §4 content-scope ruling (27 buildings, 139-script
  breadth), which the lane recorded for the Owner's ruling
  and did not decide → R2 manifest → R3 → R4–R9. The
  registration's "native Linux first per the Owner's word"
  stands against the dual-target law (contradiction C9).
- **MCP workflow.** fathom: the GDRE route (gdre-route rung
  B-07, validated for this title at 139/139). torogao:
  engine + host. lumen: VRM/media study line where the
  rebuild's presentation touches it. theodolite: LSP,
  formatter routing, DAP. bloch: R9.
- **Local-model role.** Spec-freeze support, manifest
  production, build, gates.
- **Gates.** Freeze gate (Pax, on the Owner's ruling); then
  G1–G10. No S2c expressions note exists, so no
  expression-parity rows are claimed.
- **Unknowns / hard stops.** The source README asks that the
  game not be re-uploaded and its data not
  extracted/reused (§2.1.13); content is estate-authored
  regardless. VES-D4 REVIEW note open.

### 5.49 SiNiSistar 2 — appended registration (D-1680) + intake delta, wave unassigned

- **State of record.** **Unity 2022.3.62f2, IL2CPP**
  (GameAssembly.dll 54,164,992 B; global-metadata.dat
  12,192,808 B); single-scene build; content payload = 3,146
  hash-named bundles (2,042,135,072 B). **Static intake
  COMPLETE** (phase 1 of the D-1680 registration discharged;
  INTAKE.md of record). The copy is **worked/modded**: a
  `Mod/` overlay (3,312 files — fan localization tables +
  player-sprite frames; producer UNKNOWN) — study only,
  nothing ships. D-1588 container audit: 0 AI-instruction
  hits. Dual role kept distinct: this title is also
  Niflheim's art/UI quality reference (D-1676/D-1677); that
  relationship governs Niflheim's benchmark, while this
  programme studies the game's systems — neither role copies
  assets or expression.
- **Remaining rungs.** D-1587 fathom study-evidence → spec →
  Pax freeze gate → R2 → R3 → R4–R9. The D-1587 stage has
  been built and has not run on this title.
- **MCP workflow.** fathom: D-1587 + IL2CPP routes.
  phaedrus: research seat for bundle-format questions the
  rung raises. torogao: engine + host. theodolite: LSP,
  formatter routing (build rungs). bloch: R9.
- **Local-model role.** Study-evidence labour, spec, build,
  gates.
- **Gates.** Freeze gate; then G1–G10.
- **Unknowns / hard stops.** Hard-stop screening: no trigger
  at intake level (protagonist of record Lelia, an adult
  sister per estate records; basis and limits in INTAKE.md
  §6). Mod overlay producer UNKNOWN.

### 5.50 Hachina — appended registration (2026-11-17) + full census entry, wave unassigned

- **State of record.** **Unity 2020.1.8f1, Mono**, two
  Windows builds (32-bit v1.9; 64-bit ver.1.12 "Uncensored
  and unlocked gallery"). Full intake DONE (census, engine,
  D-1588 audit, study notes, text shape): a 2D
  side-scrolling action-exploration game (HAC-D1: the
  "run-and-gun" label is not supported by the code).
  Serialized-values rung ran (275,442 MonoBehaviours —
  275,441 exact / 0 partial / 1 review) and the native-shape
  rung derived AnimationClip + PlatformEffector2D layouts
  (677 native records, 677 exact). **R1 SPEC FROZEN
  2026-11-18** (sha256 `2ca7f1a0…`, FREEZE-RECORD.md landed
  in the remakes home) with **7 UNKNOWNs carried as named
  opens**: U-4/U-5/U-6 (code, D-1587), U-8 (manifest), U-9
  (asset diff), U-10 (record check), U-12 (Owner ruling,
  incl. HAC-D1). D-1588 findings of record: a BepInEx
  Doorstop leftover in 1.12 (preloader target absent); game
  code file I/O confined to save JSON.
- **Remaining rungs.** R2 manifest (U-8) → R3 → R4 data
  layer (buildable now); the **behaviour layer is gated
  behind U-4/U-5/U-6** (a D-1587 decompile pass over the
  manager classes) → R5 → R6 → R7 → R8 → R9.
- **MCP workflow.** fathom: bundle-native reader family
  (serialized-values + native-shape rungs of record) and
  the owed D-1587 pass. torogao: engine + host. theodolite:
  LSP, formatter routing, DAP. bloch: R9.
- **Local-model role.** Manifest production, decompile-pass
  labour, data-layer build, gates.
- **Gates.** G1–G10; the behaviour-layer gate is U-4/U-5/U-6
  closure by evidence, not by the freeze.
- **Unknowns / hard stops.** The 7 named opens (§2.1.7);
  HAC-D1 rides U-12. Location divergence HAC-D2 (the tree
  sits under the remakes landing home, not
  ntranch-game-re/) is recorded in the census itself.

### 5.51 StopKing — appended registration (2026-11-18), wave unassigned

- **State of record.** **Unity 2021.3.6f1, Mono** (MEASURED
  from globalgamemanagers), x86_64. **Intake COMPLETE
  through the D-1587 decompile stage** (all routes rc=0:
  census_game_tree, exhaustive MonoBehaviour census — 430
  script classes / 14,520 instances — managed-assembly
  inventory, ilspycmd → 566 .cs files). The serialized-values
  rung ran (14,770 MonoBehaviours — 14,765 exact / 0 partial
  / 5 review), resolving SPEC U-1, U-2, U-3, U-9, U-10.
  **R1 SPEC FROZEN 2026-11-18** (sha256 `30a5a0eb…`,
  FREEZE-RECORD.md in the remakes home). Copy state: worked
  / translation-prepared (BepInEx + XUnity.AutoTranslator +
  ReiPatcher; the Assembly-CSharp of record is a patched
  assembly, deltas vs pristine UNKNOWN; the 1,954-line
  AutoTranslator cache is study evidence only).
- **Remaining rungs.** R2 manifest → R3 → R4–R9.
  Content-level work stays behind the REVIEW-grade identity
  until **U-4** closes; U-5..U-8 remain open as named items,
  none blocking the data layer.
- **MCP workflow.** fathom: census_game_tree + the
  decompile route of record; bundle-native serialized-
  values rung. torogao: engine + host. theodolite: LSP,
  formatter routing, DAP. bloch: R9.
- **Local-model role.** Manifest, oracle, build, gates.
- **Gates.** G1–G10.
- **Unknowns / hard stops.** U-4 (listing identity,
  Pax/Owner-side) gates content-level work (§2.1.6);
  identity verification stands at REVIEW grade (adult by
  content class/channels; readme cast = credited VRChat-class
  adult avatar models; no age/student evidence in shipped
  docs; hard-stop check performed, not triggered). D-1588
  open items of record: an unidentified native plugin
  (`abci.dll`); Unity Services config ships in
  StreamingAssets (live use UNKNOWN).

### 5.52 Suspended Sex Simulator vol02 v1.1.1 — lane C intake, wave unassigned

- **State of record.** **Unity 2021.3.17f1, Mono**;
  archive-only copy (one zip; listing + header reads only).
  Stack flags of record: Obi rope/cloth physics, Live2D
  CubismCore native with no Cubism managed assembly (use
  UNKNOWN), Unity Services, steam_api64, capture tooling.
  Text: no loose text files — volume + languages UNKNOWN
  until D-1587 + R2. D-1588: no AI-instruction filenames;
  the network/service surface is flagged for the full audit.
  SPEC draft exists in the lane workdir, not frozen.
- **Remaining rungs.** D-1587 disassembly → serialized-
  values rung → spec freeze (Pax gate) → R0–R9.
- **MCP workflow.** fathom: D-1587 + serialized-values
  routes. phaedrus: research seat for the CubismCore-native
  question (use UNKNOWN). torogao host at build rungs.
  theodolite: LSP, formatter routing (build rungs). bloch:
  R9.
- **Local-model role.** Rung labour, spec, build, gates.
- **Gates.** Freeze gate; then G1–G10.
- **Unknowns / hard stops.** Text volume/languages UNKNOWN;
  Cubism use UNKNOWN.

### 5.53 Elise and the Spellbinding Marionette — lane C intake, wave unassigned

- **State of record.** **RPG Maker MV** (NW.js);
  archive-only copy (listing + package.json stamp only; no
  data contents read): 123 maps, 15 database JSONs, 1,366
  encrypted images, 436 encrypted audio, 149 JS incl. a
  dedicated erotic-status plugin family by filename; one
  `www/save/` member of UNKNOWN provenance. Text volume +
  language UNKNOWN until the MV/MZ rung. D-1588: no
  AI-instruction filenames.
- **Remaining rungs.** The **MV/MZ study rung — depicted-cast
  verification is its first step** → D-1587 evidence → spec
  freeze → R0–R9. **Systems-only until verification passes,
  permanently if it fails.**
- **MCP workflow.** fathom: the MV/MZ decrypt rung.
  phaedrus: research seat for the cast-verification evidence
  question (what the build's own records establish).
  torogao host at build rungs. theodolite: LSP, formatter
  routing (build rungs). bloch: R9.
- **Local-model role.** Rung labour under the scope ceiling,
  spec, systems build, gates.
- **Gates.** The cast-verification gate precedes all content
  scope; then the freeze gate and G1–G10.
- **Unknowns / hard stops.** **ELI-U-1: depicted-cast ages
  UNVERIFIED** (§2.1.2) — the title's governing restriction.

### 5.54 Clergy n Cocktail — lane C intake, wave unassigned

- **State of record.** **Unity 2022.3.62f1, Mono**;
  archive-only copy (listing + header reads only); single
  level0 scene; streamed texture payload ≈1.94 GB. Stack
  flags: Spine (spine-unity), Unity 2D Animation/IK,
  Cinemachine, Timeline, A* Pathfinding, Visual Scripting,
  DOTween, Burst. Text: all in assembly/assets — volume +
  languages UNKNOWN until D-1587 + R2. D-1588: no
  AI-instruction filenames. SPEC draft in the lane workdir,
  not frozen.
- **Remaining rungs.** D-1587 disassembly → serialized-
  values rung → spec freeze → R0–R9.
- **MCP workflow.** fathom: D-1587 + serialized-values
  routes; lumen: Spine/media study line where the rebuild's
  presentation touches it. torogao host at build rungs.
  theodolite: LSP, formatter routing (build rungs). bloch:
  R9.
- **Local-model role.** Rung labour, spec, build, gates.
- **Gates.** Freeze gate; then G1–G10.
- **Unknowns / hard stops.** Text volume/languages UNKNOWN.

### 5.55 The Censor — appended intake + R1 freeze (2026-11-19), wave unassigned

- **State of record.** **Unity 2022.3.0f1, Mono** (URP 2D,
  Spine, YooAsset bundles, I2 Localization: 6 columns, 2,473
  terms). Intake census + **D-1588 audit DONE** (no
  persistence, credential access, or prompt-injection
  payload found; no AI-instruction files; native code not
  cleared — out of static scope) + **D-1587 DONE** at the
  IL-decompile route (ilspycmd; the fathom MCP was
  unavailable that session; coverage in DISASSEMBLY.md).
  The copy is **cheat-patched** (GM panel, money, exp
  edits). **R1 SPEC FROZEN 2026-11-19** on the Owner's
  ruling (sha256 `91141383…`, FREEZE-RECORD.md in the
  remakes home): the cheat-patched v3.1.5 stays the oracle;
  U-10 closed by ruling. (Programme history of record: this
  title's 2026-11-16 dispatch refusal closed an earlier run;
  the Owner's 2026-11-17 ruling lifted the exclusion and the
  fresh attempt proceeded under the refusal protocol.)
- **Remaining rungs.** R2 → R3 → R4–R9, with the carried
  opens worked by evidence: U-1, U-2 (decompile
  reconstruction of the touched sites), U-3 (R3 fixture),
  U-5 (seeded estate RNG), U-7(a)/(b), U-8, U-9 (bundle
  rung), U-11, U-12.
- **MCP workflow.** fathom: IL-decompile route of record +
  the bundle reader rung (U-9). torogao: engine + host.
  lumen: Spine/media line. theodolite: LSP, formatter
  routing, DAP. bloch: R9.
- **Local-model role.** Manifest, oracle, decompile
  reconstruction labour, build, gates.
- **Gates.** G1–G10.
- **Unknowns / hard stops.** Cast: main heroines stated
  adult (26 / 22 / 24); **two NPC slots carry no stated age**
  (UNKNOWN; rebuilt as estate-authored adults per the
  U-10 ruling, §2.1.5). Whether the patched assembly carries
  further edits beyond the three known is NOT DETERMINABLE,
  carried UNKNOWN.

### 5.56 Iris the Blood Servant — appended registration (2026-11-19), wave unassigned

- **State of record.** **Unity 2022.3.62f2, Mono** — the
  Censor's intake class (ilspycmd decompile route and the
  serialized-values rung apply directly). Registration facts
  are from a static archive listing only (one zip holding
  one Windows PC build); nothing extracted, nothing
  executed. Intake NOT commissioned.
- **Remaining rungs.** Intake commission: D-1588 static
  security audit + D-1587 disassembly → spec freeze (Pax
  gate) → R0–R9.
- **MCP workflow.** fathom: D-1588/D-1587 routes of the
  class. torogao host at build rungs. theodolite: LSP,
  formatter routing (build rungs). bloch: R9.
- **Local-model role.** Intake, spec, build, gates.
- **Gates.** Freeze gate; then G1–G10.
- **Unknowns / hard stops.** Everything past the listing is
  unrecorded.

### 5.57 Agent Mirai and the Submission Machines — appended registration (2026-11-19), wave unassigned

- **State of record.** **TyranoBuilder (TyranoScript) in an
  Electron shell** — a new engine family for the queue
  (scenario and data live inside `resources/app.asar`,
  1,939,280,017 B; the MV/MZ decrypt rung does not apply).
  Developer: Mediocre Umaaji Lady II; Steam version,
  2024-09-19. The copy is held two ways (zip + extracted
  tree); the extracted tree carries saves dated 2026-11-16 —
  the copy has been played; the zip's bundled saves are the
  originals. Registration facts are from static listings
  only. Intake NOT commissioned.
- **Remaining rungs.** Intake commission: D-1588 audit →
  **static asar census** → spec freeze (Pax gate) → R0–R9.
- **MCP workflow.** fathom: the Tyrano scenario-graph rung
  (the seven-games planning record's rung 2) once the asar
  census exists. phaedrus: research seat for the asar /
  TyranoScript format questions — a new family means the
  route evidence is thin. torogao host at build rungs.
  theodolite: LSP, formatter routing (build rungs). bloch:
  R9.
- **Local-model role.** Census, spec, build, gates.
- **Gates.** Freeze gate; then G1–G10.
- **Unknowns / hard stops.** Nothing past the listings is
  recorded; the engine family's coverage is itself a
  planning output.

### 5.58 Goblin Burrow — appended registration (2026-11-19), wave unassigned

- **State of record.** **Kirikiri (xp3)**, v220322
  (Peperoncino; DLsite RE291561). One archive
  (`GoblinsBurrow_AIO_V220322.7z`, sha256 `638bd67d…`),
  static listing only. **Provenance: the community "AIO
  unified edition" repack, not DLsite's original
  packaging**; the intake census verifies contents against
  the version. Nothing extracted, nothing executed. Intake
  NOT commissioned.
- **Remaining rungs.** Intake commission: D-1588 audit →
  **xp3 static census** → spec freeze → R0–R9.
- **MCP workflow.** fathom: xp3 census routes; phaedrus:
  research seat for Kirikiri/KAG format questions the
  census raises. torogao host at build rungs. theodolite:
  LSP, formatter routing (build rungs). bloch: R9.
- **Local-model role.** Census, spec, build, gates.
- **Gates.** Freeze gate; then G1–G10.
- **Unknowns / hard stops.** Repack provenance (§2.1.14).

### 5.59 Mindsweeper — appended registration (2026-11-19), wave unassigned

- **State of record.** v1.04 (Bias Factory; DLsite
  RJ01125123). One archive (sha256 `14f43c1e…`), static
  listing only (1,626 members, counting-corrected). The
  archive contains a `.git` working-tree distribution.
  **Engine identification was the planning lane's first
  job**; the registration does not state it, and the
  appended planning record's recommended rung order
  includes an **NScripter script-evidence** rung for the
  seven-title family. Nothing extracted, nothing executed.
- **Remaining rungs.** Planning lane (engine ID + fathom
  coverage) → D-1588 audit → spec freeze → R0–R9.
- **MCP workflow.** fathom: the shared native PE triage
  (planning rung 1) + the NScripter script-evidence rung
  (planning rung 3). phaedrus: research seat for the engine
  ID if triage leaves it open. torogao host at build rungs.
  theodolite: LSP, formatter routing (build rungs). bloch:
  R9.
- **Local-model role.** Planning, census, spec, build,
  gates.
- **Gates.** Freeze gate; then G1–G10.
- **Unknowns / hard stops.** Engine not stated in the
  registration itself (gap G3).

### 5.60 Hypnotic-Family Planning S — appended registration (2026-11-19), wave unassigned

- **State of record.** v1.03 (tanedukeitinengo; DLsite
  RJ01185015). **TyranoBuilder in an Electron/NW.js shell**
  (the Agent Mirai engine family). One archive (sha256
  `f79f79fe…`), static listing only (12,856 members,
  counting-corrected). Nothing extracted, nothing executed.
- **Remaining rungs.** Planning lane (fathom coverage) →
  **depicted-cast verification** → D-1588 audit → spec
  freeze → R0–R9.
- **MCP workflow.** fathom: Tyrano scenario-graph rung.
  phaedrus: research seat for the cast-verification
  evidence question. torogao host at build rungs.
  theodolite: LSP, formatter routing (build rungs). bloch:
  R9.
- **Local-model role.** Planning, census under the scope
  rule, spec, build, gates.
- **Gates.** The cast-verification gate governs scope
  before all content work; then the freeze gate and
  G1–G10.
- **Unknowns / hard stops.** **Named unknown (hard): the
  listing's tag set includes "Loli"; depicted-cast
  verification is an explicit named unknown** — if
  verification fails, this title is systems-only
  permanently (§2.1.3). The unknown governs scope
  regardless of S2c eligibility.

### 5.61 Underground Idol X — appended registration (2026-11-19), wave unassigned

- **State of record.** Final (Nylon Heart; DLsite RE117849,
  2013). **Adobe AIR / Flash** (`movie/drama.swf`,
  `scene00.swf` series, AIR application.xml). One archive
  (sha256 `1316d05c…`), static listing only (43 members,
  counting-corrected). **The RE route is SWF decompilation —
  a new fathom family.** Nothing extracted, nothing
  executed.
- **Remaining rungs.** Planning lane (fathom SWF coverage
  vs gaps) → D-1588 audit → spec freeze → R0–R9.
- **MCP workflow.** phaedrus: research seat — the SWF
  evidence route is an unsolved family question (prior art,
  decompiler evidence) feeding the planning lane and the
  friction log. fathom: the AIR/SWF static-evidence rung
  (planning rung 6) once scoped. torogao host at build
  rungs. theodolite: LSP, formatter routing (build rungs).
  bloch: R9.
- **Local-model role.** Planning, census, spec, build,
  gates.
- **Gates.** Freeze gate; then G1–G10.
- **Unknowns / hard stops.** The route itself is the named
  gap; no run commissions it implicitly.

### 5.62 Dohna Dohna — appended registration (2026-11-19), wave unassigned

- **State of record.** v1.02, English (Alicesoft;
  publisher Shiravune). **Alicesoft System4 family**
  (`dohnadohna.ain`, .afa set in the full tree). One
  archive (sha256 `c8f7e349…`), static listing only (157
  members, counting-corrected). **Provenance: a community
  "eng Cracked Patch 1.02" repack (full game tree), not
  DLsite's original packaging; the separate uncensor patch
  was NOT acquired; repack provenance is open.** Nothing
  extracted, nothing executed.
- **Remaining rungs.** Planning lane (System4 AIN/AFA
  census — planning rung 7) → D-1588 audit → spec freeze →
  R0–R9; the exe-vs-.ccn-style payload-boundary question for
  this family is answered by the census rung's evidence.
- **MCP workflow.** fathom: the System4 AIN/AFA census
  rung. phaedrus: research seat for System4 format
  documentation questions. torogao host at build rungs.
  theodolite: LSP, formatter routing (build rungs).
  bloch: R9.
- **Local-model role.** Planning, census, spec, build,
  gates.
- **Gates.** Freeze gate; then G1–G10.
- **Unknowns / hard stops.** Repack provenance open
  (§2.1.14).

### 5.63 UnHolY DisAsTeR — appended registration (2026-11-19), wave unassigned

- **State of record.** vFinal (Unholy Creation). **Clickteam
  Fusion** — the patch replaces the main `.ccn` compiled
  application, so the game is very likely Fusion; the
  exe-vs-.ccn confirmation is the intake's named question.
  Two archives (game sha256 `69c8771e…`; patch `sakey.zip`
  sha256 `e81c8b38…`), static listings only (10 + 5
  members, counting-corrected). The patch was NOT opened by
  the staged intake brief. Listing discrepancy of record:
  the censorship field says "Uncensored" while the tag list
  also carries "Censored" (contradiction C8).
- **Remaining rungs.** Planning lane (Fusion CCN events —
  planning rung 5; the staged D-1588 intake records the PE
  tool's over-cap behaviour for the 198 MB exe verbatim and
  scopes the header-bounded follow-on that can answer the
  payload-boundary question) → D-1588 audit → spec freeze →
  R0–R9.
- **MCP workflow.** fathom: the shared native PE triage
  (planning rung 1) + the Fusion CCN rung. phaedrus:
  research seat for the CCN format question. torogao host
  at build rungs.
  theodolite: LSP, formatter routing (build rungs). bloch:
  R9.
- **Local-model role.** Planning, census, spec, build,
  gates.
- **Gates.** Freeze gate; then G1–G10.
- **Unknowns / hard stops.** The exe-vs-.ccn payload
  boundary stays a named UNKNOWN until the follow-on rung
  work answers it; how the PE-triage rung handles a
  full-size game exe (as opposed to the staged brief's two
  named members) is an intake finding to record verbatim,
  not to work around silently.

### 5.64 Popolife! — appended registration (2026-11-19), wave unassigned

- **State of record.** v1.0 (NappleMill). **Cocos2d-x**
  (`player.exe` + `libcocos2d.dll` + `mozjs-33.dll`,
  `Resources/script/jsb_boot.js`). One archive
  (`Popokatsu_1.0.7z`, sha256 `2a56518b…` — byte-identical
  to the checksum the Workupload page publishes), static
  listing only (4,395 members, counting-corrected).
  Nothing extracted, nothing executed.
- **Remaining rungs.** Planning lane (Cocos project/JSB
  index — planning rung 4, conditional) → D-1588 audit →
  spec freeze → R0–R9. The staged D-1588 intake brief runs
  the shared native PE triage over the player.exe member
  first.
- **MCP workflow.** fathom: the shared native PE triage
  (planning rung 1) + the Cocos/JSB index rung; the native
  rung for the binary study.
  phaedrus: research seat for JSB binding-format questions.
  torogao host at build rungs. theodolite: LSP, formatter
  routing (build rungs). bloch: R9.
- **Local-model role.** Planning, census, spec, build,
  gates.
- **Gates.** Freeze gate; then G1–G10.
- **Unknowns / hard stops.** None recorded beyond the
  conditional status of planning rung 4.

### 5.65 Kamimachi Site — appended registration (2026-11-19), SOURCE-BLOCKED

- **State of record.** v1.021 (Lipplen, 2017). **No archive
  has been acquired**: every known download link is dead
  (the Workupload file deleted, both Mixdrop parts dead,
  Uploadhaven not found). The primo directory exists and is
  EMPTY. No listing, hash, or engine identification exists.
- **Remaining rungs.** **Source acquisition** — a fresh
  source (a live mirror or the Owner's own copy) unblocks
  this title; then planning lane (engine ID + fathom
  coverage) → D-1588 audit → spec freeze → R0–R9.
- **MCP workflow.** phaedrus: research seat — locating a
  surviving source and any format documentation is a
  research task before it is a lane task. fathom routes
  engage only once an archive lands. torogao host at build
  rungs. theodolite: LSP, formatter routing (build rungs).
  bloch: R9.
- **Local-model role.** None until a source lands.
- **Gates.** Source acquisition is the only gate that
  exists for this title today.
- **Unknowns / hard stops.** **SOURCE-BLOCKED
  (2026-11-20)** (§2.1.11) — the whole title is the unknown.

### 5.66 Present in the RE tree, not queue-registered (census §7)

The 2026-11-16 tooling audit found these titles present in
the tree that §4 / the Lane D QUEUE do not carry. The census
records their presence and (for some) an engine fingerprint
from archive listings; it assigns them **no rung, no spec,
and no wave**. They are listed here so the tracker's title
count is honest; a queue registration is an Owner act, and
no run starts on any of them from this tracker.

| Title (tree name) | What the census records |
|---|---|
| Yandere_Kamikakushi | UE; the RJ01580537 set — the set first filed under Detailed Cowgirl Simulator's name (§5.39) |
| Milking_Mira | Unity Mono |
| Do_Disciplinary_Committee_Members_Dream_of_Succubi_v1.04 | RPG Maker MV/MZ; 7,239 encrypted .rpgmvp — the tree's largest encrypted-media set |
| Female_Teacher_Brainwashing_and_Training | TyranoScript in an Electron-class shell; probable RJ423767 identity, not proven from tree bytes; the Owner's 2026-11-16 rename of the RJ423767 folder to `ftbrain` happened outside this tree (or to a copy) — `ftbrain` does not exist here |
| Annikas_Breakout | Presence only |
| Aruna_and_the_Labyrinth_of_SealedLewd_1.206 | Presence only |
| Baragon_Doll | Presence only |
| BellMaster_1.0.3_linux | Presence only |
| Bunny_Quota_Struggles | Presence only |
| Cibelle | Presence only |
| A_simulation_of_a_brainwashing_app_that_trains_a_pure_young_lady_to_be_a_pervert_ver1.2.1 | Presence only (dated 2026-11-15 in the audit) |

(Agent_Mirai also appears in the §7 list; it was
queue-registered on 2026-11-19 and is §5.57 of this tracker,
not repeated here.)

### 5.67 Not rebuild items

Recorded in the header (§2.1.16) so no wave re-litigates
them: Hands-On (done-by-construction; the reference
implementation — Rust, Android APK proven, its tree dirty-5
and hands-off), MoonDeep (design-stage flagship original;
in-development, its own programme), Strobe (flagship in
development; Bevy/Rust canon; its own programme), the
niflheim repo (pointer only), versemend (**excluded** —
qompass/SFW lane), desperado-midnight (census-grade only;
not a git repo on disk; repo-ifying it is an Owner-side
housekeeping item). The hypno-rhythm and hypnosis-card spec
repos in CENSUS §1 are not separate titles: each follows its
study-target row (§5.25, §5.6).

## 6. Part II — MCP proving-ground subjects (Owner amendment, 2026-11-20 07:25)

Eleven estate servers join the proving ground alongside the
game rebuilds: **soundings, tackle, slipway, sinfonia,
seahawk, pratique, diver mcp, brightwork, cartone, neemu,
ache**. The proving-ground work shape is the same for every
subject:

1. **Behaviour census + spec recovery** from the subject's
   live tool surface, with **assay probes as the instrument**
   (assay's canonical counter, modern-era probe, refusal
   shapes, per-variant launch-diff — the fleet test-kit built
   2026-11-19). The census output is a recovered spec: what
   the server actually serves, per profile/config variant,
   measured — not what its stale records claim.
2. **Clean-room rebuild of a bounded component** from the
   recovered spec, where the census shows a stub or missing
   lib (the enhancement plan's lib classes: stub lib ≤ ~95
   lines, or no lib at all). The bounded component is the lib
   extraction/thickening for that subject: domain modules
   behind lib.rs, the bin a thin shell.
3. **Enhancement-programme work executed BY the local-model
   stack as the validation vehicle**: proc-macro adoption
   (the contained per-crate derive, per the 2026-11-20
   commission as corrected), lib-first, profiles, and
   annotations (including the 2026-11-20 reversibility-class
   convention) — each landed under the same parity gates the
   proc-macro programme already runs (tools/list
   byte-identical, refusal probes byte-identical, cargo
   test/fmt/clippy, theodolite copy-route).
4. **A friction log per subject**, appended by the executing
   lanes in the Part I discipline (missing tool, wrong tool
   shape, model failure mode), feeding curative MCP fixes
   under their own commissions.

Identification below is from the two fleet records named in
§1 — the enhancement plan (2026-11-19, with addenda through
2026-11-20) and SERVERS.md (census of record, entries through
2026-11-19). SERVERS.md is itself a Part 2 gap in the plan
(stale and incomplete); where the two records disagree, both
readings are carried and the disagreement is the subject's
first census finding, not resolved here.

### 6.1 tackle — PILOT PAYLOAD

- **Identification (records).** Live game-editor broker
  (Unity first). SERVERS.md: PROSE-ONLY, source-read —
  "pre-standardization at census — rmcp 3.3, no era guard on
  record; 1.0 work IN FLIGHT"; launch line: primo tree only,
  "qompassai/tackle is a seed-only repo — the code has never
  landed"; U1 read core surface, 33/33 tests at U1, live
  Unity smoke ok (2026-10-24). The enhancement plan Part 2
  corrects that entry: tackle is **actually 0.3.2 published,
  live smoke PASS**. The plan's structure census classes
  tackle as a **stub lib (24 lines)**.
- **Proving-ground work.** (1) Assay census + spec recovery —
  its first job is replacing the stale census entry with a
  probed one. (2) Bounded component: stub-lib thickening
  (domain modules behind lib.rs). (3) **Its proc-macro
  adoption is the first local-model execution of Part II**
  (Owner ruling, 2026-11-20): the contained derive lands in
  tackle's repo under full parity gates, executed by the
  local-model stack. (4) Friction log from the first entry.
- **Gates.** The proc-macro programme's parity set, plus the
  census-refresh read-back (the recovered spec must match a
  fresh probe, not the stale entry).

### 6.2 slipway — graduation-rung subject (see §6.12)

- **Identification (records).** Release MCP: release
  build/install/sign-shape/bundle/certify surface (SERVERS.md;
  1.0.0 landed 2026-10-29, 15 tools at the 2026-11-03 release
  probe; config-gated launch, ≥1 project required). Plan:
  **stub lib (20 lines)**; named in Part 1's router-bug audit
  list (config-gated launch → per-variant launch-diff owed);
  Part 4 open items: the `--stdio` signing design question is
  open on the producer commit; the witness-bundle L2 consumer
  (G-W2) waits on fathom's witness disposition; four
  `not_certified` app verdicts are Owner-item-bound.
- **Proving-ground work.** (1) Assay census incl. the
  per-variant (per-config) launch-diff. (2) Bounded
  component: stub-lib thickening. (3) Enhancement work —
  **its proc-macro adoption is the graduation rung's
  payload** (§6.12): executed with a local model in the
  harness-engineer seat. (4) Friction log — kept by the
  engineer itself on the graduation rung.
- **Gates.** Parity set; the signing design question and the
  four `not_certified` verdicts are NOT this programme's to
  close (Owner-item-bound, recorded).

### 6.3 seahawk

- **Identification (records).** Born-modern broker for Unreal
  Engine 5.8's built-in MCP + S4 computer-use actuation
  (SERVERS.md, VERIFIED 2026-11-03; 8 native tools in the
  deployed configuration; lab-only charter, execute-class
  default-deny). Current line: 1.0.3 (bare launch 3 tools /
  278 tok assay-canonical class readings; `--profile full`
  8 tools / 2,709 tok). Plan: **stub lib (63 lines)**; Part 3
  counts seahawk among the profile-proven servers; Part 4:
  the full profile stands over the 2,500 budget (accepted —
  the descriptions are contract semantics); the S5 benchmark
  block is untouched (S5_BENCHMARK_GREEN still false).
- **Proving-ground work.** (1) Assay census + spec recovery
  across both profiles (launch-diff per profile). (2)
  Bounded component: stub-lib thickening. (3) Proc-macro
  adoption + annotations under parity gates, local-model
  executed. (4) Friction log.
- **Gates.** Parity set; the S5 benchmark block is not a
  proving-ground deliverable (recorded untouched, as the
  plan records it).

### 6.4 pratique

- **Identification (records).** SERVERS.md carries the whole
  of it: PROSE-ONLY, landed 2026-10-28 (created under the
  Owner's same-day word), source-read basis; qompassai/pratique
  main @ ffca6c9, version 0.2.0; tools/list 2,607 chars at
  0.2.0. **Neither record states pratique's purpose, tool
  names, or launch line** — the enhancement plan does not
  mention it at all. This tracker invents none of those.
- **Proving-ground work.** (1) The assay census IS the
  identification step for this subject: spec recovery from
  the live surface comes first, and the recovered spec is
  what scopes everything after. (2) Bounded component: only
  as the census classes its lib (unknown until probed).
  (3) Enhancement work as scoped by the recovered spec.
  (4) Friction log.
- **Gates.** Parity set once a baseline exists; before that,
  the census read-back is the gate.

### 6.5 cartone

- **Identification (records).** The generation broker MCP
  (the plan, Part 5: "its broker already routes generation
  jobs — lumen/torori are its routes made literal").
  SERVERS.md has **no entry for cartone at all** (plan Part
  2 names this as a manifest gap: the weekly conformance run
  never probes it). Plan: **stub lib (26 lines)**; Part 4:
  the youngest server (0.1.0) — no TUI (the fleet TUI
  programme predates it), and it is the **natural first
  consumer of Part 5's shared fleet-client lib**.
- **Proving-ground work.** (1) Assay census + spec recovery
  — including authoring the missing SERVERS.md entry from
  probes (a Part 2 manifest fix, executed as proving-ground
  work). (2) Bounded component: stub-lib thickening. (3)
  Proc-macro adoption + annotations, local-model executed;
  its fleet-client-lib consumption is enhancement-programme
  work that rides Part 5's own commission, sequenced with
  this subject, not inside it. (4) Friction log.
- **Gates.** Parity set; census-entry read-back against a
  fresh probe.

### 6.6 neemu

- **Identification (records).** Doujin-corpus
  census/screen/extraction registry (SERVERS.md, VERIFIED
  2026-11-03; NTRanch lane; 15 tools at 1.0.0, EFLO 6,139 tok
  at 1.0.0). Plan: a **real lib (1,249 lines)** — no stub or
  missing lib, so work-shape item (2) has no component to
  rebuild unless the census finds one. Part 3/Part 4 name
  its open question verbatim: SERVERS.md records 15 tools /
  6,139 tok at 1.0.0 while the 2026-11-18 handshake measured
  the bare launch serving **2 tools / 675 tok** at 1.0.1 —
  "one of these is wrong, or the scoping is undocumented.
  Pin the surface story down, document it, probe it." Also
  on record: the live binary lacks the D-1265/NE-1
  voice-cue enforcement gated since 2026-11-10 (programme
  state outside these two records; the 2-tool bare list is
  by design per the D-934 diet).
- **Proving-ground work.** (1) Assay census + spec recovery
  whose first deliverable is the pinned surface story
  (per-variant launch-diff across its scoping). (2) No
  bounded component on current evidence. (3) Proc-macro
  adoption + annotations + the profile documentation the
  surface story needs, local-model executed. (4) Friction
  log.
- **Gates.** Parity set; the surface-story read-back (census,
  registry record, and live surface in agreement — the
  census's own Consistency dimension).

### 6.7 ache

- **Identification (records).** Lovense haptics MCP, NTRanch
  lane (SERVERS.md: 10 tools at 1.0.0, **12 typed error
  codes**, decide() gate untouched at its gates; GitHub
  landing was ACL-deferred at census time). Plan Part 2
  corrects the version line: **actually 1.2.0 / 11 tools**.
  Plan structure census: **no lib at all** (bin-only root
  crate). Part 3 names ache's typed codes as one of the
  three home-grown error taxonomies the fleet refusal
  convention replaces.
- **Proving-ground work.** (1) Assay census + spec recovery
  (including the refusal-shape baseline — ache's 12 codes
  are taxonomy evidence for the fleet convention). (2)
  Bounded component: **lib extraction** (the no-lib class's
  named work). (3) Proc-macro adoption — the derive carries
  the Part 3 refusal taxonomy, so ache's adoption doubles as
  the taxonomy's proving case — plus lib-first, executed by
  the local-model stack. (4) Friction log. Lane law applies
  in full (NTRanch lane; §2).
- **Gates.** Parity set, with refusal probes byte-identical
  carrying extra weight here: the taxonomy migration must
  not change a single served refusal shape without that
  change being the recorded point of the rung.

### 6.8 soundings — NOT IN THE RECORDS

- **Identification.** **Neither the enhancement plan nor
  SERVERS.md carries any entry, mention, or purpose for a
  server named "soundings."** This tracker invents none.
- **Proving-ground work.** The subject's first rung is
  identification itself: establish from the estate's trees
  and deployed binaries whether a server exists under this
  name, and if it does, run work-shape item (1) (assay
  census + spec recovery) as its intake. Items (2)–(4)
  scope from whatever the census finds. Until then the
  subject is a named slot, not a scoped lane.

### 6.9 sinfonia — NOT IN THE RECORDS

- **Identification.** **Neither record carries any entry,
  mention, or purpose for a server named "sinfonia."**
  Nothing is invented here.
- **Proving-ground work.** As §6.8: identification first,
  census as intake, scope from evidence, friction log from
  the first rung.

### 6.10 diver mcp — NOT IN THE RECORDS AS A SERVER

- **Identification.** **Neither record carries an estate MCP
  server named "diver mcp."** SERVERS.md's only diver
  material is diver as the *client*: the "diver-declared
  utility servers" entry and the theodolite entry's note
  record the servers declared in diver's
  `declared_servers.lua` (the estate's phlow, periscope,
  ghidra, plus four upstream servers) — diver declares
  servers; the records do not show it serving as one. The
  enhancement plan does not name a diver server either
  (its Part 5 companion piece is the agent-card config
  generator that would *generate* diver's declared_servers
  from card + server.json pairs).
- **Proving-ground work.** Identification first, as §6.8:
  if a diver MCP server exists outside these two records,
  the census establishes its surface and the work shape
  applies from there. If the Owner's intent was the
  config-generator companion piece, that work is already
  scoped in the plan's Part 5 and rides its commission —
  recorded here as the nearest in-record referent, not as
  this subject's definition.

### 6.11 brightwork — NOT IN THE RECORDS

- **Identification.** **Neither record carries any entry,
  mention, or purpose for a server named "brightwork."**
  Nothing is invented here.
- **Proving-ground work.** As §6.8: identification first,
  census as intake, scope from evidence, friction log from
  the first rung.

### 6.12 The local-engineer graduation rung (Owner amendment, 2026-11-20 07:27)

Part II's execution model has two rungs of its own:

- **Rung 1 — the tackle pilot (§6.1).** A cloud-model
  engineer holds the harness seat; local-model workers
  execute. This proves the payload shape (parity gates,
  friction log, contained derive) with the strongest
  available engineer supervising the local stack.
- **Rung 2 — the graduation rung: slipway (§6.2).** A
  **LOCAL model sits in the harness-engineer seat** for
  slipway's proc-macro adoption — it plans the lane, drives
  the workers, works the gates, and keeps **its own friction
  log**. Conditions, pre-fixed before the run starts:
  - **Same parity gates** as the tackle pilot — no gate is
    weakened because the engineer changed.
  - **Pre-fixed retry budgets: 3 rounds per gate, 12 rounds
    total.** The budget doubles as the **D-1075
    iteration-budget measurement** (the fleet's open
    coder-iteration-budget item): the run's round counts are
    evidence for that contract, whatever the outcome.
  - **Zero driver interventions allowed.** No human or
    cloud-model hand may steer, hint, or rescue mid-run; an
    intervention ends the measurement (it does not quietly
    become a pass).
  - **Graduation is judged at the Pax gate** against the
    pre-fixed criteria — parity set green within budget,
    friction log complete, no intervention — for the
    **MCP-adoption work class**. It is **never
    self-declared** by the engineer or the lane.
- **Named expected first blocker (on record).** The **rose
  parser-tolerance defect**: models of the hf-phi-4-tools
  class emit semantically correct tool calls as fenced JSON
  text that this rose build never promotes to `tool_calls` —
  so a local engineer can be reasoning correctly while its
  calls never execute. The defect is unfixed on the current
  build (the lumen style_list schema fix and the rose parser
  tolerance fix stand as a recorded morning gate in the
  programme's records). The graduation rung expects to meet
  it first; meeting it is a friction-log entry and a
  curative-fix input, not a surprise — and a run that stalls
  on it has measured the defect, not failed silently.

## 7. Closing — wave order and the validation ledger

### 7.1 Wave order as recorded (Part I)

From CENSUS §6, mirrored by PIPELINE §1.3 (the blushline
placement conflict between CENSUS §1's table and these two
sections is contradiction C1, §8):

- **Wave 1 — consolidate Population B.** Magic Coin (pilot
  first), then Modification App, Milk Factory, Time Stop,
  Where She Asks to Fall.
- **Wave 2 — first from-scratch rebuilds.** Crescendo first
  (its staged brief stands beside PIPELINE.md), then the
  handfast and withheld platform-completion audits, then
  Hypnosis Card once the R1 rung's values are consumed.
- **Wave 3 — the JS/Lua/web jam family.** blushline first,
  then by-heart, creamery, exposure, idle-hands, milkmaid,
  standard-issue, trancefall, quartermaster, redlight,
  second-wave.
- **Wave 4 — desperado, Roseford's build, and Population C**
  rebuilds in Lane D QUEUE order as their extraction rungs
  land.
- **Unassigned in the sources:** every appended registration
  (LAS, hypno_booth, White Knight Discipline, Vesper Isles,
  SiNiSistar 2, Hachina, StopKing, Suspended Sex Simulator,
  Elise, Clergy n Cocktail, The Censor, Iris, Agent Mirai,
  Goblin Burrow, and the seven 2026-11-19 lewdzone titles)
  carries no wave in CENSUS.md or PIPELINE.md. Their next
  rungs are recorded per title in §5; their wave placement
  is an Owner/Pax act, not read into this tracker.
- Teaching Feeling stands outside the wave mechanics: its
  systems-only ceiling (§2.1.1) governs any run whenever it
  is scheduled.

### 7.2 Part II sequencing as recorded

The Owner's amendments fix two points and no more: **tackle
first** (the pilot payload, §6.1/§6.12 rung 1), then
**slipway** as the graduation rung (§6.12 rung 2). The
remaining nine subjects carry no recorded order; the
enhancement plan's own sequencing (Phase 0 pilot → Phase 1
correctness & census → Phase 2 lib-first → Phase 3
tool-calling → Phase 4 docs rollout) is the frame their
enhancement work rides, and Part II execution order beyond
the two fixed points is set when each lane is commissioned.

### 7.3 The validation ledger

The proving ground exists to produce evidence, not just
builds. Its instrument is the **friction log**:

- **Per title (Part I) and per subject (Part II)**, the
  executing lane appends every friction finding as it is
  met, in three named classes: **missing tool** (the work
  needed a call the fleet does not have), **wrong tool
  shape** (a call exists but its surface fights the work),
  **model failure mode** (the local model, not the tooling,
  is what failed — recorded as precisely as a tooling
  finding, because the two classes demand different fixes).
- Friction findings are filed as **precise issue notes**
  (PIPELINE §8.2 discipline): what was being attempted, what
  the tool/model did, what the work needed. An MCP-server
  fix is a **separate commission**, never a run subtask —
  the run records and routes around; the fix lands under
  its own gates and is then re-validated by the next run
  that touches the same ground.
- Each run's close-out (R9 for games; the parity-gate record
  for MCP subjects) carries its friction log into the
  programme's **validation ledger**: the running account,
  per title and per subject, of what the local-model + MCP
  stack demonstrably did, where it failed, and what was
  fixed curatively in consequence. The ledger is the
  proving ground's deliverable of record — a build that
  lands without friction entries is recorded as such, and a
  lane that reports no friction at all is read with the
  same suspicion as a gate that reports no failures.
- Lessons with cross-title value exit through the R9 loop
  under lane law (origin_lane + content_label + evidence
  from birth); Part II subjects in the SFW lane and Part I
  titles in the NSFW lane never share lessons across the
  lane line except as the lane law permits.

## 8. Contradictions and gaps in the sources — listed, not resolved

**Contradictions (the two Part I sources, or a source against
itself):**

- **C1 — blushline's wave.** CENSUS §1's table says wave 2;
  CENSUS §6 and PIPELINE §1.3 put blushline first in wave 3.
- **C2 — the IL2CPP/native rung.** PIPELINE §7 gap 3 and
  CENSUS §4 describe the IL2CPP/Ghidra-class rung as
  uncommissioned / not existing, with IHSA-class titles not
  starting R1 until the Owner commissions it; CENSUS §7
  records the rung as commissioned (D-1202) and proven on
  Undercity Doll Shop.
- **C3 — the R1 serialized-values rung.** PIPELINE §7 gap 5
  and CENSUS §§3–4 describe the AssetRipper-class /
  serialized-values rung as uncommissioned, with Hypnosis
  Card's and Time Stop's values open; CENSUS §7 records the
  rung as landed (D-1219) with both titles' payloads
  produced.
- **C4 — LAS figures.** CENSUS §8's registration (119 files /
  5,051,421,182 B) is superseded by the 2026-11-19 intake
  correction (118 files / 3,790,852,142 B); the registration
  text itself was not rewritten.
- **C5 — hypno_booth's S2c line.** CENSUS §9 (appended
  2026-11-16) still reads "S2c: NOT content-eligible" with
  no supersession note, although the 2026-11-17 S2c
  amendment postdates it; LAS's §8 entry, by contrast,
  carries an explicit supersession note. Whether the
  amendment reaches §9's line is not stated in §9.
- **C6 — Hachina's location (HAC-D2).** The registration
  places the tree in the RE tree; the full census entry
  records it under the remakes landing home. The census
  itself flags the divergence.
- **C7 — Detailed Cowgirl Simulator's archives.** Two
  archive sets were filed under its name on 2026-11-16; the
  genuine build is RJ01419389, and the RJ01580537 set's
  payload is Yandere Kamikakushi.
- **C8 — UnHolY DisAsTeR's listing.** Its censorship field
  reads "Uncensored" while its tag list also carries
  "Censored" — recorded as-is in the registration.
- **C9 — Vesper Isles' target law.** The D-1675 registration
  says "native Linux first per the Owner's word"; PIPELINE's
  dual-target law (D-1094) makes Linux + Android one
  deliverable, not done at Linux alone.
- **C10 — Part II records against themselves.** SERVERS.md
  vs the enhancement plan: tackle ("code has never landed"
  vs 0.3.2 published, smoke PASS); ache (1.0.0 / 10 tools vs
  1.2.0 / 11); neemu's surface (15 tools / 6,139 tok at
  1.0.0 vs a measured bare launch of 2 tools / 675 tok at
  1.0.1). The plan itself names SERVERS.md stale and
  incomplete (its Part 2) — the disagreements are recorded
  here as the subjects' first census findings.

**Gaps (what the sources do not contain):**

- **G1 — Teaching Feeling has no census entry.** Zero hits
  in CENSUS.md; its entire programme record is PIPELINE's
  amendment text (engine, rungs, and wave unstated).
- **G2 — Group rows without per-title rungs.** The Ren'Py
  remainder (dialogue trees for 10 of 12; the 12 unnamed,
  the uncovered 2 unnamed), the RPG Maker remainder, and
  the Hypnotic Eyes pair carry no per-title rung detail in
  the census.
- **G3 — Mindsweeper's engine** is not stated in its
  registration; engine identification was the planning
  lane's first job, and the appended planning record's rung
  order implies NScripter work without the census body
  naming the engine.
- **G4 — Stale S-rung assignments.** CENSUS §7 instructs
  that pre-rung S-rung assignments (IHSA class, Undercity,
  SoldGirl Town via the BQ-2 note, and kin) be re-walked
  before any wave brief commissions from them; §§3–4 were
  not rewritten to match §7.
- **G5 — Thin registrations.** Mari's Story ("intake census
  next"), NTR Idol ("pak census next"), Maeve's Academy
  (vocabulary only), Iris, Agent Mirai, and Goblin Burrow
  (listings only, intakes uncommissioned) carry no further
  record; several Population C titles (Aria, CIRCLEMATE)
  have no engine stated in the census sections.
- **G6 — The `ftbrain` name.** The Owner's 2026-11-16
  RJ423767 rename happened outside the RE tree; the title
  is present as Female_Teacher_Brainwashing_and_Training,
  and its RJ423767 identity is probable, not proven.
- **G7 — Vesper Isles' audio counts.** ~163 ogg streams at
  intake vs 160 indexed streams in the R1 record; the
  census does not reconcile the difference.
- **G8 — Part II's missing four.** soundings, sinfonia,
  brightwork, and diver mcp (as a server) appear in neither
  fleet record (§§6.8–6.11); pratique appears with no stated
  purpose, tool list, or launch line (§6.4); cartone has no
  SERVERS.md entry at all (§6.5).

*End of draft.*
