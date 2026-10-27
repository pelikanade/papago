# papago

papago is an ADE (agentic development environment): built for running and reviewing coding agents, not for typing code. This glossary defines the canonical domain language; a term enters it as soon as it is settled, and prose avoids the listed synonyms.

## Language

**ADE**:
Agentic development environment — a workspace centered on running, supervising, and reviewing coding agents, not an editor for typing code.
_Avoid_: IDE, agent-IDE

**Daemon**:
The background service that owns all Agents, Workspaces, and state; the sole holder of the agent lifecycle.
_Avoid_: server, backend, hub

**Client**:
An interface process (GUI, CLI, Plugin backend, …) that connects to the Daemon over its protocol; holds no agent state of its own.
_Avoid_: frontend, app

**Project**:
A repository-level grouping of Workspaces, keyed by the git repository root; detected automatically when a directory is opened. Non-repository directories live under Ungrouped.
_Avoid_: repo, codebase

**Workspace**:
A working-directory environment under a Project — either the local checkout or a dedicated worktree — that hosts Agents and is where they run. Isolation (local vs worktree) is a property of the Workspace, chosen at its creation, never per Agent.
_Avoid_: project, repo, folder

**Agent**:
The smallest execution unit papago schedules and a human supervises; created under one Workspace (sharing its directory with sibling Agents) and driven underneath by a single coding-agent session.
_Avoid_: worker, bot, assistant, runner

**Session**:
One conversation transcript of the underlying coding agent, owned by the agent runtime; a single Agent may resume several Sessions over its lifetime.
_Avoid_: conversation, chat, history

**Subagent**:
A child agent the underlying agent spawns by itself during a run; papago surfaces it and may relay human steering or cancellation to it, but never schedules it directly. Not an Agent.
_Avoid_: child agent, worker

**Agent Hub**:
The Client surface that aggregates live Subagents across Agents; the only place papago offers Subagent steering and cancellation.
_Avoid_: hub (collides with hosted-product naming), subagent panel

**Timeline**:
The ordered event stream of one Agent on the papago side (messages, tool steps, Asks, status transitions).
_Avoid_: log, feed, transcript

**Ask**:
A structured question or confirmation an Agent puts to a human mid-run; it enters the Permission queue and blocks that step until answered.
_Avoid_: permission request, approval, dialog

**Permission queue**:
The set of Asks awaiting human answers, aggregated across Agents.
_Avoid_: approval list, inbox

**Approval mode**:
An Agent's automation level (yolo / write-ask / always-ask): which kinds of operations proceed automatically and which are held back. papago's default is yolo.
_Avoid_: permission mode, trust level

**Host tool**:
A papago orchestration tool (dispatch agents, query status, …) exposed to the underlying agent through the host-tools mechanism, callable by the agent mid-run.
_Avoid_: MCP tool (a different mechanism), built-in tool

**Plugin**:
A standalone program the Daemon runs as a subprocess; it connects as a Client with a reserved identity, observes lifecycle events, takes orchestration actions, and may declare schema-rendered UI. Never part of the Daemon binary.
_Avoid_: extension, add-on, script

**omp**:
The single underlying coding agent papago v0 drives (oh-my-pi, the `omp` command).
_Avoid_: pi (a different upstream product), oh-my-pi (outside formal references)
