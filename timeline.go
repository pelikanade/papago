package main

import (
	"math"
	"strings"

	"github.com/egoist/mygo/ui"
)

// Timeline: every item shares ONE visual grammar — a borderless row
// (status icon · type label · muted detail · right-aligned meta). Expandable
// items (bash, edit) swap the status icon for a caret on hover; the row itself
// never has a background or border — only the expanded body does.
// Mirrors prototype/src/components/Timeline.tsx.

func (a *app) timeline(c *ui.Context, ag *Agent) {
	t := a.theme()
	st := a.timelineScroll(ag.ID)
	a.scrollContent(c, t, st, func() {
		// Selectable: the agent's output and prompts are text to copy (Ctrl+C,
		// or the right-click menu), with one selection across every paragraph.
		ui.Column(c).FillWidth().Padding(sp4).PaddingY(sp3).Gap(sp15).Selectable().Children(func() {
			for i, it := range ag.Items {
				a.timelineItem(c, t, ag, i, it)
			}
			ui.Box(c).Height(sp1)
		})
	})
}

// timelineScroll returns the agent's scroll state, pinned to the end while the
// view follows the tail (the mock's default).
func (a *app) timelineScroll(agentID string) *ui.ScrollState {
	st, ok := a.ui.scroll[agentID]
	if !ok {
		st = &ui.ScrollState{}
		a.ui.scroll[agentID] = st
	}
	if a.followsTail(agentID) {
		st.Y = math.MaxFloat32
	}
	return st
}

func (a *app) timelineItem(c *ui.Context, t Theme, ag *Agent, index int, it ChatItem) {
	switch it.Kind {
	case ItemUser:
		ui.Row(c).WidthPercent(100).Justify(ui.End).Children(func() {
			bubble := ui.Column(c).
				MaxWidthPercent(70).
				PaddingX(sp3).
				PaddingY(sp2).
				Radius(radiusLG).
				Border(hairline, whiteA(alphaBorder)).
				Background(col(t.Glass1)).
				Clip()
			bubble.Children(func() {
				label(c, it.Text, textSM, weightRegular, col(t.Ink))
			})
		})
	case ItemMd:
		richText(c, it.Text, textSM, col(t.Ink).Alpha(0.9))
	case ItemTool:
		a.toolRow(c, t, ag, index, it)
	case ItemSub:
		a.subRow(c, t, it)
	case ItemAsk:
		a.askRow(c, t, it)
	}
}

// toolRow: status icon · verb · target · diff/timing meta; expandable into the
// output card (bash) or the side-by-side diff (edit).
func (a *app) toolRow(c *ui.Context, t Theme, ag *Agent, index int, it ChatItem) {
	foldKey := ag.ID + ":" + itoa(index)
	open := a.ui.folds[foldKey]
	expandable := it.Verb == "bash" || len(it.Before) > 0 || len(it.After) > 0

	row := ui.Row(c).
		WidthPercent(100).
		PaddingX(sp1).
		PaddingY(rowPadSM).
		Gap(sp2).
		AlignItems(ui.Center).
		Focusable()
	if expandable {
		row = row.Cursor(ui.CursorPointer)
	}
	if row.Hovered() && expandable {
		// the caret replaces the status icon on hover
	}
	row.Children(func() {
		ui.Box(c).Size(sp4, sp4).Shrink(0).Draw(func(p *ui.Painter, r ui.Rect) {
			g, colr := toolStatusGlyph(t, it.ToolStatus)
			if expandable && (row.Hovered() || open) {
				g, colr = caretGlyph(open), col(t.Mut).Alpha(0.7)
			}
			drawGlyphCentered(p, r, g, textSM, colr)
		})
		label(c, typeLabel(it.Verb), textSM, weightRegular, col(t.Ink))
		ui.Text(c, it.Target).Font(fontMono).FontSize(textSM).TextColor(col(t.Mut)).SingleLine().Shrink(1)
		ui.Spacer(c)
		ui.Row(c).Gap(sp2).AlignItems(ui.Center).Shrink(0).Children(func() {
			if it.DiffAdd > 0 || it.DiffDel > 0 {
				ui.Row(c).Gap(sp1).AlignItems(ui.Center).Children(func() {
					label(c, "+"+itoa(it.DiffAdd), textXS, weightRegular, col(t.Ok))
					label(c, "−"+itoa(it.DiffDel), textXS, weightRegular, col(t.Err))
				})
			}
			label(c, it.Took, textXS, weightRegular, col(t.Mut).Alpha(0.6))
		})
	})
	if expandable && row.Clicked() {
		a.toggleFold(foldKey)
	}
	if expandable && open {
		ui.Box(c).MarginStart(sp6).Children(func() {
			body := ui.Column(c).
				PaddingX(sp3).
				PaddingY(sp2).
				Radius(radiusMD).
				Border(hairline, whiteA(alphaBorder)).
				Background(col(t.Glass1)).
				Clip()
			body.Children(func() {
				if len(it.Before) > 0 || len(it.After) > 0 {
					splitDiff(c, t, it.Before, it.After)
					return
				}
				ui.Text(c, bashPreview).
					Font(fontMono).
					FontSize(textXS).
					TextColor(col(t.Mut)).
					LineHeight(textSMLine)
			})
		})
	}
}

