package main

import (
	"sort"

	"github.com/egoist/mygo/ui"
)

// The status bar is the pane's bottom edge, aligned with the sidebar footer.
// It never clips: when the window is narrow, whole segments drop in priority
// order (send and abort never drop), and the rest keep the design's order.
// Mirrors prototype/src/components/ComposerBar.tsx.

// statusSeg is one segment before layout: its identity, how strongly it wants
// to stay, its measured width, and how it draws itself.
type statusSeg struct {
	key   string
	prio  int
	width float32
	draw  func(first bool)
}

// seg is a status-bar segment: h-7, px-3, a left hairline, nowrap content.
// The first segment drawn has no left hairline (Tailwind's first:border-l-0).
func seg(c *ui.Context, tone ui.Color, title string, first bool, onClick func(), content func()) ui.Element {
	s := ui.Row(c).
		Height(h7).
		PaddingX(sp3).
		Gap(sp15).
		AlignItems(ui.Center).
		NoWrap()
	if title != "" {
		s = s.Tooltip(title)
	}
	if onClick != nil {
		s = s.Cursor(ui.CursorPointer).Focusable()
		if s.Hovered() {
			s = s.Background(whiteA(alphaHover))
		}
	}
	if !first {
		s = s.Draw(func(p *ui.Painter, r ui.Rect) {
			p.Fill(ui.Rect{X: r.X, Y: r.Y, W: hairline, H: r.H}, whiteA(0.10), 0)
		})
	}
	s.Children(content)
	if onClick != nil && s.Clicked() {
		onClick()
	}
	return s
}

// segWidth is a segment's natural width: padding, hairline, icon, gap, label,
// plus `extra` for segments with a bar or another fixed part.
func segWidth(c *ui.Context, icon, text string, extra float32) float32 {
	w := float32(2*sp3 + hairline)
	if icon != "" {
		iw, _ := c.MeasureText(0, ui.Span{Text: icon, Font: fontIcons, Size: textXS})
		w += iw + sp15
	}
	if text != "" {
		tw, _ := c.MeasureText(0, ui.Span{Text: text, Font: fontSans + ", " + fontIcons, Size: textXS})
		w += tw
	}
	return w + extra
}

