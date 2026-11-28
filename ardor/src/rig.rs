//! Rig measurement instruments (mapping M-6): the A10.1 bend
//! battery, the face-zone census, and the at-row band measurement,
//! consolidated from the rig programme's lane scripts
//! (`measure_battery.py`, `band_at_row.py`, `census_glb.py`,
//! `census_zones.py`, `zone_truth_blend.py`, `census_blend.py`)
//! into ardor tools. Ardor is the host because cast rig work is
//! NTRanch-lane even when the instrument is SFW (fork F-2);
//! blender-gate keeps generic Blender mechanics only.
//!
//! These are READ tools: they never touch the decide() gate and
//! never write outside a per-call scratch directory. The Python
//! instruments are embedded in the crate (`scripts/`) and run
//! under the OFFICIAL Blender build for anything Blender-side
//! (the Arch build segfaults on IK pose-bone constraints — the
//! two-build rule in TOOLS.md); the binary used is reported in
//! every payload.
//!
//! Instrument-validity contract (rig-lessons ledger, carried in
//! every payload):
//! - L-01: the skeletal instrument is primary; the centroid
//!   instrument is damped by construction on blended rigs and is
//!   reported, never gated.
//! - L-02: flexion is reported anatomically (180 − posed included
//!   angle) with the rest included angle stated per joint.
//! - L-03: populations are fixed from the artefact under test
//!   (torso/zone populations derive from its own rest geometry).
//! - L-25/L-37: structural identity is judged on raw accessor
//!   bytes (colour diffs compare raw bytes per vertex — a float
//!   reinterpretation is invalid, NaN != NaN); renders are judged
//!   on pixels, never file md5. These tools emit no render verdict.
//! - L-27: z_scale comes from the build record only. The battery's
//!   recovered z_scale (height/1.68) is a biased diagnostic and is
//!   flagged as such; `rig_measure_bands` refuses to run without
//!   a build-record value. The fleet battery's scale recovery is
//!   NOT repaired here — that repair belongs to the canonical-base
//!   programme (L-15 discipline).
//! - L-29: the battery reports magnitudes plus a direction column
//!   (evaluated joint world positions, rest and posed). The pose
//!   sign law is per-rig and is probed, never assumed; the gate
//!   checks the hinge poses stay in the sagittal plane, which a
//!   mis-axed bone cannot do.
//! - L-34/L-40/L-41: zones derive from the artefact's own rings.
//!   The GLB zone census replicates the builder's position frame
//!   and inherits it — on cured artefacts the blend-side shape-key
//!   reading is the zone truth.
//! - L-44: window/threshold figures ship with neighbouring-window
//!   population probes; an empty band is an instrument miss until
//!   the probe says otherwise.
//! - L-60: the gate reads every primary-measure key by name
//!   (`achieved_skeletal_deg`, and `achieved_headframe_deg` for
//!   spine_bend). A pose with an applied angle but no readable
//!   primary measure fails the gate loudly, never vanishes.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use serde_json::{Map, Value, json};

use crate::tools::ToolFailure;

type R<T> = Result<T, ToolFailure>;

const BATTERY_PY: &str = include_str!("../scripts/rig_battery.py");
const BANDS_PY: &str = include_str!("../scripts/rig_bands.py");
const CENSUS_GLB_PY: &str = include_str!("../scripts/rig_census_glb.py");
const CENSUS_ZONES_PY: &str = include_str!("../scripts/rig_census_zones.py");
const ZONE_TRUTH_BLEND_PY: &str = include_str!("../scripts/rig_zone_truth_blend.py");
const CENSUS_BLEND_PY: &str = include_str!("../scripts/rig_census_blend.py");

const BATTERY_CONTRACT: &[&str] = &[
    "L-01: skeletal instrument is primary; centroid is reported, never gated.",
    "L-02: flexion reported anatomically (180 - posed included) with rest included angles stated.",
    "L-27: z_scale_recovered (height/1.68) is a biased diagnostic; build-record z_scale governs band placement (see rig_measure_bands).",
    "L-29: magnitudes ship with a direction column (joint world positions, rest + posed); the sign law is per-rig, probed never assumed; hinge poses are gated on the sagittal plane.",
    "L-60: the gate reads every primary key by name (achieved_skeletal_deg; achieved_headframe_deg for spine_bend); an unreadable pose fails loudly.",
];

const BANDS_CONTRACT: &[&str] = &[
    "L-15/L-27: this is the flagged at-row supplementary; the fleet battery is unmodified and its scale-recovery repair is referred to the canonical-base programme.",
    "band_circ is replicated exactly (torso population = dominant bone in {Hips, Spine, Spine1, Spine2} at weight > 0.45 on the rest mesh, +-0.015 window, Ramanujan ellipse); only the band centres move, to row_z x build-record z_scale.",
    "L-44: every row ships neighbouring-window population counts and the nearest populated row centre; an empty band is an instrument miss until the probe says otherwise.",
    "L-27 scope note: the at-row hip is contaminated by the thigh-top ring on Fleet-A lofts; the bust row is the citable figure at z_scale >~ 1.02.",
];

