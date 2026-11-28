//! Portrait-lock registry (M-1): which frame is current for each
//! character of a project, what it superseded, and the Owner
//! ruling that put it there — the lock-state half of the asset
//! surface. Provenance (how a frame was generated) lives in the
//! keeper-chain surface (`chains`); every portrait entry here is
//! born as an asset record in the same store (registered through
//! the same gate checks as `register_asset`) and the entry points
//! at it by id, so there is one asset store, not a parallel one.
//!
//! The mechanism generalizes the ardor-registry CLI
//! (tools/ardor-registry, schema `moondeep-portrait-registry/1`):
//! supersession chains, ruling provenance, and SHA-256 pins that
//! are recomputed from the frame's bytes at registration — never
//! copied from a record — and re-verified on every read
//! (`sha_state`). The document schema here is the neutral
//! `portrait-registry/1`: the game is the project itself, so no
//! per-game schema name forks the format. The cast list and the
//! set vocabulary are per-game config: the cast is the project's
//! own cast document (a portrait write must name an existing cast
//! member, as canon writes do), and the set vocabulary is the
//! recorded union of sets the project has registered.
//!
//! Supersession rule (recorded normalization): superseding a
//! frame rewrites its status to `superseded` — unless the frame
//! is `defective`. A defect is the stronger fact (the frame is
//! gated out on its own demerits) and is never laundered into a
//! mere replacement; the chain link (`superseded_by`) still
//! records what replaced it. This reproduces the CLI registry's
//! own data exactly (the Tamsin heightfix case: her defective
//! canvas-fresh frame keeps `defective` with the link set).

use std::fs::File;
use std::io::Read;
use std::path::Path;

use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use crate::gate::Refusal;
use crate::model::{arr, arr_mut, find_by_id, find_by_id_mut};
use crate::store::ProjectState;

/// The neutral registry schema this module writes.
pub const SCHEMA: &str = "portrait-registry/1";

/// Statuses a frame may be registered under. `locked` is reached
/// only through the lock operation (which requires a ruling) and
/// `superseded` only through supersession; neither is a
/// registration status.
pub const REGISTER_STATUSES: [&str; 3] = ["candidate", "defective", "working-set"];

/// Every status a stored entry may carry.
pub const STATUSES: [&str; 5] = [
    "candidate",
    "defective",
    "locked",
    "superseded",
    "working-set",
];

fn field<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str)
}

fn refuse(code: &'static str, message: impl Into<String>) -> Refusal {
    Refusal::new(code, message)
}

/// SHA-256 hex digest of a file's bytes, streamed in 1 MiB
/// chunks (frames run to tens of megabytes). `None` on any I/O
/// failure — callers turn that into a refusal or an `unreadable`
/// read state, never a guessed digest.
pub fn sha256_file(path: &Path) -> Option<String> {
    let mut file = File::open(path).ok()?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 1 << 20];
    loop {
        let n = file.read(&mut buffer).ok()?;
        if n == 0 {
            break;
        }
        hasher.update(&buffer[..n]);
    }
    let digest = hasher.finalize();
    let mut out = String::with_capacity(64);
    for byte in digest {
        out.push_str(&format!("{byte:02x}"));
    }
    Some(out)
}

/// The live verification state of one entry's pin against the
/// file on disk: `verified` (bytes hash to the pin), `mismatch`
/// (bytes hash to something else — the planted-error state the
/// test suite proves is caught), `missing` (no file at the
/// recorded path), or `unreadable` (a file is there but cannot
/// be hashed).
pub fn sha_state(entry: &Value) -> &'static str {
    let path = field(entry, "path").unwrap_or("");
    let pin = field(entry, "sha256").unwrap_or("");
    let path = Path::new(path);
    if !path.is_file() {
        return "missing";
    }
    match sha256_file(path) {
        Some(digest) if digest == pin => "verified",
        Some(_) => "mismatch",
        None => "unreadable",
    }
}

/// One entry as the read tools present it: the stored record
/// plus its live `sha_state`.
pub fn entry_view(entry: &Value) -> Value {
    let mut view = entry.clone();
    view.as_object_mut()
        .expect("portrait entry is an object")
        .insert("sha_state".to_string(), json!(sha_state(entry)));
    view
}

