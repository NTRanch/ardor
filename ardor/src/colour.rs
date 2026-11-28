//! Colour measurement instruments (MAPPING M-2): the ardor-colour
//! CLI's patch / iris-annulus / pupil-disc measurements, generalized
//! from a per-game script into MCP tools (`colour_sample`,
//! `colour_drift`, `pupil_screen` in `tools.rs`).
//!
//! Generalization shape: everything MoonDeep-specific in the CLI —
//! the cast enum, the region geometry, the set names — is CONFIG
//! here (inline arguments, a presets document passed as an argument,
//! or a shipped preset; see [`builtin_preset`]). The mechanisms are
//! universal and replicate the CLI's published methods exactly, so
//! numbers stay comparable with the CLI's records (RECORD-T1):
//!
//! - patch: mean HSV (colorsys, per-pixel) of a square patch centred
//!   on the region coords; background corner = mean RGB of the
//!   top-left square. Out-of-frame patch pixels read as black (the
//!   PIL crop padding the CLI's method implies).
//! - iris: annulus (r_in..=r_out) around a centre, pixels with
//!   v_min < V < v_max kept; circular-median hue, median saturation,
//!   median R-B, mean RGB. The separation axis is saturation /
//!   blue-share, NOT hue (the Petra finding: every annulus reads a
//!   warm-grey ~20-24 degrees; seed 1 vs seed 2 separate on sat
//!   0.88 vs 0.42 and blue share 0.036 vs 0.141).
//! - pupil: disc of `disc_radius` at a centre; median V and the dark
//!   fraction (share of disc pixels with V <= dark_v_max). Verdict
//!   GLOWING iff EVERY eye's dark fraction <= dark_fraction_max AND
//!   the mean median-V >= mean_median_val_min — the dark-core
//!   extinction + brightness conjunction calibrated on the
//!   2026-11-27 gate set (RECORD-T1 gate 3). A screen, not a
//!   verdict: the Owner's eye decides.
//!
//! Drift is signed: hue delta wrapped to [-180, 180), plus d_sat /
//! d_val, and for iris readings d_rb + amber_ward (hue moved toward
//! the 35-degree candle-amber pole). Drift flags compare the rounded
//! deltas against configurable thresholds.
//!
//! Rounding matches the CLI (hue 1dp, sat/val 3dp, round-half-even)
//! so cross-tool tables compare. Two deliberate deviations from the
//! CLI, both edge cases that never occur on real frames: iris/pupil
//! pixels outside the frame are SKIPPED (the CLI's numpy indexing
//! silently wraps), and a pupil centre whose disc holds no in-frame
//! pixel is an error, not a NaN.

use std::collections::BTreeMap;

use image::RgbImage;
use serde_json::{Value, json};

/// The candle-amber pole for `amber_ward` (degrees).
pub const AMBER_POLE_DEG: f64 = 35.0;

/// Background corner square size in px (the CLI's corner method).
pub const CORNER_SIZE: i64 = 24;

/// One named patch region: a square of `size` px centred on (x, y).
#[derive(Clone, Debug)]
pub struct Region {
    pub name: String,
    pub x: i64,
    pub y: i64,
    pub size: u32,
}

/// Iris annulus geometry + pixel filter (CLI defaults).
#[derive(Clone, Copy, Debug)]
pub struct IrisRule {
    pub r_in: f64,
    pub r_out: f64,
    pub v_min: f64,
    pub v_max: f64,
    pub min_pixels: usize,
}

impl Default for IrisRule {
    fn default() -> Self {
        IrisRule {
            r_in: 2.5,
            r_out: 6.5,
            v_min: 0.12,
            v_max: 0.85,
            min_pixels: 5,
        }
    }
}

/// The pupil screen's rule constants (CLI defaults, calibrated on
/// the 2026-11-27 gate set — RECORD-T1 gate 3).
#[derive(Clone, Copy, Debug)]
pub struct PupilRule {
    pub disc_radius: i64,
    pub dark_v_max: f64,
    pub dark_fraction_max: f64,
    pub mean_median_val_min: f64,
}

impl Default for PupilRule {
    fn default() -> Self {
        PupilRule {
            disc_radius: 8,
            dark_v_max: 0.25,
            dark_fraction_max: 0.01,
            mean_median_val_min: 0.66,
        }
    }
}

