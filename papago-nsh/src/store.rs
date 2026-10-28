//! The papago state machine, ported one-to-one from `prototype/src/store.ts`
//! (the contract the Go daemon/client will mirror): views are pure functions
//! of this store and mutate it only through the named actions below.
//! Zustand's module store becomes a `Store` of NSH signals, created by the
//! root component and provided through context; every action keeps the
//! prototype's name and transition. Model: Project → persistent orchestrator
//! + Workspaces (1:1 worktree + worker).

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use nsh_core::prelude::*;

use crate::data::{rand_f64, rand_slug, APPROVALS, MODELS, THINK, initial_data};
use crate::theme::{DEFAULT_THEME, THEME_ORDER};
use crate::types::*;

pub use crate::types::Sel;

/// The `win` union: in-app window states the pane header's own controls
/// drive (the prototype draws its own chrome).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Win {
    Normal,
    Maximized,
    Minimized,
    Closed,
}

pub struct Store {
    // ---- domain ----
    pub fleet: Signal<Fleet>,
    pub sel: Signal<Option<Sel>>,
    /// Per-Project last-used values (defaults remember).
    pub chips: Signal<HashMap<String, Chips>>,
    /// Random slug shown in the Project spawn status bar (display-only).
    pub slug_preview: Signal<String>,
    /// Diagnostics view switch.
    pub omp_missing: Signal<bool>,
    /// base16 theme (default Nord Dark).
    pub theme_id: Signal<&'static str>,
    pub win: Signal<Win>,

    // ---- ui (kept in the machine so every view stays a pure function) ----
    pub sidebar_collapsed: Signal<bool>,
    pub hub_open: Signal<bool>,
    pub palette_open: Signal<bool>,
    pub palette_query: Signal<String>,
    /// Composer/inline-input text per key.
    pub drafts: Signal<HashMap<String, String>>,
    /// Tool-card fold state, key `${agentId}:${index}`.
    pub folds: Signal<HashMap<String, bool>>,
    /// Queue/background/tree/sub popovers.
    pub popovers: Signal<HashMap<String, bool>>,
    /// Per-Agent timeline tail-follow.
    pub follow_tail: Signal<HashMap<String, bool>>,

    /// Whether any Project exists — a memo so the shell's routing gate
    /// re-renders on a project count change only, not on every fleet tick
    /// (zustand's selector equality does the same for the React app).
    pub has_projects: Memo<bool>,

    /// The `dot-pulse` keyframe value (2.4s) every running dot binds its
    /// opacity to; one animation for all of them, as the CSS class is one.
    pub pulse: AnimatedValue,
    /// Tailwind's `animate-pulse` (2s) the hollow orchestrator ring uses.
    pub ring_pulse: AnimatedValue,

    /// The mock RNG and id counter.
    rng: Arc<Mutex<()>>,
    ids: Arc<Mutex<u64>>,
}

impl Clone for Store {
    fn clone(&self) -> Self {
        Self {
            fleet: self.fleet.clone(),
            sel: self.sel.clone(),
            chips: self.chips.clone(),
            slug_preview: self.slug_preview.clone(),
            omp_missing: self.omp_missing.clone(),
            theme_id: self.theme_id.clone(),
            win: self.win.clone(),
            sidebar_collapsed: self.sidebar_collapsed.clone(),
            hub_open: self.hub_open.clone(),
            palette_open: self.palette_open.clone(),
            palette_query: self.palette_query.clone(),
            drafts: self.drafts.clone(),
            folds: self.folds.clone(),
            popovers: self.popovers.clone(),
            follow_tail: self.follow_tail.clone(),
            has_projects: self.has_projects.clone(),
            pulse: self.pulse.clone(),
            ring_pulse: self.ring_pulse.clone(),
            rng: self.rng.clone(),
            ids: self.ids.clone(),
        }
    }
}

/// The store from context (the root component provides it).
pub fn store() -> Store {
    use_context::<Store>().expect("the papago store is in context")
}

fn default_chips() -> Chips {
    Chips {
        model: MODELS[0].into(),
        thinking: THINK[2].into(),
        approval: APPROVALS[0].into(),
    }
}

fn title_of(prompt: &str) -> String {
    prompt.split_whitespace().take(5).collect::<Vec<_>>().join(" ")
}

