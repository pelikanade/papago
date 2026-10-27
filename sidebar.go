package main

import "github.com/egoist/mygo/ui"

// Sidebar: the Project row IS its orchestrator — hollow ring dot, project path
// right; each Workspace row IS its worker (1:1) — filled status dot + workspace
// name left, PR/CI right. Project blocks are separated by a vertical gap only;
// workspace rows carry tree connectors aligned with the caret, ending in an
// elbow on the last row. Subagents are ephemeral: they never appear here, the
// status bar owns them. Mirrors prototype/src/components/Sidebar.tsx.

func (a *app) sidebar(c *ui.Context) {
	t := a.theme()
	if a.railMode(c) {
		a.sidebarRail(c, t)
		return
	}
	ui.Column(c).
		Width(w72).
		Shrink(0).
		Background(col(t.Glass1)).
		Clip().
		Draw(func(p *ui.Painter, r ui.Rect) {
			// border-r border-white/10
			p.Fill(ui.Rect{X: r.X + r.W - hairline, Y: r.Y, W: hairline, H: r.H}, whiteA(alphaBorder), 0)
		}).
		Children(func() {
			a.sidebarHeader(c, t)
			a.sidebarTree(c, t)
			a.sidebarFooter(c, t)
		})
}

func (a *app) sidebarHeader(c *ui.Context, t Theme) {
	ui.Row(c).
		Height(h8).
		PaddingX(sp3).
		Gap(sp2).
		AlignItems(ui.Center).
		Draw(func(p *ui.Painter, r ui.Rect) {
			p.Fill(ui.Rect{X: r.X, Y: r.Y + r.H - hairline, W: r.W, H: hairline}, whiteA(alphaBorder), 0)
		}).
		Children(func() {
			label(c, "papago", textSM, weightSemiBold, col(t.Ink))
			label(c, "prototype", textXS, weightRegular, col(t.Mut).Alpha(0.6))
			k := label(c, "⌘K", textXS, weightRegular, col(t.Mut))
			k = k.Cursor(ui.CursorPointer).Focusable().Tooltip("palette")
			if k.Hovered() {
				k = k.TextColor(col(t.Ink))
			}
			if k.Clicked() {
				a.setPaletteOpen(true)
			}
			ui.Spacer(c)
			iconButton(c, icCaretLeft, "collapse sidebar", a.toggleSidebar)
		})
}

func (a *app) sidebarTree(c *ui.Context, t Theme) {
	tree := ui.Scroll(c).Grow(1).PaddingY(sp15).Clip().TrackScroll(a.ui.sidebarScroll)
	attachScrollbar(c, t, tree, a.ui.sidebarScroll)
	tree.Children(func() {
		for i, p := range a.fleet.Projects {
			top := float32(0)
			if i > 0 {
				top = sp4 // the block gap: a gap only, no divider
			}
			ui.Box(c).WidthPercent(100).Margin(top, 0, 0, 0).Children(func() { a.projectBlock(c, t, p) })
		}
	})
}

func (a *app) projectBlock(c *ui.Context, t Theme, p *Project) {
	o := p.Orchestrator
	open := a.treeOpen(p.ID)
	a.treeRow(c, t, treeRow{
		indent:   0,
		selected: a.selection.Kind == selAgent && a.selection.ID == o.ID,
		label:    p.Name,
		onClick:  func() { a.selectAgent(o.ID) },
		onCaret:  func() { a.togglePopover("tree:" + p.ID) },
		open:     open,
		content: func() {
			dot(c, t, o.Status, true)
			label(c, p.Name, textXS, weightMedium, col(t.Ink))
			ui.Spacer(c)
			label(c, p.Path, textXS, weightRegular, col(t.Mut).Alpha(0.5))
		},
	})
	if !open {
		return
	}
	var visible []*Workspace
	for _, w := range p.Workspaces {
		if !w.Worker.Archived {
			visible = append(visible, w)
		}
	}
	for i, w := range visible {
		w := w
		last := i == len(visible)-1
		a.treeRow(c, t, treeRow{
			indent:   1,
			selected: a.selection.Kind == selAgent && a.selection.ID == w.Worker.ID,
			label:    w.Name,
			title:    w.Name + " · " + w.Branch + " · " + w.Worker.Title + " · " + w.Path,
			onClick:  func() { a.selectAgent(w.Worker.ID) },
			last:     last,
			content: func() {
				dot(c, t, w.Worker.Status, false)
				label(c, w.Name, textXS, weightRegular, col(t.Ink))
				ui.Spacer(c)
				if w.PR > 0 {
					ui.Row(c).Gap(sp1).AlignItems(ui.Center).Children(func() {
						glyph(c, icFork, textXS, col(t.Mut).Alpha(0.6))
						label(c, itoa(w.PR), textXS, weightRegular, col(t.Mut).Alpha(0.6))
						if ci, ciCol := ciText(t, w.CI); ci != "" {
							label(c, "·", textXS, weightRegular, col(t.Mut).Alpha(0.4))
							glyph(c, ciGlyph(w.CI), textXS, ciCol)
							label(c, ci, textXS, weightRegular, ciCol)
						}
					})
				}
			},
		})
	}
}

