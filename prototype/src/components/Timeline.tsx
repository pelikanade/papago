import { useRef } from 'react'
import { Icon } from './primitives'
import { I } from '../icons'
import { usePapago } from '../store'
import type { Agent, TimelineItem } from '../types'

// Design decision (Synth, 2026-10-27): every timeline item shares ONE visual
// grammar — a borderless row: status icon · type label (Read / Edit / Bash /
// Write / Subagent / Ask) · muted detail · right-aligned meta.
// Expandable items (edit/write/bash) swap the status icon for a caret on hover;
// the row itself never has a background or border — only the expanded body does.
// All rows go through <Row>; item kinds only supply label/detail/body.

export default function Timeline({ agent }: { agent: Agent }) {
  const follow = usePapago((s) => s.followTail[agent.id] ?? true)
  const setFollowTail = usePapago((s) => s.setFollowTail)
  const scrollRef = useRef<HTMLDivElement | null>(null)
  const observerRef = useRef<MutationObserver | null>(null)

  // Tail-follow without effects (useEffect is banned): a MutationObserver on
  // the scroller keeps the view pinned to the bottom while followTail is true.
  const attachScroller = (el: HTMLDivElement | null) => {
    observerRef.current?.disconnect()
    observerRef.current = null
    scrollRef.current = el
    if (!el) return
    el.scrollTop = el.scrollHeight
    const mo = new MutationObserver(() => {
      if (usePapago.getState().followTail[agent.id] ?? true) el.scrollTop = el.scrollHeight
    })
    mo.observe(el, { childList: true, subtree: true, characterData: true })
    observerRef.current = mo
  }

  return (
    <div className="relative flex-1 min-h-0">
      <div
        ref={attachScroller}
        onScroll={() => {
          const el = scrollRef.current
          if (!el) return
          setFollowTail(agent.id, el.scrollHeight - el.scrollTop - el.clientHeight < 24)
        }}
        className="h-full overflow-y-auto px-4 py-3 space-y-1.5"
      >
        {agent.timeline.map((it, i) => (
          <Item key={i} agentId={agent.id} index={i} it={it} />
        ))}
        <div className="h-1" />
      </div>
      {!follow && (
        <button
          onClick={() => {
            setFollowTail(agent.id, true)
            if (scrollRef.current) scrollRef.current.scrollTop = scrollRef.current.scrollHeight
          }}
          className="absolute bottom-3 right-4 px-2 h-6 rounded-full bg-surface border border-line text-xs text-mut hover:text-ink"
        >
          <Icon g={I.arrowDown} /> tail
        </button>
      )}
    </div>
  )
}

const Item = ({ agentId, index, it }: { agentId: string; index: number; it: TimelineItem }) => {
  if (it.k === 'user')
    return (
      <div className="flex justify-end">
        <div className="max-w-[70%] rounded-lg bg-glass-1 backdrop-blur-md border border-white/10 px-3 py-2 text-sm text-ink">
          {it.text}
        </div>
      </div>
    )
  if (it.k === 'md') return <div className="text-sm leading-relaxed text-ink/90 whitespace-pre-wrap">{renderMd(it.text)}</div>
  if (it.k === 'tool') return <ToolLine agentId={agentId} index={index} it={it} />
  if (it.k === 'sub') return <SubLine it={it} />
  if (it.k === 'ask') return <AskLine it={it} />
  return null
}

// Shared timeline row shell (see the design decision above): icon · label ·
// detail · meta; hover swaps the status icon for a caret when expandable.
const Row = ({
  glyph,
  tone,
  label,
  detail,
  meta,
  expandable = false,
  open = false,
  onToggle,
  children,
}: {
  glyph: string
  tone: string
  label: React.ReactNode
  detail?: React.ReactNode
  meta?: React.ReactNode
  expandable?: boolean
  open?: boolean
  onToggle?: () => void
  children?: React.ReactNode
}) => (
  <div>
    <button
      onClick={expandable ? onToggle : undefined}
      className={`group w-full flex items-center gap-2 px-1 py-0.5 text-left ${expandable ? 'cursor-pointer' : 'cursor-default'}`}
    >
      <span className="w-4 h-4 flex items-center justify-center shrink-0">
        {expandable && open ? (
          <span className="nf text-sm text-mut/70">
            <Icon g={I.caretDown} />
          </span>
        ) : (
          <>
            <span className={`nf text-sm ${tone} ${expandable ? 'group-hover:hidden' : ''}`}>{glyph}</span>
            {expandable && (
              <span className="nf text-sm text-mut/70 hidden group-hover:inline">
                <Icon g={I.caretRight} />
              </span>
            )}
          </>
        )}
      </span>
      <span className="text-sm text-ink">{label}</span>
      {detail}
      {meta && <span className="ml-auto flex items-center gap-2 text-xs">{meta}</span>}
    </button>
    {expandable && open && children}
  </div>
)

