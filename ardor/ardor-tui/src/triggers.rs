//! The cross-MCP trigger layer for ardor (DESIGN §D4).
//!
//! Rules are data (`triggers.toml`, embedded verbatim and
//! loadable from a file), listed in the Fleet panel; a rule not
//! present there does not exist. The rung-1 rule set has one
//! rule: `playtest.completed` with all gates green proposes a
//! release handoff. `prepare_handoff` is consequential, so the
//! rule fires in **plan mode only** — the engine produces a
//! proposal for a human to approve; it never calls the tool.
//!
//! Semantics: bounded predicate (the single named condition
//! `gates_green`, no expression language), idempotency keyed by
//! a hash of (rule id, event payload), depth cap from the rule
//! (hard ceiling 2), and an audit line for every match, firing,
//! refusal, and drop. Every rule and audit line carries
//! `origin_lane` + `content_label` from birth (BLOCH lane law):
//! this member's are `ntranch` / `nsfw`.

use std::collections::{HashSet, VecDeque};
use std::hash::{DefaultHasher, Hash, Hasher};

use crate::gates::{GateSnapshot, GateVerdict};
use crate::qa::PlaytestRun;
use crate::workflows::Mode;
use crate::{AUDIT_LINES_MAX, truncate_chars};

/// Hard ceiling on trigger depth, whatever a rule declares.
pub const DEPTH_CEILING: u32 = 2;

/// The embedded rule file, verbatim.
pub const EMBEDDED_RULES: &str = include_str!("../triggers.toml");

/// One declared trigger rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TriggerRule {
    /// Stable rule id.
    pub id: String,
    /// Lane the rule was born in (`ntranch` here).
    pub origin_lane: String,
    /// Content label (`nsfw` here).
    pub content_label: String,
    /// Event name that can fire the rule.
    pub on_event: String,
    /// The bounded predicate: require green gates.
    pub require_gates_green: bool,
    /// Target MCP member name.
    pub target_mcp: String,
    /// Target tool the proposal would call.
    pub target_tool: String,
    /// Fire mode; consequential targets must be Plan.
    pub mode: Mode,
    /// Idempotency strategy name (only `event-hash` exists).
    pub idempotency: String,
    /// Maximum event depth this rule fires at.
    pub max_depth: u32,
    /// Operator enable state (Fleet panel can disable).
    pub enabled: bool,
}

/// A typed event on the console's own event stream.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Event {
    /// A playtest run completed and validated.
    PlaytestCompleted {
        /// The run.
        run: PlaytestRun,
        /// Trigger depth of the producing call (0 = operator).
        depth: u32,
    },
}

impl Event {
    /// The event's declared name.
    pub fn name(&self) -> &'static str {
        match self {
            Event::PlaytestCompleted { .. } => "playtest.completed",
        }
    }

    /// The event's trigger depth.
    pub fn depth(&self) -> u32 {
        match self {
            Event::PlaytestCompleted { depth, .. } => *depth,
        }
    }
}

/// A plan-mode proposal produced by a firing rule. Approving
/// it is a human act in the console; the engine stops here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Proposal {
    /// Rule that produced this proposal.
    pub rule_id: String,
    /// Project the handoff would target.
    pub project: String,
    /// Playtest hash evidencing the proposal.
    pub playtest_hash: String,
    /// Target tool (always plan mode for consequential tools).
    pub target_tool: String,
    /// Fire mode (Plan for the rung-1 rule).
    pub mode: Mode,
    /// Birth lane stamp.
    pub origin_lane: String,
    /// Birth content label.
    pub content_label: String,
}

/// What the engine did with one event.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EngineAction {
    /// A rule fired and produced this proposal.
    Proposed(Proposal),
    /// A matching event was dropped (duplicate / depth / off).
    Dropped(String),
    /// A matching event was refused (predicate unmet).
    Refused(String),
}

/// The trigger engine: rules + idempotency set + audit tail.
pub struct TriggerEngine {
    rules: Vec<TriggerRule>,
    fired: HashSet<u64>,
    audit: VecDeque<String>,
}

