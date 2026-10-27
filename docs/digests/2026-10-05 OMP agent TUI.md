---
title: OMP (oh-my-pi) — coding-agent TUI
date: 2026-10-05
tags:
  - digest
  - omp
  - oh-my-pi
  - pi
  - paseo
  - agent-tui
  - coding-agents
type: digest
publish: true
---

# OMP (oh-my-pi) — coding-agent TUI

> [!summary] One-liner
> **OMP** (`omp`, [oh-my-pi](https://github.com/can1357/oh-my-pi)) is the batteries-included terminal coding agent people mean by “omp” in 2026: a TypeScript + Rust fork of Mario Zechner’s Pi, built by Can Bölük / Stencil Labs, whose default surface is an interactive TUI.

Not OpenMP, and not the stale [open-horizon-labs/oh-omp](https://github.com/open-horizon-labs/oh-omp) fork (last meaningful release line around v0.10.0 in June 2026). Upstream is [can1357/oh-my-pi](https://github.com/can1357/oh-my-pi), site [omp.sh](https://omp.sh).

Related: [[Digests/2026-09-28 Pi agent package development]] · [[Digests/2026-09-28 Pi SDK sessions worktrees and TS coordinator]] · [[Digests/2026-09-28 Pi SDK v2 abilities boundary and data model]]

---

## Who / repo / stack / install

| | |
|---|---|
| Product | OMP, “oh my pi” — “a coding agent with the IDE wired in” |
| Who | Can Bölük ([can1357](https://github.com/can1357)); org credit [Stencil Labs](https://stencil.so). Fork of [Pi / pi-mono](https://github.com/badlogic/pi-mono) by [Mario Zechner](https://github.com/mariozechner). README copyright: Zechner 2025, Bölük 2025–2026, Stencil Labs 2026 |
| Repo | [github.com/can1357/oh-my-pi](https://github.com/can1357/oh-my-pi) — created 2025-12-31 22:01 CST, last push 2026-10-05 00:07 CST |
| License | MIT (vendored brush-shell / builtins stay under their own notices) |
| Scale | **34,266** stars, **3,704** forks, **3,375** open issues+PRs (GitHub API, 2026-10-05). Topics include `tui`, `mcp`, `bun`, `rust` |
| Stack | TypeScript monorepo (`@oh-my-pi/pi-coding-agent`, `pi-ai`, `pi-agent-core`, `pi-tui`, `pi-catalog`, …) plus a Rust N-API core (`pi-natives`, `pi-shell` / vendored brush, `pi-ast`, `pi-iso`, `pi-walker`). Runtime: **Bun ≥ 1.3.14**. CLI binary name `omp` |
| npm | [`@oh-my-pi/pi-coding-agent`](https://www.npmjs.org/package/@oh-my-pi/pi-coding-agent) — dist-tag `latest` is **18.6.0** (published 2026-10-04 06:15 CST). **237,367** downloads in the week ending 2026-10-03; **720,416** in the prior month |

Install (from the current README):

```sh
curl -fsSL https://omp.sh/install | sh          # macOS / Linux
brew install can1357/tap/omp
bun install -g @oh-my-pi/pi-coding-agent        # recommended
nix run github:can1357/oh-my-pi
mise use -g github:can1357/oh-my-pi
```

Windows: `irm https://omp.sh/install.ps1 | iex`. Alpine/musl needs `libstdc++` and `libgcc` before the prebuilt binary. Config lives under `~/.omp` (`agent/config.yml`, `agent/models.yml`). Completions: `omp completions zsh|bash|fish`.

---

## What the TUI does

Default entry is the interactive TUI. Same engine also exposes `omp -p` (one shot), a Node SDK (`createAgentSession`, `SessionManager`, `ModelRegistry`), `omp --mode rpc` / `--mode rpc-ui` (NDJSON over stdio), and `omp acp` ([Agent Client Protocol](https://github.com/zed-industries/agent-client-protocol), documented for Zed).

**Sessions.** Transcripts, `/resume`, `/fresh` (reset provider stream state without wiping the local transcript), `/collab` (relay link + QR; `omp join` or a browser guest; read-only `/collab view`; frames sealed client-side). Welcome banner as of 18.5 is logo + version + tip; recent sessions moved to `/resume`. Agent Hub (`Alt+A`) lists subagents with live transcripts, steering, revive, and kill.

**Models.** README claims **60+ providers** and nine roles: `default`, `smol`, `slow`, `plan`, `commit`, `vision`, `task`, `advisor`, `tiny`. `/model` and `Ctrl+P` swap mid-session. Auth is API key, OAuth, or coding-plan (`/login`). Custom OpenAI-compatible endpoints go in `~/.omp/agent/models.yml`. Fallback chains, path-scoped model sets, and round-robin credentials are first-class. Local: Ollama, LM Studio, llama.cpp, vLLM, LiteLLM. Magic words in prose (`ultrathink`, `orchestrate`, `workflowz`) steer a single turn.

**Tools the TUI renders as cards.** README’s headline set (counts on the badge: **31** built-in tools, **14** LSP ops, **28** DAP ops):

- Files: `read` (files, dirs, SQLite, PDFs, notebooks, URLs, `ssh://`, internal schemes), `write`, hashline `edit`, `ast_edit` / `ast_grep`, `grep`, `glob`
- Runtime: in-process `bash` (brush + coreutils, no WSL required), persistent Python and JS `eval` that can call back into agent tools
- IDE: `lsp` (diagnostics, rename via `workspace/willRenameFiles`, code actions), `debug` (DAP: lldb, dlv, debugpy, …)
- Coordination: `task` (parallel subagents, optional isolated worktrees via APFS/btrfs/overlayfs/…), `wait`, `todo`, `ask` (option picker)
- Desktop/web: `browser` (Chromium / CDP / Chrome relay), `computer` (windows, screenshots, AX, native input), `web_search` (23-provider chain), plus gated `github`, `generate_image`, `tts`
- Memory: `memory.backend` = `off` (default), `local`, `hindsight`, or Mnemopi; tools `retain` / `recall` / `reflect` / `learn`

Internal URL schemes (`pr://`, `issue://`, `agent://`, `skill://`, `conflict://`, `xd://`) are read with the same tools as files. Setting-gated and off by default: `github`, `security_scan`, `generate_image`, `tts`, `checkpoint`, `rewind`, and the memory tools.

**MCP.** Not a separate product surface. On first run, omp inherits rules, skills, and MCP servers already on disk from `.claude`, `.cursor`, `.windsurf`, `.gemini`, `.codex`, `.cline`, `.github/copilot`, and `.vscode`. Extensions/plugins hot-reload with `/reload-plugins`. Package docs (npm README) also point at an MCP config guide and MCP runtime lifecycle inside the monorepo. Exa search keys can come from `/login exa` or a public MCP fallback.

**Multi-agent.** Real, not a slogan: `task` fans out schema-validated subagents; an `advisor` role is a second model reading each turn on its own context; `/review` spawns ranked reviewer subagents; `/vibe` drives persistent `fast`/`good` workers with a read-only director toolset. v18.6.1 fixed subagent follow-up messages so they show up as background jobs that can be waited or canceled, and are delivered once.

---

## Relation to Pi and Paseo

**Pi.** OMP is an explicit fork, not a plugin. Upstream Pi ([earendil-works/pi](https://github.com/earendil-works/pi) / [badlogic/pi-mono](https://github.com/badlogic/pi-mono), docs [pi.dev](https://pi.dev)) is the minimal harness: four tools (`read`, `write`, `edit`, `bash`) plus extensions you install yourself. OMP’s own line is “the Pi you love, with batteries included” — hashline edits, LSP, DAP, subagents, advisor, native Rust search/shell, and a large default prompt. Community split tracks that: Pi users call the kitchen sink a context tax; OMP users want LSP and multi-model roles without assembling packages. Packages are **not** interchangeable (`@earendil-works/pi-*` vs `@oh-my-pi/pi-*`).

**Paseo.** [getpaseo/paseo](https://github.com/getpaseo/paseo) treats OMP as its own provider, not as “Pi with a different binary.” [docs/providers.md](https://github.com/getpaseo/paseo/blob/main/docs/providers.md) and [PR #2067](https://github.com/getpaseo/paseo/pull/2067):

- Pi launches `pi --mode rpc` and gets Paseo MCP through `pi-mcp-adapter` plus `--mcp-config`.
- OMP launches `omp --mode rpc-ui`, is **disabled by default**, discovers slash commands via `get_available_commands`, bridges approval dialogs into Paseo permissions, and imports terminal sessions from `~/.omp/agent/sessions`.
- Paseo host tools register natively (`set_host_tools`, `loadMode: "essential"`) so names like `create_agent` stay direct calls instead of being buried under `xd://`. OMP’s own `task` subagents surface as Paseo subagents; the child runtime stays owned by OMP.
- They share only a JSONL-over-stdio process transport. The PR’s real runs cited Pi 0.65.2 and OMP 17.0.1, with CI also covering OMP 16.3.9 / 17.0.0 — older than today’s 18.6 line, so wire compatibility is something to recheck before relying on it.

Homepage [omp.sh](https://omp.sh) currently teases “omp², soon.” No shipping omp² artifact in the repo releases as of this note.

---

## Status as of 2026-10-05

**Latest GitHub release: [v18.6.1](https://github.com/can1357/oh-my-pi/releases/tag/v18.6.1)**, published 2026-10-05 00:12 CST. npm `latest` is still **18.6.0** (one patch behind the GitHub tag). Cadence is daily: v18.4.12 (2026-10-03 00:03 CST) through v18.6.1 in about two days.

**v18.6.1 (patch, mostly fixes)**

- Native OpenAI compaction no longer mis-estimates screenshot-heavy sessions.
- DeepSeek-family / Command Code: keep reasoning context on warm Responses sessions; strip incomplete DSML tool-call wrappers from visible output.
- Subagent follow-ups are background jobs (wait/cancel, deliver once).
- `/switch` autocomplete matches model-picker ranking, including `@role`.
- Concurrent `skill://` searches no longer block other filesystem work.
- Hashline follow-up edits after a line shift are accepted when the anchor still carries the current content; stale anchors still reject ([#14254](https://github.com/can1357/oh-my-pi/pull/14254)).
- `pi.exec()` reports `-1` on timeout/signal instead of `0`.
- `/collab` guests can answer setting-change and tool-issue prompts.
- Tern `/btw` answers scroll instead of clipping ([#14331](https://github.com/can1357/oh-my-pi/pull/14331)).
- Git diff pane: Space stages/unstages the focused hunk.

**v18.6.0 (2026-10-04 06:06 CST)** — DeepSeek DSML tool calls parsed on every host (llama.cpp, LM Studio, vLLM, custom `models.yml`), not only a hardcoded gateway list; Antigravity client fallback bumped so Claude Opus 5.5 / Sonnet 5.5 stay in `omp models refresh`; `/models` role presets; faster secret redaction and plugin startup; Windows `EPIPE` crash when a DAP/eval/RPC peer exits mid-write.

**v18.5.x (2026-10-03)** — the larger drop under the patch train: generated RPC clients (Python, Rust, Go) and a machine-readable wire schema; GPT live-voice RPC events; read-only `archive` eval global; a lot of Windows correctness (CRLF git, RPC stdout deadlock, session path separators, clipboard); welcome banner simplified; `/jobs` opens a live sheet.

**Known gaps (do not treat older GitHub issues as still open without checking)**

- **Context weight.** The point of the fork is also the complaint. On the July 2026 HN thread, users described a ~20k-token starting context and said most of the tool surface is unused; others stayed because one session can hold several model roles and because LSP beats grep. Pi remains the “install only what you need” answer.
- **Compaction still bites.** Overflow, auto-compact stalls, and fallback onto a smaller window have been recurring bugs through 2026 (early issues #44, #638; later #2913, #8065). Several were fixed in-tree (fast-fallback on summarization timeout, GLM `model_context_window_exceeded` detection, a context-fit check on retry fallback). v18.6.1 is still patching compaction (screenshot token estimates). Long sessions are not “solved.”
- **Release skew.** GitHub tags move faster than the npm dist-tag. Paseo’s documented OMP contract was proven around 16.3.9–17.0.1, not 18.6.
- **Issue volume.** 3k+ open issues+PRs. PRs are in a temporary “open to everyone” trial; the vouch gate may return (README note).
- **Windows** is supported without WSL and is where a large share of 18.5–18.6 fixes landed — expect sharp edges, not a second-class port, but not quiet either.

---

## Field notes

**X (search 2026-10-05, relevancy, last few weeks).** OMP is treated as a named harness next to Claude Code, Codex, Cursor, OpenCode, Hermes, and Pi — not as a niche fork.

- [Sesori](https://x.com/sesori_ai/status/2097717052408750095) (2026-09-09): mobile cockpit lists ten harnesses including Pi and Oh My Pi; OMP was already in the set before Copilot CLI / Grok Build / DeepSeek were added in v1.8.2.
- [NodalSavage](https://x.com/NodalSavage/status/2101703853829689733) (2026-09-20): “Hermes is like a multitool, OMP is like a screwdriver” — better for coding, worse as a general agent.
- [kyfriedfella](https://x.com/kyfriedfella/status/2101469344047755400) (2026-09-20): daily driver Ghostty + OMP, Grok and DeepSeek v4.1 in one session.
- [p3inkilar](https://x.com/p3inkilar/status/2101275957051170967) (2026-09-19): Windows + WSL Docker user says the one-shot install is why they stopped using Codex and Claude Code; TUI is the reason.
- [Aurea](https://x.com/AureaLibe/status/2100891327382884470) (2026-09-18): staying on Pi because OMP ships tools they will never use; recommends OMP for people who do not want to configure.
- [Ariel Frischer](https://x.com/ArielFrischer/status/2105889481253691418) (2026-10-02): “mostly Oh my pi actually for coding.”
- [openwiki](https://x.com/colifran_/status/2102782542252490793) (2026-09-23): `openwiki integrations install omp` alongside Antigravity.
- Comparison posts (local Qwen through Grok Build vs OpenCode vs OMP) are taste, not benchmarks.

**Hacker News** ([Algolia](https://hn.algolia.com/api/v1/search?query=oh-my-pi), 2026-10-05)

- [Oh-my-pi: A coding agent with the IDE wired in](https://news.ycombinator.com/item?id=48994611) — 2026-07-21, 42 points, 15 comments. The thread that states the fork thesis: multi-model roles and LSP vs a 20k-token prompt; “just use Pi” is the top counter.
- [Having fun with oh my pi, DeepSeek-V4-Flash, GPT-5.6 Luna and Antigravity CLI](https://news.ycombinator.com/item?id=49145463) — 2026-08-02, 25 points, 9 comments.
- [Pi vs. OMP](https://composio.dev/content/pi-vs-omp) submitted 2026-09-01, 1 point — comparison piece, no discussion.
- Side comments: Pi or OMP as the model-agnostic alternative when you refuse Claude/Codex/Cursor lock-in (Gemini 3.8 Flash thread, 2026-09-02); local Qwen on a 3090 “works but attention falls apart ~64k” with both Codex and oh-my-pi (2026-05-07).

**Reddit.** Exa `site:reddit.com` did not return a dedicated 2026 thread, and `reddit.com/search.json` returned a block page from this host. No Reddit signal to cite. Discord is the project’s own channel: [discord.gg/4NMW9cdXZa](https://discord.gg/4NMW9cdXZa) (from the README badge).

---

## Sources

- Repo README (fetched 2026-10-05 via GitHub contents API): [can1357/oh-my-pi](https://github.com/can1357/oh-my-pi)
- GitHub repo object: created 2025-12-31, pushed 2026-10-04 16:07 UTC, 34,266 stars, MIT, homepage omp.sh
- Releases: [v18.6.1](https://github.com/can1357/oh-my-pi/releases/tag/v18.6.1) (2026-10-04 16:12 UTC), [v18.6.0](https://github.com/can1357/oh-my-pi/releases/tag/v18.6.0), [v18.5.1](https://github.com/can1357/oh-my-pi/releases/tag/v18.5.1)
- npm registry: `@oh-my-pi/pi-coding-agent` dist-tag `latest` = 18.6.0; downloads API last-week 237,367 (2026-09-27–10-03), last-month 720,416
- Site: [omp.sh](https://omp.sh) · harness writeup linked from README: [The Harness Problem](https://blog.can.ac/2026/02/12/the-harness-problem/) · docs index [omp.sh/docs/tools](https://omp.sh/docs/tools), [providers](https://omp.sh/docs/providers), [sdk](https://omp.sh/docs/sdk)
- Upstream Pi: [pi.dev](https://pi.dev) · [badlogic/pi-mono](https://github.com/badlogic/pi-mono)
- Paseo: [docs/providers.md](https://github.com/getpaseo/paseo/blob/main/docs/providers.md) · [feat(omp): add native OMP provider #2067](https://github.com/getpaseo/paseo/pull/2067)
- Secondary explainers (numbers drift; prefer the README over these): [YUV.AI, 2026-08-06](https://yuv.ai/blog/oh-my-pi-omp-explained) · [explainx.ai, 2026-05-21](https://explainx.ai/blog/oh-my-pi-terminal-coding-agent-omp-mario-zechner-2026)
- HN Algolia stories/comments cited above; X posts cited inline
