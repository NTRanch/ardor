//! Stage 2: seven validation and seven adversarial tests for
//! `canon-census 1.0` (ARDOR-PIPELINES.md §2 and §7).

#[allow(dead_code)]
#[path = "../examples/canon_census_fixture.rs"]
mod fixture_gen;

use ardor::{
    canon,
    model::{canonical, sha256_hex},
    store::Store,
    tools,
};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

struct Fixture {
    root: PathBuf,
    store: Store,
}

impl Fixture {
    fn new(tag: &str) -> Self {
        let root = std::env::temp_dir().join(format!("ardor-canon-{tag}-{}", std::process::id()));
        assert!(
            !root.exists(),
            "unique fixture must not overwrite another run"
        );
        let store = Store::open(&root).unwrap();
        Self { root, store }
    }
    fn project_path(&self, doc: &str) -> PathBuf {
        self.root.join(fixture_gen::PROJECT).join(doc)
    }
    fn census(&mut self) -> Value {
        call(
            &mut self.store,
            "get_canon_census",
            json!({"project": fixture_gen::PROJECT}),
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

fn project_args(extra: Value) -> Value {
    let mut args = json!({"project": fixture_gen::PROJECT});
    for (key, value) in extra.as_object().unwrap() {
        args[key] = value.clone();
    }
    args
}

// ---------- validation ----------

#[test]
fn validation_denylist_replaces_whole_and_ledger_records_it() {
    let mut f = Fixture::new("denylist");
    fixture_gen::create_project(&mut f.store);
    call(
        &mut f.store,
        "record_denylist",
        project_args(json!({"terms": ["Studio Prime"], "ruling_ref": "FIXTURE-A"})),
    );
    call(
        &mut f.store,
        "record_denylist",
        project_args(
            json!({"terms": ["Studio Prime", "Jane Realperson"], "ruling_ref": "FIXTURE-B"}),
        ),
    );
    let state = call(
        &mut f.store,
        "get_genesis_state",
        json!({"project": fixture_gen::PROJECT}),
    );
    assert_eq!(
        state["sheet"]["denylist"],
        json!(["Studio Prime", "Jane Realperson"])
    );
    let entries: Vec<&Value> = state["sheet"]["ledger"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|e| e["kind"] == "denylist")
        .collect();
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0]["ruling_ref"], "FIXTURE-A");
    assert_eq!(entries[1]["ruling_ref"], "FIXTURE-B");
    assert_eq!(
        entries[1]["payload"]["terms"],
        json!(["Studio Prime", "Jane Realperson"])
    );
    assert_eq!(f.census()["census"]["counts"]["denylist_terms"], 2);
}

#[test]
fn validation_census_counts_differentiation_and_unverified_status() {
    let mut f = Fixture::new("census");
    fixture_gen::author_fixture(&mut f.store);
    let out = f.census();
    assert_eq!(out["gate"], "canon-census 1.0");
    let census = &out["census"];
    assert_eq!(census["verdict"], "VERIFIED");
    assert_eq!(census["counts"]["characters"], 2);
    assert_eq!(census["counts"]["canon_rules"], 2);
    assert_eq!(census["counts"]["cast_pairs"], 1);
    assert_eq!(census["counts"]["denylist_terms"], 3);
    assert_eq!(census["counts"]["orphans"], 0);
    assert_eq!(census["counts"]["sanitization_hits"], 0);
    assert_eq!(census["counts"]["tables"][0]["declared"], 2);
    assert_eq!(census["counts"]["tables"][0]["found"], 2);
    let diff = &census["differentiation"][0];
    assert_eq!(diff["verdict"], "within_threshold");
    assert_eq!(diff["max_permille"], 500);
    assert_eq!(diff["threshold_permille"], 500);
    assert_eq!(diff["overlaps"].as_array().unwrap().len(), 1);
    assert_eq!(diff["overlaps"][0]["home"], "wren");
    assert_eq!(diff["overlaps"][0]["foreign"], "Mina Voss");
    // No run recorded yet: a pure census verifies nothing by itself.
    assert_eq!(out["status"]["verified"], false);
    assert_eq!(out["status"]["code"], "canon_unverified");
    assert_eq!(out["runs_recorded"], 0);
    assert!(out["latest_run"].is_null());
}

#[test]
fn validation_run_record_verifies_and_touches_only_census_doc() {
    let mut f = Fixture::new("run");
    fixture_gen::author_fixture(&mut f.store);
    let safety_before = fs::read(f.project_path("safety.json")).unwrap();
    let before = tree(&f.root);
    let out = fixture_gen::run_census(&mut f.store);
    assert_eq!(out["status"]["verified"], true);
    assert_eq!(out["runs_recorded"], 1);
    let run = &out["run"];
    assert_eq!(run["pipeline"], "canon");
    assert_eq!(run["pipeline_version"], "1.0");
    assert_eq!(run["gate"], "canon-census 1.0");
    assert_eq!(run["seq"], 1);
    assert_eq!(run["verdict"], "VERIFIED");
    assert_eq!(run["input_state_hash"], run["artifact"]["input_state_hash"]);
    assert_eq!(
        run["output_artifact_hashes"]["census"],
        sha256_hex(&canonical(&run["artifact"]))
    );
    let refs: Vec<&str> = run["ruling_refs"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(Value::as_str)
        .collect();
    for expected in ["FIXTURE-CANON-1", "FIXTURE-DIFF-1", "FIXTURE-DENYLIST-1"] {
        assert!(refs.contains(&expected), "missing ruling ref {expected}");
    }
    // The run writes exactly one document; safety bytes are untouched.
    let after = tree(&f.root);
    assert_eq!(
        before.keys().collect::<Vec<_>>(),
        after.keys().collect::<Vec<_>>()
    );
    let changed: Vec<&PathBuf> = before.keys().filter(|p| before[*p] != after[*p]).collect();
    assert_eq!(changed, vec![&f.project_path("census.json")]);
    assert_eq!(
        fs::read(f.project_path("safety.json")).unwrap(),
        safety_before
    );
    // The persisted document is the store's pretty form, byte-stable:
    // re-serializing the parsed document reproduces the file exactly.
    let file = fs::read_to_string(f.project_path("census.json")).unwrap();
    assert_eq!(
        file,
        format!(
            "{}\n",
            serde_json::to_string_pretty(&serde_json::from_str::<Value>(&file).unwrap()).unwrap()
        )
    );
}

#[test]
fn validation_frozen_cell_amendment_records_history() {
    let mut f = Fixture::new("frozen");
    fixture_gen::author_fixture(&mut f.store);
    let table = fixture_gen::signature_table();
    // Identical redeclaration: version bumps, no amendment recorded.
    let out = call(
        &mut f.store,
        "declare_canon_table",
        project_args(json!({"table": table})),
    );
    assert_eq!(out["table"]["version"], 2);
    assert_eq!(out["history"].as_array().unwrap().len(), 1);
    assert_eq!(out["history"][0]["amended"], false);
    assert_eq!(out["history"][0]["frozen_changes"], json!([]));
    // Deliberate change of the frozen cell, with a reason: recorded.
    let mut changed = fixture_gen::signature_table();
    changed["rows"][0]["cells"]["trait"] = json!("storm-calling");
    let out = call(
        &mut f.store,
        "declare_canon_table",
        project_args(json!({"table": changed, "amendment": {"reason": "Owner retcon"}})),
    );
    assert_eq!(out["table"]["version"], 3);
    assert_eq!(out["table"]["rows"][0]["cells"]["trait"], "storm-calling");
    let history = out["history"].as_array().unwrap();
    assert_eq!(history.len(), 2);
    assert_eq!(history[1]["amended"], true);
    assert_eq!(history[1]["reason"], "Owner retcon");
    assert_eq!(history[1]["frozen_changes"], json!(["wren.trait"]));
}

#[test]
fn validation_two_identical_histories_have_identical_census_bytes() {
    let mut a = Fixture::new("bake-a");
    let mut b = Fixture::new("bake-b");
    fixture_gen::author_fixture(&mut a.store);
    fixture_gen::author_fixture(&mut b.store);
    fixture_gen::run_census(&mut a.store);
    fixture_gen::run_census(&mut b.store);
    for doc in ["census.json", "canon.json", "genesis.json"] {
        assert_eq!(
            fs::read(a.project_path(doc)).unwrap(),
            fs::read(b.project_path(doc)).unwrap(),
            "{doc} must be byte-identical for identical histories"
        );
    }
}

#[test]
fn validation_overlap_formula_is_the_stated_jaccard_permille() {
    let set = |items: &[&str]| items.iter().map(|s| s.to_string()).collect::<BTreeSet<_>>();
    assert_eq!(canon::overlap_permille(&set(&[]), &set(&[])), 0);
    assert_eq!(canon::overlap_permille(&set(&["a"]), &set(&["a"])), 1000);
    assert_eq!(canon::overlap_permille(&set(&["a"]), &set(&["b"])), 0);
    assert_eq!(
        canon::overlap_permille(&set(&["hair=black", "style=bob"]), &set(&["hair=black"])),
        500
    );
    assert_eq!(
        canon::overlap_permille(&set(&["a", "b", "c"]), &set(&["a"])),
        333
    );
}

#[test]
fn validation_fixture_artifact_matches_committed_evidence() {
    let mut f = Fixture::new("evidence");
    fixture_gen::author_fixture(&mut f.store);
    let out = fixture_gen::run_census(&mut f.store);
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures/canon-census-1.0");
    let artifact = fs::read_to_string(dir.join("census-artifact.json")).unwrap();
    assert_eq!(
        artifact,
        format!("{}\n", canonical(&out["run"]["artifact"])),
        "committed census artifact must re-derive from the fixture"
    );
    let record = fs::read_to_string(dir.join("run-record.json")).unwrap();
    assert_eq!(record, format!("{}\n", canonical(&out["run"])));
}

// ---------- adversarial ----------

#[test]
fn adversarial_denylist_terms_are_validated_atomically() {
    let mut f = Fixture::new("bad-denylist");
    fixture_gen::create_project(&mut f.store);
    for (terms, code) in [
        (json!(["Studio Prime", "studio  prime"]), "invalid_denylist"),
        (json!(["Studio Prime", "Studio Prime"]), "invalid_denylist"),
        (json!([""]), "invalid_denylist"),
        (json!(["Studio Prime", 7]), "invalid_params"),
    ] {
        f.refused(
            "record_denylist",
            project_args(json!({"terms": terms, "ruling_ref": "FIXTURE-X"})),
            code,
        );
    }
    let many: Vec<String> = (0..=canon::MAX_ROWS).map(|i| format!("term-{i}")).collect();
    f.refused(
        "record_denylist",
        project_args(json!({"terms": many, "ruling_ref": "FIXTURE-X"})),
        "invalid_denylist",
    );
    f.refused(
        "record_denylist",
        project_args(json!({"terms": ["Studio Prime"], "ruling_ref": " "})),
        "ruling_ref_required",
    );
}

#[test]
fn adversarial_table_and_pair_declarations_are_validated() {
    let mut f = Fixture::new("bad-declare");
    fixture_gen::create_project(&mut f.store);
    let mut cases: Vec<Value> = Vec::new();
    let mut bad = fixture_gen::signature_table();
    bad["subject"] = json!("bogus");
    cases.push(bad);
    let mut bad = fixture_gen::signature_table();
    bad["columns"][1]["name"] = json!("trait");
    cases.push(bad);
    let mut bad = fixture_gen::signature_table();
    bad["rows"][0]["cells"]["trait"] = json!(7);
    cases.push(bad);
    let mut bad = fixture_gen::signature_table();
    bad["rows"][0]["frozen"] = json!(["nope"]);
    cases.push(bad);
    let mut bad = fixture_gen::signature_table();
    bad["declared_rows"] = json!(canon::MAX_ROWS + 1);
    cases.push(bad);
    let mut bad = fixture_gen::signature_table();
    bad["rows"][0].as_object_mut().unwrap().remove("ruling_ref");
    cases.push(bad);
    for table in cases {
        f.refused(
            "declare_canon_table",
            project_args(json!({"table": table})),
            "invalid_params",
        );
    }
    let mut pairs: Vec<Value> = Vec::new();
    let mut bad = fixture_gen::foreign_pair();
    bad["members"][1]["name"] = json!("mina  voss");
    pairs.push(bad);
    let mut bad = fixture_gen::foreign_pair();
    bad["threshold"]["permille"] = json!(1001);
    pairs.push(bad);
    let mut bad = fixture_gen::foreign_pair();
    bad["descriptor_fields"] = json!(["name"]);
    pairs.push(bad);
    let mut bad = fixture_gen::foreign_pair();
    bad["members"][0]["descriptors"]["undeclared"] = json!("x");
    pairs.push(bad);
    for pair in pairs {
        f.refused(
            "declare_cast_pair",
            project_args(json!({"pair": pair})),
            "invalid_params",
        );
    }
}

#[test]
fn adversarial_frozen_cell_change_or_unfreeze_is_refused() {
    let mut f = Fixture::new("frozen-refused");
    fixture_gen::author_fixture(&mut f.store);
    let mut changed = fixture_gen::signature_table();
    changed["rows"][0]["cells"]["trait"] = json!("storm-calling");
    f.refused(
        "declare_canon_table",
        project_args(json!({"table": changed})),
        "canon_frozen_cell",
    );
    let mut unfrozen = fixture_gen::signature_table();
    unfrozen["rows"][0]["frozen"] = json!([]);
    f.refused(
        "declare_canon_table",
        project_args(json!({"table": unfrozen})),
        "canon_frozen_cell",
    );
    // An amendment without a reason is no amendment.
    let mut changed = fixture_gen::signature_table();
    changed["rows"][0]["cells"]["trait"] = json!("storm-calling");
    f.refused(
        "declare_canon_table",
        project_args(json!({"table": changed, "amendment": {"reason": " "}})),
        "canon_frozen_cell",
    );
}

#[test]
fn adversarial_orphans_and_count_mismatch_refuse_verification() {
    let mut f = Fixture::new("orphans");
    fixture_gen::create_project(&mut f.store);
    call(
        &mut f.store,
        "record_denylist",
        project_args(json!({"terms": ["Studio Prime"], "ruling_ref": "FIXTURE-X"})),
    );
    for (id, name) in [("wren", "Wren Halloway"), ("sorrel", "Sorrel Adeyemi")] {
        call(
            &mut f.store,
            "upsert_character",
            project_args(json!({"character": {"id": id, "name": name}})),
        );
    }
    call(
        &mut f.store,
        "declare_canon_table",
        project_args(json!({"table": {
            "id": "kinship", "subject": "character", "required": false, "declared_rows": 1,
            "columns": [{"name": "kin", "type": "character"}],
            "rows": [{"key": "wren", "cells": {"kin": "ghost-person"},
                      "ruling_ref": "FIXTURE-X"}]
        }})),
    );
    call(
        &mut f.store,
        "declare_canon_table",
        project_args(json!({"table": {
            "id": "roster", "subject": "character", "required": true, "declared_rows": 2,
            "columns": [{"name": "trait", "type": "string"}],
            "rows": [{"key": "wren", "cells": {"trait": "storm-sense"},
                      "ruling_ref": "FIXTURE-X"}]
        }})),
    );
    let census = f.census()["census"].clone();
    assert_eq!(census["verdict"], "REFUSED");
    let orphan_kinds: Vec<&str> = census["orphans"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|o| o["kind"].as_str())
        .collect();
    assert!(
        orphan_kinds.contains(&"cell_ref_missing"),
        "{orphan_kinds:?}"
    );
    assert!(
        orphan_kinds.contains(&"record_missing_from_table"),
        "{orphan_kinds:?}"
    );
    assert!(
        census["findings"]["count_mismatch"]
            .as_array()
            .unwrap()
            .iter()
            .any(|m| m["table"] == "roster")
    );
    fixture_gen::run_census(&mut f.store);
    let out = f.census();
    assert_eq!(out["status"]["verified"], false);
    assert_eq!(out["status"]["code"], "canon_orphan");
}

#[test]
fn adversarial_sanitization_hit_and_name_collision_block_verification() {
    // Sanitization: a denylist term in a shippable field, folded.
    let mut f = Fixture::new("sanitize");
    fixture_gen::create_project(&mut f.store);
    call(
        &mut f.store,
        "record_denylist",
        project_args(json!({"terms": ["Studio Prime"], "ruling_ref": "FIXTURE-X"})),
    );
    call(
        &mut f.store,
        "upsert_character",
        project_args(json!({"character": {
            "id": "wren", "name": "Wren Halloway",
            "bio": "Former lookout for STUDIO   prime nights."}})),
    );
    let census = f.census()["census"].clone();
    assert_eq!(census["verdict"], "REFUSED");
    assert_eq!(census["counts"]["sanitization_hits"], 1);
    assert_eq!(census["sanitization"]["hits"][0]["term"], "Studio Prime");
    assert_eq!(census["sanitization"]["hits"][0]["doc"], "cast");
    fixture_gen::run_census(&mut f.store);
    assert_eq!(f.census()["status"]["code"], "sanitization_hit");

    // Differentiation: a folded name equality is a collision outright.
    let mut f = Fixture::new("collision");
    fixture_gen::author_fixture(&mut f.store);
    let mut pair = fixture_gen::foreign_pair();
    pair["id"] = json!("foreign-cast-b");
    pair["members"][0]["name"] = json!("  Wren   Halloway ");
    call(
        &mut f.store,
        "declare_cast_pair",
        project_args(json!({"pair": pair})),
    );
    let census = f.census()["census"].clone();
    let diff = census["differentiation"]
        .as_array()
        .unwrap()
        .iter()
        .find(|d| d["pair"] == "foreign-cast-b")
        .unwrap();
    assert_eq!(diff["verdict"], "collision");
    assert_eq!(diff["name_collisions"][0]["home"], "wren");
    fixture_gen::run_census(&mut f.store);
    assert_eq!(f.census()["status"]["code"], "differentiation_collision");
}

#[test]
fn adversarial_canon_write_stales_the_census_and_tamper_is_review() {
    let mut f = Fixture::new("stale");
    fixture_gen::author_fixture(&mut f.store);
    fixture_gen::run_census(&mut f.store);
    assert_eq!(f.census()["status"]["verified"], true);
    // Any canon-state write after the run makes the census stale.
    call(
        &mut f.store,
        "upsert_character",
        project_args(json!({"character": {
            "id": "wren", "name": "Wren Halloway", "hair": "black", "style": "bob",
            "bio": "A lighthouse keeper who maps drowned roads.", "mood": "calm"}})),
    );
    let out = f.census();
    assert_eq!(out["status"]["verified"], false);
    assert_eq!(out["status"]["code"], "census_stale");
    // A fresh run verifies again.
    fixture_gen::run_census(&mut f.store);
    assert_eq!(f.census()["status"]["verified"], true);
    assert_eq!(f.census()["runs_recorded"], 2);
    // A forged verdict in the recorded run is REVIEW, never verified.
    let path = f.project_path("census.json");
    let mut doc: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    doc["runs"][1]["artifact"]["verdict"] = json!("FORGED");
    fs::write(&path, canonical(&doc)).unwrap();
    let out = f.census();
    assert_eq!(out["status"]["verified"], false);
    assert_eq!(out["status"]["code"], "census_run_mismatch");
}

#[test]
fn adversarial_unruled_denylist_leaves_canon_unverified() {
    let mut f = Fixture::new("unruled");
    fixture_gen::create_project(&mut f.store);
    call(
        &mut f.store,
        "upsert_character",
        project_args(json!({"character": {"id": "wren", "name": "Wren Halloway"}})),
    );
    let census = f.census()["census"].clone();
    assert_eq!(census["verdict"], "REFUSED");
    assert!(census["counts"]["denylist_terms"].is_null());
    assert!(
        census["refusal_codes"]
            .as_array()
            .unwrap()
            .contains(&json!("denylist_unruled"))
    );
    assert_eq!(census["sanitization"]["swept_ok"], false);
    fixture_gen::run_census(&mut f.store);
    let out = f.census();
    assert_eq!(out["status"]["verified"], false);
    assert_eq!(out["status"]["code"], "canon_unverified");
}