func ciGlyph(ci string) string {
	switch ci {
	case "passed":
		return icCheckCir
	case "failed":
		return icXCir
	case "running":
		return icSpinner
	}
	return ""
}

func ciText(t Theme, ci string) (string, ui.Color) {
	switch ci {
	case "passed":
		return "passed", col(t.Ok)
	case "failed":
		return "failed", col(t.Err)
	case "running":
		return "running", col(t.Acc)
	}
	return "", ui.Color{}
}

// treeRow is one row of the sidebar tree. A row with `last` set draws the
// connector's elbow instead of a full-height rule.
type treeRow struct {
	indent   int
	selected bool
	label    string // accessibility label / Tester handle
	title    string // tooltip
	onClick  func()
	onCaret  func()
	open     bool
	last     bool
	hasConn  bool
	content  func()
}

func (a *app) treeRow(c *ui.Context, t Theme, r treeRow) {
	pads := [4]float32{sp2, sp6, sp10, sp14}
	row := ui.Row(c).
		WidthPercent(100).
		PaddingY(rowPadXS).
		AlignItems(ui.Center).
		Gap(sp2).
		PaddingStart(pads[r.indent]).
		PaddingEnd(sp2).
		Label(r.label).
		Cursor(ui.CursorPointer).
		Focusable()
	if r.title != "" {
		row = row.Tooltip(r.title)
	}
	if r.selected {
		row = row.Background(whiteA(alphaSelected))
	} else if row.Hovered() {
		row = row.Background(whiteA(alphaHover))
	}
	if r.onCaret != nil || r.hasConn {
		// Drawn after the row's own background so the selected/hover fill
		// never covers the caret or the connector.
		row = row.Draw(func(p *ui.Painter, box ui.Rect) {
			if r.hasConn {
				x := box.X + treeCaretCenter
				mid := box.Y + box.H/2
				if r.last {
					p.Fill(ui.Rect{X: x, Y: box.Y, W: hairline, H: box.H / 2}, whiteA(alphaBorder), 0)
				} else {
					p.Fill(ui.Rect{X: x, Y: box.Y, W: hairline, H: box.H}, whiteA(alphaBorder), 0)
				}
				if r.last {
					p.Fill(ui.Rect{X: x, Y: mid, W: sp35, H: hairline}, whiteA(alphaBorder), 0)
				}
			}
		})
	}
	row.Children(func() {
		if r.onCaret != nil {
			caret := glyph(c, caretGlyph(r.open), textXS, col(t.Mut).Alpha(0.6))
			caret = caret.Cursor(ui.CursorPointer)
			if caret.Hovered() {
				caret = caret.TextColor(col(t.Ink))
			}
			if caret.Clicked() {
				r.onCaret()
			}
		}
		r.content()
	})
	if row.Clicked() && r.onClick != nil {
		r.onClick()
	}
}

func caretGlyph(open bool) string {
	if open {
		return icCaretDown
	}
	return icCaretRight
}

// treeOpen mirrors `openP["tree:<id>"] ?? true`: trees start expanded.
func (a *app) treeOpen(id string) bool {
	v, ok := a.ui.popovers["tree:"+id]
	return !ok || v
}

