import type { AgentStatus, Fleet } from './types'

// Mock fleet for the papago prototype.
// Model: Project -> persistent orchestrator (read/plan/dispatch only) + Workspaces,
// each 1:1 with a worktree and a single worker agent. Subagents are ephemeral
// timeline entries on the worker and surface only in the status bar.

const WORDS = ['swift', 'brave', 'quiet', 'amber', 'lunar', 'coral', 'misty', 'iron', 'sable', 'fern']
const ANIMALS = ['fox', 'otter', 'heron', 'lynx', 'marten', 'ibex', 'crane', 'vole', 'newt', 'drake']

export const randSlug = (): string =>
  `${WORDS[Math.floor(Math.random() * WORDS.length)]}-${ANIMALS[Math.floor(Math.random() * ANIMALS.length)]}`

let idc = 0
export const nextId = (p: string): string => `${p}-${++idc}`

export const MODELS = ['Claude Sonnet 5.5', 'Claude Opus 5.5', 'Claude Haiku 5.5', 'GPT-5.6 Luna', 'DeepSeek V4.1']
export const THINK = ['off', 'low', 'medium', 'high']
export const THINK_LABELS: Record<string, string> = { off: 'Off', low: 'Low', medium: 'Medium', high: 'High' }
export const APPROVALS = ['yolo', 'write-ask', 'always-ask']
export const APPROVAL_LABELS: Record<string, string> = { yolo: 'Full Access', 'write-ask': 'Write Ask', 'always-ask': 'Always Ask' }
export const BRANCHES = ['main', 'develop', 'release/1.0']

