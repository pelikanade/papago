package main

// Nerd Font icon glyphs (Font Awesome / Material sets shipped in FiraCode NF
// Mono) — mirrors prototype/src/icons.ts. Every visual icon in the app comes
// from here: no emoji, no plain-text arrows, no hand-drawn SVG.
const (
	icCaretDown  = "\uf0d7"
	icCaretRight = "\uf0da"
	icCaretLeft  = "\uf0d9"
	icAngleRight = "\uf105"
	icCheck      = "\uf00c"
	icCheckCir   = "\uf058"
	icXCir       = "\uf057"
	icX          = "\uf00d"
	icSpinner    = "\uf110"
	icFolder     = "\uf07b"
	icFork       = "\uf126"
	icHourglass  = "\uf252"
	icStop       = "\uf04d"
	icSend       = "\uf1d8"
	icArrowRight = "\uf061"
	icArrowUp    = "\uf062"
	icArrowDown  = "\uf063"
	icModel      = "\uf2db"
	icFilter     = "\uf0b0"
	icPlus       = "\uf067"
	icGear       = "\uf013"
	icTasks      = "\uf0ae"
	icSitemap    = "\uf0e8"
	icWinMin     = "\uf2d1"
	icWinMax     = "\uf2d0"
	icWinClose   = "\uf2d3"
	icWinRestore = "\uf2d2"
	icShield     = "\uf512"
)

// Font families, resolved through fontconfig exactly as the prototype's CSS
// stacks resolve in the browser: `ui-sans-serif, system-ui, "Noto Sans SC",
// sans-serif` lands on Noto Sans CJK SC; `ui-monospace, "JetBrains Mono",
// monospace` lands on FiraCode Nerd Font; `.nf` is FiraCode Nerd Font Mono.
const (
	fontSans  = "Noto Sans CJK SC"
	fontMono  = "FiraCode Nerd Font"
	fontIcons = "FiraCode Nerd Font Mono"
)
