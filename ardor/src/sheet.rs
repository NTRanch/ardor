//! Contact-sheet builder: the ardor-sheet kit, generalized.
//!
//! Panels (native-resolution stills) are composed into a sheet
//! where every panel carries a caption band naming it and its
//! Rule-9 class — a frame shown to anyone must say what it is
//! (estate REVIEW rule 9). Two modes: `row` (one horizontal
//! strip at native resolution) and `cast` (a grid of panels
//! upscaled by the preset's cast scale). Outputs are a PNG
//! master plus the chat-delivery JPEG derivatives the Owner
//! ruled 2026-11-27 (a wide q90 JPEG and a phone-width JPEG).
//!
//! Layout constants live in named per-game presets, never loose
//! numbers: the `moondeep` preset carries the values the
//! MoonDeep lanes gated on (RECORD-T3: rebuilds pixel-identical
//! to the delivered sheets, 0 differing px).

use std::path::{Path, PathBuf};

use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};

use crate::pngcodec::{decode_png_rgba, encode_png_rgb};
use crate::raster::{RgbImage, resize_lanczos};
use crate::text::{TextEngine, draw_caption};

/// A named per-game sheet preset: every layout constant the
/// builder uses.
#[derive(Clone, Copy, Debug)]
pub struct SheetPreset {
    pub name: &'static str,
    /// Chat-delivery JPEG width (the ~4800px q90 format).
    pub chat_jpeg_width: u32,
    /// Phone-delivery JPEG width.
    pub phone_jpeg_width: u32,
    pub jpeg_quality: u8,
    pub panel_field: [u8; 3],
    pub sheet_field: [u8; 3],
    pub caption_fill: [u8; 3],
    pub caption_x: i64,
    pub caption_y: i64,
    /// Caption band height = band_per_scale * scale + band_base.
    pub band_per_scale: i64,
    pub band_base: i64,
    /// Caption font px = font_per_scale * scale + font_base.
    pub font_per_scale: i64,
    pub font_base: i64,
    pub row_gap: i64,
    pub row_trailing_pad: i64,
    pub cast_cols: u32,
    pub cast_scale: u32,
    pub cast_pad: i64,
    pub max_panels: usize,
    pub max_panel_pixels: u64,
    pub font_candidates: &'static [&'static str],
}

/// The preset the MoonDeep lanes gated on (T3): dark fields,
/// DejaVu Sans-Bold captions, 4800px q90 + 2600px phone JPEGs.
pub const MOONDEEP_PRESET: SheetPreset = SheetPreset {
    name: "moondeep",
    chat_jpeg_width: 4800,
    phone_jpeg_width: 2600,
    jpeg_quality: 90,
    panel_field: [18, 18, 18],
    sheet_field: [40, 40, 40],
    caption_fill: [240, 240, 240],
    caption_x: 14,
    caption_y: 12,
    band_per_scale: 30,
    band_base: 24,
    font_per_scale: 14,
    font_base: 16,
    row_gap: 8,
    row_trailing_pad: 16,
    cast_cols: 4,
    cast_scale: 2,
    cast_pad: 12,
    max_panels: 64,
    max_panel_pixels: 100_000_000,
    font_candidates: &[
        "/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf",
        "/usr/share/fonts/TTF/DejaVuSans-Bold.ttf",
    ],
};

/// Look up a preset by name. Unknown names are an error, never a
/// silent fallback — a sheet built with the wrong game's
/// constants is a wrong sheet.
pub fn preset_by_name(name: &str) -> Option<SheetPreset> {
    match name {
        "moondeep" => Some(MOONDEEP_PRESET),
        _ => None,
    }
}

/// The preset names this build knows, for tool errors and docs.
pub fn preset_names() -> Vec<&'static str> {
    vec![MOONDEEP_PRESET.name]
}

/// One panel to place on the sheet.
pub struct PanelInput {
    pub path: String,
    pub class: String,
    pub label: String,
}

/// A sheet_build request (already shape-validated by the handler;
/// semantic validation happens here, before anything is written).
pub struct SheetRequest {
    pub mode: String, // "row" | "cast"
    pub panels: Vec<PanelInput>,
    pub out_name: String,
    pub out_dir: String,
    pub preset: SheetPreset,
    pub cols: Option<u32>,
    pub scale: Option<u32>,
    pub pad: Option<i64>,
    pub gap: Option<i64>,
    pub band: Option<i64>,
    pub font_size: Option<i64>,
    pub sidecar: bool,
}

