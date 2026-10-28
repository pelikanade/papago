//! The papago view tests, ported from the mygo app's `main_test.go` (the
//! mygo-maintenance skill's Tester for UI regressions): the view runs
//! without a window in tests, which click and type as a user would.

use nsh_core::platform::Os;
use nsh_core::prelude::*;
use nsh_engine::{EngineConfig, Root};
use nsh_testing::Headless;

/// The prototype's window: 1280×720 (the mygo tests drive 1280×640).
fn engine() -> EngineConfig {
    let mut config = EngineConfig::new(Root::of::<nsh_app_papago::App>());
    config.viewport = Size { width: px(1280.0), height: px(720.0) };
    // The prototype is a desktop app: run as Linux, where a drag selects
    // text and Ctrl+C copies it (the mygo window is the same).
    config.environment.os = Os::Linux;
    config
}

#[test]
fn view() {
    let mut app = Headless::start(engine()).unwrap();
    // The window opens on the first worker with its mock run.
    app.wait_for("fix-login bug").unwrap();
    assert!(app.shows("src/auth/session.ts"), "timeline missing");

    // The project row IS its orchestrator: clicking it opens that agent, and
    // a prompt steers that agent (Enter = prompt). The composer's input
    // matches its placeholder only while empty: once the prompt is typed,
    // the input matches the text it holds.
    app.press("label:myrepo · orchestrator").unwrap();
    app.wait_for("orchestrator").unwrap();
    app.type_text("What are we cooking?", "plan the health endpoint work").unwrap();
    app.submit("text:plan the health endpoint work").unwrap();
    app.wait_for("plan the health endpoint work").unwrap();

    // The project page's composer dispatches: a workspace, its worktree and
    // its worker are created together (1:1) — the spawn transition. Reach
    // the project page through the palette (⌘K), like a user would.
    app.press_key("mod+k").unwrap();
    app.wait_for("Open Agent Hub").unwrap();
    app.press("label:p:p1").unwrap();
    // The project page: the orchestrator card + the workspace cards.
    app.wait_for("read · plan · dispatch — workers own writes").unwrap();
    app.type_text("New Agent in a fresh worktree…", "add a health endpoint").unwrap();
    app.submit("text:add a health endpoint").unwrap();
    // The spawned worker is selected: its pane header is the prompt's first
    // five words.
    app.wait_for("add a health endpoint").unwrap();

    // Alt+Enter queues instead of sending (DEC-014): the queue segment
    // shows one item; opening it shows the follow-up.
    app.type_text("What are we cooking?", "then bump the deps").unwrap();
    app.press_key("alt+enter").unwrap();
    app.press("text:1").unwrap();
    app.wait_for("then bump the deps").unwrap();

    // And the project counts one more workspace (seen back on its page).
    app.press_key("mod+k").unwrap();
    app.wait_for("Open Agent Hub").unwrap();
    app.press("label:p:p1").unwrap();
    app.wait_for("read · plan · dispatch — workers own writes").unwrap();
    assert!(
        app.screen().texts().iter().any(|t| t.contains("7 workspaces")),
        "project sub after a spawn: {:?}",
        app.screen().texts()
    );
}

/// The mygo TestTimelineSelectAndCopy's second half: a click on a row
/// still does the row's action (the timeline is a selectable view — the
/// headless driver does not simulate the platform's drag-selection and
/// Ctrl+C; the real GTK app under Xvfb does, verified by hand).
#[test]
fn timeline_click_still_folds() {
    let mut app = Headless::start(engine()).unwrap();
    app.wait_for("fix-login bug").unwrap();
    // Short timeline: the md paragraph is on screen.
    app.press("text:triage-482").unwrap();
    app.wait_for("triage issue #482").unwrap();
    // The rendered md: the code spans draw without their backticks.
    const MD: &str = "Lint failed with 3 errors in src/auth/session.ts. Paused here; rerun after the fix lands.";
    app.wait_for(MD).unwrap();
    // A click still folds (the bash row's preview body shows). waitFor's
    // not-found path spins on an app that renders every tick, so wait a
    // bounded time and check.
    app.press("text:Bash").unwrap();
    app.wait(std::time::Duration::from_millis(600)).unwrap();
    assert!(
        app.screen().texts().iter().any(|t| t.contains("ok  api/rate  8.1s")),
        "fold body missing: {:?}",
        app.screen().texts()
    );
}
