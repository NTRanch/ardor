//! Group 10 (D-127): channel credentials and sessions.
//!
//! The hard rule, enforced here at the gate: Ardor manages
//! credential REFERENCES and STATUS, never values. A channel
//! account record names the channel, the account identity, the
//! channel's real credential kind, and where the value lives (a
//! primo `pass` path or a Secure Vault entry reference) — plus
//! scopes, expiry, and verification status. Nothing else is
//! accepted: registration and verification payloads pass a
//! secret-material scan, and anything shaped like a raw credential
//! (or carried under a secret-named field) is refused before it
//! can touch state. Tool outputs are built from these records, so
//! they inherit the same property — and the tests assert it.
//!
//! Verification model: Ardor never holds the value, so it cannot
//! run a probe itself. Channels with a real probe (itch.io key
//! validation, Patreon token validation) name it in the registry;
//! the holder of the value (estate store side) runs it and the
//! result is recorded here as an attestation with its timestamp.
//! Portal-only channels get honest login-reference records and
//! login attestations — no invented token flows: registering a
//! credential kind that differs from the channel's real kind is
//! refused.

use serde_json::{Value, json};

use crate::channels::{auth_for, find_channel};
use crate::gate::{Refusal, safety_audit};
use crate::model::arr;
use crate::store::ProjectState;

fn str_field<'a>(value: &'a Value, key: &str) -> Option<&'a str> {
    value.get(key).and_then(Value::as_str)
}

// ---------- secret-material scan ----------

/// Field names that would carry a value rather than a reference.
/// A payload using any of these is refused outright.
const SECRET_FIELD_NAMES: [&str; 10] = [
    "access_token",
    "api_key",
    "apikey",
    "credential_value",
    "passwd",
    "password",
    "private_key",
    "secret",
    "token",
    "value",
];

/// Does this string look like a raw credential value rather than
/// a reference? Conservative on purpose: references in this
/// system are slash paths (`pass`) or short entry ids, neither of
/// which is a long mixed-case alphanumeric run.
pub fn looks_like_secret(s: &str) -> bool {
    let trimmed = s.trim();
    for prefix in [
        "-----BEGIN",
        "AKIA",
        "ghp_",
        "gho_",
        "glpat-",
        "sk-",
        "xoxb-",
        "xoxp-",
    ] {
        if trimmed.starts_with(prefix) {
            return true;
        }
    }
    // JWT shape: three long base64url segments.
    let segments: Vec<&str> = trimmed.split('.').collect();
    if segments.len() == 3
        && trimmed.len() >= 40
        && segments.iter().all(|seg| {
            seg.len() >= 8
                && seg
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        })
    {
        return true;
    }
    // Long mixed-class run with no path separators: the shape of
    // API keys and tokens, not of references. Checked per
    // whitespace-separated token as well as for the whole string,
    // so a value pasted into a prose field is still caught.
    let mixed_run = |s: &str| {
        s.len() >= 32
            && !s.contains('/')
            && s.chars().any(|c| c.is_ascii_lowercase())
            && s.chars().any(|c| c.is_ascii_uppercase())
            && s.chars().any(|c| c.is_ascii_digit())
    };
    if mixed_run(trimmed) {
        return true;
    }
    trimmed.split_whitespace().any(mixed_run)
}

/// Recursively scan a payload for secret material: secret-named
/// fields or secret-shaped strings. Returns the offending field
/// path (never the value itself).
pub fn scan_for_secret_material(value: &Value) -> Option<String> {
    fn walk(value: &Value, path: &str) -> Option<String> {
        match value {
            Value::Object(map) => {
                for (key, inner) in map {
                    if SECRET_FIELD_NAMES.contains(&key.to_lowercase().as_str()) {
                        return Some(format!("{path}.{key} (secret-named field)"));
                    }
                    if let Some(hit) = walk(inner, &format!("{path}.{key}")) {
                        return Some(hit);
                    }
                }
                None
            }
            Value::Array(items) => {
                for (i, item) in items.iter().enumerate() {
                    if let Some(hit) = walk(item, &format!("{path}[{i}]")) {
                        return Some(hit);
                    }
                }
                None
            }
            Value::String(s) => {
                if looks_like_secret(s) {
                    Some(format!("{path} (secret-shaped value)"))
                } else {
                    None
                }
            }
            _ => None,
        }
    }
    walk(value, "$")
}

fn refuse_secret(hit: &str) -> Refusal {
    Refusal::new(
        "secret_value_refused",
        format!(
            "payload carries what looks like a raw credential at {hit}. \
             Ardor stores references and status only — put the value in pass or the \
             Secure Vault through the estate's secure flow and register the reference."
        ),
    )
}

// ---------- timestamps ----------