const bashPreview = "$ go test ./... -run Rate\nok  api/rate  8.1s\nok  api/auth  0.9s"

// subRow: an ephemeral subagent — icon · Subagent · title · model/tool/tokens.
func (a *app) subRow(c *ui.Context, t Theme, it ChatItem) {
	ui.Row(c).
		WidthPercent(100).
		PaddingX(sp1).
		PaddingY(rowPadSM).
		Gap(sp2).
		AlignItems(ui.Center).
		Children(func() {
			ui.Box(c).Size(sp4, sp4).Shrink(0).Draw(func(p *ui.Painter, r ui.Rect) {
				g, colr := toolStatusGlyph(t, it.SubStatus)
				drawGlyphCentered(p, r, g, textSM, colr)
			})
			label(c, "Subagent", textSM, weightRegular, col(t.Ink))
			label(c, it.SubName, textSM, weightRegular, col(t.Mut)).SingleLine().Shrink(1)
			ui.Spacer(c)
			label(c, it.SubModel+" · "+it.SubTool+" · "+it.SubTokens, textXS, weightRegular, col(t.Mut).Alpha(0.7)).SingleLine().Shrink(0)
		})
}

// askRow: the question with its resolution on the right, struck through once
// answered.
func (a *app) askRow(c *ui.Context, t Theme, it ChatItem) {
	done := it.AskResolved != ""
	ui.Row(c).
		WidthPercent(100).
		PaddingX(sp1).
		PaddingY(rowPadSM).
		Gap(sp2).
		AlignItems(ui.Center).
		Children(func() {
			ui.Box(c).Size(sp4, sp4).Shrink(0).Draw(func(p *ui.Painter, r ui.Rect) {
				if done {
					drawGlyphCentered(p, r, icCheck, textSM, col(t.Ok))
				} else {
					drawGlyphCentered(p, r, "?", textSM, col(t.Warn))
				}
			})
			label(c, "Ask", textSM, weightRegular, col(t.Ink))
			q := label(c, it.Text, textSM, weightRegular, col(t.Mut))
			if done {
				q = q.Strikethrough().TextColor(col(t.Mut).Alpha(0.5))
			}
			q.SingleLine().Shrink(1)
			ui.Spacer(c)
			if done {
				label(c, it.AskResolved, textXS, weightRegular, col(t.Ink))
			} else {
				label(c, "answer below", textXS, weightRegular, col(t.Warn).Alpha(0.8))
			}
		})
}