const CENSUS_CONTRACT: &[&str] = &[
    "L-03: populations are fixed from the artefact under test (its own rest geometry and rings).",
    "L-25: colour differences compare raw bytes per vertex; float reinterpretation is invalid (NaN != NaN).",
    "L-34/L-40: zones derive from the artefact's own rings; a morph's name is never evidence of its population (Rule 10).",
    "L-41: the GLB zone census replicates the builder's position frame; on cured artefacts the blend-side shape-key reading is the zone truth.",
    "L-37: structural identity is judged on accessor bytes; renders are judged on pixels, never file md5.",
];

// ---------- argument helpers (tools.rs keeps its own private set) ----------

fn strict_object(args: &Value, allowed: &[&str]) -> R<Map<String, Value>> {
    let Some(object) = args.as_object() else {
        return Err(ToolFailure::params("arguments must be a JSON object"));
    };
    for key in object.keys() {
        if !allowed.contains(&key.as_str()) {
            return Err(ToolFailure::params(format!("unknown argument: {key}")));
        }
    }
    Ok(object.clone())
}

fn req_str(map: &Map<String, Value>, key: &str) -> R<String> {
    match map.get(key).and_then(Value::as_str) {
        Some(s) if !s.is_empty() => Ok(s.to_string()),
        _ => Err(ToolFailure::params(format!(
            "missing or empty string argument: {key}"
        ))),
    }
}

fn opt_str(map: &Map<String, Value>, key: &str) -> Option<String> {
    map.get(key).and_then(Value::as_str).map(str::to_string)
}

fn opt_f64(map: &Map<String, Value>, key: &str) -> R<Option<f64>> {
    match map.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(v) => v.as_f64().map(Some).ok_or_else(|| {
            ToolFailure::params(format!("argument {key} must be a number"))
        }),
    }
}

fn timeout_s(map: &Map<String, Value>) -> R<u64> {
    match map.get("timeout_s") {
        None | Some(Value::Null) => Ok(1200),
        Some(v) => {
            let n = v.as_u64().ok_or_else(|| {
                ToolFailure::params("argument timeout_s must be a non-negative integer")
            })?;
            if n == 0 || n > 3600 {
                return Err(ToolFailure::params(
                    "argument timeout_s must be within 1..=3600",
                ));
            }
            Ok(n)
        }
    }
}

/// A required artefact path: must exist as a file and carry one of
/// the allowed extensions (artefacts are read, never written).
fn req_artefact(map: &Map<String, Value>, key: &str, exts: &[&str]) -> R<PathBuf> {
    let raw = req_str(map, key)?;
    let path = PathBuf::from(&raw);
    if !path.is_file() {
        return Err(ToolFailure::domain(
            "artefact_not_found",
            format!("argument {key} is not an existing file: {raw}"),
        ));
    }
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if !exts.contains(&ext.as_str()) {
        return Err(ToolFailure::params(format!(
            "argument {key} must name a {} file: {raw}",
            exts.join(" or ")
        )));
    }
    Ok(path)
}

fn opt_artefact(map: &Map<String, Value>, key: &str, exts: &[&str]) -> R<Option<PathBuf>> {
    if map.get(key).is_none() || map.get(key) == Some(&Value::Null) {
        return Ok(None);
    }
    req_artefact(map, key, exts).map(Some)
}

// ---------- subprocess plumbing ----------

static SCRATCH_SEQ: AtomicU64 = AtomicU64::new(0);

struct Scratch(PathBuf);

impl Scratch {
    fn new(tool: &str) -> R<Scratch> {
        let seq = SCRATCH_SEQ.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!(
            "ardor-rig-{tool}-{}-{seq}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).map_err(|e| {
            ToolFailure::domain("scratch_failed", format!("cannot create scratch dir: {e}"))
        })?;
        Ok(Scratch(dir))
    }

    fn write(&self, name: &str, content: &str) -> R<PathBuf> {
        let path = self.0.join(name);
        fs::write(&path, content).map_err(|e| {
            ToolFailure::domain("scratch_failed", format!("cannot write {name}: {e}"))
        })?;
        Ok(path)
    }

    fn path(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// Run a command with stdout/stderr captured to scratch files and a
/// hard timeout (killed, never left running).
fn run_capture(scratch: &Scratch, mut cmd: Command, timeout: u64, what: &str) -> R<String> {
    let out_path = scratch.path("stdout.log");
    let err_path = scratch.path("stderr.log");
    let out_file = fs::File::create(&out_path).map_err(|e| {
        ToolFailure::domain("instrument_failed", format!("cannot open stdout log: {e}"))
    })?;
    let err_file = fs::File::create(&err_path).map_err(|e| {
        ToolFailure::domain("instrument_failed", format!("cannot open stderr log: {e}"))
    })?;
    let mut child = cmd
        .stdout(Stdio::from(out_file))
        .stderr(Stdio::from(err_file))
        .spawn()
        .map_err(|e| {
            ToolFailure::domain(
                "instrument_failed",
                format!("cannot start {what}: {e}"),
            )
        })?;
    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let stdout = fs::read_to_string(&out_path).unwrap_or_default();
                if status.success() {
                    return Ok(stdout);
                }
                let stderr = fs::read_to_string(&err_path).unwrap_or_default();
                let tail: String = stderr
                    .lines()
                    .rev()
                    .take(12)
                    .collect::<Vec<_>>()
                    .into_iter()
                    .rev()
                    .collect::<Vec<_>>()
                    .join("\n");
                return Err(ToolFailure::domain(
                    "instrument_failed",
                    format!("{what} exited with {status}; stderr tail:\n{tail}"),
                ));
            }
            Ok(None) => {
                if start.elapsed() > Duration::from_secs(timeout) {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(ToolFailure::domain(
                        "instrument_timeout",
                        format!("{what} exceeded timeout_s={timeout} and was killed"),
                    ));
                }
                std::thread::sleep(Duration::from_millis(100));
            }
            Err(e) => {
                return Err(ToolFailure::domain(
                    "instrument_failed",
                    format!("cannot poll {what}: {e}"),
                ));
            }
        }
    }
}