/// Elapsed time for a running tool is advanced by the mock clock
/// (`tick_clock`), which stands in for omp's run source — never by
/// component-local timers or effects.
fn next_took(took: &str) -> String {
    let t: f32 = took.trim_end_matches('s').parse().unwrap_or(0.0);
    format!("{:.1}s", t + 0.1)
}

fn new_worker(prompt: &str, chips: &Chips, id: String) -> Agent {
    Agent {
        id,
        title: title_of(prompt),
        role: AgentRole::Worker,
        status: AgentStatus::Running,
        model: chips.model.clone(),
        thinking: chips.thinking.clone(),
        approval: chips.approval.clone(),
        ctx: 4,
        tps: 9,
        todo: None,
        streaming: true,
        archived: false,
        queue: vec![],
        timeline: vec![TimelineItem::User { text: prompt.into() }],
    }
}

fn new_orchestrator(name: &str, id: String) -> Agent {
    Agent {
        id,
        title: "orchestrator".into(),
        role: AgentRole::Orchestrator,
        status: AgentStatus::Idle,
        model: MODELS[1].into(),
        thinking: "high".into(),
        approval: "yolo".into(),
        ctx: 4,
        tps: 0,
        todo: None,
        streaming: false,
        archived: false,
        queue: vec![],
        timeline: vec![TimelineItem::Md {
            text: format!("Project **{name}** registered. I read, plan, and dispatch — workers own writes."),
        }],
    }
}

// The actions the prototype defines but its mock UI does not reach yet
// (rename, archive, delete, new conversation): kept for the state-machine
// contract the daemon/client mirrors.
#[allow(dead_code)]
impl Store {
    pub fn new() -> Self {
        let fleet = signal(initial_data());
        // The routing gate's memo (zustand's hasProjects selector): the
        // shell re-renders on a project count change only, not on every
        // fleet tick.
        let has_projects = {
            let fleet = fleet.clone();
            computed(move || !fleet.get().projects.is_empty())
        };
        let s = Store {
            fleet,
            sel: signal(Some(Sel::Agent { id: "a1".into() })),
            chips: signal(HashMap::new()),
            slug_preview: signal(rand_slug()),
            omp_missing: signal(false),
            theme_id: signal(DEFAULT_THEME),
            win: signal(Win::Normal),
            sidebar_collapsed: signal(false),
            hub_open: signal(false),
            palette_open: signal(false),
            palette_query: signal(String::new()),
            drafts: signal(HashMap::new()),
            folds: signal(HashMap::new()),
            popovers: signal(HashMap::new()),
            follow_tail: signal(HashMap::new()),
            has_projects,
            pulse: animated_value(1.0),
            ring_pulse: animated_value(1.0),
            rng: Arc::new(Mutex::new(())),
            ids: Arc::new(Mutex::new(0)),
        };
        // dot-pulse (index.css): 1 → 0.35 → 1 over 2.4s, ease-in-out, forever.
        s.pulse.keyframes(
            [Keyframe::new(1.0), Keyframe::at(0.5, 0.35), Keyframe::new(1.0)],
            KeyframeTiming::new(2400.0).easing(Easing::EaseInOut).infinite(),
        );
        // Tailwind's animate-pulse for the orchestrator ring: 1 → 0.5 → 1
        // over 2s, cubic-bezier(.4,0,.6,1) ≈ ease-in-out, forever.
        s.ring_pulse.keyframes(
            [Keyframe::new(1.0), Keyframe::at(0.5, 0.5), Keyframe::new(1.0)],
            KeyframeTiming::new(2000.0).easing(Easing::EaseInOut).infinite(),
        );
        s
    }

    fn next_id(&self, prefix: &str) -> String {
        let mut c = self.ids.lock().unwrap();
        *c += 1;
        format!("{prefix}-{c}")
    }

    // ---- ui actions ----

    pub fn select(&self, sel: Sel) {
        self.sel.set(Some(sel));
    }

    pub fn toggle_sidebar(&self) {
        self.sidebar_collapsed.update(|v| *v = !*v);
    }

    pub fn set_hub_open(&self, v: bool) {
        self.hub_open.set(v);
    }

    pub fn set_palette_open(&self, v: bool) {
        self.palette_open.set(v);
        if !v {
            self.palette_query.set(String::new());
        }
    }

