package main

import "github.com/egoist/mygo/ui"

// The pane's pages, one per selection. Each page is the pane header, the
// surface itself, and the composer bar along the bottom edge — the shape of
// prototype/src/pages/*.tsx.

// agentPage: one agent's timeline + its composer. Workers sit inside a
// workspace; the orchestrator sits directly under the project.
func (a *app) agentPage(c *ui.Context) {
	t := a.theme()
	p, w, ag := a.projectOf(a.selection.ID)
	if ag == nil {
		a.welcomePage(c)
		return
	}
	title := ag.Title
	if ag.Role == RoleOrchestrator {
		title = "orchestrator"
	}
	a.header(c, title, "", func() {
		crumb(c, t, p.Name)
		if w != nil {
			crumb(c, t, w.Name)
		}
	})
	ui.Column(c).Grow(1).Basis(0).Children(func() {
		a.timeline(c, ag)
		workspaceID := ""
		if w != nil {
			workspaceID = w.ID
		}
		a.composerBar(c, p.ID, workspaceID, ag.ID)
	})
}

// projectPage: the orchestrator row (read/plan/dispatch) + its workspaces.
// Sending from the composer goes to the orchestrator, which creates a
// workspace + worktree + worker for the prompt.
func (a *app) projectPage(c *ui.Context) {
	t := a.theme()
	p := a.project(a.selection.ID)
	if p == nil {
		a.welcomePage(c)
		return
	}
	visible := 0
	for _, w := range p.Workspaces {
		if !w.Worker.Archived {
			visible++
		}
	}
	a.header(c, p.Name, p.Path+" · "+p.Branch+" · "+itoa(visible)+" workspaces", nil)
	ui.Column(c).Grow(1).Basis(0).Children(func() {
		a.scrollContent(c, t, a.pageScroll("project:"+p.ID), func() {
			ui.Column(c).FillWidth().PaddingX(sp4).PaddingY(sp3).Gap(sp2).Selectable().Children(func() {
				o := p.Orchestrator
				cardRow(c, t, func() { a.selectAgent(o.ID) }, "orchestrator", func() {
					dot(c, t, o.Status, true)
					glyph(c, icSitemap, textXS, col(t.Mut).Alpha(0.6))
					label(c, "orchestrator", textSM, weightMedium, col(t.Ink))
					label(c, "read · plan · dispatch — workers own writes", textXS, weightRegular, col(t.Mut))
					ui.Spacer(c)
					label(c, o.Model, textXS, weightRegular, col(t.Mut).Alpha(0.6))
				})
				for _, w := range p.Workspaces {
					if w.Worker.Archived {
						continue
					}
					w := w
					isolation := "local checkout"
					if w.Isolation == "worktree" {
						isolation = "worktree · " + w.Branch
					}
					cardRow(c, t, func() { a.selectWorkspace(w.ID) }, w.Name, func() {
						dot(c, t, w.Worker.Status, false)
						label(c, w.Name, textSM, weightMedium, col(t.Ink))
						label(c, isolation, textXS, weightRegular, col(t.Mut))
						ui.Spacer(c)
						label(c, w.Worker.Title, textXS, weightRegular, col(t.Mut).Alpha(0.6)).SingleLine()
					})
				}
			})
		})
		a.composerBar(c, p.ID, "", "")
	})
}

// workspacePage: the graph (its single worker; subagents are ephemeral) plus
// the worker's composer, so it can be prompted straight from the graph.
func (a *app) workspacePage(c *ui.Context) {
	t := a.theme()
	p, w := a.workspace(a.selection.ID)
	if w == nil {
		a.welcomePage(c)
		return
	}
	isolation := "local checkout"
	if w.Isolation == "worktree" {
		isolation = "worktree · base " + p.Branch
	}
	a.header(c, w.Name, w.Path+" · "+isolation, func() { crumb(c, t, p.Name) })
	ui.Column(c).Grow(1).Basis(0).Children(func() {
		a.scrollContent(c, t, a.pageScroll("workspace:"+w.ID), func() {
			ui.Column(c).FillWidth().PaddingX(sp4).PaddingY(sp3).Selectable().Children(func() {
				graph(c, t, []*Agent{w.Worker}, a.selection.ID, func(id string) { a.selectAgent(id) })
			})
		})
		a.composerBar(c, p.ID, w.ID, w.Worker.ID)
	})
}

