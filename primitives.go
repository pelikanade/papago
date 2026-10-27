package main

import (
	"math"
	"time"

	"github.com/egoist/mygo/ui"
)

// Shared primitives — the Go form of prototype/src/components/primitives.tsx.
// Every surface composes these instead of repeating markup: rows, icon
// buttons, status dots, status-bar segments, tree connectors.

// lineHeight is Tailwind's paired line height for a type size — text-xs 16px,
// text-sm 20px, text-base 24px. The design's rhythm (h-7 rows, h-8 strips)
// assumes those exact line boxes, so the port fixes them in DIPs.
func lineHeight(size float32) float32 {
	switch size {
	case textXS:
		return textXSLine
	case textSM:
		return textSMLine
	case textBase:
		return textBaseLine
	}
	return size * 1.4
}

// label is a text in the design's UI font (with the icon font behind it, so a
// Nerd Font glyph in the string still resolves).
func label(c *ui.Context, s string, size float32, weight int, color ui.Color) ui.Element {
	return ui.Text(c, s).
		Font(fontSans + ", " + fontIcons).
		FontSize(size).
		FontWeight(weight).
		FixedLineHeight(lineHeight(size)).
		TextColor(color)
}

// glyph is a Nerd Font icon: the prototype's `.nf` class (line-height: 1).
func glyph(c *ui.Context, g string, size float32, color ui.Color) ui.Element {
	return ui.Text(c, g).Font(fontIcons).FontSize(size).FixedLineHeight(size).TextColor(color)
}

// dotColor carries the dot semantics: gray idle · blue working · yellow
// needs-attention · green done · red failed.
func dotColor(t Theme, s AgentStatus) ui.Color {
	switch s {
	case StatusRunning:
		return col(t.Work)
	case StatusNeedsYou:
		return col(t.Warn)
	case StatusDone:
		return col(t.Ok)
	case StatusError:
		return col(t.Err)
	default:
		return col(t.Mut)
	}
}

// dot is the 10px status dot; `ring` draws the hollow orchestrator form
// (same status color, no fill) that keeps the two row kinds apart. A running
// agent's dot pulses, as the design's dot-pulse keyframes do (1 → 0.35 → 1
// over 2.4s, ease in out).
func dot(c *ui.Context, t Theme, status AgentStatus, ring bool) ui.Element {
	colr := dotColor(t, status)
	if status == StatusRunning {
		colr = colr.Alpha(pulseAlpha(c.Now()))
	}
	d := ui.Box(c).Size(w25, w25).Radius(radiusFull).Shrink(0)
	if ring {
		return d.Background(ui.Transparent).Border(2, colr)
	}
	return d.Background(colr)
}

// pulseAlpha is the design's dot-pulse opacity at now: 1 at the start of each
// 2.4s cycle, 0.35 at the midpoint.
func pulseAlpha(now time.Time) float32 {
	const period = 2.4
	phase := math.Mod(float64(now.UnixNano())/1e9, period) / period
	return float32(0.675 + 0.325*math.Cos(2*math.Pi*phase))
}

// iconButton is the h-7 w-7 toolbar button (window controls, panel header
// actions): one shell so size, hover and tone never drift between surfaces.
func iconButton(c *ui.Context, g, title string, onClick func()) ui.Element {
	b := ui.Row(c).
		Size(h7, h7).
		AlignItems(ui.Center).
		Justify(ui.Center).
		Radius(radiusSM).
		Cursor(ui.CursorPointer).
		Tooltip(title).
		Focusable()
	if b.Hovered() {
		b = b.Background(whiteA(alphaHoverAlt))
	}
	b = b.Draw(func(p *ui.Painter, r ui.Rect) {
		w, h := p.MeasureText(0, ui.Span{Text: g, Font: fontIcons, Size: textSM})
		p.Text(r.X+(r.W-w)/2, r.Y+(r.H-h)/2, g, textSM, c.Theme().TextMuted)
	})
	if b.Clicked() && onClick != nil {
		onClick()
	}
	return b
}

// scrollContent wraps a page's scrollable content. It scrolls vertically; under
// paneMinWidth the content keeps that width and scrolls sideways instead of
// being squeezed, and the design's always-visible bar tracks both axes.
func (a *app) scrollContent(c *ui.Context, t Theme, st *ui.ScrollState, build func()) {
	narrow := a.paneWidth(c) < paneMinWidth
	var el ui.Element
	if narrow {
		el = ui.ScrollBoth(c).Grow(1).Basis(0).TrackScroll(st)
	} else {
		el = ui.Scroll(c).Grow(1).Basis(0).TrackScroll(st)
	}
	attachScrollbar(c, t, el, st)
	el.Children(func() {
		col := ui.Column(c)
		if narrow {
			col = col.Width(paneMinWidth)
		} else {
			col = col.FillWidth()
		}
		col.Children(build)
	})
}

// popover shows a floating panel anchored to its trigger while the key is
// open, with a click-catcher behind it — the prototype's fixed inset-0
// backdrop plus its overlay glass (glass-3, 1px white/10, 6px radius).
func (a *app) popover(c *ui.Context, t Theme, key string, trigger ui.Element, at, self ui.Anchor, width float32, body func()) {
	if !a.popoverOpen(key) {
		return
	}
	ui.Overlay(c, func() {
		catcher := ui.Box(c).Absolute().Left(0).Top(0).Right(0).Bottom(0).Focusable()
		if catcher.Clicked() {
			a.togglePopover(key)
		}
		panel := ui.Column(c).
			AttachTo(trigger, at, self).
			Width(width).
			Padding(sp1).
			Radius(radiusMD).
			Border(hairline, whiteA(alphaBorder)).
			Background(col(t.Glass3)).
			Shadow(0, 25, 50, -12, col(RGBA{0, 0, 0, 0.25})).
			Clip().
			Focusable()
		panel.Children(body)
	})
}
