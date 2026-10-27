import { createRoot } from 'react-dom/client'
import App from './App'
import './index.css'
import { usePapago } from './store'
import { applyTheme } from './themes'

// React Grab (dev only): hover any element, ⌘C / Ctrl+C to copy its
// component stack + source location for the agent. Installed per its Vite docs.
if (import.meta.env.DEV) {
  import('react-grab')
}

// Prototype drivers live outside React (useEffect is banned, see AGENTS.md):
// initial theme, streaming heartbeat, and the global palette shortcut.
applyTheme(usePapago.getState().themeId)
setInterval(() => usePapago.getState().tick(), 1000)
setInterval(() => usePapago.getState().tickClock(), 100)
window.addEventListener('keydown', (e) => {
  const state = usePapago.getState()
  if ((e.ctrlKey || e.metaKey) && e.key.toLowerCase() === 'k') {
    e.preventDefault()
    state.setPaletteOpen(!state.paletteOpen)
  }
  if (e.key === 'Escape' && state.paletteOpen) state.setPaletteOpen(false)
})

const root = document.getElementById('root')
if (!root) throw new Error('missing #root')
createRoot(root).render(<App />)
