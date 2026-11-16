//! ardor-tui — the maker console for the ardor MCP server.
//!
//! Fleet TUI pattern (programme D-1562, DESIGN §D2): this crate
//! depends only on ratatui + crossterm + serde_json and programs
//! against a narrow [`facade::ArdorFacade`] trait. The real
//! facade is an MCP stdio client spawning the member's own
//! server binary; tests use the in-memory
//! [`facade::FakeFacade`].
//!
//! Screens: Overview, Workflows, Safety gates, Log/Live, Fleet.
//! There is deliberately **no Media screen**: ardor's playtest
//! tooling records screenshot *specifications*
//! (`add_screenshot_spec`) but produces no image bytes, so there
//! is nothing honest to view (DESIGN §D2 screen 5 is absent, not
//! stubbed, where the member has no media).
//!
//! Safety posture: gate states shown here are parsed from the
//! server's own tool results (`audit_safety`,
//! `publish_preflight`) — never asserted by the TUI. A release
//! handoff may only be staged when, **in the same session**, a
//! playtest run has completed and the server's gates read green
//! (see [`handoff`]); `prepare_handoff` is consequential, so it
//! is plan-mode only (see [`triggers`] and [`workflows`]).

#![forbid(unsafe_code)]

pub mod app;
pub mod client;
pub mod facade;
pub mod gates;
pub mod handoff;
pub mod line;
pub mod panels;
pub mod qa;
pub mod screens;
pub mod triggers;
pub mod workflows;

/// Maximum lines retained in the Log/Live buffer.
pub const LOG_LINES_MAX: usize = 512;
/// Maximum lines retained in the trigger audit tail.
pub const AUDIT_LINES_MAX: usize = 256;
/// Maximum bytes accepted for one operator input line.
pub const INPUT_BYTES_MAX: usize = 4096;
/// Maximum bytes accepted for one MCP response line.
pub const RESPONSE_BYTES_MAX: usize = 8 * 1024 * 1024;
/// Deadline for one MCP request/response round trip, seconds.
pub const CALL_TIMEOUT_SECS: u64 = 30;
/// Maximum characters of one rendered title before truncation.
pub const TITLE_CHARS_MAX: usize = 48;
/// Maximum characters of one rendered log line before truncation.
pub const LINE_CHARS_MAX: usize = 160;
/// Maximum tool arguments payload, bytes, accepted from input.
pub const ARGS_BYTES_MAX: usize = 16 * 1024;
/// Maximum playtest steps replayed as progress events.
pub const PROGRESS_EVENTS_MAX: usize = 64;
/// The one MCP protocol version ardor serves (modern era only).
pub const PROTOCOL_VERSION: &str = "2026-07-28";
/// This crate's version (inherits the ardor workspace version).
pub const TUI_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Truncate `text` to at most `max_chars` characters, marking
/// the cut with an ellipsis. Pure; used by every panel.
pub fn truncate_chars(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_string();
    }
    let keep = max_chars.saturating_sub(1);
    let mut out: String = text.chars().take(keep).collect();
    out.push('\u{2026}');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_bounds_and_marks() {
        assert_eq!(truncate_chars("abc", 5), "abc");
        assert_eq!(truncate_chars("abcdef", 4), "abc\u{2026}");
        assert_eq!(truncate_chars("", 0), "");
        // Multibyte input is cut on char boundaries, never bytes.
        assert_eq!(truncate_chars("héllo", 3), "hé\u{2026}");
    }
}
