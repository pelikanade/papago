---
title: omp RPC mode
date: 2026-10-27
tags:
  - digests
  - omp
  - rpc
  - agents
  - harness
type: digest
aliases:
  - omp rpc
  - rpc-ui
  - omp NDJSON
publish: true
---

# omp RPC mode (`omp --mode rpc` / `rpc-ui`)

**Ask:** dedicated digest on omp RPC mode — NDJSON host-control over stdio, when to use it, lifecycle, prompt completion, steer/abort, protocol v1/v2, commands/events sketch, extension UI, host tools, clients, and the Noa pick.

**Checked:** 2026-10-27 (Asia/Shanghai). Sources: [omp RPC](https://omp.sh/docs/rpc), [omp modes](https://omp.sh/docs/modes), [oh-my-pi `docs/rpc.md`](https://github.com/can1357/oh-my-pi/blob/main/docs/rpc.md), prior [[2026-10-27 OMP RPC mode and Tern UI]], [[2026-10-12 omp and Tern Surface Protocol]].

Evidence tags: **[doc]** official docs · **[src]** prior vault / checkout notes (not re-run this turn) · **[post]** person / third-party.

> [!abstract] TL;DR
> `omp --mode rpc` (or `rpc-ui`) is a **custom NDJSON protocol over stdio** — not JSON-RPC 2.0. Spawn omp as a long-lived child, wait for `ready`, drive prompts/steer/abort/state over stdin, parse responses + events on stdout. Use it to embed omp from ade / Paseo / Softwared / any daemon. Leave Tern’s polished agent UI to interactive omp + TSP ([[2026-10-27 Tern Surface Protocol]]); do not scrape the pty for control.

Related: [[2026-10-27 OMP RPC mode and Tern UI]] · [[2026-10-12 omp and Tern Surface Protocol]] · [[2026-10-10 Paseo]] · [[2026-10-05 OMP agent TUI]] · [[2026-10-05 OMP features and highlights]]

> [!note] Not TSP
> Tern’s GUI path is **Tern Surface Protocol** (semantic UI over APC on the pty), not RPC. One-line split: host/orchestrator → **RPC**; human in a Tern pane → **interactive omp + TSP**. Details: [[2026-10-27 Tern Surface Protocol]].

## 1. What it is [doc]

From [omp.sh/docs/rpc](https://omp.sh/docs/rpc) and [modes](https://omp.sh/docs/modes): run omp as a **long-lived subprocess** and control it with **one JSON object per line** on stdin/stdout. Official wording: *custom JSONL protocol, not JSON-RPC 2.0*. Correlate commands on optional `id`. [doc]

**Choose RPC when** you own the host process (any language with subprocess + JSON), need a long-lived session, streaming, cancel, state controls, or host callbacks. Prefer other surfaces when those fit better: [doc]

| Surface | When | What you see | How it ends |
|---------|------|--------------|-------------|
| Interactive TUI | Person in a terminal | Full editor, dialogs, tool views (ANSI, or TSP under Tern) | `/exit`, Ctrl+D |
| `-p` / `--mode text` | One plain answer | Final assistant text on stdout | Exits after turn |
| `--mode json` | One-shot event stream | JSONL events; **no** RPC commands | Exits after turn |
| SDK | TS in-process | Typed APIs, no process boundary | App lifecycle |
| ACP (`omp acp`) | Editor that already speaks Agent Client Protocol | Editor owns UI | Client disconnect |
| `--mode rpc` | Long-lived embed | `ready` + NDJSON responses/events; **no TUI** | stdin EOF / shutdown |
| `--mode rpc-ui` | Embed + host-rendered asks | Same wire as RPC; UI-capable bridge | stdin EOF / shutdown |

**Start:** [doc]

```sh
omp --mode rpc --no-session
```

Omit `--no-session` to persist under omp’s normal session store. Normal model / provider / cwd / config flags still apply. **`@file` CLI args are rejected** — send file references in the prompt instead. [doc]

## 2. Lifecycle [doc]

1. Spawn `omp --mode rpc` (or `rpc-ui`) with **separate** stdin, stdout, and stderr.
2. Wait for the `ready` frame before sending commands.
3. Read stdout **continuously** — a host callback or UI request can arrive while a command is outstanding.
4. Send commands as compact, unchunked JSON objects followed by `\n`; flush after each line.
5. **Close stdin** to disconnect. omp rejects pending UI/tool/URI callbacks, drains accepted commands, disposes the session, and exits `0` (session-persistence failure at disposal can exit `1` after a `notice`). [doc]

**Stdout = protocol only; stderr = logs.** Never merge stderr into the JSON parser. Keep reading stdout after closing stdin until EOF — a client that closes stdin but stops draining stdout can delay exit indefinitely. [doc]

Minimal shape of the opening exchange: [doc]

```json
{"type":"ready","protocolVersion":1,"supportedProtocolVersions":[1,2],"maxFrameBytes":1048576,"maxReassembledFrameBytes":67108864}
{"id":"p1","type":"prompt","message":"Reply with only the word ok"}
{"id":"p1","type":"response","command":"prompt","success":true}
```

(Other events such as `available_commands_update` may interleave after `ready`. Do not assume the `response` always arrives before streaming events.) [doc]

## 3. Prompt completion semantics [doc]

**Ack ≠ done.** `prompt` and `abort_and_prompt` acknowledge **admission/scheduling** immediately (`type: "response"`, `success: true`). That is not turn completion. [doc]

A normal agent run finishes when:

```text
frame.type == "agent_end" and frame.isTerminal != false
```

- `isTerminal: false` → asynchronous delivery or maintenance will resume the session.
- An **absent** `isTerminal` field is terminal-compatible. [doc]

Local / slash handling may emit **no** `agent_end`. Detect that with either: [doc]

- `response.data.agentInvoked === false`, or
- a later `{ "type": "prompt_result", "id": …, "agentInvoked": false }`.

Also watch:

- **`prompt_result`** — `{ type, id?, agentInvoked, status, error?, sessionSettled }`. Written after the response for that `id` when work was not finished locally on the ack. Status values include completed / aborted / error (see canonical docs). [doc]
- **`session_settled`** — fires when background work is done (quiescence). Distinct from “this prompt’s agent_end”; use it (and `get_state.isSettled`) when you care that the session is idle, not only that one turn ended. [doc][src]

`agentInvoked: true` means agent lifecycle events will identify completion. If the field is absent, keep watching events and `prompt_result`. Local commands can also emit `command_output`. [doc]

## 4. Steer / follow_up / abort family [doc]

While a turn is active: [doc]

| Command | Role |
|---------|------|
| `steer` | Insert a user message into the **active** run |
| `follow_up` | Queue a message for **after** the active run |
| `prompt` + `streamingBehavior: "steer"` or `"followUp"` | Second prompt while streaming must set this (camel-case `followUp`) |
| `abort` | Stop the active agent run; wait for both the `abort` response **and** the terminal `agent_end` |
| `abort_and_prompt` | Abort and schedule a replacement prompt (no `data.agentInvoked` / `prompt_result`; rely on lifecycle events or a same-id scheduling error) |
| `abort_retry` | Stop an automatic retry delay |
| `abort_bash` | Stop the current RPC-started shell command |
| `remove_queued_message` / `promote_queued_message` / `abort_and_restore_queue` | Queue surgery (see canonical schema) |

Queue defaults: steering and follow-up modes `"one-at-a-time"` (switchable to `"all"` via `set_steering_mode` / `set_follow_up_mode`). Interrupt mode defaults to `"immediate"`; `"wait"` defers steering until the turn finishes. [doc]

**`bash` exception:** runs concurrently so `abort_bash` can overtake it. Its response may be out of order — **correlate by `id`**. [doc]

Host-side cancels: treat `host_tool_cancel`, `host_uri_cancel`, and UI `method: "cancel"` as cancellation of the referenced host operation. [doc]

## 5. Protocol v1 vs v2 + `rpc_chunk` [doc]

- Startup is **v1**. `ready` advertises `supportedProtocolVersions: [1,2]`, `maxFrameBytes` **1 MiB**, `maxReassembledFrameBytes` **64 MiB**.
- Clients that can reassemble should send `negotiate_protocol` with `protocolVersion: 2` immediately after `ready`. The success response itself is still a v1 physical line; then oversized **logical** stdout becomes an uninterrupted sequence of `rpc_chunk` frames (base64 segments of UTF-8 JSON). [doc]

```json
{"id":"protocol-1","type":"negotiate_protocol","protocolVersion":2}
{"type":"rpc_chunk","chunkId":"rpc-1","index":0,"count":2,"byteLength":1200000,"data":"BASE64_DATA"}
```

Validate `chunkId` / `index` / `count` / `byteLength`; reject interleaved sequences; concatenate, UTF-8-decode, parse **one** JSON object. **Never** dispatch individual chunks as app events. Official TS / Python / Rust / Go clients negotiate v2 when advertised. [doc]

In v1, oversized responses fail with `RPC response exceeded the transport limit`; events may be compacted. Never treat `agent_end.messages` as the only transcript — collect `message_*` events or page with `get_messages_page`. [doc]

## 6. Commands & `get_state` sketch · events sketch [doc]

Canonical schema: [docs/rpc.md](https://github.com/can1357/oh-my-pi/blob/main/docs/rpc.md) (and the summary tables on [omp.sh/docs/rpc](https://omp.sh/docs/rpc)). Do not invent fields beyond those docs. Sketch only: [doc]

**Commands (selection):** `negotiate_protocol`; `prompt` / `steer` / `follow_up` / abort family; `get_state`, `get_messages` / `get_messages_page`, `get_entries`, `get_tree`, `get_last_assistant_text`, `get_session_stats`; `set_model`, `cycle_model`, `get_available_models`; `set_thinking_level`, `cycle_thinking_level`; `compact`, `set_auto_compaction`; `bash` / `abort_bash`; `set_host_tools`, `set_host_uri_schemes`; subagent `set_subagent_subscription` / `get_subagents` / `cancel_subagent` / `steer_subagent`; session `new_session` / `open_session` / `switch_session` / `branch` / `fork` / `handoff` / `set_session_name`; queue mode setters; `set_event_filter`; login / extension UI responses; `goal`, `set_todos`, `set_fast_mode`, `set_cache_warming`, … [doc]

**`get_state` fields (selection):** model, thinking level, `isStreaming` / `isCompacting` / `isSettled`, `sessionFile` / `sessionId`, `messageCount`, `queuedMessages`, `todoPhases`, `contextUsage` (`tokens`, `contextWindow`, `percent`), `usageLimit`, `goal`. [doc][src]

**Events (selection):** `agent_start` / `agent_end`, `turn_*`, `message_*` / `message_update`, `tool_execution_*`, compaction, retry, `queue_update`, `notice`, `model_changed`, subagent frames, `prompt_result`, `session_settled`, `extension_ui_request`, `host_tool_call` / `host_tool_cancel`, `host_uri_request` / `host_uri_cancel`, `extension_error`, `available_commands_update`, `command_output`, `rpc_frame_error`, plus optional live-voice / side-question frames. [doc]

Errors are ordinary `type: "response"` frames with `success: false` and a human-readable `error` — usually leave the process usable. Malformed JSON → recoverable `command: "parse"` (no `id`). [doc]

## 7. Extension UI sub-protocol + deadlock warning [doc]

omp may emit `extension_ui_request`; the host answers with `extension_ui_response` (select, confirm, input, editor, notify, setStatus, setWidget, open_url, cancel, …). Login and some extension dialogs use this path even on plain `rpc`. [doc]

> [!warning] Deadlock
> A host that **ignores** `extension_ui_request`, `host_tool_call`, or `host_uri_request` can **deadlock** the operation that asked. Dispatch these frames while other commands are pending. [doc]

## 8. `rpc` vs `rpc-ui` vs `--no-ui` · env notes [doc]

| Mode / flag | Wire | UI behavior |
|-------------|------|-------------|
| `--mode rpc` | NDJSON stdio | No TUI. Prefer for headless automation that auto-resolves or fails closed on approvals |
| `--mode rpc-ui` | **Same** wire | Marks session UI-capable; bridges tool/extension UI to the host. Does **not** launch a TUI or add tool-card frames. Use only when the host implements those UI behaviors |
| `--no-ui` (with either) | same | Extensions headless: `ctx.hasUI` false; dialogs → defaults; presentation updates dropped. With `rpc-ui`, tool UI such as `ask` can still flow. With `rpc`, no UI requests except host-issued `login` |

**Env side effects:** CLI RPC modes set `PI_NO_TITLE=1` (skip automatic session title generation / extra model call). `rpc-ui` also sets `PI_NO_PTY=1` (disables PTY-backed shell execution). [doc]

## 9. Host tools / host URI schemes [doc]

- Host sends `set_host_tools` with tool definitions → omp calls back with `host_tool_call` (and `host_tool_cancel`); host replies with `host_tool_update` / `host_tool_result`.
- Same pattern for URI schemes: `set_host_uri_schemes` → `host_uri_request` / `host_uri_cancel` → `host_uri_result`. [doc]

Useful for Softwared-style daemons: expose `report_progress`, `open_pull_request`, `ask_maintainer`, etc. as host tools and receive structured callbacks. [src]

## 10. Official clients · Paseo [doc][src]

| Client | Notes |
|--------|--------|
| Go | `github.com/can1357/oh-my-pi/sdk/go/omp-rpc` (`omprpc`) — stdlib only; `Start`, `Frames()`, `PromptAndWait`, negotiate v2 + reassemble |
| TypeScript | Bundled `RpcClient` / `RpcFrameDecoder` (`@oh-my-pi/pi-coding-agent/modes/rpc/…`) |
| Python | Bundled `RpcClient` |
| Rust | Ecosystem client; negotiates v2 when advertised |

**Paseo** already drives omp via `rpc` / `rpc-ui` (provider builds argv, maps events to its timeline, answers `ask` via `rpc-ui`, host-tools / MCP bridge). See [[2026-10-10 Paseo]] and prior Softwared notes in [[2026-10-12 omp and Tern Surface Protocol#4. How a third party can use it]]. [src]

## 11. Practical pick for Noa [src]

| Goal | Use |
|------|-----|
| Drive omp from ade / Paseo / Softwared / a daemon | **RPC** (`rpc-ui` if you need ask/approvals in *your* host UI) |
| Sit in Tern and use omp as a person | Interactive omp + TSP — [[2026-10-27 Tern Surface Protocol]] |
| Automate / orchestrate without a human pane | **RPC** — do **not** scrape TSP or the pty |

Softwared-shaped path: official Go `omp-rpc`, `PromptAndWait`, `get_state`, answer `ExtensionUiRequest`, register host tools. Tern is irrelevant on a headless box. [src]

## 12. Twin / prior notes

- Full RPC≠TSP split (with Tern UI mechanism): [[2026-10-27 OMP RPC mode and Tern UI]]
- Earlier one-screen RPC + Softwared / Paseo section: [[2026-10-12 omp and Tern Surface Protocol]]
- Tern GUI protocol (assume twin exists): [[2026-10-27 Tern Surface Protocol]]

## Unverified / caveats

- Did **not** run omp live this turn; facts are from official docs and prior vault digests.
- Exact live command surface can churn with omp releases — treat [docs/rpc.md](https://github.com/can1357/oh-my-pi/blob/main/docs/rpc.md) as canonical when wiring a client.
- Go module fetch/`go get` from the monorepo subdirectory and per-command parity across Rust/Go clients were not re-verified this turn. [src]

## Sources

- https://omp.sh/docs/rpc
- https://omp.sh/docs/modes
- https://github.com/can1357/oh-my-pi/blob/main/docs/rpc.md
- Prior vault: [[2026-10-27 OMP RPC mode and Tern UI]]
- Prior vault: [[2026-10-12 omp and Tern Surface Protocol]]