/// Drift flag thresholds. `None` disables a flag. Defaults are the
/// 2026-11-27 screen values: hue — the full-colour lanes' named
/// defect line (>~15 degrees); sat — 0.25 sits below every flagged
/// candle-carry finding (smallest: Umbra hair +0.372) and above the
/// unflagged Sable garment (+0.137); val had no screen line, so it
/// ships disabled.
#[derive(Clone, Copy, Debug)]
pub struct DriftThresholds {
    pub warn_hue_deg: Option<f64>,
    pub warn_sat: Option<f64>,
    pub warn_val: Option<f64>,
}

impl Default for DriftThresholds {
    fn default() -> Self {
        DriftThresholds {
            warn_hue_deg: Some(15.0),
            warn_sat: Some(0.25),
            warn_val: None,
        }
    }
}

impl DriftThresholds {
    pub fn to_json(&self) -> Value {
        json!({
            "warn_hue_deg": self.warn_hue_deg,
            "warn_sat": self.warn_sat,
            "warn_val": self.warn_val,
        })
    }
}

// ---------- numeric helpers (CLI-exact) ----------

/// Round half-to-even at `dp` decimal places, matching Python's
/// `round` as the CLI applies it.
fn round_dp(v: f64, dp: u32) -> f64 {
    let m = 10f64.powi(dp as i32);
    (v * m).round_ties_even() / m
}

fn wrap_deg(d: f64) -> f64 {
    (d + 180.0).rem_euclid(360.0) - 180.0
}

/// colorsys.rgb_to_hsv, transcribed: inputs in 0..=1, hue out in
/// 0..1, sat/val in 0..=1.
fn rgb_to_hsv(r: f64, g: f64, b: f64) -> (f64, f64, f64) {
    let maxc = r.max(g).max(b);
    let minc = r.min(g).min(b);
    let v = maxc;
    if minc == maxc {
        return (0.0, 0.0, v);
    }
    let d = maxc - minc;
    let s = d / maxc;
    let rc = (maxc - r) / d;
    let gc = (maxc - g) / d;
    let bc = (maxc - b) / d;
    let h6 = if r == maxc {
        bc - gc
    } else if g == maxc {
        2.0 + rc - bc
    } else {
        4.0 + gc - rc
    };
    ((h6 / 6.0).rem_euclid(1.0), s, v)
}

/// numpy-style median (mean of the two middle values when even).
fn median(values: &mut [f64]) -> f64 {
    values.sort_by(|a, b| a.partial_cmp(b).expect("no NaNs in measurements"));
    let n = values.len();
    if n % 2 == 1 {
        values[n / 2]
    } else {
        (values[n / 2 - 1] + values[n / 2]) / 2.0
    }
}

/// Circular median in degrees (the CLI's circ_median_deg): the
/// sample angle minimizing total wrapped absolute deviation; first
/// strict minimum in sorted order wins.
fn circ_median_deg(angles: &[f64]) -> f64 {
    let mut sorted = angles.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).expect("no NaNs in measurements"));
    let mut best = sorted[0];
    let mut best_d = f64::INFINITY;
    for cand in &sorted {
        let d: f64 = sorted.iter().map(|a| wrap_deg(a - cand).abs()).sum();
        if d < best_d {
            best_d = d;
            best = *cand;
        }
    }
    best
}

// ---------- image access ----------

/// Load an image as 8-bit RGB. Any failure is a domain error string.
pub fn load_rgb(path: &str) -> Result<RgbImage, String> {
    image::open(path)
        .map(|im| im.to_rgb8())
        .map_err(|e| format!("cannot load image {path}: {e}"))
}

/// Pixel at (x, y) as 0..=1 floats; out-of-frame reads black (the
/// PIL crop padding the CLI's patch method implies).
fn px(img: &RgbImage, x: i64, y: i64) -> (f64, f64, f64) {
    if x < 0 || y < 0 || x >= img.width() as i64 || y >= img.height() as i64 {
        return (0.0, 0.0, 0.0);
    }
    let p = img.get_pixel(x as u32, y as u32);
    (
        p[0] as f64 / 255.0,
        p[1] as f64 / 255.0,
        p[2] as f64 / 255.0,
    )
}

fn px_in_frame(img: &RgbImage, x: i64, y: i64) -> Option<(f64, f64, f64)> {
    if x < 0 || y < 0 || x >= img.width() as i64 || y >= img.height() as i64 {
        None
    } else {
        Some(px(img, x, y))
    }
}

