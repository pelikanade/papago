---
title: ADE landscape
date: 2026-10-20
tags: [digest, ade, agent-ide, competitive]
type: digest
---

# ADE landscape

Related: [[2026-10-20 React Native component libraries for agent IDEs]] · [[2026-10-12 Software factory competitive analysis]] · [[2026-10-12 Paseo multi-platform and plugin API]] · [[2026-10-10 Paseo]]
See also: [[2026-10-21 Cloud agents test environments and E2E evidence]] (cloud-agent environments, mobile emulators, E2E evidence in PRs).

**Scope.** An "ADE" (agentic development environment) is a workspace built for running and reviewing coding agents, not for typing code. This survey covers the products Noa's planned React Native ADE would compete with. Star counts and last-push times come from the GitHub API on 2026-10-20 around 23:00 UTC+8. Pricing is from vendor pages or the cited reviews on the same date. No Projects note for this app exists yet; I grepped `Projects/` for ADE, workspace IDE, T3Code and react native and found nothing.

## TL;DR

- **The baseline has converged.** Every serious ADE gives you these:
  - several agents running in parallel
  - one git worktree per task
  - a diff review that sends comments back to the agent
  - an embedded terminal
  - PR create/merge
  - a command palette
  - bring-your-own CLI agent, using your existing subscriptions
- **The fight has moved to four areas:**
  - **remote and mobile:** T3, Paseo, Orca, Superset, Conductor, Claude Code
  - **cloud handoff:** Cursor, Claude Code, Conductor Cloud, Superset, Warp Oz
  - **planning and artifacts:** HumanLayer, Nimbalyst, Jean, Maestro
  - **in-app browser and design mode:** Orca, Superset, Claude Code, Vibe Kanban
- **Mobile is the weakest surface everywhere.** X users call Orca's mobile app "janky" and say T3's is better. T3 maintains two UIs: React DOM for web/desktop and a separate React Native app. Orca does the same (Electron + React DOM, plus a separate Expo app). **Only Paseo ships one RN codebase across iOS, Android, web and Electron.**
- **Gap to aim at:** one RN codebase where native mobile and tablet are first-class and dense desktop isn't an afterthought. It should also have first-class omp/Pi support, a daemon you own (no SaaS dependency), and review/verification gates. The verification gap was already flagged in [[2026-10-12 Software factory competitive analysis]].

## Products

Columns: OSS/license · platforms · agents · architecture · standout. ★ and last push are in UTC+8.