/// The character's current frame: her locked frame if one
/// stands, else her working-set frame. A frame carrying a
/// `superseded_by` link is never current, whatever its status
/// says — the chain link outranks the status field (the CLI's
/// verdict precedence).
pub fn current_entry<'a>(doc: &'a Value, character: &str) -> Option<&'a Value> {
    for status in ["locked", "working-set"] {
        let found = arr(doc, "entries").iter().find(|e| {
            field(e, "character") == Some(character)
                && field(e, "status") == Some(status)
                && e.get("superseded_by").map(Value::is_null).unwrap_or(false)
        });
        if found.is_some() {
            return found;
        }
    }
    None
}

/// Whether the id names a cast member of the project.
pub fn character_exists(state: &ProjectState, character: &str) -> bool {
    find_by_id(state.doc("cast"), "characters", character).is_some()
}

/// Whether any portrait entry names this character (a character
/// with entries but no cast record is still readable history).
pub fn has_entries(state: &ProjectState, character: &str) -> bool {
    arr(state.doc("portraits"), "entries")
        .iter()
        .any(|e| field(e, "character") == Some(character))
}

/// The `list_portrait_locks` payload: the manifest view — every
/// cast member's current frame with its pin re-verified, plus
/// registry counts and the set vocabulary.
pub fn list_view(state: &ProjectState) -> Value {
    let doc = state.doc("portraits");
    let entries = arr(doc, "entries");
    let mut counts = serde_json::Map::new();
    for status in STATUSES {
        counts.insert(status.to_string(), json!(0));
    }
    for entry in entries {
        if let Some(status) = field(entry, "status")
            && let Some(slot) = counts.get_mut(status)
        {
            *slot = json!(slot.as_u64().unwrap_or(0) + 1);
        }
    }
    let mut characters = Vec::new();
    for member in arr(state.doc("cast"), "characters") {
        let Some(id) = field(member, "id") else {
            continue;
        };
        let current = current_entry(doc, id).map(entry_view);
        characters.push(json!({"character": id, "current": current}));
    }
    json!({
        "project": state.slug,
        "schema": doc.get("schema").and_then(Value::as_str).unwrap_or(SCHEMA),
        "game": state.slug,
        "characters": characters,
        "sets": doc.get("sets").cloned().unwrap_or_else(|| json!([])),
        "total": entries.len(),
        "counts": counts,
    })
}

/// The `get_portrait_lock` payload for one character.
pub fn get_view(state: &ProjectState, character: &str) -> Value {
    let doc = state.doc("portraits");
    let mine: Vec<&Value> = arr(doc, "entries")
        .iter()
        .filter(|e| field(e, "character") == Some(character))
        .collect();
    json!({
        "project": state.slug,
        "character": character,
        "current": current_entry(doc, character).map(entry_view),
        "registered": mine.len(),
        "entries": mine.iter().map(|e| entry_view(e)).collect::<Vec<_>>(),
    })
}

/// The `list_portrait_history` payload: the character's full
/// chain in registration order, links intact.
pub fn history_view(state: &ProjectState, character: &str) -> Value {
    let doc = state.doc("portraits");
    let mine: Vec<Value> = arr(doc, "entries")
        .iter()
        .filter(|e| field(e, "character") == Some(character))
        .map(entry_view)
        .collect();
    json!({
        "project": state.slug,
        "character": character,
        "count": mine.len(),
        "entries": mine,
    })
}

/// The CLI `check` verdict for a path, as data: resolve by path
/// first, then by content (a byte-identical copy at another path
/// resolves to the registered entry). Verdict precedence follows
/// the chain: superseded (with the successor named), defective,
/// current; no match is `unknown`.
pub fn verdict_for_path(state: &ProjectState, path: &str) -> Value {
    let doc = state.doc("portraits");
    let entries = arr(doc, "entries");
    let mut found = entries.iter().find(|e| field(e, "path") == Some(path));
    if found.is_none()
        && let Some(digest) = sha256_file(Path::new(path))
    {
        found = entries
            .iter()
            .find(|e| field(e, "sha256") == Some(digest.as_str()));
    }
    let Some(entry) = found else {
        return json!({
            "project": state.slug,
            "path": path,
            "verdict": "unknown",
            "entry": null,
        });
    };
    let verdict = if let Some(by) = entry.get("superseded_by").and_then(Value::as_str) {
        return json!({
            "project": state.slug,
            "path": path,
            "verdict": "superseded",
            "entry": entry_view(entry),
            "superseded_by": by,
        });
    } else if field(entry, "status") == Some("defective") {
        "defective"
    } else {
        "current"
    };
    json!({
        "project": state.slug,
        "path": path,
        "verdict": verdict,
        "entry": entry_view(entry),
    })
}

