//! The app's Rust UI: its root component, which `nsh run`, `nsh package`
//! and `cargo run` mount. App shell only (App.tsx): window frame + states
//! (normal/maximized/minimized/closed), routing to pages, theme
//! application, and the streaming heartbeat. Views live in `pages`,
//! reusable pieces in `components`.

mod components;
mod data;
mod icons;
mod pages;
mod store;
mod theme;
mod tokens;
mod types;

use std::time::Duration;

use nsh_core::desktop::TitleBarStyle;
use nsh_core::element::ambient::{KeyOptions, on_key, on_key_with, set_interval};
use nsh_core::element::desktop::set_title_bar;
use nsh_core::keymap::When;
use nsh_base::button;
use nsh_core::prelude::*;

use crate::components::hub::hub;
use crate::components::palette::palette;
use crate::components::primitives::shadow_2xl;
use crate::components::sidebar::sidebar;
use crate::pages::agent::agent_view;
use crate::pages::diagnostics::diagnostics;
use crate::pages::project::project_view;
use crate::pages::welcome::welcome;
use crate::pages::workspace::workspace_view;
use crate::icons::icon;
use crate::store::{Store, Win, store};
use crate::theme::pal;
use crate::tokens::*;
use crate::types::Sel;

/// The app's UI set (`nsh.json` `"rust"`): what a TSX entry may import from
/// `papago/native` (README.md, "Adding a TSX entry").
#[native_module("papago/native")]
fn native(registry: &mut Registry) {
    registry.register::<App>().assets(assets!());
}

/// The application's root component.
#[component]
pub fn App() -> impl IntoElement {
    let s = Store::new();
    provide_context(s.clone());

    // main.tsx's module drivers live outside React (useEffect is banned,
    // AGENTS.md): the 1s streaming heartbeat, the 100ms mock clock, and the
    // global palette shortcuts — all owned by the root here.
    let tick_store = s.clone();
    set_interval(Duration::from_secs(1), move || tick_store.tick());
    let clock_store = s.clone();
    set_interval(Duration::from_millis(100), move || clock_store.tick_clock());

    let palette_store = s.clone();
    on_key("mod+k", move |_| {
        let open = !palette_store.palette_open.get();
        palette_store.set_palette_open(open);
    });
    let esc_store = s.clone();
    on_key("escape", move |_| {
        if esc_store.palette_open.get() {
            esc_store.set_palette_open(false);
        }
    });
    // DEC-014: Alt+Enter queues instead of sending. The prototype binds it
    // on the composer's textarea only; an NSH keymap binding is global, so
    // it queues the selected agent's draft wherever the focus is, with the
    // same empty-draft guard.
    let alt_store = s.clone();
    on_key_with(
        "alt+enter",
        KeyOptions { when: When::Always, ..Default::default() },
        move |_| {
            if let Some(Sel::Agent { id }) = alt_store.sel.get() {
                let key = format!("agent:{id}");
                let t = alt_store.draft(&key).trim().to_string();
                if !t.is_empty() {
                    alt_store.send_to_agent(&id, &t, true);
                    alt_store.set_draft(&key, String::new());
                }
            }
        },
    );

    // The prototype's frameless window: hide the system title bar; the pane
    // headers draw the window controls (mygo: Frameless + Vibrancy — GTK
    // keeps only its window buttons, which the header reserves space for).
    set_title_bar(TitleBarStyle::Hidden, None);

    view()
        .flex_1()
        .relative()
        // The body behind the glass window (the wallpaper's dark base,
        // opaque: NSH GTK windows draw over the desktop without vibrancy).
        .bg(rgb(BODY))
        .text_color(move || pal().ink)
        .child(dynamic(move || match store().win.get() {
            Win::Closed => closed_view(),
            Win::Minimized => minimized_view(),
            win => window_frame(win),
        }))
        .child(hub())
        .child(palette())
}

/// `win === 'closed'`: a centered reopen button.
fn closed_view() -> AnyElement {
    view()
        .flex_1()
        .flex_row()
        .items_center()
        .justify_center()
        .child(
            button()
                .px(px(SP4))
                .h(px(H9))
                .items_center()
                .justify_center()
                .rounded(px(RADIUS_LG))
                .bg(move || pal().glass3)
                .border(px(HAIRLINE))
                .border_color(white_a(ALPHA_BORDER))
                .shadow(shadow_2xl())
                .text_size(px(TEXT_SM))
                .text_color(move || pal().ink)
                .hover(|s| s.border_color(move || pal().acc))
                .accessibility_label("reopen papago")
                .on_press(|| store().set_win(Win::Normal))
                .child(text("reopen papago")),
        )
        .into_element()
}

/// `win === 'minimized'`: the restore button at the bottom-left.
fn minimized_view() -> AnyElement {
    view()
        .flex_1()
        .flex_col()
        .items_start()
        .justify_end()
        .p(px(SP6))
        .child(
            button()
                .px(px(SP4))
                .h(px(H9))
                .flex_row()
                .items_center()
                .justify_center()
                .gap(px(SP2))
                .rounded(px(RADIUS_LG))
                .bg(move || pal().glass3)
                .border(px(HAIRLINE))
                .border_color(white_a(ALPHA_BORDER))
                .shadow(shadow_2xl())
                .text_size(px(TEXT_SM))
                .text_color(move || pal().ink)
                .hover(|s| s.border_color(move || pal().acc))
                .accessibility_label("restore")
                .on_press(|| store().set_win(Win::Normal))
                .child(icon(crate::icons::WIN_RESTORE, TEXT_SM))
                .child(text("papago — minimized")),
        )
        .into_element()
}

/// The window: `h-full p-3` (p-0 maximized) around the rounded glass frame.
fn window_frame(win: Win) -> AnyElement {
    let maximized = win == Win::Maximized;
    let mut wrapper = view().flex_1().flex_row();
    wrapper = if maximized { wrapper.p(px(0.0)) } else { wrapper.p(px(SP3)) };
    wrapper
        .child(
            view()
                .flex_1()
                .min_h(px(0.0))
                .flex_row()
                .overflow_hidden()
                .rounded(if maximized { px(0.0) } else { px(SP3) })
                .border(px(HAIRLINE))
                .border_color(white_a(ALPHA_BORDER))
                .shadow(shadow_2xl())
                .bg(move || pal().glass0)
                .backdrop_blur(px(BLUR_3XL))
                .child(sidebar())
                .child(
                    view()
                        .flex_1()
                        .min_w(px(0.0))
                        .flex_col()
                        .child(routed()),
                ),
        )
        .into_element()
}

/// `{ompMissing ? <Diagnostics/> : !hasProjects ? <Welcome/> : <Routed/>}`.
/// Reads only the routing inputs (sel, ompMissing, the has-projects memo),
/// so the pages mount once per route and never rebuild on fleet ticks.
fn routed() -> AnyElement {
    dynamic(move || {
        let s = store();
        if s.omp_missing.get() {
            return diagnostics();
        }
        if !s.has_projects.get() {
            return welcome();
        }
        match s.sel.get() {
            None => welcome(),
            Some(Sel::Project { id }) => project_view(&id),
            Some(Sel::Workspace { id }) => workspace_view(&id),
            Some(Sel::Agent { id }) => agent_view(&id),
        }
    })
    .into_element()
}