export const initialData = (): Fleet => ({
  projects: [
    {
      id: 'p1',
      name: 'myrepo',
      path: '~/code/myrepo',
      branch: 'main',
      orchestrator: {
        id: 'o1',
        title: 'orchestrator',
        role: 'orchestrator',
        status: 'idle',
        model: 'Claude Opus 5.5',
        thinking: 'high',
        approval: 'yolo',
        ctx: 9,
        tps: 0,
        todo: null,
        streaming: false,
        queue: [],
        timeline: [
          { k: 'user', text: 'ship the login fix and the rate-limit work' },
          { k: 'tool', verb: 'read', target: 'src/auth', status: 'done', took: '0.3s' },
          { k: 'tool', verb: 'bash', target: 'git worktree add ../swift-otter -b fix/login', status: 'done', took: '1.1s' },
          { k: 'tool', verb: 'bash', target: 'launch worker swift-otter', status: 'done', took: '0.2s' },
          { k: 'tool', verb: 'bash', target: 'git worktree add ../brave-otter -b fix/rate-limit', status: 'done', took: '0.9s' },
          { k: 'tool', verb: 'bash', target: 'launch worker brave-otter', status: 'done', took: '0.2s' },
          { k: 'md', text: '**Plan:** two workspaces created. Workers own write access; I only read, plan, and dispatch.' },
        ],
      },
      workspaces: [
        {
          id: 'w1',
          name: 'swift-otter',
          isolation: 'worktree',
          branch: 'fix/login',
          path: '~/code/myrepo-wt/swift-otter',
          dirty: '*1',
          untracked: '?4',
          pr: 181,
          ci: 'passed',
          worker: {
            id: 'a1',
            title: 'fix-login bug',
            role: 'worker',
            status: 'running',
            model: 'Claude Sonnet 5.5',
            thinking: 'medium',
            approval: 'yolo',
            ctx: 42,
            tps: 12,
            todo: [2, 5],
            streaming: true,
            queue: [],
            timeline: [
              { k: 'user', text: 'the login redirect loops when the session cookie expires mid-request' },
              { k: 'tool', verb: 'read', target: 'src/auth/session.ts', status: 'done', took: '0.4s' },
              {
                k: 'tool',
                verb: 'edit',
                target: 'src/auth/session.ts',
                status: 'done',
                took: '1.2s',
                diff: { add: 12, del: 3 },
                before: [
                  'if (cookie.expired) {',
                  "  return redirect('/login')",
                  '}',
                  'session = refresh(cookie)',
                ],
                after: [
                  'session = refresh(cookie)',
                  'if (cookie.expired) {',
                  "  return redirect('/login')",
                  '}',
                  'touch(lastSeen)',
                ],
              },
              { k: 'tool', verb: 'bash', target: 'go test ./auth', status: 'running', took: '3.1s' },
              { k: 'user', text: 'also cover the refresh-race path while you are in there' },
              { k: 'sub', title: 'write regression tests', status: 'running', model: 'Claude Haiku 5.5', tool: 'bash go test', tokens: '4.2k' },
              { k: 'sub', title: 'audit cookie flags', status: 'done', model: 'Claude Haiku 5.5', tool: 'grep cookie', tokens: '1.1k' },
              { k: 'sub', title: 'check header parsing', status: 'error', model: 'Claude Haiku 5.5', tool: 'bash curl', tokens: '0.8k' },
              { k: 'ask', kind: 'confirm', q: 'Restart the dev server to pick up the new session middleware?', resolved: 'Approve' },
              { k: 'ask', kind: 'select', q: 'Which store for the rate-limit counters?', options: ['in-memory', 'redis', 'sqlite'], resolved: 'redis' },
              {
                k: 'md',
                text: '**Root cause:** the cookie refresh raced the middleware. `refresh(cookie)` now runs before the guard, and the redirect path re-reads the session once.\n\nNext: regression test for the expiry path, then a quick audit of the cookie flags.',
              },
              { k: 'md', text: '— previous run aborted — queued follow-ups preserved for the next turn' },
            ],
          },
        },
        {
          id: 'w2',
          name: 'brave-otter',
          isolation: 'worktree',
          branch: 'fix/rate-limit',
          path: '~/code/myrepo-wt/brave-otter',
          dirty: '*2',
          pr: 182,
          ci: 'running',
          worker: {
            id: 'a6',
            title: 'add rate-limit',
            role: 'worker',
            status: 'needs-you',
            model: 'Claude Sonnet 5.5',
            thinking: 'medium',
            approval: 'yolo',
            ctx: 61,
            tps: 0,
            todo: [1, 4],
            streaming: false,
            queue: ['run the full suite', 'bump deps'],
            timeline: [
              { k: 'user', text: 'add per-IP rate limiting to the public API' },
              { k: 'tool', verb: 'bash', target: 'go test ./... -run Rate', status: 'done', took: '8.1s' },
              { k: 'sub', title: 'check header parsing', status: 'error', model: 'Claude Haiku 5.5', tool: 'bash curl', tokens: '0.8k' },
              { k: 'ask', kind: 'select', q: 'Which store for the rate-limit counters?', options: ['in-memory', 'redis', 'sqlite'], resolved: null },
            ],
          },
        },
        {
          id: 'w3',
          name: 'tidy-docs',
          isolation: 'worktree',
          branch: 'docs/v2',
          path: '~/code/myrepo-wt/tidy-docs',
          worker: {
            id: 'a7',
            title: 'docs pass',
            role: 'worker',
            status: 'done',
            model: 'Claude Haiku 5.5',
            thinking: 'off',
            approval: 'yolo',
            ctx: 18,
            tps: 0,
            todo: [3, 3],
            streaming: false,
            queue: [],
            timeline: [
              { k: 'user', text: 'refresh the API docs for v2 endpoints' },
              { k: 'tool', verb: 'read', target: 'docs/api-v2.md', status: 'done', took: '0.3s' },
              { k: 'tool', verb: 'write', target: 'docs/api-v2.md', status: 'done', took: '0.8s' },
              { k: 'md', text: 'Docs updated: 14 endpoints, 3 new examples.' },
            ],
          },
        },
        {
          id: 'w4',
          name: 'lint-clean',
          isolation: 'worktree',
          branch: 'chore/lint',
          path: '~/code/myrepo-wt/lint-clean',
          worker: {
            id: 'a8',
            title: 'lint cleanup',
            role: 'worker',
            status: 'error',
            model: 'DeepSeek V4.1',
            thinking: 'low',
            approval: 'always-ask',
            ctx: 27,
            tps: 0,
            todo: [0, 2],
            streaming: false,
            queue: [],
            timeline: [
              { k: 'user', text: 'clear the lint backlog on this branch' },
              { k: 'tool', verb: 'bash', target: 'eslint .', status: 'error', took: '2.2s' },
              { k: 'md', text: 'Failed: `eslint` not installed in this worktree (exit 127). Stopped.' },
            ],
          },
        },
        {
          id: 'w5',
          name: 'review-pr',
          isolation: 'worktree',
          branch: 'review/182',
          path: '~/code/myrepo-wt/review-pr',
          worker: {
            id: 'a10',
            title: 'review PR 182',
            role: 'worker',
            status: 'needs-you',
            model: 'Claude Opus 5.5',
            thinking: 'high',
            approval: 'yolo',
            ctx: 33,
            tps: 0,
            todo: null,
            streaming: false,
            queue: [],
            timeline: [
              { k: 'user', text: 'review PR 182 for the rate-limit change' },
              { k: 'tool', verb: 'bash', target: 'gh pr diff 182', status: 'done', took: '1.9s' },
              { k: 'ask', kind: 'input', q: 'Paste the failing CI log, or submit empty to fetch it from Actions.', resolved: null },
            ],
          },
        },
        {
          id: 'w6',
          name: 'triage-482',
          isolation: 'worktree',
          branch: 'triage/482',
          path: '~/code/myrepo-wt/triage-482',
          worker: {
            id: 'a9',
            title: 'triage issue #482',
            role: 'worker',
            status: 'idle',
            model: 'GPT-5.6 Luna',
            thinking: 'low',
            approval: 'write-ask',
            ctx: 21,
            tps: 0,
            todo: null,
            streaming: false,
            queue: [],
            timeline: [
              { k: 'user', text: 'triage issue #482 — the lint job fails on main' },
              { k: 'md', text: '— new conversation —' },
              { k: 'tool', verb: 'bash', target: 'npm run lint', status: 'error', took: '4.4s' },
              { k: 'md', text: 'Lint failed with 3 errors in `src/auth/session.ts`. Paused here; rerun after the fix lands.' },
            ],
          },
        },
      ],
    },
    {
      id: 'p2',
      name: 'papago-docs',
      path: '~/code/papago-docs',
      branch: 'main',
      orchestrator: {
        id: 'o2',
        title: 'orchestrator',
        role: 'orchestrator',
        status: 'idle',
        model: 'GPT-5.6 Luna',
        thinking: 'medium',
        approval: 'yolo',
        ctx: 5,
        tps: 0,
        todo: null,
        streaming: false,
        queue: [],
        timeline: [
          { k: 'user', text: 'regenerate the OpenAPI spec after the v2 rename' },
          { k: 'tool', verb: 'bash', target: 'launch worker regen-openapi', status: 'done', took: '0.2s' },
          { k: 'md', text: 'Handed off to worker `regen-openapi`.' },
        ],
      },
      workspaces: [
        {
          id: 'w7',
          name: 'regen-openapi',
          isolation: 'worktree',
          branch: 'docs/openapi',
          path: '~/code/papago-docs-wt/regen-openapi',
          pr: 179,
          ci: 'failed',
          worker: {
            id: 'a13',
            title: 'regenerate OpenAPI',
            role: 'worker',
            status: 'needs-you',
            model: 'Claude Sonnet 5.5',
            thinking: 'medium',
            approval: 'write-ask',
            ctx: 15,
            tps: 0,
            todo: null,
            streaming: false,
            queue: [],
            timeline: [
              { k: 'user', text: 'regenerate the OpenAPI spec after the v2 rename' },
              { k: 'tool', verb: 'bash', target: 'go run ./cmd/openapi', status: 'done', took: '2.7s' },
              { k: 'ask', kind: 'confirm', q: 'Overwrite docs/openapi.yaml with the regenerated spec?', resolved: null },
            ],
          },
        },
      ],
    },
  ],
})

export const STATUS_DOT: Record<AgentStatus, string> = {
  running: 'bg-work dot-pulse',
  'needs-you': 'bg-warn',
  done: 'bg-ok',
  idle: 'bg-mut',
  error: 'bg-err',
}

// Orchestrator marker: the same status colours in hollow-ring form, so an
// orchestrator row never reads as a sibling of a worker row at a glance.
export const STATUS_RING: Record<AgentStatus, string> = {
  running: 'border-work',
  'needs-you': 'border-warn',
  done: 'border-ok',
  idle: 'border-mut',
  error: 'border-err',
}

export const URGENCY: Record<AgentStatus, number> = {
  'needs-you': 0,
  running: 1,
  error: 1,
  done: 2,
  idle: 3,
}