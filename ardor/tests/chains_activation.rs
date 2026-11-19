//! Stage 4b: seven validation and eight adversarial tests for
//! Ardor-side chain activation (Amendment A1, ARDOR-PIPELINES.md
//! §4/§5): the four validator legs — (a) the session record
//! validates whole against the published `torogao-scene-record/1`
//! schema, (b) the recipe's conditioning hash equals the hash
//! Ardor derives from the conditioning record it holds, (c) the
//! chain's final-file hash equals the registered asset's, (d) the
//! custody map is well-formed — plus the §7 conformance run
//! record. Every refused activation is proven to leave the tree
//! byte-identical and the chain in manual-intake form.

use ardor::{
    model::sha256_hex,
    scene_record::{self},
    store::Store,
    tools,
};
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

const PROJECT: &str = "chains-fixture";

fn hex(c: char) -> String {
    c.to_string().repeat(64)
}

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

fn conditioning() -> Value {
    json!({"schema": 1, "profile": "hero-openpose", "ratios": {"hero": 0.5}})
}

/// A session record that validates whole; `conditioning_hash` is
/// whatever the recipe claims (honest or transcribed-wrong).
fn record(conditioning_hash: &str) -> Value {
    json!({
        "schema": 1,
        "id": "hero-scene",
        "recipe": {
            "conditioning_sha256": conditioning_hash,
            "graph_sha256": hex('b'),
            "seed": 42,
            "backend": "seam-c",
            "execution_context": "",
            "rerender": {"best_effort": {"reason": "execution-context jitter unpinned"}}
        },
        "extensions": [
            {"kind": "puppet2d", "state": {"puppet2d": {
                "format": "lumen-rig-model", "format_version": 1,
                "model_sha256": hex('c')}}},
            {"kind": "armature3d", "state": "not_yet_populated"},
            {"kind": "sprite_cels", "state": "not_yet_populated"},
            {"kind": "audio_events", "state": "not_yet_populated"}
        ],
        "custody": {"entries": []},
        "provenance": [],
        "frozen_tracks": [],
        "palette_snaps": []
    })
}

fn honest_hash() -> String {
    scene_record::conditioning_sha256(&conditioning())
}

struct Fixture {
    root: PathBuf,
    store: Store,
}

impl Fixture {
    fn new(tag: &str) -> Self {
        let root =
            std::env::temp_dir().join(format!("ardor-activate-{tag}-{}", std::process::id()));
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
        call(
            &mut store,
            "register_chain",
            json!({"project": PROJECT, "chain": chain(&asset_hash())}),
        );
        Self { root, store }
    }

    fn intake(&mut self, record: Value) -> Value {
        call(
            &mut self.store,
            "intake_session_record",
            json!({"project": PROJECT, "chain": "chain-hero",
                   "record": record, "conditioning": conditioning()}),
        )
    }

    fn view(&mut self) -> Value {
        call(
            &mut self.store,
            "get_chain",
            json!({"project": PROJECT, "chain": "chain-hero"}),
        )
    }

    fn activate(&mut self) -> Value {
        call(
            &mut self.store,
            "activate_chain",
            json!({"project": PROJECT, "chain": "chain-hero"}),
        )
    }

