# No code editor, ever

Decision: papago is an ADE, not an IDE. It ships read-only file browsing and (in a later phase) read-only diff review whose comments flow back to the Agent; no editable code surface is ever introduced. Rationale: typing code is the agent's job; an editor drags in dependencies and scope creep (keybindings, buffers, an LSP client) and collides head-on with the "running and reviewing agents" positioning.