/// Light ISO-8601 shape check (YYYY-MM-DDTHH:MM:SS, optional
/// suffix). Enough to keep records honest and comparable.
pub fn is_iso_timestamp(s: &str) -> bool {
    let bytes = s.as_bytes();
    if bytes.len() < 19 {
        return false;
    }
    let digit = |i: usize| bytes[i].is_ascii_digit();
    digit(0)
        && digit(1)
        && digit(2)
        && digit(3)
        && bytes[4] == b'-'
        && digit(5)
        && digit(6)
        && bytes[7] == b'-'
        && digit(8)
        && digit(9)
        && bytes[10] == b'T'
        && digit(11)
        && digit(12)
        && bytes[13] == b':'
        && digit(14)
        && digit(15)
        && bytes[16] == b':'
        && digit(17)
        && digit(18)
}

/// Days since the civil epoch for the date part of an ISO
/// timestamp (Howard Hinnant's algorithm), for age arithmetic.
fn iso_days(s: &str) -> Option<i64> {
    if !is_iso_timestamp(s) {
        return None;
    }
    let year: i64 = s[0..4].parse().ok()?;
    let month: i64 = s[5..7].parse().ok()?;
    let day: i64 = s[8..10].parse().ok()?;
    let y = if month <= 2 { year - 1 } else { year };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = (month + 9) % 12;
    let doy = (153 * mp + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    Some(era * 146097 + doe - 719468)
}

// ---------- account operations ----------

/// One proposed account write (same decide/apply discipline as
/// project writes).
#[derive(Debug)]
pub enum AccountOp {
    RecordVerification {
        account_id: String,
        result: String,
        method: String,
        verified_at: String,
        detail: Option<String>,
    },
    RegisterAccount {
        record: Value,
    },
}

/// Find an account record by id.
pub fn find_account<'a>(accounts: &'a Value, account_id: &str) -> Option<&'a Value> {
    arr(accounts, "accounts")
        .iter()
        .find(|a| str_field(a, "id") == Some(account_id))
}

/// The verified account for a channel, if one exists.
pub fn verified_account_for<'a>(accounts: &'a Value, channel: &str) -> Option<&'a Value> {
    arr(accounts, "accounts").iter().find(|a| {
        str_field(a, "channel") == Some(channel)
            && a.get("verification").and_then(|v| str_field(v, "status")) == Some("verified")
    })
}

/// Judge an account operation. Pure, like the project gate.
pub fn decide_account(accounts: &Value, op: &AccountOp) -> Result<(), Refusal> {
    match op {
        AccountOp::RegisterAccount { record } => {
            if let Some(hit) = scan_for_secret_material(record) {
                return Err(refuse_secret(&hit));
            }
            let channel_id = str_field(record, "channel").unwrap_or("");
            let Some(channel) = find_channel(channel_id) else {
                return Err(Refusal::new(
                    "unknown_channel",
                    format!("'{channel_id}' is not in the channel-target registry"),
                ));
            };
            if channel.verification_status != "verified" {
                return Err(Refusal::new(
                    "channel_flagged",
                    format!(
                        "channel '{channel_id}' is flagged ({}): {}",
                        channel.verification_status, channel.verification_note
                    ),
                ));
            }
            let real_kind = auth_for(channel_id).map(|a| a.kind).unwrap_or("");
            let kind = str_field(record, "credential_kind").unwrap_or("");
            if kind != real_kind {
                return Err(Refusal::new(
                    "auth_kind_mismatch",
                    format!(
                        "channel '{channel_id}' really uses '{real_kind}' credentials; \
                         registering kind '{kind}' would invent an auth flow this channel does not have"
                    ),
                ));
            }
            // The reference must be a reference: one of the two
            // estate stores, with a path/entry that is shaped like
            // one (no whitespace, bounded length).
            let store_ref = record.get("store_ref").cloned().unwrap_or(Value::Null);
            let store = str_field(&store_ref, "store").unwrap_or("");
            if store != "pass" && store != "secure_vault" {
                return Err(Refusal::new(
                    "unknown_credential_store",
                    "store_ref.store must be \"pass\" (primo) or \"secure_vault\" — the estate's two existing stores",
                ));
            }
            let reference = if store == "pass" {
                str_field(&store_ref, "path").unwrap_or("")
            } else {
                str_field(&store_ref, "entry").unwrap_or("")
            };
            if reference.is_empty()
                || reference.len() > 200
                || reference.chars().any(char::is_whitespace)
                || reference.starts_with('/')
                || reference.contains("..")
            {
                return Err(Refusal::new(
                    "malformed_reference",
                    "store_ref must name a pass path or Secure Vault entry (relative, no whitespace), never a value",
                ));
            }
            // One account record per (channel, account identity).
            let identity = str_field(record, "account").unwrap_or("");
            let duplicate = arr(accounts, "accounts").iter().any(|a| {
                str_field(a, "channel") == Some(channel_id)
                    && str_field(a, "account") == Some(identity)
            });
            if duplicate {
                return Err(Refusal::new(
                    "account_exists",
                    format!("an account record for '{identity}' on '{channel_id}' already exists"),
                ));
            }
            if let Some(expiry) = record.get("expires_at").and_then(Value::as_str)
                && !is_iso_timestamp(expiry)
            {
                return Err(Refusal::new(
                    "malformed_timestamp",
                    "expires_at must be an ISO-8601 timestamp (YYYY-MM-DDTHH:MM:SSZ)",
                ));
            }
            Ok(())
        }
        AccountOp::RecordVerification {
            account_id,
            result,
            method,
            verified_at,
            detail,
        } => {
            if find_account(accounts, account_id).is_none() {
                return Err(Refusal::new(
                    "unknown_account",
                    format!("no channel account with id '{account_id}'"),
                ));
            }
            if result != "verified" && result != "failed" {
                return Err(Refusal::new(
                    "malformed_verification",
                    "verification result must be \"verified\" or \"failed\"",
                ));
            }
            if !is_iso_timestamp(verified_at) {
                return Err(Refusal::new(
                    "malformed_timestamp",
                    "verified_at must be an ISO-8601 timestamp (YYYY-MM-DDTHH:MM:SSZ)",
                ));
            }
            let mut payload = json!({"method": method});
            if let Some(d) = detail {
                payload["detail"] = json!(d);
            }
            if let Some(hit) = scan_for_secret_material(&payload) {
                return Err(refuse_secret(&hit));
            }
            let _ = method;
            Ok(())
        }
    }
}