/// Resolve the Blender binary: explicit argument, then
/// $ARDOR_BLENDER, then the official-build cache (the two-build
/// rule: the Arch build segfaults on IK), then PATH (flagged).
fn resolve_blender(arg: &Option<String>) -> (String, &'static str) {
    if let Some(bin) = arg {
        return (bin.clone(), "argument");
    }
    if let Ok(bin) = std::env::var("ARDOR_BLENDER") {
        if !bin.is_empty() {
            return (bin, "env:ARDOR_BLENDER");
        }
    }
    if let Some(home) = std::env::var_os("HOME") {
        let cache = Path::new(&home).join(".cache/azoth/blender-official");
        if let Ok(entries) = fs::read_dir(&cache) {
            let mut candidates: Vec<PathBuf> = entries
                .flatten()
                .map(|e| e.path().join("blender"))
                .filter(|p| p.is_file())
                .collect();
            candidates.sort();
            if let Some(bin) = candidates.into_iter().next_back() {
                // &'static str source label; the path itself is returned owned.
                return (bin.to_string_lossy().into_owned(), "official-build cache");
            }
        }
    }
    (
        "blender".to_string(),
        "PATH fallback (NOT the official build; IK-carrying artefacts require it)",
    )
}

fn run_blender(
    scratch: &Scratch,
    blender: &str,
    script: &Path,
    args: &[String],
    timeout: u64,
    what: &str,
) -> R<String> {
    let mut cmd = Command::new(blender);
    cmd.arg("-b")
        .arg("-Y")
        .arg("--factory-startup")
        .arg("--offline-mode")
        .arg("-noaudio")
        .arg("-P")
        .arg(script)
        .arg("--")
        .args(args);
    run_capture(scratch, cmd, timeout, what)
}

fn run_python(
    scratch: &Scratch,
    script: &Path,
    args: &[String],
    timeout: u64,
    what: &str,
) -> R<String> {
    let python = std::env::var("ARDOR_PYTHON3").unwrap_or_else(|_| "python3".to_string());
    let mut cmd = Command::new(python);
    cmd.arg(script).args(args);
    run_capture(scratch, cmd, timeout, what)
}

fn read_json_output(scratch: &Scratch, name: &str, what: &str) -> R<Value> {
    let path = scratch.path(name);
    let text = fs::read_to_string(&path).map_err(|e| {
        ToolFailure::domain(
            "instrument_failed",
            format!("{what} produced no output file {name}: {e}"),
        )
    })?;
    serde_json::from_str(&text).map_err(|e| {
        ToolFailure::domain(
            "instrument_failed",
            format!("{what} produced unparseable JSON in {name}: {e}"),
        )
    })
}

fn contract(lines: &[&str]) -> Value {
    Value::Array(lines.iter().map(|l| json!(l)).collect())
}

// ---------- rig_bend_battery ----------

pub fn rig_bend_battery(arguments: &Value) -> R<Value> {
    let map = strict_object(
        arguments,
        &["artefact", "mode", "frames_dir", "blender", "timeout_s"],
    )?;
    let artefact = req_artefact(&map, "artefact", &["glb"])?;
    let mode = opt_str(&map, "mode").unwrap_or_else(|| "full".to_string());
    if mode != "full" && mode != "measure" {
        return Err(ToolFailure::params(format!(
            "argument mode must be \"full\" or \"measure\": {mode}"
        )));
    }
    let frames = opt_str(&map, "frames_dir");
    let timeout = timeout_s(&map)?;
    let (blender, blender_source) = resolve_blender(&opt_str(&map, "blender"));

    let scratch = Scratch::new("battery")?;
    let script = scratch.write("rig_battery.py", BATTERY_PY)?;
    let out = scratch.path("out.json");
    let args = vec![
        artefact.to_string_lossy().into_owned(),
        out.to_string_lossy().into_owned(),
        frames.clone().unwrap_or_else(|| "none".to_string()),
        mode.clone(),
    ];
    let start = Instant::now();
    run_blender(&scratch, &blender, &script, &args, timeout, "rig battery")?;
    let elapsed = start.elapsed().as_secs_f64();
    let instrument = read_json_output(&scratch, "out.json", "rig battery")?;

    let mut payload = json!({
        "tool": "rig_bend_battery",
        "artefact": artefact.to_string_lossy(),
        "mode": mode,
        "blender": {"bin": blender, "source": blender_source},
        "elapsed_s": (elapsed * 10.0).round() / 10.0,
        "step0": instrument.get("step0").cloned().unwrap_or(Value::Null),
        "leak": instrument.get("leak").cloned().unwrap_or(Value::Null),
        "contract": contract(BATTERY_CONTRACT),
    });
    if mode == "full" {
        payload["battery"] = instrument.get("battery").cloned().unwrap_or(Value::Null);
        payload["joints"] = instrument.get("joints").cloned().unwrap_or(Value::Null);
        payload["gate"] = battery_gate(&instrument);
    }
    Ok(payload)
}

