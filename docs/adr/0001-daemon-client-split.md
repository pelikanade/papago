# Daemon and Client are separate from day one

papago adopts Paseo's working logic: agents belong to the Daemon, the UI is just a Client. Decision: from v0 there is a standalone Daemon process (owning every omp child process, Workspace, and all state), and the GUI and CLI connect to it as Clients over the same local protocol. Rationale: agents must not die because a window closed — that is the ADE's core job; and once the protocol seam exists, a CLI, remote access, and multiple clients all grow out of the same seam at almost no extra architectural cost.

## Considered Options

- Single process (the GUI owns the omp children directly): simplest, but closing the window kills agents and recovery leans entirely on omp session resume; the protocol becomes an afterthought. Rejected.