func (a *app) sidebarFooter(c *ui.Context, t Theme) {
	ui.Row(c).
		Height(h7).
		PaddingX(sp15).
		AlignItems(ui.Center).
		// The footer is the left end of the same bottom chrome strip as the
		// composer's status bar: the chrome glass, not the sidebar's tint, so
		// the two never read as different opacities.
		Background(col(t.Glass2)).
		Draw(func(p *ui.Painter, r ui.Rect) {
			p.Fill(ui.Rect{X: r.X, Y: r.Y, W: r.W, H: hairline}, whiteA(alphaBorder), 0)
		}).
		Children(func() {
			add := ui.Row(c).
				Height(h7).
				PaddingStart(sp15).
				PaddingEnd(sp2).
				Gap(sp15).
				AlignItems(ui.Center).
				Radius(radiusSM).
				Cursor(ui.CursorPointer).
				Tooltip("add project / workspace").
				Label("add project / workspace").
				Focusable()
			if add.Hovered() {
				add = add.Background(whiteA(alphaHover))
			}
			add.Children(func() {
				glyph(c, icPlus, textXS, col(t.Mut))
				label(c, "Project", textXS, weightRegular, col(t.Mut))
			})
			if add.Clicked() {
				a.togglePopover("sel:project")
			}
			ui.Spacer(c)
			settings := iconButton(c, icGear, "settings", func() { a.togglePopover("footer:settings") })
			a.popover(c, t, "footer:settings", settings, ui.AnchorTopEnd, ui.AnchorBottomEnd, rem(15), func() {
				item := ui.Row(c).
					Height(h6).
					PaddingX(sp2).
					AlignItems(ui.Center).
					Radius(radiusSM).
					Cursor(ui.CursorPointer).
					Focusable()
				if item.Hovered() {
					item = item.Background(whiteA(alphaHoverAlt))
				}
				item.Children(func() {
					label(c, "Theme: "+Palettes[a.themeID].Scheme+" → next", textXS, weightRegular, col(t.Ink))
				})
				if item.Clicked() {
					a.cycleTheme()
					a.togglePopover("footer:settings")
				}
				muted(c, t, "daemon · local")
				mono(c, t, "~/.papago/config.json")
			})
			a.popover(c, t, "sel:project", add, ui.AnchorTopStart, ui.AnchorBottomStart, rem(16), func() {
				a.createProjectForm(c, t)
			})
		})
}

// createProjectForm is the footer's "+ Project" panel: a path input and the
// create button (prototype: directory path or name → Create Project).
func (a *app) createProjectForm(c *ui.Context, t Theme) {
	key := "sel:project"
	ui.Column(c).Gap(sp15).Children(func() {
		in := ui.TextInputBase(c, a.draftPtr(key)).
			Placeholder("directory path or name").
			Font(fontSans+", "+fontIcons).
			FontSize(textXS).
			Height(h7).
			PaddingX(sp2).
			Radius(radiusSM).
			Border(hairline, col(t.Line)).
			Background(ui.Transparent).
			WidthPercent(100)
		in.OnSubmit(func() { a.createProjectFromDraft(key) })
		commit := ui.Row(c).
			Height(h7).
			WidthPercent(100).
			AlignItems(ui.Center).
			Justify(ui.Center).
			Radius(radiusSM).
			Background(col(t.Acc)).
			Cursor(ui.CursorPointer).
			Label("Create Project").
			Focusable()
		commit.Children(func() { label(c, "Create Project", textXS, weightRegular, col(t.Bg)) })
		if commit.Clicked() {
			a.createProjectFromDraft(key)
		}
	})
}

func (a *app) createProjectFromDraft(key string) {
	name := a.drafts(key)
	if name == "" {
		return
	}
	a.openDirectory(name)
	a.setDraft(key, "")
	a.togglePopover(key)
}

// railMode reports whether the sidebar takes its rail form: collapsed by the
// user, or the window too narrow for the wide panel (the design has no rail
// breakpoint of its own — a web page never shrinks; a tiled window does).
func (a *app) railMode(c *ui.Context) bool {
	if a.ui.sidebarCollapsed {
		return true
	}
	winW, _ := c.Size()
	return winW < railBreakpoint
}

// surfaceWidth is the width the window content is laid out at.
func (a *app) surfaceWidth(c *ui.Context) float32 {
	winW, _ := c.Size()
	return winW
}

// paneWidth is the pane's width this frame: the window minus the sidebar.
func (a *app) paneWidth(c *ui.Context) float32 {
	return a.surfaceWidth(c) - a.sidebarWidth(c)
}

