package main

import (
	"strings"

	"github.com/egoist/mygo/ui"
)

// Command palette (⌘K / Ctrl+K): jumps + actions. The only shortcut surface.
// Mirrors prototype/src/components/Palette.tsx.

type paletteRow struct {
	id    string
	label string
	hint  string
	icon  string
	run   func()
}

func (a *app) palette(c *ui.Context) {
	if !a.ui.paletteOpen {
		return
	}
	t := a.theme()
	winW, winH := c.Size()
	width := min(float32(640), winW-2*sp4) // w-160, clamped to the window
	ui.Overlay(c, func() {
		backdrop := ui.Box(c).
			Absolute().
			Left(0).Top(0).
			Size(winW, winH).
			Background(col(RGBA{0, 0, 0, 0.40})).
			Focusable()
		if backdrop.Clicked() {
			a.setPaletteOpen(false)
		}
		panel := ui.Column(c).
			Absolute().
			Left((winW-width)/2).
			Top(rem(4)).
			Width(width).
			Radius(radiusLG).
			Border(hairline, whiteA(alphaBorder)).
			Background(col(t.Glass3)).
			Clip().
			Shadow(0, 25, 50, -12, col(RGBA{0, 0, 0, 0.25})).
			Focusable()
		panel.Children(func() {
			query := ui.TextInputBase(c, a.ui.paletteBuf).
				Placeholder("jump to Agent / Workspace · run action…").
				Font(fontSans + ", " + fontIcons).
				FontSize(textSM).
				Height(h9).
				PaddingX(sp3).
				Background(ui.Transparent).
				WidthPercent(100)
			_ = query
			// the input's edit is visible to the statements after it
			a.setPaletteQuery(*a.ui.paletteBuf)
			rows := a.paletteRows()
			filtered := rows[:0:0]
			q := strings.ToLower(a.ui.paletteQuery)
			for _, r := range rows {
				if strings.Contains(strings.ToLower(r.label), q) {
					filtered = append(filtered, r)
				}
			}
			ui.Column(c).
				Fill().
				MaxHeight(min(float32(416), winH-2*rem(4))). // max-h-105
				Clip().
				Draw(func(p *ui.Painter, r ui.Rect) {
					p.Fill(ui.Rect{X: r.X, Y: r.Y, W: r.W, H: hairline}, whiteA(alphaBorder), 0)
				}).
				Children(func() {
					sc := ui.Scroll(c).Grow(1).TrackScroll(a.ui.paletteScroll)
					attachScrollbar(c, t, sc, a.ui.paletteScroll)
					sc.Children(func() {
						ui.Column(c).FillWidth().PaddingY(sp1).Children(func() {
							for _, row := range filtered {
								row := row
								item := ui.Row(c).
									WidthPercent(100).
									PaddingX(sp3).
									PaddingY(rowPadXS).
									Gap(sp2).
									AlignItems(ui.Center).
									Cursor(ui.CursorPointer).
									Label(row.label).
									Focusable()
								if item.Hovered() {
									item = item.Background(whiteA(alphaHoverAlt))
								}
								item.Children(func() {
									if row.icon != "" {
										glyph(c, row.icon, textXS, col(t.Mut).Alpha(0.6))
									}
									label(c, row.label, textSM, weightRegular, col(t.Ink)).SingleLine()
									if row.hint != "" {
										ui.Spacer(c)
										label(c, row.hint, textXS, weightRegular, col(t.Mut).Alpha(0.6))
									}
								})
								if item.Clicked() {
									row.run()
									a.setPaletteOpen(false)
								}
							}
							if len(filtered) == 0 {
								ui.Box(c).PaddingX(sp3).PaddingY(sp2).Children(func() {
									label(c, "no match", textSM, weightRegular, col(t.Mut).Alpha(0.6))
								})
							}
						})
					})
				})
		})
	})
}

// paletteRows is the action list, rebuilt each frame from state: pending asks
// first, then everything to jump to, then the actions.
func (a *app) paletteRows() []paletteRow {
	var rows []paletteRow
	for _, ask := range a.pendingAsks() {
		ask := ask
		rows = append(rows, paletteRow{
			id:    "ask:" + ask.AgentID + ":" + itoa(ask.Index),
			label: "Next Ask — " + ask.Question,
			hint:  "jump",
			run:   func() { a.selectAgent(ask.AgentID) },
		})
	}
	for _, p := range a.fleet.Projects {
		p := p
		rows = append(rows, paletteRow{
			id:    "p:" + p.ID,
			label: p.Name,
			hint:  "project",
			icon:  icArrowRight,
			run:   func() { a.selectProject(p.ID) },
		})
		o := p.Orchestrator
		rows = append(rows, paletteRow{
			id:    "o:" + o.ID,
			label: p.Name + " / orchestrator",
			hint:  string(o.Status),
			icon:  icSitemap,
			run:   func() { a.selectAgent(o.ID) },
		})
		for _, w := range p.Workspaces {
			w := w
			rows = append(rows, paletteRow{
				id:    "w:" + w.ID,
				label: p.Name + " / " + w.Name,
				hint:  w.Isolation,
				icon:  icArrowRight,
				run:   func() { a.selectWorkspace(w.ID) },
			})
			ag := w.Worker
			rows = append(rows, paletteRow{
				id:    "a:" + ag.ID,
				label: ag.Title,
				hint:  string(ag.Status),
				icon:  icArrowRight,
				run:   func() { a.selectAgent(ag.ID) },
			})
		}
		rows = append(rows, paletteRow{
			id:    "prune:" + p.ID,
			label: "Prune worktrees — " + p.Name,
			hint:  "destructive",
			run:   func() { a.pruneWorktrees(p.ID) },
		})
	}
	rows = append(rows,
		paletteRow{id: "hub", label: "Open Agent Hub", run: func() { a.setHubOpen(true) }},
		paletteRow{
			id:    "theme",
			label: "Theme: " + Palettes[a.themeID].Scheme + " → next",
			hint:  "base16",
			run:   func() { a.cycleTheme() },
		},
		paletteRow{id: "open", label: "Open directory…", run: func() { a.openDirectory("newrepo") }},
		paletteRow{
			id:    "diag",
			label: "Toggle omp diagnostics view",
			hint:  "debug",
			run:   func() { a.setOmpMissing(!a.ompMissing) },
		},
	)
	return rows
}
