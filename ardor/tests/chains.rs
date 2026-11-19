//! Stage 4a: seven validation and six adversarial tests for the
//! `keeper-chain 1.0` registry (ARDOR-PIPELINES.md §4): manual
//! intake form, the typed `chains` document, the validator and
//! its §7 run records, and the gate's typed refusals — every
//! refusal proven to leave the tree byte-identical.

use ardor::{model::sha256_hex, store::Store, tools};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

const PROJECT: &str = "chains-fixture";

fn asset_hash() -> String {
    sha256_hex("the final shipped bytes")
}

fn chain(sha: &str) -> Value {
    json!({
        "id": "chain-hero",
        "asset": "hero-sprite",
        "kind": "sprite",
        "record_id": "warden",
        "canon_refs": [{"id": "warden-authority", "sha256": sha256_hex("authority image")}],
        "prompt_pos": "a keeper portrait, lantern light",
        "prompt_neg": "blurry, extra fingers",
        "model": {
            "checkpoint": "wai-v14.safetensors",
            "sha256": sha256_hex("checkpoint bytes"),
            "sampler": "euler_a", "steps": 28, "cfg": 7.5, "size": "1024x1024"
        },
        "conditioning": [{"model": "openpose", "weight": 0.5}],
        "seed": 42,
        "finish_ops": [{"name": "palette-snap", "params": {"tolerance": 0}, "region": "all"}],
        "sha256": sha,
        "verdict": {"state": "keeper", "ruling_ref": "D-629 pack 12, item 3"}
    })
}

struct Fixture {
    root: PathBuf,
    store: Store,
}

impl Fixture {
    fn new(tag: &str) -> Self {
        let root = std::env::temp_dir().join(format!("ardor-chains-{tag}-{}", std::process::id()));
        assert!(!root.exists(), "unique fixture root");
        let mut store = Store::open(&root).unwrap();
        call(
            &mut store,
            "create_project",
            json!({"name": "Chains Fixture", "slug": PROJECT, "rating": "all_ages"}),
        );
        call(
            &mut store,
            "upsert_character",
            json!({"project": PROJECT, "character": {"id": "warden", "name": "Wren Halloway"}}),
        );
        call(
            &mut store,
            "register_asset",
            json!({"project": PROJECT, "asset": {
                "id": "hero-sprite", "kind": "sprite", "sha256": asset_hash(),
                "provenance": {"source": "fixture studio"},
                "origin_lane": "ntranch", "content_label": "nsfw", "route_to": "local"
            }}),
        );
        Self { root, store }
    }

