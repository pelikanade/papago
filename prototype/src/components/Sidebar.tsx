import { Dot, Icon, IconButton, Popover, TreeConnector, TreeRow } from './primitives'
import { I } from '../icons'
import { THEMES } from '../themes'
import { usePapago } from '../store'
import { URGENCY } from '../data'

// Sidebar: the Project row IS its orchestrator — hollow ring dot, project path
// right; each Workspace row IS its worker (1:1) — filled status dot + workspace
// name left, PR/CI right. Project blocks are separated by a vertical gap only;
// workspace rows carry tree connectors aligned with the caret, ending in an
// elbow on the last row. Subagents are ephemeral: they never appear here,
// the status bar owns them. All rows share <TreeRow>; the footer hosts + Project
// and settings.

// Shared "add project / workspace" control (footer + collapsed rail share one
// creation path and one popover, per the componentization rule).
function AddProjectControl({ iconOnly = false }: { iconOnly?: boolean }) {
  const popovers = usePapago((s) => s.popovers)
  const drafts = usePapago((s) => s.drafts)
  const setDraft = usePapago((s) => s.setDraft)
  const openDirectory = usePapago((s) => s.openDirectory)
  const togglePopover = usePapago((s) => s.togglePopover)

  const create = () => {
    const v = (drafts['sel:project'] ?? '').trim()
    if (!v) return
    openDirectory(v)
    setDraft('sel:project', '')
    togglePopover('sel:project')
  }

  return (
    <span className="relative flex">
      {iconOnly ? (
        <IconButton g={I.plus} title="add project / workspace" onClick={() => togglePopover('sel:project')} />
      ) : (
        <button
          onClick={() => togglePopover('sel:project')}
          className="h-7 pl-1.5 pr-2 flex items-center gap-1.5 rounded text-xs text-mut hover:text-ink hover:bg-white/[0.06]"
          title="add project / workspace"
        >
          <Icon g={I.plus} /> Project
        </button>
      )}
      <Popover open={!!popovers['sel:project']} onClose={() => togglePopover('sel:project')} anchor="left" width="16rem">
        <div className="p-1 space-y-1.5">
          <input
            value={drafts['sel:project'] ?? ''}
            onChange={(e) => setDraft('sel:project', e.target.value)}
            onKeyDown={(e) => {
              if (e.key === 'Enter') create()
            }}
            placeholder="directory path or name"
            className="w-full h-7 rounded border border-line bg-transparent px-2 text-xs outline-none focus:border-acc"
          />
          <button onClick={create} className="w-full h-7 rounded bg-acc text-bg text-xs">
            Create Project
          </button>
        </div>
      </Popover>
    </span>
  )
}

