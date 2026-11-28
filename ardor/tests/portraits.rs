//! Portrait-lock registry (M-1): validation and adversarial
//! tests for register_portrait / lock_portrait /
//! get_portrait_lock / list_portrait_locks /
//! list_portrait_history, driven through `tools::dispatch`
//! against a scratch store with real frame files on disk.
//! Roughly half validation, half adversarial, per estate rule.
//! The adversarial half includes the verifier clause's positive
//! control: a planted wrong-sha entry that the read tools must
//! catch as `mismatch`.

use ardor::{portraits, store::Store, tools};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

struct Fixture {
    root: PathBuf,
    frames: PathBuf,
    store: Store,
}

impl Fixture {
    fn new(tag: &str) -> Self {
        let root =
            std::env::temp_dir().join(format!("ardor-portraits-{tag}-{}", std::process::id()));
        assert!(
            !root.exists(),
            "unique fixture must not overwrite another run"
        );
        let frames = root.join("frames");
        fs::create_dir_all(&frames).unwrap();
        let mut store = Store::open(&root).unwrap();
        call(
            &mut store,
            "create_project",
            json!({"name": "Portrait Demo", "slug": "demo", "rating": "all_ages"}),
        );
        for (id, name) in [("solenne", "Solenne"), ("wrenna", "Wrenna")] {
            call(
                &mut store,
                "upsert_character",
                json!({"project": "demo", "character": {"id": id, "name": name}}),
            );
        }
        Self {
            root,
            frames,
            store,
        }
    }

    /// Write a frame file with the given bytes; returns its path.
    fn frame(&self, name: &str, bytes: &[u8]) -> String {
        let path = self.frames.join(name);
        fs::write(&path, bytes).unwrap();
        path.to_string_lossy().to_string()
    }

    fn register(&mut self, portrait: Value) -> Value {
        call(
            &mut self.store,
            "register_portrait",
            json!({"project": "demo", "portrait": portrait}),
        )
    }

