# ardor-colour

The estate's one colour/drift measurement tool for MoonDeep portrait
work. Consolidates the four colour samplers written by hand on
2026-11-27 (the judged-picks candle carry's 48×48-patch method, the
C1 full-colour lane's iris-mode `sampler.py` + `measure*.py`, the
judged-picks full-colour lane's `drift_sampler.py`, and the fresh-c2
lane's `measure.py`) so that drift numbers are comparable across
sets — the portrait lock is a cross-set comparison and needs one
ruler. Build record and gate evidence: `RECORD-T1.md` in
`~/workspace/moondeep-tooling-build-2026-11-27/` (estate workspace).

## Usage

```
python3 ardor-colour sample  GIRL IMAGE [--patches SET] [--iris SET]
python3 ardor-colour drift   GIRL BASE CANDIDATE [--patches SET] [--iris SET]
python3 ardor-colour pupil   GIRL IMAGE [--set SET] [--centre X Y]...
python3 ardor-colour presets
```

- `GIRL`: solenne, umbra, wrenna, petra, sable, tamsin, vesper.
- `sample` — background corner RGB, per-region patch HSV, and iris
  annulus readings for one frame, as one JSON object.
- `drift` — the same for a base/candidate pair plus per-region signed
  deltas (hue wrapped to [-180,180), d_sat, d_val; iris adds d_rb and
  `amber_ward` — whether the iris hue moved toward the 35°
  candle-amber pole).
- `pupil` — the Owner's 2026-11-27 glowing-pupil screen: per-eye
  pupil-disc statistics (median value, dark-core fraction, face-crop
  box for the judging-size crop) and a per-girl verdict. `GLOWING`
  iff every eye's dark fraction ≤ 0.01 and the mean median-V ≥ 0.66 —
  a blown-out pupil is bright across the whole disc with no dark core
  left. It is a screen, not a verdict; the Owner's eye decides.
- `presets` — dump the preset table.

Preset sets (all coordinates in 832×1216 frame space, in
`presets.json` beside the script): patch sets `judged` (the
judged-picks lane's published coords) and `fresh` (the fresh-c2
lane's signature patches); iris sets `c1` (set-1 geometry) and
`fresh`; pupil centres default to the `c1` iris centres, with Sable
additionally carrying `judged` centres from the full-colour lane's
eye-mask geometry.

## Methods

- **patch** — mean HSV (colorsys, per-pixel) of a 48×48 patch centred
  on the preset coords; background corner = mean RGB of the 24×24
  top-left corner. Reproduces the candle lane's published
  `hue-drift.json` exactly (Vesper silver saturation 0.061 → 0.550,
  Tamsin dress hue 35.8° → 26.8°, Solenne against her judged base).
- **iris** — annulus (2.5–6.5 px) around the pupil centre, pixels
  with 0.12 < V < 0.85 kept; circular-median hue, median saturation,
  median R−B. On Petra this reproduces the C1 lane's finding —
  candle seed 1 amber-drifted, seed 2 grey-held — on the saturation /
  blue-share axis (seed 1 annulus sat ≈ 0.71–0.89 vs seed 2 ≈
  0.43–0.51 at the same centre); hue alone does not discriminate
  there, and the tool does not claim it does.
- **pupil** — disc r=8 statistics as above; calibrated on the
  2026-11-27 gate set (details and margins in the build record).

Dependencies: Python 3 stdlib + numpy + Pillow.
