import { usePapago } from '../store'

export default function Welcome() {
  const openDirectory = usePapago((s) => s.openDirectory)
  return (
    <div className="flex-1 flex flex-col items-center justify-center gap-3">
      <div className="text-base font-semibold">papago</div>
      <div className="text-sm text-mut">run and review your omp agents — no editor, no terminal, no ceremony</div>
      <button onClick={() => openDirectory('myrepo')} className="mt-2 px-3 h-7 rounded bg-acc text-bg text-sm">
        Open directory…
      </button>
      <div className="text-xs text-mut/60">git root becomes a Project; a local Workspace appears with it</div>
    </div>
  )
}