impl TriggerEngine {
    /// Build an engine from parsed rules.
    pub fn new(rules: Vec<TriggerRule>) -> Self {
        TriggerEngine {
            rules,
            fired: HashSet::new(),
            audit: VecDeque::new(),
        }
    }

    /// The declared rules (Fleet panel source).
    pub fn rules(&self) -> &[TriggerRule] {
        &self.rules
    }

    /// Enable/disable one rule by id; takes effect immediately.
    pub fn set_enabled(&mut self, id: &str, enabled: bool) -> bool {
        match self.rules.iter_mut().find(|r| r.id == id) {
            Some(rule) => {
                rule.enabled = enabled;
                true
            }
            None => false,
        }
    }

    /// The bounded audit tail, oldest first.
    pub fn audit(&self) -> Vec<String> {
        self.audit.iter().cloned().collect()
    }

    fn log_audit(&mut self, line: String) {
        if self.audit.len() >= AUDIT_LINES_MAX {
            self.audit.pop_front();
        }
        self.audit.push_back(truncate_chars(&line, 300));
    }

    /// Evaluate one event against every rule. Gate state is the
    /// caller's current snapshot — the engine never reads gates
    /// itself, so a stale or unknown snapshot cannot fire.
    pub fn on_event(&mut self, event: &Event, gates: Option<&GateSnapshot>) -> Vec<EngineAction> {
        let mut actions = Vec::new();
        for rule in self.rules.clone() {
            if rule.on_event != event.name() {
                continue;
            }
            actions.push(self.evaluate(&rule, event, gates));
        }
        actions
    }

    fn evaluate(
        &mut self,
        rule: &TriggerRule,
        event: &Event,
        gates: Option<&GateSnapshot>,
    ) -> EngineAction {
        let stamp = format!("lane={} label={}", rule.origin_lane, rule.content_label);
        if !rule.enabled {
            let msg = format!(
                "drop rule={} event={} reason=disabled {stamp}",
                rule.id,
                event.name()
            );
            self.log_audit(msg.clone());
            return EngineAction::Dropped(msg);
        }
        let cap = rule.max_depth.min(DEPTH_CEILING);
        if event.depth() > cap {
            let msg = format!(
                "drop rule={} event={} reason=depth {} > {cap} {stamp}",
                rule.id,
                event.name(),
                event.depth()
            );
            self.log_audit(msg.clone());
            return EngineAction::Dropped(msg);
        }
        let Event::PlaytestCompleted { run, .. } = event;
        if rule.require_gates_green {
            let green = matches!(
                gates,
                Some(snap) if snap.verdict == GateVerdict::Green
                    && snap.project == run.project
            );
            if !green {
                let msg = format!(
                    "refuse rule={} event={} project={} reason=gates-not-green {stamp}",
                    rule.id,
                    event.name(),
                    run.project
                );
                self.log_audit(msg.clone());
                return EngineAction::Refused(msg);
            }
        }
        let key = event_key(&rule.id, run);
        if !self.fired.insert(key) {
            let msg = format!(
                "drop rule={} event={} project={} reason=duplicate {stamp}",
                rule.id,
                event.name(),
                run.project
            );
            self.log_audit(msg.clone());
            return EngineAction::Dropped(msg);
        }
        let proposal = Proposal {
            rule_id: rule.id.clone(),
            project: run.project.clone(),
            playtest_hash: run.hash.clone(),
            target_tool: rule.target_tool.clone(),
            mode: rule.mode,
            origin_lane: rule.origin_lane.clone(),
            content_label: rule.content_label.clone(),
        };
        self.log_audit(format!(
            "fire rule={} event={} project={} tool={} mode={} {stamp}",
            rule.id,
            event.name(),
            run.project,
            rule.target_tool,
            rule.mode.label()
        ));
        EngineAction::Proposed(proposal)
    }
}

fn event_key(rule_id: &str, run: &PlaytestRun) -> u64 {
    let mut hasher = DefaultHasher::new();
    rule_id.hash(&mut hasher);
    run.project.hash(&mut hasher);
    run.hash.hash(&mut hasher);
    run.seed.hash(&mut hasher);
    hasher.finish()
}

