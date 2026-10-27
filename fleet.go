// Mock fleet for the papago UI port — mirrors prototype/src/data.ts.
package main

import (
	"fmt"
	"math/rand/v2"
)

type AgentStatus string

const (
	StatusRunning  AgentStatus = "running"
	StatusNeedsYou AgentStatus = "needs-you"
	StatusDone     AgentStatus = "done"
	StatusIdle     AgentStatus = "idle"
	StatusError    AgentStatus = "error"
)

type AgentRole string

const (
	RoleOrchestrator AgentRole = "orchestrator"
	RoleWorker       AgentRole = "worker"
)

// ChatItem is the Go form of the prototype's discriminated TimelineItem union.
type ChatItem struct {
	Kind ChatItemKind

	// user / md
	Text string

	// tool
	Verb       string
	Target     string
	ToolStatus AgentStatus // done | running | error
	Took       string
	DiffAdd    int // 0 when the tool has no diff
	DiffDel    int
	Before     []string // edit diff (empty when none)
	After      []string

	// sub — ephemeral subagents live only inside their worker's timeline
	SubName   string
	SubStatus AgentStatus
	SubModel  string
	SubTool   string
	SubTokens string

	// ask
	AskKind     string // select | confirm | input
	AskOptions  []string
	AskResolved string // "" = still pending
}

type ChatItemKind uint8

const (
	ItemUser ChatItemKind = iota
	ItemMd
	ItemTool
	ItemSub
	ItemAsk
)

type Agent struct {
	ID        string
	Title     string
	Role      AgentRole
	Status    AgentStatus
	Model     string
	Thinking  string
	Approval  string
	Ctx       int
	TPS       int
	Todo      [2]int
	TodoSet   bool // false when the TS has todo: null
	Streaming bool
	Archived  bool
	Queue     []string
	Items     []ChatItem
}

type Workspace struct {
	ID        string
	Name      string
	Isolation string // "worktree" | "local"
	Branch    string
	Path      string
	Dirty     string // "" when absent
	Untracked string // "" when absent
	PR        int    // 0 when absent
	CI        string // "passed" | "failed" | "running" | ""
	Worker    *Agent
}

type Project struct {
	ID           string
	Name         string
	Path         string
	Branch       string
	Orchestrator *Agent
	Workspaces   []*Workspace
}

type Fleet struct {
	Projects []*Project
}

var (
	slugWords   = []string{"swift", "brave", "quiet", "amber", "lunar", "coral", "misty", "iron", "sable", "fern"}
	slugAnimals = []string{"fox", "otter", "heron", "lynx", "marten", "ibex", "crane", "vole", "newt", "drake"}
)

func randSlug() string {
	return fmt.Sprintf("%s-%s", slugWords[rand.IntN(len(slugWords))], slugAnimals[rand.IntN(len(slugAnimals))])
}

var idc int

func nextID(prefix string) string {
	idc++
	return fmt.Sprintf("%s-%d", prefix, idc)
}

var Models = []string{"Claude Sonnet 5.5", "Claude Opus 5.5", "Claude Haiku 5.5", "GPT-5.6 Luna", "DeepSeek V4.1"}

var ThinkLevels = []string{"off", "low", "medium", "high"}

var ThinkLabels = map[string]string{"off": "Off", "low": "Low", "medium": "Medium", "high": "High"}

var Approvals = []string{"yolo", "write-ask", "always-ask"}

var ApprovalLabels = map[string]string{"yolo": "Full Access", "write-ask": "Write Ask", "always-ask": "Always Ask"}

var Branches = []string{"main", "develop", "release/1.0"}

// StatusDotClasses maps a status to the dot colour class (TS STATUS_DOT).
var StatusDotClasses = map[AgentStatus]string{
	StatusRunning:  "bg-work dot-pulse",
	StatusNeedsYou: "bg-warn",
	StatusDone:     "bg-ok",
	StatusIdle:     "bg-mut",
	StatusError:    "bg-err",
}

// Urgency ranks statuses for the status bar: lower sorts first.
var Urgency = map[AgentStatus]int{
	StatusNeedsYou: 0,
	StatusRunning:  1,
	StatusError:    1,
	StatusDone:     2,
	StatusIdle:     3,
}