// ---------- measurements ----------

/// Mean HSV of the square patch centred on the region (CLI patch).
fn patch_hsv(img: &RgbImage, region: &Region) -> Value {
    let size = region.size as i64;
    let x0 = region.x - size / 2;
    let y0 = region.y - size / 2;
    let (mut h_sum, mut s_sum, mut v_sum) = (0.0f64, 0.0f64, 0.0f64);
    let mut n = 0f64;
    for y in y0..y0 + size {
        for x in x0..x0 + size {
            let (r, g, b) = px(img, x, y);
            let (h, s, v) = rgb_to_hsv(r, g, b);
            h_sum += h;
            s_sum += s;
            v_sum += v;
            n += 1.0;
        }
    }
    json!({
        "at": [region.x, region.y],
        "hue": round_dp(h_sum / n * 360.0, 1),
        "sat": round_dp(s_sum / n, 3),
        "val": round_dp(v_sum / n, 3),
    })
}

/// Mean RGB (0-255 scale) of the top-left `size` square (CLI corner).
fn corner_rgb(img: &RgbImage, size: i64) -> Value {
    let (mut r_sum, mut g_sum, mut b_sum) = (0.0f64, 0.0f64, 0.0f64);
    let mut n = 0f64;
    for y in 0..size {
        for x in 0..size {
            let p = px(img, x, y);
            r_sum += p.0 * 255.0;
            g_sum += p.1 * 255.0;
            b_sum += p.2 * 255.0;
            n += 1.0;
        }
    }
    json!([
        round_dp(r_sum / n, 1),
        round_dp(g_sum / n, 1),
        round_dp(b_sum / n, 1)
    ])
}

/// Iris annulus reading at one centre (CLI iris). Returns the
/// short {at, n} form when too few pixels survive the V filter.
fn iris_annulus(img: &RgbImage, cx: i64, cy: i64, rule: &IrisRule) -> Value {
    let scan = rule.r_out.ceil() as i64;
    let mut kept: Vec<(f64, f64, f64)> = Vec::new();
    for dy in -scan..=scan {
        for dx in -scan..=scan {
            let d = ((dx * dx + dy * dy) as f64).sqrt();
            if d < rule.r_in || d > rule.r_out {
                continue;
            }
            if let Some(p) = px_in_frame(img, cx + dx, cy + dy) {
                let v = p.0.max(p.1).max(p.2);
                if v > rule.v_min && v < rule.v_max {
                    kept.push(p);
                }
            }
        }
    }
    let n = kept.len();
    if n < rule.min_pixels {
        return json!({"at": [cx, cy], "n": n});
    }
    let mut hues = Vec::with_capacity(n);
    let mut sats = Vec::with_capacity(n);
    let mut rbs = Vec::with_capacity(n);
    let (mut r_sum, mut g_sum, mut b_sum) = (0.0f64, 0.0f64, 0.0f64);
    for (r, g, b) in &kept {
        let (h, s, _v) = rgb_to_hsv(*r, *g, *b);
        hues.push(h * 360.0);
        sats.push(s);
        rbs.push(r - b);
        r_sum += r;
        g_sum += g;
        b_sum += b;
    }
    let nf = n as f64;
    json!({
        "at": [cx, cy],
        "n": n,
        "hue": round_dp(circ_median_deg(&hues), 1),
        "sat": round_dp(median(&mut sats), 3),
        "rb": round_dp(median(&mut rbs), 3),
        "rgb": [
            round_dp(r_sum / nf, 3),
            round_dp(g_sum / nf, 3),
            round_dp(b_sum / nf, 3)
        ],
    })
}

/// Pupil disc reading at one centre (CLI pupil). Err when the disc
/// holds no in-frame pixel.
fn pupil_disc(img: &RgbImage, cx: i64, cy: i64, rule: &PupilRule) -> Result<Value, String> {
    let r = rule.disc_radius;
    let mut vs: Vec<f64> = Vec::new();
    for dy in -r..=r {
        for dx in -r..=r {
            if dx * dx + dy * dy > r * r {
                continue;
            }
            if let Some(p) = px_in_frame(img, cx + dx, cy + dy) {
                vs.push(p.0.max(p.1).max(p.2));
            }
        }
    }
    if vs.is_empty() {
        return Err(format!(
            "pupil centre ({cx}, {cy}) has no in-frame pixels in this image"
        ));
    }
    let dark = vs.iter().filter(|v| **v <= rule.dark_v_max).count();
    let dark_fraction = dark as f64 / vs.len() as f64;
    Ok(json!({
        "at": [cx, cy],
        "median_val": round_dp(median(&mut vs), 3),
        "dark_fraction": round_dp(dark_fraction, 3),
        "face_crop": [cx - 30, cy - 24, cx + 30, cy + 24],
    }))
}

