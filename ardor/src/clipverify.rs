//! Clip verification: the ardor-verify kit, generalized.
//!
//! Freeze check: a frozen clip carries a `freeze.sha256` over
//! its canonical form — the raw clip text with the whole
//! `"freeze":{...}` member removed (span located by a
//! string-aware brace scan; preceding comma, or trailing comma
//! when freeze is the first member) and at most one trailing
//! newline stripped. This is torogao-format's h0
//! canonicalisation; it is the spec of record and is cited, not
//! forked — see MAPPING.md fork F-1. A freeze that does not
//! recompute is a tampered clip, full stop.
//!
//! Bind check: a bind report's twin frames and loop-closure
//! frames must diff at exactly what gates.json claims for the
//! clip (differing px, max channel delta, pixel count) —
//! recomputed from the pixels, never from file identity
//! (rig-lessons L-37).

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::pngcodec::pixel_diff;

// ------------------------------------------------------------------
// freeze
// ------------------------------------------------------------------

/// Locate the raw `"freeze":{...}` span in clip text (string-
/// aware brace scan). The span includes the preceding comma, or
/// the trailing comma when freeze is the first member.
pub fn freeze_span(text: &str) -> Option<(usize, usize)> {
    let bytes = text.as_bytes();
    // The kit searches for the exact bytes `"freeze":{`.
    let needle = b"\"freeze\":{";
    let key = find_subslice(bytes, needle)?;
    let brace = key + needle.len() - 1;
    let mut depth = 0i64;
    let mut in_str = false;
    let mut esc = false;
    let mut i = brace;
    let end = loop {
        if i >= bytes.len() {
            return None;
        }
        let c = bytes[i];
        if in_str {
            if esc {
                esc = false;
            } else if c == b'\\' {
                esc = true;
            } else if c == b'"' {
                in_str = false;
            }
        } else if c == b'"' {
            in_str = true;
        } else if c == b'{' {
            depth += 1;
        } else if c == b'}' {
            depth -= 1;
            if depth == 0 {
                break i + 1;
            }
        }
        i += 1;
    };
    // Span: from the preceding comma (or the key) to the object
    // end (or past a trailing comma when freeze came first).
    let mut start = key;
    let mut j = key;
    while j > 0 {
        j -= 1;
        if !(bytes[j] as char).is_whitespace() {
            break;
        }
    }
    // j now indexes the significant char before the key (or the
    // key itself when the key is at offset 0).
    let mut span_end = end;
    if j < key && bytes[j] == b',' {
        start = j;
    } else {
        // Freeze is the first member: absorb a trailing comma.
        let mut k = end;
        while k < bytes.len() && (bytes[k] as char).is_whitespace() {
            k += 1;
        }
        if k < bytes.len() && bytes[k] == b',' {
            span_end = k + 1;
        }
    }
    Some((start, span_end))
}

fn find_subslice(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack.windows(needle.len()).position(|w| w == needle)
}

/// Recompute a clip's freeze hash from its raw bytes. Returns
/// (recomputed, recorded), or None when the clip carries no
/// verifiable freeze member.
pub fn freeze_recompute(raw: &[u8]) -> Option<(String, String)> {
    let text = std::str::from_utf8(raw).ok()?;
    let (start, end) = freeze_span(text)?;
    let parsed: Value = serde_json::from_str(text).ok()?;
    let recorded = parsed.get("freeze")?.get("sha256")?.as_str()?.to_string();
    let mut canon = String::with_capacity(text.len());
    canon.push_str(&text[..start]);
    canon.push_str(&text[end..]);
    if canon.ends_with('\n') {
        canon.pop();
        if canon.ends_with('\r') {
            canon.pop();
        }
    }
    let recomputed = format!("{:x}", Sha256::digest(canon.as_bytes()));
    Some((recomputed, recorded))
}

