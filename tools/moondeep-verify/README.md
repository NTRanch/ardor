# moondeep-verify

Freeze-hash / bind verifier for MoonDeep motion clips. Recomputes every
number from the artefacts themselves instead of trusting transcriptions —
the standing form of the hand verification done for the tranche-1
verdict packs (2026-11-27), where a 59-char truncated freeze hash
survived into two documents before recomputation caught it.

Python 3, stdlib only. No dependencies, no network.

## What it checks

**Freeze hash** (frozen `torogao-motion-clip/2` JSON, D-1397).
Canonicalisation (torogao-format h0):

    sha256( raw_file_text with the `,"freeze":{...}` block removed,
            trailing newline stripped )

The recomputed hash is compared against the `freeze.sha256` value
recorded inside the file's freeze block.

**Bind claims.** Given a `bind_report.json`, the twin and loop-closure
pixel diffs are recomputed from the frame PNGs beside it — channel-max
mask (rig-lessons L-37: pixels, never file md5; a pixel counts as
differing when any channel differs) — and compared against the claims
in the sibling `gates.json`. Without a claim, recomputed numbers are
reported as UNCLAIMED.

## Usage

    moondeep-verify PATH [PATH...]            files or dirs (recursive)
    moondeep-verify --claim HEX PATH...       also compare each recomputed
                                              freeze hash against an
                                              externally transcribed HEX
                                              (the document-checking mode)
    moondeep-verify --self-test CLIP BIND_DIR planted controls on scratch
                                              copies: one corrupted hash
                                              digit, one corrupted channel
                                              byte, one corrupted frame
                                              pixel — all must be caught

Exit codes: 0 = everything verified PASS · 1 = any MISMATCH ·
2 = unreadable / nothing verifiable in an explicitly named file.

## Gate record (2026-11-27)

- Tranche-1 corpus: **33/33** freeze hashes reproduce (28 admitted
  clips, both struck v1s, the Petra point v3 and Tamsin v2 swap-ins).
- Planted historical control: the truncated 59-char rite_salute hash
  is caught as MISMATCH via `--claim`; the correct hash passes.
- Synthetic corruptions: **3/3** caught (hash digit, channel byte,
  frame pixel +17 → recomputed diff of exactly 1 px, max_delta 17).
- Bind claims sweep: **64/64** (32 binds × twin + loop-closure)
  recompute to the claimed 0/0.

Full record: `~/workspace/moondeep-tooling-build-20261127/RECORD-T4.md`.
