package main

import (
	"time"

	"github.com/egoist/mygo/ui"
)

// The state machine: the single source of truth, mirrored from
// prototype/src/store.ts. Views are pure functions of this state and change it
// only through the named transitions below — no side effects while building a
// frame, exactly as MyGo's views work (a frame is built from app state, then
// rebuilt after input, Window.Update or while something animates).
//
// Transitions stay explicit and serializable so the Go daemon/client split
// (ADR-0001) can carry them over the wire one-to-one.

type selKind uint8

const (
	selNone selKind = iota
	selProject
	selWorkspace
	selAgent
)

type sel struct {
	Kind selKind
	ID   string
}

// chips are a Project's last-used values: what the next spawn starts with.
// They also drive the composer's model / thinking / approval segments.
type chips struct {
	Model    string
	Thinking string
	Approval string
}

func defaultChips() chips {
	return chips{Model: Models[0], Thinking: ThinkLevels[2], Approval: Approvals[0]}
}

// winState mirrors the prototype's window frame states.
type winState string

const (
	winNormal    winState = "normal"
	winMaximized winState = "maximized"
	winMinimized winState = "minimized"
	winClosed    winState = "closed"
)

// app is the state the window shows (replaces the scaffold's demo app).
type app struct {
	// ---- domain ----
	fleet       *Fleet
	selection   sel
	chipsByID   map[string]chips // per-Project last-used values (defaults remember)
	slugPreview string           // random slug shown in the Project spawn status bar
	ompMissing  bool             // diagnostics view switch
	themeID     string           // base16 theme (default Nord Dark)
	win         winState

	// the mock clock's bookkeeping (see clockTick)
	lastClock  time.Time
	lastSecond time.Time

	// ---- ui (kept in the machine so every view stays a pure function) ----
	ui struct {
		sidebarCollapsed bool
		hubOpen          bool
		paletteOpen      bool
		paletteQuery     string
		paletteBuf       *string // the palette's input buffer (addressable binding)
		drafts           map[string]*string
		folds            map[string]bool            // tool-card fold state, key "agentID:index"
		popovers         map[string]bool            // queue/background/tree/sub popovers
		followTail       map[string]bool            // per-Agent timeline tail-follow
		scroll           map[string]*ui.ScrollState // per-Agent timeline scroll position
		sidebarScroll    *ui.ScrollState
		hubScroll        *ui.ScrollState
		paletteScroll    *ui.ScrollState
	}
}

func newApp() *app {
	a := &app{
		fleet:       mockFleet(),
		selection:   sel{Kind: selAgent, ID: "a1"},
		chipsByID:   map[string]chips{},
		slugPreview: randSlug(),
		themeID:     defaultTheme,
		win:         winNormal,
	}
	a.ui.drafts = map[string]*string{}
	a.ui.paletteBuf = new(string)
	a.ui.folds = map[string]bool{}
	a.ui.popovers = map[string]bool{}
	a.ui.followTail = map[string]bool{}
	a.ui.scroll = map[string]*ui.ScrollState{}
	a.ui.sidebarScroll = &ui.ScrollState{}
	a.ui.hubScroll = &ui.ScrollState{}
	a.ui.paletteScroll = &ui.ScrollState{}
	a.lastClock = time.Now()
	a.lastSecond = a.lastClock
	return a
}

// anyStreaming reports whether any agent is mid-run: the view keeps asking for
// frames only while something moves.
func (a *app) anyStreaming() bool {
	for _, ag := range a.allAgents() {
		if ag.Streaming {
			return true
		}
	}
	return false
}

// anyRunning reports whether any dot pulses: the design's running agents keep
// their frame loop alive even when nothing streams.
func (a *app) anyRunning() bool {
	for _, ag := range a.allAgents() {
		if ag.Status == StatusRunning {
			return true
		}
	}
	return false
}

// clockTick advances the mock clock — the stand-in for omp's run source —
// from the view, guarded by streaming: 0.1s steps for the tool stopwatch,
// 1s for the tps/ctx jitter. The prototype drives the same two intervals
// from its module bootstrap (main.tsx).
func (a *app) clockTick(now time.Time) {
	const step = 100 * time.Millisecond
	if d := now.Sub(a.lastClock); d >= step {
		steps := min(int(d/step), 10)
		a.lastClock = a.lastClock.Add(time.Duration(steps) * step)
		for range steps {
			a.tickClock()
		}
	}
	if now.Sub(a.lastSecond) >= time.Second {
		a.lastSecond = now
		a.tick()
	}
}

