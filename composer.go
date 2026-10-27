package main

import "github.com/egoist/mygo/ui"

// DEC-015 Composer Bar: a clean text box (no chrome inside) above the pane's
// bottom-edge status bar — a full-bleed footer. Every interactive segment is a
// trigger + popover; vitals are plain segments. Icons are Nerd Font glyphs.
// DEC-014: Enter = send/steer, Alt+Enter = queue, Shift+Enter = newline.
// Mirrors prototype/src/components/ComposerBar.tsx.

// composerBar builds the writing surface and the status bar for the pane.
// agentID empty = the Project composer (its orchestrator dispatches).
func (a *app) composerBar(c *ui.Context, projectID, workspaceID, agentID string) {
	t := a.theme()
	var ag *Agent
	if agentID != "" {
		ag = a.agent(agentID)
	}
	inAgent := ag != nil

	ui.Column(c).Shrink(0).Children(func() {
		// writing surface: clean box — or the pending Ask, which replaces it
		ui.Box(c).PaddingX(sp3).PaddingY(sp15).Children(func() {
			if inAgent {
				if i := pendingAskIndex(ag); i >= 0 {
					a.askForm(c, t, ag, i)
					return
				}
			}
			key := "project:" + projectID
			placeholder := "New Agent in a fresh worktree…"
			if inAgent {
				key = "agent:" + agentID
				placeholder = "What are we cooking?"
			}
			// The prototype's box renders 45.8px tall (textarea py-2 + a taller
			// line box); with MyGo's 22px ink for text-sm this padding matches it.
			box := ui.Column(c).
				PaddingX(sp35).
				PaddingY(11).
				Radius(radiusLG).
				Border(hairline, whiteA(alphaBorder)).
				Background(col(t.Glass1)).
				Clip()
			box.Children(func() {
				// TextAreaBase: no toolkit look, so the design's box (hairline,
				// glass, radius) is the only chrome, as in the prototype.
				ta := ui.TextAreaBase(c, a.draftPtr(key)).
					Placeholder(placeholder).
					Label("composer").
					Font(fontSans+", "+fontIcons).
					FontSize(textSM).
					FixedLineHeight(textSMLine).
					TextColor(col(t.Ink)).
					Background(ui.Transparent).
					Padding(0).
					Lines(1, 6).
					WidthPercent(100)
				ta.OnTextKey(func(k ui.TextKey) bool {
					if k.Key != ui.KeyEnter {
						return false
					}
					switch k.Mods {
					case 0:
						if s := trimSpace(k.Text()); s != "" {
							k.SetText("")
							a.composeSend(projectID, ag, s, false)
						}
						return true
					case ui.Alt:
						if s := trimSpace(k.Text()); s != "" {
							k.SetText("")
							a.composeSend(projectID, ag, s, true)
						}
						return true
					}
					return false // Shift+Enter inserts a newline
				})
			})
		})
		a.statusBar(c, t, projectID, workspaceID, ag)
	})
}

// composeSend routes the composer's text: a project prompt spawns a workspace +
// worker through the orchestrator; an agent prompt steers or queues (DEC-014).
func (a *app) composeSend(projectID string, ag *Agent, text string, alt bool) {
	if ag == nil {
		a.spawnFromProject(projectID, text)
		return
	}
	a.sendToAgent(ag.ID, text, alt)
}

