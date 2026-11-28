//! Dispatch-level tests for the M-2 colour tools (colour_sample,
//! colour_drift, pupil_screen): roughly half validation (exact
//! numbers on synthetic frames, preset resolution, the planted
//! blown-out pupil the screen must catch), half adversarial
//! (missing/corrupt images, unknown arguments, unknown presets,
//! out-of-frame geometry — errors, never panics).

use std::path::{Path, PathBuf};

use ardor::store::Store;
use ardor::tools::{ToolFailure, dispatch};
use image::{Rgb, RgbImage};
use serde_json::{Value, json};

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("ardor-colour-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn store(dir: &Path) -> Store {
    Store::open(dir).unwrap()
}

fn solid_png(dir: &Path, name: &str, w: u32, h: u32, rgb: [u8; 3]) -> String {
    let path = dir.join(name);
    RgbImage::from_pixel(w, h, Rgb(rgb)).save(&path).unwrap();
    path.to_string_lossy().into_owned()
}

fn call(store: &mut Store, tool: &str, args: Value) -> Result<Value, ToolFailure> {
    dispatch(store, tool, &args)
}

/// ToolFailure has no Debug impl, so unwrap/unwrap_err are
/// unavailable on dispatch results; unwrap by hand.
fn ok(r: Result<Value, ToolFailure>) -> Value {
    match r {
        Ok(v) => v,
        Err(e) => panic!("tool failed [{}]: {}", e.code, e.message),
    }
}

fn err(r: Result<Value, ToolFailure>) -> ToolFailure {
    match r {
        Ok(v) => panic!("tool unexpectedly succeeded: {v}"),
        Err(e) => e,
    }
}

// ---------- validation ----------

#[test]
fn sample_solid_patch_and_corner_are_exact() {
    let dir = scratch("sample");
    let mut store = store(&dir);
    let img = solid_png(&dir, "solid.png", 64, 64, [51, 102, 153]);
    let out = call(
        &mut store,
        "colour_sample",
        json!({
            "image": img,
            "regions": [{"name": "robe", "at": [32, 32]}],
        }),
    );
    let out = ok(out);
    assert_eq!(out["mode"], json!("sample"));
    assert_eq!(out["regions"]["robe"]["hue"], json!(210.0));
    assert_eq!(out["regions"]["robe"]["sat"], json!(0.667));
    assert_eq!(out["regions"]["robe"]["val"], json!(0.6));
    assert_eq!(out["background_corner_rgb"], json!([51.0, 102.0, 153.0]));
}

#[test]
fn drift_between_solids_is_signed_and_flagged() {
    let dir = scratch("drift");
    let mut store = store(&dir);
    let base = solid_png(&dir, "base.png", 64, 64, [255, 0, 0]);
    let cand = solid_png(&dir, "cand.png", 64, 64, [0, 0, 255]);
    let out = call(
        &mut store,
        "colour_drift",
        json!({
            "base": base,
            "candidate": cand,
            "regions": [{"name": "field", "at": [32, 32]}],
        }),
    );
    let out = ok(out);
    let d = &out["regions"]["field"]["drift"];
    assert_eq!(d["d_hue"], json!(-120.0));
    assert_eq!(d["d_sat"], json!(0.0));
    assert_eq!(d["d_val"], json!(0.0));
    assert_eq!(d["flags"], json!(["hue_drift"]));
}

#[test]
fn pupil_screen_flags_planted_blown_out_pupil() {
    // Positive control: a pupil with no dark core at all MUST flag.
    let dir = scratch("pupil-pos");
    let mut store = store(&dir);
    let img = solid_png(&dir, "bright.png", 64, 64, [240, 240, 240]);
    let out = call(
        &mut store,
        "pupil_screen",
        json!({"image": img, "pupil_centres": [[32, 32]]}),
    );
    let out = ok(out);
    assert_eq!(out["eyes"][0]["dark_fraction"], json!(0.0));
    assert_eq!(out["verdict"], json!("GLOWING"));
}

#[test]
fn pupil_screen_passes_dark_pupil() {
    let dir = scratch("pupil-neg");
    let mut store = store(&dir);
    let img = solid_png(&dir, "dark.png", 64, 64, [10, 10, 10]);
    let out = call(
        &mut store,
        "pupil_screen",
        json!({"image": img, "pupil_centres": [[32, 32]]}),
    );
    let out = ok(out);
    assert_eq!(out["eyes"][0]["dark_fraction"], json!(1.0));
    assert_eq!(out["verdict"], json!("ok"));
}

