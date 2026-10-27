import Header from '../components/Header'
import ComposerBar from '../components/ComposerBar'
import Timeline from '../components/Timeline'
import { Icon } from '../components/primitives'
import { I } from '../icons'
import { findAgent, usePapago } from '../store'
import { useShallow } from 'zustand/react/shallow'

// Agent view: one agent's timeline + its composer. Workers sit inside a
// workspace; the orchestrator sits directly under the project (no workspace).
export default function AgentView({ id }: { id: string }) {
  const found = usePapago(useShallow((s) => findAgent(s, id)))
  if (!found) return null
  const { p, w, a } = found
  return (
    <>
      <Header
        title={a.role === 'orchestrator' ? 'orchestrator' : a.title}
        crumbs={
          <>
            {p.name} <Icon g={I.angleRight} /> {w ? (
              <>
                {' '}
                {w.name} <Icon g={I.angleRight} />
              </>
            ) : null}
          </>
        }
      />
      <Timeline agent={a} />
      <ComposerBar variant="agent" projectId={p.id} workspaceId={w?.id} agentId={a.id} />
    </>
  )
}
