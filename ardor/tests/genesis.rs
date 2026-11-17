//! Stage 1: six validation and six adversarial tests.
use ardor::{
    genesis,
    model::{canonical, sha256_hex},
    store::Store,
    tools,
};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

struct Fixture {
    root: PathBuf,
    store: Store,
}
impl Fixture {
    fn new(tag: &str) -> Self {
        let root = std::env::temp_dir().join(format!("ardor-genesis-{tag}-{}", std::process::id()));
        assert!(
            !root.exists(),
            "unique fixture must not overwrite another run"
        );
        let mut store = Store::open(&root).unwrap();
        call(
            &mut store,
            "create_project",
            json!({"name":"Candidate","slug":"demo","rating":"all_ages","channel_targets":["itch_io"]}),
        );
        Self { root, store }
    }
    fn read(&mut self) -> Value {
        call(
            &mut self.store,
            "get_genesis_state",
            json!({"project":"demo"}),
        )
    }
    fn rule(&mut self) {
        let p = genesis::profile();
        let rulings: Vec<_> = genesis::ITEMS
            .iter()
            .map(|id| json!({"item":id,"state":"confirmed","value":p["items"][id]}))
            .collect();
        call(
            &mut self.store,
            "record_default_ruling",
            json!({"project":"demo","rulings":rulings,"ruling_ref":"OWNER-test-block"}),
        );
    }
    fn record(&mut self, sheet: Value, adopt: bool) -> Value {
        call(
            &mut self.store,
            "record_clearance_sheet",
            json!({"project":"demo","sheet":sheet,"adopt":adopt,"ruling_ref":"OWNER-test-adoption"}),
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
        fs::remove_dir_all(&self.root).unwrap();
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
fn clear() -> Value {
    let legs:Vec<_>=["package:crates.io","namespace:github","namespace:mcp","storefront:itch_io","trademark","domain"].iter().map(|id|json!({
        "id":id,"finding":"No collision in test evidence (fixture only)","verified":true,
        "evidence":[{"url":"https://example.test/primary-record","grade":"primary"}],"owed_steps":[],"outcome":"CLEAR"
    })).collect();
    json!({"candidate":"Candidate","verdict":"CLEAR","legs":legs,"notes":[]})
}

#[test]
fn validation_inherited_profile_and_read_only_census() {
    let mut f = Fixture::new("inherited");
    let before = tree(&f.root);
    let state = f.read();
    assert_eq!(state["census"]["complete"], false);
    assert_eq!(state["sheet"]["defaults"].as_array().unwrap().len(), 7);
    assert!(
        state["sheet"]["defaults"]
            .as_array()
            .unwrap()
            .iter()
            .all(|i| i["state"] == "inherited" && i["ruling_ref"].is_null())
    );
    assert_eq!(before, tree(&f.root));
}
#[test]
fn validation_block_ruling_and_re_ruling_preserve_history() {
    let mut f = Fixture::new("block");
    f.rule();
    let before = f.read()["sheet"]["ledger"].clone();
    let result = call(
        &mut f.store,
        "record_default_ruling",
        json!({"project":"demo","ruling_ref":"OWNER-revision","rulings":[{"item":"voice","state":"re-ruled","value":"No narration"}]}),
    );
    assert_eq!(result["sheet"]["ledger"][0], before[0]);
    assert_eq!(result["sheet"]["ledger"][1]["id"], "D-2");
    assert_eq!(result["sheet"]["defaults"][4]["value"], "No narration");
}
#[test]
fn validation_complete_genesis_persists_and_hashes_runs() {
    let mut f = Fixture::new("complete");
    f.rule();
    let state = f.record(clear(), true);
    assert_eq!(state["census"]["complete"], true);
    assert_eq!(state["runs"][1]["verdict"], "PASS");
    for run in state["runs"].as_array().unwrap() {
        assert_eq!(
            run["output_artifact_hashes"]["sheet"],
            sha256_hex(&canonical(&run["artifact"]))
        );
    }
    let mut reopened = Store::open(&f.root).unwrap();
    assert_eq!(
        state,
        call(
            &mut reopened,
            "get_genesis_state",
            json!({"project":"demo"})
        )
    );
    let file = fs::read_to_string(f.root.join("demo/genesis.json")).unwrap();
    assert_eq!(
        file,
        format!(
            "{}\n",
            canonical(&serde_json::from_str::<Value>(&file).unwrap())
        )
    );
}
#[test]
fn validation_secondary_evidence_is_notes_with_explicit_acceptance() {
    let mut f = Fixture::new("notes");
    f.rule();
    let mut sheet = clear();
    sheet["legs"][4]["verified"] = json!(false);
    sheet["legs"][4]["evidence"][0]["grade"] = json!("secondary");
    sheet["legs"][4]["owed_steps"] =
        json!(["Search direct national trademark register before filing"]);
    sheet["verdict"] = json!("CLEAR WITH NOTES");
    assert_eq!(f.record(sheet.clone(), false)["census"]["complete"], false);
    assert_eq!(f.record(sheet, true)["census"]["complete"], true);
}
#[test]
fn validation_two_identical_histories_have_identical_bytes() {
    let mut a = Fixture::new("bake-a");
    let mut b = Fixture::new("bake-b");
    a.rule();
    b.rule();
    a.record(clear(), true);
    b.record(clear(), true);
    assert_eq!(
        fs::read(a.root.join("demo/genesis.json")).unwrap(),
        fs::read(b.root.join("demo/genesis.json")).unwrap()
    );
    for file in a.read()["sheet"]["seed_manifest"].as_array().unwrap() {
        let text = fs::read_to_string(
            a.root
                .join("demo/repo-seed")
                .join(file["path"].as_str().unwrap()),
        )
        .unwrap();
        assert_eq!(file["sha256"], sha256_hex(&text));
    }
}
#[test]
fn validation_collision_pick_can_be_superseded_and_legacy_is_review() {
    let mut f = Fixture::new("supersede");
    let mut sheet = clear();
    sheet["legs"][0]["outcome"] = json!("COLLISION");
    sheet["verdict"] = json!("COLLISION");
    f.record(sheet, false);
    let mut next = clear();
    next["candidate"] = json!("Second candidate");
    f.rule();
    assert_eq!(f.record(next, true)["census"]["complete"], true);
    call(
        &mut f.store,
        "update_project_meta",
        json!({"project":"demo","name":"Second candidate"}),
    );
    let legacy = Store::blank_state("legacy");
    assert_eq!(genesis::census(&legacy)["complete"], false);
}
#[test]
fn adversarial_default_rulings_are_atomic_and_safety_is_immutable() {
    let mut f = Fixture::new("bad-rulings");
    for (rulings, code) in [
        (json!([]), "invalid_default_ruling"),
        (
            json!([{"item":"bogus","state":"confirmed","value":"x"}]),
            "invalid_default_ruling",
        ),
        (
            json!([{"item":"voice","state":"confirmed","value":"changed"}]),
            "default_value_changed",
        ),
        (
            json!([{"item":"accessibility","state":"re-ruled","value":"disabled"}]),
            "safety_surface_immutable",
        ),
        (
            json!([{"item":"voice","state":"inherited","value":"x"}]),
            "invalid_params",
        ),
        (
            json!([{"item":"voice","state":"re-ruled","value":"x","extra":true}]),
            "invalid_params",
        ),
        (
            json!([{"item":"voice","state":"re-ruled","value":"x"},{"item":"voice","state":"re-ruled","value":"y"}]),
            "invalid_default_ruling",
        ),
    ] {
        f.refused(
            "record_default_ruling",
            json!({"project":"demo","rulings":rulings,"ruling_ref":"OWNER-x"}),
            code,
        );
    }
}
#[test]
fn adversarial_evidence_cannot_claim_primary_verification() {
    let mut f = Fixture::new("evidence");
    for (verified, owed) in [
        (true, json!([])),
        (true, json!(["Search register"])),
        (false, json!([])),
    ] {
        let mut sheet = clear();
        sheet["legs"][4]["evidence"][0]["grade"] = json!("secondary");
        sheet["legs"][4]["verified"] = json!(verified);
        sheet["legs"][4]["owed_steps"] = owed;
        f.refused(
            "record_clearance_sheet",
            json!({"project":"demo","sheet":sheet,"ruling_ref":"OWNER-x"}),
            "clearance_evidence_unverified",
        );
    }
}
#[test]
fn adversarial_missing_duplicate_unknown_and_evidence_free_legs() {
    let mut f = Fixture::new("legs");
    let mut missing = clear();
    missing["legs"].as_array_mut().unwrap().pop();
    f.refused(
        "record_clearance_sheet",
        json!({"project":"demo","sheet":missing,"ruling_ref":"OWNER-x"}),
        "name_conditional",
    );
    for change in [json!("duplicate"), json!("unknown"), json!("empty")] {
        let mut sheet = clear();
        if change == "duplicate" {
            sheet["legs"][1] = sheet["legs"][0].clone();
        }
        if change == "unknown" {
            sheet["legs"][0]["id"] = json!("unknown");
        }
        if change == "empty" {
            sheet["legs"][0]["evidence"] = json!([]);
        }
        f.refused(
            "record_clearance_sheet",
            json!({"project":"demo","sheet":sheet,"ruling_ref":"OWNER-x"}),
            "invalid_clearance",
        );
    }
}
#[test]
fn adversarial_collision_and_conditional_name_refuse_adoption() {
    let mut f = Fixture::new("adoption");
    f.refused(
        "update_project_meta",
        json!({"project":"demo","name":"Public name"}),
        "name_conditional",
    );
    let mut sheet = clear();
    sheet["legs"][0]["outcome"] = json!("COLLISION");
    sheet["verdict"] = json!("COLLISION");
    f.refused(
        "record_clearance_sheet",
        json!({"project":"demo","sheet":sheet,"adopt":true,"ruling_ref":"OWNER-x"}),
        "clearance_collision",
    );
    f.record(sheet, false);
    f.refused(
        "update_project_meta",
        json!({"project":"demo","name":"Public name"}),
        "clearance_collision",
    );
}
#[test]
fn adversarial_verdict_notes_and_malformed_params() {
    let mut f = Fixture::new("verdict");
    let mut sheet = clear();
    sheet["notes"] = json!(["Unresolved note"]);
    f.refused(
        "record_clearance_sheet",
        json!({"project":"demo","sheet":sheet,"ruling_ref":"OWNER-x"}),
        "clearance_verdict_mismatch",
    );
    f.refused(
        "record_clearance_sheet",
        json!({"project":"demo","sheet":clear(),"adopt":true,"ruling_ref":" "}),
        "ruling_ref_required",
    );
    f.refused(
        "record_clearance_sheet",
        json!({"project":"demo","sheet":clear(),"adopt":"yes","ruling_ref":"OWNER-x"}),
        "invalid_params",
    );
    f.refused(
        "get_genesis_state",
        json!({"project":"../demo"}),
        "invalid_params",
    );
    f.refused(
        "get_genesis_state",
        json!({"project":"demo","extra":true}),
        "invalid_params",
    );
}
#[test]
fn adversarial_seed_tamper_is_detected_without_writes() {
    let mut f = Fixture::new("tamper");
    f.rule();
    f.record(clear(), true);
    let path = f.root.join("demo/genesis.json");
    let original = fs::read(&path).unwrap();
    for field in [
        "defaults",
        "ledger",
        "runs",
        "input_hash",
        "verdict",
        "missing_runs",
    ] {
        let mut g: Value = serde_json::from_slice(&original).unwrap();
        match field {
            "defaults" => g["defaults"][0]["value"] = json!("forged"),
            "ledger" => g["ledger"][0]["ruling_ref"] = json!("forged"),
            "input_hash" => g["runs"][0]["input_state_hash"] = json!("0".repeat(64)),
            "verdict" => g["runs"][0]["verdict"] = json!("PASS"),
            "missing_runs" => g["runs"] = json!([]),
            _ => g["runs"][0]["artifact"]["candidate"] = json!("forged"),
        }
        fs::write(&path, canonical(&g)).unwrap();
        let before = tree(&f.root);
        assert_eq!(f.read()["census"]["complete"], false);
        assert_eq!(tree(&f.root), before);
    }
    fs::write(&path, original).unwrap();
    fs::write(f.root.join("demo/repo-seed/README.md"), "tamper").unwrap();
    f.refused(
        "get_genesis_state",
        json!({"project":"demo"}),
        "seed_manifest_mismatch",
    );
    let safety = call(&mut f.store, "audit_safety", json!({"project":"demo"}));
    assert_eq!(safety["audit"]["ok"], true);
}
