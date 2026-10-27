# AGENTS.md

## Documentation

- All documentation is written in English: the README, `GLOSSARY.md`, ADRs in `docs/adr/`, and everything under `docs/`.

## Prototype (`prototype/`)

- Use Tailwind's size system (`text-sm`, `h-9`, `w-72`, `p-3`, …). Do not use arbitrary px values in Tailwind classes (`text-[15px]`, `h-[30px]`) unless the user explicitly specifies them.
- Icons are Nerd Font glyphs only: register them in `prototype/src/icons.ts` and render through `<Icon>` (`.nf` = FiraCode Nerd Font Mono). Never use emoji, plain-text arrow characters, or custom-drawn SVG icons — no inline `<svg>` icons and no icon SVG assets anywhere in the prototype.
- Timeline items share one visual grammar: a borderless row of `status icon · type label · muted detail · right-aligned timing`. Expandable items show a caret instead of the status icon on hover; only the expanded body gets a background and border.
- Record every design decision as a comment in the component that implements it (the prototype is the carrier; comments preserve the rationale for the Go port).
- When the user comments on a prototype component without requesting a change, capture that comment as a code comment in the relevant component — remarks are part of the design record, not chat noise.
- Componentize as soon as a design decision is stable enough: extract the reusable component instead of repeating markup across surfaces (e.g. timeline rows, status-bar segments, tree rows). A stable pattern that appears twice is a component, not a copy.
- React effects are banned in the prototype: no `useEffect` (enforced by the ESLint `no-restricted-syntax` rule). Drive side effects from zustand store actions or the module bootstrap in `src/main.tsx`; use CSS (`field-sizing-content`) or DOM observers where an effect would be the reflex.
- The React + Tailwind prototype is the single source of truth for UI and interaction design. Do not create separate UI/behavior design documents; iterate on the prototype itself.
- The prototype toolchain is **bun**: `bun install`, `bun run dev`, `bunx tsc --noEmit`. Never use npm/pnpm/yarn in this repo.
- Prototype verification — visual checks (screenshots) and interaction tests — runs through **agent-browser** (`agent-browser …` CLI, headless Chromium over CDP): open the dev server URL, snapshot/click/fill, then screenshot. Never use Tern's managed Chromium or the omp `browser` helper for this repo.
- All view components are pure functions of the zustand state machine: they read state through selectors and change it only by dispatching explicit store actions. No domain state lives in component-local state.
- The state machine (`prototype/src/store.ts`) is the contract for the future Go implementation: keep transitions explicit, named, and serializable so the Go daemon/client can mirror them one-to-one.