/// The screen verdict over rounded per-eye readings (CLI rule).
fn pupil_verdict(eyes: &[Value], rule: &PupilRule) -> Value {
    if eyes.is_empty() {
        return Value::Null;
    }
    let all_dark_gone = eyes
        .iter()
        .all(|e| e["dark_fraction"].as_f64().unwrap_or(1.0) <= rule.dark_fraction_max);
    let mean_med = eyes
        .iter()
        .map(|e| e["median_val"].as_f64().unwrap_or(0.0))
        .sum::<f64>()
        / eyes.len() as f64;
    if all_dark_gone && mean_med >= rule.mean_median_val_min {
        json!("GLOWING")
    } else {
        json!("ok")
    }
}

/// Signed drift between two readings (CLI drift_of) + flags.
fn drift_of(base: &Value, cand: &Value, thresholds: &DriftThresholds) -> Value {
    let d_hue = round_dp(
        wrap_deg(cand["hue"].as_f64().unwrap() - base["hue"].as_f64().unwrap()),
        1,
    );
    let d_sat = round_dp(
        cand["sat"].as_f64().unwrap() - base["sat"].as_f64().unwrap(),
        3,
    );
    let mut out = json!({"d_hue": d_hue, "d_sat": d_sat});
    let mut flags: Vec<&str> = Vec::new();
    if let Some(w) = thresholds.warn_hue_deg
        && d_hue.abs() >= w
    {
        flags.push("hue_drift");
    }
    if let Some(w) = thresholds.warn_sat
        && d_sat.abs() >= w
    {
        flags.push("sat_drift");
    }
    if base.get("val").is_some() && cand.get("val").is_some() {
        let d_val = round_dp(
            cand["val"].as_f64().unwrap() - base["val"].as_f64().unwrap(),
            3,
        );
        out["d_val"] = json!(d_val);
        if let Some(w) = thresholds.warn_val
            && d_val.abs() >= w
        {
            flags.push("val_drift");
        }
    }
    if base.get("rb").is_some() && cand.get("rb").is_some() {
        let d_rb = round_dp(
            cand["rb"].as_f64().unwrap() - base["rb"].as_f64().unwrap(),
            3,
        );
        out["d_rb"] = json!(d_rb);
        let to_b = wrap_deg(AMBER_POLE_DEG - base["hue"].as_f64().unwrap()).abs();
        let to_c = wrap_deg(AMBER_POLE_DEG - cand["hue"].as_f64().unwrap()).abs();
        out["amber_ward"] = json!(to_c < to_b);
    }
    out["flags"] = json!(flags);
    out
}

// ---------- public tool operations ----------

/// The resolved measurement geometry for one call.
pub struct Geometry {
    pub regions: Vec<Region>,
    pub iris_centres: Vec<(i64, i64)>,
    pub pupil_centres: Vec<(i64, i64)>,
    /// How the geometry was resolved: "inline", "config", or the
    /// preset name — echoed in outputs for provenance.
    pub source: String,
    pub character: Option<String>,
    pub patch_set: Option<String>,
    pub iris_set: Option<String>,
    pub pupil_set: Option<String>,
}

/// `colour_sample`: corner + region patches + iris annuli for one
/// frame.
pub fn sample(img: &RgbImage, image_path: &str, geom: &Geometry, iris_rule: &IrisRule) -> Value {
    let mut regions = serde_json::Map::new();
    for region in &geom.regions {
        regions.insert(region.name.clone(), patch_hsv(img, region));
    }
    let iris: Vec<Value> = geom
        .iris_centres
        .iter()
        .map(|(cx, cy)| iris_annulus(img, *cx, *cy, iris_rule))
        .collect();
    json!({
        "tool": "ardor-colour",
        "version": 1,
        "mode": "sample",
        "image": image_path,
        "config_source": geom.source,
        "character": geom.character,
        "patch_set": geom.patch_set,
        "iris_set": geom.iris_set,
        "background_corner_rgb": corner_rgb(img, CORNER_SIZE),
        "regions": regions,
        "iris": iris,
    })
}