// askForm is the interactive Ask, rendered in the composer slot: it replaces
// the text box while a pending Ask exists (the timeline entry stays read-only).
func (a *app) askForm(c *ui.Context, t Theme, ag *Agent, index int) {
	it := ag.Items[index]
	ui.Column(c).
		Padding(sp25).
		Gap(sp2).
		Radius(radiusLG).
		Border(hairline, col(t.Warn).Alpha(0.5)).
		Background(col(t.Warn).Alpha(0.05)).
		Clip().
		Children(func() {
			ui.Row(c).Gap(sp2).AlignItems(ui.Center).Children(func() {
				badge := ui.Row(c).
					Height(h5).
					PaddingX(sp15).
					Radius(radiusSM).
					Background(col(t.Warn).Alpha(0.2)).
					AlignItems(ui.Center)
				badge.Children(func() { label(c, "Ask · "+it.AskKind, textXS, weightRegular, col(t.Warn)) })
				label(c, "blocks this step until answered · never expires", textXS, weightRegular, col(t.Mut))
			})
			label(c, it.Text, textSM, weightRegular, col(t.Ink))
			switch it.AskKind {
			case "select":
				ui.Row(c).Gap(sp15).Wrap().Children(func() {
					for i, o := range it.AskOptions {
						opt := o
						idx := i
						b := ui.Row(c).
							Height(h7).
							PaddingX(sp2).
							Gap(sp1).
							Radius(radiusSM).
							Border(hairline, col(t.Line)).
							AlignItems(ui.Center).
							Cursor(ui.CursorPointer).
							Label(opt).
							Focusable()
						if b.Hovered() {
							b = b.BorderColor(col(t.Acc))
						}
						b.Children(func() {
							label(c, itoa(idx+1), textSM, weightRegular, col(t.Mut).Alpha(0.6))
							label(c, opt, textSM, weightRegular, col(t.Ink))
						})
						if b.Clicked() {
							a.answerAsk(ag.ID, index, opt)
						}
					}
				})
			case "confirm":
				ui.Row(c).Gap(sp15).Children(func() {
					ok := ui.Row(c).
						Height(h7).
						PaddingX(sp25).
						Radius(radiusSM).
						Background(col(t.Acc)).
						AlignItems(ui.Center).
						Cursor(ui.CursorPointer).
						Label("Approve").
						Focusable()
					ok.Children(func() { label(c, "Approve", textSM, weightRegular, col(t.Bg)) })
					if ok.Clicked() {
						a.answerAsk(ag.ID, index, "Approve")
					}
					cancel := ui.Row(c).
						Height(h7).
						PaddingX(sp25).
						Radius(radiusSM).
						Border(hairline, col(t.Line)).
						AlignItems(ui.Center).
						Cursor(ui.CursorPointer).
						Label("Cancel").
						Focusable()
					if cancel.Hovered() {
						cancel = cancel.BorderColor(col(t.Err))
					}
					cancel.Children(func() { label(c, "Cancel", textSM, weightRegular, col(t.Mut)) })
					if cancel.Clicked() {
						a.answerAsk(ag.ID, index, "Cancel")
					}
				})
			case "input":
				key := "ask:" + ag.ID + ":" + itoa(index)
				ui.Row(c).Gap(sp15).Children(func() {
					in := ui.TextInputBase(c, a.draftPtr(key)).
						Font(fontSans+", "+fontIcons).
						FontSize(textSM).
						Placeholder("").
						Height(h7).
						PaddingX(sp2).
						Radius(radiusSM).
						Border(hairline, col(t.Line)).
						Background(ui.Transparent).
						Grow(1)
					in.OnSubmit(func() { a.answerDraft(ag.ID, index, key) })
					submit := ui.Row(c).
						Height(h7).
						PaddingX(sp25).
						Radius(radiusSM).
						Background(col(t.Acc)).
						AlignItems(ui.Center).
						Cursor(ui.CursorPointer).
						Label("Submit").
						Focusable()
					submit.Children(func() { label(c, "Submit", textSM, weightRegular, col(t.Bg)) })
					if submit.Clicked() {
						a.answerDraft(ag.ID, index, key)
					}
				})
			}
		})
}

func (a *app) answerDraft(agentID string, index int, key string) {
	if v := trimSpace(a.drafts(key)); v != "" {
		a.answerAsk(agentID, index, v)
	}
}

func (a *app) composerText(ag *Agent, projectID string) string {
	if ag != nil {
		return a.drafts("agent:" + ag.ID)
	}
	return a.drafts("project:" + projectID)
}

// menuItem is one row of a status-bar popover menu: a 16px check slot, the
// label, the accent color while active.
func (a *app) menuItem(c *ui.Context, t Theme, active bool, text string, onPick func()) {
	item := ui.Row(c).
		WidthPercent(100).
		PaddingX(sp2).
		PaddingY(menuPadY).
		Gap(sp2).
		AlignItems(ui.Center).
		Radius(radiusSM).
		Cursor(ui.CursorPointer).
		Label(text).
		Focusable()
	if item.Hovered() && !active {
		item = item.Background(whiteA(alphaHoverAlt))
	}
	if active {
		item = item.Background(whiteA(alphaHover))
	}
	item.Children(func() {
		ui.Box(c).Width(sp4).Shrink(0).Children(func() {
			if active {
				glyph(c, icCheck, textXS, col(t.Acc))
			}
		})
		label(c, text, textXS, weightRegular, col(t.Ink))
	})
	if item.Clicked() && onPick != nil {
		onPick()
	}
}