// ---------- the battery gate (L-01/L-29/L-60) ----------

const GATE_TOLERANCE_DEG: f64 = 2.0;
/// Sagittal-plane bound for hinge poses: the posed distal segment's
/// lateral (|x|, Blender frame) fraction of its length. A fold in
/// the sagittal plane stays near 0; a mis-axed fold points sideways.
const GATE_MAX_LATERAL_FRACTION: f64 = 0.42;

fn vec3(value: &Value) -> Option<[f64; 3]> {
    let a = value.as_array()?;
    if a.len() != 3 {
        return None;
    }
    Some([
        a[0].as_f64()?,
        a[1].as_f64()?,
        a[2].as_f64()?,
    ])
}

fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn norm(v: [f64; 3]) -> f64 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

fn unit(v: [f64; 3]) -> Option<[f64; 3]> {
    let n = norm(v);
    if n < 1e-9 {
        None
    } else {
        Some([v[0] / n, v[1] / n, v[2] / n])
    }
}

fn joint_pos(joints: &Value, when: &str, pose: Option<&str>, name: &str) -> Option<[f64; 3]> {
    let scope = match pose {
        Some(p) => joints.get(when)?.get(p)?,
        None => joints.get(when)?,
    };
    vec3(scope.get(name)?)
}

/// Gate one battery run. Primary measures are read by name (L-60);
/// centroid values are echoed as diagnostics and never gated (L-01);
/// hinge poses must keep their distal segment in the sagittal plane
/// (L-29 direction column — a mis-axed bone fails here even when
/// its magnitude happens to read correctly).
pub fn battery_gate(instrument: &Value) -> Value {
    // (pose, primary key, hinge (prox a, prox b, distal b), segment (a, b))
    let poses: [(
        &str,
        &str,
        Option<(&str, &str, &str)>,
        Option<(&str, &str)>,
    ); 5] = [
        (
            "elbow_flexion",
            "achieved_skeletal_deg",
            Some((
                "mixamorig:LeftShoulder",
                "mixamorig:LeftForeArm",
                "mixamorig:LeftHand",
            )),
            None,
        ),
        (
            "knee_flexion",
            "achieved_skeletal_deg",
            Some((
                "mixamorig:Hips",
                "mixamorig:LeftLeg",
                "mixamorig:LeftFoot",
            )),
            None,
        ),
        (
            "hip_flexion",
            "achieved_skeletal_deg",
            None,
            Some(("mixamorig:LeftUpLeg", "mixamorig:LeftLeg")),
        ),
        (
            "shoulder_overhead",
            "achieved_skeletal_deg",
            None,
            Some(("mixamorig:LeftArm", "mixamorig:LeftForeArm")),
        ),
        (
            "spine_bend",
            "achieved_headframe_deg",
            None,
            Some(("mixamorig:Hips", "mixamorig:Head")),
        ),
    ];
    let battery = instrument.get("battery");
    let joints = instrument.get("joints");
    let mut verdicts = serde_json::Map::new();
    let mut failed: Vec<String> = Vec::new();
    let mut unmeasured_direction: Vec<String> = Vec::new();

    for (pose, primary_key, hinge, segment) in poses {
        let entry = battery.and_then(|b| b.get(pose));
        let Some(entry) = entry else {
            verdicts.insert(
                pose.to_string(),
                json!({"verdict": "fail", "reason": "pose missing from battery output"}),
            );
            failed.push(pose.to_string());
            continue;
        };
        let applied = entry.get("applied_deg").and_then(Value::as_f64);
        // L-60: the primary measure is read by name; its absence is
        // a loud failure, never a silently shrinking pose set.
        let primary = entry.get(primary_key).and_then(Value::as_f64);
        let (Some(applied), Some(primary)) = (applied, primary) else {
            verdicts.insert(
                pose.to_string(),
                json!({
                    "verdict": "fail",
                    "reason": format!("primary measure {primary_key} unreadable (L-60: fails loudly, never vanishes)"),
                    "applied_deg": applied,
                }),
            );
            failed.push(pose.to_string());
            continue;
        };
        let delta = (primary - applied).abs();
        let mut pose_fail: Option<String> = None;
        if delta > GATE_TOLERANCE_DEG {
            pose_fail = Some(format!(
                "primary measure off by {delta:.2} deg (tolerance {GATE_TOLERANCE_DEG})"
            ));
        }
        // Direction column (L-29): reported for every pose; gated
        // for the two hinge poses via the sagittal-plane check.
        let mut direction = json!({
            "centroid_deg_diagnostic_only": entry.get("achieved_centroid_deg").cloned().unwrap_or(Value::Null),
        });
        let direction_measured = (|| -> Option<()> {
            let joints = joints?;
            if let Some((_, distal_a, distal_b)) = hinge {
                let from = joint_pos(joints, "posed", Some(pose), distal_a)?;
                let to = joint_pos(joints, "posed", Some(pose), distal_b)?;
                let dir = unit(sub(to, from))?;
                direction["distal_dir_posed"] = json!(dir);
                direction["lateral_fraction"] = json!((dir[0].abs() * 100.0).round() / 100.0);
                if dir[0].abs() > GATE_MAX_LATERAL_FRACTION {
                    direction["plane"] = json!("OUT OF SAGITTAL PLANE");
                    if pose_fail.is_none() {
                        pose_fail = Some(format!(
                            "distal segment folds out of the sagittal plane (lateral fraction {:.2} > {GATE_MAX_LATERAL_FRACTION}); magnitude alone cannot see a mis-axed bone (L-29)",
                            dir[0].abs()
                        ));
                    }
                } else {
                    direction["plane"] = json!("sagittal");
                }
            } else if let Some((a, b)) = segment {
                let rest_from = joint_pos(joints, "rest", None, a)?;
                let rest_to = joint_pos(joints, "rest", None, b)?;
                let posed_from = joint_pos(joints, "posed", Some(pose), a)?;
                let posed_to = joint_pos(joints, "posed", Some(pose), b)?;
                direction["segment_dir_rest"] = json!(unit(sub(rest_to, rest_from))?);
                direction["segment_dir_posed"] = json!(unit(sub(posed_to, posed_from))?);
            }
            Some(())
        })()
        .is_some();
        if !direction_measured {
            unmeasured_direction.push(pose.to_string());
        }
        let mut verdict = json!({
            "verdict": if pose_fail.is_none() { "pass" } else { "fail" },
            "applied_deg": applied,
            "primary_key": primary_key,
            "primary_deg": primary,
            "delta_deg": (delta * 100.0).round() / 100.0,
            "direction": direction,
        });
        if let Some(reason) = &pose_fail {
            verdict["reason"] = json!(reason);
            failed.push(pose.to_string());
        }
        verdicts.insert(pose.to_string(), verdict);
    }

    json!({
        "verdict": if failed.is_empty() { "pass" } else { "fail" },
        "tolerance_deg": GATE_TOLERANCE_DEG,
        "plane_max_lateral_fraction": GATE_MAX_LATERAL_FRACTION,
        "poses": Value::Object(verdicts),
        "failed_poses": failed,
        "direction_unmeasured_poses": unmeasured_direction,
        "notes": [
            "centroid figures are diagnostics and never gated (L-01)",
            "a pose whose primary measure is unreadable fails the gate (L-60)",
        ],
    })
}