export default function Sidebar() {
  const projects = usePapago((s) => s.fleet.projects)
  const sel = usePapago((s) => s.sel)
  const select = usePapago((s) => s.select)
  const collapsed = usePapago((s) => s.sidebarCollapsed)
  const toggleSidebar = usePapago((s) => s.toggleSidebar)
  const openP = usePapago((s) => s.popovers)
  const togglePopover = usePapago((s) => s.togglePopover)
  const setPaletteOpen = usePapago((s) => s.setPaletteOpen)
  const popovers = usePapago((s) => s.popovers)
  const themeId = usePapago((s) => s.themeId)
  const cycleTheme = usePapago((s) => s.cycleTheme)

  // Collapsed rail: one status dot per Agent (orchestrators + workers), click to open.
  if (collapsed) {
    const rail = projects
      .flatMap((p) => [
        { a: p.orchestrator, label: `${p.name} · orchestrator` },
        ...p.workspaces
          .filter((w) => !w.worker.archived)
          .map((w) => ({ a: w.worker, label: `${p.name} / ${w.name} · ${w.worker.title}` })),
      ])
      .sort((x, y) => URGENCY[x.a.status] - URGENCY[y.a.status])
    return (
      <div className="relative z-20 w-8 shrink-0 border-r border-white/10 bg-glass-1 backdrop-blur-xl flex flex-col">
        {/* top strip mirrors the pane header (h-10 + hairline) */}
        <div className="h-8 shrink-0 border-b border-white/10 flex items-center justify-center">
          <IconButton g={I.caretRight} title="expand sidebar" onClick={toggleSidebar} />
        </div>
        <div className="flex-1 w-full flex flex-col items-center gap-1.5 overflow-y-auto py-1.5">
          {rail.map(({ a, label }) => (
            <button
              key={a.id}
              title={label}
              onClick={() => select({ t: 'agent', id: a.id })}
              className={`w-6 h-6 flex items-center justify-center rounded hover:bg-white/[0.08] ${
                sel?.t === 'agent' && sel.id === a.id ? 'bg-white/[0.10]' : ''
              }`}
            >
              <Dot status={a.status} ring={a.role === 'orchestrator'} />
            </button>
          ))}
        </div>
        {/* bottom strip mirrors the status bar (h-7 + hairline) */}
        <div className="h-7 shrink-0 border-t border-white/10 flex items-center justify-center">
          <AddProjectControl iconOnly />
        </div>
      </div>
    )
  }

  return (
    <div className="relative z-20 w-72 shrink-0 border-r border-white/10 bg-glass-1 backdrop-blur-xl flex flex-col">
      <div className="h-8 px-3 flex items-center gap-2 border-b border-white/10">
        <span className="text-sm font-semibold tracking-wide text-ink">papago</span>
        <span className="text-xs text-mut/60">prototype</span>
        <button onClick={() => setPaletteOpen(true)} className="text-xs text-mut hover:text-ink" title="palette">⌘K</button>
        <IconButton g={I.caretLeft} title="collapse sidebar" onClick={toggleSidebar} className="ml-auto" />
      </div>

      <div className="flex-1 overflow-y-auto py-1.5">
        {projects.map((p, pi) => {
          const pOpen = openP[`tree:${p.id}`] ?? true
          const o = p.orchestrator
          return (
            // Each Project is its own block: a larger vertical gap separates the
            // blocks, so an orchestrator header never reads as a sibling row.
            <div key={p.id} className={pi > 0 ? 'mt-4' : ''}>
              {/* The Project row IS the orchestrator: clicking it opens that agent. */}
              <TreeRow
                indent={0}
                selected={sel?.t === 'agent' && sel.id === o.id}
                onClick={() => select({ t: 'agent', id: o.id })}
                onCaret={() => togglePopover(`tree:${p.id}`)}
                open={pOpen}
                title={`${p.name} · orchestrator`}
              >
                <Dot status={o.status} ring />
                <span className="text-xs text-ink font-medium truncate">{p.name}</span>
                <span className="ml-auto text-xs text-mut/50 truncate">{p.path}</span>
              </TreeRow>

              {/* Workspace rows double as their worker agent rows (1:1): status dot +
                  workspace name on the left, PR/CI on the right, plus the tree
                  connector that wraps the group (elbow on the last row). */}
              {pOpen &&
                p.workspaces.map((w, i) => {
                  const a = w.worker
                  if (a.archived) return null
                  const last = p.workspaces.slice(i + 1).every((x) => x.worker.archived)
                  return (
                    <div key={w.id} className="relative">
                      <TreeRow
                        indent={1}
                        selected={sel?.t === 'agent' && sel.id === a.id}
                        onClick={() => select({ t: 'agent', id: a.id })}
                        title={`${w.name} · ${w.branch} · ${a.title} · ${w.path}`}
                      >
                        <Dot status={a.status} />
                        <span className="text-xs text-ink truncate">{w.name}</span>
                        {w.pr != null && (
                          <span className="ml-auto flex items-center gap-1 text-xs text-mut/60">
                            <Icon g={I.fork} /> {w.pr}
                            {w.ci && (
                              <>
                                <span className="text-mut/40">·</span>
                                <Icon
                                  g={w.ci === 'passed' ? I.checkCircle : w.ci === 'failed' ? I.xCircle : I.spinner}
                                  className={w.ci === 'passed' ? 'text-ok' : w.ci === 'failed' ? 'text-err' : 'text-acc'}
                                />
                                <span className={w.ci === 'passed' ? 'text-ok' : w.ci === 'failed' ? 'text-err' : 'text-acc'}>{w.ci}</span>
                              </>
                            )}
                          </span>
                        )}
                      </TreeRow>
                      <TreeConnector last={last} />
                    </div>
                  )
                })}
            </div>
          )
        })}
      </div>

      <div className="h-7 px-1.5 border-t border-white/10 flex items-center">
        <AddProjectControl />
        <div className="flex-1" />
        <span className="relative flex">
          <IconButton g={I.gear} title="settings" onClick={() => togglePopover('footer:settings')} />
          <Popover open={!!popovers['footer:settings']} onClose={() => togglePopover('footer:settings')} anchor="right" width="15rem">
            <div className="p-1 space-y-0.5">
              <button onClick={() => cycleTheme()} className="w-full flex items-center gap-2 px-2 py-2 rounded text-xs text-ink hover:bg-white/[0.07]">
                Theme: {THEMES[themeId]?.scheme ?? themeId}
              </button>
              <div className="px-2 py-1.5 text-xs text-mut/70">daemon · local</div>
              <div className="px-2 py-1.5 text-xs text-mut/70 font-mono truncate">~/.papago/config.json</div>
            </div>
          </Popover>
        </span>
      </div>
    </div>
  )
}