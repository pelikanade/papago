import Sidebar from './components/Sidebar'
import Hub from './components/Hub'
import Palette from './components/Palette'
import { Icon } from './components/primitives'
import { I } from './icons'
import { usePapago } from './store'
import Welcome from './pages/Welcome'
import Diagnostics from './pages/Diagnostics'
import ProjectView from './pages/ProjectView'
import WorkspaceView from './pages/WorkspaceView'
import AgentView from './pages/AgentView'

// App shell only: window frame + states (normal/maximized/minimized/closed),
// routing to pages, theme application, and the streaming heartbeat.
// Views live in src/pages, reusable pieces in src/components.

export default function App() {
  const ompMissing = usePapago((s) => s.ompMissing)
  const hasProjects = usePapago((s) => s.fleet.projects.length > 0)
  const win = usePapago((s) => s.win)
  const setWin = usePapago((s) => s.setWin)

  if (win === 'closed')
    return (
      <div className="h-full flex items-center justify-center">
        <button
          onClick={() => setWin('normal')}
          className="px-4 h-9 rounded-lg bg-glass-3 border border-white/10 shadow-2xl text-sm text-ink hover:border-acc"
        >
          reopen papago
        </button>
      </div>
    )

  if (win === 'minimized')
    return (
      <div className="h-full flex items-end justify-start p-6">
        <button
          onClick={() => setWin('normal')}
          className="px-4 h-9 rounded-lg bg-glass-3 border border-white/10 shadow-2xl text-sm text-ink flex items-center gap-2 hover:border-acc"
        >
          <Icon g={I.winRestore} /> papago — minimized
        </button>
      </div>
    )

  return (
    <div className={win === 'maximized' ? 'h-full p-0' : 'h-full p-3'}>
      <div
        className={`h-full flex ${win === 'maximized' ? 'rounded-none' : 'rounded-xl'} overflow-hidden border border-white/10 shadow-2xl bg-glass-0 backdrop-blur-3xl text-ink font-sans select-none`}
      >
        <Sidebar />
        <div className="flex-1 min-w-0 flex flex-col">
          {ompMissing ? <Diagnostics /> : !hasProjects ? <Welcome /> : <Routed />}
        </div>
        <Hub />
        <Palette />
      </div>
    </div>
  )
}

const Routed = () => {
  const sel = usePapago((s) => s.sel)
  if (!sel) return <Welcome />
  if (sel.t === 'project') return <ProjectView id={sel.id} />
  if (sel.t === 'workspace') return <WorkspaceView id={sel.id} />
  return <AgentView id={sel.id} />
}