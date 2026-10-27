package main

import "github.com/egoist/mygo/ui"

// Agent Hub: the floating panel that groups every live subagent across agents
// (the prototype's fixed panel at the top right). Read-only rows + jump to the
// owning agent. Mirrors prototype/src/components/Hub.tsx.

// windowW is the window's width in DIPs.
func windowW(c *ui.Context) float32 {
	w, _ := c.Size()
	return w
}

func (a *app) hub(c *ui.Context) {
	if !a.ui.hubOpen {
		return
	}
	t := a.theme()
	subs := a.liveSubs()
	live := 0
	for _, s := range subs {
		if s.Sub.SubStatus == StatusRunning {
			live++
		}
	}
	ui.Overlay(c, func() {
		panel := ui.Column(c).
			Absolute().
			Right(sp6).
			Top(h10).
			Width(min(float32(520), windowW(c)-2*sp6)). // w-130
			MaxHeightPercent(70).
			Radius(radiusLG).
			Border(hairline, whiteA(alphaBorder)).
			Background(col(t.Glass3)).
			Clip().
			Shadow(0, 25, 50, -12, col(RGBA{0, 0, 0, 0.25})).
			Focusable()
		panel.Children(func() {
			ui.Row(c).
				Height(h10).
				PaddingX(sp3).
				Gap(sp2).
				AlignItems(ui.Center).
				Draw(func(p *ui.Painter, r ui.Rect) {
					p.Fill(ui.Rect{X: r.X, Y: r.Y + r.H - hairline, W: r.W, H: hairline}, whiteA(alphaBorder), 0)
				}).
				Children(func() {
					label(c, "Agent Hub", textSM, weightSemiBold, col(t.Ink))
					label(c, itoa(live)+" live", textXS, weightRegular, col(t.Mut).Alpha(0.6))
					ui.Spacer(c)
					iconButton(c, icX, "close hub", func() { a.setHubOpen(false) })
				})
			sc := ui.Scroll(c).Grow(1).TrackScroll(a.ui.hubScroll)
			attachScrollbar(c, t, sc, a.ui.hubScroll)
			sc.Children(func() {
				ui.Column(c).FillWidth().Padding(sp2).Gap(sp2).Children(func() {
					if len(subs) == 0 {
						ui.Box(c).Padding(sp3).Children(func() {
							label(c, "no live subagents", textSM, weightRegular, col(t.Mut).Alpha(0.6))
						})
						return
					}
					lastTitle := ""
					for _, s := range subs {
						if s.AgentTitle != lastTitle {
							lastTitle = s.AgentTitle
							ui.Box(c).PaddingX(sp1).PaddingY(2).Children(func() {
								label(c, s.AgentTitle, textXS, weightRegular, col(t.Mut).Alpha(0.7)).SingleLine()
							})
						}
						s := s
						row := ui.Column(c).
							PaddingX(sp2).
							PaddingY(rowPadXS).
							Margin(0, 0, sp1, 0).
							Gap(sp1).
							Radius(radiusMD).
							Border(hairline, whiteA(alphaBorder)).
							Background(col(t.Glass1)).
							Clip()
						row.Children(func() {
							ui.Row(c).Gap(sp2).AlignItems(ui.Center).Children(func() {
								dot(c, t, s.Sub.SubStatus, false)
								label(c, s.Sub.SubName, textSM, weightRegular, col(t.Ink)).SingleLine().Shrink(1)
								label(c, s.Sub.SubModel+" · "+s.Sub.SubTokens, textXS, weightRegular, col(t.Mut).Alpha(0.7)).SingleLine()
								ui.Spacer(c)
								jump := glyph(c, icArrowRight, textXS, col(t.Mut).Alpha(0.6))
								jump = jump.Cursor(ui.CursorPointer).Tooltip("open agent")
								if jump.Clicked() {
									a.selectAgent(s.AgentID)
								}
							})
							ui.Box(c).PaddingStart(sp4).Children(func() {
								label(c, s.Sub.SubTool, textXS, weightRegular, col(t.Mut).Alpha(0.7)).SingleLine()
							})
						})
					}
				})
			})
		})
	})
}