const statusGlyph = (status: 'done' | 'running' | 'error') => (status === 'done' ? I.check : status === 'error' ? I.x : I.spinner)
const statusTone = (status: 'done' | 'running' | 'error') =>
  status === 'done' ? 'text-ok' : status === 'error' ? 'text-err' : 'text-acc'
const asStatus = (s: string): 'done' | 'running' | 'error' => (s === 'error' ? 'error' : s === 'done' ? 'done' : 'running')
const typeLabel = (verb: string) => verb.charAt(0).toUpperCase() + verb.slice(1)

const ToolLine = ({ agentId, index, it }: { agentId: string; index: number; it: Extract<TimelineItem, { k: 'tool' }> }) => {
  const foldKey = `${agentId}:${index}`
  const expandable = it.verb === 'bash' || !!it.before || !!it.after
  const open = usePapago((s) => s.folds[foldKey] ?? false)
  const toggleFold = usePapago((s) => s.toggleFold)

  // Synth-confirmed (2026-10-27): the running-time number MUST move at 0.1s
  // resolution — the stopwatch feel is the intended design, not jitter. Elapsed time
  // advances from the run source (mock: 100ms store clock, tickClock); no local timers.

  return (
    <Row
      glyph={statusGlyph(it.status)}
      tone={statusTone(it.status)}
      label={typeLabel(it.verb)}
      detail={<span className="text-sm text-mut font-mono truncate">{it.target}</span>}
      meta={
        <>
          {it.diff && (
            <span className="text-mut/70">
              <span className="text-ok">+{it.diff.add}</span> <span className="text-err">−{it.diff.del}</span>
            </span>
          )}
          <span className="text-mut/60 tabular-nums">{it.took}</span>
        </>
      }
      expandable={expandable}
      open={open}
      onToggle={() => toggleFold(foldKey)}
    >
      <div className="mt-1 ml-6 rounded-md border border-white/10 bg-glass-1 backdrop-blur-sm px-3 py-2">
        {it.before || it.after ? (
          <SplitDiff before={it.before ?? []} after={it.after ?? []} />
        ) : (
          <pre className="text-xs font-mono text-mut leading-relaxed whitespace-pre-wrap">
            {it.verb === 'bash' ? '$ go test ./... -run Rate\nok  api/rate  8.1s\nok  api/auth  0.9s' : '…output preview (tail)…'}
          </pre>
        )}
      </div>
    </Row>
  )
}

const SubLine = ({ it }: { it: Extract<TimelineItem, { k: 'sub' }> }) => (
  <Row
    glyph={statusGlyph(asStatus(it.status))}
    tone={statusTone(asStatus(it.status))}
    label="Subagent"
    detail={<span className="text-sm text-mut truncate">{it.title}</span>}
    meta={<span className="text-mut/70 truncate">{it.model} · {it.tool} · {it.tokens}</span>}
  />
)

const AskLine = ({ it }: { it: Extract<TimelineItem, { k: 'ask' }> }) => {
  const done = !!it.resolved
  return (
    <Row
      glyph={done ? I.check : '?'}
      tone={done ? 'text-ok' : 'text-warn'}
      label="Ask"
      detail={<span className={`text-sm truncate ${done ? 'text-mut line-through decoration-mut/40' : 'text-mut'}`}>{it.q}</span>}
      meta={<span className={done ? 'text-ink' : 'text-warn/80'}>{done ? it.resolved : 'answer below'}</span>}
    />
  )
}

// side-by-side diff: old lines left, new lines right; compact mono at text-xs
const SplitDiff = ({ before, after }: { before: string[]; after: string[] }) => (
  <div className="grid grid-cols-2 gap-3 text-xs font-mono leading-relaxed">
    <div className="min-w-0">
      <div className="text-xs text-mut/60 mb-1">before</div>
      {before.map((l, i) => (
        <div key={i} className="text-err/90 whitespace-pre overflow-hidden text-ellipsis">
          <span className="text-mut/40 mr-2 select-none">−</span>
          {l}
        </div>
      ))}
    </div>
    <div className="min-w-0">
      <div className="text-xs text-mut/60 mb-1">after</div>
      {after.map((l, i) => (
        <div key={i} className="text-ok/90 whitespace-pre overflow-hidden text-ellipsis">
          <span className="text-mut/40 mr-2 select-none">+</span>
          {l}
        </div>
      ))}
    </div>
  </div>
)

const renderMd = (text: string) =>
  text.split(/(`[^`]+`|\*\*[^*]+\*\*)/g).map((part, i) => {
    if (part.startsWith('`'))
      return (
        <code key={i} className="font-mono text-xs bg-surface border border-line/60 rounded px-1 text-acc/90">
          {part.slice(1, -1)}
        </code>
      )
    if (part.startsWith('**')) return <strong key={i} className="font-semibold">{part.slice(2, -2)}</strong>
    return <span key={i}>{part}</span>
  })