// ---------- rig_measure_bands ----------

const DEFAULT_ROWS: [(&str, f64); 3] = [("bust", 1.235), ("waist", 1.045), ("hip", 0.905)];

fn resolve_z_scale(map: &Map<String, Value>) -> R<(f64, String)> {
    if let Some(zs) = opt_f64(map, "z_scale")? {
        if zs <= 0.0 {
            return Err(ToolFailure::params("argument z_scale must be positive"));
        }
        return Ok((zs, "argument".to_string()));
    }
    if let Some(spec_raw) = opt_str(map, "spec") {
        let spec_path = PathBuf::from(&spec_raw);
        let text = fs::read_to_string(&spec_path).map_err(|e| {
            ToolFailure::domain(
                "spec_unreadable",
                format!("cannot read spec {spec_raw}: {e}"),
            )
        })?;
        let spec: Value = serde_json::from_str(&text).map_err(|e| {
            ToolFailure::domain(
                "spec_unreadable",
                format!("spec {spec_raw} is not valid JSON: {e}"),
            )
        })?;
        if let Some(zs) = spec.get("z_scale").and_then(Value::as_f64) {
            if zs > 0.0 {
                return Ok((zs, format!("spec:{spec_raw}")));
            }
        }
        return Err(ToolFailure::domain(
            "z_scale_unrecorded",
            format!("spec {spec_raw} carries no positive z_scale"),
        ));
    }
    Err(ToolFailure::domain(
        "z_scale_unrecorded",
        "no build-record z_scale: pass z_scale or a spec carrying one. \
         The battery's recovered scale (height/1.68) is biased and must not \
         place bands (L-27); this tool refuses rather than guess.",
    ))
}

fn resolve_rows(map: &Map<String, Value>) -> R<Vec<(String, f64)>> {
    let Some(raw) = map.get("rows") else {
        return Ok(DEFAULT_ROWS
            .iter()
            .map(|(n, z)| (n.to_string(), *z))
            .collect());
    };
    let Some(items) = raw.as_array() else {
        return Err(ToolFailure::params(
            "argument rows must be an array of {name, row_z}",
        ));
    };
    if items.is_empty() || items.len() > 16 {
        return Err(ToolFailure::params("argument rows must hold 1..=16 rows"));
    }
    let mut rows: Vec<(String, f64)> = Vec::new();
    for item in items {
        let Some(obj) = item.as_object() else {
            return Err(ToolFailure::params("each row must be an object {name, row_z}"));
        };
        let name = obj
            .get("name")
            .and_then(Value::as_str)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| ToolFailure::params("each row needs a non-empty name"))?;
        let row_z = obj
            .get("row_z")
            .and_then(Value::as_f64)
            .filter(|z| *z > 0.0)
            .ok_or_else(|| {
                ToolFailure::params("each row needs a positive row_z (unscaled frame)")
            })?;
        if rows.iter().any(|(n, _)| n == name) {
            return Err(ToolFailure::params(format!("duplicate row name: {name}")));
        }
        rows.push((name.to_string(), row_z));
    }
    Ok(rows)
}

