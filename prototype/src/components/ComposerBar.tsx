import { Dot, Icon, MenuItem, Seg, StatusMenu } from './primitives'
import { I } from '../icons'
import { chipsFor, usePapago } from '../store'
import { APPROVAL_LABELS, APPROVALS, BRANCHES, MODELS, THINK, THINK_LABELS } from '../data'
import type { TimelineItem } from '../types'
import { AskForm } from './Ask'

// DEC-015 Composer Bar: clean text box (no chrome inside) above the pane's
// bottom-edge status bar — a full-bleed footer. Tern-style flat segments.
// Every interactive segment is a <StatusMenu> (shared trigger+popover shell);
// vitals are plain <Seg>. Icons are Nerd Font glyphs.
// DEC-014: Enter = send/steer, Alt+Enter = queue, Shift+Enter = newline.
// Pure view: text/drafts live in the store (AGENTS.md).

interface Props {
  variant: 'project' | 'agent'
  projectId: string
  workspaceId?: string
  agentId?: string
}

export default function ComposerBar({ variant, projectId, workspaceId, agentId }: Props) {
  const draftKey = variant === 'project' ? `project:${projectId}` : `agent:${agentId}`
  const text = usePapago((s) => s.drafts[draftKey] ?? '')
  const setDraft = usePapago((s) => s.setDraft)
  const chips = usePapago((s) => chipsFor(s, projectId))
  const setChip = usePapago((s) => s.setChip)
  const setProjectBranch = usePapago((s) => s.setProjectBranch)
  const slugPreview = usePapago((s) => s.slugPreview)
  const project = usePapago((s) => s.fleet.projects.find((p) => p.id === projectId))
  const workspace = usePapago((s) =>
    s.fleet.projects.flatMap((p) => p.workspaces).find((w) => w.id === workspaceId),
  )
  const agent = usePapago((s) =>
    s.fleet.projects
      .flatMap((p) => [p.orchestrator, ...p.workspaces.map((w) => w.worker)])
      .find((a) => a.id === agentId),
  )
  const spawnFromProject = usePapago((s) => s.spawnFromProject)
  const sendToAgent = usePapago((s) => s.sendToAgent)
  const abortAgent = usePapago((s) => s.abortAgent)
  const promoteQueued = usePapago((s) => s.promoteQueued)
  const removeQueued = usePapago((s) => s.removeQueued)

  const streaming = variant === 'agent' && !!agent?.streaming
  const queue = variant === 'agent' ? agent?.queue ?? [] : []
  const pendingAsk = variant === 'agent' && agent ? agent.timeline.findIndex((it) => it.k === 'ask' && !it.resolved) : -1
  // Background tasks: the worker's ephemeral subagents, surfaced at status-bar level.
  const subs = variant === 'agent' && agent ? agent.timeline.filter((it): it is Extract<TimelineItem, { k: 'sub' }> => it.k === 'sub') : []
  const subsRunning = subs.filter((s) => s.status === 'running').length
  const hasText = text.trim().length > 0
  const send = (alt: boolean) => {
    const t = text.trim()
    if (!t) return
    if (variant === 'project') spawnFromProject(projectId, t)
    else if (agentId) sendToAgent(agentId, t, alt)
  }

  const placeholder = variant === 'project' ? 'New Agent in a fresh worktree…' : 'What are we cooking?'

  return (
    <div className="shrink-0">
      {/* writing surface: clean box — or the pending Ask, which replaces it */}
      <div className="px-3 pt-1.5 pb-1.5">
        {pendingAsk >= 0 && agent ? (
          <AskForm agentId={agent.id} index={pendingAsk} it={agent.timeline[pendingAsk] as Extract<TimelineItem, { k: 'ask' }>} />
        ) : (
          <div className="rounded-lg border border-white/10 bg-glass-1 backdrop-blur-md focus-within:border-acc/60 transition-colors">
            <textarea
              rows={1}
              value={text}
              onChange={(e) => setDraft(draftKey, e.target.value)}
              onKeyDown={(e) => {
                if (e.key === 'Enter' && !e.shiftKey) {
                  e.preventDefault()
                  send(e.altKey)
                }
              }}
              placeholder={placeholder}
              className="w-full bg-transparent resize-none outline-none px-3.5 py-2 text-sm text-ink placeholder:text-mut/70 leading-relaxed max-h-40 overflow-y-auto field-sizing-content"
            />
          </div>
        )}
      </div>

      {/* pane bottom edge: full-bleed status bar, aligned with the sidebar footer */}
      <div className="h-7 bg-glass-2 backdrop-blur-xl border-t border-white/10 flex items-stretch">
        <StatusMenu
          id={`chip:model:${projectId}`}
          icon={I.model}
          iconClassName="text-acc/80"
          label={chips.model}
          title="model (per-Project last-used)"
          width="18rem"
        >
          {(close) =>
            MODELS.map((m) => (
              <MenuItem
                key={m}
                active={chips.model === m}
                onClick={() => {
                  setChip(projectId, 'model', m)
                  close()
                }}
              >
                {m}
              </MenuItem>
            ))
          }
        </StatusMenu>

        <StatusMenu id={`chip:th:${projectId}`} icon={I.filter} label={THINK_LABELS[chips.thinking]} title="thinking level" width="12rem">
          {(close) =>
            THINK.map((t) => (
              <MenuItem
                key={t}
                active={chips.thinking === t}
                onClick={() => {
                  setChip(projectId, 'thinking', t)
                  close()
                }}
              >
                {THINK_LABELS[t]}
              </MenuItem>
            ))
          }
        </StatusMenu>

        {agent?.role === 'orchestrator' ? (
          <Seg tone="muted" title="orchestrator: read + plan + dispatch only — workers own writes">
            <Icon g={I.shieldSlash} /> Plan · read-only
          </Seg>
        ) : (
          <StatusMenu
            id={`chip:approval:${projectId}`}
            icon={I.shieldSlash}
            label={APPROVAL_LABELS[chips.approval]}
            title="approval mode (ADR-0004)"
            width="14rem"
          >
            {(close) =>
              APPROVALS.map((a) => (
                <MenuItem
                  key={a}
                  active={chips.approval === a}
                  onClick={() => {
                    setChip(projectId, 'approval', a)
                    close()
                  }}
                >
                  {APPROVAL_LABELS[a]}
                </MenuItem>
              ))
            }
          </StatusMenu>
        )}

        {variant === 'project' ? (
          <>
            <StatusMenu id={`base:${projectId}`} icon={I.fork} label={`base: ${project?.branch}`} title="base branch" width="11rem">
              {(close) =>
                BRANCHES.map((b) => (
                  <MenuItem
                    key={b}
                    active={project?.branch === b}
                    onClick={() => {
                      setProjectBranch(projectId, b)
                      close()
                    }}
                  >
                    {b}
                  </MenuItem>
                ))
              }
            </StatusMenu>
            <Seg tone="muted" title="random slug, display-only (DEC-002)">
              slug: {slugPreview}
            </Seg>
          </>
        ) : workspace ? (
          <>
            <Seg tone="muted">
              <Icon g={I.folder} /> {workspace.name}
            </Seg>
            <Seg tone="warn" title={workspace.path}>
              <Icon g={I.fork} /> {workspace.branch}
              {workspace.dirty && <span className="text-warn">{workspace.dirty}</span>}
              {workspace.untracked && <span className="text-acc">{workspace.untracked}</span>}
            </Seg>
          </>
        ) : (
          <Seg tone="muted" title={project?.path}>
            <Icon g={I.folder} /> {project?.name}
          </Seg>
        )}

        {variant === 'agent' && agent && (
          <>
            <Seg tone="muted" title="context usage">
              <span className="w-8 h-2 rounded-sm bg-line overflow-hidden inline-block">
                <span className="block h-full bg-acc" style={{ width: `${agent.ctx}%` }} />
              </span>
              {agent.ctx}%
            </Seg>
            {agent.todo && (
              <Seg tone="muted" title="todo phases">
                todo {agent.todo[0]}/{agent.todo[1]}
              </Seg>
            )}
            {agent.streaming && <Seg tone="muted">{agent.tps} t/s</Seg>}
          </>
        )}

        {variant === 'agent' && queue.length > 0 && (
          <StatusMenu id={`queue:${agentId}`} icon={I.hourglass} label={queue.length} title="queued follow-ups (Alt+Enter)" width="20rem">
            {() => (
              <div className="space-y-0.5">
                {queue.map((q, i) => (
                  <div key={q} className="flex items-center gap-1.5 px-1.5 py-1 rounded hover:bg-white/[0.07] text-xs text-ink">
                    <span className="truncate flex-1">{q}</span>
                    <button className="text-mut hover:text-acc" title="promote" onClick={() => agentId && promoteQueued(agentId, i)}>
                      <Icon g={I.arrowUp} />
                    </button>
                    <button className="text-mut hover:text-err" title="remove" onClick={() => agentId && removeQueued(agentId, i)}>
                      <Icon g={I.x} />
                    </button>
                  </div>
                ))}
              </div>
            )}
          </StatusMenu>
        )}

        {variant === 'agent' && subs.length > 0 && (
          <StatusMenu
            id={`bg:${agentId}`}
            icon={I.tasks}
            label={`${subsRunning}/${subs.length}`}
            title="background tasks (ephemeral subagents)"
            width="20rem"
          >
            {() => (
              <div className="space-y-0.5">
                {subs.map((s, i) => (
                  <div key={i} className="flex items-center gap-1.5 px-1.5 py-1 rounded text-xs text-ink">
                    <Dot status={s.status} />
                    <span className="truncate flex-1">{s.title}</span>
                    <span className="text-mut/60">{s.status}</span>
                  </div>
                ))}
              </div>
            )}
          </StatusMenu>
        )}

        <div className="flex-1" />

        {streaming && (
          <Seg tone="err" onClick={() => agentId && abortAgent(agentId)} title="abort (immediate, no confirm)">
            <Icon g={I.stop} />
          </Seg>
        )}
        <Seg onClick={() => send(false)} tone={hasText ? 'accent' : 'muted'} title="Send · Enter steers while streaming · Alt+Enter queues">
          <span className={hasText ? 'text-acc' : 'text-mut/40'}>
            <Icon g={I.send} />
          </span>
        </Seg>
      </div>
    </div>
  )
}
