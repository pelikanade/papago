import { Dot } from './primitives'
import type { Agent } from '../types'

// Workspace graph: nodes = the workspace's agents (1:1 today, its worker).
// Subagents are ephemeral and never become nodes — the status bar shows them.
// Clicking a node opens the Agent view.
export default function WorkspaceGraph({
  agents,
  selectedId,
  onSelect,
}: {
  agents: Agent[]
  selectedId?: string
  onSelect: (id: string) => void
}) {
  if (agents.length === 0) return <div className="text-sm text-mut/60 py-6 text-center">no agents here yet — send a prompt below</div>

  const COL = 260
  const NODE_W = 212
  const NODE_H = 64
  const width = Math.max(...agents.map((_, i) => i * COL + NODE_W + 12))
  const height = NODE_H + 24

  return (
    <div className="relative" style={{ width, height }}>
      {agents.map((agent, i) => (
        <button
          key={agent.id}
          onClick={() => onSelect(agent.id)}
          style={{ left: i * COL + 12, top: 12, width: NODE_W, height: NODE_H }}
          className={`absolute rounded-lg border bg-glass-1 backdrop-blur-md px-3 py-2 text-left hover:border-acc/70 ${
            selectedId === agent.id ? 'border-acc' : 'border-white/10'
          }`}
        >
          <div className="flex items-center gap-2">
            <Dot status={agent.status} />
            <span className="text-sm text-ink truncate">{agent.title}</span>
            <span className="ml-auto text-xs text-mut/60">{agent.status}</span>
          </div>
          <div className="mt-1 text-xs text-mut/70 truncate">{agent.model}</div>
        </button>
      ))}
    </div>
  )
}