pub fn rig_measure_bands(arguments: &Value) -> R<Value> {
    let map = strict_object(
        arguments,
        &["artefact", "z_scale", "spec", "rows", "blender", "timeout_s"],
    )?;
    let artefact = req_artefact(&map, "artefact", &["glb"])?;
    let (z_scale, z_scale_source) = resolve_z_scale(&map)?;
    let rows = resolve_rows(&map)?;
    let timeout = timeout_s(&map)?;
    let (blender, blender_source) = resolve_blender(&opt_str(&map, "blender"));

    let scratch = Scratch::new("bands")?;
    let script = scratch.write("rig_bands.py", BANDS_PY)?;
    let rows_json: Vec<Value> = rows
        .iter()
        .map(|(name, z)| json!({"name": name, "row_z": z}))
        .collect();
    let rows_text = serde_json::to_string(&rows_json).map_err(|e| {
        ToolFailure::domain("scratch_failed", format!("cannot serialise rows: {e}"))
    })?;
    let rows_path = scratch.write("rows.json", &rows_text)?;
    let out = scratch.path("out.json");
    let args = vec![
        artefact.to_string_lossy().into_owned(),
        format!("{z_scale}"),
        rows_path.to_string_lossy().into_owned(),
        out.to_string_lossy().into_owned(),
    ];
    let start = Instant::now();
    run_blender(&scratch, &blender, &script, &args, timeout, "rig bands")?;
    let elapsed = start.elapsed().as_secs_f64();
    let instrument = read_json_output(&scratch, "out.json", "rig bands")?;

    let mut payload = json!({
        "tool": "rig_measure_bands",
        "artefact": artefact.to_string_lossy(),
        "z_scale": z_scale,
        "z_scale_source": z_scale_source,
        "blender": {"bin": blender, "source": blender_source},
        "elapsed_s": (elapsed * 10.0).round() / 10.0,
        "rows": instrument.get("rows").cloned().unwrap_or(Value::Null),
        "contract": contract(BANDS_CONTRACT),
    });
    for name in ["bust_circ_at_row_m", "waist_circ_at_row_m", "hip_circ_at_row_m"] {
        if let Some(v) = instrument.get(name) {
            payload[name] = v.clone();
        }
    }
    Ok(payload)
}

// ---------- rig_zone_census ----------