/// `colour_drift`: sample both frames, report signed per-region and
/// per-iris drift with threshold flags.
pub fn drift(
    base: &RgbImage,
    base_path: &str,
    cand: &RgbImage,
    cand_path: &str,
    geom: &Geometry,
    iris_rule: &IrisRule,
    thresholds: &DriftThresholds,
) -> Value {
    let measure = |img: &RgbImage| {
        let mut regions = serde_json::Map::new();
        for region in &geom.regions {
            regions.insert(region.name.clone(), patch_hsv(img, region));
        }
        let iris: Vec<Value> = geom
            .iris_centres
            .iter()
            .map(|(cx, cy)| iris_annulus(img, *cx, *cy, iris_rule))
            .collect();
        (corner_rgb(img, CORNER_SIZE), regions, iris)
    };
    let (b_corner, b_regions, b_iris) = measure(base);
    let (c_corner, c_regions, c_iris) = measure(cand);
    let mut regions = serde_json::Map::new();
    for (name, b) in &b_regions {
        if let Some(c) = c_regions.get(name) {
            regions.insert(
                name.clone(),
                json!({
                    "base": b,
                    "candidate": c,
                    "drift": drift_of(b, c, thresholds),
                }),
            );
        }
    }
    let mut iris = Vec::new();
    for (b, c) in b_iris.iter().zip(c_iris.iter()) {
        let mut entry = json!({"base": b, "candidate": c});
        if b.get("hue").is_some() && c.get("hue").is_some() {
            entry["drift"] = drift_of(b, c, thresholds);
        }
        iris.push(entry);
    }
    json!({
        "tool": "ardor-colour",
        "version": 1,
        "mode": "drift",
        "base": base_path,
        "candidate": cand_path,
        "config_source": geom.source,
        "character": geom.character,
        "patch_set": geom.patch_set,
        "iris_set": geom.iris_set,
        "background_corner_rgb": {"base": b_corner, "candidate": c_corner},
        "regions": regions,
        "iris": iris,
        "thresholds": thresholds.to_json(),
    })
}

/// `pupil_screen`: per-eye disc readings + the GLOWING/ok verdict.
pub fn pupil(
    img: &RgbImage,
    image_path: &str,
    geom: &Geometry,
    rule: &PupilRule,
) -> Result<Value, String> {
    let mut eyes = Vec::new();
    for (cx, cy) in &geom.pupil_centres {
        eyes.push(pupil_disc(img, *cx, *cy, rule)?);
    }
    Ok(json!({
        "tool": "ardor-colour",
        "version": 1,
        "mode": "pupil",
        "image": image_path,
        "config_source": geom.source,
        "character": geom.character,
        "pupil_set": geom.pupil_set,
        "eyes": eyes,
        "verdict": pupil_verdict(&eyes, rule),
        "rule": {
            "disc_radius": rule.disc_radius,
            "dark_v_max": rule.dark_v_max,
            "dark_fraction_max": rule.dark_fraction_max,
            "mean_median_val_min": rule.mean_median_val_min,
        },
    }))
}

// ---------- per-game config (presets documents) ----------

/// One character's preset geometry: named patch sets, iris sets,
/// pupil sets — the CLI presets.json shape, generalized.
#[derive(Default)]
pub struct CharacterPresets {
    pub patches: BTreeMap<String, Vec<Region>>,
    pub iris: BTreeMap<String, Vec<(i64, i64)>>,
    pub pupil: BTreeMap<String, Vec<(i64, i64)>>,
}

/// A parsed presets document: characters by name. The document's
/// top-level cast key is `characters`; the ardor-colour CLI's
/// `girls` key is accepted for back-compat.
pub struct PresetDoc {
    pub characters: BTreeMap<String, CharacterPresets>,
}

fn parse_centre(v: &Value) -> Option<(i64, i64)> {
    let a = v.as_array()?;
    if a.len() != 2 {
        return None;
    }
    Some((a[0].as_i64()?, a[1].as_i64()?))
}

