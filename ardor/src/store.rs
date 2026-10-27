//! Authored project state on disk.
//!
//! One directory per project under the server home; one JSON
//! document per concern inside it (`project.json`, `cast.json`,
//! `canon.json`, `graph.json`, `systems.json`, `story.json`,
//! `levels.json`, `safety.json`, `assets.json`, `qa.json`,
//! `releases.json`). Documents load fully into memory; a canonical
//! write replaces exactly the documents an operation touched, via
//! write-temp-then-rename so a crash never leaves a torn file.
//! Nothing is written before the gate (see `gate.rs`) approves the
//! operation — a refused write leaves every file byte-identical.

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::{Value, json};

use crate::model::empty_object;

/// Document names (also the in-memory field keys), alphabetical.
pub const DOCS: [&str; 11] = [
    "assets", "canon", "cast", "graph", "levels", "project", "qa", "releases", "safety", "story",
    "systems",
];

/// A store failure (I/O or corrupt state). Carried as a stable
/// code + message, like the tool layer's domain failures.
#[derive(Debug)]
pub struct StoreError {
    pub code: &'static str,
    pub message: String,
}

impl std::fmt::Display for StoreError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for StoreError {}

fn io_error(context: &str, e: std::io::Error) -> StoreError {
    StoreError {
        code: "io",
        message: format!("{context}: {e}"),
    }
}

/// The whole state of one project, documents as JSON values.
#[derive(Debug, Clone)]
pub struct ProjectState {
    pub slug: String,
    pub docs: serde_json::Map<String, Value>,
}

impl ProjectState {
    pub fn doc(&self, name: &str) -> &Value {
        self.docs.get(name).expect("all documents loaded")
    }

    pub fn doc_mut(&mut self, name: &str) -> &mut Value {
        self.docs.get_mut(name).expect("all documents loaded")
    }

    /// The project metadata document (`project.json` itself).
    pub fn meta(&self) -> &Value {
        self.doc("project")
    }
}

/// The server home: a root directory holding project directories.
pub struct Store {
    root: PathBuf,
}

impl Store {
    pub fn open(root: &Path) -> Result<Store, StoreError> {
        fs::create_dir_all(root).map_err(|e| io_error("cannot create server home", e))?;
        Ok(Store {
            root: root.to_path_buf(),
        })
    }

    fn project_dir(&self, slug: &str) -> PathBuf {
        self.root.join(slug)
    }

    pub fn project_exists(&self, slug: &str) -> bool {
        self.project_dir(slug).join("project.json").is_file()
    }