#[test]
fn moondeep_preset_resolves_cast_geometry() {
    let dir = scratch("preset");
    let mut store = store(&dir);
    let img = solid_png(&dir, "frame.png", 832, 1216, [8, 8, 8]);
    let out = call(
        &mut store,
        "pupil_screen",
        json!({"image": img, "preset": "moondeep", "character": "sable"}),
    );
    let out = ok(out);
    assert_eq!(out["eyes"][0]["at"], json!([387, 238]));
    assert_eq!(out["eyes"][1]["at"], json!([441, 228]));
    assert_eq!(out["verdict"], json!("ok"));

    let out = call(
        &mut store,
        "colour_sample",
        json!({"image": img, "preset": "moondeep", "character": "vesper"}),
    );
    let out = ok(out);
    assert!(out["regions"].get("armour_silver").is_some());
    assert!(out["regions"].get("hair").is_some());
    assert_eq!(out["iris"].as_array().unwrap().len(), 2);
}

// ---------- adversarial ----------

#[test]
fn missing_image_is_a_domain_error_not_a_params_error() {
    let dir = scratch("missing");
    let mut store = store(&dir);
    let err = call(
        &mut store,
        "colour_sample",
        json!({"image": "/nonexistent/frame.png"}),
    )
    .unwrap_err();
    assert!(!err.is_params);
    assert_eq!(err.code, "colour_error");
}

#[test]
fn corrupt_image_is_a_domain_error() {
    let dir = scratch("corrupt");
    let mut store = store(&dir);
    let path = dir.join("bad.png");
    std::fs::write(&path, b"this is not an image").unwrap();
    let err = call(
        &mut store,
        "pupil_screen",
        json!({
            "image": path.to_string_lossy(),
            "pupil_centres": [[4, 4]],
        }),
    )
    .unwrap_err();
    assert!(!err.is_params);
    assert_eq!(err.code, "colour_error");
}

#[test]
fn unknown_argument_is_a_params_error() {
    let dir = scratch("unknown-arg");
    let mut store = store(&dir);
    let img = solid_png(&dir, "x.png", 16, 16, [0, 0, 0]);
    let err = call(
        &mut store,
        "colour_sample",
        json!({"image": img, "bogus": 1}),
    )
    .unwrap_err();
    assert!(err.is_params);
}

#[test]
fn unknown_preset_and_character_are_params_errors() {
    let dir = scratch("unknown-preset");
    let mut store = store(&dir);
    let img = solid_png(&dir, "x.png", 16, 16, [0, 0, 0]);
    let err = call(
        &mut store,
        "pupil_screen",
        json!({"image": img, "preset": "nope", "character": "sable"}),
    )
    .unwrap_err();
    assert!(err.is_params);
    let err = call(
        &mut store,
        "pupil_screen",
        json!({"image": img, "preset": "moondeep", "character": "nope"}),
    )
    .unwrap_err();
    assert!(err.is_params);
}

#[test]
fn pupil_centre_outside_frame_is_a_domain_error_not_a_panic() {
    let dir = scratch("outside");
    let mut store = store(&dir);
    let img = solid_png(&dir, "x.png", 64, 64, [0, 0, 0]);
    let err = call(
        &mut store,
        "pupil_screen",
        json!({"image": img, "pupil_centres": [[-500, -500]]}),
    )
    .unwrap_err();
    assert!(!err.is_params);
    assert_eq!(err.code, "colour_error");
}

#[test]
fn malformed_regions_are_params_errors() {
    let dir = scratch("malformed");
    let mut store = store(&dir);
    let img = solid_png(&dir, "x.png", 64, 64, [0, 0, 0]);
    for regions in [
        json!([{"name": "x"}]),                             // missing at
        json!([{"name": "x", "at": [8, 8], "size": 0}]),    // size 0
        json!([{"name": "x", "at": [8, 8], "size": 4096}]), // size unbounded
        json!([{"at": [8, 8]}]),                            // missing name
    ] {
        let err = call(
            &mut store,
            "colour_sample",
            json!({"image": img, "regions": regions}),
        )
        .unwrap_err();
        assert!(err.is_params, "regions {regions} must be a params error");
    }
}

#[test]
fn pupil_screen_without_any_centres_is_a_params_error() {
    let dir = scratch("no-centres");
    let mut store = store(&dir);
    let img = solid_png(&dir, "x.png", 64, 64, [0, 0, 0]);
    let err = call(&mut store, "pupil_screen", json!({"image": img})).unwrap_err();
    assert!(err.is_params);
}