| Product | License / price | Platforms | Agents | Architecture | Standout |
|---|---|---|---|---|---|
| [T3 Code](https://github.com/pingdotgg/t3code) | MIT, free · ★28,759 · 10-20 22:45 | Web (`npx t3`, app.t3.codes), Electron (mac/win/linux), iOS/Android RN | Codex, Claude, Cursor, Grok Build, OpenCode, Antigravity (no Pi/omp) | Node WS server wraps CLIs; event-sourced orchestrator; checkpoint = hidden git ref per turn; T3 Connect tunnel | Performance focus; "remote ready"; strongest mobile per X users |
| [Paseo](https://github.com/getpaseo/paseo) | Apache-2.0, free · ★21,510 · 10-20 21:10 | iOS, Android, web, Electron, CLI | Claude, Codex, OpenCode, Pi, **omp**, Copilot, Antigravity, Muse; ~40 via ACP | Node daemon, Zod WS protocol, E2EE relay; **one Expo/RNW UI** | Plugins compiled by the daemon into RN; schedules; Hub; voice |
| [Orca](https://github.com/stablyai/orca) | MIT · ★99,260 · 10-20 22:56 | mac/win/linux (Electron 43); iOS + Android companion (Expo 55, separate app) | Any CLI; logos list Claude Code, Codex, Grok, OpenCode, Pi, **omp** | Electron + React DOM + Monaco + xterm; `orca` CLI; SSH worktrees; relay in `cloud/` | Fan one prompt out to N agents; Design Mode; annotate AI diffs; account switcher/usage |
| [Conductor](https://conductor.build) | Closed · Free / Pro $50/mo / Teams $60/user/mo | macOS only; iOS app (0.90.0, 2026-10-02, per review) | Claude Code, Codex, Cursor, OpenCode | Local workspaces (city-named worktrees, `$CONDUCTOR_PORT`); Cloud on Vercel sandboxes (Pro) | Polished Mac UX; checkpoints per turn; multiplayer (Pro) |
| [Superset](https://github.com/superset-sh/superset) | **Elastic License 2.0** (source-available) · ★15,396 · 10-20 22:38 | macOS (Linux experimental, no Windows); iPhone/iPad needs **Pro** | Claude Code, Codex, OpenCode, Cursor Agent, Pi, omp, Hermes, Muse, any CLI | Desktop + cloud workspaces + remote hosts; CLI, TS SDK, MCP server | Automations (Sentry/Slack/Linear triggers); Pages for team review; browser Design mode |
| [Emdash](https://github.com/generalaction/emdash) | Apache-2.0 · ★6,036 · 10-20 11:13 (YC W26) | mac/win/linux | Claude Code, Codex, Cursor, OpenCode, Amp, Devin, Qwen, Droid, Copilot | Local-first SQLite; SSH/SFTP remotes; installs agent lifecycle hooks | Issue intake from ~10 trackers (Linear, Jira, GitLab, etc.); CI checks in-app |
| [Claude Code Desktop/web](https://code.claude.com/docs/en/desktop) | Closed · Claude Pro/Max/Team | mac, win, Linux beta; web; iOS/Android Claude app | Claude only | Local / cloud / SSH sessions; worktree toggle; Remote Control; Dispatch | Drag-and-drop panes (chat/diff/browser/terminal/editor); iOS Simulator pane; side chats |
| [Codex app](https://openai.com/index/introducing-the-codex-app/) | Closed (CLI Apache-2.0) · ChatGPT plans | macOS (2026-02-02), Windows (2026-03-04) | Codex only | Threads per project; built-in worktrees; OS sandbox (native on Windows) | Automations; skills; Computer Use; shares CLI history |
| [Cursor Agents Window](https://cursor.com/docs/agent/agents-window) | Closed · Cursor plans | mac/win/linux; agents reachable from phone/web/Slack/GitHub | Cursor agent + models | Cursor 3 (2026-04-02); local ↔ cloud VM handoff; `.cursor/worktrees.json` | `/in-cloud` subagents, `/autopilot` PRs, `/best-of-n` |
| [Warp](https://github.com/warpdotdev/warp) | Client AGPL-3.0 (UI crates MIT) · ★65,586 · 10-20 21:36 | mac/win/linux | Built-in agent + Claude Code, Codex, Gemini CLI | Rust terminal; Oz cloud agents; Warp Factories | Terminal-first ADE; OpenAI is the "founding sponsor" of the open repo |
| [Zed](https://zed.dev/docs/ai/external-agents) | GPL/AGPL/Apache mix (GitHub: NOASSERTION) · ★91,956 | mac/win/linux | Zed Agent + **ACP** external agents (Claude, Codex, OpenCode, Copilot, Cursor, Pi) via the ACP Registry | Editor-first; agents as separate ACP processes; terminal threads | Defines ACP, the open agent↔client protocol |
| [Jean](https://github.com/coollabsio/jean) | Apache-2.0 · ★1,349 · 10-20 20:12 | Tauri desktop (mac tested); headless `jean-server` (Linux); web UI on phones | Claude, Codex, Cursor, OpenCode, Pi, Grok, Kimi, Antigravity, Muse, … | Tauri v2 + React 19 + shadcn + zustand + TanStack Query + @pierre/diffs + xterm | Plan/Build/Yolo modes with plan approval; "Mr. Robot" issue→plan→execute |
| [HumanLayer (CodeLayer)](https://www.humanlayer.com) | Closed (old OSS repo deprecated, ★11,706) | Web, desktop, phone (site claim) | BYO harness (site claim) | Local daemon + cloud daemons synced via HumanLayer API | Multiplayer tasks; versioned research/design/plan artifacts with comment-driven refinement |
| [Happy](https://github.com/slopus/happy) | MIT · ★24,236 · 10-20 06:55 | iOS/Android/web (Expo) + desktop (separate repo) | Claude, Codex, Grok subscriptions; own "Happy Harness" | E2EE relay server; CLI wrapper (`happy claude`) | Multiplayer sessions; mobile-first origin |

**Also seen:**
- [Nimbalyst](https://github.com/nimbalyst/nimbalyst): the former Crystal, MIT, ★1,945. Desktop on all three OSes plus a SwiftUI iOS app. WYSIWYG red/green review of markdown, Mermaid and Excalidraw; session kanban.
- [Sculptor](https://github.com/imbue-ai/sculptor) (Imbue): MIT, ★240, mac/Linux, Claude Code and Pi. Ships workflow skills such as `fix-bug`; container backend is experimental.
- [Maestro](https://github.com/RunMaestro/Maestro): AGPL-3.0, ★3,483. Keyboard-first; Auto Run playbooks; omp (beta); phone access via QR code to a web UI.
- [Vibe Kanban](https://github.com/BloopAI/vibe-kanban): ★28,412. **Bloop shut down on 2026-04-10** ([post](https://www.vibekanban.com/blog/shutdown)); the project is now community-maintained and local-only.
- Omnara pivoted to a managed agent API.
- Claude Squad, opcode, and Kepler/OpenChamber/ctx: seen but not checked. Kepler and ctx were seen only on X/HN.

## Common baseline (table stakes)

1. **Workspace = worktree + branch + terminal + diff**, with setup scripts and per-worktree ports. Examples: Conductor `$CONDUCTOR_PORT`, Cursor `worktrees.json`, Claude `.worktreeinclude`, Superset setup/teardown, Paseo `paseo.json`.
2. **Parallel agents** with a status sidebar (running / needs you / done) and notifications.
3. **Diff review → comments go back to the agent** → commit, PR, merge, archive.
4. **Bring your own agent and subscription.** Wrap the vendor CLIs and don't resell tokens. The exceptions are the vendor apps: Claude, Codex and Cursor each run only their own agent.
5. **Command palette and keyboard-first navigation.** Terminal with splits and persistent scrollback.
6. **Remote access:** SSH, Tailscale, or a relay or tunnel. Most now also have some kind of phone client.

## Differentiators actually in play

- **One codebase on every surface:** Paseo (RN + RNW + Electron). T3 shares contracts and client runtime between two UIs. Orca and Superset keep mobile as a separate app.
- **Cloud handoff and long-running work:** Cursor `/in-cloud`, Claude `--teleport`, Conductor Cloud, Superset cloud workspaces, Warp Oz.
- **Planning and artifacts as first-class objects:** HumanLayer (research→design→plan, comments), Nimbalyst (visual docs), Jean (plan approval), Maestro (playbooks). This is the [[2026-10-12 Software factory competitive analysis]] gate idea moved into the UI.
- **Browser and design mode:** click an element and send its context to the agent (Orca, Superset, Vibe Kanban). Claude adds an iOS Simulator pane.
- **Fan-out / best-of-N:** Orca, Cursor `/best-of-n`.
- **Extensibility:** Paseo plugins (cross-platform RN), Nimbalyst `EditorHost`, Superset skills/SDK/MCP, Zed ACP registry.
- **Usage and account switching:** Orca, Paseo usage dialog, Maestro dashboard.

## Gaps a React Native multi-platform ADE could win

- **Native-quality mobile and tablet.** Users' public complaints are mobile connectivity, scrolling and lost prompts (Orca on X). Superset gates mobile behind Pro; Conductor is Mac + iOS only. A single RN codebase where the phone and iPad UX is designed in, not ported, is the clearest opening. Paseo is the only peer with this architecture.
- **First-class omp/Pi.** T3, Conductor, Claude, Codex and Cursor don't run omp. Orca, Superset, Maestro and Paseo do. Noa runs omp under Paseo, so deep omp support (its own tools, subagents, task list) beats the generic "any CLI" PTY approach.
- **Owned daemon with no SaaS.** Superset's mobile app and Conductor's cloud are paid hosted layers. A self-hosted daemon plus E2EE relay (Paseo/Happy pattern) with free mobile access is a wedge for people who own their infra.
- **Verification and review gates in the UI:** design gate, reviewer agent, CI loop, holdout scenarios. Nobody ships holdout scenarios. HumanLayer is the only one with plan artifacts plus comments, and it's closed and team-priced.
- **Protocol openness.** Speaking ACP (Zed) in addition to custom adapters gets broad agent coverage cheaply.

## Unverified / caveats

- **Conductor:** the iOS app, Cloud on Vercel sandboxes and the 0.78/0.90 dates come from a third-party review (aiignitions.com, 2026-10-13). Pricing tiers are from conductor.build/pricing (the snippet had no plan names; names are from the review).
- **Superset:** Pro price not checked. **HumanLayer:** platforms and "BYO harness" are marketing-site claims.
- **Warp and Zed pricing:** not checked.
- **T3 mobile:** the root README links App Store and Play listings, but `apps/mobile/README.md` still says "not distributed yet" (stale doc, or a different variant).
- **Orca's ★99k** is an exceptionally high count for a 2026 project. Recheck before citing it publicly.
- **Reddit:** not used.
- **X:** 3 queries ([pseudokid on Paseo vs T3](https://x.com/pseudokid/status/2110088700956967002), [ferran9908 Orca vs T3](https://x.com/ferran9908/status/2100978609653534807), [DemetreMinchev on Orca mobile](https://x.com/DemetreMinchev/status/2108729090698297688), [bjtitus on Orca iOS](https://x.com/bjtitus/status/2109026824894459979)).
- **HN:** [Emdash Show HN (206)](https://news.ycombinator.com/item?id=47140322), [Superset Launch HN (108)](https://news.ycombinator.com/item?id=48236770), [Sculptor Show HN (176)](https://news.ycombinator.com/item?id=45427697), [T3 Code](https://news.ycombinator.com/item?id=47283489), ["Current ADE: Orca and Oh My Pi"](https://news.ycombinator.com/item?id=49916232).

## Sources

- **READMEs** (raw GitHub, 2026-10-20): stablyai/orca, superset-sh/superset, generalaction/emdash, BloopAI/vibe-kanban, imbue-ai/sculptor, nimbalyst/nimbalyst, slopus/happy, omnara-ai/omnara, humanlayer/humanlayer, coollabsio/jean, warpdotdev/warp (`master`), RunMaestro/Maestro, pingdotgg/t3code.
- **Orca stack:** `package.json` and `mobile/package.json`.
- **Docs:** [conductor.build](https://www.conductor.build/) and its [pricing](https://www.conductor.build/pricing), [Conductor review](https://www.aiignitions.com/conductor-review), [Claude desktop](https://code.claude.com/docs/en/desktop), [Claude mobile](https://code.claude.com/docs/en/mobile), [Codex on Windows](https://community.openai.com/t/the-codex-app-is-now-on-windows/1375704), [Cursor worktrees](https://cursor.com/docs/configuration/worktrees), [Zed agent panel](https://zed.dev/docs/ai/agent-panel), [humanlayer.com](https://www.humanlayer.com/).
- **Prior art in clones:** `/tmp/rnr/paseo/CLAUDE.md`, `/tmp/rnr/t3code/AGENTS.md`.