/// One file's freeze outcome.
pub struct FreezeOutcome {
    pub path: String,
    /// pass | mismatch | no_freeze | unreadable
    pub status: String,
    pub recomputed: Option<String>,
    pub recorded: Option<String>,
    pub claim_match: Option<bool>,
}

pub fn check_freeze_file(path: &Path, claim: Option<&str>) -> FreezeOutcome {
    let base = FreezeOutcome {
        path: path.to_string_lossy().to_string(),
        status: String::new(),
        recomputed: None,
        recorded: None,
        claim_match: None,
    };
    let raw = match std::fs::read(path) {
        Ok(r) => r,
        Err(_) => {
            return FreezeOutcome {
                status: "unreadable".to_string(),
                ..base
            };
        }
    };
    match freeze_recompute(&raw) {
        None => FreezeOutcome {
            status: "no_freeze".to_string(),
            ..base
        },
        Some((recomputed, recorded)) => {
            let status = if recomputed.eq_ignore_ascii_case(&recorded) {
                "pass"
            } else {
                "mismatch"
            };
            let claim_match = claim.map(|c| recomputed.eq_ignore_ascii_case(c));
            FreezeOutcome {
                status: status.to_string(),
                recomputed: Some(recomputed),
                recorded: Some(recorded),
                claim_match,
                ..base
            }
        }
    }
}

// ------------------------------------------------------------------
// bind
// ------------------------------------------------------------------

struct Gates {
    clips: Value,
}

fn find_gates(dir: &Path) -> Option<Gates> {
    let mut cur = Some(dir);
    for _ in 0..3 {
        let d = cur?;
        let candidate = d.join("gates.json");
        if candidate.is_file() {
            let text = std::fs::read_to_string(&candidate).ok()?;
            let parsed: Value = serde_json::from_str(&text).ok()?;
            return Some(Gates {
                clips: parsed.get("clips").cloned().unwrap_or(Value::Null),
            });
        }
        cur = d.parent();
    }
    None
}

/// One pair check inside a bind report.
pub struct BindCheck {
    pub kind: String, // twin | loop_closure
    /// pass | mismatch | unclaimed | skipped
    pub status: String,
    pub differing: Option<u64>,
    pub pixels: Option<u64>,
    pub max_delta: Option<u32>,
    pub claimed_differing: Option<Value>,
    pub claimed_max_delta: Option<Value>,
    pub note: Option<String>,
}

/// One bind report's outcome.
pub struct BindOutcome {
    pub path: String,
    pub clip: String,
    /// pass | mismatch | no_checks | unreadable
    pub status: String,
    pub checks: Vec<BindCheck>,
}

