---
title: OMP features and highlights
date: 2026-10-05
tags:
  - digest
  - omp
  - oh-my-pi
  - agent-tui
type: digest
publish: true
---

# OMP features and highlights

> [!summary] One-liner
> OMP is the batteries-included terminal coding agent: one process that edits by content hash, talks to your language server and debugger, and fans work out to subagents, instead of a four-tool harness you assemble yourself.

Companion overview (who, install, stack, Pi/Paseo, release status): [[Digests/2026-10-05 OMP agent TUI]]. This note is the feature surface as of the README on `main` and the docs at [omp.sh](https://omp.sh), checked 2026-10-05. Latest GitHub tag is [v18.6.1](https://github.com/can1357/oh-my-pi/releases/tag/v18.6.1) (published 2026-10-05 00:12 CST). There is no separate “features” page on omp.sh; the homepage is the marketing cut of the README, and the real inventory is [docs/tools](https://omp.sh/docs/tools).

---

## Surfaces: TUI, one-shot, SDK, RPC, ACP

Same engine, five wrappers. The README groups the last four as “four entry points” and then documents the SDK beside them ([README](https://github.com/can1357/oh-my-pi/blob/main/README.md#four-entry-points-interactive-one-shot-rpc-and-acp), [SDK](https://omp.sh/docs/sdk)).

| Surface | How you enter | Why it exists |
|---|---|---|
| Interactive TUI | `omp` | Default. Tool calls render as cards, edits preview before they land, ambiguity goes through `ask` (a structured option picker). The same cards also go out over ACP. |
| One-shot | `omp -p` | Answer one prompt and exit. Pair with `--no-session` when a pipeline must not write a transcript. |
| SDK | `bun add @oh-my-pi/pi-coding-agent` | In-process embed. `createAgentSession()` discovers credentials, settings, extensions, skills, rules, MCP, and LSP unless you override them. Hosts can add `customTools`, subscribe to stream events (`text_delta`, tool calls, compaction, retry), and `steer` / `followUp` / `abort`. |
| RPC | `omp --mode rpc` | NDJSON over stdio for a non-Bun host or process isolation. `--mode rpc-ui` adds tool cards, selectors, and dialogs as `extension_ui_request` frames the host must answer. |
| ACP | `omp acp` | [Agent Client Protocol](https://github.com/zed-industries/agent-client-protocol) JSON-RPC, documented for Zed. `bash` maps to `terminal/create`, `read`/`write` to `fs/*`, and destructive tools pause on `session/request_permission`. |

**SDK correction.** The README heading still says “embed in Node.” The SDK page is explicit: Bun ≥ 1.3.14, and “it is not a Node.js SDK.” The overview digest called it a Node SDK; that wording matches the README heading, not the current docs page. Do not put an RPC child behind the SDK just to call the same process twice.

---

## Editing model: hashline, AST, conflicts

**Hashline (`edit`).** The model points at short content-hash anchors instead of retyping the lines it wants to change. Stale anchors are rejected before the file is written. The README’s own numbers, which are marketing until you re-run the bench: Grok Code Fast 1 edit success 6.7% → 68.3% once the format stopped eating the model; Gemini 3 Flash +5 pp over `str_replace`; Grok 4 Fast −61% output tokens; MiniMax 2.1× pass rate. Implementation lives in [`@oh-my-pi/hashline`](https://github.com/can1357/oh-my-pi). v18.6.1 ([#14254](https://github.com/can1357/oh-my-pi/pull/14254)) accepts a follow-up edit after a line shift when the anchor still carries the current content, and still rejects a genuinely stale line.

**`ast_edit` / `ast_grep`.** Structural rewrites via ast-grep over 50+ tree-sitter grammars. `ast_edit` returns a *proposed* card; the disk write happens only after the agent writes a one-line reason to `xd://resolve` and the TUI shows Accept (atomic, all or nothing). Docs: `ast_edit` is on by default (`astEdit.enabled`); `ast_grep` search is **off** by default (`astGrep.enabled`). The README lists both in the tool catalog without that split.

**`write`** creates or overwrites a file, archive entry, or SQLite row. **`conflict://N`** turns each merge conflict into a URL; the agent writes `@theirs`, `@ours`, or `@base` (bulk: `conflict://*`).

---

## Code intelligence: LSP and DAP

**LSP** is on by default (`lsp.enabled`; `--no-lsp` for one session). The README’s badge is **14 lsp ops**. Documented uses: diagnostics, definitions, references, symbols, hover, code actions, raw requests, and rename through `workspace/willRenameFiles` so re-exports and aliased imports update before a file moves. Inspection is classed Read; fixes and renames are Write. The server still has to be installed or configured. People who left an IDE for the terminal cite this as the reason OMP beats a grep-only agent (see Field).

**DAP (`debug`)** is on by default (`debug.enabled`) but only appears when an adapter is actually available. README badge: **28 dap ops**. The tool launches or attaches, sets breakpoints, steps, and reads threads, stack, scopes, and variables. README’s worked examples: lldb-dap on a native segfault, dlv on a hung Go service, debugpy on a wedged Python process. Docs also name gdb and js-debug-adapter. Inspection is Read; launch and process control are Exec.

---

## Subagents, advisor, and the other multi-agent modes

**`task`.** Fans independent work to child agents, optionally in isolated worktrees (APFS clones, btrfs/zfs reflinks, overlayfs, projfs, rcopy — crate `pi-iso`). Workers can DM each other on an in-process IRC bus instead of routing every message through the parent. The parent reads a schema-validated object, not prose. Default recursion depth is 2 (`task.maxRecursionDepth`). v18.6.1 made subagent follow-ups background jobs you can wait or cancel, delivered once.

**Agent Hub (`Alt+A`).** Roster of live subagents: transcript, steering message, revive a parked worker, kill a stuck one without aborting the parent.

**`hub`.** Message peer agents, follow or cancel background jobs, supervise shared long-running processes (dev servers, debuggers). Available when collaboration is possible at the current task depth.

**Advisor.** A second model on the `advisor` role, with its own context, reads each turn and injects a quiet aside, a concern, or a hard blocker. The main agent course-corrects or explains why it won’t. This is the feature people name unprompted (see Field).

**Other director modes** ([tools index](https://omp.sh/docs/tools)):

- `/review` — reviewer subagents over a branch, commit, uncommitted work, or PR; findings ranked P0–P3 with a ship/no-ship verdict.
- `/vibe` — the main session becomes a read-only director over persistent `fast` / `good` workers.
- `/plan` — planning turn on the `plan` role, sandboxed read-only, then execute-and-purge, keep the transcript, or compact.
- `/goal` and `/guided-goal` — one durable, budgeted objective across turns.
- `/autoresearch` — repeated benchmark-backed experiments on an `autoresearch/*` branch. Some reset controls are destructive.
- Magic words in prose only (not code spans): `ultrathink`, `orchestrate`, `workflowz`.

---

## MCP inheritance

OMP is an MCP client, not a separate MCP product. On first run it reads rules, skills, and MCP servers already on disk. The README names `.claude`, `.cursor`, `.windsurf`, `.gemini`, `.codex`, `.cline`, `.github/copilot`, and `.vscode`. The [MCP docs](https://omp.sh/docs/mcp) are slightly tighter for *servers*: Claude Code, Codex, Gemini CLI, OpenCode, Cursor, Windsurf, VS Code, installed plugins, and extension packages. `/mcp list` shows which file supplied each server. First definition wins; duplicates are not merged. `/mcp disable` hides an imported server without editing the other tool’s config. OAuth writes only omp-managed files.

Own config: `.omp/mcp.json` (project), `~/.omp/agent/mcp.json` (user), or `~/.omp/profiles/<name>/agent/mcp.json`. Transports: `stdio` (default), `http` (Streamable HTTP), `sse` (legacy; a URL ending in `/sse` does not select it by itself). Tool names surface as `mcp__<server>_<tool>`. Prompts become `/server:prompt`. Resource-update injection is off unless `mcp.notifications` is on.

A `stdio` server runs as your user. The docs tell you to review project MCP files before opening an unfamiliar repo, and to use a separate profile for untrusted checkouts because OAuth is bound to profile + URL.

---

## Models and providers

Nine roles: `default`, `smol`, `slow`, `plan`, `commit`, `vision`, `task`, `advisor`, `tiny`. Swap mid-session with `/model`; cycle the active role’s models with `Ctrl+P`. Launch overrides: `--smol`, `--slow`, `--plan`.

README claim: **60+ providers**, a thousand models. Auth is API key, OAuth (`/login`), or a coding-plan subscription. Custom OpenAI-compatible (and several other wire APIs) go in `~/.omp/agent/models.yml`. Routing knobs that are actually specified, not slogans:

- **Fallback chains** under `retry.fallbackChains` — next entry takes the rest of the turn on 429/quota, restored on cooldown.
- **Path-scoped** `enabledModels` / `disabledProviders` so one repo can pin a different set.
- **Round-robin credentials** with session affinity and per-key backoff.

Local: Ollama, LM Studio, llama.cpp, vLLM, LiteLLM. Full list: [omp.sh/docs/providers](https://omp.sh/docs/providers).

`web_search` is one tool over a 23-provider chain (`auto`, or pin one). Site-aware extractors cover GitHub/GitLab, package registries, arXiv, Stack Overflow, Reddit, HN, MDN, docs.rs. Exa accepts `/login exa` or a public MCP fallback. Vuln lookups hit NVD, OSV, and CISA KEV.

---

## Sessions

Sessions are append-only JSONL trees under `~/.omp/agent/sessions/`, one history per working directory, ids are UUID v7 ([sessions](https://omp.sh/docs/sessions)).

- `omp -c` continues the most recent session in this cwd, preferring a per-terminal breadcrumb so tmux panes don’t steal each other.
- `omp -r` / `/resume` picks; a short id prefix is enough.
- `/branch` starts a new leaf in the same file. `/fork` (or `omp --fork`) clones into a new file with a `parentSession` marker.
- `/tree` moves the leaf without forking. Shift+L labels survive compaction.
- `/compact` summarizes the older half of the active branch; the file on disk still holds the pre-compact history.
- `/fresh` resets provider stream state (stale prompt cache, wedged stream) without wiping the local transcript.
- `/handoff` writes a structured wrap-up. `/share` renders HTML and uploads a secret gist via `gh`, or calls `~/.omp/agent/share.ts`. `/export` is the HTML without sharing.
- `--no-session` is ephemeral and disables `/fork`, `/export`, and `/share`.

**`/collab`.** Live session on a relay, link plus QR. `omp join` or a browser guest. `/collab view` is read-only. Frames are sealed client-side. v18.6.1 lets guests answer setting-change and tool-issue prompts.

---

## Standout UX, beyond the tool list

- **In-process shell.** `bash` is brush plus a builtins crate (README: 58 utilities in one place, 46 “coreutils” in the tool blurb, 67 in the crate table — the counts don’t match). No WSL required; the same binary is the Windows port. Persistent shell sessions, optional PTY.
- **`eval`.** Persistent Python and JavaScript by default; Ruby and Julia are opt-in. Kernels can call back into agent tools.
- **Stream rules.** A regex match aborts the stream mid-token, injects a dormant rule, and retries. Injections survive compaction, so you don’t pay the rule on every turn.
- **Internal URLs** through the same `read`/`grep` you use on files: `pr://`, `issue://`, `agent://`, `skill://`, `ssh://`, `conflict://`, `xd://`, plus `artifact://` and `memory://` in the docs conventions. `agent://…/findings.0.path` pulls a field out of a subagent result.
- **`ask`.** Option picker mid-turn, also over ACP.
- **`omp commit`.** Splits the working tree into dependency-ordered atomic commits; cycles rejected before anything is written; lockfiles excluded from analysis. Push is opt-in.
- **Browser.** Headless Chromium, CDP attach, or a relay extension that adopts Chrome tabs you already have open. README also claims the same API can drive Electron apps (Slack called out). Stealth on by default.
- **`computer`.** Real desktop: windows, screenshots, native input, accessibility tree, clipboard. **Off by default** (`computer.enabled`). Not the same as `browser`.
- **Git diff pane.** Space stages or unstages the focused hunk (v18.6.1).
- **`/btw` in Tern.** Side questions scroll instead of clipping (v18.6.1, [#14331](https://github.com/can1357/oh-my-pi/pull/14331)).
- **Memory.** `memory.backend` is `off` by default. Tools page: `recall` / `retain` / `reflect` need `hindsight` or `mnemopi`; `memory_edit` is Mnemopi only; `learn` also accepts `local` but needs `autolearn.enabled`. README’s “local, Hindsight, or Mnemopi” is the backend enum, not “every memory tool is on.”
- **Gated off by default** (README): `github`, `security_scan`, `generate_image`, `tts`, `checkpoint`, `rewind`, and the memory tools. `inspect_image` appears automatically when the active model can’t see.
- **`tools.xdev`** (on by default) keeps rare tools behind `xd://` so their descriptions aren’t in the prompt up front. `read xd://` lists them.
- **Approvals.** Persistent mode defaults to **`yolo`** (Read, Write, and Exec all auto-allowed). `always-ask`, `write`, or per-capability `allow`/`prompt`/`deny` exist. An approval prompt is not a sandbox.
- **Extensions.** TypeScript modules with the same tool, slash, and hotkey APIs. `/reload-plugins` hot-reloads. Plugins can ship to a marketplace or npm.
- **Voice crate** (`pi-voice`: Opus, WebRTC) and v18.5’s GPT live-voice RPC events are in the overview’s release notes; this pass did not re-read the voice docs, so treat “live voice” as release-note, not a verified user workflow.

---

## What people highlight

X search 2026-10-05, English, not from @grok, roughly July–September 2026. The repeated nouns are **advisor**, **LSP**, **hashline**, and **subagents**. MCP inheritance is a docs feature; it did not show up as something people brag about. Posts that mention MCP are usually “OMP is one of N clients,” not “it imported my `.cursor` config.”

- [brandon galang](https://x.com/brandon_galang/status/2073076052390478077) (2026-07-03): favorite addition is the advisor, “another model riding shotgun.”
- [Jared Smith](https://x.com/woodchipdaddy/status/2073103934621962641) (2026-07-03): Anthropic and ChatGPT subscriptions, configurable `task` subagents, advisor “like my eyes.”
- [Gokay Yilmaz](https://x.com/gokayai/status/2089260078155112692) (2026-08-17): “built in LSP is the hack,” annoyed it was missing from a harness tier list.
- [NOVA](https://x.com/N0V4Dev/status/2075385244178325739) (2026-07-10): LSP ops so “everything your IDE knows” is in the session.
- [Chris Short](https://x.com/ChrisShort/status/2081774084073922671) and [EveryDev.ai](https://x.com/EveryDevAi/status/2081889375659684007) (2026-07-27): the short list they repeat is hash-anchored edits, LSP, DAP, subagents, browser.
- [brrrock](https://x.com/bchap1n/status/2076878521398595957) (2026-07-14): didn’t import Claude Code config; set a custom PowerShell LSP and fallbacks per model role.
- [aaronjmars](https://x.com/aaronjmars/status/2100682204946796884) (2026-09-17): ranks omp first for “deploy today,” citing `omp -p` JSONL (tokens, USD, tool events) and a profile-isolated credential vault.
- [Kevin Gray](https://x.com/graykevinb/status/2100473392080064761) (2026-09-17): recommends OMP over Pi specifically for subagents.

---

## Caveats / what it is not

- **Not Pi.** Pi’s pitch is four tools and a small prompt. OMP’s pitch is the opposite. The context-weight complaint (a large starting prompt, most tools unused) is argued in the overview digest’s July 2026 HN thread; this note did not re-fetch that thread.
- **Not a sandbox.** Default approval mode is `yolo`. MCP stdio servers, project extensions, and `computer` run as you. Isolation worktrees are for sibling agents not stepping on each other’s files, not a security boundary.
- **Not “31 tools always in the prompt.”** Several are off until you flip a setting (`github`, memory, `computer`, `ast_grep`, image, tts, security scan). `tools.xdev` hides more behind `xd://`. `/tools` is the live list.
- **Not a Node library.** Bun SDK, or RPC/ACP if the host isn’t Bun.
- **Not a quiet Windows port, and not “needs WSL.”** Native Windows is a stated goal; 18.5–18.6 still shipped a pile of Windows fixes (see the overview).
- **Not done with compaction or hashline edge cases.** v18.6.1 is still patching screenshot token estimates and shift-consistent anchors.
- **Counts drift inside the README** (coreutils 46 vs 58 vs 67). Cite the docs page for whether a tool is on, and the release notes for what last week’s patch actually changed.
- **omp²** is a homepage tease (“soon”), not a feature in this tree.
- **`security_scan` “drives Codex Security cloud scans”** is a README phrase for an off-by-default tool. This note did not verify that cloud path.

---

## Sources

- README, `main`, fetched 2026-10-05: [can1357/oh-my-pi](https://github.com/can1357/oh-my-pi/blob/main/README.md)
- Docs: [overview](https://omp.sh/docs) · [tools](https://omp.sh/docs/tools) · [sessions](https://omp.sh/docs/sessions) · [sdk](https://omp.sh/docs/sdk) · [mcp](https://omp.sh/docs/mcp) · [providers](https://omp.sh/docs/providers)
- Homepage (no separate features page): [omp.sh](https://omp.sh)
- Release: [v18.6.1](https://github.com/can1357/oh-my-pi/releases/tag/v18.6.1) (2026-10-04 16:12 UTC / 2026-10-05 00:12 CST)
- X posts linked inline. Searches: feature keywords and MCP-inherit keywords, `lang:en`, excluding @grok, 2026-10-05.
- Prior overview, not rewritten: [[Digests/2026-10-05 OMP agent TUI]]
