import Header from '../components/Header'
import ComposerBar from '../components/ComposerBar'
import WorkspaceGraph from '../components/WorkspaceGraph'
import { findWorkspace, usePapago } from '../store'
import { useShallow } from 'zustand/react/shallow'

// Workspace view: graph (its single worker agent; subagents are ephemeral) +
// the worker's composer, so you can prompt it straight from the graph.
export default function WorkspaceView({ id }: { id: string }) {
  const found = usePapago(useShallow((s) => findWorkspace(s, id)))
  const select = usePapago((s) => s.select)
  const selAgentId = usePapago((s) => (s.sel?.t === 'agent' ? s.sel.id : undefined))
  if (!found) return null
  const { p, w } = found
  const { worker } = w
  return (
    <>
      <Header
        title={w.name}
        sub={`${w.path} · ${w.isolation === 'worktree' ? `worktree · base ${p.branch}` : 'local checkout'}`}
        crumbs={`${p.name} ▸`}
      />
      <div className="flex-1 min-h-0 overflow-auto px-4 py-3">
        <WorkspaceGraph
          agents={worker.archived ? [] : [worker]}
          selectedId={selAgentId}
          onSelect={(agentId) => select({ t: 'agent', id: agentId })}
        />
      </div>
      <ComposerBar variant="agent" projectId={p.id} workspaceId={w.id} agentId={worker.id} />
    </>
  )
}
