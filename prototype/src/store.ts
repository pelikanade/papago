// The papago prototype state machine. Single source of truth: views are pure
// functions of this store and mutate it only through the named actions below.
// Model: Project -> persistent orchestrator + Workspaces (1:1 worktree + worker).
// Transitions stay explicit and serializable for the Go daemon/client mirror.

import { create } from 'zustand'
import type { Agent, Chips, Fleet, Sel, TimelineItem } from './types'
import { APPROVALS, initialData, MODELS, nextId, randSlug, THINK } from './data'
import { applyTheme, DEFAULT_THEME, THEME_ORDER } from './themes'

const DEFAULT_CHIPS: Chips = { model: MODELS[0], thinking: THINK[2], approval: APPROVALS[0] }

export interface PapagoState {
  // ---- domain ----
  fleet: Fleet
  sel: Sel | null
  chips: Record<string, Chips> // per-Project last-used values (defaults remember)
  slugPreview: string // random slug shown in the Project spawn status bar (display-only)
  ompMissing: boolean // diagnostics view switch
  themeId: string // base16 theme (default Nord Dark)
  win: 'normal' | 'maximized' | 'minimized' | 'closed'

  // ---- ui (kept in the machine so every view stays a pure function) ----
  sidebarCollapsed: boolean
  hubOpen: boolean
  paletteOpen: boolean
  paletteQuery: string
  drafts: Record<string, string> // composer/inline-input text per key
  folds: Record<string, boolean> // tool-card fold state, key `${agentId}:${index}`
  popovers: Record<string, boolean> // queue/background/tree/sub popovers
  followTail: Record<string, boolean> // per-Agent timeline tail-follow

  // ---- actions (explicit transitions) ----
  select: (sel: Sel) => void
  toggleSidebar: () => void
  setHubOpen: (v: boolean) => void
  setPaletteOpen: (v: boolean) => void
  setPaletteQuery: (q: string) => void
  setDraft: (key: string, v: string) => void
  toggleFold: (key: string) => void
  togglePopover: (key: string) => void
  setFollowTail: (agentId: string, v: boolean) => void
  setChip: (projectId: string, field: keyof Chips, value: string) => void
  setProjectBranch: (projectId: string, branch: string) => void

  spawnFromProject: (projectId: string, prompt: string) => void
  sendToAgent: (agentId: string, text: string, alt: boolean) => void
  abortAgent: (agentId: string) => void
  promoteQueued: (agentId: string, i: number) => void
  removeQueued: (agentId: string, i: number) => void
  answerAsk: (agentId: string, index: number, answer: string) => void
  newConversation: (agentId: string) => void
  renameAgent: (agentId: string, name: string) => void
  archiveAgent: (agentId: string) => void
  deleteAgent: (agentId: string) => void
  pruneWorktrees: (projectId: string) => void
  openDirectory: (name: string) => void
  setOmpMissing: (v: boolean) => void
  cycleTheme: () => void
  setWin: (v: 'normal' | 'maximized' | 'minimized' | 'closed') => void
  tick: () => void // 1s heartbeat: tps/ctx jitter
  tickClock: () => void // 100ms clock: advances running tool elapsed time
}

const findAgentIn = (fleet: Fleet, agentId: string) => {
  for (const p of fleet.projects) {
    if (p.orchestrator.id === agentId) return { p, w: undefined, a: p.orchestrator }
    for (const w of p.workspaces) if (w.worker.id === agentId) return { p, w, a: w.worker }
  }
  return null
}

const mapAgent = (fleet: Fleet, agentId: string, fn: (a: Agent) => Agent): Fleet => ({
  projects: fleet.projects.map((p) => ({
    ...p,
    orchestrator: p.orchestrator.id === agentId ? fn(p.orchestrator) : p.orchestrator,
    workspaces: p.workspaces.map((w) => (w.worker.id === agentId ? { ...w, worker: fn(w.worker) } : w)),
  })),
})

const titleOf = (prompt: string) => prompt.split(/\s+/).slice(0, 5).join(' ')

// Elapsed time for a running tool is advanced by the mock clock (tickClock), which
// stands in for omp's run source — never by component-local timers or effects.
const nextTook = (took: string): string => `${(Number.parseFloat(took) + 0.1).toFixed(1)}s`

const newWorker = (prompt: string, chips: Chips): Agent => ({
  id: nextId('a'),
  title: titleOf(prompt),
  role: 'worker',
  status: 'running',
  model: chips.model,
  thinking: chips.thinking,
  approval: chips.approval,
  ctx: 4,
  tps: 9,
  todo: null,
  streaming: true,
  queue: [],
  timeline: [{ k: 'user', text: prompt }],
})

const newOrchestrator = (name: string): Agent => ({
  id: nextId('o'),
  title: 'orchestrator',
  role: 'orchestrator',
  status: 'idle',
  model: MODELS[1],
  thinking: 'high',
  approval: 'yolo',
  ctx: 4,
  tps: 0,
  todo: null,
  streaming: false,
  queue: [],
  timeline: [{ k: 'md', text: `Project **${name}** registered. I read, plan, and dispatch — workers own writes.` }],
})