fn parse_centres(v: &Value) -> Result<Vec<(i64, i64)>, String> {
    let arr = v
        .as_array()
        .ok_or("preset centres must be an array of [x, y] pairs")?;
    arr.iter()
        .map(|c| parse_centre(c).ok_or_else(|| "preset centre must be [x, y] integers".to_string()))
        .collect()
}

pub fn parse_preset_doc(doc: &Value) -> Result<PresetDoc, String> {
    let root = doc
        .as_object()
        .ok_or("presets document must be a JSON object")?;
    let cast = root
        .get("characters")
        .or_else(|| root.get("girls"))
        .and_then(Value::as_object)
        .ok_or("presets document needs a `characters` object")?;
    let mut characters = BTreeMap::new();
    for (name, entry) in cast {
        let entry = entry
            .as_object()
            .ok_or_else(|| format!("preset character {name} must be an object"))?;
        let mut cp = CharacterPresets::default();
        if let Some(patches) = entry.get("patches").and_then(Value::as_object) {
            for (set, regions) in patches {
                let regions = regions
                    .as_object()
                    .ok_or_else(|| format!("patch set {set} for {name} must be an object"))?;
                let mut list = Vec::new();
                for (region_name, at) in regions {
                    let (x, y) = parse_centre(at)
                        .ok_or_else(|| format!("patch {region_name} for {name} must be [x, y]"))?;
                    list.push(Region {
                        name: region_name.clone(),
                        x,
                        y,
                        size: 48,
                    });
                }
                cp.patches.insert(set.clone(), list);
            }
        }
        if let Some(iris) = entry.get("iris").and_then(Value::as_object) {
            for (set, centres) in iris {
                cp.iris.insert(set.clone(), parse_centres(centres)?);
            }
        }
        if let Some(pupil) = entry.get("pupil").and_then(Value::as_object) {
            for (set, centres) in pupil {
                cp.pupil.insert(set.clone(), parse_centres(centres)?);
            }
        }
        characters.insert(name.clone(), cp);
    }
    Ok(PresetDoc { characters })
}

/// Presets shipped with the server, by game name. Each is a config
/// file under `ardor/config/` — a new game adds a file + a line
/// here; nothing game-specific lives in code.
pub fn builtin_preset(game: &str) -> Option<Result<PresetDoc, String>> {
    let text = match game {
        "moondeep" => include_str!("../config/moondeep-colour-presets.json"),
        _ => return None,
    };
    Some(
        serde_json::from_str(text)
            .map_err(|e| format!("built-in preset {game} does not parse: {e}"))
            .and_then(|doc| parse_preset_doc(&doc)),
    )
}

pub fn builtin_preset_names() -> &'static [&'static str] {
    &["moondeep"]
}

