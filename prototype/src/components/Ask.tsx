import { usePapago } from '../store'
import type { TimelineItem } from '../types'

type AskItem = Extract<TimelineItem, { k: 'ask' }>

// Interactive Ask form, rendered in the composer slot (replaces the text box while a
// pending Ask exists). Inline timeline entries stay read-only references.
export const AskForm = ({ agentId, index, it }: { agentId: string; index: number; it: AskItem }) => {
  const answerAsk = usePapago((s) => s.answerAsk)
  const draft = usePapago((s) => s.drafts[`ask:${agentId}:${index}`] ?? '')
  const setDraft = usePapago((s) => s.setDraft)

  return (
    <div className="rounded-lg border border-warn/50 bg-warn/5 backdrop-blur-md p-2.5 space-y-2">
      <div className="flex items-center gap-2">
        <span className="text-xs px-1.5 rounded bg-warn/20 text-warn">Ask · {it.kind}</span>
        <span className="text-xs text-mut">blocks this step until answered · never expires</span>
      </div>
      <div className="text-sm text-ink">{it.q}</div>

      {it.kind === 'select' && (
        <div className="flex flex-wrap gap-1.5">
          {(it.options ?? []).map((o, i) => (
            <button
              key={o}
              onClick={() => answerAsk(agentId, index, o)}
              className="px-2 h-7 rounded border border-line text-sm text-ink hover:border-acc hover:text-acc"
            >
              <span className="text-mut/60 mr-1">{i + 1}</span>
              {o}
            </button>
          ))}
        </div>
      )}

      {it.kind === 'confirm' && (
        <div className="flex gap-1.5">
          <button onClick={() => answerAsk(agentId, index, 'Approve')} className="px-2.5 h-7 rounded bg-acc text-bg text-sm">
            Approve
          </button>
          <button
            onClick={() => answerAsk(agentId, index, 'Cancel')}
            className="px-2.5 h-7 rounded border border-line text-sm text-mut hover:text-err hover:border-err"
          >
            Cancel
          </button>
        </div>
      )}

      {it.kind === 'input' && (
        <div className="flex gap-1.5">
          <input
            value={draft}
            onChange={(e) => setDraft(`ask:${agentId}:${index}`, e.target.value)}
            onKeyDown={(e) => {
              if (e.key === 'Enter' && draft.trim()) answerAsk(agentId, index, draft.trim())
            }}
            className="flex-1 h-7 rounded border border-line bg-transparent px-2 text-sm outline-none focus:border-acc"
          />
          <button
            onClick={() => draft.trim() && answerAsk(agentId, index, draft.trim())}
            className="px-2.5 h-7 rounded bg-acc text-bg text-sm"
          >
            Submit
          </button>
        </div>
      )}
    </div>
  )
}