// base16 theme system. Every semantic color token derives from a base16 palette
// (base00..base0F), so the prototype can swap themes without touching components.
// Semantic mapping (dark schemes):
//   bg=base00 · panel=base01 · surface=base02 · line=base03 · mut=base04
//   ink=base05 · acc=base0C · ok=base0B · warn=base0A · err=base08
// Glass layers derive from the same slots (base00/base01/base04 with alpha).

export interface Base16Palette {
  scheme: string
  base00: string
  base01: string
  base02: string
  base03: string
  base04: string
  base05: string
  base06: string
  base07: string
  base08: string
  base09: string
  base0A: string
  base0B: string
  base0C: string
  base0D: string
  base0E: string
  base0F: string
}

export const THEMES: Record<string, Base16Palette> = {
  'nord-dark': {
    scheme: 'Nord Dark',
    base00: '#2E3440',
    base01: '#3B4252',
    base02: '#434C5E',
    base03: '#4C566A',
    base04: '#D8DEE9',
    base05: '#E5E9F0',
    base06: '#ECEFF4',
    base07: '#8FBCBB',
    base08: '#BF616A',
    base09: '#D08770',
    base0A: '#EBCB8B',
    base0B: '#A3BE8C',
    base0C: '#88C0D0',
    base0D: '#81A1C1',
    base0E: '#B48EAD',
    base0F: '#5E81AC',
  },
  'gruvbox-dark': {
    scheme: 'Gruvbox Dark',
    base00: '#282828',
    base01: '#3C3836',
    base02: '#504945',
    base03: '#665C54',
    base04: '#BDAE93',
    base05: '#D5C4A1',
    base06: '#EBDBB2',
    base07: '#FBF1C7',
    base08: '#FB4934',
    base09: '#FE8019',
    base0A: '#FABD2F',
    base0B: '#B8BB26',
    base0C: '#8EC07C',
    base0D: '#83A598',
    base0E: '#D3869B',
    base0F: '#D65D0E',
  },
  'onedark-dark': {
    scheme: 'One Dark',
    base00: '#282C34',
    base01: '#353B45',
    base02: '#3E4451',
    base03: '#545862',
    base04: '#565C64',
    base05: '#ABB2BF',
    base06: '#B6BDCA',
    base07: '#C8CCD4',
    base08: '#E06C75',
    base09: '#D19A66',
    base0A: '#E5C07B',
    base0B: '#98C379',
    base0C: '#56B6C2',
    base0D: '#61AFEF',
    base0E: '#C678DD',
    base0F: '#BE5046',
  },
}

export const DEFAULT_THEME = 'nord-dark'
export const THEME_ORDER = Object.keys(THEMES)

const hexToRgba = (hex: string, alpha: number) => {
  const v = hex.replace('#', '')
  const r = parseInt(v.slice(0, 2), 16)
  const g = parseInt(v.slice(2, 4), 16)
  const b = parseInt(v.slice(4, 6), 16)
  return `rgb(${r} ${g} ${b} / ${alpha})`
}

// CSS custom properties consumed by the Tailwind @theme tokens.
export const themeVars = (p: Base16Palette): Record<string, string> => ({
  '--color-bg': p.base00,
  '--color-panel': p.base01,
  '--color-surface': p.base02,
  '--color-line': p.base03,
  '--color-mut': p.base04,
  '--color-ink': p.base05,
  '--color-acc': p.base0C,
  '--color-work': p.base0D,
  '--color-ok': p.base0B,
  '--color-warn': p.base0A,
  '--color-err': p.base08,
  '--color-glass-0': hexToRgba(p.base00, 0.52),
  '--color-glass-1': hexToRgba(p.base04, 0.05),
  '--color-glass-2': hexToRgba(p.base01, 0.62),
  '--color-glass-3': hexToRgba(p.base01, 0.88),
})

export const applyTheme = (id: string) => {
  const palette = THEMES[id] ?? THEMES[DEFAULT_THEME]
  const root = document.documentElement
  for (const [key, value] of Object.entries(themeVars(palette))) root.style.setProperty(key, value)
}