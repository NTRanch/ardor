//! The workflow catalogue: ardor's primary operations as a
//! selectable list (DESIGN §D2 screen 2).
//!
//! Every workflow names the server tool it drives. Two mode
//! rules are structural, not preferences:
//!
//! - **Consequential** workflows (writes, records, handoffs)
//!   default to plan mode: the exact tool + args are staged and
//!   shown, and nothing is called until approval.
//! - `prepare_handoff` is **plan-only**: auto mode is refused
//!   for it outright, and execution additionally passes the
//!   [`crate::handoff`] evidence state machine.
//!
//! Provider routing (cloud/local) is honestly absent: ardor
//! routes no model calls, so there is no provider control to
//! show (DESIGN §D3).

/// Run mode for one workflow invocation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Stage the call, show exact tool + args, wait for approval.
    Plan,
    /// Execute a declared operation directly.
    Auto,
}

impl Mode {
    /// Short label for panels.
    pub fn label(self) -> &'static str {
        match self {
            Mode::Plan => "plan",
            Mode::Auto => "auto",
        }
    }
}

/// Workflow grouping for the Workflows screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Group {
    /// Authoring operations (projects, cast, canon, story).
    Authoring,
    /// Playtest QA runs.
    Qa,
    /// Release handoffs and their preflight.
    Release,
}

impl Group {
    /// Short label for panels.
    pub fn label(self) -> &'static str {
        match self {
            Group::Authoring => "authoring",
            Group::Qa => "playtest QA",
            Group::Release => "release",
        }
    }
}

/// One selectable workflow.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkflowDef {
    /// Stable id used on the command line (`run <id>`).
    pub id: &'static str,
    /// Human title.
    pub title: &'static str,
    /// The ardor tool this workflow calls.
    pub tool: &'static str,
    /// Grouping.
    pub group: Group,
    /// Consequential: defaults to plan mode.
    pub consequential: bool,
    /// Plan-only: auto mode is refused outright.
    pub plan_only: bool,
    /// Whether the workflow targets the selected project.
    pub needs_project: bool,
}

/// The full catalogue, in display order.
pub fn catalogue() -> Vec<WorkflowDef> {
    vec![
        WorkflowDef {
            id: "validate-story",
            title: "Validate story graph",
            tool: "validate_story",
            group: Group::Authoring,
            consequential: false,
            plan_only: false,
            needs_project: true,
        },
        WorkflowDef {
            id: "list-characters",
            title: "List cast + amendment history",
            tool: "list_characters",
            group: Group::Authoring,
            consequential: false,
            plan_only: false,
            needs_project: true,
        },
        WorkflowDef {
            id: "list-nodes",
            title: "List graph nodes",
            tool: "list_nodes",
            group: Group::Authoring,
            consequential: false,
            plan_only: false,
            needs_project: true,
        },
        WorkflowDef {
            id: "upsert-character",
            title: "Create/update a cast member (gate-checked)",
            tool: "upsert_character",
            group: Group::Authoring,
            consequential: true,
            plan_only: false,
            needs_project: true,
        },
        WorkflowDef {
            id: "create-project",
            title: "Scaffold a new project",
            tool: "create_project",
            group: Group::Authoring,
            consequential: true,
            plan_only: false,
            needs_project: false,
        },
        WorkflowDef {
            id: "generate-playtest",
            title: "Generate playtest (not recorded)",
            tool: "generate_playtest",
            group: Group::Qa,
            consequential: false,
            plan_only: false,
            needs_project: true,
        },
        WorkflowDef {
            id: "run-playtest",
            title: "Run + record playtest QA run",
            tool: "run_playtest",
            group: Group::Qa,
            consequential: true,
            plan_only: false,
            needs_project: true,
        },
        WorkflowDef {
            id: "verify-replay",
            title: "Verify a recorded run replays identically",
            tool: "verify_replay",
            group: Group::Qa,
            consequential: false,
            plan_only: false,
            needs_project: true,
        },
        WorkflowDef {
            id: "list-defects",
            title: "List QA defects",
            tool: "list_defects",
            group: Group::Qa,
            consequential: false,
            plan_only: false,
            needs_project: true,
        },
        WorkflowDef {
            id: "publish-preflight",
            title: "Publish preflight (channel + gates + auth)",
            tool: "publish_preflight",
            group: Group::Release,
            consequential: false,
            plan_only: false,
            needs_project: true,
        },
        WorkflowDef {
            id: "prepare-handoff",
            title: "Prepare release handoff (plan-only)",
            tool: "prepare_handoff",
            group: Group::Release,
            consequential: true,
            plan_only: true,
            needs_project: true,
        },
    ]
}

/// Look one workflow up by id.
pub fn by_id(id: &str) -> Option<WorkflowDef> {
    catalogue().into_iter().find(|w| w.id == id)
}

/// The effective mode for a workflow under a requested mode:
/// plan-only workflows are always Plan; consequential ones
/// default to Plan unless Auto was explicitly requested for a
/// non-plan-only workflow.
pub fn effective_mode(def: &WorkflowDef, requested: Mode) -> Mode {
    if def.plan_only {
        return Mode::Plan;
    }
    requested
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handoff_is_plan_only_and_consequential() {
        let def = by_id("prepare-handoff").expect("handoff workflow");
        assert!(def.plan_only);
        assert!(def.consequential);
        assert_eq!(effective_mode(&def, Mode::Auto), Mode::Plan);
    }

    #[test]
    fn catalogue_ids_are_unique_and_tools_named() {
        let cat = catalogue();
        let mut ids: Vec<&str> = cat.iter().map(|w| w.id).collect();
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(ids.len(), cat.len());
        assert!(cat.iter().all(|w| !w.tool.is_empty()));
    }

    #[test]
    fn read_workflows_may_run_auto() {
        let def = by_id("validate-story").expect("validate");
        assert!(!def.consequential);
        assert_eq!(effective_mode(&def, Mode::Auto), Mode::Auto);
    }
}
