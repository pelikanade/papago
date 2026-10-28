//! The mock fleet, ported one-to-one from `prototype/src/data.ts`: Project →
//! persistent orchestrator (read/plan/dispatch only) + Workspaces, each 1:1
//! with a worktree and a single worker agent. Subagents are ephemeral
//! timeline entries on the worker and surface only in the status bar.

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::types::*;

const WORDS: [&str; 10] = ["swift", "brave", "quiet", "amber", "lunar", "coral", "misty", "iron", "sable", "fern"];
const ANIMALS: [&str; 10] = ["fox", "otter", "heron", "lynx", "marten", "ibex", "crane", "vole", "newt", "drake"];

/// A tiny xorshift standing in for `Math.random` (the prototype has no RNG
/// crate; the values only pick mock slugs and jitter vitals).
fn rand_u64() -> u64 {
    static SEED: AtomicU64 = AtomicU64::new(0);
    let mut s = SEED.load(Ordering::Relaxed);
    if s == 0 {
        s = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0x9E3779B97F4A7C15)
            | 1;
    }
    s ^= s << 13;
    s ^= s >> 7;
    s ^= s << 17;
    SEED.store(s, Ordering::Relaxed);
    s
}

/// `Math.random()`, 0..1.
pub fn rand_f64() -> f64 {
    (rand_u64() >> 11) as f64 / (1u64 << 53) as f64
}

pub fn rand_slug() -> String {
    let w = WORDS[(rand_f64() * WORDS.len() as f64) as usize % WORDS.len()];
    let a = ANIMALS[(rand_f64() * ANIMALS.len() as f64) as usize % ANIMALS.len()];
    format!("{w}-{a}")
}

pub const MODELS: [&str; 5] = [
    "Claude Sonnet 5.5",
    "Claude Opus 5.5",
    "Claude Haiku 5.5",
    "GPT-5.6 Luna",
    "DeepSeek V4.1",
];
pub const THINK: [&str; 4] = ["off", "low", "medium", "high"];

pub fn think_label(thinking: &str) -> &'static str {
    match thinking {
        "off" => "Off",
        "low" => "Low",
        "medium" => "Medium",
        _ => "High",
    }
}

pub const APPROVALS: [&str; 3] = ["yolo", "write-ask", "always-ask"];

pub fn approval_label(approval: &str) -> &'static str {
    match approval {
        "yolo" => "Full Access",
        "write-ask" => "Write Ask",
        _ => "Always Ask",
    }
}

pub const BRANCHES: [&str; 3] = ["main", "develop", "release/1.0"];

/// URGENCY (data.ts): the collapsed rail's sort order.
pub fn urgency(status: AgentStatus) -> u8 {
    match status {
        AgentStatus::NeedsYou => 0,
        AgentStatus::Running | AgentStatus::Error => 1,
        AgentStatus::Done => 2,
        AgentStatus::Idle => 3,
    }
}

fn user(text: &str) -> TimelineItem {
    TimelineItem::User { text: text.into() }
}

fn md(text: &str) -> TimelineItem {
    TimelineItem::Md { text: text.into() }
}

fn tool(verb: &str, target: &str, status: ToolStatus, took: &str) -> TimelineItem {
    TimelineItem::Tool {
        verb: verb.into(),
        target: target.into(),
        status,
        took: took.into(),
        diff: None,
        before: None,
        after: None,
    }
}

fn sub(title: &str, status: AgentStatus, model: &str, tool: &str, tokens: &str) -> TimelineItem {
    TimelineItem::Sub {
        title: title.into(),
        status,
        model: model.into(),
        tool: tool.into(),
        tokens: tokens.into(),
    }
}

fn ask(kind: AskKind, q: &str, options: Option<Vec<&str>>, resolved: Option<&str>) -> TimelineItem {
    TimelineItem::Ask {
        kind,
        q: q.into(),
        options: options.map(|o| o.iter().map(|s| s.to_string()).collect()),
        resolved: resolved.map(|s| s.to_string()),
    }
}