/// Resolve the geometry for one call from a preset document +
/// character + set selectors (CLI defaults: patches "judged",
/// iris "c1", pupil "c1" with fallback to "c1" when a named pupil
/// set is absent; a missing patch/iris set yields empty geometry,
/// as the CLI's `.get(set, {})` did).
pub fn geometry_from_presets(
    doc: &PresetDoc,
    source: String,
    character: &str,
    patch_set: Option<&str>,
    iris_set: Option<&str>,
    pupil_set: Option<&str>,
) -> Result<Geometry, String> {
    let cp = doc
        .characters
        .get(character)
        .ok_or_else(|| format!("unknown character in presets: {character}"))?;
    let patch_set = patch_set.unwrap_or("judged");
    let iris_set = iris_set.unwrap_or("c1");
    let pupil_set = pupil_set.unwrap_or("c1");
    let pupil_centres = cp
        .pupil
        .get(pupil_set)
        .or_else(|| cp.pupil.get("c1"))
        .cloned()
        .unwrap_or_default();
    Ok(Geometry {
        regions: cp.patches.get(patch_set).cloned().unwrap_or_default(),
        iris_centres: cp.iris.get(iris_set).cloned().unwrap_or_default(),
        pupil_centres,
        source,
        character: Some(character.to_string()),
        patch_set: Some(patch_set.to_string()),
        iris_set: Some(iris_set.to_string()),
        pupil_set: Some(pupil_set.to_string()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn solid(w: u32, h: u32, rgb: [u8; 3]) -> RgbImage {
        RgbImage::from_pixel(w, h, image::Rgb(rgb))
    }

    #[test]
    fn hsv_matches_colorsys_reference_values() {
        // Pure primaries + a mixed tone, computed via colorsys.
        let cases = [
            ((1.0, 0.0, 0.0), (0.0, 1.0, 1.0)),
            ((0.0, 1.0, 0.0), (120.0, 1.0, 1.0)),
            ((0.0, 0.0, 1.0), (240.0, 1.0, 1.0)),
            ((0.2, 0.4, 0.6), (210.0, 2.0 / 3.0, 0.6)),
            ((0.5, 0.5, 0.5), (0.0, 0.0, 0.5)),
        ];
        for ((r, g, b), (h_deg, s, v)) in cases {
            let (h, ss, vv) = rgb_to_hsv(r, g, b);
            assert!((h * 360.0 - h_deg).abs() < 1e-9, "hue for ({r}, {g}, {b})");
            assert!((ss - s).abs() < 1e-9, "sat for ({r}, {g}, {b})");
            assert!((vv - v).abs() < 1e-9, "val for ({r}, {g}, {b})");
        }
    }

    #[test]
    fn wrap_deg_wraps_to_half_open_range() {
        assert_eq!(wrap_deg(190.0), -170.0);
        assert_eq!(wrap_deg(-190.0), 170.0);
        assert_eq!(wrap_deg(180.0), -180.0);
        // The Solenne hair delta from hue-drift.json, after the
        // 1dp rounding drift_of applies (raw f64 wrap is 157.6000…2).
        assert_eq!(round_dp(wrap_deg(26.4 - 228.8), 1), 157.6);
    }

    #[test]
    fn circ_median_picks_the_cluster_centre() {
        // 10 and 11 tie at total deviation 173 (the 200 outlier is
        // 170+ away from both); the CLI takes the FIRST strict
        // minimum in sorted order, and so does this port.
        assert_eq!(circ_median_deg(&[10.0, 12.0, 11.0, 200.0]), 10.0);
        assert_eq!(circ_median_deg(&[350.0, 10.0, 0.0]), 0.0);
    }

    #[test]
    fn patch_of_solid_colour_is_exact() {
        let img = solid(64, 64, [51, 102, 153]);
        let region = Region {
            name: "x".into(),
            x: 32,
            y: 32,
            size: 48,
        };
        let reading = patch_hsv(&img, &region);
        assert_eq!(reading["hue"], json!(210.0));
        assert_eq!(reading["sat"], json!(0.667));
        assert_eq!(reading["val"], json!(0.6));
    }

    #[test]
    fn pupil_positive_control_blown_out_disc_flags_glowing() {
        // The planted error the screen exists to catch: a pupil
        // disc with NO dark core at all.
        let img = solid(64, 64, [240, 240, 240]);
        let rule = PupilRule::default();
        let eye = pupil_disc(&img, 32, 32, &rule).unwrap();
        assert_eq!(eye["dark_fraction"], json!(0.0));
        assert_eq!(pupil_verdict(&[eye], &rule), json!("GLOWING"));
    }

    #[test]
    fn pupil_dark_core_passes() {
        let img = solid(64, 64, [10, 10, 10]);
        let rule = PupilRule::default();
        let eye = pupil_disc(&img, 32, 32, &rule).unwrap();
        assert_eq!(eye["dark_fraction"], json!(1.0));
        assert_eq!(pupil_verdict(&[eye], &rule), json!("ok"));
    }

    #[test]
    fn pupil_verdict_needs_every_eye_dark_gone() {
        // Wrenna's calibration case in miniature: one bright eye
        // with a surviving dark fraction saves the pair.
        let rule = PupilRule::default();
        let bright_thin = json!({"median_val": 0.9, "dark_fraction": 0.01});
        let dark_core = json!({"median_val": 0.9, "dark_fraction": 0.269});
        assert_eq!(pupil_verdict(&[bright_thin, dark_core], &rule), json!("ok"));
    }

    #[test]
    fn drift_flags_fire_at_thresholds() {
        let base = json!({"hue": 72.2, "sat": 0.061, "val": 0.524});
        let cand = json!({"hue": 23.7, "sat": 0.55, "val": 0.386});
        let d = drift_of(&base, &cand, &DriftThresholds::default());
        assert_eq!(d["d_hue"], json!(-48.5));
        assert_eq!(d["d_sat"], json!(0.489));
        assert_eq!(d["flags"], json!(["hue_drift", "sat_drift"]));
    }
}
