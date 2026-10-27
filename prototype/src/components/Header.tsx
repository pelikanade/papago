import { IconButton } from './primitives'
import { I } from '../icons'
import { usePapago } from '../store'

// Shared pane header: breadcrumbs · title · window controls (minimize/maximize/close).
export default function Header({ title, sub, crumbs }: { title: string; sub?: string; crumbs?: React.ReactNode }) {
  const setWin = usePapago((s) => s.setWin)
  const win = usePapago((s) => s.win)
  return (
    <div className="h-8 shrink-0 px-4 flex items-center gap-3 border-b border-white/10">
      {crumbs && <span className="text-xs text-mut/60">{crumbs}</span>}
      <span className="text-xs font-semibold text-ink truncate">{title}</span>
      {sub && <span className="text-xs text-mut truncate">{sub}</span>}
      <div className="ml-auto flex items-center gap-1">
        <IconButton g={I.winMin} title="minimize" onClick={() => setWin('minimized')} />
        <IconButton
          g={win === 'maximized' ? I.winRestore : I.winMax}
          title={win === 'maximized' ? 'restore' : 'maximize'}
          onClick={() => setWin(win === 'maximized' ? 'normal' : 'maximized')}
        />
        <IconButton g={I.winClose} title="close" tone="err" onClick={() => setWin('closed')} />
      </div>
    </div>
  )
}
