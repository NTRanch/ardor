//! The facade the UI programs against (DESIGN §D2).
//!
//! A narrow trait returning `serde_json::Value`: `status()`,
//! `tools()`, and `call(tool, args)`. Two implementations: the
//! real MCP stdio client in [`crate::client`], and the in-memory
//! scripted [`FakeFacade`] every test uses. The fake records
//! every call, which is what makes the plan-mode proof possible:
//! staging a workflow must leave the record empty.

use std::collections::{BTreeMap, VecDeque};
use std::fmt;

use serde_json::Value;

/// One facade failure. Typed, never a bare string at call sites.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FacadeError {
    /// The server process could not be spawned.
    Spawn(String),
    /// Transport I/O failed (includes a dead server).
    Io(String),
    /// No response inside the deadline.
    Timeout,
    /// The peer sent bytes that are not a valid response.
    Protocol(String),
    /// The server returned a domain (`isError`) tool result.
    Tool { code: String, message: String },
    /// The facade has been marked dead (fake) or the child exited.
    Dead,
}

impl fmt::Display for FacadeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FacadeError::Spawn(m) => write!(f, "spawn failed: {m}"),
            FacadeError::Io(m) => write!(f, "transport: {m}"),
            FacadeError::Timeout => write!(f, "call timed out"),
            FacadeError::Protocol(m) => write!(f, "protocol: {m}"),
            FacadeError::Tool { code, message } => write!(f, "tool {code}: {message}"),
            FacadeError::Dead => write!(f, "server unreachable (dead)"),
        }
    }
}

impl std::error::Error for FacadeError {}

/// The narrow server surface the console needs.
pub trait ArdorFacade {
    /// Identity of the live session; gate evidence is only
    /// valid for the session that produced it.
    fn session_id(&self) -> &str;
    /// Server identity/version from `server/discover`.
    fn status(&mut self) -> Result<Value, FacadeError>;
    /// The server's own tool list (`tools/list` result).
    fn tools(&mut self) -> Result<Value, FacadeError>;
    /// Call one of the server's tools; success payload parsed
    /// from the tool's JSON text content.
    fn call(&mut self, tool: &str, args: &Value) -> Result<Value, FacadeError>;
}

/// In-memory scripted facade for tests and `--fake` demos.
///
/// Responses are queued per method/tool and consumed in order;
/// an unscripted call is a protocol error, so a test that drives
/// an unexpected call fails loudly instead of passing on a
/// default. Every call is recorded in order.
pub struct FakeFacade {
    session: String,
    dead: bool,
    status_script: VecDeque<Result<Value, FacadeError>>,
    tools_script: VecDeque<Result<Value, FacadeError>>,
    call_scripts: BTreeMap<String, VecDeque<Result<Value, FacadeError>>>,
    calls: Vec<(String, Value)>,
    status_calls: usize,
    tools_calls: usize,
}

impl FakeFacade {
    /// An empty fake on a fixed session id.
    pub fn new() -> Self {
        FakeFacade {
            session: "fake-session-1".to_string(),
            dead: false,
            status_script: VecDeque::new(),
            tools_script: VecDeque::new(),
            call_scripts: BTreeMap::new(),
            calls: Vec::new(),
            status_calls: 0,
            tools_calls: 0,
        }
    }

    /// Queue one `status()` response.
    pub fn script_status(&mut self, result: Result<Value, FacadeError>) {
        self.status_script.push_back(result);
    }

    /// Queue one `tools()` response.
    pub fn script_tools(&mut self, result: Result<Value, FacadeError>) {
        self.tools_script.push_back(result);
    }

    /// Queue one response for `call(tool, ..)`.
    pub fn script_call(&mut self, tool: &str, result: Result<Value, FacadeError>) {
        self.call_scripts
            .entry(tool.to_string())
            .or_default()
            .push_back(result);
    }

    /// Every recorded tool call, in order.
    pub fn calls(&self) -> &[(String, Value)] {
        &self.calls
    }

    /// How many times `status()` was invoked.
    pub fn status_calls(&self) -> usize {
        self.status_calls
    }

    /// Simulate the server dying: every later call fails.
    pub fn set_dead(&mut self, dead: bool) {
        self.dead = dead;
    }

    /// Simulate a respawn under a new session identity.
    pub fn set_session(&mut self, session: &str) {
        self.session = session.to_string();
    }

    fn next(
        queue: &mut VecDeque<Result<Value, FacadeError>>,
        what: &str,
    ) -> Result<Value, FacadeError> {
        queue
            .pop_front()
            .unwrap_or_else(|| Err(FacadeError::Protocol(format!("unscripted {what}"))))
    }
}

impl Default for FakeFacade {
    fn default() -> Self {
        Self::new()
    }
}

impl ArdorFacade for FakeFacade {
    fn session_id(&self) -> &str {
        &self.session
    }

    fn status(&mut self) -> Result<Value, FacadeError> {
        self.status_calls += 1;
        if self.dead {
            return Err(FacadeError::Dead);
        }
        Self::next(&mut self.status_script, "status")
    }

    fn tools(&mut self) -> Result<Value, FacadeError> {
        self.tools_calls += 1;
        if self.dead {
            return Err(FacadeError::Dead);
        }
        Self::next(&mut self.tools_script, "tools")
    }

    fn call(&mut self, tool: &str, args: &Value) -> Result<Value, FacadeError> {
        self.calls.push((tool.to_string(), args.clone()));
        if self.dead {
            return Err(FacadeError::Dead);
        }
        match self.call_scripts.get_mut(tool) {
            Some(queue) => Self::next(queue, tool),
            None => Err(FacadeError::Protocol(format!("unscripted call {tool}"))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn fake_records_calls_in_order() {
        let mut fake = FakeFacade::new();
        fake.script_call("list_projects", Ok(json!({"projects": []})));
        fake.script_call("audit_safety", Ok(json!({"audit": {"ok": true}})));
        let _ = fake.call("list_projects", &json!({})).expect("scripted");
        let _ = fake
            .call("audit_safety", &json!({"project": "x"}))
            .expect("scripted");
        assert_eq!(fake.calls().len(), 2);
        assert_eq!(fake.calls()[0].0, "list_projects");
        assert_eq!(fake.calls()[1].0, "audit_safety");
    }

    #[test]
    fn fake_unscripted_call_is_loud() {
        let mut fake = FakeFacade::new();
        let err = fake.call("nope", &json!({})).expect_err("unscripted");
        assert!(matches!(err, FacadeError::Protocol(_)));
    }

    #[test]
    fn fake_dead_fails_everything() {
        let mut fake = FakeFacade::new();
        fake.script_status(Ok(json!({})));
        fake.set_dead(true);
        assert_eq!(fake.status(), Err(FacadeError::Dead));
        assert_eq!(
            fake.call("list_projects", &json!({})),
            Err(FacadeError::Dead)
        );
    }
}