// mockFleet is the ported initialData(): Project -> persistent orchestrator
// (read/plan/dispatch only) + Workspaces, each 1:1 with a worktree and a
// single worker agent.
func mockFleet() *Fleet {
	return &Fleet{
		Projects: []*Project{
			{
				ID:     "p1",
				Name:   "myrepo",
				Path:   "~/code/myrepo",
				Branch: "main",
				Orchestrator: &Agent{
					ID:        "o1",
					Title:     "orchestrator",
					Role:      RoleOrchestrator,
					Status:    StatusIdle,
					Model:     "Claude Opus 5.5",
					Thinking:  "high",
					Approval:  "yolo",
					Ctx:       9,
					TPS:       0,
					Streaming: false,
					Queue:     []string{},
					Items: []ChatItem{
						{Kind: ItemUser, Text: "ship the login fix and the rate-limit work"},
						{Kind: ItemTool, Verb: "read", Target: "src/auth", ToolStatus: StatusDone, Took: "0.3s"},
						{Kind: ItemTool, Verb: "bash", Target: "git worktree add ../swift-otter -b fix/login", ToolStatus: StatusDone, Took: "1.1s"},
						{Kind: ItemTool, Verb: "bash", Target: "launch worker swift-otter", ToolStatus: StatusDone, Took: "0.2s"},
						{Kind: ItemTool, Verb: "bash", Target: "git worktree add ../brave-otter -b fix/rate-limit", ToolStatus: StatusDone, Took: "0.9s"},
						{Kind: ItemTool, Verb: "bash", Target: "launch worker brave-otter", ToolStatus: StatusDone, Took: "0.2s"},
						{Kind: ItemMd, Text: "**Plan:** two workspaces created. Workers own write access; I only read, plan, and dispatch."},
					},
				},
				Workspaces: []*Workspace{
					{
						ID:        "w1",
						Name:      "swift-otter",
						Isolation: "worktree",
						Branch:    "fix/login",
						Path:      "~/code/myrepo-wt/swift-otter",
						Dirty:     "*1",
						Untracked: "?4",
						PR:        181,
						CI:        "passed",
						Worker: &Agent{
							ID:        "a1",
							Title:     "fix-login bug",
							Role:      RoleWorker,
							Status:    StatusRunning,
							Model:     "Claude Sonnet 5.5",
							Thinking:  "medium",
							Approval:  "yolo",
							Ctx:       42,
							TPS:       12,
							Todo:      [2]int{2, 5},
							TodoSet:   true,
							Streaming: true,
							Queue:     []string{},
							Items: []ChatItem{
								{Kind: ItemUser, Text: "the login redirect loops when the session cookie expires mid-request"},
								{Kind: ItemTool, Verb: "read", Target: "src/auth/session.ts", ToolStatus: StatusDone, Took: "0.4s"},
								{
									Kind:       ItemTool,
									Verb:       "edit",
									Target:     "src/auth/session.ts",
									ToolStatus: StatusDone,
									Took:       "1.2s",
									DiffAdd:    12,
									DiffDel:    3,
									Before: []string{
										"if (cookie.expired) {",
										"  return redirect('/login')",
										"}",
										"session = refresh(cookie)",
									},
									After: []string{
										"session = refresh(cookie)",
										"if (cookie.expired) {",
										"  return redirect('/login')",
										"}",
										"touch(lastSeen)",
									},
								},
								{Kind: ItemTool, Verb: "bash", Target: "go test ./auth", ToolStatus: StatusRunning, Took: "3.1s"},
								{Kind: ItemUser, Text: "also cover the refresh-race path while you are in there"},
								{Kind: ItemSub, SubName: "write regression tests", SubStatus: StatusRunning, SubModel: "Claude Haiku 5.5", SubTool: "bash go test", SubTokens: "4.2k"},
								{Kind: ItemSub, SubName: "audit cookie flags", SubStatus: StatusDone, SubModel: "Claude Haiku 5.5", SubTool: "grep cookie", SubTokens: "1.1k"},
								{Kind: ItemSub, SubName: "check header parsing", SubStatus: StatusError, SubModel: "Claude Haiku 5.5", SubTool: "bash curl", SubTokens: "0.8k"},
								{Kind: ItemAsk, AskKind: "confirm", Text: "Restart the dev server to pick up the new session middleware?", AskResolved: "Approve"},
								{Kind: ItemAsk, AskKind: "select", Text: "Which store for the rate-limit counters?", AskOptions: []string{"in-memory", "redis", "sqlite"}, AskResolved: "redis"},
								{
									Kind: ItemMd,
									Text: "**Root cause:** the cookie refresh raced the middleware. `refresh(cookie)` now runs before the guard, and the redirect path re-reads the session once.\n\nNext: regression test for the expiry path, then a quick audit of the cookie flags.",
								},
								{Kind: ItemMd, Text: "— previous run aborted — queued follow-ups preserved for the next turn"},
							},
						},
					},
					{
						ID:        "w2",
						Name:      "brave-otter",
						Isolation: "worktree",
						Branch:    "fix/rate-limit",
						Path:      "~/code/myrepo-wt/brave-otter",
						Dirty:     "*2",
						PR:        182,
						CI:        "running",
						Worker: &Agent{
							ID:        "a6",
							Title:     "add rate-limit",
							Role:      RoleWorker,
							Status:    StatusNeedsYou,
							Model:     "Claude Sonnet 5.5",
							Thinking:  "medium",
							Approval:  "yolo",
							Ctx:       61,
							TPS:       0,
							Todo:      [2]int{1, 4},
							TodoSet:   true,
							Streaming: false,
							Queue:     []string{"run the full suite", "bump deps"},
							Items: []ChatItem{
								{Kind: ItemUser, Text: "add per-IP rate limiting to the public API"},
								{Kind: ItemTool, Verb: "bash", Target: "go test ./... -run Rate", ToolStatus: StatusDone, Took: "8.1s"},
								{Kind: ItemSub, SubName: "check header parsing", SubStatus: StatusError, SubModel: "Claude Haiku 5.5", SubTool: "bash curl", SubTokens: "0.8k"},
								{Kind: ItemAsk, AskKind: "select", Text: "Which store for the rate-limit counters?", AskOptions: []string{"in-memory", "redis", "sqlite"}},
							},
						},
					},
					{
						ID:        "w3",
						Name:      "tidy-docs",
						Isolation: "worktree",
						Branch:    "docs/v2",
						Path:      "~/code/myrepo-wt/tidy-docs",
						Worker: &Agent{
							ID:        "a7",
							Title:     "docs pass",
							Role:      RoleWorker,
							Status:    StatusDone,
							Model:     "Claude Haiku 5.5",
							Thinking:  "off",
							Approval:  "yolo",
							Ctx:       18,
							TPS:       0,
							Todo:      [2]int{3, 3},
							TodoSet:   true,
							Streaming: false,
							Queue:     []string{},
							Items: []ChatItem{
								{Kind: ItemUser, Text: "refresh the API docs for v2 endpoints"},
								{Kind: ItemTool, Verb: "read", Target: "docs/api-v2.md", ToolStatus: StatusDone, Took: "0.3s"},
								{Kind: ItemTool, Verb: "write", Target: "docs/api-v2.md", ToolStatus: StatusDone, Took: "0.8s"},
								{Kind: ItemMd, Text: "Docs updated: 14 endpoints, 3 new examples."},
							},
						},
					},
					{
						ID:        "w4",
						Name:      "lint-clean",
						Isolation: "worktree",
						Branch:    "chore/lint",
						Path:      "~/code/myrepo-wt/lint-clean",
						Worker: &Agent{
							ID:        "a8",
							Title:     "lint cleanup",
							Role:      RoleWorker,
							Status:    StatusError,
							Model:     "DeepSeek V4.1",
							Thinking:  "low",
							Approval:  "always-ask",
							Ctx:       27,
							TPS:       0,
							Todo:      [2]int{0, 2},
							TodoSet:   true,
							Streaming: false,
							Queue:     []string{},
							Items: []ChatItem{
								{Kind: ItemUser, Text: "clear the lint backlog on this branch"},
								{Kind: ItemTool, Verb: "bash", Target: "eslint .", ToolStatus: StatusError, Took: "2.2s"},
								{Kind: ItemMd, Text: "Failed: `eslint` not installed in this worktree (exit 127). Stopped."},
							},
						},
					},
					{
						ID:        "w5",
						Name:      "review-pr",
						Isolation: "worktree",
						Branch:    "review/182",
						Path:      "~/code/myrepo-wt/review-pr",
						Worker: &Agent{
							ID:        "a10",
							Title:     "review PR 182",
							Role:      RoleWorker,
							Status:    StatusNeedsYou,
							Model:     "Claude Opus 5.5",
							Thinking:  "high",
							Approval:  "yolo",
							Ctx:       33,
							TPS:       0,
							Streaming: false,
							Queue:     []string{},
							Items: []ChatItem{
								{Kind: ItemUser, Text: "review PR 182 for the rate-limit change"},
								{Kind: ItemTool, Verb: "bash", Target: "gh pr diff 182", ToolStatus: StatusDone, Took: "1.9s"},
								{Kind: ItemAsk, AskKind: "input", Text: "Paste the failing CI log, or submit empty to fetch it from Actions."},
							},
						},
					},
					{
						ID:        "w6",
						Name:      "triage-482",
						Isolation: "worktree",
						Branch:    "triage/482",
						Path:      "~/code/myrepo-wt/triage-482",
						Worker: &Agent{
							ID:        "a9",
							Title:     "triage issue #482",
							Role:      RoleWorker,
							Status:    StatusIdle,
							Model:     "GPT-5.6 Luna",
							Thinking:  "low",
							Approval:  "write-ask",
							Ctx:       21,
							TPS:       0,
							Streaming: false,
							Queue:     []string{},
							Items: []ChatItem{
								{Kind: ItemUser, Text: "triage issue #482 — the lint job fails on main"},
								{Kind: ItemMd, Text: "— new conversation —"},
								{Kind: ItemTool, Verb: "bash", Target: "npm run lint", ToolStatus: StatusError, Took: "4.4s"},
								{Kind: ItemMd, Text: "Lint failed with 3 errors in `src/auth/session.ts`. Paused here; rerun after the fix lands."},
							},
						},
					},
				},
			},
			{
				ID:     "p2",
				Name:   "papago-docs",
				Path:   "~/code/papago-docs",
				Branch: "main",
				Orchestrator: &Agent{
					ID:        "o2",
					Title:     "orchestrator",
					Role:      RoleOrchestrator,
					Status:    StatusIdle,
					Model:     "GPT-5.6 Luna",
					Thinking:  "medium",
					Approval:  "yolo",
					Ctx:       5,
					TPS:       0,
					Streaming: false,
					Queue:     []string{},
					Items: []ChatItem{
						{Kind: ItemUser, Text: "regenerate the OpenAPI spec after the v2 rename"},
						{Kind: ItemTool, Verb: "bash", Target: "launch worker regen-openapi", ToolStatus: StatusDone, Took: "0.2s"},
						{Kind: ItemMd, Text: "Handed off to worker `regen-openapi`."},
					},
				},
				Workspaces: []*Workspace{
					{
						ID:        "w7",
						Name:      "regen-openapi",
						Isolation: "worktree",
						Branch:    "docs/openapi",
						Path:      "~/code/papago-docs-wt/regen-openapi",
						PR:        179,
						CI:        "failed",
						Worker: &Agent{
							ID:        "a13",
							Title:     "regenerate OpenAPI",
							Role:      RoleWorker,
							Status:    StatusNeedsYou,
							Model:     "Claude Sonnet 5.5",
							Thinking:  "medium",
							Approval:  "write-ask",
							Ctx:       15,
							TPS:       0,
							Streaming: false,
							Queue:     []string{},
							Items: []ChatItem{
								{Kind: ItemUser, Text: "regenerate the OpenAPI spec after the v2 rename"},
								{Kind: ItemTool, Verb: "bash", Target: "go run ./cmd/openapi", ToolStatus: StatusDone, Took: "2.7s"},
								{Kind: ItemAsk, AskKind: "confirm", Text: "Overwrite docs/openapi.yaml with the regenerated spec?"},
							},
						},
					},
				},
			},
		},
	}
}