/// Follow `supersedes` links transitively from `from`; true if
/// `needle` is reachable (a cycle if `needle` is the entry being
/// written). Bounded by the entry count — chains are short, and
/// the bound keeps a corrupt document from looping the gate.
fn supersedes_reaches(doc: &Value, from: &Value, needle: &str) -> bool {
    let bound = arr(doc, "entries").len() + 1;
    let mut frontier: Vec<String> = arr(from, "supersedes")
        .iter()
        .filter_map(|v| v.as_str().map(str::to_string))
        .collect();
    let mut steps = 0;
    while let Some(id) = frontier.pop() {
        if id == needle {
            return true;
        }
        steps += 1;
        if steps > bound {
            return false;
        }
        if let Some(entry) = find_by_id(doc, "entries", &id) {
            for next in arr(entry, "supersedes") {
                if let Some(next) = next.as_str() {
                    frontier.push(next.to_string());
                }
            }
        }
    }
    false
}

/// Gate judgment for a portrait registration. The tool layer has
/// already validated shapes and computed the pin from the frame's
/// bytes; everything here is semantic: the character must exist,
/// the frame must be new, supersession targets must be live
/// frames of the same character, and the working-set seat is
/// singular.
pub fn decide_register(state: &ProjectState, entry: &Value, asset: &Value) -> Result<(), Refusal> {
    crate::gate::check_asset_record(asset)?;
    let id = field(entry, "id").unwrap_or("");
    let character = field(entry, "character").unwrap_or("");
    let path = field(entry, "path").unwrap_or("");
    if field(entry, "sha256") != field(asset, "sha256") {
        return Err(refuse(
            "invalid_content_hash",
            "the portrait entry and its asset record must carry the same SHA-256 pin",
        ));
    }
    if !character_exists(state, character) {
        return Err(refuse(
            "unknown_character",
            format!("portrait names character '{character}', who is not in this project's cast"),
        ));
    }
    let doc = state.doc("portraits");
    if find_by_id(doc, "entries", id).is_some() {
        return Err(refuse(
            "already_registered",
            format!("a portrait with id '{id}' is already registered"),
        ));
    }
    if arr(doc, "entries")
        .iter()
        .any(|e| field(e, "path") == Some(path))
    {
        return Err(refuse(
            "already_registered",
            format!("the frame at {path} is already registered"),
        ));
    }
    if find_by_id(state.doc("assets"), "assets", id).is_some() {
        return Err(refuse(
            "asset_exists",
            format!(
                "an asset with id '{id}' already exists; portrait registration must be the asset's birth"
            ),
        ));
    }
    if let Some(ruling) = entry.get("ruling")
        && !ruling.is_null()
    {
        let quote = field(ruling, "quote").unwrap_or("");
        let date = field(ruling, "date").unwrap_or("");
        if quote.is_empty() || date.is_empty() {
            return Err(refuse(
                "ruling_required",
                "a ruling recorded on a portrait must carry both a date and the ruling text",
            ));
        }
    }
    let supersedes: Vec<&str> = arr(entry, "supersedes")
        .iter()
        .filter_map(Value::as_str)
        .collect();
    for target_id in &supersedes {
        if *target_id == id {
            return Err(refuse(
                "supersession_cycle",
                "a frame cannot supersede itself",
            ));
        }
        let Some(target) = find_by_id(doc, "entries", target_id) else {
            return Err(refuse(
                "unknown_portrait",
                format!("supersedes references unregistered portrait '{target_id}'"),
            ));
        };
        if field(target, "character") != Some(character) {
            return Err(refuse(
                "character_mismatch",
                format!(
                    "portrait '{target_id}' belongs to {}, not {character}; supersession stays inside one character's chain",
                    field(target, "character").unwrap_or("another character")
                ),
            ));
        }
        if target
            .get("superseded_by")
            .map(|v| !v.is_null())
            .unwrap_or(false)
        {
            return Err(refuse(
                "already_superseded",
                format!(
                    "portrait '{target_id}' already has a successor; a frame is replaced exactly once"
                ),
            ));
        }
        if field(target, "status") == Some("locked") {
            return Err(refuse(
                "frame_locked",
                format!(
                    "portrait '{target_id}' is {character}'s locked frame; superseding a lock is lock_portrait's act, with a ruling"
                ),
            ));
        }
        if supersedes_reaches(doc, target, id) {
            return Err(refuse(
                "supersession_cycle",
                format!("superseding '{target_id}' would close a cycle back to '{id}'"),
            ));
        }
    }
    if field(entry, "status") == Some("working-set") {
        for other in arr(doc, "entries") {
            if field(other, "character") == Some(character)
                && field(other, "status") == Some("locked")
            {
                return Err(refuse(
                    "frame_locked",
                    format!(
                        "{character} has a locked frame; registering a working-set frame behind it would shadow the lock — lock the new frame instead"
                    ),
                ));
            }
            if field(other, "character") == Some(character)
                && field(other, "status") == Some("working-set")
                && !supersedes.contains(&field(other, "id").unwrap_or(""))
            {
                return Err(refuse(
                    "working_set_conflict",
                    format!(
                        "{} is already {character}'s working-set frame; supersede it explicitly or register as a candidate",
                        field(other, "id").unwrap_or("another frame")
                    ),
                ));
            }
        }
    }
    Ok(())
}

