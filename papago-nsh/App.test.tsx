// The Rust root tested from TypeScript: `nsh test` runs this file in a
// headless runner it builds with the crate, and `open(".")` starts the root
// (`nsh.json` "root"): the mygo app's TestView and
// TestTimelineSelectAndCopy, ported. Selectors note: an input matches its
// placeholder only while empty — once a prompt is typed, the input matches
// the text it holds.
import { expect, open, test } from "nsh/test";

test("view", async () => {
  const app = await open(".", { width: 1280, height: 720 });
  await app.waitFor("fix-login bug");
  expect(app.texts()).toContain("src/auth/session.ts");

  // The project row IS its orchestrator: clicking it opens that agent, and
  // a prompt steers that agent (Enter = prompt).
  await app.press("label:myrepo · orchestrator");
  await app.waitFor("orchestrator");
  await app.type("What are we cooking?", "plan the health endpoint work");
  await app.submit("text:plan the health endpoint work");
  await app.waitFor("plan the health endpoint work");

  // The project page's composer dispatches: a workspace, its worktree and
  // its worker are created together (1:1) — the spawn transition, reached
  // through the palette (⌘K) as a user would.
  await app.keyboard.press("mod+k");
  await app.waitFor("Open Agent Hub");
  await app.press("label:p:p1");
  await app.waitFor("read · plan · dispatch — workers own writes");
  await app.type("New Agent in a fresh worktree…", "add a health endpoint");
  await app.submit("text:add a health endpoint");
  await app.waitFor("add a health endpoint");

  // Alt+Enter queues instead of sending (DEC-014).
  await app.type("What are we cooking?", "then bump the deps");
  await app.keyboard.press("alt+enter");
  await app.press("text:1");
  await app.waitFor("then bump the deps");

  // And the project counts one more workspace (seen back on its page).
  await app.keyboard.press("mod+k");
  await app.waitFor("Open Agent Hub");
  await app.press("label:p:p1");
  await app.waitFor("read · plan · dispatch — workers own writes");
  expect(app.texts().some((t) => t.includes("7 workspaces"))).toBe(true);
});

test("timeline click still folds", async () => {
  const app = await open(".", { width: 1280, height: 720 });
  await app.press("text:triage-482");
  await app.waitFor("triage issue #482");
  // The rendered md: the code spans draw without their backticks.
  const md = "Lint failed with 3 errors in src/auth/session.ts. Paused here; rerun after the fix lands.";
  await app.waitFor(md);
  // A click still folds (the bash row's preview body shows; waitFor's
  // not-found path spins on an app that renders every tick, so wait a
  // bounded time and check — the body renders as one multi-line text).
  await app.press("text:Bash");
  await app.wait(600);
  expect(app.texts().some((t) => t.includes("ok  api/rate  8.1s"))).toBe(true);
});