    fn refused(&mut self, tool: &str, args: Value, code: &str) {
        let before = tree(&self.root);
        let error = tools::dispatch(&mut self.store, tool, &args).expect_err("refusal");
        assert_eq!(error.code, code, "{}", error.message);
        assert_eq!(before, tree(&self.root), "refusal must preserve every file");
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn call(store: &mut Store, name: &str, args: Value) -> Value {
    tools::dispatch(store, name, &args)
        .unwrap_or_else(|e| panic!("{name}: {} {}", e.code, e.message))
}

fn tree(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut out = BTreeMap::new();
    for entry in fs::read_dir(root).unwrap() {
        let p = entry.unwrap().path();
        if p.is_dir() {
            out.extend(tree(&p));
        } else {
            out.insert(p.clone(), fs::read(p).unwrap());
        }
    }
    out
}

const RULING: &str = "Owner lock, test fixture";

// ---------------- validation ----------------

#[test]
fn validation_register_computes_pin_and_borns_asset() {
    let mut f = Fixture::new("register");
    // "abc" is a public SHA-256 test vector: the registered pin
    // must be the digest of the bytes, computed by the server.
    let path = f.frame("sol-a.png", b"abc");
    let out = f.register(json!({
        "id": "sol-a", "character": "solenne", "set": "set1", "path": path
    }));
    let entry = &out["portrait"];
    assert_eq!(
        entry["sha256"],
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    assert_eq!(entry["status"], "candidate");
    assert_eq!(entry["seq"], 1);
    assert_eq!(entry["superseded_by"], Value::Null);
    // The asset record is born alongside, lane-stamped.
    let asset = &out["asset"];
    assert_eq!(asset["kind"], "sprite");
    assert_eq!(asset["origin_lane"], "ntranch");
    assert_eq!(asset["content_label"], "sfw");
    assert_eq!(asset["sha256"], entry["sha256"]);
    let assets = call(&mut f.store, "list_assets", json!({"project": "demo"}));
    assert_eq!(assets["assets"].as_array().unwrap().len(), 1);
    // A candidate is not current; the pin verifies live.
    let view = call(
        &mut f.store,
        "get_portrait_lock",
        json!({"project": "demo", "character": "solenne"}),
    );
    assert_eq!(view["current"], Value::Null);
    assert_eq!(view["entries"][0]["sha_state"], "verified");
}

#[test]
fn validation_lock_flow_supersedes_working_set() {
    let mut f = Fixture::new("lockflow");
    let a = f.frame("sol-a.png", b"frame a");
    let b = f.frame("sol-b.png", b"frame b");
    f.register(json!({
        "id": "sol-a", "character": "solenne", "set": "set1", "path": a, "status": "working-set"
    }));
    f.register(json!({
        "id": "sol-b", "character": "solenne", "set": "canvas-fresh", "path": b,
        "ruling": {"date": "2026-11-27", "quote": "working set adopted"}
    }));
    let locked = call(
        &mut f.store,
        "lock_portrait",
        json!({
            "project": "demo", "character": "solenne", "portrait": "sol-b",
            "ruling": {"date": "2026-11-28", "quote": RULING}
        }),
    );
    assert_eq!(locked["portrait"]["status"], "locked");
    assert_eq!(locked["portrait"]["ruling"]["quote"], RULING);
    assert_eq!(locked["portrait"]["supersedes"], json!(["sol-a"]));
    let view = call(
        &mut f.store,
        "get_portrait_lock",
        json!({"project": "demo", "character": "solenne"}),
    );
    assert_eq!(view["current"]["id"], "sol-b");
    let a_now = view["entries"]
        .as_array()
        .unwrap()
        .iter()
        .find(|e| e["id"] == "sol-a")
        .unwrap();
    assert_eq!(a_now["status"], "superseded");
    assert_eq!(a_now["superseded_by"], "sol-b");
    // The manifest view agrees, per character.
    let locks = call(
        &mut f.store,
        "list_portrait_locks",
        json!({"project": "demo"}),
    );
    assert_eq!(locks["schema"], portraits::SCHEMA);
    assert_eq!(locks["total"], 2);
    assert_eq!(locks["counts"]["locked"], 1);
    assert_eq!(locks["counts"]["superseded"], 1);
    let sol = locks["characters"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["character"] == "solenne")
        .unwrap();
    assert_eq!(sol["current"]["id"], "sol-b");
    assert_eq!(sol["current"]["sha_state"], "verified");
    let wre = locks["characters"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["character"] == "wrenna")
        .unwrap();
    assert_eq!(wre["current"], Value::Null);
    assert_eq!(locks["sets"], json!(["canvas-fresh", "set1"]));
}

#[test]
fn validation_check_verdicts_by_path_and_content() {
    let mut f = Fixture::new("verdicts");
    let a = f.frame("sol-a.png", b"frame a");
    let b = f.frame("sol-b.png", b"frame b");
    let d = f.frame("sol-d.png", b"frame d");
    f.register(json!({"id": "sol-a", "character": "solenne", "set": "set1", "path": a}));
    f.register(json!({
        "id": "sol-b", "character": "solenne", "set": "edge-cure", "path": b,
        "supersedes": ["sol-a"]
    }));
    f.register(json!({
        "id": "sol-d", "character": "solenne", "set": "set1", "path": d, "status": "defective"
    }));
    let verdict = |f: &mut Fixture, path: &str| {
        call(
            &mut f.store,
            "get_portrait_lock",
            json!({"project": "demo", "path": path}),
        )
    };
    assert_eq!(verdict(&mut f, &b)["verdict"], "current");
    let sup = verdict(&mut f, &a);
    assert_eq!(sup["verdict"], "superseded");
    assert_eq!(sup["superseded_by"], "sol-b");
    assert_eq!(verdict(&mut f, &d)["verdict"], "defective");
    // A byte-identical copy at a new path resolves by content.
    let copy = f.frame("sol-a-copy.png", b"frame a");
    let by_content = verdict(&mut f, &copy);
    assert_eq!(by_content["verdict"], "superseded");
    assert_eq!(by_content["entry"]["id"], "sol-a");
    // An unregistered file is unknown.
    let stray = f.frame("stray.png", b"never registered");
    assert_eq!(verdict(&mut f, &stray)["verdict"], "unknown");
}

#[test]
fn validation_defective_keeps_its_status_when_superseded() {
    // The Tamsin heightfix shape: the cure supersedes a defective
    // frame; the defect stays on the record, the link is set.
    let mut f = Fixture::new("defective");
    let d = f.frame("tam-d.png", b"oversized");
    let h = f.frame("tam-h.png", b"height fixed");
    f.register(json!({
        "id": "tam-d", "character": "wrenna", "set": "canvas-fresh", "path": d,
        "status": "defective",
        "ruling": {"date": "2026-11-27", "quote": "figure generated oversized"}
    }));
    f.register(json!({
        "id": "tam-h", "character": "wrenna", "set": "heightfix", "path": h,
        "status": "working-set", "supersedes": ["tam-d"]
    }));
    let history = call(
        &mut f.store,
        "list_portrait_history",
        json!({"project": "demo", "character": "wrenna"}),
    );
    assert_eq!(history["count"], 2);
    let d_now = &history["entries"][0];
    assert_eq!(d_now["status"], "defective");
    assert_eq!(d_now["superseded_by"], "tam-h");
    assert_eq!(history["entries"][1]["status"], "working-set");
    let view = call(
        &mut f.store,
        "get_portrait_lock",
        json!({"project": "demo", "character": "wrenna"}),
    );
    assert_eq!(view["current"]["id"], "tam-h");
}

#[test]
fn validation_working_set_registration_supersedes_predecessor() {
    let mut f = Fixture::new("ws");
    let a = f.frame("sol-a.png", b"frame a");
    let b = f.frame("sol-b.png", b"frame b");
    f.register(json!({
        "id": "sol-a", "character": "solenne", "set": "set1", "path": a, "status": "working-set"
    }));
    f.register(json!({
        "id": "sol-b", "character": "solenne", "set": "canvas-fresh", "path": b,
        "status": "working-set", "supersedes": ["sol-a"]
    }));
    let view = call(
        &mut f.store,
        "get_portrait_lock",
        json!({"project": "demo", "character": "solenne"}),
    );
    assert_eq!(view["current"]["id"], "sol-b");
    assert_eq!(view["entries"][0]["status"], "superseded");
}

// ---------------- adversarial ----------------

#[test]
fn adversarial_lock_without_ruling_is_refused() {
    let mut f = Fixture::new("noruling");
    let a = f.frame("sol-a.png", b"frame a");
    f.register(json!({"id": "sol-a", "character": "solenne", "set": "set1", "path": a}));
    // No ruling argument at all.
    f.refused(
        "lock_portrait",
        json!({"project": "demo", "character": "solenne", "portrait": "sol-a"}),
        "ruling_required",
    );
    // An empty ruling object.
    f.refused(
        "lock_portrait",
        json!({
            "project": "demo", "character": "solenne", "portrait": "sol-a", "ruling": {}
        }),
        "ruling_required",
    );
    // A quote with no date.
    f.refused(
        "lock_portrait",
        json!({
            "project": "demo", "character": "solenne", "portrait": "sol-a",
            "ruling": {"quote": "lock it"}
        }),
        "ruling_required",
    );
    // And the frame is still a mere candidate afterwards.
    let view = call(
        &mut f.store,
        "get_portrait_lock",
        json!({"project": "demo", "character": "solenne"}),
    );
    assert_eq!(view["entries"][0]["status"], "candidate");
}

#[test]
fn adversarial_planted_wrong_sha_is_caught() {
    // Positive control (verifier clause): corrupt one stored pin
    // on disk, reload, and the read tools must flag exactly that
    // entry as mismatch while the honest entry stays verified.
    let mut f = Fixture::new("planted");
    let a = f.frame("sol-a.png", b"frame a");
    let b = f.frame("sol-b.png", b"frame b");
    f.register(json!({"id": "sol-a", "character": "solenne", "set": "set1", "path": a}));
    f.register(json!({"id": "sol-b", "character": "solenne", "set": "set1", "path": b}));
    let doc_path = f.root.join("demo").join("portraits.json");
    let mut doc: Value = serde_json::from_str(&fs::read_to_string(&doc_path).unwrap()).unwrap();
    doc["entries"][0]["sha256"] =
        json!("0000000000000000000000000000000000000000000000000000000000000000");
    fs::write(&doc_path, serde_json::to_string_pretty(&doc).unwrap()).unwrap();
    let mut reloaded = Store::open(&f.root).unwrap();
    let history = call(
        &mut reloaded,
        "list_portrait_history",
        json!({"project": "demo", "character": "solenne"}),
    );
    assert_eq!(history["entries"][0]["id"], "sol-a");
    assert_eq!(history["entries"][0]["sha_state"], "mismatch");
    assert_eq!(history["entries"][1]["sha_state"], "verified");
    // Tampering with the frame bytes instead is caught the same
    // way: rewrite sol-b's file and its pin no longer matches.
    fs::write(&b, b"tampered bytes").unwrap();
    let history = call(
        &mut reloaded,
        "list_portrait_history",
        json!({"project": "demo", "character": "solenne"}),
    );
    assert_eq!(history["entries"][1]["sha_state"], "mismatch");
}

#[test]
fn adversarial_supersession_cannot_cycle_or_fork() {
    let mut f = Fixture::new("cycle");
    let a = f.frame("sol-a.png", b"frame a");
    let b = f.frame("sol-b.png", b"frame b");
    let c = f.frame("sol-c.png", b"frame c");
    let d = f.frame("sol-d.png", b"frame d");
    f.register(json!({"id": "sol-a", "character": "solenne", "set": "set1", "path": a}));
    f.register(json!({
        "id": "sol-b", "character": "solenne", "set": "edge-cure", "path": b,
        "supersedes": ["sol-a"]
    }));
    // Self-supersession is a cycle of length one.
    f.refused(
        "register_portrait",
        json!({
            "project": "demo",
            "portrait": {"id": "sol-c", "character": "solenne", "set": "set1", "path": c,
                         "supersedes": ["sol-c"]}
        }),
        "supersession_cycle",
    );
    // A frame is replaced exactly once: sol-a already has its
    // successor, so a second successor forks the chain.
    f.refused(
        "register_portrait",
        json!({
            "project": "demo",
            "portrait": {"id": "sol-d", "character": "solenne", "set": "set1", "path": d,
                         "supersedes": ["sol-a"]}
        }),
        "already_superseded",
    );
    // Closing the loop the other way — re-registering the old
    // frame's path as the head of the chain — is refused as a
    // duplicate before it can point backwards.
    f.refused(
        "register_portrait",
        json!({
            "project": "demo",
            "portrait": {"id": "sol-a2", "character": "solenne", "set": "set1", "path": a,
                         "supersedes": ["sol-b"]}
        }),
        "already_registered",
    );
    // And a superseded frame cannot be locked back to the head.
    f.refused(
        "lock_portrait",
        json!({
            "project": "demo", "character": "solenne", "portrait": "sol-a",
            "ruling": {"date": "2026-11-28", "quote": "re-lock the old one"}
        }),
        "already_superseded",
    );
}

#[test]
fn adversarial_register_rejections() {
    let mut f = Fixture::new("reject");
    let a = f.frame("sol-a.png", b"frame a");
    // A frame that does not exist cannot be pinned.
    f.refused(
        "register_portrait",
        json!({
            "project": "demo",
            "portrait": {"id": "ghost", "character": "solenne", "set": "set1",
                         "path": "/nonexistent/frame.png"}
        }),
        "frame_unreadable",
    );
    // A directory is not a frame.
    let dir = f.frames.to_string_lossy().to_string();
    f.refused(
        "register_portrait",
        json!({
            "project": "demo",
            "portrait": {"id": "dir", "character": "solenne", "set": "set1", "path": dir}
        }),
        "frame_unreadable",
    );
    // A character outside the cast.
    f.refused(
        "register_portrait",
        json!({
            "project": "demo",
            "portrait": {"id": "sol-a", "character": "petra", "set": "set1", "path": a.clone()}
        }),
        "unknown_character",
    );
    f.register(json!({"id": "sol-a", "character": "solenne", "set": "set1", "path": a}));
    // Duplicate id, then the same path under a fresh id.
    let b = f.frame("sol-b.png", b"frame b");
    f.refused(
        "register_portrait",
        json!({
            "project": "demo",
            "portrait": {"id": "sol-a", "character": "solenne", "set": "set1", "path": b}
        }),
        "already_registered",
    );
    let a_path = f.frames.join("sol-a.png").to_string_lossy().to_string();
    f.refused(
        "register_portrait",
        json!({
            "project": "demo",
            "portrait": {"id": "sol-b", "character": "solenne", "set": "set1", "path": a_path}
        }),
        "already_registered",
    );
    // Locked is not a registration status (params-layer refusal).
    f.refused(
        "register_portrait",
        json!({
            "project": "demo",
            "portrait": {"id": "sol-z", "character": "solenne", "set": "set1",
                         "path": f.frame("sol-z.png", b"frame z"), "status": "locked"}
        }),
        "invalid_params",
    );
}

#[test]
fn adversarial_lock_rejections() {
    let mut f = Fixture::new("lockreject");
    let a = f.frame("sol-a.png", b"frame a");
    let d = f.frame("sol-d.png", b"frame d");
    let w = f.frame("wre-a.png", b"frame w");
    f.register(json!({
        "id": "sol-a", "character": "solenne", "set": "set1", "path": a, "status": "working-set"
    }));
    f.register(json!({
        "id": "sol-d", "character": "solenne", "set": "set1", "path": d, "status": "defective"
    }));
    f.register(json!({"id": "wre-a", "character": "wrenna", "set": "set1", "path": w}));
    let ruling = json!({"date": "2026-11-28", "quote": RULING});
    // A defective frame cannot be locked.
    f.refused(
        "lock_portrait",
        json!({
            "project": "demo", "character": "solenne", "portrait": "sol-d", "ruling": ruling
        }),
        "frame_defective",
    );
    // Another character's frame cannot be locked under this name.
    f.refused(
        "lock_portrait",
        json!({
            "project": "demo", "character": "solenne", "portrait": "wre-a", "ruling": ruling
        }),
        "character_mismatch",
    );
    // An unknown portrait id.
    f.refused(
        "lock_portrait",
        json!({
            "project": "demo", "character": "solenne", "portrait": "nope", "ruling": ruling
        }),
        "unknown_portrait",
    );
    // Locking sol-a works; afterwards a plain registration may
    // not supersede the lock — that is lock_portrait's act.
    call(
        &mut f.store,
        "lock_portrait",
        json!({
            "project": "demo", "character": "solenne", "portrait": "sol-a", "ruling": ruling
        }),
    );
    let e = f.frame("sol-e.png", b"frame e");
    f.refused(
        "register_portrait",
        json!({
            "project": "demo",
            "portrait": {"id": "sol-e", "character": "solenne", "set": "set1", "path": e,
                         "supersedes": ["sol-a"]}
        }),
        "frame_locked",
    );
}

#[test]
fn adversarial_working_set_conflict_is_refused() {
    let mut f = Fixture::new("wsconflict");
    let a = f.frame("sol-a.png", b"frame a");
    let b = f.frame("sol-b.png", b"frame b");
    f.register(json!({
        "id": "sol-a", "character": "solenne", "set": "set1", "path": a, "status": "working-set"
    }));
    // A second working-set frame that does not supersede the
    // first would leave "current" ambiguous.
    f.refused(
        "register_portrait",
        json!({
            "project": "demo",
            "portrait": {"id": "sol-b", "character": "solenne", "set": "set1", "path": b,
                         "status": "working-set"}
        }),
        "working_set_conflict",
    );
}