/// Execute an approved account operation; returns the stored
/// record (references and status only, by construction).
pub fn apply_account(accounts: &mut Value, op: AccountOp) -> Value {
    match op {
        AccountOp::RegisterAccount { mut record } => {
            let seq = arr(accounts, "accounts").len() + 1;
            let channel = str_field(&record, "channel").unwrap_or("").to_string();
            let identity = str_field(&record, "account").unwrap_or("").to_string();
            record
                .as_object_mut()
                .expect("record is an object")
                .insert("id".to_string(), json!(format!("{channel}:{identity}")));
            record.as_object_mut().expect("record is an object").insert(
                "verification".to_string(),
                json!({"status": "unverified", "last_verified_at": null, "method": null, "detail": null}),
            );
            record
                .as_object_mut()
                .expect("record is an object")
                .insert("seq".to_string(), json!(seq));
            let stored = record.clone();
            crate::model::arr_mut(accounts, "accounts").push(record);
            stored
        }
        AccountOp::RecordVerification {
            account_id,
            result,
            method,
            verified_at,
            detail,
        } => {
            let accounts_arr = crate::model::arr_mut(accounts, "accounts");
            let slot = accounts_arr
                .iter_mut()
                .find(|a| str_field(a, "id") == Some(account_id.as_str()))
                .expect("gate checked the account exists");
            slot.as_object_mut().expect("account is an object").insert(
                "verification".to_string(),
                json!({
                    "status": result,
                    "last_verified_at": verified_at,
                    "method": method,
                    "detail": detail
                }),
            );
            slot.clone()
        }
    }
}

/// The public status of one account record: everything except
/// value material — which the record never held. `now` (optional
/// ISO timestamp) enables expiry and age computation.
pub fn account_status(account: &Value, now: Option<&str>) -> Value {
    let verification = account.get("verification").cloned().unwrap_or(Value::Null);
    let status = verification
        .get("status")
        .and_then(Value::as_str)
        .unwrap_or("unverified");
    let expires_at = account.get("expires_at").and_then(Value::as_str);
    let last_verified = verification.get("last_verified_at").and_then(Value::as_str);
    let (expired, verification_age_days) = match now {
        Some(now) if is_iso_timestamp(now) => {
            let expired = expires_at.is_some_and(|e| e < now);
            let age = match (iso_days(now), last_verified.and_then(iso_days)) {
                (Some(n), Some(v)) => Some(n - v),
                _ => None,
            };
            (expired, age)
        }
        _ => (false, None),
    };
    json!({
        "id": account.get("id"),
        "channel": account.get("channel"),
        "account": account.get("account"),
        "credential_kind": account.get("credential_kind"),
        "store_ref": account.get("store_ref"),
        "scopes": account.get("scopes"),
        "expires_at": account.get("expires_at"),
        "status": if expired { "expired" } else { status },
        "verification": verification,
        "expired": expired,
        "verification_age_days": verification_age_days,
        "rotation_due": expired
    })
}

// ---------- publish preflight ----------

