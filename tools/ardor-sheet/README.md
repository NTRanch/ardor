# ardor-sheet — sheet + delivery kit (TOOLING-PLAN T3)

One builder for the MoonDeep standard sheet deliveries, converging
the four hand-written `build_sheets.py` variants the 2026-11-27 art
lanes each re-derived (fresh, carry, and both full-colour carries),
and the per-delivery re-derivation of the chat-JPEG format after the
Owner's sheet-format ruling (2026-11-27 20:08 PT: chat deliveries are
~4800 px-wide JPEG q90; PNG masters stay on disk).

Layout constants are the fresh lane's — the layout the Tamsin
heightfix sheet also used: per-panel caption band on top of each
frame, DejaVu Sans-Bold, panel field (18,18,18), sheet field
(40,40,40).

## Usage

```
ardor-sheet row  --out NAME [--dir DIR] --panel SPEC [--panel SPEC ...]
ardor-sheet cast --out NAME [--dir DIR] --panel SPEC [--panel SPEC ...]
                    [--cols 4] [--scale 2] [--pad 12]

SPEC = PATH|CLASS|LABEL   (pipe-separated; paths must not contain '|')
```

- `row` — panels side by side at native res, 8 px gaps: the per-girl
  two-up / three-up sheet (base beside candidates).
- `cast` — panels upscaled `--scale`× inside the sheet, laid out in a
  `--cols`-wide grid with `--pad` px padding: the cast sheet.

Caption discipline (estate Rule 9) is built in: every panel must
carry a class and a label, and the tool refuses an uncaptioned panel
(exit 2, nothing written). The rendered caption is `LABEL - CLASS`,
or the label alone when the class already appears in it.

## Outputs (into `--dir`, default `.`)

| file | what |
|---|---|
| `NAME.png` | native-res PNG master sheet |
| `NAME-4800.jpg` | chat JPEG, 4800 px wide, q90 (the 20:08 ruling) |
| `NAME-phone.jpg` | phone JPEG, 2600 px wide, q90 |
| `NAME.json` | only with `--sidecar`: sidecar skeleton — per-panel path / sha256 / class / label / caption plus `base_pin` / `prompt` / `seed` / `grade` fields (null, for the lane to fill) and the outputs' dimensions |

Options: `--band N` / `--font-size N` override the caption band /
font size (defaults derive from scale: band = 30·scale + 24, font =
14·scale + 16 — scale 1 → 54/30, scale 2 → 84/44, the fresh lane's
two operating points). `--gap N` sets the row-mode gap (default 8).

Failure behaviour: all panels are validated (existence, caption,
bounds — max 64 panels, max 100 MP per panel) before anything is
built or written; a validation failure leaves the output directory
untouched.

## Gate evidence (2026-11-27)

Rebuilt with this kit and compared against the lanes' delivered
sheets (full record: `~/workspace/moondeep-tooling-build-20261127/
RECORD-T3.md`):

- Tamsin heightfix sheet: PNG master pixel-identical in all image
  regions; caption bands of panels 2–3 pixel-identical; panel 1
  differs only in caption text (see the record's named gap G1).
- Fresh cast sheet: PNG master pixel-identical over all
  67,992,784 px; both JPEG derivatives dimension-identical to the
  delivered files.

## Dependencies

Python 3 + Pillow. DejaVu Sans-Bold (falls back to PIL's default
font if absent — captions then render in the bitmap font; the gate
above ran with DejaVu present).