export const usePapago = create<PapagoState>((set, get) => ({
  fleet: initialData(),
  sel: { t: 'agent', id: 'a1' },
  chips: {},
  slugPreview: randSlug(),
  ompMissing: false,
  themeId: DEFAULT_THEME,
  win: 'normal',

  sidebarCollapsed: false,
  hubOpen: false,
  paletteOpen: false,
  paletteQuery: '',
  drafts: {},
  folds: {},
  popovers: {},
  followTail: {},

  select: (sel) => set({ sel }),
  toggleSidebar: () => set((s) => ({ sidebarCollapsed: !s.sidebarCollapsed })),
  setHubOpen: (v) => set({ hubOpen: v }),
  setPaletteOpen: (v) => set({ paletteOpen: v, paletteQuery: v ? get().paletteQuery : '' }),
  setPaletteQuery: (q) => set({ paletteQuery: q }),
  setDraft: (key, v) => set((s) => ({ drafts: { ...s.drafts, [key]: v } })),
  toggleFold: (key) => set((s) => ({ folds: { ...s.folds, [key]: !s.folds[key] } })),
  togglePopover: (key) => set((s) => ({ popovers: { ...s.popovers, [key]: !s.popovers[key] } })),
  setFollowTail: (agentId, v) => set((s) => ({ followTail: { ...s.followTail, [agentId]: v } })),
  setChip: (projectId, field, value) =>
    set((s) => {
      const cur = s.chips[projectId] ?? DEFAULT_CHIPS
      return { chips: { ...s.chips, [projectId]: { ...cur, [field]: value } } }
    }),
  setProjectBranch: (projectId, branch) =>
    set((s) => ({ fleet: { projects: s.fleet.projects.map((p) => (p.id === projectId ? { ...p, branch } : p)) } })),

  // The project orchestrator dispatches: a workspace, its worktree, and its worker
  // agent are created together (1:1).
  spawnFromProject: (projectId, prompt) =>
    set((s) => {
      const chips = s.chips[projectId] ?? DEFAULT_CHIPS
      const p = s.fleet.projects.find((x) => x.id === projectId)
      if (!p) return {}
      const base = s.slugPreview
      let slug = base
      let n = 2
      while (p.workspaces.some((w) => w.name === slug)) slug = `${base}-${n++}`
      const agent = newWorker(prompt, chips)
      return {
        fleet: {
          projects: s.fleet.projects.map((x) =>
            x.id === projectId
              ? {
                  ...x,
                  workspaces: [
                    ...x.workspaces,
                    {
                      id: nextId('w'),
                      name: slug,
                      isolation: 'worktree',
                      branch: `fix/${slug}`,
                      path: `${x.path}-wt/${slug}`,
                      worker: agent,
                    },
                  ],
                }
              : x,
          ),
        },
        sel: { t: 'agent', id: agent.id },
        slugPreview: randSlug(),
        drafts: { ...s.drafts, [`project:${projectId}`]: '' },
      }
    }),

  // DEC-014: Enter = prompt/steer, Alt+Enter = queue.
  sendToAgent: (agentId, text, alt) =>
    set((s) => ({
      fleet: mapAgent(s.fleet, agentId, (a) => {
        if (alt) return { ...a, queue: [...a.queue, text] }
        const item: TimelineItem = { k: 'user', text }
        return { ...a, streaming: true, status: 'running', timeline: [...a.timeline, item] }
      }),
      drafts: { ...s.drafts, [`agent:${agentId}`]: '' },
      followTail: { ...s.followTail, [agentId]: true },
    })),

  abortAgent: (agentId) =>
    set((s) => ({
      fleet: mapAgent(s.fleet, agentId, (a) => ({
        ...a,
        streaming: false,
        status: 'idle',
        tps: 0,
        timeline: [...a.timeline, { k: 'md', text: '— previous run aborted — queued follow-ups preserved for the next turn' }],
      })),
    })),

  promoteQueued: (agentId, i) =>
    set((s) => ({
      fleet: mapAgent(s.fleet, agentId, (a) => {
        const q = [...a.queue]
        const [x] = q.splice(i, 1)
        q.unshift(x)
        return { ...a, queue: q }
      }),
    })),

  removeQueued: (agentId, i) =>
    set((s) => ({ fleet: mapAgent(s.fleet, agentId, (a) => ({ ...a, queue: a.queue.filter((_, j) => j !== i) })) })),

  answerAsk: (agentId, index, answer) =>
    set((s) => ({
      fleet: mapAgent(s.fleet, agentId, (a) => ({
        ...a,
        status: 'running',
        streaming: true,
        timeline: a.timeline.map((it, j) => (j === index && it.k === 'ask' ? { ...it, resolved: answer } : it)),
      })),
    })),

  newConversation: (agentId) =>
    set((s) => ({
      fleet: mapAgent(s.fleet, agentId, (a) => ({ ...a, timeline: [...a.timeline, { k: 'md', text: '— new conversation —' }] })),
    })),

  renameAgent: (agentId, name) => set((s) => ({ fleet: mapAgent(s.fleet, agentId, (a) => ({ ...a, title: name })) })),

  // Archive/delete apply to workers (and their workspace); the orchestrator persists.
  archiveAgent: (agentId) =>
    set((s) => ({
      fleet: {
        projects: s.fleet.projects.map((p) => ({
          ...p,
          workspaces: p.workspaces.map((w) =>
            w.worker.id === agentId ? { ...w, worker: { ...w.worker, archived: true, streaming: false, status: 'idle' } } : w,
          ),
        })),
      },
    })),

  deleteAgent: (agentId) =>
    set((s) => ({
      fleet: {
        projects: s.fleet.projects.map((p) => ({ ...p, workspaces: p.workspaces.filter((w) => w.worker.id !== agentId) })),
      },
      sel: s.sel?.t === 'agent' && s.sel.id === agentId ? null : s.sel,
    })),

  pruneWorktrees: (projectId) =>
    set((s) => ({
      fleet: {
        projects: s.fleet.projects.map((p) =>
          p.id === projectId ? { ...p, workspaces: p.workspaces.filter((w) => !w.worker.archived) } : p,
        ),
      },
    })),

  openDirectory: (name) =>
    set((s) => {
      const pid = nextId('p')
      return {
        fleet: {
          projects: [
            ...s.fleet.projects,
            {
              id: pid,
              name,
              path: `~/${name}`,
              branch: 'main',
              orchestrator: newOrchestrator(name),
              workspaces: [],
            },
          ],
        },
        sel: { t: 'project', id: pid },
      }
    }),

  setOmpMissing: (v) => set({ ompMissing: v }),

  cycleTheme: () => {
    const next = THEME_ORDER[(THEME_ORDER.indexOf(get().themeId) + 1) % THEME_ORDER.length]
    set({ themeId: next })
    applyTheme(next)
  },

  setWin: (v) => set({ win: v }),

  tick: () =>
    set((s) => ({
      fleet: {
        projects: s.fleet.projects.map((p) => ({
          ...p,
          orchestrator: p.orchestrator.streaming
            ? { ...p.orchestrator, tps: 8 + Math.round(Math.random() * 8) }
            : p.orchestrator,
          workspaces: p.workspaces.map((w) => ({
            ...w,
            worker: w.worker.streaming
              ? { ...w.worker, tps: 8 + Math.round(Math.random() * 8), ctx: Math.min(99, w.worker.ctx + (Math.random() < 0.2 ? 1 : 0)) }
              : w.worker,
          })),
        })),
      },
    })),

  tickClock: () =>
    set((s) => ({
      fleet: {
        projects: s.fleet.projects.map((p) => ({
          ...p,
          workspaces: p.workspaces.map((w) => ({
            ...w,
            worker: w.worker.streaming
              ? {
                  ...w.worker,
                  timeline: w.worker.timeline.map((it) =>
                    it.k === 'tool' && it.status === 'running' ? { ...it, took: nextTook(it.took) } : it,
                  ),
                }
              : w.worker,
          })),
        })),
      },
    })),
}))