// graph draws the workspace's agent nodes (1:1 today) as clickable cards.
func graph(c *ui.Context, t Theme, agents []*Agent, selectedID string, onSelect func(string)) {
	if len(agents) == 0 {
		ui.Box(c).PaddingY(sp6).WidthPercent(100).Children(func() {
			label(c, "no agents here yet — send a prompt below", textSM, weightRegular, col(t.Mut).Alpha(0.6))
		})
		return
	}
	ui.Row(c).Gap(sp4).Children(func() {
		for _, ag := range agents {
			ag := ag
			card := ui.Column(c).
				Width(212).
				PaddingX(sp3).
				PaddingY(sp2).
				Gap(sp1).
				Radius(radiusLG).
				Background(col(t.Glass1)).
				Border(hairline, whiteA(alphaBorder)).
				Cursor(ui.CursorPointer).
				Label("agent " + ag.Title).
				Focusable()
			if selectedID == ag.ID {
				card = card.BorderColor(col(t.Acc))
			} else if card.Hovered() {
				card = card.BorderColor(col(t.Acc).Alpha(0.7))
			}
			card.Children(func() {
				ui.Row(c).Gap(sp2).AlignItems(ui.Center).Children(func() {
					dot(c, t, ag.Status, ag.Role == RoleOrchestrator)
					label(c, ag.Title, textSM, weightRegular, col(t.Ink)).SingleLine().Shrink(1)
					ui.Spacer(c)
					label(c, string(ag.Status), textXS, weightRegular, col(t.Mut).Alpha(0.6))
				})
				label(c, ag.Model, textXS, weightRegular, col(t.Mut).Alpha(0.7)).SingleLine()
			})
			if card.Clicked() {
				onSelect(ag.ID)
			}
		}
	})
}

// welcomePage: no projects yet.
func (a *app) welcomePage(c *ui.Context) {
	t := a.theme()
	ui.Column(c).Fill().AlignItems(ui.Center).Justify(ui.Center).Gap(sp3).Children(func() {
		label(c, "papago", textBase, weightSemiBold, col(t.Ink))
		label(c, "run and review your omp agents — no editor, no terminal, no ceremony", textSM, weightRegular, col(t.Mut))
		b := ui.Row(c).
			Height(h7).
			Margin(sp2, 0, 0, 0).
			PaddingX(sp3).
			Radius(radiusSM).
			Background(col(t.Acc)).
			AlignItems(ui.Center).
			Cursor(ui.CursorPointer).
			Label("Open directory…").
			Focusable()
		b.Children(func() { label(c, "Open directory…", textSM, weightRegular, col(t.Bg)) })
		if b.Clicked() {
			a.openDirectory("myrepo")
		}
		label(c, "git root becomes a Project; a local Workspace appears with it", textXS, weightRegular, col(t.Mut).Alpha(0.6))
	})
}

// diagnosticsPage: omp missing from PATH (the switch the palette toggles).
func (a *app) diagnosticsPage(c *ui.Context) {
	t := a.theme()
	ui.Column(c).Fill().AlignItems(ui.Center).Justify(ui.Center).Gap(sp3).Children(func() {
		label(c, "omp not found on PATH", textSM, weightSemiBold, col(t.Warn))
		pre := ui.Box(c).
			Padding(sp3).
			Radius(radiusMD).
			Border(hairline, col(t.Line)).
			Background(col(t.Panel))
		pre.Children(func() {
			ui.Text(c, "curl -fsSL https://omp.sh/install | sh\n# or: bun install -g @oh-my-pi/pi-coding-agent").
				Font(fontMono).
				FontSize(textXS).
				TextColor(col(t.Mut))
		})
		label(c, "or set an absolute path in ~/.papago/config.json → omp.path", textXS, weightRegular, col(t.Mut))
		b := ui.Row(c).
			Height(h7).
			PaddingX(sp3).
			Radius(radiusSM).
			Border(hairline, col(t.Line)).
			AlignItems(ui.Center).
			Cursor(ui.CursorPointer).
			Label("Retry detection").
			Focusable()
		if b.Hovered() {
			b = b.BorderColor(col(t.Acc))
		}
		b.Children(func() { label(c, "Retry detection", textSM, weightRegular, col(t.Mut)) })
		if b.Clicked() {
			a.setOmpMissing(false)
		}
	})
}

// cardRow is the project page's list row: hairline card, hover accent.
func cardRow(c *ui.Context, t Theme, onClick func(), labelText string, content func()) {
	row := ui.Row(c).
		WidthPercent(100).
		PaddingX(sp3).
		PaddingY(cardPadY).
		Gap(sp2).
		Radius(radiusMD).
		Border(hairline, col(t.Line)).
		AlignItems(ui.Center).
		Cursor(ui.CursorPointer).
		Label(labelText).
		Focusable()
	if row.Hovered() {
		row = row.BorderColor(col(t.Mut).Alpha(0.6))
	}
	row.Children(content)
	if row.Clicked() && onClick != nil {
		onClick()
	}
}