/// Parse the bounded `triggers.toml` shape: `[[rule]]` blocks
/// of `key = "value"` lines plus the inline `on = { .. }` /
/// `then = { .. }` tables. This is a deliberately small parser
/// for this one file shape — no expression language, unknown
/// keys are errors, and a consequential target tool in auto
/// mode is refused at load.
pub fn parse_rules(text: &str) -> Result<Vec<TriggerRule>, String> {
    let mut rules = Vec::new();
    let mut block: Vec<String> = Vec::new();
    let mut in_block = false;
    for raw_line in text.lines() {
        let line = raw_line.trim();
        if line == "[[rule]]" {
            if in_block {
                rules.push(parse_block(&block)?);
                block.clear();
            }
            in_block = true;
            continue;
        }
        if !in_block || line.is_empty() || line.starts_with('#') {
            continue;
        }
        block.push(line.to_string());
    }
    if in_block {
        rules.push(parse_block(&block)?);
    }
    Ok(rules)
}

fn quoted(line: &str, key: &str) -> Option<String> {
    let start = line.find(key)? + key.len();
    let rest = line[start..].trim_start_matches([' ', '=']);
    let inner = rest.strip_prefix('"')?;
    let end = inner.find('"')?;
    Some(inner[..end].to_string())
}

fn block_value<'a>(block: &'a [String], key: &str) -> Option<&'a str> {
    block
        .iter()
        .map(String::as_str)
        .find(|line| line.starts_with(key))
}

