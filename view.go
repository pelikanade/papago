package main

import (
	"time"
)

import "github.com/egoist/mygo/ui"

// view builds one frame of the window from app state: the design's frame
// (p-3), the window surface (rounded, glass, hairline) and, inside it, the
// sidebar and the pane. It is a pure function — it changes state only through
// the app's named transitions, never performs I/O, and may run several times
// for one event.

func (a *app) view(c *ui.Context) {
	t := a.theme()
	c.SetTheme(uiTheme(t))

	// The palette's shortcut is window-wide (⌘K / Ctrl+K).
	c.OnShortcut(ui.Cmd, ui.KeyK, func() { a.setPaletteOpen(!a.ui.paletteOpen) })
	c.OnShortcut(0, ui.KeyEscape, func() {
		if a.ui.paletteOpen {
			a.setPaletteOpen(false)
		}
	})

	switch a.win {
	case winClosed:
		a.frameButton(c, "reopen papago")
		return
	case winMinimized:
		a.frameButton(c, icWinRestore+"  papago — minimized")
		return
	}

	// The window is the surface: the app paints edge to edge and draws no
	// window chrome of its own — no corner radius, no outer border, no shadow.
	// The shape of the window belongs to the desktop: macOS and Windows 11
	// round it, Hyprland's rounding (0 here) and GNOME leave it square. Only
	// content keeps its small radii.
	ui.Box(c).Fill().Background(col(t.Glass0)).Children(func() { a.surface(c) })

	a.hub(c)
	a.palette(c)

	// A pulsing dot keeps the frame loop alive at ~30fps (CSS's dot-pulse).
	if a.anyRunning() {
		c.After(33 * time.Millisecond)
	}
	// While anything streams, the view asks for another frame in 100ms and
	// advances the mock clock — the prototype's setInterval pair.
	if a.anyStreaming() {
		c.After(100 * time.Millisecond)
		a.clockTick(c.Now())
	}
}

// surface is the window content, edge to edge: the sidebar and the pane. It
// draws no window chrome of its own; the desktop decides the corners.
func (a *app) surface(c *ui.Context) {
	ui.Row(c).
		Grow(1).
		AlignItems(ui.Stretch).
		Clip().
		Children(func() {
			a.sidebar(c)
			// flex-1 min-w-0 flex flex-col
			ui.Column(c).Grow(1).Basis(0).Shrink(1).Children(func() { a.pane(c) })
		})
}

// frameButton is the minimized/closed stand-in: the prototype's small glass
// button floating in the middle of an otherwise empty window.
func (a *app) frameButton(c *ui.Context, text string) {
	t := a.theme()
	ui.Box(c).Fill().AlignItems(ui.Center).Justify(ui.Center).Children(func() {
		b := ui.Row(c).
			Height(h9).
			PaddingX(sp4).
			Radius(radiusLG).
			Border(hairline, whiteA(alphaBorder)).
			Background(col(t.Glass3)).
			AlignItems(ui.Center).
			Cursor(ui.CursorPointer).
			Focusable()
		if b.Hovered() {
			b = b.BorderColor(col(t.Acc))
		}
		b.Children(func() { label(c, text, textSM, weightRegular, col(t.Ink)) })
		if b.Clicked() {
			a.setWin(winNormal)
		}
	})
}

// pane is the right-hand side: the pane header, the selected surface, and the
// composer bar along its bottom edge.
func (a *app) pane(c *ui.Context) {
	if a.ompMissing {
		a.diagnosticsPage(c)
		return
	}
	switch a.selection.Kind {
	case selProject:
		a.projectPage(c)
	case selWorkspace:
		a.workspacePage(c)
	case selAgent:
		a.agentPage(c)
	default:
		a.welcomePage(c)
	}
}
