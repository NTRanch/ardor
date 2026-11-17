//! The real facade: an MCP stdio client spawning ardor's own
//! server binary.
//!
//! Protocol (verified against ardor's `tests/mcp_smoke.rs`):
//! sessions open with `server/discover`; there is no
//! `initialize` handshake; every request after discovery
//! carries `_meta` stamped with the modern protocol version
//! (`2026-07-28`). Tool success payloads are JSON text content;
//! domain failures arrive as `isError` content carrying
//! `{error: {code, message}}`.
//!
//! Ownership: this struct owns the child process, its stdin,
//! and the reader thread feeding the response channel. `Drop`
//! kills the child. Every request is bounded by
//! [`crate::CALL_TIMEOUT_SECS`] and [`crate::RESPONSE_BYTES_MAX`].

use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{Receiver, channel};
use std::time::{Duration, Instant};

use serde_json::{Value, json};

use crate::facade::{ArdorFacade, FacadeError};
use crate::{CALL_TIMEOUT_SECS, PROTOCOL_VERSION, RESPONSE_BYTES_MAX, TUI_VERSION};

/// An MCP stdio session with one ardor server process.
pub struct McpStdioFacade {
    child: Child,
    stdin: ChildStdin,
    responses: Receiver<String>,
    next_id: u64,
    session: String,
}

impl McpStdioFacade {
    /// Spawn `server_bin` (with `--home <dir>` when given) and
    /// complete the `server/discover` opening.
    pub fn spawn(server_bin: &str, home: Option<&str>) -> Result<Self, FacadeError> {
        let mut command = Command::new(server_bin);
        if let Some(dir) = home {
            command.arg("--home").arg(dir);
        }
        let mut child = command
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| FacadeError::Spawn(format!("{server_bin}: {e}")))?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| FacadeError::Spawn("server stdin unavailable".to_string()))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| FacadeError::Spawn("server stdout unavailable".to_string()))?;
        let (tx, rx) = channel::<String>();
        std::thread::spawn(move || {
            let mut reader = BufReader::new(stdout);
            let mut line = String::new();
            loop {
                line.clear();
                match reader.read_line(&mut line) {
                    Ok(0) | Err(_) => break,
                    Ok(_) => {
                        if tx.send(line.clone()).is_err() {
                            break;
                        }
                    }
                }
            }
        });
        let session = format!("stdio-{}", child.id());
        let mut facade = McpStdioFacade {
            child,
            stdin,
            responses: rx,
            next_id: 0,
            session,
        };
        facade.status()?;
        Ok(facade)
    }

    fn meta() -> Value {
        json!({
            "io.modelcontextprotocol/protocolVersion": PROTOCOL_VERSION,
            "io.modelcontextprotocol/clientInfo": {"name": "ardor-tui", "version": TUI_VERSION},
            "io.modelcontextprotocol/clientCapabilities": {}
        })
    }

    /// Send one JSON-RPC request and wait, bounded, for the
    /// response carrying the same id. Notifications and foreign
    /// ids are skipped, never mistaken for the answer.
    fn request(&mut self, method: &str, params: &Value) -> Result<Value, FacadeError> {
        self.next_id += 1;
        let id = self.next_id;
        let mut params = params.clone();
        if let Some(obj) = params.as_object_mut() {
            obj.insert("_meta".to_string(), Self::meta());
        }
        let line = json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params});
        let mut bytes = line.to_string();
        bytes.push('\n');
        self.stdin
            .write_all(bytes.as_bytes())
            .and_then(|()| self.stdin.flush())
            .map_err(|e| FacadeError::Io(e.to_string()))?;
        let deadline = Instant::now() + Duration::from_secs(CALL_TIMEOUT_SECS);
        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                return Err(FacadeError::Timeout);
            }
            let raw = self
                .responses
                .recv_timeout(remaining)
                .map_err(|_| FacadeError::Dead)?;
            if raw.len() > RESPONSE_BYTES_MAX {
                return Err(FacadeError::Protocol("response line over bound".into()));
            }
            let msg: Value = serde_json::from_str(&raw)
                .map_err(|e| FacadeError::Protocol(format!("bad json: {e}")))?;
            if msg.get("id").and_then(Value::as_u64) != Some(id) {
                continue;
            }
            if let Some(error) = msg.get("error") {
                let code = error
                    .get("code")
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| "?".to_string());
                let message = error
                    .get("message")
                    .and_then(Value::as_str)
                    .unwrap_or("")
                    .to_string();
                return Err(FacadeError::Tool { code, message });
            }
            return Ok(msg.get("result").cloned().unwrap_or(Value::Null));
        }
    }
}

impl ArdorFacade for McpStdioFacade {
    fn session_id(&self) -> &str {
        &self.session
    }

    fn status(&mut self) -> Result<Value, FacadeError> {
        let result = self.request("server/discover", &json!({}))?;
        let info = result
            .get("_meta")
            .and_then(|m| m.get("io.modelcontextprotocol/serverInfo"))
            .cloned()
            .unwrap_or(Value::Null);
        Ok(json!({
            "server": info.get("name"),
            "server_version": info.get("version"),
            "supported_versions": result.get("supportedVersions"),
            "protocol": PROTOCOL_VERSION,
            "session": self.session,
        }))
    }

    fn tools(&mut self) -> Result<Value, FacadeError> {
        self.request("tools/list", &json!({}))
    }

    fn call(&mut self, tool: &str, args: &Value) -> Result<Value, FacadeError> {
        let result = self.request("tools/call", &json!({"name": tool, "arguments": args}))?;
        let text = result
            .get("content")
            .and_then(Value::as_array)
            .and_then(|blocks| blocks.first())
            .and_then(|block| block.get("text"))
            .and_then(Value::as_str)
            .ok_or_else(|| FacadeError::Protocol(format!("{tool}: no text content")))?;
        let is_error = result.get("isError").and_then(Value::as_bool) == Some(true);
        let payload: Value = serde_json::from_str(text).unwrap_or(Value::String(text.to_string()));
        if is_error {
            let error = payload.get("error").cloned().unwrap_or(Value::Null);
            return Err(FacadeError::Tool {
                code: error
                    .get("code")
                    .and_then(Value::as_str)
                    .unwrap_or("domain")
                    .to_string(),
                message: error
                    .get("message")
                    .and_then(Value::as_str)
                    .unwrap_or(text)
                    .to_string(),
            });
        }
        Ok(payload)
    }
}

impl Drop for McpStdioFacade {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