pub fn rig_zone_census(arguments: &Value) -> R<Value> {
    let map = strict_object(
        arguments,
        &["artefact", "pair", "spec", "blender", "timeout_s"],
    )?;
    let artefact = req_artefact(&map, "artefact", &["glb", "blend"])?;
    let pair = opt_artefact(&map, "pair", &["glb", "blend"])?;
    let spec = opt_str(&map, "spec");
    if let Some(spec_raw) = &spec {
        if !Path::new(spec_raw).is_file() {
            return Err(ToolFailure::domain(
                "spec_unreadable",
                format!("spec is not an existing file: {spec_raw}"),
            ));
        }
    }
    let ext = artefact
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if let Some(pair_path) = &pair {
        let pair_ext = pair_path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_ascii_lowercase();
        if pair_ext != ext {
            return Err(ToolFailure::params(format!(
                "argument pair must match the artefact family ({ext}): {}",
                pair_path.to_string_lossy()
            )));
        }
    }
    let timeout = timeout_s(&map)?;
    let (blender, blender_source) = resolve_blender(&opt_str(&map, "blender"));
    let scratch = Scratch::new("census")?;
    let start = Instant::now();

    let mut payload = json!({
        "tool": "rig_zone_census",
        "artefact": artefact.to_string_lossy(),
        "pair": pair.as_ref().map(|p| p.to_string_lossy().into_owned()),
        "artefact_family": ext,
        "contract": contract(CENSUS_CONTRACT),
    });

    if ext == "glb" {
        let script = scratch.write("rig_census_glb.py", CENSUS_GLB_PY)?;
        let out = scratch.path("census.json");
        match &pair {
            Some(pair_path) => {
                let args = vec![
                    artefact.to_string_lossy().into_owned(),
                    pair_path.to_string_lossy().into_owned(),
                    out.to_string_lossy().into_owned(),
                ];
                run_python(&scratch, &script, &args, timeout, "GLB pair census")?;
                let census = read_json_output(&scratch, "census.json", "GLB pair census")?;
                payload["census"] = census.get("a").cloned().unwrap_or(Value::Null);
                payload["census_pair"] = census.get("b").cloned().unwrap_or(Value::Null);
                payload["one_body_proof"] =
                    census.get("one_body_proof").cloned().unwrap_or(Value::Null);
            }
            None => {
                let args = vec![
                    artefact.to_string_lossy().into_owned(),
                    out.to_string_lossy().into_owned(),
                ];
                run_python(&scratch, &script, &args, timeout, "GLB census")?;
                let census = read_json_output(&scratch, "census.json", "GLB census")?;
                payload["census"] = census.get("a").cloned().unwrap_or(Value::Null);
            }
        }
        if let Some(spec_raw) = &spec {
            let zscript = scratch.write("rig_census_zones.py", CENSUS_ZONES_PY)?;
            let args = vec![
                artefact.to_string_lossy().into_owned(),
                spec_raw.clone(),
            ];
            let stdout = run_python(&scratch, &zscript, &args, timeout, "GLB zone census")?;
            let zones: Value = serde_json::from_str(&stdout).map_err(|e| {
                ToolFailure::domain(
                    "instrument_failed",
                    format!("GLB zone census produced unparseable JSON: {e}"),
                )
            })?;
            payload["zones"] = zones;
            payload["zone_frame"] = json!(
                "builder-position replication (L-41: citable for shipped/uncured artefacts; cured artefacts read blend-side truth)"
            );
        } else {
            payload["zones"] = Value::Null;
            payload["zone_frame"] = json!("no spec supplied: structural census only");
        }
    } else {
        // .blend family: A10.2 blend census (pair-shaped; a single
        // artefact is censused against itself) + zone truth.
        let cscript = scratch.write("rig_census_blend.py", CENSUS_BLEND_PY)?;
        let cout = scratch.path("census_blend.json");
        let second = pair
            .as_ref()
            .unwrap_or(&artefact)
            .to_string_lossy()
            .into_owned();
        let args = vec![
            artefact.to_string_lossy().into_owned(),
            second,
            cout.to_string_lossy().into_owned(),
        ];
        run_blender(&scratch, &blender, &cscript, &args, timeout, "blend census")?;
        let census = read_json_output(&scratch, "census_blend.json", "blend census")?;
        payload["blend_census"] = census.clone();
        if pair.is_some() {
            let a_key = artefact.to_string_lossy().into_owned();
            let b_key = pair.as_ref().unwrap().to_string_lossy().into_owned();
            let (Some(a), Some(b)) = (census.get(&a_key), census.get(&b_key)) else {
                return Err(ToolFailure::domain(
                    "instrument_failed",
                    "blend census output is missing an artefact entry",
                ));
            };
            payload["census_comparison"] = json!({
                "bone_sets_equal": a.get("bones") == b.get("bones"),
                "bone_count": [a.get("bone_count"), b.get("bone_count")],
                "body_shape_keys_equal": a.get("body_shape_keys") == b.get("body_shape_keys"),
                "objects_equal": a.get("objects") == b.get("objects"),
            });
        }
        if let Some(spec_raw) = &spec {
            let zscript = scratch.write("rig_zone_truth_blend.py", ZONE_TRUTH_BLEND_PY)?;
            let zout = scratch.path("zone_truth.json");
            let args = vec![
                artefact.to_string_lossy().into_owned(),
                spec_raw.clone(),
                zout.to_string_lossy().into_owned(),
            ];
            run_blender(&scratch, &blender, &zscript, &args, timeout, "blend zone truth")?;
            payload["zone_truth"] =
                read_json_output(&scratch, "zone_truth.json", "blend zone truth")?;
            if let Some(pair_path) = &pair {
                let zout_b = scratch.path("zone_truth_b.json");
                let args = vec![
                    pair_path.to_string_lossy().into_owned(),
                    spec_raw.clone(),
                    zout_b.to_string_lossy().into_owned(),
                ];
                run_blender(&scratch, &blender, &zscript, &args, timeout, "blend zone truth (pair)")?;
                payload["zone_truth_pair"] =
                    read_json_output(&scratch, "zone_truth_b.json", "blend zone truth (pair)")?;
            }
            payload["zone_frame"] = json!(
                "the artefact's own shape-key deltas by exact ring-slot membership (L-41 truth for cured artefacts)"
            );
        } else {
            payload["zone_truth"] = Value::Null;
            payload["zone_frame"] =
                json!("no spec supplied: blend census only; zone truth requires the build spec");
        }
        payload["blender"] = json!({"bin": blender, "source": blender_source});
    }

    payload["elapsed_s"] = json!((start.elapsed().as_secs_f64() * 10.0).round() / 10.0);
    Ok(payload)
}

// ---------- tests ----------

#[cfg(test)]
mod tests {
    use super::*;