// splitDiff is the edit card: old lines left, new lines right, mono text-xs.
func splitDiff(c *ui.Context, t Theme, before, after []string) {
	ui.Row(c).Gap(sp3).WidthPercent(100).Children(func() {
		ui.Column(c).Grow(1).Basis(0).Children(func() {
			ui.Box(c).Margin(0, 0, sp1, 0).Children(func() {
				label(c, "before", textXS, weightRegular, col(t.Mut).Alpha(0.6))
			})
			for _, line := range before {
				ui.Row(c).Gap(sp2).Children(func() {
					ui.Text(c, "−").Font(fontMono).FontSize(textXS).TextColor(col(t.Mut).Alpha(0.4))
					ui.Text(c, line).Font(fontMono).FontSize(textXS).TextColor(col(t.Err).Alpha(0.9)).SingleLine()
				})
			}
		})
		ui.Column(c).Grow(1).Basis(0).Children(func() {
			ui.Box(c).Margin(0, 0, sp1, 0).Children(func() {
				label(c, "after", textXS, weightRegular, col(t.Mut).Alpha(0.6))
			})
			for _, line := range after {
				ui.Row(c).Gap(sp2).Children(func() {
					ui.Text(c, "+").Font(fontMono).FontSize(textXS).TextColor(col(t.Mut).Alpha(0.4))
					ui.Text(c, line).Font(fontMono).FontSize(textXS).TextColor(col(t.Ok).Alpha(0.9)).SingleLine()
				})
			}
		})
	})
}

// typography: the timeline's small helpers

func toolStatusGlyph(t Theme, s AgentStatus) (string, ui.Color) {
	switch s {
	case StatusDone:
		return icCheck, col(t.Ok)
	case StatusError:
		return icX, col(t.Err)
	default:
		return icSpinner, col(t.Acc)
	}
}

func drawGlyphCentered(p *ui.Painter, r ui.Rect, g string, size float32, c ui.Color) {
	if g == "" {
		return
	}
	w, h := p.MeasureText(0, ui.Span{Text: g, Font: fontIcons, Size: size})
	p.Text(r.X+(r.W-w)/2, r.Y+(r.H-h)/2, g, size, c)
}

// typeLabel capitalizes a tool verb: read → Read.
func typeLabel(verb string) string {
	if verb == "" {
		return ""
	}
	return strings.ToUpper(verb[:1]) + verb[1:]
}

// richText renders the prototype's tiny markdown: **bold** and `code`, with
// newlines kept as line breaks.
func richText(c *ui.Context, text string, size float32, color ui.Color) {
	spans := mdSpans(text, size, color)
	txt := ui.RichText(c, spans...)
	txt.Font(fontSans + ", " + fontIcons).FontSize(size).TextColor(color)
}

// mdSpans splits the markdown subset the mock data uses into spans: inline
// code (mono, accent, on a tinted background) and bold runs.
func mdSpans(text string, size float32, color ui.Color) []ui.Span {
	var spans []ui.Span
	rest := text
	for len(rest) > 0 {
		i := strings.IndexByte(rest, '`')
		j := strings.Index(rest, "**")
		switch {
		case i >= 0 && (j < 0 || i < j):
			if i > 0 {
				spans = append(spans, ui.Span{Text: rest[:i], Size: size, Color: color})
			}
			end := strings.IndexByte(rest[i+1:], '`')
			if end < 0 {
				spans = append(spans, ui.Span{Text: rest[i:], Size: size, Color: color})
				rest = ""
				continue
			}
			spans = append(spans, ui.Span{
				Text:       rest[i+1 : i+1+end],
				Font:       fontMono,
				Size:       size - 2,
				Color:      color,
				Background: whiteA(alphaHover),
				Radius:     radiusSM,
			})
			rest = rest[i+2+end:]
		case j >= 0:
			if j > 0 {
				spans = append(spans, ui.Span{Text: rest[:j], Size: size, Color: color})
			}
			end := strings.Index(rest[j+2:], "**")
			if end < 0 {
				spans = append(spans, ui.Span{Text: rest[j:], Size: size, Color: color})
				rest = ""
				continue
			}
			spans = append(spans, ui.Span{Text: rest[j+2 : j+2+end], Size: size, Weight: weightSemiBold, Color: color})
			rest = rest[j+4+end:]
		default:
			spans = append(spans, ui.Span{Text: rest, Size: size, Color: color})
			rest = ""
		}
	}
	return spans
}