// sidebarWidth is the horizontal space the sidebar takes this frame: the wide
// panel or the rail. The status bar budgets its segments against the rest.
func (a *app) sidebarWidth(c *ui.Context) float32 {
	if a.railMode(c) {
		return w8
	}
	return w72
}

// sidebarRail is the collapsed sidebar: one status dot per agent, in urgency
// order, between the two strips that mirror the pane header and status bar.
func (a *app) sidebarRail(c *ui.Context, t Theme) {
	ui.Column(c).
		Width(w8).
		Shrink(0).
		Background(col(t.Glass1)).
		Draw(func(p *ui.Painter, r ui.Rect) {
			p.Fill(ui.Rect{X: r.X + r.W - hairline, Y: r.Y, W: hairline, H: r.H}, whiteA(alphaBorder), 0)
		}).
		Children(func() {
			ui.Row(c).
				Height(h8).
				Justify(ui.Center).
				AlignItems(ui.Center).
				Draw(func(p *ui.Painter, r ui.Rect) {
					p.Fill(ui.Rect{X: r.X, Y: r.Y + r.H - hairline, W: r.W, H: hairline}, whiteA(alphaBorder), 0)
				}).
				Children(func() {
					iconButton(c, icCaretRight, "expand sidebar", a.toggleSidebar)
				})
			ui.Scroll(c).Grow(1).Children(func() {
				ui.Column(c).
					WidthPercent(100).
					AlignItems(ui.Center).
					Gap(sp15).
					PaddingY(sp15).
					Children(func() {
						for _, ag := range a.agentsByUrgency() {
							ag := ag
							title := a.railTitle(ag)
							b := ui.Row(c).
								Size(w6, w6).
								AlignItems(ui.Center).
								Justify(ui.Center).
								Radius(radiusSM).
								Cursor(ui.CursorPointer).
								Tooltip(title).
								Label(title).
								Focusable()
							if a.selection.Kind == selAgent && a.selection.ID == ag.ID {
								b = b.Background(whiteA(alphaSelected))
							} else if b.Hovered() {
								b = b.Background(whiteA(alphaSelected))
							}
							b.Children(func() { dot(c, t, ag.Status, ag.Role == RoleOrchestrator) })
							if b.Clicked() {
								a.selectAgent(ag.ID)
							}
						}
					})
			})
			ui.Row(c).
				Height(h7).
				Justify(ui.Center).
				AlignItems(ui.Center).
				Draw(func(p *ui.Painter, r ui.Rect) {
					p.Fill(ui.Rect{X: r.X, Y: r.Y, W: r.W, H: hairline}, whiteA(alphaBorder), 0)
				}).
				Children(func() {
					iconButton(c, icPlus, "add project / workspace", func() { a.togglePopover("sel:project") })
				})
		})
}

// railTitle names a rail dot: `<project> · orchestrator` or `<project> / <workspace> · <title>`.
func (a *app) railTitle(ag *Agent) string {
	for _, p := range a.fleet.Projects {
		if p.Orchestrator.ID == ag.ID {
			return p.Name + " · orchestrator"
		}
		for _, w := range p.Workspaces {
			if w.Worker.ID == ag.ID {
				return p.Name + " / " + w.Name + " · " + ag.Title
			}
		}
	}
	return ag.Title
}

// agentsByUrgency sorts every agent by the dot's urgency (needs-you first).
func (a *app) agentsByUrgency() []*Agent {
	ags := a.allAgents()
	for i := 1; i < len(ags); i++ {
		for j := i; j > 0 && Urgency[ags[j].Status] < Urgency[ags[j-1].Status]; j-- {
			ags[j], ags[j-1] = ags[j-1], ags[j]
		}
	}
	return ags
}

func muted(c *ui.Context, t Theme, s string) {
	ui.Box(c).PaddingX(sp2).PaddingY(sp15).Children(func() {
		label(c, s, textXS, weightRegular, col(t.Mut).Alpha(0.7))
	})
}

func mono(c *ui.Context, t Theme, s string) {
	ui.Box(c).PaddingX(sp2).PaddingY(sp15).Children(func() {
		ui.Text(c, s).Font(fontMono).FontSize(textXS).TextColor(col(t.Mut).Alpha(0.7)).SingleLine()
	})
}