/// The mock fleet the app opens on (`initialData`).
#[allow(clippy::too_many_lines)]
pub fn initial_data() -> Fleet {
    Fleet {
        projects: vec![
            Project {
                id: "p1".into(),
                name: "myrepo".into(),
                path: "~/code/myrepo".into(),
                branch: "main".into(),
                orchestrator: Agent {
                    id: "o1".into(),
                    title: "orchestrator".into(),
                    role: AgentRole::Orchestrator,
                    status: AgentStatus::Idle,
                    model: "Claude Opus 5.5".into(),
                    thinking: "high".into(),
                    approval: "yolo".into(),
                    ctx: 9,
                    tps: 0,
                    todo: None,
                    streaming: false,
                    archived: false,
                    queue: vec![],
                    timeline: vec![
                        user("ship the login fix and the rate-limit work"),
                        tool("read", "src/auth", ToolStatus::Done, "0.3s"),
                        tool("bash", "git worktree add ../swift-otter -b fix/login", ToolStatus::Done, "1.1s"),
                        tool("bash", "launch worker swift-otter", ToolStatus::Done, "0.2s"),
                        tool("bash", "git worktree add ../brave-otter -b fix/rate-limit", ToolStatus::Done, "0.9s"),
                        tool("bash", "launch worker brave-otter", ToolStatus::Done, "0.2s"),
                        md("**Plan:** two workspaces created. Workers own write access; I only read, plan, and dispatch."),
                    ],
                },
                workspaces: vec![
                    Workspace {
                        id: "w1".into(),
                        name: "swift-otter".into(),
                        isolation: Isolation::Worktree,
                        branch: "fix/login".into(),
                        path: "~/code/myrepo-wt/swift-otter".into(),
                        dirty: Some("*1".into()),
                        untracked: Some("?4".into()),
                        pr: Some(181),
                        ci: Some(CiStatus::Passed),
                        worker: Agent {
                            id: "a1".into(),
                            title: "fix-login bug".into(),
                            role: AgentRole::Worker,
                            status: AgentStatus::Running,
                            model: "Claude Sonnet 5.5".into(),
                            thinking: "medium".into(),
                            approval: "yolo".into(),
                            ctx: 42,
                            tps: 12,
                            todo: Some((2, 5)),
                            streaming: true,
                            archived: false,
                            queue: vec![],
                            timeline: vec![
                                user("the login redirect loops when the session cookie expires mid-request"),
                                tool("read", "src/auth/session.ts", ToolStatus::Done, "0.4s"),
                                TimelineItem::Tool {
                                    verb: "edit".into(),
                                    target: "src/auth/session.ts".into(),
                                    status: ToolStatus::Done,
                                    took: "1.2s".into(),
                                    diff: Some(DiffStat { add: 12, del: 3 }),
                                    before: Some(vec![
                                        "if (cookie.expired) {".into(),
                                        "  return redirect('/login')".into(),
                                        "}".into(),
                                        "session = refresh(cookie)".into(),
                                    ]),
                                    after: Some(vec![
                                        "session = refresh(cookie)".into(),
                                        "if (cookie.expired) {".into(),
                                        "  return redirect('/login')".into(),
                                        "}".into(),
                                        "touch(lastSeen)".into(),
                                    ]),
                                },
                                tool("bash", "go test ./auth", ToolStatus::Running, "3.1s"),
                                user("also cover the refresh-race path while you are in there"),
                                sub("write regression tests", AgentStatus::Running, "Claude Haiku 5.5", "bash go test", "4.2k"),
                                sub("audit cookie flags", AgentStatus::Done, "Claude Haiku 5.5", "grep cookie", "1.1k"),
                                sub("check header parsing", AgentStatus::Error, "Claude Haiku 5.5", "bash curl", "0.8k"),
                                ask(
                                    AskKind::Confirm,
                                    "Restart the dev server to pick up the new session middleware?",
                                    None,
                                    Some("Approve"),
                                ),
                                ask(
                                    AskKind::Select,
                                    "Which store for the rate-limit counters?",
                                    Some(vec!["in-memory", "redis", "sqlite"]),
                                    Some("redis"),
                                ),
                                md("**Root cause:** the cookie refresh raced the middleware. `refresh(cookie)` now runs before the guard, and the redirect path re-reads the session once.\n\nNext: regression test for the expiry path, then a quick audit of the cookie flags."),
                                md("— previous run aborted — queued follow-ups preserved for the next turn"),
                            ],
                        },
                    },
                    Workspace {
                        id: "w2".into(),
                        name: "brave-otter".into(),
                        isolation: Isolation::Worktree,
                        branch: "fix/rate-limit".into(),
                        path: "~/code/myrepo-wt/brave-otter".into(),
                        dirty: Some("*2".into()),
                        untracked: None,
                        pr: Some(182),
                        ci: Some(CiStatus::Running),
                        worker: Agent {
                            id: "a6".into(),
                            title: "add rate-limit".into(),
                            role: AgentRole::Worker,
                            status: AgentStatus::NeedsYou,
                            model: "Claude Sonnet 5.5".into(),
                            thinking: "medium".into(),
                            approval: "yolo".into(),
                            ctx: 61,
                            tps: 0,
                            todo: Some((1, 4)),
                            streaming: false,
                            archived: false,
                            queue: vec!["run the full suite".into(), "bump deps".into()],
                            timeline: vec![
                                user("add per-IP rate limiting to the public API"),
                                tool("bash", "go test ./... -run Rate", ToolStatus::Done, "8.1s"),
                                sub("check header parsing", AgentStatus::Error, "Claude Haiku 5.5", "bash curl", "0.8k"),
                                ask(
                                    AskKind::Select,
                                    "Which store for the rate-limit counters?",
                                    Some(vec!["in-memory", "redis", "sqlite"]),
                                    None,
                                ),
                            ],
                        },
                    },
                    Workspace {
                        id: "w3".into(),
                        name: "tidy-docs".into(),
                        isolation: Isolation::Worktree,
                        branch: "docs/v2".into(),
                        path: "~/code/myrepo-wt/tidy-docs".into(),
                        dirty: None,
                        untracked: None,
                        pr: None,
                        ci: None,
                        worker: Agent {
                            id: "a7".into(),
                            title: "docs pass".into(),
                            role: AgentRole::Worker,
                            status: AgentStatus::Done,
                            model: "Claude Haiku 5.5".into(),
                            thinking: "off".into(),
                            approval: "yolo".into(),
                            ctx: 18,
                            tps: 0,
                            todo: Some((3, 3)),
                            streaming: false,
                            archived: false,
                            queue: vec![],
                            timeline: vec![
                                user("refresh the API docs for v2 endpoints"),
                                tool("read", "docs/api-v2.md", ToolStatus::Done, "0.3s"),
                                tool("write", "docs/api-v2.md", ToolStatus::Done, "0.8s"),
                                md("Docs updated: 14 endpoints, 3 new examples."),
                            ],
                        },
                    },
                    Workspace {
                        id: "w4".into(),
                        name: "lint-clean".into(),
                        isolation: Isolation::Worktree,
                        branch: "chore/lint".into(),
                        path: "~/code/myrepo-wt/lint-clean".into(),
                        dirty: None,
                        untracked: None,
                        pr: None,
                        ci: None,
                        worker: Agent {
                            id: "a8".into(),
                            title: "lint cleanup".into(),
                            role: AgentRole::Worker,
                            status: AgentStatus::Error,
                            model: "DeepSeek V4.1".into(),
                            thinking: "low".into(),
                            approval: "always-ask".into(),
                            ctx: 27,
                            tps: 0,
                            todo: Some((0, 2)),
                            streaming: false,
                            archived: false,
                            queue: vec![],
                            timeline: vec![
                                user("clear the lint backlog on this branch"),
                                tool("bash", "eslint .", ToolStatus::Error, "2.2s"),
                                md("Failed: `eslint` not installed in this worktree (exit 127). Stopped."),
                            ],
                        },
                    },
                    Workspace {
                        id: "w5".into(),
                        name: "review-pr".into(),
                        isolation: Isolation::Worktree,
                        branch: "review/182".into(),
                        path: "~/code/myrepo-wt/review-pr".into(),
                        dirty: None,
                        untracked: None,
                        pr: None,
                        ci: None,
                        worker: Agent {
                            id: "a10".into(),
                            title: "review PR 182".into(),
                            role: AgentRole::Worker,
                            status: AgentStatus::NeedsYou,
                            model: "Claude Opus 5.5".into(),
                            thinking: "high".into(),
                            approval: "yolo".into(),
                            ctx: 33,
                            tps: 0,
                            todo: None,
                            streaming: false,
                            archived: false,
                            queue: vec![],
                            timeline: vec![
                                user("review PR 182 for the rate-limit change"),
                                tool("bash", "gh pr diff 182", ToolStatus::Done, "1.9s"),
                                ask(
                                    AskKind::Input,
                                    "Paste the failing CI log, or submit empty to fetch it from Actions.",
                                    None,
                                    None,
                                ),
                            ],
                        },
                    },
                    Workspace {
                        id: "w6".into(),
                        name: "triage-482".into(),
                        isolation: Isolation::Worktree,
                        branch: "triage/482".into(),
                        path: "~/code/myrepo-wt/triage-482".into(),
                        dirty: None,
                        untracked: None,
                        pr: None,
                        ci: None,
                        worker: Agent {
                            id: "a9".into(),
                            title: "triage issue #482".into(),
                            role: AgentRole::Worker,
                            status: AgentStatus::Idle,
                            model: "GPT-5.6 Luna".into(),
                            thinking: "low".into(),
                            approval: "write-ask".into(),
                            ctx: 21,
                            tps: 0,
                            todo: None,
                            streaming: false,
                            archived: false,
                            queue: vec![],
                            timeline: vec![
                                user("triage issue #482 — the lint job fails on main"),
                                md("— new conversation —"),
                                tool("bash", "npm run lint", ToolStatus::Error, "4.4s"),
                                md("Lint failed with 3 errors in `src/auth/session.ts`. Paused here; rerun after the fix lands."),
                            ],
                        },
                    },
                ],
            },
            Project {
                id: "p2".into(),
                name: "papago-docs".into(),
                path: "~/code/papago-docs".into(),
                branch: "main".into(),
                orchestrator: Agent {
                    id: "o2".into(),
                    title: "orchestrator".into(),
                    role: AgentRole::Orchestrator,
                    status: AgentStatus::Idle,
                    model: "GPT-5.6 Luna".into(),
                    thinking: "medium".into(),
                    approval: "yolo".into(),
                    ctx: 5,
                    tps: 0,
                    todo: None,
                    streaming: false,
                    archived: false,
                    queue: vec![],
                    timeline: vec![
                        user("regenerate the OpenAPI spec after the v2 rename"),
                        tool("bash", "launch worker regen-openapi", ToolStatus::Done, "0.2s"),
                        md("Handed off to worker `regen-openapi`."),
                    ],
                },
                workspaces: vec![Workspace {
                    id: "w7".into(),
                    name: "regen-openapi".into(),
                    isolation: Isolation::Worktree,
                    branch: "docs/openapi".into(),
                    path: "~/code/papago-docs-wt/regen-openapi".into(),
                    dirty: None,
                    untracked: None,
                    pr: Some(179),
                    ci: Some(CiStatus::Failed),
                    worker: Agent {
                        id: "a13".into(),
                        title: "regenerate OpenAPI".into(),
                        role: AgentRole::Worker,
                        status: AgentStatus::NeedsYou,
                        model: "Claude Sonnet 5.5".into(),
                        thinking: "medium".into(),
                        approval: "write-ask".into(),
                        ctx: 15,
                        tps: 0,
                        todo: None,
                        streaming: false,
                        archived: false,
                        queue: vec![],
                        timeline: vec![
                            user("regenerate the OpenAPI spec after the v2 rename"),
                            tool("bash", "go run ./cmd/openapi", ToolStatus::Done, "2.7s"),
                            ask(
                                AskKind::Confirm,
                                "Overwrite docs/openapi.yaml with the regenerated spec?",
                                None,
                                None,
                            ),
                        ],
                    },
                }],
            },
        ],
    }
}