/// Execute an approved registration: the asset record is born
/// in the assets document, the entry lands with its sequence
/// number, supersession targets are linked (and lose currency,
/// per the module's normalization rule), and the set joins the
/// project's vocabulary.
pub fn apply_register(state: &mut ProjectState, entry: Value, asset: Value) {
    let id = field(&entry, "id").unwrap_or("").to_string();
    let supersedes: Vec<String> = arr(&entry, "supersedes")
        .iter()
        .filter_map(|v| v.as_str().map(str::to_string))
        .collect();
    let set = field(&entry, "set").unwrap_or("").to_string();
    arr_mut(state.doc_mut("assets"), "assets").push(asset);
    let slug = state.slug.clone();
    let doc = state.doc_mut("portraits");
    {
        let object = doc
            .as_object_mut()
            .expect("portraits document is an object");
        object
            .entry("schema".to_string())
            .or_insert_with(|| json!(SCHEMA));
        object
            .entry("game".to_string())
            .or_insert_with(|| json!(slug));
    }
    let seq = arr(doc, "entries").len() + 1;
    let mut entry = entry;
    entry
        .as_object_mut()
        .expect("portrait entry is an object")
        .insert("seq".to_string(), json!(seq));
    arr_mut(doc, "entries").push(entry);
    for target_id in &supersedes {
        if let Some(target) = find_by_id_mut(doc, "entries", target_id) {
            let keep_defective = field(target, "status") == Some("defective");
            let object = target.as_object_mut().expect("portrait entry is an object");
            object.insert("superseded_by".to_string(), json!(id));
            if !keep_defective {
                object.insert("status".to_string(), json!("superseded"));
            }
        }
    }
    let sets = arr_mut(doc, "sets");
    if !sets.iter().any(|s| s.as_str() == Some(set.as_str())) {
        sets.push(json!(set));
        sets.sort_by(|a, b| a.as_str().cmp(&b.as_str()));
    }
}