    pub fn set_palette_query(&self, q: String) {
        self.palette_query.set(q);
    }

    pub fn set_draft(&self, key: &str, v: String) {
        self.drafts.update(|m| {
            m.insert(key.into(), v);
        });
    }

    pub fn draft(&self, key: &str) -> String {
        self.drafts.get().get(key).cloned().unwrap_or_default()
    }

    pub fn toggle_fold(&self, key: &str) {
        let k = key.to_string();
        self.folds.update(|m| {
            let e = m.entry(k).or_insert(false);
            *e = !*e;
        });
    }

    pub fn fold(&self, key: &str) -> bool {
        self.folds.get().get(key).copied().unwrap_or(false)
    }

    pub fn toggle_popover(&self, key: &str) {
        let k = key.to_string();
        self.popovers.update(|m| {
            let e = m.entry(k).or_insert(false);
            *e = !*e;
        });
    }

    pub fn popover(&self, key: &str) -> bool {
        self.popovers.get().get(key).copied().unwrap_or(false)
    }

    pub fn set_follow_tail(&self, agent_id: &str, v: bool) {
        self.follow_tail.update(|m| {
            m.insert(agent_id.into(), v);
        });
    }

    pub fn follow(&self, agent_id: &str) -> bool {
        self.follow_tail.get().get(agent_id).copied().unwrap_or(true)
    }

    // ---- chips / branch ----

    pub fn set_chip(&self, project_id: &str, field: &str, value: String) {
        self.chips.update(|m| {
            let cur = m.get(project_id).cloned().unwrap_or_else(default_chips);
            let mut next = cur;
            match field {
                "model" => next.model = value,
                "thinking" => next.thinking = value,
                _ => next.approval = value,
            }
            m.insert(project_id.into(), next);
        });
    }

    pub fn set_project_branch(&self, project_id: &str, branch: &str) {
        self.fleet.update(|f| {
            for p in &mut f.projects {
                if p.id == project_id {
                    p.branch = branch.into();
                }
            }
        });
    }

    // ---- domain transitions ----

    /// The project orchestrator dispatches: a workspace, its worktree, and
    /// its worker agent are created together (1:1).
    pub fn spawn_from_project(&self, project_id: &str, prompt: &str) {
        let chips = self.chips.get().get(project_id).cloned().unwrap_or_else(default_chips);
        let base = self.slug_preview.get();
        let agent_id = self.next_id("a");
        let workspace_id = self.next_id("w");
        self.fleet.update(|fleet| {
            if let Some(p) = fleet.projects.iter_mut().find(|p| p.id == project_id) {
                let mut slug = base.clone();
                let mut n = 2;
                while p.workspaces.iter().any(|w| w.name == slug) {
                    slug = format!("{base}-{n}");
                    n += 1;
                }
                let path = format!("{}-wt/{slug}", p.path);
                p.workspaces.push(Workspace {
                    id: workspace_id,
                    name: slug.clone(),
                    isolation: Isolation::Worktree,
                    branch: format!("fix/{slug}"),
                    path,
                    dirty: None,
                    untracked: None,
                    pr: None,
                    ci: None,
                    worker: new_worker(prompt, &chips, agent_id.clone()),
                });
            }
        });
        self.sel.set(Some(Sel::Agent { id: agent_id }));
        self.slug_preview.set(rand_slug());
        self.drafts.update(|m| {
            m.insert(format!("project:{project_id}"), String::new());
        });
    }

    /// DEC-014: Enter = prompt/steer, Alt+Enter = queue.
    pub fn send_to_agent(&self, agent_id: &str, text: &str, alt: bool) {
        self.fleet.update(|fleet| map_agent_mut(fleet, agent_id, |a| {
            if alt {
                a.queue.push(text.into());
            } else {
                a.streaming = true;
                a.status = AgentStatus::Running;
                a.timeline.push(TimelineItem::User { text: text.into() });
            }
        }));
        self.drafts.update(|m| {
            m.insert(format!("agent:{agent_id}"), String::new());
        });
        self.set_follow_tail(agent_id, true);
    }

