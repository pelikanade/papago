# Agents run yolo by default; only Asks enter the queue

omp's default approval mode is already yolo (reads, writes, and execs all auto-allowed), and an approval prompt was never a sandbox. papago targets the owner's machines and repositories, where throughput and low interruption win. Decision: Agents papago launches run with yolo approvals by default; only ask-style interactions (`extension_ui_request` and friends) are bridged over rpc-ui into the Permission queue, and they must be dispatched promptly — ignoring these frames deadlocks the agent, as the omp docs explicitly warn. The safety boundary is worktree isolation, not approval prompts.

Per-Agent override: the Approval mode is selectable at Agent creation (yolo / write-ask / always-ask); the default stays yolo.
