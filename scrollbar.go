package main

import "github.com/egoist/mygo/ui"

// Persistent scrollbars. The design styles ::-webkit-scrollbar (8px, a
// line-colored thumb, radius 4), which is always visible; MyGo's own bars fade
// in only while the container is hovered, so the port draws the design's
// instead, on top of each scroll container (DrawOver, after its children).
//
// ScrollState carries everything a thumb needs: the offsets (X, Y) and how far
// the content scrolls (MaxX, MaxY), so no extra bookkeeping is needed.

const scrollbarThickness = 8.0

// attachScrollbar paints the container's scrollbar thumbs after its children.
func attachScrollbar(c *ui.Context, t Theme, el ui.Element, st *ui.ScrollState) ui.Element {
	return el.DrawOver(func(p *ui.Painter, r ui.Rect) {
		scrollThumbs(p, r, st, t)
	})
}

// scrollThumbs draws the vertical and horizontal thumbs of st over r.
func scrollThumbs(p *ui.Painter, r ui.Rect, st *ui.ScrollState, t Theme) {
	if st == nil {
		return
	}
	thumb := col(t.Line)
	if st.MaxY > 0 && r.H > 0 {
		content := r.H + st.MaxY
		h := max(r.H*r.H/content, 24)
		y := r.Y + (st.Y/st.MaxY)*(r.H-h)
		p.Fill(ui.Rect{X: r.X + r.W - scrollbarThickness, Y: y, W: scrollbarThickness, H: h}, thumb, scrollbarThickness/2)
	}
	if st.MaxX > 0 && r.W > 0 {
		content := r.W + st.MaxX
		w := max(r.W*r.W/content, 24)
		x := r.X + (st.X/st.MaxX)*(r.W-w)
		p.Fill(ui.Rect{X: x, Y: r.Y + r.H - scrollbarThickness, W: w, H: scrollbarThickness}, thumb, scrollbarThickness/2)
	}
}
