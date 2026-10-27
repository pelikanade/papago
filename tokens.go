package main

// Tailwind's spacing and type scale, as the design uses it. The prototype is
// written in Tailwind classes; this file gives the Go port the same numbers so
// a class like `gap-2` or `text-sm` reads the same on both sides. Values are
// CSS pixels == MyGo DIPs (logical pixels).

const (
	// spacing scale (1 unit = 4px)
	sp1  = 4.0  // gap-1, p-1
	sp15 = 6.0  // gap-1.5
	sp2  = 8.0  // gap-2, p-2, px-2
	sp25 = 10.0 // left-2.5 (tree connector)
	sp3  = 12.0 // gap-3, p-3, px-3
	sp35 = 14.0 // w-3.5 (connector elbow)
	sp4  = 16.0 // p-4
	sp6  = 24.0 // pl-6 (indent 1)
	sp10 = 40.0 // pl-10 (indent 2)
	sp14 = 56.0 // pl-14 (indent 3)

	// type scale (font size / line height)
	textXS       = 12.0 // text-xs
	textXSLine   = 16.0
	textSM       = 14.0 // text-sm
	textSMLine   = 20.0
	textBase     = 16.0 // text-base
	textBaseLine = 24.0

	// font weights. The UI family (Noto Sans CJK SC) has no semibold face:
	// CSS's 600 resolves to Bold there too, so the port asks for 700. Asking
	// MyGo for 600 picks a face whose advances come out narrower than regular
	// (64px instead of 74px for "fix-login bug"), which crowds and overlaps
	// glyphs — see the header title.
	weightRegular  = 400
	weightMedium   = 500
	weightSemiBold = 700
	weightBold     = 700

	// radii
	radiusSM   = 4.0 // rounded
	radiusMD   = 6.0 // rounded-md
	radiusLG   = 8.0 // rounded-lg
	radiusFull = 9999.0

	// control heights & strip heights
	h5  = 20.0 // h-5 (chips, badges)
	h6  = 24.0 // h-6
	h7  = 28.0 // h-7 (icon buttons, status bar, footer)
	h8  = 32.0 // h-8 (pane header, sidebar header)
	h9  = 36.0 // h-9
	h10 = 40.0 // h-10

	// fixed widths
	w6  = 24.0  // w-6 (rail button)
	w8  = 32.0  // w-8 (collapsed rail)
	w72 = 288.0 // w-72 (sidebar)
	w25 = 10.0  // w-2.5 / h-2.5 (status dot)

	// hairlines
	hairline = 1.0

	// MyGo's text boxes carry this font's full ink height (18px at text-xs,
	// 22px at text-sm) rather than Tailwind's 16/20px line box, so rows use
	// paddings 1–2px tighter to keep the design's rhythm:
	// py-1.5 (6px) + 16px line → rowPadXS + 18px = 28px.
	rowPadXS = 5.0 // tree rows (text-xs)
	rowPadSM = 1.0 // timeline rows (text-sm)
	menuPadY = 7.0 // popover menu items (py-2 → 32px rows)
	cardPadY = 7.0 // project card rows (py-2 → 36px rows)

	// paneMinWidth is the narrowest the pane's content gets: under it, a page's
	// content keeps this width and scrolls sideways inside its own box (the
	// header, composer and status bar stay at the window's width). Squeezing
	// past it wraps and drops labels instead of reflowing.
	paneMinWidth = 520.0

	// railBreakpoint: narrower than this, the sidebar takes its rail form so
	// the pane keeps room., the sidebar takes its rail form so the
	// pane keeps room. The design has no breakpoints of its own — the prototype
	// is a web page where the compositor never shrinks the window below a
	// useful size; native windows do, so narrow widths get the rail.
	railBreakpoint = 720.0

	// a node's dot is 10px; the tree connector runs at the caret's centre
	treeCaretCenter = sp25
)

// rem is Tailwind's rem: the browser's 16px root font size, which the
// prototype's user unit (w-72 = 18rem, popovers 15rem/16rem) is written in.
func rem(n float64) float32 { return float32(n * 16) }

// Alpha tints the prototype writes as Tailwind opacity modifiers over white
// (bg-white/[0.x]) and as border-white/10.
const (
	alphaBorder   = 0.10
	alphaHover    = 0.06
	alphaHoverAlt = 0.07
	alphaSelected = 0.08
)
