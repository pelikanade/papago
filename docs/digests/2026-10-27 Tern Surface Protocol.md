---
title: Tern Surface Protocol (TSP)
date: 2026-10-27
tags:
  - digests
  - tern
  - tsp
  - omp
  - agents
  - terminal
type: digest
aliases:
  - TSP
  - Tern Surface Protocol
  - omp Tern UI
publish: true
---

# Tern Surface Protocol (TSP) — how Tern paints omp’s polished UI

**Ask:** dedicated digest on the Tern Surface Protocol — mechanism, why the UI feels good, maturity. Not a rewrite of the Oct 12 mega-note.

**Checked:** 2026-10-27 (Asia/Shanghai). Sources: [TSP overview](https://docs.stencil.so/tern/protocol/), [transport](https://docs.stencil.so/tern/protocol/transport.html), [handshake](https://docs.stencil.so/tern/protocol/handshake.html), [work kinds](https://docs.stencil.so/tern/elements/work.html), Can [@can1357](https://x.com/can1357/status/2111635804053443026) (2026-10-18), [@__kutluk](https://x.com/__kutluk/status/2106783863767019786), vault [[2026-10-12 omp and Tern Surface Protocol]], [[2026-10-27 OMP RPC mode and Tern UI]].

> [!note] Twin channel
> **RPC is the host-control channel, not this.** Drive omp from a daemon with `omp --mode rpc` / `rpc-ui` (NDJSON over stdio). TSP is how *interactive* omp describes UI to Tern over the pty. See [[2026-10-27 omp RPC mode]].

Evidence tags: **[doc]** official docs · **[post]** person / third-party · **[src]** prior vault notes (package / checkout; not re-run live this turn).

> [!abstract] TL;DR
> omp stops painting cells. It sends a **semantic component tree** over in-band APC (`ESC _ tsp ; … ESC \`). Tern owns layout, draw, animation, hover, and scroll. Handshake is hello + DA1; no reply → ANSI fallback. That retained, GPU-drawn tree — foldable `tool` cards, streaming `md` append, Tern-clocked spinners, real `prefs`, composer `dock`, ask overlays — is why omp in Tern feels like a GUI. Young, single-vendor, closed-beta Tern. Human in Agent pane → interactive omp + TSP; automation → RPC, not TSP scrape.

Related: [[2026-10-12 omp and Tern Surface Protocol]] · [[2026-10-27 OMP RPC mode and Tern UI]] · [[2026-10-27 omp RPC mode]] · [[2026-10-05 OMP agent TUI]]

## 1. What TSP is [doc]

The Tern Surface Protocol lets a program in a pane describe its UI as a tree of **semantic components**. Tern lays that tree out, draws it, and animates it natively. A TSP program **stops painting cells**: no box-drawing glyphs, no wrap/truncation math, no spinner timers, no redraw-on-resize. It sends intent — “a card, pending, body these ANSI bytes, folded to 10 lines” — and Tern decides how it looks. [doc]

| Concern | Owner |
|---------|--------|
| What is shown (nodes + frames of ops) | Program |
| Layout, animation, hover, scroll position | Tern |
| Keys, editors, modes, history | Program |
| Fold / checkbox click (reported back) | Tern flips immediately; program mirrors |

Every node stays addressable by id while its surface is open — including deep scrollback — until Tern drops it under retention (`gone`). There are no “final” rows: a card that scrolled away can still change. [doc]

omp is the flagship speaker (coding agent). The same protocol serves CLIs that ask one question and exit, and plugin views Tern wraps for you. Spec: [docs.stencil.so/tern/protocol/](https://docs.stencil.so/tern/protocol/). Long protocol dump, Softwared advice, plugin examples: [[2026-10-12 omp and Tern Surface Protocol]].

## 2. Transport [doc]

TSP rides **in-band on the pty** as APC strings — ordered with every other byte, works over **ssh**, and Tern’s **session daemon** can keep/replay surfaces with no second channel. [doc]

```text
ESC _ tsp ; <verb> [; key=value]* ; <body> ESC \
```

Body is UTF-8 JSON (base64 for blobs). Unknown verbs/fields are ignored both ways; incompatible change bumps version. [doc]

| Verb | Direction | Role |
|------|-----------|------|
| `q` / `o` / `f` / `b` / `t` / `s` / `x` | program → terminal | query, open surface, frame, blob, palette, stylesheet, close |
| `r` / `e` | terminal → program | reply, event |

Chunking when body > `apc` (default 65 536): `c=<token>;m=1` on non-final chunks. Tern drops a chunked message over 24 MiB, after 1 s silence, or if interrupted. Blobs are SHA-256 content-addressed, max 16 MiB. [doc]

**Windows:** ConPTY’s input parser discards APC, so **terminal → program** uses **OSC 877** (`ESC ] 877 ; tsp ; … ESC \`) with the same body. Program → terminal stays APC; Tern bundles its own ConPTY so APC survives. Accept both framings. [doc]

tmux / screen / zellij swallow APC — don’t probe inside them (docs; Can has also posted that TSP-in-tmux may be improving — treat docs as conservative). [doc][post]

## 3. Handshake and fallback [doc]

Program writes hello + DA1:

```text
ESC _ tsp;q;{"q":"hello","v":[1],"app":"omp","features":["edit","undo","send"]} ESC \  ESC [ c
```

A TSP terminal replies **before** the DA1 answer (`tsp;r` with `term:"tern"`, `kinds`, `features`, `apc`, `credits`, `cols`, …). DA1 first with no reply → **no TSP**: keep ANSI. Tern speaks only version `1`. [doc]

- **`TERM_PROGRAM=tern`** (and not in a multiplexer): optimistic start — send `o` + first `f` immediately, reconcile when reply lands (omp does this). Env does not survive ssh; the query does. [doc]
- **`PI_TUI_NATIVE=0`**: force ANSI. Tern’s “Native program surfaces” setting also makes Tern never answer hello. [doc][src]

Program features that matter for agents: `edit` (native selection edits), `undo`, `send` (whole-prompt submit — also needs node `sendable:true`). Terminal features include `dock`, `blobs`, `styles`, `flow`, `reduce-motion`, `scroll`, … [doc]

## 4. Surfaces, frames, flow control [doc]

A **surface** has regions: **`main`** (flowing content), **`dock`** (sticky bottom — composer, HUD), **`layer`** (overlays / sheets). Modes include `inline`, `screen`, and `flow` (cursor-row among output). [doc]

A **node** is `{id, k, p, c}` — id, kind, props, children. A **frame** `{sf, s, ops}` is an atomic batch with a per-surface sequence. Ops include `add` / `set` / `text` / `splice` / `move` / `del` / `settle` / `focus` / `reveal` / `scroll` / … Streaming Markdown grows via **`text` append** without resending the whole body. [doc][src]

**Credits / ack:** at most `credits` (default 2) unacknowledged frames; Tern sends `ack` once a frame is drawn, so omp’s render cadence follows the terminal. [doc]

## 5. Agent-relevant kinds [doc]

From [work elements](https://docs.stencil.so/tern/elements/work.html) and the broader vocabulary:

| Kind | Role in omp |
|------|-------------|
| `tool` | One step: status head + collapsible body (bash/read/edit/…) |
| `agent` | Subagent row: status, model, tools, tokens/cost |
| `checklist` | Todo phases; HUD / dock pill modes |
| `prefs` | Whole `/settings` as a real settings page |
| `editor` / `picker` | Composer / model·session·rewind hubs |
| `md` / `diff` / `meter` / `chart` / `effort` | Prose, patches, usage visuals |

`tool` heads are data-driven (icon, verb, target with `targetKind`, status, timer from `age`/`took`, exit chip). Collapse + `preview: { lines|tail }` is Tern-local fold state. Broader kinds (`card`, `spinner`, `shimmer`, `elapsed`, `rows` ANSI fallback, …) fill the rest. [doc]

## 6. Input [doc]

- **Keys** still arrive as raw pty input (kitty keyboard, bracketed paste). Editors stay omp’s state machines. [doc]
- **Pointer / view** are semantic events: `action`, `select`, `activate`, `toggle`, `change` (prefs), `focus`, `visible`, `resize`, `gone`, `error`. Tern handles `copy` / `open` / `zoom` itself. [doc]
- **Native edit** (`edit` feature): Tern keeps selection in the composer; sends `{from,to,text,cursor,len}` in UTF-16 offsets. [doc]
- **`send`:** `{"ev":"send",…,"text":"…"}` submits a whole prompt through the normal path (multi-line, commands — no fake Enter). Only when the node reports `sendable:true` (omp sets after startup). How Tern / Carly starts an agent with an initial prompt. [doc][src]

## 7. Why the UI feels good [doc][post]

Mechanism, not marketing:

1. **Retained semantic tree** vs cell painting — no wrap math in the agent; cards deep in scrollback stay live.
2. **GPU chrome** (wgpu Tern) + native font mixing; program never measures text.
3. **Foldable `tool` cards** with real status/timers/previews; optional work-block folding.
4. **Streaming text append** — Markdown grows without full resend.
5. **Tern-clocked motion** (`spinner` / `shimmer` / `elapsed`); Reduce Motion honored; no per-frame spinner traffic from omp.
6. **`prefs` as a real settings sheet**, not a fake TUI form.
7. **Composer in `dock`**; **`ask` overlays** can sit in the composer slot (hide/show while reading transcript).
8. **Side signals (not TSP):** OSC 9;4 progress (working / idle / waiting_input); OSC 1337 `omp_session_file` for daemon resume under Tern. [doc][src]

**Field quotes:**

- Can [@can1357](https://x.com/can1357/status/2111635804053443026) (2026-10-18): *“when it's running under tern omp cli negotiates support for tern surface protocol and upgrades itself to the retained UI you see.”* [post]
- [@__kutluk](https://x.com/__kutluk/status/2106783863767019786): *“omp gui and terminal multiplexer at the same”* — the phrase people use for the combo. [post]

## 8. Maturity [doc][post][src]

- Tern is **closed beta** (Stencil Labs / Can Bölük); app closed source; public piece is `stencil-hq/tern-sdk` (MIT).
- TSP is **v1-only**, additive evolution, **single vendor** — a TSP program is useful only inside Tern (or a hypothetical second host).
- Fast churn; docs vs pi-wire types already disagree in places; real bugs have hit the path (oversized first frame vs 24 MiB cap, ConPTY OSC 877 input, etc.). Bug list and Softwared/plugin guidance: [[2026-10-12 omp and Tern Surface Protocol#5. Maturity, caveats, field reports]].
- Hermes and others chase **TSP presentation parity** with omp-in-Tern — that is TSP work, not omp RPC. [post]

## 9. Practical pick

| Goal | Use |
|------|-----|
| Human sitting in Tern Agent pane | **Interactive omp + TSP** (default) |
| Drive omp from ade / Paseo / Softwared / a daemon | **RPC** — [[2026-10-27 omp RPC mode]] |
| Automate without a human pane | **RPC** — do **not** scrape TSP or the pty for control |

Tern is irrelevant on a headless box: without a TSP terminal, omp stays on ANSI (interactive) or never renders a TUI (RPC). [doc]

## Unverified

- Did **not** run omp or Tern live this turn; facts are from public docs, cited X posts, and prior vault digests.
- Private `crates/tern/SURFACE_PROTOCOL.md` not read — public docs + prior pi-wire notes only; do not invent crate contents.
- Whether TSP-in-tmux is fully reliable yet; live Tern version / beta matrix.
- For the long protocol dump, plugin patterns (tern-model-usage, pi-tern), and Softwared Go-client path, prefer [[2026-10-12 omp and Tern Surface Protocol]].

## Sources

- https://docs.stencil.so/tern/protocol/
- https://docs.stencil.so/tern/protocol/transport.html
- https://docs.stencil.so/tern/protocol/handshake.html
- https://docs.stencil.so/tern/elements/work.html
- https://x.com/can1357/status/2111635804053443026
- https://x.com/__kutluk/status/2106783863767019786
- https://github.com/stencil-hq/tern-sdk
- Prior vault: [[2026-10-12 omp and Tern Surface Protocol]] · [[2026-10-27 OMP RPC mode and Tern UI]] · [[2026-10-27 omp RPC mode]]
