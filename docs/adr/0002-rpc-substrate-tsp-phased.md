# RPC is the control substrate; the TSP host is a later phase

Status: partially superseded by [ADR-0009](0009-no-tsp.md) — RPC-as-control-substrate remains in force; the TSP-host phase and the attach-semantics paragraph below no longer apply.

omp has two channels: NDJSON RPC (the host control plane) and TSP (the presentation protocol interactive omp uses to paint a semantic UI over a pty; v1-only, single-vendor, closed beta, fast-churning). Decision: v0 uses `omp --mode rpc-ui` as the only control plane, mapping RPC events onto papago's native Timeline. The TSP host — embedding interactive omp in a pty with papago acting as the terminal side that renders its semantic component tree — is the phase-2 flagship "attach" feature. TSP's ideas (retained semantic component tree, streaming text append, foldable tool cards) guide the native Timeline design from the start. Rationale: contain MVP risk without giving up the UI-fidelity differentiator; the bet on an unstable spec is confined to one isolatable pane.

Attach semantics — whether the TSP pane binds to an Agent's Session or runs as an independent interactive omp inside the Workspace — are deliberately left open, to be decided when phase-2 design starts.