func (a *app) theme() Theme { return themeFor(a.themeID) }

// ---------------------------------------------------------------------------
// selection & ui transitions

func (a *app) selectAgent(id string)     { a.selection = sel{Kind: selAgent, ID: id} }
func (a *app) selectProject(id string)   { a.selection = sel{Kind: selProject, ID: id} }
func (a *app) selectWorkspace(id string) { a.selection = sel{Kind: selWorkspace, ID: id} }
func (a *app) clearSelection()           { a.selection = sel{Kind: selNone} }

func (a *app) toggleSidebar() { a.ui.sidebarCollapsed = !a.ui.sidebarCollapsed }

func (a *app) setHubOpen(v bool) { a.ui.hubOpen = v }

func (a *app) setPaletteOpen(v bool) {
	a.ui.paletteOpen = v
	if !v {
		a.ui.paletteQuery = ""
	}
}

func (a *app) setPaletteQuery(q string) { a.ui.paletteQuery = q }

// pageScroll is a page's scroll state, kept per page id.
func (a *app) pageScroll(key string) *ui.ScrollState {
	st, ok := a.ui.scroll["page:"+key]
	if !ok {
		st = &ui.ScrollState{}
		a.ui.scroll["page:"+key] = st
	}
	return st
}

// draftPtr returns the addressable draft text of a composer/inline input: the
// toolkit's inputs bind to a *string the app owns.
func (a *app) draftPtr(key string) *string {
	p, ok := a.ui.drafts[key]
	if !ok {
		p = new(string)
		a.ui.drafts[key] = p
	}
	return p
}

func (a *app) drafts(key string) string { return *a.draftPtr(key) }

func (a *app) setDraft(key, v string) { *a.draftPtr(key) = v }

func (a *app) toggleFold(key string) { a.ui.folds[key] = !a.ui.folds[key] }

func (a *app) togglePopover(key string) { a.ui.popovers[key] = !a.ui.popovers[key] }

func (a *app) popoverOpen(key string) bool { return a.ui.popovers[key] }

func (a *app) setFollowTail(agentID string, v bool) { a.ui.followTail[agentID] = v }

func (a *app) followsTail(agentID string) bool {
	v, ok := a.ui.followTail[agentID]
	return !ok || v // default true
}

func (a *app) setChip(projectID, field, value string) {
	c, ok := a.chipsByID[projectID]
	if !ok {
		c = defaultChips()
	}
	switch field {
	case "model":
		c.Model = value
	case "thinking":
		c.Thinking = value
	case "approval":
		c.Approval = value
	}
	a.chipsByID[projectID] = c
}

func (a *app) chips(projectID string) chips {
	if c, ok := a.chipsByID[projectID]; ok {
		return c
	}
	return defaultChips()
}

func (a *app) setProjectBranch(projectID, branch string) {
	if p := a.project(projectID); p != nil {
		p.Branch = branch
	}
}

func (a *app) setOmpMissing(v bool) { a.ompMissing = v }

func (a *app) cycleTheme() {
	for i, id := range themeOrder {
		if id == a.themeID {
			a.themeID = themeOrder[(i+1)%len(themeOrder)]
			return
		}
	}
	a.themeID = defaultTheme
}

func (a *app) setWin(w winState) { a.win = w }

// ---------------------------------------------------------------------------
// domain transitions

// spawnFromProject: the project orchestrator dispatches — a workspace, its
// worktree and its worker agent are created together (1:1).
func (a *app) spawnFromProject(projectID, prompt string) {
	p := a.project(projectID)
	if p == nil || prompt == "" {
		return
	}
	base := a.slugPreview
	slug := base
	for n := 2; a.workspaceByName(p, slug) != nil; n++ {
		slug = base + "-" + itoa(n)
	}
	agent := newWorker(prompt, a.chips(projectID))
	p.Workspaces = append(p.Workspaces, &Workspace{
		ID:        nextID("w"),
		Name:      slug,
		Isolation: "worktree",
		Branch:    "fix/" + slug,
		Path:      p.Path + "-wt/" + slug,
		Worker:    agent,
	})
	a.selectAgent(agent.ID)
	a.slugPreview = randSlug()
	a.setDraft("project:"+projectID, "")
}

func newWorker(prompt string, c chips) *Agent {
	return &Agent{
		ID:        nextID("a"),
		Title:     titleOf(prompt),
		Role:      RoleWorker,
		Status:    StatusRunning,
		Model:     c.Model,
		Thinking:  c.Thinking,
		Approval:  c.Approval,
		Ctx:       4,
		TPS:       9,
		Streaming: true,
		Items:     []ChatItem{{Kind: ItemUser, Text: prompt}},
	}
}