    pub fn abort_agent(&self, agent_id: &str) {
        self.fleet.update(|fleet| map_agent_mut(fleet, agent_id, |a| {
            a.streaming = false;
            a.status = AgentStatus::Idle;
            a.tps = 0;
            a.timeline.push(TimelineItem::Md {
                text: "— previous run aborted — queued follow-ups preserved for the next turn".into(),
            });
        }));
    }

    pub fn promote_queued(&self, agent_id: &str, i: usize) {
        self.fleet.update(|fleet| map_agent_mut(fleet, agent_id, |a| {
            if i < a.queue.len() {
                let x = a.queue.remove(i);
                a.queue.insert(0, x);
            }
        }));
    }

    pub fn remove_queued(&self, agent_id: &str, i: usize) {
        self.fleet.update(|fleet| map_agent_mut(fleet, agent_id, |a| {
            if i < a.queue.len() {
                a.queue.remove(i);
            }
        }));
    }

    pub fn answer_ask(&self, agent_id: &str, index: usize, answer: &str) {
        self.fleet.update(|fleet| map_agent_mut(fleet, agent_id, |a| {
            a.status = AgentStatus::Running;
            a.streaming = true;
            if let Some(TimelineItem::Ask { resolved, .. }) = a.timeline.get_mut(index) {
                *resolved = Some(answer.into());
            }
        }));
    }

    pub fn new_conversation(&self, agent_id: &str) {
        self.fleet.update(|fleet| map_agent_mut(fleet, agent_id, |a| {
            a.timeline.push(TimelineItem::Md { text: "— new conversation —".into() });
        }));
    }

    pub fn rename_agent(&self, agent_id: &str, name: &str) {
        self.fleet.update(|fleet| map_agent_mut(fleet, agent_id, |a| {
            a.title = name.into();
        }));
    }

    /// Archive/delete apply to workers (and their workspace); the
    /// orchestrator persists.
    pub fn archive_agent(&self, agent_id: &str) {
        self.fleet.update(|fleet| {
            for p in &mut fleet.projects {
                for w in &mut p.workspaces {
                    if w.worker.id == agent_id {
                        w.worker.archived = true;
                        w.worker.streaming = false;
                        w.worker.status = AgentStatus::Idle;
                    }
                }
            }
        });
    }

    pub fn delete_agent(&self, agent_id: &str) {
        let mut clear_sel = false;
        self.fleet.update(|fleet| {
            for p in &mut fleet.projects {
                p.workspaces.retain(|w| w.worker.id != agent_id);
            }
        });
        if let Some(Sel::Agent { id }) = self.sel.get() {
            clear_sel = id == agent_id;
        }
        if clear_sel {
            self.sel.set(None);
        }
    }

    pub fn prune_worktrees(&self, project_id: &str) {
        self.fleet.update(|fleet| {
            for p in &mut fleet.projects {
                if p.id == project_id {
                    p.workspaces.retain(|w| !w.worker.archived);
                }
            }
        });
    }

    pub fn open_directory(&self, name: &str) {
        let pid = self.next_id("p");
        let orchestrator = new_orchestrator(name, self.next_id("o"));
        self.fleet.update(|fleet| {
            fleet.projects.push(Project {
                path: format!("~/{name}"),
                branch: "main".into(),
                id: pid.clone(),
                name: name.into(),
                orchestrator,
                workspaces: vec![],
            });
        });
        self.sel.set(Some(Sel::Project { id: pid }));
    }

    pub fn set_omp_missing(&self, v: bool) {
        self.omp_missing.set(v);
    }

    pub fn cycle_theme(&self) {
        let next = THEME_ORDER
            .iter()
            .position(|t| *t == self.theme_id.get())
            .map(|i| THEME_ORDER[(i + 1) % THEME_ORDER.len()])
            .unwrap_or(DEFAULT_THEME);
        self.theme_id.set(next);
    }

    pub fn set_win(&self, v: Win) {
        self.win.set(v);
    }

    /// 1s heartbeat: tps/ctx jitter (`tick`).
    pub fn tick(&self) {
        self.fleet.update(|fleet| {
            for p in &mut fleet.projects {
                if p.orchestrator.streaming {
                    p.orchestrator.tps = 8 + (rand_f64() * 8.0).round() as u32;
                }
                for w in &mut p.workspaces {
                    if w.worker.streaming {
                        w.worker.tps = 8 + (rand_f64() * 8.0).round() as u32;
                        if rand_f64() < 0.2 && w.worker.ctx < 99 {
                            w.worker.ctx += 1;
                        }
                    }
                }
            }
        });
    }