pub fn check_bind_file(path: &Path) -> BindOutcome {
    let unreadable = |clip: &str| BindOutcome {
        path: path.to_string_lossy().to_string(),
        clip: clip.to_string(),
        status: "unreadable".to_string(),
        checks: Vec::new(),
    };
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(_) => return unreadable("?"),
    };
    let report: Value = match serde_json::from_str(&text) {
        Ok(v) => v,
        Err(_) => return unreadable("?"),
    };
    let clip = report
        .get("clip")
        .and_then(Value::as_str)
        .unwrap_or("?")
        .to_string();
    let renders: Vec<String> = report
        .get("renders")
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(|r| r.get("file").and_then(Value::as_str).map(str::to_string))
                .collect()
        })
        .unwrap_or_default();
    let frames_bound: Vec<u64> = report
        .get("frames_bound")
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(Value::as_u64).collect())
        .unwrap_or_default();
    if report.get("renders").is_none() || report.get("frames_bound").is_none() {
        return unreadable(&clip);
    }
    let dir = path.parent().unwrap_or(Path::new(".")).to_path_buf();
    let gates = find_gates(&dir);
    let claim = gates
        .as_ref()
        .and_then(|g| g.clips.get(&clip))
        .cloned()
        .unwrap_or(Value::Null);
    let by_file: BTreeSet<&str> = renders.iter().map(String::as_str).collect();
    let front = |f: u64| -> Option<PathBuf> {
        let name = format!("bind_f{f:03}_front.png");
        if by_file.contains(name.as_str()) {
            Some(dir.join(name))
        } else {
            None
        }
    };
    let mut pairs: Vec<(String, PathBuf, PathBuf, Value)> = Vec::new();
    let twin_a = renders.iter().find(|f| f.ends_with("_twinA.png"));
    let twin_b = renders.iter().find(|f| f.ends_with("_twinB.png"));
    if let (Some(a), Some(b)) = (twin_a, twin_b) {
        pairs.push((
            "twin".to_string(),
            dir.join(a),
            dir.join(b),
            claim.get("twin").cloned().unwrap_or(Value::Null),
        ));
    }
    if frames_bound.len() >= 2 {
        let (lo, hi) = (frames_bound[0], frames_bound[frames_bound.len() - 1]);
        if lo != hi
            && let (Some(a), Some(b)) = (front(lo), front(hi))
        {
            pairs.push((
                "loop_closure".to_string(),
                a,
                b,
                claim.get("loop_closure").cloned().unwrap_or(Value::Null),
            ));
        }
    }
    let mut checks = Vec::new();
    for (kind, a, b, pair_claim) in pairs {
        let mut check = BindCheck {
            kind,
            status: String::new(),
            differing: None,
            pixels: None,
            max_delta: None,
            claimed_differing: None,
            claimed_max_delta: None,
            note: None,
        };
        if !a.is_file() || !b.is_file() {
            check.status = "skipped".to_string();
            check.note = Some("frames missing on disk".to_string());
            checks.push(check);
            continue;
        }
        let bytes_a = std::fs::read(&a).unwrap_or_default();
        let bytes_b = std::fs::read(&b).unwrap_or_default();
        match pixel_diff(&bytes_a, &bytes_b) {
            Err(e) => {
                check.status = "mismatch".to_string();
                check.note = Some(format!("pixel diff failed: {e}"));
            }
            Ok((differing, pixels, max_delta)) => {
                check.differing = Some(differing);
                check.pixels = Some(pixels);
                check.max_delta = Some(max_delta);
                if pair_claim.is_null() {
                    check.status = "unclaimed".to_string();
                } else {
                    check.claimed_differing = pair_claim.get("differing").cloned();
                    check.claimed_max_delta = pair_claim.get("max_channel_delta").cloned();
                    let ok = check.claimed_differing.as_ref().and_then(Value::as_u64)
                        == Some(differing)
                        && pair_claim
                            .get("max_channel_delta")
                            .and_then(Value::as_u64)
                            .is_none_or(|v| v == max_delta as u64)
                        && pair_claim
                            .get("pixels")
                            .and_then(Value::as_u64)
                            .is_none_or(|v| v == pixels);
                    check.status = if ok { "pass" } else { "mismatch" }.to_string();
                }
            }
        }
        checks.push(check);
    }
    let status = if checks.is_empty() {
        "no_checks"
    } else if checks.iter().any(|c| c.status == "mismatch") {
        "mismatch"
    } else if checks.iter().any(|c| c.status == "pass") {
        "pass"
    } else {
        "no_checks"
    };
    BindOutcome {
        path: path.to_string_lossy().to_string(),
        clip,
        status: status.to_string(),
        checks,
    }
}

// ------------------------------------------------------------------
// target collection + tool payloads
// ------------------------------------------------------------------

/// Expand file/dir arguments into .json targets: directories are
/// walked recursively; explicit files are taken as named.
/// Returns (targets, explicit_file_count).
pub fn collect_targets(paths: &[String]) -> (Vec<PathBuf>, usize) {
    let mut targets: Vec<PathBuf> = Vec::new();
    let mut explicit = 0usize;
    for p in paths {
        let path = PathBuf::from(p);
        if path.is_dir() {
            let mut found = Vec::new();
            collect_json(&path, &mut found);
            found.sort();
            targets.extend(found);
        } else {
            explicit += 1;
            targets.push(path);
        }
    }
    (targets, explicit)
}