/// A build refusal: invalid input, refused before any output
/// exists (the kit's exit-2 discipline). Maps to a stable
/// `sheet_refused` domain error at the handler.
pub struct SheetRefused(pub String);

impl From<std::io::Error> for SheetRefused {
    fn from(e: std::io::Error) -> SheetRefused {
        SheetRefused(format!("io: {e}"))
    }
}

impl From<serde_json::Error> for SheetRefused {
    fn from(e: serde_json::Error) -> SheetRefused {
        SheetRefused(format!("json: {e}"))
    }
}

/// The caption a panel carries: the class is appended unless the
/// label already names it (Rule 9: the class is always legible).
pub fn compose_caption(label: &str, class: &str) -> String {
    if label.to_lowercase().contains(&class.to_lowercase()) {
        label.to_string()
    } else {
        format!("{label} - {class}")
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

/// Decode panel bytes (PNG, or JPEG by magic bytes) to RGB.
fn load_panel_rgb(path: &Path, bytes: &[u8]) -> Result<RgbImage, SheetRefused> {
    if bytes.starts_with(b"\x89PNG") {
        let decoded = decode_png_rgba(bytes)
            .map_err(|e| SheetRefused(format!("cannot decode panel {}: {e}", path.display())))?;
        Ok(RgbImage::from_rgba(
            decoded.width,
            decoded.height,
            &decoded.rgba,
        ))
    } else if bytes.starts_with(&[0xFF, 0xD8]) {
        let mut decoder = zune_jpeg::JpegDecoder::new(std::io::Cursor::new(bytes));
        let pixels = decoder
            .decode()
            .map_err(|e| SheetRefused(format!("cannot decode panel {}: {e:?}", path.display())))?;
        let info = decoder.info().expect("info after decode");
        let (w, h) = (info.width as u32, info.height as u32);
        // zune-jpeg decodes to RGB by default for colour images;
        // grayscale yields one byte per pixel.
        if pixels.len() == (w * h * 3) as usize {
            Ok(RgbImage {
                width: w,
                height: h,
                px: pixels,
            })
        } else if pixels.len() == (w * h) as usize {
            let mut px = Vec::with_capacity((w * h * 3) as usize);
            for g in pixels {
                px.extend_from_slice(&[g, g, g]);
            }
            Ok(RgbImage {
                width: w,
                height: h,
                px,
            })
        } else {
            Err(SheetRefused(format!(
                "cannot decode panel {}: unexpected sample count {}",
                path.display(),
                pixels.len()
            )))
        }
    } else {
        Err(SheetRefused(format!(
            "panel {} is neither PNG nor JPEG",
            path.display()
        )))
    }
}

fn encode_jpeg(img: &RgbImage, quality: u8) -> Result<Vec<u8>, SheetRefused> {
    let mut out = Vec::new();
    let encoder = jpeg_encoder::Encoder::new(&mut out, quality);
    encoder
        .encode(
            &img.px,
            img.width as u16,
            img.height as u16,
            jpeg_encoder::ColorType::Rgb,
        )
        .map_err(|e| SheetRefused(format!("jpeg encode failed: {e}")))?;
    Ok(out)
}

/// Pillow's `int(h * target / w)` derivative height, in f64.
fn derivative_height(master: &RgbImage, target_width: u32) -> u32 {
    let h = (master.height as f64 * target_width as f64 / master.width as f64) as u32;
    h.max(1)
}

/// Build the sheet and its derivatives. All validation happens
/// before anything is built or written; a refused build leaves
/// no outputs behind.
pub fn build_sheet(req: &SheetRequest) -> Result<Value, SheetRefused> {
    let preset = req.preset;
    // ---- validation (Rule 9 + kit refusals) ----
    if req.panels.is_empty() {
        return Err(SheetRefused("spec has no panels".to_string()));
    }
    if req.panels.len() > preset.max_panels {
        return Err(SheetRefused(format!(
            "{} panels exceeds the {}-panel sheet limit",
            req.panels.len(),
            preset.max_panels
        )));
    }
    for p in &req.panels {
        if p.path.is_empty() {
            return Err(SheetRefused("a panel is missing its path".to_string()));
        }
        if p.class.is_empty() {
            return Err(SheetRefused(format!(
                "panel {} has no class: Rule 9 requires every panel to name its class",
                p.path
            )));
        }
        if p.label.is_empty() {
            return Err(SheetRefused(format!("panel {} has no label", p.path)));
        }
    }
    let scale = req.scale.unwrap_or(if req.mode == "cast" {
        preset.cast_scale
    } else {
        1
    });
    if scale < 1 {
        return Err(SheetRefused("scale must be >= 1".to_string()));
    }
    let cols = req.cols.unwrap_or(preset.cast_cols);
    if req.mode == "cast" && cols < 1 {
        return Err(SheetRefused("cols must be >= 1".to_string()));
    }
    let band = req
        .band
        .unwrap_or(preset.band_per_scale * scale as i64 + preset.band_base);
    let font_px = req
        .font_size
        .unwrap_or(preset.font_per_scale * scale as i64 + preset.font_base);

    // ---- load every panel before building anything ----
    let mut images: Vec<RgbImage> = Vec::with_capacity(req.panels.len());
    let mut source_sha: Vec<String> = Vec::with_capacity(req.panels.len());
    for p in &req.panels {
        let bytes = std::fs::read(&p.path)
            .map_err(|e| SheetRefused(format!("cannot read panel {}: {e}", p.path)))?;
        source_sha.push(sha256_hex(&bytes));
        let img = load_panel_rgb(Path::new(&p.path), &bytes)?;
        if img.width as u64 * img.height as u64 > preset.max_panel_pixels {
            return Err(SheetRefused(format!(
                "panel {} exceeds the 100MP guard",
                p.path
            )));
        }
        images.push(img);
    }

    // ---- font ----
    let font_path = preset
        .font_candidates
        .iter()
        .find(|c| Path::new(c).is_file())
        .ok_or_else(|| {
            SheetRefused(format!(
                "no caption font found (looked for {})",
                preset.font_candidates.join(", ")
            ))
        })?;
    let engine = TextEngine::new().map_err(SheetRefused)?;
    let face = engine.face(font_path, font_px).map_err(SheetRefused)?;

    // ---- build panels (image under its caption band) ----
    let mut panels: Vec<RgbImage> = Vec::with_capacity(images.len());
    for (img, p) in images.iter().zip(req.panels.iter()) {
        let scaled = if scale != 1 {
            resize_lanczos(img, img.width * scale, img.height * scale)
        } else {
            img.clone()
        };
        let mut panel = RgbImage::new_filled(
            scaled.width,
            scaled.height + band.max(0) as u32,
            preset.panel_field,
        );
        panel.paste(&scaled, 0, band);
        let caption = compose_caption(&p.label, &p.class);
        draw_caption(
            &face,
            &mut panel,
            preset.caption_x,
            preset.caption_y,
            &caption,
            preset.caption_fill,
        )
        .map_err(SheetRefused)?;
        panels.push(panel);
    }

    // ---- assemble ----
    let sheet = if req.mode == "cast" {
        let pad = req.pad.unwrap_or(preset.cast_pad);
        let cell_w = panels.iter().map(|p| p.width).max().unwrap_or(0) as i64;
        let cell_h = panels.iter().map(|p| p.height).max().unwrap_or(0) as i64;
        let rows = (panels.len() as i64 + cols as i64 - 1) / cols as i64;
        let width = cols as i64 * cell_w + (cols as i64 + 1) * pad;
        let height = rows * cell_h + (rows + 1) * pad;
        let mut sheet = RgbImage::new_filled(width as u32, height as u32, preset.sheet_field);
        for (i, panel) in panels.iter().enumerate() {
            let x = pad + (i as i64 % cols as i64) * (cell_w + pad);
            let y = pad + (i as i64 / cols as i64) * (cell_h + pad);
            sheet.paste(panel, x, y);
        }
        sheet
    } else {
        let gap = req.gap.unwrap_or(preset.row_gap) * scale as i64;
        let content_width: i64 =
            panels.iter().map(|p| p.width as i64).sum::<i64>() + gap * (panels.len() as i64 - 1);
        let width = (panels.iter().map(|p| p.width as i64).sum::<i64>() + preset.row_trailing_pad)
            .max(content_width);
        let height = panels.iter().map(|p| p.height).max().unwrap_or(0);
        let mut sheet = RgbImage::new_filled(width as u32, height, preset.sheet_field);
        let mut x = 0i64;
        for panel in &panels {
            sheet.paste(panel, x, 0);
            x += panel.width as i64 + gap;
        }
        sheet
    };

    // ---- write outputs ----
    let out_dir = PathBuf::from(&req.out_dir);
    std::fs::create_dir_all(&out_dir)?;
    let master_path = out_dir.join(format!("{}.png", req.out_name));
    let master_bytes = encode_png_rgb(sheet.width, sheet.height, &sheet.px);
    std::fs::write(&master_path, &master_bytes)?;

    let chat_w = preset.chat_jpeg_width;
    let chat = resize_lanczos(&sheet, chat_w, derivative_height(&sheet, chat_w));
    let chat_path = out_dir.join(format!("{}-{}.jpg", req.out_name, chat_w));
    let chat_bytes = encode_jpeg(&chat, preset.jpeg_quality)?;
    std::fs::write(&chat_path, &chat_bytes)?;

    let phone_w = preset.phone_jpeg_width;
    let phone = resize_lanczos(&sheet, phone_w, derivative_height(&sheet, phone_w));
    let phone_path = out_dir.join(format!("{}-phone.jpg", req.out_name));
    let phone_bytes = encode_jpeg(&phone, preset.jpeg_quality)?;
    std::fs::write(&phone_path, &phone_bytes)?;

    let output_entry = |path: &PathBuf, img: &RgbImage, bytes: &[u8]| {
        json!({
            "path": path.to_string_lossy(),
            "width": img.width,
            "height": img.height,
            "sha256": sha256_hex(bytes),
        })
    };
    let outputs = json!({
        "png": output_entry(&master_path, &sheet, &master_bytes),
        "jpeg_chat": output_entry(&chat_path, &chat, &chat_bytes),
        "jpeg_phone": output_entry(&phone_path, &phone, &phone_bytes),
    });

    // ---- optional sidecar (kit-compatible provenance record) ----
    let mut sidecar_path: Option<String> = None;
    if req.sidecar {
        let mut panel_records = Vec::new();
        for (p, sha) in req.panels.iter().zip(source_sha.iter()) {
            let mut rec = Map::new();
            rec.insert("path".to_string(), json!(p.path));
            rec.insert("sha256".to_string(), json!(sha));
            rec.insert("class".to_string(), json!(p.class));
            rec.insert("label".to_string(), json!(p.label));
            rec.insert(
                "caption".to_string(),
                json!(compose_caption(&p.label, &p.class)),
            );
            rec.insert("base_pin".to_string(), Value::Null);
            rec.insert("prompt".to_string(), Value::Null);
            rec.insert("seed".to_string(), Value::Null);
            rec.insert("grade".to_string(), Value::Null);
            panel_records.push(Value::Object(rec));
        }
        let mut out_map = Map::new();
        out_map.insert(
            "png".to_string(),
            json!({"path": master_path.to_string_lossy(), "width": sheet.width, "height": sheet.height}),
        );
        out_map.insert(
            format!("jpeg_{chat_w}"),
            json!({"path": chat_path.to_string_lossy(), "width": chat.width, "height": chat.height}),
        );
        out_map.insert(
            "jpeg_phone".to_string(),
            json!({"path": phone_path.to_string_lossy(), "width": phone.width, "height": phone.height}),
        );
        let sidecar = json!({
            "sheet": req.out_name,
            "mode": req.mode,
            "panels": panel_records,
            "outputs": out_map,
        });
        let path = out_dir.join(format!("{}.json", req.out_name));
        std::fs::write(&path, serde_json::to_string_pretty(&sidecar)? + "\n")?;
        sidecar_path = Some(path.to_string_lossy().to_string());
    }

    let panels_payload: Vec<Value> = req
        .panels
        .iter()
        .map(|p| {
            json!({
                "path": p.path,
                "class": p.class,
                "label": p.label,
                "caption": compose_caption(&p.label, &p.class),
            })
        })
        .collect();
    Ok(json!({
        "mode": req.mode,
        "preset": preset.name,
        "out": req.out_name,
        "dir": req.out_dir,
        "panels": panels_payload,
        "outputs": outputs,
        "sidecar": sidecar_path,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(panels: Vec<PanelInput>) -> SheetRequest {
        SheetRequest {
            mode: "row".to_string(),
            panels,
            out_name: "t".to_string(),
            out_dir: std::env::temp_dir().to_string_lossy().to_string(),
            preset: MOONDEEP_PRESET,
            cols: None,
            scale: None,
            pad: None,
            gap: None,
            band: None,
            font_size: None,
            sidecar: false,
        }
    }

    fn panel(path: &str, class: &str, label: &str) -> PanelInput {
        PanelInput {
            path: path.to_string(),
            class: class.to_string(),
            label: label.to_string(),
        }
    }

    #[test]
    fn caption_appends_class_when_absent() {
        assert_eq!(
            compose_caption("SOLENNE c1 - seed 274101", "CANDIDATE"),
            "SOLENNE c1 - seed 274101 - CANDIDATE"
        );
    }

    #[test]
    fn caption_keeps_label_when_class_named() {
        // Kit sidecar of record: label already naming the class
        // (any case) is used alone.
        assert_eq!(
            compose_caption("TAMSIN - DESIGN SOURCE c1", "DESIGN SOURCE"),
            "TAMSIN - DESIGN SOURCE c1"
        );
        assert_eq!(
            compose_caption("x - design source", "DESIGN SOURCE"),
            "x - design source"
        );
        // And the gated heightfix panel-1 pair appends, exactly as
        // the kit's sidecar records.
        assert_eq!(
            compose_caption("TAMSIN - SET-1 c1", "DESIGN SOURCE"),
            "TAMSIN - SET-1 c1 - DESIGN SOURCE"
        );
    }

    #[test]
    fn moondeep_preset_carries_gated_values() {
        let p = preset_by_name("moondeep").expect("preset");
        assert_eq!(
            (p.chat_jpeg_width, p.phone_jpeg_width, p.jpeg_quality),
            (4800, 2600, 90)
        );
        assert_eq!((p.cast_cols, p.cast_scale, p.cast_pad), (4, 2, 12));
        assert_eq!((p.band_per_scale, p.band_base), (30, 24));
        assert_eq!((p.font_per_scale, p.font_base), (14, 16));
    }

    #[test]
    fn unknown_preset_is_none_not_fallback() {
        // Adversarial: a misspelled preset must not silently build
        // with another game's constants.
        assert!(preset_by_name("moondep").is_none());
        assert!(preset_by_name("").is_none());
    }

    #[test]
    fn empty_class_refused_rule9() {
        // Adversarial: Rule 9 — a panel without a class is refused
        // before anything is built.
        let err = build_sheet(&request(vec![panel("/x.png", "", "LABEL")]))
            .expect_err("empty class must refuse");
        assert!(err.0.contains("Rule 9"), "{}", err.0);
    }

    #[test]
    fn empty_label_refused() {
        let err = build_sheet(&request(vec![panel("/x.png", "CANDIDATE", "")]))
            .expect_err("empty label must refuse");
        assert!(err.0.contains("no label"), "{}", err.0);
    }

    #[test]
    fn missing_panel_file_refused_by_name() {
        let err = build_sheet(&request(vec![panel(
            "/nonexistent/panel.png",
            "CANDIDATE",
            "LABEL",
        )]))
        .expect_err("missing file must refuse");
        assert!(err.0.contains("/nonexistent/panel.png"), "{}", err.0);
    }

    #[test]
    fn too_many_panels_refused() {
        let panels: Vec<PanelInput> = (0..65).map(|_| panel("/x.png", "C", "L")).collect();
        let err = build_sheet(&request(panels)).expect_err("65 panels must refuse");
        assert!(err.0.contains("exceeds"), "{}", err.0);
    }

    #[test]
    fn no_panels_refused() {
        let err = build_sheet(&request(vec![])).expect_err("empty spec must refuse");
        assert!(err.0.contains("no panels"), "{}", err.0);
    }

    #[test]
    fn derivative_height_truncates_like_python() {
        let master = RgbImage::new_filled(2512, 1270, [0, 0, 0]);
        assert_eq!(derivative_height(&master, 4800), 2426);
        assert_eq!(derivative_height(&master, 2600), 1314);
    }
}
