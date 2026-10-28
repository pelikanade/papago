//! Domain types for the papago prototype, mirroring `prototype/src/types.ts`
//! (which mirrors `GLOSSARY.md`).

/// An agent's status. Dot semantics: gray idle · blue slow-pulse working ·
/// yellow needs-attention · green done · red failed.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AgentStatus {
    Running,
    NeedsYou,
    Done,
    Idle,
    Error,
}

impl AgentStatus {
    /// The lowercase label the prototype writes into the UI
    /// (`{w.ci}`, `{a.status}`, palette hints).
    pub fn label(self) -> &'static str {
        match self {
            AgentStatus::Running => "running",
            AgentStatus::NeedsYou => "needs-you",
            AgentStatus::Done => "done",
            AgentStatus::Idle => "idle",
            AgentStatus::Error => "error",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Isolation {
    Worktree,
    /// A local checkout (the mock fleet only spawns worktrees).
    #[allow(dead_code)]
    Local,
}

impl Isolation {
    pub fn label(self) -> &'static str {
        match self {
            Isolation::Worktree => "worktree",
            Isolation::Local => "local",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AgentRole {
    Orchestrator,
    Worker,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ToolStatus {
    Done,
    Running,
    Error,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AskKind {
    Select,
    Confirm,
    Input,
}

impl AskKind {
    pub fn label(self) -> &'static str {
        match self {
            AskKind::Select => "select",
            AskKind::Confirm => "confirm",
            AskKind::Input => "input",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CiStatus {
    Passed,
    Failed,
    Running,
}

impl CiStatus {
    pub fn label(self) -> &'static str {
        match self {
            CiStatus::Passed => "passed",
            CiStatus::Failed => "failed",
            CiStatus::Running => "running",
        }
    }
}

/// `+12 −3` on an edit's row.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct DiffStat {
    pub add: i64,
    pub del: i64,
}

/// One timeline entry.
#[derive(Clone, PartialEq, Debug)]
pub enum TimelineItem {
    User {
        text: String,
    },
    Md {
        text: String,
    },
    Tool {
        verb: String,
        target: String,
        status: ToolStatus,
        took: String,
        diff: Option<DiffStat>,
        before: Option<Vec<String>>,
        after: Option<Vec<String>>,
    },
    /// Ephemeral subagents: they live only inside the worker's timeline and
    /// surface in the status bar as background tasks — never as sidebar rows.
    Sub {
        title: String,
        status: AgentStatus,
        model: String,
        tool: String,
        tokens: String,
    },
    Ask {
        kind: AskKind,
        q: String,
        options: Option<Vec<String>>,
        resolved: Option<String>,
    },
}

#[derive(Clone, PartialEq, Debug)]
pub struct Agent {
    pub id: String,
    pub title: String,
    pub role: AgentRole,
    pub status: AgentStatus,
    pub model: String,
    pub thinking: String,
    pub approval: String,
    /// Context usage, percent.
    pub ctx: u32,
    /// Tokens per second.
    pub tps: u32,
    /// `todo done/total`.
    pub todo: Option<(u32, u32)>,
    pub streaming: bool,
    pub archived: bool,
    pub queue: Vec<String>,
    pub timeline: Vec<TimelineItem>,
}

/// One workspace = one worktree = one worker agent (1:1, created by the
/// project-level orchestrator).
#[derive(Clone, PartialEq, Debug)]
pub struct Workspace {
    pub id: String,
    pub name: String,
    pub isolation: Isolation,
    pub branch: String,
    pub path: String,
    /// `*1` dirty marker.
    pub dirty: Option<String>,
    /// `?4` untracked marker.
    pub untracked: Option<String>,
    pub pr: Option<u32>,
    pub ci: Option<CiStatus>,
    pub worker: Agent,
}

#[derive(Clone, PartialEq, Debug)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub path: String,
    pub branch: String,
    /// Persistent project-level agent: read + plan + create
    /// workspaces/workers only.
    pub orchestrator: Agent,
    pub workspaces: Vec<Workspace>,
}

#[derive(Clone, PartialEq, Debug, Default)]
pub struct Fleet {
    pub projects: Vec<Project>,
}

/// What the main pane shows.
#[derive(Clone, PartialEq, Debug)]
pub enum Sel {
    Project { id: String },
    Workspace { id: String },
    Agent { id: String },
}

/// Per-Project last-used composer values (defaults remember).
#[derive(Clone, PartialEq, Debug)]
pub struct Chips {
    pub model: String,
    pub thinking: String,
    pub approval: String,
}