    /// Slugs of all projects present, sorted.
    pub fn list_projects(&self) -> Vec<String> {
        let mut slugs = Vec::new();
        if let Ok(entries) = fs::read_dir(&self.root) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir()
                    && path.join("project.json").is_file()
                    && let Some(name) = path.file_name().and_then(|n| n.to_str())
                {
                    slugs.push(name.to_string());
                }
            }
        }
        slugs.sort();
        slugs
    }

    /// Load every document of a project. Missing documents (a
    /// project directory written by an older build) load as empty
    /// objects; a malformed one is a hard error, never a guess.
    pub fn load(&self, slug: &str) -> Result<ProjectState, StoreError> {
        let dir = self.project_dir(slug);
        if !dir.join("project.json").is_file() {
            return Err(StoreError {
                code: "project_not_found",
                message: format!("no project named {slug}"),
            });
        }
        let mut docs = serde_json::Map::new();
        for name in DOCS {
            let path = dir.join(format!("{name}.json"));
            let value = if path.is_file() {
                let text = fs::read_to_string(&path)
                    .map_err(|e| io_error(&format!("cannot read {}", path.display()), e))?;
                let parsed: Value = serde_json::from_str(&text).map_err(|e| StoreError {
                    code: "corrupt_state",
                    message: format!("{} is not valid JSON: {e}", path.display()),
                })?;
                if !parsed.is_object() {
                    return Err(StoreError {
                        code: "corrupt_state",
                        message: format!("{} must hold a JSON object", path.display()),
                    });
                }
                parsed
            } else {
                empty_object()
            };
            docs.insert(name.to_string(), value);
        }
        Ok(ProjectState {
            slug: slug.to_string(),
            docs,
        })
    }

    /// Persist the named documents of a state, each atomically.
    /// Callers pass exactly the documents the operation changed.
    pub fn save(&self, state: &ProjectState, changed: &[&str]) -> Result<(), StoreError> {
        let dir = self.project_dir(&state.slug);
        fs::create_dir_all(&dir).map_err(|e| io_error("cannot create project dir", e))?;
        for name in changed {
            let value = state.doc(name);
            let text = serde_json::to_string_pretty(value).expect("values serialize");
            let tmp = dir.join(format!(".{name}.json.tmp"));
            let target = dir.join(format!("{name}.json"));
            fs::write(&tmp, format!("{text}\n"))
                .map_err(|e| io_error(&format!("cannot write {}", tmp.display()), e))?;
            fs::rename(&tmp, &target)
                .map_err(|e| io_error(&format!("cannot replace {}", target.display()), e))?;
        }
        Ok(())
    }

    /// The studio-level channel-account document (`accounts.json`
    /// in the server home): credential REFERENCES and status for
    /// group 10 — never values (the gate enforces that). Loads as
    /// an empty registry when absent.
    pub fn load_accounts(&self) -> Result<Value, StoreError> {
        let path = self.root.join("accounts.json");
        if !path.is_file() {
            return Ok(json!({"accounts": []}));
        }
        let text = fs::read_to_string(&path)
            .map_err(|e| io_error(&format!("cannot read {}", path.display()), e))?;
        let parsed: Value = serde_json::from_str(&text).map_err(|e| StoreError {
            code: "corrupt_state",
            message: format!("{} is not valid JSON: {e}", path.display()),
        })?;
        if !parsed.is_object() {
            return Err(StoreError {
                code: "corrupt_state",
                message: format!("{} must hold a JSON object", path.display()),
            });
        }
        Ok(parsed)
    }

    /// Persist the channel-account document atomically.
    pub fn save_accounts(&self, accounts: &Value) -> Result<(), StoreError> {
        let text = serde_json::to_string_pretty(accounts).expect("values serialize");
        let tmp = self.root.join(".accounts.json.tmp");
        let target = self.root.join("accounts.json");
        fs::write(&tmp, format!("{text}\n"))
            .map_err(|e| io_error(&format!("cannot write {}", tmp.display()), e))?;
        fs::rename(&tmp, &target)
            .map_err(|e| io_error(&format!("cannot replace {}", target.display()), e))?;
        Ok(())
    }

    /// Fresh document set for a new project (everything empty
    /// except what the caller fills in before the first save).
    pub fn blank_state(slug: &str) -> ProjectState {
        let mut docs = serde_json::Map::new();
        for name in DOCS {
            docs.insert(name.to_string(), json!({}));
        }
        ProjectState {
            slug: slug.to_string(),
            docs,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_root(tag: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("ardor-store-test-{tag}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    #[test]
    fn save_load_round_trip() {
        let root = temp_root("roundtrip");
        let store = Store::open(&root).unwrap();
        let mut state = Store::blank_state("demo");
        state
            .doc_mut("project")
            .as_object_mut()
            .unwrap()
            .insert("name".to_string(), json!("Demo"));
        store.save(&state, &["project"]).unwrap();
        let loaded = store.load("demo").unwrap();
        assert_eq!(loaded.meta()["name"], "Demo");
        assert_eq!(store.list_projects(), vec!["demo".to_string()]);
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn missing_project_is_a_typed_error() {
        let root = temp_root("missing");
        let store = Store::open(&root).unwrap();
        let err = store.load("nope").unwrap_err();
        assert_eq!(err.code, "project_not_found");
        let _ = fs::remove_dir_all(&root);
    }
}