    fn register(&mut self, chain: Value) -> Value {
        call(
            &mut self.store,
            "register_chain",
            json!({"project": PROJECT, "chain": chain}),
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

// ---------- validation ----------

#[test]
fn register_lands_in_manual_intake_form() {
    let mut fx = Fixture::new("manual");
    let out = fx.register(chain(&asset_hash()));
    let stored = &out["chain"];
    assert_eq!(stored["state"], "manual-intake");
    assert_eq!(
        stored["session_record"],
        json!({"state": "absent-unverified"}),
        "the session-record field carries the typed absent state, never a silent gap"
    );
    assert_eq!(stored["version"], 1);
    assert!(stored["held_record"].is_null());
    let view = call(
        &mut fx.store,
        "get_chain",
        json!({"project": PROJECT, "chain": "chain-hero"}),
    );
    assert_eq!(view["gate"], "keeper-chain 1.0");
    assert_eq!(view["chain"]["id"], "chain-hero");
    let list = call(&mut fx.store, "list_chains", json!({"project": PROJECT}));
    assert_eq!(list["chains"].as_array().unwrap().len(), 1);
    let project = call(&mut fx.store, "get_project", json!({"project": PROJECT}));
    assert_eq!(project["counts"]["chains"], 1);
}

#[test]
fn register_with_record_id_and_full_field_set_round_trips() {
    let mut fx = Fixture::new("roundtrip");
    let input = chain(&asset_hash());
    fx.register(input.clone());
    let view = call(
        &mut fx.store,
        "get_chain",
        json!({"project": PROJECT, "chain": "chain-hero"}),
    );
    let stored = &view["chain"];
    for field in [
        "canon_refs",
        "prompt_pos",
        "prompt_neg",
        "model",
        "conditioning",
        "seed",
        "finish_ops",
        "sha256",
        "verdict",
        "record_id",
    ] {
        assert_eq!(stored[field], input[field], "field {field} round-trips");
    }
}

#[test]
fn validation_run_records_a_valid_keeper() {
    let mut fx = Fixture::new("validrun");
    fx.register(chain(&asset_hash()));
    let out = call(
        &mut fx.store,
        "run_chain_validation",
        json!({"project": PROJECT, "chain": "chain-hero"}),
    );
    assert_eq!(out["validation"]["verdict"], "VALID");
    assert_eq!(out["validation"]["complete"], true);
    assert_eq!(out["validation"]["keeper"], true);
    let run = &out["run"];
    assert_eq!(run["pipeline"], "keeper-chain");
    assert_eq!(run["gate"], "keeper-chain 1.0");
    assert_eq!(run["kind"], "validation");
    assert_eq!(run["verdict"], "VALID");
    // The run record's artifact hash re-derives from its artifact.
    let artifact_hash = ardor::model::sha256_hex(&ardor::model::canonical(&run["artifact"]));
    assert_eq!(run["output_artifact_hashes"]["validation"], artifact_hash);
}

#[test]
fn candidate_chain_is_complete_but_not_a_keeper() {
    let mut fx = Fixture::new("candidate");
    let mut input = chain(&asset_hash());
    input["verdict"]["state"] = json!("candidate");
    fx.register(input);
    let out = call(
        &mut fx.store,
        "run_chain_validation",
        json!({"project": PROJECT, "chain": "chain-hero"}),
    );
    assert_eq!(out["validation"]["complete"], true);
    assert_eq!(out["validation"]["keeper"], false);
    assert_eq!(out["validation"]["verdict"], "VALID");
}

#[test]
fn upsert_bumps_version_and_writes_history() {
    let mut fx = Fixture::new("upsert");
    fx.register(chain(&asset_hash()));
    let mut second = chain(&asset_hash());
    second["prompt_pos"] = json!("a keeper portrait, dawn light");
    let out = fx.register(second);
    assert_eq!(out["chain"]["version"], 2);
    let view = call(
        &mut fx.store,
        "get_chain",
        json!({"project": PROJECT, "chain": "chain-hero"}),
    );
    assert_eq!(view["chain"]["prompt_pos"], "a keeper portrait, dawn light");
    // History lives in the chains document beside the chains.
    let doc: Value = serde_json::from_str(
        &fs::read_to_string(fx.root.join(PROJECT).join("chains.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(doc["chain_history"].as_array().unwrap().len(), 1);
    assert_eq!(doc["chain_history"][0]["version"], 2);
    assert_eq!(doc["chain_history"][0]["activation_reset"], false);
}

#[test]
fn custody_refusals_given_at_registration_are_stored_verbatim() {
    let mut fx = Fixture::new("custodyreg");
    let mut input = chain(&asset_hash());
    input["custody_refusals"] = json!(["TG-GENCOND-CUSTODY-FROZEN: hero/hand is held by owner"]);
    fx.register(input);
    let view = call(
        &mut fx.store,
        "get_chain",
        json!({"project": PROJECT, "chain": "chain-hero"}),
    );
    assert_eq!(
        view["chain"]["custody_refusals"],
        json!(["TG-GENCOND-CUSTODY-FROZEN: hero/hand is held by owner"])
    );
}

#[test]
fn record_id_may_be_null_for_a_pure_ornament() {
    let mut fx = Fixture::new("nullrecord");
    let mut input = chain(&asset_hash());
    input["record_id"] = Value::Null;
    let out = fx.register(input);
    assert!(out["chain"]["record_id"].is_null());
}

// ---------- adversarial ----------

#[test]
fn unknown_asset_is_refused_and_preserves_the_tree() {
    let mut fx = Fixture::new("unknownasset");
    let mut input = chain(&asset_hash());
    input["asset"] = json!("ghost-asset");
    fx.refused(
        "register_chain",
        json!({"project": PROJECT, "chain": input}),
        "unknown_asset",
    );
}

#[test]
fn kind_mismatch_with_the_asset_is_refused() {
    let mut fx = Fixture::new("kindmismatch");
    let mut input = chain(&asset_hash());
    input["kind"] = json!("audio");
    fx.refused(
        "register_chain",
        json!({"project": PROJECT, "chain": input}),
        "chain_kind_mismatch",
    );
}

#[test]
fn unknown_record_id_is_refused() {
    let mut fx = Fixture::new("unknownrecord");
    let mut input = chain(&asset_hash());
    input["record_id"] = json!("ghost");
    fx.refused(
        "register_chain",
        json!({"project": PROJECT, "chain": input}),
        "unknown_character",
    );
}

#[test]
fn unknown_chain_field_is_a_params_error() {
    let mut fx = Fixture::new("unknownfield");
    let mut input = chain(&asset_hash());
    input["surprise"] = json!(true);
    fx.refused(
        "register_chain",
        json!({"project": PROJECT, "chain": input}),
        "invalid_params",
    );
}

#[test]
fn verdict_without_a_ruling_reference_is_a_params_error() {
    let mut fx = Fixture::new("noruling");
    let mut input = chain(&asset_hash());
    input["verdict"] = json!({"state": "keeper"});
    fx.refused(
        "register_chain",
        json!({"project": PROJECT, "chain": input}),
        "invalid_params",
    );
}

#[test]
fn final_file_mismatch_records_a_refused_validation() {
    let mut fx = Fixture::new("mismatch");
    // Registration is structural: the mismatch is the validator's
    // finding, recorded as a REFUSED run — never hidden, never a
    // silent pass.
    fx.register(chain(&sha256_hex("different bytes entirely")));
    let out = call(
        &mut fx.store,
        "run_chain_validation",
        json!({"project": PROJECT, "chain": "chain-hero"}),
    );
    assert_eq!(out["validation"]["verdict"], "REFUSED");
    assert_eq!(out["validation"]["keeper"], false);
    assert_eq!(
        out["validation"]["refusal_codes"],
        json!(["chain_hash_mismatch"])
    );
    assert_eq!(out["run"]["verdict"], "REFUSED");
}
