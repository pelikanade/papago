import Header from '../components/Header'
import ComposerBar from '../components/ComposerBar'
import { Dot, Icon } from '../components/primitives'
import { I } from '../icons'
import { usePapago } from '../store'

// Project view: the orchestrator row (read/plan/dispatch) + its workspaces.
// Sending from the composer below goes to the project orchestrator, which
// creates a workspace + worktree + worker for the prompt.

// One card row for every entry in the list (orchestrator, then workspaces).
const CardRow = ({ onClick, children }: { onClick: () => void; children: React.ReactNode }) => (
  <button
    onClick={onClick}
    className="w-full rounded-md border border-line px-3 py-2 flex items-center gap-2 text-left hover:border-mut/60"
  >
    {children}
  </button>
)

export default function ProjectView({ id }: { id: string }) {
  const p = usePapago((s) => s.fleet.projects.find((x) => x.id === id))
  const select = usePapago((s) => s.select)
  if (!p) return null
  const o = p.orchestrator
  return (
    <>
      <Header title={p.name} sub={`${p.path} · ${p.branch} · ${p.workspaces.filter((w) => !w.worker.archived).length} workspaces`} />
      <div className="flex-1 min-h-0 overflow-y-auto px-4 py-3 space-y-2">
        <CardRow onClick={() => select({ t: 'agent', id: o.id })}>
          <Dot status={o.status} ring />
          <span className="text-xs text-mut/60">
            <Icon g={I.sitemap} />
          </span>
          <span className="text-sm text-ink font-medium">orchestrator</span>
          <span className="text-xs text-mut">read · plan · dispatch — workers own writes</span>
          <span className="ml-auto text-xs text-mut/60">{o.model}</span>
        </CardRow>

        {p.workspaces
          .filter((w) => !w.worker.archived)
          .map((w) => (
            <CardRow key={w.id} onClick={() => select({ t: 'workspace', id: w.id })}>
              <Dot status={w.worker.status} />
              <span className="text-sm text-ink font-medium">{w.name}</span>
              <span className="text-xs text-mut">{w.isolation === 'worktree' ? `worktree · ${w.branch}` : 'local checkout'}</span>
              <span className="ml-auto text-xs text-mut/60 truncate">{w.worker.title}</span>
            </CardRow>
          ))}
      </div>
      <ComposerBar variant="project" projectId={p.id} />
    </>
  )
}
