//! Fixture builder for the committed `canon-census 1.0` evidence
//! (`fixtures/canon-census-1.0/`). Builds a small synthetic canon —
//! two characters, two canon rules, one required canon table, one
//! foreign cast pair, a ruled genesis denylist — through the public
//! tool surface, exactly as an operator would, then runs the census.
//!
//! The fixture is synthetic gate evidence, not a real title's canon.
//! Its text is all-ages; the lane stamp below is the work's origin.
//!
//! origin_lane: ntranch
//! content_label: nsfw
//!
//! Run: `cargo run --example canon_census_fixture -- <home> <out>`
//! Writes `<out>/census-artifact.json` and `<out>/run-record.json`.

use std::path::Path;

use ardor::model::canonical;
use ardor::{store::Store, tools};
use serde_json::{Value, json};

pub const PROJECT: &str = "census-fixture";

fn call(store: &mut Store, name: &str, args: Value) -> Value {
    tools::dispatch(store, name, &args)
        .unwrap_or_else(|e| panic!("{name}: {} {}", e.code, e.message))
}

/// Create the fixture project (genesis state seeded by group 1).
pub fn create_project(store: &mut Store) {
    call(
        store,
        "create_project",
        json!({"name": "Census Fixture", "slug": PROJECT, "rating": "all_ages",
               "channel_targets": ["itch_io"]}),
    );
}

/// The fixture's required character table: one signature row per
/// character, wren's trait cell frozen.
pub fn signature_table() -> Value {
    json!({
        "id": "signature-traits",
        "title": "Signature traits",
        "subject": "character",
        "required": true,
        "declared_rows": 2,
        "columns": [
            {"name": "trait", "type": "string"},
            {"name": "motif", "type": "string"}
        ],
        "rows": [
            {"key": "wren",
             "cells": {"trait": "storm-sense", "motif": "gull feather"},
             "ruling_ref": "FIXTURE-CANON-1", "frozen": ["trait"]},
            {"key": "sorrel",
             "cells": {"trait": "root-memory", "motif": "orchard key"},
             "ruling_ref": "FIXTURE-CANON-1"}
        ]
    })
}

/// The fixture's foreign cast pair. Mina shares exactly half of
/// wren's descriptor tokens (500 per-mille, at the ruled threshold);
/// Petra shares none.
pub fn foreign_pair() -> Value {
    json!({
        "id": "foreign-cast-a",
        "foreign_cast": "Fixture Foreign Cast",
        "source_ref": "fixture:foreign-cast-v1",
        "descriptor_fields": ["hair", "style"],
        "members": [
            {"name": "Mina Voss", "descriptors": {"hair": "black"}},
            {"name": "Petra Ilves", "descriptors": {"hair": "blonde", "style": "ponytail"}}
        ],
        "threshold": {"permille": 500, "ruling_ref": "FIXTURE-DIFF-1"}
    })
}

/// Author the whole fixture canon except the census run.
pub fn author_fixture(store: &mut Store) {
    create_project(store);
    call(
        store,
        "record_denylist",
        json!({"project": PROJECT,
               "terms": ["Studio Prime", "Jane Realperson", "Project Realname"],
               "ruling_ref": "FIXTURE-DENYLIST-1"}),
    );
    call(
        store,
        "upsert_character",
        json!({"project": PROJECT, "character": {
            "id": "wren", "name": "Wren Halloway", "hair": "black", "style": "bob",
            "bio": "A lighthouse keeper who maps drowned roads."}}),
    );
    call(
        store,
        "upsert_character",
        json!({"project": PROJECT, "character": {
            "id": "sorrel", "name": "Sorrel Adeyemi", "hair": "red", "style": "braids",
            "bio": "A cartographer of orchards and low roads."}}),
    );
    call(
        store,
        "add_canon_rule",
        json!({"project": PROJECT, "rule": {
            "id": "lock-wren-hair", "kind": "lock", "character": "wren",
            "fields": ["hair"]}}),
    );
    call(
        store,
        "add_canon_rule",
        json!({"project": PROJECT, "rule": {
            "id": "fact-sorrel-hair", "kind": "fact", "character": "sorrel",
            "field": "hair", "value": "red"}}),
    );
    call(
        store,
        "declare_canon_table",
        json!({"project": PROJECT, "table": signature_table()}),
    );
    call(
        store,
        "declare_cast_pair",
        json!({"project": PROJECT, "pair": foreign_pair()}),
    );
}

/// Run the census and return the `run_canon_census` tool output.
pub fn run_census(store: &mut Store) -> Value {
    call(store, "run_canon_census", json!({"project": PROJECT}))
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    assert!(
        args.len() == 3,
        "usage: canon_census_fixture <home-dir> <out-dir>"
    );
    let home = Path::new(&args[1]);
    let out = Path::new(&args[2]);
    assert!(!home.exists(), "home must not exist: {}", home.display());
    let mut store = Store::open(home).expect("open store");
    author_fixture(&mut store);
    let result = run_census(&mut store);
    std::fs::create_dir_all(out).expect("create out dir");
    let run = &result["run"];
    std::fs::write(
        out.join("census-artifact.json"),
        format!("{}\n", canonical(&run["artifact"])),
    )
    .expect("write census artifact");
    std::fs::write(out.join("run-record.json"), format!("{}\n", canonical(run)))
        .expect("write run record");
    println!(
        "verdict={} input_state_hash={}",
        run["verdict"], run["input_state_hash"]
    );
}
