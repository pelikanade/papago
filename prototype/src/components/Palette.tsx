import { useMemo } from 'react'
import { collectAsks, usePapago } from '../store'
import { Icon } from './primitives'
import { I } from '../icons'
import { THEMES } from '../themes'

// Command palette (Ctrl+K): jumps + actions. The only shortcut surface.
// Pure view: query + open state live in the store.

interface Row {
  id: string
  label: string
  hint?: string
  icon?: string
  run: () => void
}

export default function Palette() {
  const open = usePapago((s) => s.paletteOpen)
  const q = usePapago((s) => s.paletteQuery)
  const setPaletteOpen = usePapago((s) => s.setPaletteOpen)
  const setPaletteQuery = usePapago((s) => s.setPaletteQuery)
  const fleet = usePapago((s) => s.fleet)
  const select = usePapago((s) => s.select)
  const setHubOpen = usePapago((s) => s.setHubOpen)
  const pruneWorktrees = usePapago((s) => s.pruneWorktrees)
  const openDirectory = usePapago((s) => s.openDirectory)
  const setOmpMissing = usePapago((s) => s.setOmpMissing)
  const themeId = usePapago((s) => s.themeId)
  const cycleTheme = usePapago((s) => s.cycleTheme)
  const asks = useMemo(() => collectAsks(fleet), [fleet])

  if (!open) return null

  const rows: Row[] = [
    ...asks.map((a) => ({
      id: `ask:${a.agentId}:${a.index}`,
      label: `Next Ask — ${a.q}`,
      hint: 'jump',
      run: () => select({ t: 'agent', id: a.agentId }),
    })),
    ...fleet.projects.flatMap((p) => [
      { id: `p:${p.id}`, label: p.name, icon: I.arrowRight, hint: 'project', run: () => select({ t: 'project', id: p.id }) },
      {
        id: `o:${p.orchestrator.id}`,
        label: `${p.name} / orchestrator`,
        icon: I.sitemap,
        hint: p.orchestrator.status,
        run: () => select({ t: 'agent', id: p.orchestrator.id }),
      },
      ...p.workspaces.flatMap((w) => [
        { id: `w:${w.id}`, label: `${p.name} / ${w.name}`, icon: I.arrowRight, hint: w.isolation, run: () => select({ t: 'workspace', id: w.id }) },
        { id: `a:${w.worker.id}`, label: w.worker.title, icon: I.arrowRight, hint: w.worker.status, run: () => select({ t: 'agent', id: w.worker.id }) },
      ]),
      { id: `prune:${p.id}`, label: `Prune worktrees — ${p.name}`, hint: 'destructive', run: () => pruneWorktrees(p.id) },
    ]),
    { id: 'hub', label: 'Open Agent Hub', run: () => setHubOpen(true) },
    { id: 'theme', label: `Theme: ${THEMES[themeId]?.scheme ?? themeId} → next`, hint: 'base16', run: () => cycleTheme() },
    { id: 'open', label: 'Open directory…', run: () => openDirectory('newrepo') },
    { id: 'diag', label: 'Toggle omp diagnostics view', hint: 'debug', run: () => setOmpMissing(!usePapago.getState().ompMissing) },
  ]

  const filtered = rows.filter((r) => r.label.toLowerCase().includes(q.toLowerCase()))

  return (
    <>
      <div className="fixed inset-0 z-40 bg-black/40" onClick={() => setPaletteOpen(false)} />
      <div className="fixed top-16 left-1/2 -translate-x-1/2 w-160 bg-glass-3 backdrop-blur-2xl border border-white/10 rounded-lg shadow-2xl z-50 overflow-hidden">
        <input
          autoFocus
          value={q}
          onChange={(e) => setPaletteQuery(e.target.value)}
          placeholder="jump to Agent / Workspace · run action…"
          className="w-full h-9 px-3 bg-transparent outline-none text-sm text-ink placeholder:text-mut/60 border-b border-white/10"
        />
        <div className="max-h-105 overflow-y-auto py-1">
          {filtered.map((r) => (
            <button
              key={r.id}
              onClick={() => {
                r.run()
                setPaletteOpen(false)
              }}
              className="w-full flex items-center gap-2 px-3 py-1.5 text-left hover:bg-white/[0.07]"
            >
              {r.icon && <Icon g={r.icon} className="text-mut/60" />}
              <span className="text-sm text-ink truncate">{r.label}</span>
              {r.hint && <span className="ml-auto text-xs text-mut/60">{r.hint}</span>}
            </button>
          ))}
          {filtered.length === 0 && <div className="px-3 py-2 text-sm text-mut/60">no match</div>}
        </div>
      </div>
    </>
  )
}