fn parse_block(block: &[String]) -> Result<TriggerRule, String> {
    let get = |key: &str| -> Result<String, String> {
        let line = block_value(block, key).ok_or(format!("rule missing {key}"))?;
        quoted(line, key).ok_or(format!("rule {key} is not a quoted string"))
    };
    let on_line = block_value(block, "on").ok_or("rule missing on")?;
    let then_line = block_value(block, "then").ok_or("rule missing then")?;
    let mode = match get("mode")?.as_str() {
        "plan" => Mode::Plan,
        "auto" => Mode::Auto,
        other => return Err(format!("unknown trigger mode {other}")),
    };
    let target_tool = quoted(then_line, "tool").ok_or("rule then missing tool")?;
    // Consequential targets may only ever fire in plan mode
    // (DESIGN §D4 gate rule): refuse the rule at load.
    if target_tool == "prepare_handoff" && mode != Mode::Plan {
        return Err("prepare_handoff trigger must be plan mode".to_string());
    }
    let max_depth = block_value(block, "max_depth")
        .and_then(|line| line.split('=').nth(1))
        .and_then(|v| v.trim().parse::<u32>().ok())
        .ok_or("rule max_depth missing or not a number")?;
    Ok(TriggerRule {
        id: get("id")?,
        origin_lane: get("origin_lane")?,
        content_label: get("content_label")?,
        on_event: quoted(on_line, "event").ok_or("rule on missing event")?,
        require_gates_green: quoted(on_line, "where")
            .map(|w| w == "gates_green")
            .unwrap_or(false),
        target_mcp: quoted(then_line, "mcp").ok_or("rule then missing mcp")?,
        target_tool,
        mode,
        idempotency: get("idempotency")?,
        max_depth,
        enabled: block_value(block, "enabled")
            .map(|line| line.contains("true"))
            .unwrap_or(true),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gates::GateSnapshot;
    use serde_json::json;

    const HASH: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

    fn run() -> PlaytestRun {
        PlaytestRun {
            project: "demo".to_string(),
            id: "run-1".to_string(),
            seed: 1,
            hash: HASH.to_string(),
            steps_count: 2,
            stop_reason: "terminal".to_string(),
            truncated: false,
            step_nodes: Vec::new(),
        }
    }

    fn green() -> GateSnapshot {
        GateSnapshot::from_audit(
            "s1",
            &json!({"project": "demo", "audit": {"ok": true, "checks": [
                {"check": "safety_surface:stop", "ok": true, "detail": "ok"}
            ]}}),
        )
        .expect("green")
    }

    fn engine() -> TriggerEngine {
        TriggerEngine::new(parse_rules(EMBEDDED_RULES).expect("embedded rules parse"))
    }

    #[test]
    fn embedded_rules_parse_with_lane_stamps() {
        let rules = parse_rules(EMBEDDED_RULES).expect("parse");
        assert_eq!(rules.len(), 1);
        let rule = &rules[0];
        assert_eq!(rule.origin_lane, "ntranch");
        assert_eq!(rule.content_label, "nsfw");
        assert_eq!(rule.on_event, "playtest.completed");
        assert!(rule.require_gates_green);
        assert_eq!(rule.target_tool, "prepare_handoff");
        assert_eq!(rule.mode, Mode::Plan);
        assert_eq!(rule.max_depth, 1);
    }

    #[test]
    fn green_playtest_proposes_handoff_in_plan_mode() {
        let mut eng = engine();
        let event = Event::PlaytestCompleted {
            run: run(),
            depth: 0,
        };
        let actions = eng.on_event(&event, Some(&green()));
        assert_eq!(actions.len(), 1);
        match &actions[0] {
            EngineAction::Proposed(p) => {
                assert_eq!(p.mode, Mode::Plan);
                assert_eq!(p.origin_lane, "ntranch");
                assert_eq!(p.content_label, "nsfw");
                assert_eq!(p.project, "demo");
            }
            other => panic!("expected proposal, got {other:?}"),
        }
        assert!(eng.audit().iter().any(|l| l.contains("fire rule=")));
    }

    #[test]
    fn red_or_unknown_gates_refuse() {
        let mut eng = engine();
        let event = Event::PlaytestCompleted {
            run: run(),
            depth: 0,
        };
        let actions = eng.on_event(&event, None);
        assert!(matches!(actions[0], EngineAction::Refused(_)));
        let unknown = GateSnapshot::unknown("s1", "demo", "dead");
        let actions = eng.on_event(&event, Some(&unknown));
        assert!(matches!(actions[0], EngineAction::Refused(_)));
    }

    #[test]
    fn duplicate_event_is_dropped() {
        let mut eng = engine();
        let event = Event::PlaytestCompleted {
            run: run(),
            depth: 0,
        };
        let first = eng.on_event(&event, Some(&green()));
        assert!(matches!(first[0], EngineAction::Proposed(_)));
        let second = eng.on_event(&event, Some(&green()));
        assert!(matches!(second[0], EngineAction::Dropped(_)));
    }

    #[test]
    fn depth_beyond_cap_is_dropped() {
        let mut eng = engine();
        let event = Event::PlaytestCompleted {
            run: run(),
            depth: 2,
        };
        let actions = eng.on_event(&event, Some(&green()));
        assert!(matches!(actions[0], EngineAction::Dropped(_)));
    }

    #[test]
    fn disabled_rule_drops_and_reenable_fires() {
        let mut eng = engine();
        assert!(eng.set_enabled("ardor-playtest-to-handoff", false));
        let event = Event::PlaytestCompleted {
            run: run(),
            depth: 0,
        };
        let actions = eng.on_event(&event, Some(&green()));
        assert!(matches!(actions[0], EngineAction::Dropped(_)));
        assert!(eng.set_enabled("ardor-playtest-to-handoff", true));
        let actions = eng.on_event(&event, Some(&green()));
        assert!(matches!(actions[0], EngineAction::Proposed(_)));
    }

    #[test]
    fn auto_mode_handoff_rule_is_refused_at_load() {
        let bad = EMBEDDED_RULES.replace("mode = \"plan\"", "mode = \"auto\"");
        assert!(parse_rules(&bad).is_err());
    }

    #[test]
    fn malformed_rules_are_refused() {
        assert!(parse_rules("[[rule]]\nid = \"x\"\n").is_err());
        assert!(parse_rules("[[rule]]\nid = 7\n").is_err());
    }
}