func newOrchestrator(name string) *Agent {
	return &Agent{
		ID:       nextID("o"),
		Title:    "orchestrator",
		Role:     RoleOrchestrator,
		Status:   StatusIdle,
		Model:    Models[1],
		Thinking: "high",
		Approval: "yolo",
		Ctx:      4,
		Items: []ChatItem{{
			Kind: ItemMd,
			Text: "Project **" + name + "** registered. I read, plan, and dispatch — workers own writes.",
		}},
	}
}

// titleOf is the prototype's five-word prompt title.
func titleOf(prompt string) string {
	words := splitWords(prompt)
	if len(words) > 5 {
		words = words[:5]
	}
	return joinWords(words)
}

// sendToAgent: DEC-014 — Enter = prompt/steer, Alt+Enter = queue.
func (a *app) sendToAgent(agentID, text string, alt bool) {
	ag := a.agent(agentID)
	if ag == nil || text == "" {
		return
	}
	if alt {
		ag.Queue = append(ag.Queue, text)
	} else {
		ag.Items = append(ag.Items, ChatItem{Kind: ItemUser, Text: text})
		ag.Streaming = true
		ag.Status = StatusRunning
	}
	a.setDraft("agent:"+agentID, "")
	a.setFollowTail(agentID, true)
}

func (a *app) abortAgent(agentID string) {
	if ag := a.agent(agentID); ag != nil {
		ag.Streaming = false
		ag.Status = StatusIdle
		ag.TPS = 0
		ag.Items = append(ag.Items, ChatItem{Kind: ItemMd, Text: "— previous run aborted — queued follow-ups preserved for the next turn"})
	}
}

func (a *app) promoteQueued(agentID string, i int) {
	ag := a.agent(agentID)
	if ag == nil || i < 0 || i >= len(ag.Queue) {
		return
	}
	q := append([]string{ag.Queue[i]}, append(ag.Queue[:i:i], ag.Queue[i+1:]...)...)
	ag.Queue = q
}

func (a *app) removeQueued(agentID string, i int) {
	ag := a.agent(agentID)
	if ag == nil || i < 0 || i >= len(ag.Queue) {
		return
	}
	ag.Queue = append(ag.Queue[:i:i], ag.Queue[i+1:]...)
}

func (a *app) answerAsk(agentID string, index int, answer string) {
	ag := a.agent(agentID)
	if ag == nil || index < 0 || index >= len(ag.Items) {
		return
	}
	if it := &ag.Items[index]; it.Kind == ItemAsk {
		it.AskResolved = answer
		ag.Status = StatusRunning
		ag.Streaming = true
	}
}

func (a *app) newConversation(agentID string) {
	if ag := a.agent(agentID); ag != nil {
		ag.Items = append(ag.Items, ChatItem{Kind: ItemMd, Text: "— new conversation —"})
	}
}

func (a *app) renameAgent(agentID, name string) {
	if ag := a.agent(agentID); ag != nil {
		ag.Title = name
	}
}

// archiveAgent/deleteAgent apply to workers (and their workspace); the
// orchestrator persists with its project.
func (a *app) archiveAgent(agentID string) {
	for _, p := range a.fleet.Projects {
		for _, w := range p.Workspaces {
			if w.Worker.ID == agentID {
				w.Worker.Archived = true
				w.Worker.Streaming = false
				w.Worker.Status = StatusIdle
			}
		}
	}
}

func (a *app) deleteAgent(agentID string) {
	for _, p := range a.fleet.Projects {
		kept := p.Workspaces[:0]
		for _, w := range p.Workspaces {
			if w.Worker.ID != agentID {
				kept = append(kept, w)
			}
		}
		p.Workspaces = kept
	}
	if a.selection.Kind == selAgent && a.selection.ID == agentID {
		a.clearSelection()
	}
}

func (a *app) pruneWorktrees(projectID string) {
	if p := a.project(projectID); p != nil {
		kept := p.Workspaces[:0]
		for _, w := range p.Workspaces {
			if !w.Worker.Archived {
				kept = append(kept, w)
			}
		}
		p.Workspaces = kept
	}
}

func (a *app) openDirectory(name string) {
	id := nextID("p")
	p := &Project{ID: id, Name: name, Path: "~/" + name, Branch: "main", Orchestrator: newOrchestrator(name)}
	a.fleet.Projects = append(a.fleet.Projects, p)
	a.selectProject(id)
}

