// Domain types for the papago prototype — mirrors GLOSSARY.md.

export type AgentStatus = 'running' | 'needs-you' | 'done' | 'idle' | 'error'
export type Isolation = 'worktree' | 'local'
export type AgentRole = 'orchestrator' | 'worker'

export interface TimelineUser {
  k: 'user'
  text: string
}
export interface TimelineMd {
  k: 'md'
  text: string
}
export interface TimelineTool {
  k: 'tool'
  verb: string
  target: string
  status: 'done' | 'running' | 'error'
  took: string
  diff?: { add: number; del: number }
  before?: string[]
  after?: string[]
}
// Ephemeral subagents: they live only inside the worker's timeline and surface in
// the status bar as background tasks — never as sidebar rows.
export interface TimelineSub {
  k: 'sub'
  title: string
  status: AgentStatus
  model: string
  tool: string
  tokens: string
}
export interface TimelineAsk {
  k: 'ask'
  kind: 'select' | 'confirm' | 'input'
  q: string
  options?: string[]
  resolved: string | null
}
export type TimelineItem = TimelineUser | TimelineMd | TimelineTool | TimelineSub | TimelineAsk

export interface Agent {
  id: string
  title: string
  role: AgentRole
  status: AgentStatus
  model: string
  thinking: string
  approval: string
  ctx: number
  tps: number
  todo: [number, number] | null
  streaming: boolean
  archived?: boolean
  queue: string[]
  timeline: TimelineItem[]
}

// One workspace = one worktree = one worker agent (1:1, created by the
// project-level orchestrator).
export interface Workspace {
  id: string
  name: string
  isolation: Isolation
  branch: string
  path: string
  dirty?: string
  untracked?: string
  pr?: number
  ci?: 'passed' | 'failed' | 'running'
  worker: Agent
}

export interface Project {
  id: string
  name: string
  path: string
  branch: string
  // Persistent project-level agent: read + plan + create workspaces/workers only.
  orchestrator: Agent
  workspaces: Workspace[]
}

export interface Fleet {
  projects: Project[]
}

export type Sel =
  | { t: 'project'; id: string }
  | { t: 'workspace'; id: string }
  | { t: 'agent'; id: string }

export interface Chips {
  model: string
  thinking: string
  approval: string
}