// ---- pure selectors (views use these; no view computes domain facts itself) ----

export const findWorkspace = (s: PapagoState, workspaceId: string) => {
  for (const p of s.fleet.projects) {
    const w = p.workspaces.find((x) => x.id === workspaceId)
    if (w) return { p, w }
  }
  return null
}

export const findAgent = (s: PapagoState, agentId: string) => findAgentIn(s.fleet, agentId)

export const chipsFor = (s: PapagoState, projectId: string): Chips => s.chips[projectId] ?? DEFAULT_CHIPS

const agentTimelines = (fleet: Fleet) =>
  fleet.projects.flatMap((p) => [p.orchestrator, ...p.workspaces.map((w) => w.worker)])

export const collectAsks = (fleet: Fleet) => {
  const out: { agentId: string; index: number; q: string }[] = []
  for (const a of agentTimelines(fleet))
    a.timeline.forEach((it, i) => {
      if (it.k === 'ask' && !it.resolved) out.push({ agentId: a.id, index: i, q: it.q })
    })
  return out
}

export const collectSubs = (fleet: Fleet) => {
  const out: { agentId: string; agentTitle: string; index: number; sub: Extract<TimelineItem, { k: 'sub' }> }[] = []
  for (const p of fleet.projects)
    for (const w of p.workspaces)
      w.worker.timeline.forEach((it, i) => {
        if (it.k === 'sub') out.push({ agentId: w.worker.id, agentTitle: w.worker.title, index: i, sub: it })
      })
  return out
}