/// Gate judgment for a lock. A lock is an Owner act: without a
/// ruling (date + text) there is nothing to record, and the gate
/// refuses — this is the registry's core invariant, that lock
/// state is ruling provenance, not a toggle.
pub fn decide_lock(
    state: &ProjectState,
    character: &str,
    entry_id: &str,
    ruling: &Value,
) -> Result<(), Refusal> {
    if !character_exists(state, character) {
        return Err(refuse(
            "unknown_character",
            format!("'{character}' is not in this project's cast"),
        ));
    }
    let quote = ruling.get("quote").and_then(Value::as_str).unwrap_or("");
    let date = ruling.get("date").and_then(Value::as_str).unwrap_or("");
    if quote.is_empty() || date.is_empty() {
        return Err(refuse(
            "ruling_required",
            "locking a portrait requires the Owner ruling that sets it: a ruling with both a date and the ruling text",
        ));
    }
    let doc = state.doc("portraits");
    let Some(entry) = find_by_id(doc, "entries", entry_id) else {
        return Err(refuse(
            "unknown_portrait",
            format!("no registered portrait named '{entry_id}'"),
        ));
    };
    if field(entry, "character") != Some(character) {
        return Err(refuse(
            "character_mismatch",
            format!(
                "portrait '{entry_id}' belongs to {}, not {character}",
                field(entry, "character").unwrap_or("another character")
            ),
        ));
    }
    if field(entry, "status") == Some("defective") {
        return Err(refuse(
            "frame_defective",
            format!(
                "portrait '{entry_id}' is recorded defective; a defective frame cannot be locked"
            ),
        ));
    }
    if field(entry, "status") == Some("superseded")
        || entry
            .get("superseded_by")
            .map(|v| !v.is_null())
            .unwrap_or(false)
    {
        return Err(refuse(
            "already_superseded",
            format!(
                "portrait '{entry_id}' has already been superseded; lock its successor instead"
            ),
        ));
    }
    Ok(())
}

/// Execute an approved lock: the character's standing locked /
/// working-set frames are superseded by the locked one (and join
/// its supersedes list), and the entry takes `locked` with the
/// ruling recorded.
pub fn apply_lock(state: &mut ProjectState, character: &str, entry_id: &str, ruling: Value) {
    let doc = state.doc_mut("portraits");
    let mut superseded_now: Vec<String> = Vec::new();
    for other in arr_mut(doc, "entries").iter_mut() {
        let is_other = field(other, "id") != Some(entry_id);
        let is_current = matches!(field(other, "status"), Some("locked") | Some("working-set"));
        if is_other && field(other, "character") == Some(character) && is_current {
            let object = other.as_object_mut().expect("portrait entry is an object");
            object.insert("status".to_string(), json!("superseded"));
            object.insert("superseded_by".to_string(), json!(entry_id));
            superseded_now.push(field(other, "id").unwrap_or("").to_string());
        }
    }
    if let Some(entry) = find_by_id_mut(doc, "entries", entry_id) {
        let object = entry.as_object_mut().expect("portrait entry is an object");
        object.insert("status".to_string(), json!("locked"));
        object.insert("superseded_by".to_string(), Value::Null);
        object.insert("ruling".to_string(), ruling);
        let list = arr_mut(entry, "supersedes");
        for id in superseded_now {
            if !list.iter().any(|v| v.as_str() == Some(id.as_str())) {
                list.push(json!(id));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::Store;

    fn frame(tag: &str, bytes: &[u8]) -> (String, String) {
        let dir =
            std::env::temp_dir().join(format!("ardor-portraits-test-{tag}-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("frame.png");
        std::fs::write(&path, bytes).unwrap();
        let digest = sha256_file(&path).expect("hash temp frame");
        (path.to_string_lossy().to_string(), digest)
    }

    #[test]
    fn sha256_file_matches_known_digest() {
        // SHA-256 of the empty string and of "abc" are public
        // test vectors; the streaming reader must reproduce them.
        let (path, digest) = frame("empty", b"");
        assert_eq!(
            digest,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        let (path2, digest2) = frame("abc", b"abc");
        assert_eq!(
            digest2,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert!(sha256_file(Path::new(&path)).is_some());
        assert!(sha256_file(Path::new("/nonexistent/frame.png")).is_none());
        let _ = std::fs::remove_file(&path2);
    }

    #[test]
    fn current_prefers_locked_then_working_set() {
        let root =
            std::env::temp_dir().join(format!("ardor-portraits-pick-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let store = Store::open(&root).unwrap();
        let mut state = Store::blank_state("demo");
        let doc = state.doc_mut("portraits");
        *doc = json!({
            "entries": [
                {"id": "a", "character": "sol", "status": "working-set", "superseded_by": null},
                {"id": "b", "character": "sol", "status": "locked", "superseded_by": null},
                {"id": "c", "character": "sol", "status": "locked", "superseded_by": "b"}
            ]
        });
        assert_eq!(
            field(current_entry(&state.doc("portraits"), "sol").unwrap(), "id"),
            Some("b"),
            "a superseded locked frame never wins over the standing lock"
        );
        drop(store);
        let _ = std::fs::remove_dir_all(&root);
    }
}
