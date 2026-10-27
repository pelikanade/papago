import { usePapago } from '../store'

export default function Diagnostics() {
  const setOmpMissing = usePapago((s) => s.setOmpMissing)
  return (
    <div className="flex-1 flex flex-col items-center justify-center gap-3">
      <div className="text-sm text-warn font-semibold">omp not found on PATH</div>
      <pre className="text-xs font-mono text-mut bg-panel border border-line rounded-md p-3">
        {'curl -fsSL https://omp.sh/install | sh\n# or: bun install -g @oh-my-pi/pi-coding-agent'}
      </pre>
      <div className="text-xs text-mut">or set an absolute path in ~/.papago/config.json → omp.path</div>
      <button onClick={() => setOmpMissing(false)} className="px-3 h-7 rounded border border-line text-sm text-mut hover:text-ink">
        Retry detection
      </button>
    </div>
  )
}