    /// A refused activation: the typed code, and afterwards the
    /// whole tree — and the chain's activation state — unchanged.
    fn activate_refused(&mut self, code: &str, message_part: &str) {
        let before = tree(&self.root);
        let error = tools::dispatch(
            &mut self.store,
            "activate_chain",
            &json!({"project": PROJECT, "chain": "chain-hero"}),
        )
        .expect_err("refusal");
        assert_eq!(error.code, code, "{}", error.message);
        assert!(error.message.contains(message_part), "{}", error.message);
        assert_eq!(before, tree(&self.root), "refusal must preserve every file");
        let view = self.view();
        assert_eq!(view["chain"]["state"], "manual-intake");
        assert_ne!(view["chain"]["session_record"]["state"], "verified");
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
fn intake_moves_the_field_to_present_unverified() {
    let mut fx = Fixture::new("intake");
    let rec = record(&honest_hash());
    let out = fx.intake(rec.clone());
    let expected = scene_record::record_sha256(&rec);
    assert_eq!(
        out["chain"]["session_record"],
        json!({"state": "present-unverified",
               "format": "torogao-scene-record/1",
               "record_sha256": expected})
    );
    assert_eq!(out["chain"]["state"], "manual-intake");
    assert_eq!(out["chain"]["held_record"], rec);
}

#[test]
fn activation_happy_path_verifies_and_records_the_run() {
    let mut fx = Fixture::new("happy");
    let rec = record(&honest_hash());
    let hash = scene_record::record_sha256(&rec);
    fx.intake(rec);
    let out = fx.activate();
    assert_eq!(out["activated"], true);
    assert_eq!(
        out["chain"]["session_record"],
        json!({"state": "verified",
               "format": "torogao-scene-record/1",
               "record_sha256": hash}),
        "the verified shape is exactly Amendment A1's triple"
    );
    assert_eq!(out["chain"]["state"], "active");
    let run = &out["run"];
    assert_eq!(run["kind"], "activation");
    assert_eq!(run["verdict"], "ACTIVATED");
    assert_eq!(run["output_artifact_hashes"]["session_record"], hash);
    assert_eq!(run["ruling_refs"], json!(["D-629 pack 12, item 3"]));
    assert_eq!(run["artifact"]["conditioning_sha256"], honest_hash());
    let view = fx.view();
    assert_eq!(view["runs"].as_array().unwrap().len(), 1);
}

#[test]
fn activation_records_custody_refusals_verbatim() {
    let mut fx = Fixture::new("custodyactivate");
    fx.intake(record(&honest_hash()));
    let out = call(
        &mut fx.store,
        "activate_chain",
        json!({"project": PROJECT, "chain": "chain-hero",
               "custody_refusals": ["TG-CUSTODY-77: node hero/hand frozen by wren, reason: held"]}),
    );
    assert_eq!(out["activated"], true);
    assert_eq!(
        out["chain"]["custody_refusals"],
        json!(["TG-CUSTODY-77: node hero/hand frozen by wren, reason: held"])
    );
}

#[test]
fn custody_refusal_verb_appends_verbatim_before_activation() {
    let mut fx = Fixture::new("custodyverb");
    let out = call(
        &mut fx.store,
        "record_custody_refusal",
        json!({"project": PROJECT, "chain": "chain-hero",
               "refusal": "custody refusal TG-9: node palette frozen"}),
    );
    assert_eq!(
        out["chain"]["custody_refusals"],
        json!(["custody refusal TG-9: node palette frozen"])
    );
    assert_eq!(out["chain"]["state"], "manual-intake");
}

#[test]
fn pinned_execution_context_satisfies_recipe_honesty() {
    let mut fx = Fixture::new("pinned");
    let mut rec = record(&honest_hash());
    rec["recipe"]["rerender"] = json!("pinned_context_byte_identical");
    rec["recipe"]["execution_context"] = json!("host primo, driver 580.1, seed lock on");
    fx.intake(rec);
    let out = fx.activate();
    assert_eq!(out["activated"], true);
}

#[test]
fn reintake_after_activation_returns_the_chain_to_manual_form() {
    let mut fx = Fixture::new("reintake");
    fx.intake(record(&honest_hash()));
    fx.activate();
    let mut second = record(&honest_hash());
    second["id"] = json!("hero-scene-2");
    let out = fx.intake(second.clone());
    assert_eq!(out["chain"]["state"], "manual-intake");
    assert_eq!(
        out["chain"]["session_record"]["state"],
        "present-unverified"
    );
    assert_eq!(
        out["chain"]["session_record"]["record_sha256"],
        scene_record::record_sha256(&second)
    );
}

#[test]
fn changing_the_final_hash_on_an_active_chain_resets_activation() {
    let mut fx = Fixture::new("reset");
    fx.intake(record(&honest_hash()));
    fx.activate();
    // The asset is re-registered with new bytes, and the chain
    // re-registered pointing at them: the activation was earned
    // against the old bytes and does not carry over.
    let new_hash = sha256_hex("revised final bytes");
    call(
        &mut fx.store,
        "register_asset",
        json!({"project": PROJECT, "asset": {
            "id": "hero-sprite", "kind": "sprite", "sha256": new_hash,
            "provenance": {"source": "fixture studio"},
            "origin_lane": "ntranch", "content_label": "nsfw", "route_to": "local"
        }}),
    );
    let out = call(
        &mut fx.store,
        "register_chain",
        json!({"project": PROJECT, "chain": chain(&new_hash)}),
    );
    assert_eq!(out["chain"]["state"], "manual-intake");
    assert_eq!(
        out["chain"]["session_record"]["state"],
        "present-unverified"
    );
    let doc: Value = serde_json::from_str(
        &fs::read_to_string(fx.root.join(PROJECT).join("chains.json")).unwrap(),
    )
    .unwrap();
    assert_eq!(doc["chain_history"][0]["activation_reset"], true);
}

// ---------- adversarial ----------

#[test]
fn transcribed_but_wrong_conditioning_hash_is_refused() {
    let mut fx = Fixture::new("wronghash");
    // The recipe presents a hash that was copied from somewhere —
    // well-formed hex, but not the hash of the conditioning
    // record Ardor holds. Derived != presented: refuse.
    fx.intake(record(&hex('f')));
    fx.activate_refused("chain_hash_mismatch", "derived");
}

#[test]
fn byte_identity_claim_without_a_pinned_context_is_refused() {
    let mut fx = Fixture::new("unpinned");
    let mut rec = record(&honest_hash());
    rec["recipe"]["rerender"] = json!("pinned_context_byte_identical");
    rec["recipe"]["execution_context"] = json!("");
    fx.intake(rec);
    fx.activate_refused("record_schema_invalid", "leg (a)");
}

#[test]
fn final_file_hash_mismatch_is_refused_at_activation() {
    let mut fx = Fixture::new("finalmismatch");
    // Re-register the chain pointing at bytes the asset does not
    // have, then intake an otherwise honest record.
    call(
        &mut fx.store,
        "register_chain",
        json!({"project": PROJECT, "chain": chain(&sha256_hex("other bytes"))}),
    );
    fx.intake(record(&honest_hash()));
    fx.activate_refused("chain_hash_mismatch", "leg (c)");
}

#[test]
fn malformed_custody_map_is_refused() {
    let mut fx = Fixture::new("badcustody");
    let mut rec = record(&honest_hash());
    rec["custody"] = json!({"entries": [{"node": "Bad Node", "state": "open"}]});
    fx.intake(rec);
    fx.activate_refused("record_schema_invalid", "custody");
}

#[test]
fn duplicated_medium_block_is_refused() {
    let mut fx = Fixture::new("dupblock");
    let mut rec = record(&honest_hash());
    rec["extensions"][3] = json!({"kind": "puppet2d", "state": "not_yet_populated"});
    fx.intake(rec);
    fx.activate_refused("record_schema_invalid", "appears 2 times");
}

#[test]
fn activation_without_intake_is_refused() {
    let mut fx = Fixture::new("nointake");
    fx.activate_refused("chain_incomplete", "no session record");
}

#[test]
fn puppet2d_reference_on_another_medium_is_refused() {
    let mut fx = Fixture::new("wrongmedium");
    let mut rec = record(&honest_hash());
    rec["extensions"][1]["state"] = json!({"puppet2d": {
        "format": "lumen-rig-model", "format_version": 1,
        "model_sha256": hex('c')}});
    fx.intake(rec);
    fx.activate_refused("record_schema_invalid", "leg (a)");
}

#[test]
fn double_activation_is_refused() {
    let mut fx = Fixture::new("double");
    fx.intake(record(&honest_hash()));
    fx.activate();
    let before = tree(&fx.root);
    let error = tools::dispatch(
        &mut fx.store,
        "activate_chain",
        &json!({"project": PROJECT, "chain": "chain-hero"}),
    )
    .expect_err("refusal");
    assert_eq!(error.code, "chain_already_active");
    assert_eq!(before, tree(&fx.root));
}
