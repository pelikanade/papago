import { useMemo } from 'react'
import { Dot, Icon } from './primitives'
import { I } from '../icons'
import { collectSubs, usePapago } from '../store'

// Agent Hub panel window (floating): cross-Agent grouped live subagents.
// Read-only rows + inline steer (expanding input) + cancel. Pure view.

export default function Hub() {
  const open = usePapago((s) => s.hubOpen)
  const setHubOpen = usePapago((s) => s.setHubOpen)
  const fleet = usePapago((s) => s.fleet)
  const subs = useMemo(() => collectSubs(fleet), [fleet])
  const select = usePapago((s) => s.select)

  if (!open) return null
  const groups = subs.reduce<Record<string, typeof subs>>((acc, x) => {
    ;(acc[x.agentTitle] ??= []).push(x)
    return acc
  }, {})

  return (
    <div className="fixed top-10 right-6 w-130 max-h-[70vh] bg-glass-3 backdrop-blur-2xl border border-white/10 rounded-lg shadow-2xl flex flex-col z-30">
      <div className="h-10 px-3 flex items-center gap-2 border-b border-white/10">
        <span className="text-sm font-semibold text-ink">Agent Hub</span>
        <span className="text-xs text-mut/60">{subs.filter((s) => s.sub.status === 'running').length} live</span>
        <button onClick={() => setHubOpen(false)} className="ml-auto text-mut hover:text-ink text-sm"><Icon g={I.x} /></button>
      </div>
      <div className="flex-1 overflow-y-auto p-2 space-y-2">
        {subs.length === 0 && <div className="text-sm text-mut/60 p-3">no live subagents</div>}
        {Object.entries(groups).map(([title, rows]) => (
          <div key={title}>
            <div className="text-xs text-mut/70 px-1 py-0.5">{title}</div>
            {rows.map(({ agentId, index, sub }) => (
              <div key={`${agentId}:${index}`} className="rounded-md border border-white/10 bg-glass-1 px-2 py-1.5 mb-1 space-y-1">
                <div className="flex items-center gap-2">
                  <Dot status={sub.status} />
                  <span className="text-sm text-ink truncate">{sub.title}</span>
                  <span className="text-xs text-mut/70 truncate">{sub.model} · {sub.tokens}</span>
                  <span className="ml-auto">
                    <button className="text-xs text-mut/60 hover:text-ink" onClick={() => select({ t: 'agent', id: agentId })}><Icon g={I.arrowRight} /></button>
                  </span>
                </div>
                <div className="text-xs text-mut/70 pl-4 truncate">{sub.tool}</div>
              </div>
            ))}
          </div>
        ))}
      </div>
    </div>
  )
}