    /// 100ms clock: advances running tool elapsed time (`tickClock`).
    pub fn tick_clock(&self) {
        self.fleet.update(|fleet| {
            for p in &mut fleet.projects {
                for w in &mut p.workspaces {
                    if w.worker.streaming {
                        for it in &mut w.worker.timeline {
                            if let TimelineItem::Tool { status: ToolStatus::Running, took, .. } = it {
                                *took = next_took(took);
                            }
                        }
                    }
                }
            }
        });
    }
}

// ---- pure selectors (views use these; no view computes domain facts itself) ----

fn map_agent_mut(fleet: &mut Fleet, agent_id: &str, f: impl FnOnce(&mut Agent)) {
    for p in &mut fleet.projects {
        if p.orchestrator.id == agent_id {
            f(&mut p.orchestrator);
            return;
        }
        for w in &mut p.workspaces {
            if w.worker.id == agent_id {
                f(&mut w.worker);
                return;
            }
        }
    }
}

/// What `findAgentIn` returns: the project, the workspace (workers), the agent.
pub struct Found<'a> {
    pub project: &'a Project,
    pub workspace: Option<&'a Workspace>,
    pub agent: &'a Agent,
}

pub fn find_agent<'a>(fleet: &'a Fleet, agent_id: &str) -> Option<Found<'a>> {
    for p in &fleet.projects {
        if p.orchestrator.id == agent_id {
            return Some(Found { project: p, workspace: None, agent: &p.orchestrator });
        }
        for w in &p.workspaces {
            if w.worker.id == agent_id {
                return Some(Found { project: p, workspace: Some(w), agent: &w.worker });
            }
        }
    }
    None
}

pub fn find_workspace<'a>(fleet: &'a Fleet, workspace_id: &str) -> Option<(&'a Project, &'a Workspace)> {
    for p in &fleet.projects {
        if let Some(w) = p.workspaces.iter().find(|x| x.id == workspace_id) {
            return Some((p, w));
        }
    }
    None
}

pub fn find_project<'a>(fleet: &'a Fleet, project_id: &str) -> Option<&'a Project> {
    fleet.projects.iter().find(|x| x.id == project_id)
}

pub fn chips_for(store: &Store, project_id: &str) -> Chips {
    store.chips.get().get(project_id).cloned().unwrap_or_else(default_chips)
}

fn agent_timelines(fleet: &Fleet) -> impl Iterator<Item = &Agent> {
    fleet
        .projects
        .iter()
        .flat_map(|p| std::iter::once(&p.orchestrator).chain(p.workspaces.iter().map(|w| &w.worker)))
}

pub struct AskRef {
    pub agent_id: String,
    /// The ask's timeline index (part of the prototype's data shape; the
    /// jump action needs only the agent).
    #[allow(dead_code)]
    pub index: usize,
    pub q: String,
}

pub fn collect_asks(fleet: &Fleet) -> Vec<AskRef> {
    let mut out = Vec::new();
    for a in agent_timelines(fleet) {
        for (i, it) in a.timeline.iter().enumerate() {
            if let TimelineItem::Ask { q, resolved: None, .. } = it {
                out.push(AskRef { agent_id: a.id.clone(), index: i, q: q.clone() });
            }
        }
    }
    out
}

pub struct SubRef {
    pub agent_id: String,
    pub agent_title: String,
    /// The sub's timeline index (part of the prototype's data shape).
    #[allow(dead_code)]
    pub index: usize,
    pub sub: TimelineItem,
}

pub fn collect_subs(fleet: &Fleet) -> Vec<SubRef> {
    let mut out = Vec::new();
    for p in &fleet.projects {
        for w in &p.workspaces {
            for (i, it) in w.worker.timeline.iter().enumerate() {
                if matches!(it, TimelineItem::Sub { .. }) {
                    out.push(SubRef {
                        agent_id: w.worker.id.clone(),
                        agent_title: w.worker.title.clone(),
                        index: i,
                        sub: it.clone(),
                    });
                }
            }
        }
    }
    out
}