/// The publish preflight: everything a group-9 handoff needs,
/// checked and reported as data — channel verified, safety audit
/// green, and the channel's auth registered AND verified (and not
/// expired, when `now` is supplied). The handoff gate enforces
/// the same checks; this tool is how an author sees them first.
pub fn publish_preflight(
    state: &ProjectState,
    accounts: &Value,
    channel_id: &str,
    now: Option<&str>,
) -> Value {
    let mut checks: Vec<Value> = Vec::new();
    let channel = find_channel(channel_id);
    checks.push(json!({
        "check": "channel_known",
        "ok": channel.is_some(),
        "detail": channel.map_or("not in the registry".to_string(), |c| c.name.to_string())
    }));
    if let Some(c) = channel {
        checks.push(json!({
            "check": "channel_verified",
            "ok": c.verification_status == "verified",
            "detail": c.verification_note
        }));
        // Section discipline (D-128 annex), mirroring the
        // handoff gate: services channels take commissions, not
        // releases; SFW-only channels never take adult releases.
        let rating = state
            .meta()
            .get("rating")
            .and_then(Value::as_str)
            .unwrap_or("adult");
        let (ok, detail) = if c.section == "services" || c.section == "services_sfw" {
            (
                false,
                "services channel: tracked for commissions/contracts, not release handoffs"
                    .to_string(),
            )
        } else if c.section == "assets_sfw" && rating == "adult" {
            (
                false,
                "SFW-only channel, and this project is adult-rated".to_string(),
            )
        } else {
            (true, "a release target for this project".to_string())
        };
        checks.push(json!({"check": "release_target", "ok": ok, "detail": detail}));
    }
    let audit = safety_audit(state);
    checks.push(json!({
        "check": "safety_audit",
        "ok": audit["ok"].as_bool() == Some(true),
        "detail": if audit["ok"].as_bool() == Some(true) { "all safety checks pass".to_string() } else { "see audit_safety for the failing checks".to_string() }
    }));
    let account = arr(accounts, "accounts")
        .iter()
        .find(|a| str_field(a, "channel") == Some(channel_id));
    checks.push(json!({
        "check": "auth_registered",
        "ok": account.is_some(),
        "detail": account.map_or("no channel account recorded — register the credential reference first".to_string(), |a| format!("account {}", a["id"].as_str().unwrap_or("?")))
    }));
    if let Some(a) = account {
        let status = account_status(a, now);
        let verified = status["verification"]["status"].as_str() == Some("verified");
        checks.push(json!({
            "check": "auth_verified",
            "ok": verified,
            "detail": if verified { "a verification attestation is on record".to_string() } else { "no successful verification recorded — run the channel probe via the value holder and record it".to_string() }
        }));
        checks.push(json!({
            "check": "auth_not_expired",
            "ok": !status["expired"].as_bool().unwrap_or(false),
            "detail": if now.is_none() { "not evaluated (no 'now' supplied)".to_string() } else if status["expired"].as_bool() == Some(true) { "credential is past its recorded expiry — rotate and re-verify".to_string() } else { "within its recorded validity".to_string() }
        }));
    }
    let ok = checks.iter().all(|c| c["ok"].as_bool() == Some(true));
    json!({
        "ok": ok,
        "channel": channel_id,
        "executor": channel.map(|c| c.handoff_executor),
        "checks": checks
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secret_shapes_are_caught() {
        assert!(looks_like_secret(
            "AbCdEfGhIjKlMnOpQrStUvWxYz0123456789abcd"
        ));
        assert!(looks_like_secret(
            "ghp_AbCdEfGhIjKlMnOpQrStUvWxYz0123456789"
        ));
        assert!(looks_like_secret(
            "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.dozjgNryP4J3jVmNHl0w5N_XgL0n3AAA"
        ));
        // References are not secrets.
        assert!(!looks_like_secret("ntranch/itch/butler-key"));
        assert!(!looks_like_secret("vault-entry-itch-main"));
        assert!(!looks_like_secret("ntranch"));
    }

    #[test]
    fn scan_finds_named_fields_and_values() {
        let bad = json!({"account": "x", "api_key": "whatever"});
        assert!(scan_for_secret_material(&bad).is_some());
        let bad2 = json!({"detail": "checked AbCdEfGhIjKlMnOpQrStUvWxYz0123456789abcd"});
        assert!(scan_for_secret_material(&bad2).is_some());
        let good = json!({"account": "ntranch", "store_ref": {"store": "pass", "path": "ntranch/itch/key"}});
        assert!(scan_for_secret_material(&good).is_none());
    }

    #[test]
    fn iso_shape_and_day_math() {
        assert!(is_iso_timestamp("2026-10-27T14:00:00Z"));
        assert!(!is_iso_timestamp("2026-10-27"));
        assert!(!is_iso_timestamp("soon"));
        let a = iso_days("2026-10-27T00:00:00Z").unwrap();
        let b = iso_days("2026-10-20T00:00:00Z").unwrap();
        assert_eq!(a - b, 7);
    }
}
