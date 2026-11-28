//! Rig instrument tools (M-6): validation + adversarial tests.
//! The Blender-side happy paths are gated as lane acceptance runs
//! (they need the official Blender build and real rigs); these
//! tests cover the surface that must hold everywhere: the GLB
//! census on synthetic fixtures, argument discipline, and the
//! refusals that protect the instruments' validity (L-27).

use ardor::store::Store;
use ardor::tools::{self, ToolFailure};
use serde_json::{Value, json};
use std::fs;
use std::path::PathBuf;

struct Fixture {
    root: PathBuf,
    store: Store,
}

impl Fixture {
    fn new(tag: &str) -> Fixture {
        let root = std::env::temp_dir().join(format!(
            "ardor-rig-{tag}-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        let store = Store::open(&root).unwrap();
        Fixture { root, store }
    }

    /// ToolFailure carries no Debug impl (fleet style), so the
    /// expectations unwrap by hand.
    fn call_ok(&mut self, name: &str, args: Value) -> Value {
        match tools::dispatch(&mut self.store, name, &args) {
            Ok(v) => v,
            Err(e) => panic!("{name} must succeed: [{}] {}", e.code, e.message),
        }
    }

    fn call_err(&mut self, name: &str, args: Value) -> ToolFailure {
        match tools::dispatch(&mut self.store, name, &args) {
            Ok(v) => panic!("{name} must fail, got success: {v}"),
            Err(e) => e,
        }
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

/// A minimal but structurally real GLB: one joint, one `_Body`
/// mesh with a morph target name and a single-vertex POSITION.
fn write_glb(path: &PathBuf, joint_name: &str) {
    let json_text = json!({
        "asset": {"version": "2.0"},
        "nodes": [{"name": joint_name}, {"name": "Solo_Body"}],
        "skins": [{"joints": [0]}],
        "meshes": [{
            "name": "Solo_Body",
            "primitives": [{"attributes": {"POSITION": 0, "JOINTS_0": 1, "WEIGHTS_0": 2}}],
            "extras": {"targetNames": ["Fac_Mth_Aa"]},
        }],
        "accessors": [
            {"bufferView": 0, "componentType": 5126, "count": 1, "type": "VEC3"},
            {"bufferView": 1, "componentType": 5121, "count": 1, "type": "VEC4"},
            {"bufferView": 2, "componentType": 5126, "count": 1, "type": "VEC4"},
        ],
        "bufferViews": [
            {"buffer": 0, "byteOffset": 0, "byteLength": 12},
            {"buffer": 0, "byteOffset": 12, "byteLength": 4},
            {"buffer": 0, "byteOffset": 16, "byteLength": 16},
        ],
        "buffers": [{"byteLength": 32}],
    })
    .to_string();
    let mut json_bytes = json_text.into_bytes();
    while json_bytes.len() % 4 != 0 {
        json_bytes.push(b' ');
    }
    let bin_bytes = [0u8; 32];
    let total = 12 + 8 + json_bytes.len() + 8 + bin_bytes.len();
    let mut out = Vec::new();
    out.extend_from_slice(b"glTF");
    out.extend_from_slice(&2u32.to_le_bytes());
    out.extend_from_slice(&(total as u32).to_le_bytes());
    out.extend_from_slice(&(json_bytes.len() as u32).to_le_bytes());
    out.extend_from_slice(b"JSON");
    out.extend_from_slice(&json_bytes);
    out.extend_from_slice(&(bin_bytes.len() as u32).to_le_bytes());
    out.extend_from_slice(b"BIN\0");
    out.extend_from_slice(&bin_bytes);
    fs::write(path, out).unwrap();
}

// ---------- validation ----------

#[test]
fn census_single_glb_reports_structure() {
    let mut f = Fixture::new("census-single");
    let glb = f.root.join("solo.glb");
    write_glb(&glb, "mixamorig:Hips");
    let out = f.call_ok(
        "rig_zone_census",
        json!({"artefact": glb.to_string_lossy()}),
    );
    assert_eq!(out["census"]["joints"], json!(["mixamorig:Hips"]));
    assert_eq!(out["census"]["meshes"]["Solo_Body"]["verts"], 1);
    assert_eq!(
        out["census"]["meshes"]["Solo_Body"]["targets"],
        json!(["Fac_Mth_Aa"])
    );
    // No spec: structural census only, and the payload says so.
    assert_eq!(out["zones"], Value::Null);
}

#[test]
fn census_pair_proves_one_body_for_identical_glbs() {
    let mut f = Fixture::new("census-pair");
    let a = f.root.join("a.glb");
    let b = f.root.join("b.glb");
    write_glb(&a, "mixamorig:Hips");
    write_glb(&b, "mixamorig:Hips");
    let out = f.call_ok(
        "rig_zone_census",
        json!({"artefact": a.to_string_lossy(), "pair": b.to_string_lossy()}),
    );
    let proof = &out["one_body_proof"];
    assert_eq!(proof["joint_sets_equal"], true);
    assert_eq!(proof["position_equal"], true);
    assert_eq!(proof["morph_sets_equal"], true);
    assert_eq!(proof["joint_count"], 1);
}

#[test]
fn bands_refuses_without_build_record_z_scale() {
    // L-27: no z_scale argument and no spec — the tool must refuse
    // rather than recover a scale from the artefact's height.
    let mut f = Fixture::new("bands-refusal");
    let glb = f.root.join("solo.glb");
    write_glb(&glb, "mixamorig:Hips");
    let err = f.call_err(
        "rig_measure_bands",
        json!({"artefact": glb.to_string_lossy()}),
    );
    assert_eq!(err.code, "z_scale_unrecorded");
    assert!(!err.is_params);
}

#[test]
fn battery_missing_artefact_is_a_domain_error() {
    let mut f = Fixture::new("battery-missing");
    let err = f.call_err(
        "rig_bend_battery",
        json!({"artefact": "/nonexistent/definitely-not-here.glb"}),
    );
    assert_eq!(err.code, "artefact_not_found");
}

// ---------- adversarial ----------

#[test]
fn census_pair_detects_a_joint_difference() {
    let mut f = Fixture::new("census-diff");
    let a = f.root.join("a.glb");
    let b = f.root.join("b.glb");
    write_glb(&a, "mixamorig:Hips");
    write_glb(&b, "mixamorig:Other");
    let out = f.call_ok(
        "rig_zone_census",
        json!({"artefact": a.to_string_lossy(), "pair": b.to_string_lossy()}),
    );
    assert_eq!(out["one_body_proof"]["joint_sets_equal"], false);
}

#[test]
fn battery_rejects_unknown_arguments() {
    let mut f = Fixture::new("battery-unknown-arg");
    let err = f.call_err(
        "rig_bend_battery",
        json!({"artefact": "/nonexistent/x.glb", "surprise": true}),
    );
    assert!(err.is_params);
    assert!(err.message.contains("unknown argument: surprise"));
}

#[test]
fn census_rejects_wrong_extension() {
    let mut f = Fixture::new("census-ext");
    let txt = f.root.join("notes.txt");
    fs::write(&txt, "not a rig").unwrap();
    let err = f.call_err(
        "rig_zone_census",
        json!({"artefact": txt.to_string_lossy()}),
    );
    assert!(err.is_params);
}

#[test]
fn census_rejects_mixed_pair_families() {
    let mut f = Fixture::new("census-mixed");
    let glb = f.root.join("a.glb");
    let blend = f.root.join("b.blend");
    write_glb(&glb, "mixamorig:Hips");
    fs::write(&blend, "dummy").unwrap();
    let err = f.call_err(
        "rig_zone_census",
        json!({"artefact": glb.to_string_lossy(), "pair": blend.to_string_lossy()}),
    );
    assert!(err.is_params);
}

#[test]
fn bands_rejects_duplicate_row_names() {
    let mut f = Fixture::new("bands-dup-rows");
    let glb = f.root.join("solo.glb");
    write_glb(&glb, "mixamorig:Hips");
    let err = f.call_err(
        "rig_measure_bands",
        json!({
            "artefact": glb.to_string_lossy(),
            "z_scale": 1.0,
            "rows": [{"name": "bust", "row_z": 1.235}, {"name": "bust", "row_z": 1.3}]
        }),
    );
    assert!(err.is_params);
}

#[test]
fn bands_rejects_nonpositive_z_scale() {
    let mut f = Fixture::new("bands-bad-zs");
    let glb = f.root.join("solo.glb");
    write_glb(&glb, "mixamorig:Hips");
    let err = f.call_err(
        "rig_measure_bands",
        json!({"artefact": glb.to_string_lossy(), "z_scale": -1.0}),
    );
    assert!(err.is_params);
}

#[test]
fn census_rejects_a_directory_as_artefact() {
    let mut f = Fixture::new("census-dir");
    let dir = f.root.join("fake.glb");
    fs::create_dir_all(&dir).unwrap();
    let err = f.call_err(
        "rig_zone_census",
        json!({"artefact": dir.to_string_lossy()}),
    );
    assert_eq!(err.code, "artefact_not_found");
}