    /// The cured-Sable battery figures of record
    /// (fleet-builder-cure proof, battery_sable_cured.json) with
    /// sagittal joint positions: the gate must PASS.
    fn record_instrument() -> Value {
        json!({
            "battery": {
                "elbow_flexion": {"applied_deg": 110, "rest_included_deg": 171.35,
                    "achieved_skeletal_deg": 111.17, "achieved_distal_deg": 111.4,
                    "achieved_centroid_deg": 102.52},
                "knee_flexion": {"applied_deg": 100, "rest_included_deg": 168.45,
                    "achieved_skeletal_deg": 99.32, "achieved_distal_deg": 102.31,
                    "achieved_centroid_deg": 94.07},
                "hip_flexion": {"applied_deg": 90,
                    "achieved_centroid_deg": 64.44, "achieved_skeletal_deg": 90.0},
                "shoulder_overhead": {"applied_deg": 165,
                    "achieved_centroid_deg": 118.53, "achieved_skeletal_deg": 165.0},
                "spine_bend": {"applied_deg": 30, "achieved_headframe_deg": 30.0,
                    "achieved_chord_deg": 20.3, "achieved_centroid_deg": 21.53}
            },
            "joints": {
                "rest": {
                    "mixamorig:Hips": [0.0, 0.0, 0.95],
                    "mixamorig:LeftUpLeg": [0.09, 0.0, 0.95],
                    "mixamorig:LeftLeg": [0.09, 0.0, 0.52],
                    "mixamorig:LeftFoot": [0.09, -0.02, 0.08],
                    "mixamorig:LeftShoulder": [0.02, -0.01, 1.42],
                    "mixamorig:LeftArm": [0.19, 0.0, 1.40],
                    "mixamorig:LeftForeArm": [0.44, 0.0, 1.38],
                    "mixamorig:LeftHand": [0.68, 0.0, 1.37],
                    "mixamorig:Head": [0.0, 0.0, 1.55]
                },
                "posed": {
                    "elbow_flexion": {"mixamorig:LeftForeArm": [0.44, 0.0, 1.38],
                                      "mixamorig:LeftHand": [0.46, -0.06, 1.62]},
                    "knee_flexion": {"mixamorig:LeftLeg": [0.09, 0.0, 0.52],
                                     "mixamorig:LeftFoot": [0.09, 0.40, 0.50]},
                    "hip_flexion": {"mixamorig:LeftUpLeg": [0.09, 0.0, 0.95],
                                    "mixamorig:LeftLeg": [0.09, -0.42, 0.93]},
                    "shoulder_overhead": {"mixamorig:LeftArm": [0.19, 0.0, 1.40],
                                          "mixamorig:LeftForeArm": [0.16, -0.02, 1.65]},
                    "spine_bend": {"mixamorig:Hips": [0.0, 0.0, 0.95],
                                   "mixamorig:Head": [0.0, -0.16, 1.52]}
                }
            }
        })
    }

    #[test]
    fn gate_passes_on_record_figures() {
        let gate = battery_gate(&record_instrument());
        assert_eq!(gate["verdict"], "pass", "gate: {gate}");
        assert_eq!(gate["failed_poses"], json!([]));
    }

    #[test]
    fn gate_fails_loudly_when_spine_primary_key_missing() {
        // L-60 adversarial: spine_bend without achieved_headframe_deg
        // must fail by name, not shrink the pose set.
        let mut inst = record_instrument();
        inst["battery"]["spine_bend"]
            .as_object_mut()
            .unwrap()
            .remove("achieved_headframe_deg");
        let gate = battery_gate(&inst);
        assert_eq!(gate["verdict"], "fail");
        assert_eq!(gate["failed_poses"], json!(["spine_bend"]));
        assert!(
            gate["poses"]["spine_bend"]["reason"]
                .as_str()
                .unwrap()
                .contains("achieved_headframe_deg")
        );
    }

    #[test]
    fn gate_fails_on_misaxed_knee_despite_correct_magnitude() {
        // Positive-control shape: the shin folds sideways (lateral
        // direction) while the skeletal magnitude reads fine.
        let mut inst = record_instrument();
        inst["joints"]["posed"]["knee_flexion"]["mixamorig:LeftFoot"] = json!([0.47, 0.0, 0.50]);
        let gate = battery_gate(&inst);
        assert_eq!(gate["verdict"], "fail");
        assert_eq!(gate["failed_poses"], json!(["knee_flexion"]));
        assert_eq!(
            gate["poses"]["knee_flexion"]["direction"]["plane"],
            "OUT OF SAGITTAL PLANE"
        );
    }

    #[test]
    fn gate_fails_on_angle_miss() {
        let mut inst = record_instrument();
        inst["battery"]["elbow_flexion"]["achieved_skeletal_deg"] = json!(95.0);
        let gate = battery_gate(&inst);
        assert_eq!(gate["verdict"], "fail");
        assert_eq!(gate["failed_poses"], json!(["elbow_flexion"]));
    }

    #[test]
    fn gate_never_gates_on_centroid() {
        // L-01 adversarial: absurd centroid values, sound skeletal
        // figures — the gate must still pass.
        let mut inst = record_instrument();
        for pose in [
            "elbow_flexion",
            "knee_flexion",
            "hip_flexion",
            "shoulder_overhead",
            "spine_bend",
        ] {
            inst["battery"][pose]["achieved_centroid_deg"] = json!(3.0);
        }
        let gate = battery_gate(&inst);
        assert_eq!(gate["verdict"], "pass", "gate: {gate}");
    }

    #[test]
    fn gate_fails_when_a_pose_is_missing_entirely() {
        let mut inst = record_instrument();
        inst["battery"].as_object_mut().unwrap().remove("hip_flexion");
        let gate = battery_gate(&inst);
        assert_eq!(gate["verdict"], "fail");
        assert_eq!(gate["failed_poses"], json!(["hip_flexion"]));
    }
}