// tick: 1s heartbeat — tps/ctx jitter for streaming agents.
func (a *app) tick() {
	for _, p := range a.fleet.Projects {
		if p.Orchestrator.Streaming {
			p.Orchestrator.TPS = 8 + randInt(9)
		}
		for _, w := range p.Workspaces {
			ag := w.Worker
			if !ag.Streaming {
				continue
			}
			ag.TPS = 8 + randInt(9)
			if randInt(5) == 0 && ag.Ctx < 99 {
				ag.Ctx++
			}
		}
	}
}

// tickClock: 100ms clock — advances the elapsed time of running tools, which
// stands in for omp's run source (the stopwatch feel is the intended design).
func (a *app) tickClock() {
	for _, p := range a.fleet.Projects {
		for _, w := range p.Workspaces {
			ag := w.Worker
			if !ag.Streaming {
				continue
			}
			for i := range ag.Items {
				it := &ag.Items[i]
				if it.Kind == ItemTool && it.ToolStatus == StatusRunning {
					it.Took = addTenths(it.Took)
				}
			}
		}
	}
}

// ---------------------------------------------------------------------------
// selectors (views use these; no view computes domain facts itself)

func (a *app) project(id string) *Project {
	for _, p := range a.fleet.Projects {
		if p.ID == id {
			return p
		}
	}
	return nil
}

func (a *app) projectOf(agentID string) (*Project, *Workspace, *Agent) {
	for _, p := range a.fleet.Projects {
		if p.Orchestrator.ID == agentID {
			return p, nil, p.Orchestrator
		}
		for _, w := range p.Workspaces {
			if w.Worker.ID == agentID {
				return p, w, w.Worker
			}
		}
	}
	return nil, nil, nil
}

func (a *app) agent(id string) *Agent {
	_, _, ag := a.projectOf(id)
	return ag
}

func (a *app) workspace(id string) (*Project, *Workspace) {
	for _, p := range a.fleet.Projects {
		for _, w := range p.Workspaces {
			if w.ID == id {
				return p, w
			}
		}
	}
	return nil, nil
}

func (a *app) workspaceByName(p *Project, name string) *Workspace {
	for _, w := range p.Workspaces {
		if w.Name == name {
			return w
		}
	}
	return nil
}

// allAgents lists every schedulable agent (orchestrators + workers) in fleet
// order — the collapsed rail and the Agent Hub walk this.
func (a *app) allAgents() []*Agent {
	var out []*Agent
	for _, p := range a.fleet.Projects {
		out = append(out, p.Orchestrator)
		for _, w := range p.Workspaces {
			out = append(out, w.Worker)
		}
	}
	return out
}

// pendingAsks: every unresolved Ask, in fleet order (the palette's "Next Ask").
func (a *app) pendingAsks() []askRef {
	var out []askRef
	for _, ag := range a.allAgents() {
		for i, it := range ag.Items {
			if it.Kind == ItemAsk && it.AskResolved == "" {
				out = append(out, askRef{AgentID: ag.ID, Index: i, Question: it.Text})
			}
		}
	}
	return out
}

type askRef struct {
	AgentID  string
	Index    int
	Question string
}

// liveSubs: the ephemeral subagents of every worker — the status bar's
// background tasks and the Agent Hub both read this.
func (a *app) liveSubs() []subRef {
	var out []subRef
	for _, p := range a.fleet.Projects {
		for _, w := range p.Workspaces {
			for i, it := range w.Worker.Items {
				if it.Kind == ItemSub {
					out = append(out, subRef{AgentID: w.Worker.ID, AgentTitle: w.Worker.Title, Index: i, Sub: it})
				}
			}
		}
	}
	return out
}

type subRef struct {
	AgentID    string
	AgentTitle string
	Index      int
	Sub        ChatItem
}

// subsOf: one agent's own subagents (the composer's background-tasks menu).
func (a *app) subsOf(agentID string) []ChatItem {
	ag := a.agent(agentID)
	if ag == nil {
		return nil
	}
	var out []ChatItem
	for _, it := range ag.Items {
		if it.Kind == ItemSub {
			out = append(out, it)
		}
	}
	return out
}

// pendingAskIndex: the first unanswered Ask of an agent, or -1.
func pendingAskIndex(ag *Agent) int {
	for i, it := range ag.Items {
		if it.Kind == ItemAsk && it.AskResolved == "" {
			return i
		}
	}
	return -1
}
