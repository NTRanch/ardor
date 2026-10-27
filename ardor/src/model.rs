//! Shared data-model helpers: identifier validation, canonical
//! JSON (for hashing), SHA-256 digests, and the deterministic
//! SplitMix64 stream used by playtest generation. No wall-clock
//! time and no ambient randomness anywhere in this crate: every
//! record the server stores is reproducible from its inputs, which
//! is what makes replay/hash verification meaningful.

use serde_json::{Map, Value};
use sha2::{Digest, Sha256};

/// Maximum length of any identifier (project slug, node id, ...).
pub const ID_MAX_LEN: usize = 64;
/// Maximum steps in one generated playtest (bounded work).
pub const PLAYTEST_MAX_STEPS: usize = 10_000;
/// Default playtest step budget when the caller does not set one.
pub const PLAYTEST_DEFAULT_STEPS: usize = 512;
/// How often one node may appear in a single playtest before the
/// walk is declared truncated (loop guard).
pub const PLAYTEST_NODE_VISIT_CAP: usize = 8;

/// Identifiers are lowercase slugs: `[a-z0-9]` then
/// `[a-z0-9_-]*`, at most [`ID_MAX_LEN`] bytes. Hand-checked, no
/// regex dependency.
pub fn is_valid_id(s: &str) -> bool {
    if s.is_empty() || s.len() > ID_MAX_LEN {
        return false;
    }
    let mut chars = s.chars();
    let first = chars.next().expect("non-empty checked");
    if !(first.is_ascii_lowercase() || first.is_ascii_digit()) {
        return false;
    }
    chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-')
}

/// Canonical JSON: object keys sorted recursively, arrays in
/// order, no insignificant whitespace. Hashed content in this
/// crate is strings and integers only, so serialization is exact.
pub fn canonical(value: &Value) -> String {
    let mut out = String::new();
    write_canonical(value, &mut out);
    out
}

fn write_canonical(value: &Value, out: &mut String) {
    match value {
        Value::Null => out.push_str("null"),
        Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Value::Number(n) => out.push_str(&n.to_string()),
        Value::String(s) => {
            out.push_str(&serde_json::to_string(s).expect("string serialization cannot fail"));
        }
        Value::Array(items) => {
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_canonical(item, out);
            }
            out.push(']');
        }
        Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            out.push('{');
            for (i, key) in keys.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                out.push_str(&serde_json::to_string(key).expect("key serialization cannot fail"));
                out.push(':');
                write_canonical(&map[*key], out);
            }
            out.push('}');
        }
    }
}

/// SHA-256 hex digest of a string.
pub fn sha256_hex(text: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(text.as_bytes());
    let digest = hasher.finalize();
    let mut out = String::with_capacity(64);
    for byte in digest {
        out.push_str(&format!("{byte:02x}"));
    }
    out
}

/// True for a 64-character lowercase hex string (a SHA-256 digest).
pub fn is_sha256_hex(s: &str) -> bool {
    s.len() == 64
        && s.chars()
            .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c))
}

/// SplitMix64: a small deterministic stream. Playtest generation
/// draws branch choices from this, seeded by the caller, so a run
/// is reproducible byte-for-byte.
pub struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    pub fn new(seed: u64) -> Self {
        SplitMix64 { state: seed }
    }

    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Draw in `0..n` (n must be positive).
    pub fn below(&mut self, n: usize) -> usize {
        assert!(n > 0, "below() needs a positive bound");
        (self.next_u64() % (n as u64)) as usize
    }
}

/// Read a named array out of a document object (empty if absent).
pub fn arr<'a>(doc: &'a Value, key: &str) -> &'a Vec<Value> {
    static EMPTY: Vec<Value> = Vec::new();
    doc.get(key).and_then(Value::as_array).unwrap_or(&EMPTY)
}

/// Mutable variant of [`arr`]; creates the array if absent.
pub fn arr_mut<'a>(doc: &'a mut Value, key: &str) -> &'a mut Vec<Value> {
    let object = doc.as_object_mut().expect("document is an object");
    object
        .entry(key.to_string())
        .or_insert_with(|| Value::Array(Vec::new()))
        .as_array_mut()
        .expect("entry is an array")
}

/// Find an element of a document array by its `id` field.
pub fn find_by_id<'a>(doc: &'a Value, key: &str, id: &str) -> Option<&'a Value> {
    arr(doc, key)
        .iter()
        .find(|item| item.get("id").and_then(Value::as_str) == Some(id))
}

/// Mutable variant of [`find_by_id`].
pub fn find_by_id_mut<'a>(doc: &'a mut Value, key: &str, id: &str) -> Option<&'a mut Value> {
    arr_mut(doc, key)
        .iter_mut()
        .find(|item| item.get("id").and_then(Value::as_str) == Some(id))
}

/// An empty JSON object value.
pub fn empty_object() -> Value {
    Value::Object(Map::new())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn ids_are_slugs() {
        assert!(is_valid_id("strobe"));
        assert!(is_valid_id("scene-01_a"));
        assert!(!is_valid_id(""));
        assert!(!is_valid_id("Has Caps"));
        assert!(!is_valid_id("-leading-dash"));
        assert!(!is_valid_id("with space"));
        assert!(!is_valid_id(&"a".repeat(ID_MAX_LEN + 1)));
    }

    #[test]
    fn canonical_sorts_keys_recursively() {
        let v = json!({"b": 1, "a": {"d": [3, {"f": 1, "e": 2}], "c": null}});
        assert_eq!(
            canonical(&v),
            "{\"a\":{\"c\":null,\"d\":[3,{\"e\":2,\"f\":1}]},\"b\":1}"
        );
    }

    #[test]
    fn splitmix_is_deterministic() {
        let mut a = SplitMix64::new(42);
        let mut b = SplitMix64::new(42);
        for _ in 0..100 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
        let mut c = SplitMix64::new(42);
        for _ in 0..100 {
            assert!(c.below(7) < 7);
        }
    }

    #[test]
    fn sha256_shape() {
        let digest = sha256_hex("ardor");
        assert!(is_sha256_hex(&digest));
        assert!(!is_sha256_hex("xyz"));
        assert!(!is_sha256_hex(&digest.to_uppercase()));
    }
}