fn collect_json(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    let mut entries: Vec<PathBuf> = entries.filter_map(|e| e.ok().map(|e| e.path())).collect();
    entries.sort();
    for entry in entries {
        if entry.is_dir() {
            collect_json(&entry, out);
        } else if entry.extension().is_some_and(|x| x == "json") {
            out.push(entry);
        }
    }
}

struct Tallies {
    passed: u64,
    mismatched: u64,
    skipped: u64,
    unreadable: u64,
}

impl Tallies {
    fn new() -> Tallies {
        Tallies {
            passed: 0,
            mismatched: 0,
            skipped: 0,
            unreadable: 0,
        }
    }
    fn ok(&self, explicit_files: usize) -> bool {
        self.mismatched == 0 && self.unreadable == 0 && (self.passed > 0 || explicit_files == 0)
    }
    fn json(&self) -> Value {
        json!({
            "passed": self.passed,
            "mismatched": self.mismatched,
            "skipped": self.skipped,
            "unreadable": self.unreadable,
        })
    }
}

/// clip_verify_freeze payload.
pub fn run_freeze(paths: &[String], claim: Option<&str>) -> Value {
    let (targets, explicit) = collect_targets(paths);
    let mut tallies = Tallies::new();
    let mut files = Vec::new();
    for target in &targets {
        let outcome = check_freeze_file(target, claim);
        match outcome.status.as_str() {
            "pass" => tallies.passed += 1,
            "mismatch" => tallies.mismatched += 1,
            "no_freeze" => tallies.skipped += 1,
            _ => tallies.unreadable += 1,
        }
        if let Some(m) = outcome.claim_match {
            if m {
                tallies.passed += 1;
            } else {
                tallies.mismatched += 1;
            }
        }
        files.push(json!({
            "path": outcome.path,
            "status": outcome.status,
            "recomputed": outcome.recomputed,
            "recorded": outcome.recorded,
            "claim_match": outcome.claim_match,
        }));
    }
    json!({
        "files": files,
        "summary": tallies.json(),
        "ok": tallies.ok(explicit),
    })
}

/// clip_verify_bind payload.
pub fn run_bind(paths: &[String]) -> Value {
    let (targets, explicit) = collect_targets(paths);
    let mut tallies = Tallies::new();
    let mut reports = Vec::new();
    for target in &targets {
        if target.file_name().is_some_and(|n| n == "bind_report.json") {
            let outcome = check_bind_file(target);
            if outcome.status == "unreadable" {
                tallies.unreadable += 1;
            } else if outcome.checks.is_empty() {
                tallies.skipped += 1;
            } else {
                for c in &outcome.checks {
                    match c.status.as_str() {
                        "pass" => tallies.passed += 1,
                        "mismatch" => tallies.mismatched += 1,
                        _ => tallies.skipped += 1,
                    }
                }
            }
            reports.push(json!({
                "path": outcome.path,
                "clip": outcome.clip,
                "status": outcome.status,
                "checks": outcome.checks.iter().map(|c| json!({
                    "kind": c.kind,
                    "status": c.status,
                    "differing": c.differing,
                    "pixels": c.pixels,
                    "max_delta": c.max_delta,
                    "claimed_differing": c.claimed_differing,
                    "claimed_max_delta": c.claimed_max_delta,
                    "note": c.note,
                })).collect::<Vec<_>>(),
            }));
        }
    }
    json!({
        "reports": reports,
        "summary": tallies.json(),
        "ok": tallies.ok(explicit),
    })
}

