# omp child processes spawn on demand and stop when idle

Decision: the Daemon spawns an Agent's omp child process only when work arrives, keeps it alive across turns, and stops it after an idle TTL (default 15 minutes, configurable) once the session is settled — not streaming, no queued messages, no pending async work. The next prompt transparently resumes the Session via `open_session`; Clients do not observe the difference. Rationale: every live omp is a Bun process; a Daemon hosting many Agents cannot keep them all resident, and settled-idle is a safe stop point where resume is cheap.