// statusBar builds the pane's bottom edge.
func (a *app) statusBar(c *ui.Context, t Theme, projectID, workspaceID string, ag *Agent) {
	ch := a.chips(projectID)
	p := a.project(projectID)
	var w *Workspace
	if workspaceID != "" {
		_, w = a.workspace(workspaceID)
	}

	var segs []statusSeg
	add := func(key string, prio int, icon, text string, extra float32, draw func(first bool)) {
		segs = append(segs, statusSeg{key: key, prio: prio, width: segWidth(c, icon, text, extra), draw: draw})
	}

	// model
	add("model", 9, icModel, ch.Model, 0, func(first bool) {
		trig := seg(c, col(t.Acc).Alpha(0.8), "model (per-Project last-used)", first, func() { a.togglePopover("chip:model:" + projectID) }, func() {
			glyph(c, icModel, textXS, col(t.Acc).Alpha(0.8))
			label(c, ch.Model, textXS, weightRegular, col(t.Mut))
		})
		a.popover(c, t, "chip:model:"+projectID, trig, ui.AnchorTopStart, ui.AnchorBottomStart, rem(18), func() {
			for _, m := range Models {
				a.menuItem(c, t, m == ch.Model, m, func() {
					a.setChip(projectID, "model", m)
					a.togglePopover("chip:model:" + projectID)
				})
			}
		})
	})
	// thinking
	add("thinking", 3, icFilter, ThinkLabels[ch.Thinking], 0, func(first bool) {
		trig := seg(c, col(t.Mut), "thinking level", first, func() { a.togglePopover("chip:th:" + projectID) }, func() {
			glyph(c, icFilter, textXS, col(t.Mut))
			label(c, ThinkLabels[ch.Thinking], textXS, weightRegular, col(t.Mut))
		})
		a.popover(c, t, "chip:th:"+projectID, trig, ui.AnchorTopStart, ui.AnchorBottomStart, rem(12), func() {
			for _, lvl := range ThinkLevels {
				a.menuItem(c, t, lvl == ch.Thinking, ThinkLabels[lvl], func() {
					a.setChip(projectID, "thinking", lvl)
					a.togglePopover("chip:th:" + projectID)
				})
			}
		})
	})
	// approval — the orchestrator is read + plan only, so it shows a fixed
	// label instead of the menu
	if ag != nil && ag.Role == RoleOrchestrator {
		add("approval", 6, icShield, "Plan · read-only", 0, func(first bool) {
			seg(c, col(t.Mut).Alpha(0.7), "orchestrator: read + plan + dispatch only — workers own writes", first, nil, func() {
				glyph(c, icShield, textXS, col(t.Mut).Alpha(0.7))
				label(c, "Plan · read-only", textXS, weightRegular, col(t.Mut).Alpha(0.7))
			})
		})
	} else {
		add("approval", 4, icShield, ApprovalLabels[ch.Approval], 0, func(first bool) {
			trig := seg(c, col(t.Mut), "approval mode (ADR-0004)", first, func() { a.togglePopover("chip:approval:" + projectID) }, func() {
				glyph(c, icShield, textXS, col(t.Mut))
				label(c, ApprovalLabels[ch.Approval], textXS, weightRegular, col(t.Mut))
			})
			a.popover(c, t, "chip:approval:"+projectID, trig, ui.AnchorTopStart, ui.AnchorBottomStart, rem(14), func() {
				for _, ap := range Approvals {
					a.menuItem(c, t, ap == ch.Approval, ApprovalLabels[ap], func() {
						a.setChip(projectID, "approval", ap)
						a.togglePopover("chip:approval:" + projectID)
					})
				}
			})
		})
	}
	// identity
	if ag == nil {
		add("base", 5, icFork, "base: "+p.Branch, 0, func(first bool) {
			trig := seg(c, col(t.Mut), "base branch", first, func() { a.togglePopover("base:" + projectID) }, func() {
				glyph(c, icFork, textXS, col(t.Mut))
				label(c, "base: "+p.Branch, textXS, weightRegular, col(t.Mut))
			})
			a.popover(c, t, "base:"+projectID, trig, ui.AnchorTopStart, ui.AnchorBottomStart, rem(11), func() {
				for _, br := range Branches {
					a.menuItem(c, t, br == p.Branch, br, func() {
						a.setProjectBranch(projectID, br)
						a.togglePopover("base:" + projectID)
					})
				}
			})
		})
		add("slug", 2, "", "slug: "+a.slugPreview, 0, func(first bool) {
			seg(c, col(t.Mut).Alpha(0.7), "random slug, display-only (DEC-002)", first, nil, func() {
				label(c, "slug: "+a.slugPreview, textXS, weightRegular, col(t.Mut).Alpha(0.7))
			})
		})
	} else if w != nil {
		add("workspace", 7, icFolder, w.Name, 0, func(first bool) {
			seg(c, col(t.Mut).Alpha(0.7), w.Name, first, nil, func() {
				glyph(c, icFolder, textXS, col(t.Mut).Alpha(0.7))
				label(c, w.Name, textXS, weightRegular, col(t.Mut).Alpha(0.7))
			})
		})
		add("branch", 5, icFork, w.Branch+w.Dirty+w.Untracked, 0, func(first bool) {
			seg(c, col(t.Warn), w.Path, first, nil, func() {
				glyph(c, icFork, textXS, col(t.Warn))
				label(c, w.Branch, textXS, weightRegular, col(t.Warn))
				if w.Dirty != "" || w.Untracked != "" {
					ui.Row(c).Gap(sp1).AlignItems(ui.Center).Children(func() {
						if w.Dirty != "" {
							label(c, w.Dirty, textXS, weightRegular, col(t.Warn))
						}
						if w.Untracked != "" {
							label(c, w.Untracked, textXS, weightRegular, col(t.Acc))
						}
					})
				}
			})
		})
	}
	// vitals
	if ag != nil {
		add("ctx", 8, "", itoa(ag.Ctx)+"%", w8+sp15, func(first bool) {
			seg(c, col(t.Mut).Alpha(0.7), "context usage", first, nil, func() {
				bar := ui.Box(c).Size(w8, sp2).Radius(radiusSM).Background(col(t.Line)).Clip()
				bar.Children(func() {
					ui.Box(c).WidthPercent(float32(ag.Ctx)).Height(sp2).Background(col(t.Acc))
				})
				label(c, itoa(ag.Ctx)+"%", textXS, weightRegular, col(t.Mut).Alpha(0.7))
			})
		})
		if ag.TodoSet {
			add("todo", 2, "", "todo "+itoa(ag.Todo[0])+"/"+itoa(ag.Todo[1]), 0, func(first bool) {
				seg(c, col(t.Mut).Alpha(0.7), "todo phases", first, nil, func() {
					label(c, "todo "+itoa(ag.Todo[0])+"/"+itoa(ag.Todo[1]), textXS, weightRegular, col(t.Mut).Alpha(0.7))
				})
			})
		}
		if ag.Streaming {
			add("tps", 2, "", itoa(ag.TPS)+" t/s", 0, func(first bool) {
				seg(c, col(t.Mut).Alpha(0.7), "tokens per second", first, nil, func() {
					label(c, itoa(ag.TPS)+" t/s", textXS, weightRegular, col(t.Mut).Alpha(0.7))
				})
			})
		}
		// background tasks: the worker's ephemeral subagents
		if subs := a.subsOf(ag.ID); len(subs) > 0 {
			running := 0
			for _, s := range subs {
				if s.SubStatus == StatusRunning {
					running++
				}
			}
			subs := subs
			count := itoa(running) + "/" + itoa(len(subs))
			add("tasks", 7, icTasks, count, 0, func(first bool) {
				trig := seg(c, col(t.Mut), "background tasks (ephemeral subagents)", first, func() { a.togglePopover("bg:" + ag.ID) }, func() {
					glyph(c, icTasks, textXS, col(t.Mut))
					label(c, count, textXS, weightRegular, col(t.Mut))
				})
				a.popover(c, t, "bg:"+ag.ID, trig, ui.AnchorTopStart, ui.AnchorBottomStart, rem(20), func() {
					for _, s := range subs {
						ui.Row(c).Gap(sp15).PaddingX(sp15).PaddingY(sp1).AlignItems(ui.Center).Children(func() {
							dot(c, t, s.SubStatus, false)
							label(c, s.SubName, textXS, weightRegular, col(t.Ink)).SingleLine().Shrink(1)
							ui.Spacer(c)
							label(c, string(s.SubStatus), textXS, weightRegular, col(t.Mut).Alpha(0.6))
						})
					}
				})
			})
		}
		// queued follow-ups (Alt+Enter)
		if n := len(ag.Queue); n > 0 {
			id := ag.ID
			add("queue", 7, icHourglass, itoa(n), 0, func(first bool) {
				trig := seg(c, col(t.Mut), "queued follow-ups (Alt+Enter)", first, func() { a.togglePopover("queue:" + id) }, func() {
					glyph(c, icHourglass, textXS, col(t.Mut))
					label(c, itoa(n), textXS, weightRegular, col(t.Mut))
				})
				a.popover(c, t, "queue:"+id, trig, ui.AnchorTopStart, ui.AnchorBottomStart, rem(20), func() {
					for i, text := range ag.Queue {
						i, text := i, text
						row := ui.Row(c).Gap(sp15).PaddingX(sp15).PaddingY(sp1).AlignItems(ui.Center)
						if row.Hovered() {
							row = row.Background(whiteA(alphaHoverAlt))
						}
						row.Children(func() {
							label(c, text, textXS, weightRegular, col(t.Ink)).SingleLine().Shrink(1)
							ui.Spacer(c)
							up := glyph(c, icArrowUp, textXS, col(t.Mut))
							up = up.Cursor(ui.CursorPointer).Tooltip("promote")
							if up.Clicked() {
								a.promoteQueued(id, i)
							}
							x := glyph(c, icX, textXS, col(t.Mut))
							x = x.Cursor(ui.CursorPointer).Tooltip("remove")
							if x.Clicked() {
								a.removeQueued(id, i)
							}
						})
					}
				})
			})
		}
	}

	// Which segments fit: send (and abort) always stay; the rest compete by
	// priority and keep their design order when drawn.
	budget := a.surfaceWidth(c) - a.sidebarWidth(c) - segWidth(c, icSend, "", 0)
	if ag != nil && ag.Streaming {
		budget -= segWidth(c, icStop, "", 0)
	}
	keep := make(map[string]bool, len(segs))
	byPrio := append([]statusSeg(nil), segs...)
	sort.SliceStable(byPrio, func(i, j int) bool { return byPrio[i].prio > byPrio[j].prio })
	for _, s := range byPrio {
		if s.width <= budget {
			keep[s.key] = true
			budget -= s.width
		}
	}

	ui.Row(c).
		Height(h7).
		Shrink(0).
		Background(col(t.Glass2)).
		Clip().
		Draw(func(p *ui.Painter, r ui.Rect) {
			p.Fill(ui.Rect{X: r.X, Y: r.Y, W: r.W, H: hairline}, whiteA(alphaBorder), 0)
		}).
		Children(func() {
			first := true
			for _, s := range segs {
				if !keep[s.key] {
					continue
				}
				s.draw(first)
				first = false
			}
			ui.Spacer(c)
			if ag != nil && ag.Streaming {
				seg(c, col(t.Err), "abort (immediate, no confirm)", first, func() { a.abortAgent(ag.ID) }, func() {
					glyph(c, icStop, textXS, col(t.Err))
				})
				first = false
			}
			hasText := trimSpace(a.composerText(ag, projectID)) != ""
			tone := col(t.Mut).Alpha(0.4)
			if hasText {
				tone = col(t.Acc)
			}
			seg(c, tone, "Send · Enter steers while streaming · Alt+Enter queues", first, func() {
				a.composeSend(projectID, ag, trimSpace(a.composerText(ag, projectID)), false)
			}, func() {
				glyph(c, icSend, textXS, tone)
			})
		})
}