/// clip_verify_corpus payload: freeze over every clip JSON, bind
/// over every bind_report.json, one combined verdict.
pub fn run_corpus(paths: &[String], claim: Option<&str>) -> Value {
    let freeze = run_freeze(paths, claim);
    let bind = run_bind(paths);
    let (fs, bs) = (&freeze["summary"], &bind["summary"]);
    let combined = json!({
        "passed": fs["passed"].as_u64().unwrap_or(0) + bs["passed"].as_u64().unwrap_or(0),
        "mismatched": fs["mismatched"].as_u64().unwrap_or(0) + bs["mismatched"].as_u64().unwrap_or(0),
        "skipped": fs["skipped"].as_u64().unwrap_or(0) + bs["skipped"].as_u64().unwrap_or(0),
        "unreadable": fs["unreadable"].as_u64().unwrap_or(0) + bs["unreadable"].as_u64().unwrap_or(0),
    });
    let ok = freeze["ok"].as_bool().unwrap_or(false) && bind["ok"].as_bool().unwrap_or(false);
    json!({
        "freeze": freeze,
        "bind": bind,
        "summary": combined,
        "ok": ok,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pngcodec::encode_png_rgba;

    const CLIP: &str = include_str!("../tests/fixtures/frozen_clip.json");
    const CLIP_SHA: &str = "c889e0f47c02c764f451e5b135d89b3a56f35cf0d0fdde6c34fc6bf3b2d60f8d";

    fn tempdir(tag: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("ardor-clipverify-{tag}-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("tempdir");
        dir
    }

    #[test]
    fn freeze_recompute_matches_kit() {
        // Ground truth: the ardor-verify kit over this real frozen
        // clip recomputes exactly the recorded hash.
        let (recomputed, recorded) = freeze_recompute(CLIP.as_bytes()).expect("freeze");
        assert_eq!(recomputed, CLIP_SHA);
        assert_eq!(recorded, CLIP_SHA);
    }

    #[test]
    fn freeze_tampered_channel_value_detected() {
        // Adversarial: flip one digit inside a channel value; the
        // recorded hash no longer recomputes.
        let at = CLIP.find("\"ease\"").unwrap_or(0);
        let mut tampered = CLIP.to_string();
        let digit_at = (at..tampered.len())
            .find(|&i| tampered.as_bytes()[i].is_ascii_digit())
            .expect("a digit");
        let b = tampered.as_bytes()[digit_at];
        let replacement = if b == b'9' { "8" } else { "9" };
        tampered.replace_range(digit_at..digit_at + 1, replacement);
        let (recomputed, recorded) = freeze_recompute(tampered.as_bytes()).expect("freeze");
        assert_ne!(recomputed, recorded);
    }

    #[test]
    fn freeze_tampered_recorded_hash_detected() {
        // Adversarial: corrupt one hex digit of the recorded hash.
        let tampered = CLIP.replacen("c889e0f47c02c764", "d889e0f47c02c764", 1);
        let (recomputed, recorded) = freeze_recompute(tampered.as_bytes()).expect("freeze");
        assert_ne!(recomputed, recorded);
        assert_eq!(recomputed, CLIP_SHA);
    }

    #[test]
    fn freeze_absent_is_no_freeze() {
        assert!(freeze_recompute(br#"{"name":"x","channels":{}}"#).is_none());
        assert!(freeze_recompute(b"not json at all").is_none());
    }

    #[test]
    fn freeze_span_handles_braces_inside_strings() {
        // A string value containing braces must not end the span.
        let text = r#"{"name":"a{b}c","freeze":{"sha256":"ab"},"x":1}"#;
        let (start, end) = freeze_span(text).expect("span");
        assert_eq!(&text[start..end], ",\"freeze\":{\"sha256\":\"ab\"}");
    }

    #[test]
    fn freeze_first_member_absorbs_trailing_comma() {
        let text = "{\"freeze\":{\"sha256\":\"ab\"},\"name\":\"x\"}";
        let (recomputed, recorded) = freeze_recompute(text.as_bytes()).expect("freeze");
        assert_eq!(recorded, "ab");
        let expect = format!("{:x}", Sha256::digest(br#"{"name":"x"}"#));
        assert_eq!(recomputed, expect);
    }

    #[test]
    fn bind_catches_one_pixel_corruption() {
        // Adversarial positive control: twin B differs from twin A
        // by one pixel (+17); gates claim 0 differing. The check
        // must report differing=1, max_delta=17, status mismatch.
        let dir = tempdir("bindcorrupt");
        let a: Vec<u8> = (0..16 * 4).map(|i| (i * 3 % 256) as u8).collect();
        let mut b = a.clone();
        b[0] = b[0].wrapping_add(17);
        std::fs::write(dir.join("x_twinA.png"), encode_png_rgba(4, 4, &a)).expect("write a");
        std::fs::write(dir.join("x_twinB.png"), encode_png_rgba(4, 4, &b)).expect("write b");
        std::fs::write(
            dir.join("bind_report.json"),
            r#"{"clip":"clip_x","frames_bound":[0,10],"renders":[{"file":"x_twinA.png"},{"file":"x_twinB.png"}]}"#,
        )
        .expect("write report");
        std::fs::write(
            dir.join("gates.json"),
            r#"{"clips":{"clip_x":{"twin":{"differing":0,"max_channel_delta":0,"pixels":16}}}}"#,
        )
        .expect("write gates");
        let outcome = check_bind_file(&dir.join("bind_report.json"));
        assert_eq!(outcome.status, "mismatch");
        assert_eq!(outcome.checks.len(), 1);
        let check = &outcome.checks[0];
        assert_eq!(check.differing, Some(1));
        assert_eq!(check.max_delta, Some(17));
        assert_eq!(check.status, "mismatch");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn bind_identical_twins_pass_against_claim() {
        let dir = tempdir("bindpass");
        let a: Vec<u8> = (0..16 * 4).map(|i| (i * 7 % 256) as u8).collect();
        std::fs::write(dir.join("x_twinA.png"), encode_png_rgba(4, 4, &a)).expect("write a");
        std::fs::write(dir.join("x_twinB.png"), encode_png_rgba(4, 4, &a)).expect("write b");
        std::fs::write(
            dir.join("bind_report.json"),
            r#"{"clip":"clip_x","frames_bound":[0,10],"renders":[{"file":"x_twinA.png"},{"file":"x_twinB.png"}]}"#,
        )
        .expect("write report");
        std::fs::write(
            dir.join("gates.json"),
            r#"{"clips":{"clip_x":{"twin":{"differing":0,"max_channel_delta":0,"pixels":16}}}}"#,
        )
        .expect("write gates");
        let outcome = check_bind_file(&dir.join("bind_report.json"));
        assert_eq!(outcome.status, "pass");
        assert_eq!(outcome.checks[0].status, "pass");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn bind_without_gates_claim_is_unclaimed() {
        let dir = tempdir("bindunclaimed");
        let a: Vec<u8> = vec![0u8; 16 * 4];
        std::fs::write(dir.join("x_twinA.png"), encode_png_rgba(4, 4, &a)).expect("write a");
        std::fs::write(dir.join("x_twinB.png"), encode_png_rgba(4, 4, &a)).expect("write b");
        std::fs::write(
            dir.join("bind_report.json"),
            r#"{"clip":"clip_x","frames_bound":[0,10],"renders":[{"file":"x_twinA.png"},{"file":"x_twinB.png"}]}"#,
        )
        .expect("write report");
        let outcome = check_bind_file(&dir.join("bind_report.json"));
        assert_eq!(outcome.checks[0].status, "unclaimed");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn corpus_walk_collects_json_recursively() {
        let dir = tempdir("walk");
        let sub = dir.join("sub");
        std::fs::create_dir_all(&sub).expect("sub");
        std::fs::write(dir.join("a.json"), "{}").expect("a");
        std::fs::write(sub.join("b.json"), "{}").expect("b");
        std::fs::write(sub.join("c.txt"), "{}").expect("c");
        let (targets, explicit) = collect_targets(&[dir.to_string_lossy().to_string()]);
        assert_eq!(targets.len(), 2);
        assert_eq!(explicit, 0);
        std::fs::remove_dir_all(&dir).ok();
    }
}
