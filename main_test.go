package main

import (
	"testing"

	"github.com/egoist/mygo/ui"
)

// The view runs without a window in tests, which click and type as a user
// would (see the mygo-maintenance skill: Tester for UI regressions).
func TestView(t *testing.T) {
	a := newApp()
	tt := ui.NewTester(a.view, 1280, 640)

	// The window opens on the first worker with its mock run.
	if !tt.HasText("fix-login bug") {
		t.Errorf("pane header missing, texts %q", tt.Texts())
	}
	if !tt.HasText("src/auth/session.ts") {
		t.Errorf("timeline missing, texts %q", tt.Texts())
	}

	// The project row IS its orchestrator: clicking it opens that agent, and a
	// prompt steers that agent (Enter = prompt).
	if err := tt.Click("myrepo"); err != nil {
		t.Fatal(err)
	}
	if a.selection.Kind != selAgent || a.selection.ID != "o1" {
		t.Fatalf("selection %+v after clicking the project row", a.selection)
	}
	if err := tt.Click("composer"); err != nil {
		t.Fatal(err)
	}
	tt.Type("plan the health endpoint work")
	tt.Frame()
	tt.Key(0, ui.KeyEnter)
	tt.Frame()
	o := a.agent("o1")
	if got := o.Items[len(o.Items)-1]; got.Kind != ItemUser || got.Text != "plan the health endpoint work" {
		t.Fatalf("orchestrator timeline tail %+v, want the prompt", got)
	}

	// The project page's composer dispatches: a workspace, its worktree and its
	// worker are created together (1:1) — the spawn transition.
	a.selectProject("p1")
	tt.Frame()
	p := a.project("p1")
	before := len(p.Workspaces)
	if err := tt.Click("composer"); err != nil {
		t.Fatal(err)
	}
	tt.Type("add a health endpoint")
	tt.Frame()
	tt.Key(0, ui.KeyEnter)
	tt.Frame()
	if got := len(p.Workspaces); got != before+1 {
		t.Fatalf("workspaces %d after a prompt, want %d", got, before+1)
	}
	w := p.Workspaces[len(p.Workspaces)-1]
	if w.Branch == "" || w.Path == "" {
		t.Errorf("spawned workspace has no worktree: %+v", w)
	}
	if a.selection.Kind != selAgent || a.selection.ID != w.Worker.ID {
		t.Errorf("selection %+v after a spawn, want the new worker %s", a.selection, w.Worker.ID)
	}
	if len(w.Worker.Items) == 0 || w.Worker.Items[0].Text != "add a health endpoint" {
		t.Errorf("new worker timeline %+v, want the prompt", w.Worker.Items)
	}
	// Alt+Enter queues instead of sending (DEC-014).
	tt.Type("then bump the deps")
	tt.Frame()
	tt.Key(ui.Alt, ui.KeyEnter)
	tt.Frame()
	if n := len(w.Worker.Queue); n != 1 || w.Worker.Queue[0] != "then bump the deps" {
		t.Errorf("queue %v after Alt+Enter, want the follow-up", w.Worker.Queue)
	}
}

// The timeline's text is selectable: a drag selects across paragraphs and
// Ctrl+C copies them, while a click on a row still does the row's action.
func TestTimelineSelectAndCopy(t *testing.T) {
	a := newApp()
	a.selectAgent("a9") // short timeline: the md paragraph is on screen
	tt := ui.NewTester(a.view, 1280, 640)
	const md = "Lint failed with 3 errors in src/auth/session.ts. Paused here; rerun after the fix lands."
	r, ok := tt.Find(md)
	if !ok {
		t.Fatalf("md paragraph not found in %q", tt.Texts())
	}
	tt.Move(r.X+4, r.Y+8)
	tt.Press(r.X+4, r.Y+8)
	tt.Move(r.X+r.W-40, r.Y+8)
	tt.Release(r.X+r.W-40, r.Y+8)
	tt.Frame()
	tt.Command("copy")
	if got := tt.Clipboard(); got != md {
		t.Errorf("copied %q, want the paragraph", got)
	}

	// Selection must not turn the rows into drag targets: a click still folds.
	if r, ok := tt.Find("Bash"); ok {
		tt.ClickAt(r.X+r.W/2, r.Y+r.H/2)
		tt.Frame()
		if len(a.ui.folds) == 0 {
			t.Error("clicking a tool row did not fold it")
		}
	} else {
		t.Error("Bash row not found")
	}
}
