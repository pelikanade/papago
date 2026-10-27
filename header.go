package main

import "github.com/egoist/mygo/ui"

// Shared pane header: breadcrumbs · title · sub · window controls. The window
// is frameless, so the header is also the drag area (double-click maximizes)
// and its maximize button is a real maximize button.
// Mirrors prototype/src/components/Header.tsx.

func (a *app) header(c *ui.Context, title string, sub string, crumbs func()) {
	t := a.theme()
	w := a.surfaceWidth(c)
	showSub := sub != "" && w >= 900
	showCrumbs := crumbs != nil && w >= 640
	ui.Row(c).
		Height(h8).
		Shrink(0).
		PaddingX(sp4).
		Gap(sp3).
		AlignItems(ui.Center).
		DragWindow().
		Draw(func(p *ui.Painter, r ui.Rect) {
			p.Fill(ui.Rect{X: r.X, Y: r.Y + r.H - hairline, W: r.W, H: hairline}, whiteA(alphaBorder), 0)
		}).
		Children(func() {
			if showCrumbs {
				ui.Row(c).Gap(sp1).AlignItems(ui.Center).Shrink(0).Children(func() {
					crumbs()
				})
			}
			label(c, title, textXS, weightSemiBold, col(t.Ink)).SingleLine().Shrink(1)
			if showSub {
				label(c, sub, textXS, weightRegular, col(t.Mut)).SingleLine().Shrink(1)
			}
			ui.Spacer(c)
			ui.Row(c).Gap(sp1).AlignItems(ui.Center).Shrink(0).Children(func() {
				a.windowButtons(c)
			})
		})
}

// windowButtons builds the three window controls (minimize, maximize/restore,
// close) that end the pane header.
func (a *app) windowButtons(c *ui.Context) {
	iconButton(c, icWinMin, "minimize", func() { a.setWin(winMinimized) })
	max := ui.Row(c).
		Size(h7, h7).
		AlignItems(ui.Center).
		Justify(ui.Center).
		Radius(radiusSM).
		Cursor(ui.CursorPointer).
		Tooltip(maximizeTitle(a.win)).
		MaximizeButton()
	if max.Hovered() {
		max = max.Background(whiteA(alphaHoverAlt))
	}
	max.Draw(func(p *ui.Painter, r ui.Rect) {
		g := icWinMax
		if a.win == winMaximized {
			g = icWinRestore
		}
		w, h := p.MeasureText(0, ui.Span{Text: g, Font: fontIcons, Size: textSM})
		p.Text(r.X+(r.W-w)/2, r.Y+(r.H-h)/2, g, textSM, c.Theme().TextMuted)
	})
	iconButton(c, icWinClose, "close", func() { a.setWin(winClosed) })
}

func maximizeTitle(w winState) string {
	if w == winMaximized {
		return "restore"
	}
	return "maximize"
}

// crumb renders one breadcrumb segment and the ▸ that follows it.
func crumb(c *ui.Context, t Theme, s string) {
	label(c, s, textXS, weightRegular, col(t.Mut).Alpha(0.6))
	glyph(c, icAngleRight, textXS, col(t.Mut).Alpha(0.6))
}
