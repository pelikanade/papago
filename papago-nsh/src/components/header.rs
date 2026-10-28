//! `Header` — the shared pane header: breadcrumbs · title · window
//! controls (minimize/maximize/close), ported from
//! `prototype/src/components/Header.tsx`.

use nsh_core::prelude::*;
use nsh_core::element::desktop::title_bar;

use crate::components::primitives::icon_button;
use crate::icons as ic;
use crate::store::{store, Win};
use crate::theme::pal;
use crate::tokens::*;

pub fn header(title: &str, sub: Option<&str>, crumbs: Option<Vec<AnyElement>>) -> AnyElement {
    let s = store();
    let mut h = view()
        .h(px(H8))
        .flex_shrink_0()
        .px(px(SP4))
        .flex_row()
        .items_center()
        .gap(px(SP3))
        .border_b(px(HAIRLINE))
        .border_color(white_a(ALPHA_BORDER))
        // The system keeps its window buttons when the title bar is hidden;
        // reserve their width so they never sit on the pane's own controls.
        .pr(move || title_bar().right + px(SP4));
    if let Some(crumbs) = crumbs {
        h = h.child(crumbs);
    }
    h = h.child(
        text(title.to_owned())
            .text_size(px(TEXT_XS))
            .font_weight(FontWeight::SEMIBOLD)
            .number_of_lines(1)
            .flex_shrink(1.0),
    );
    if let Some(sub) = sub {
        h = h.child(
            text(sub.to_owned())
                .text_size(px(TEXT_XS))
                .text_color(move || pal().muted)
                .number_of_lines(1)
                .flex_shrink(1.0),
        );
    }
    let min = s.clone();
    let close = s.clone();
    h.child(
        view()
            .ml(Length::Auto)
            .flex_row()
            .items_center()
            .gap(px(SP1))
            .child(icon_button(ic::WIN_MIN, "minimize", false, move || min.set_win(Win::Minimized)))
            .child(dynamic(move || {
                let maximized = store().win.get() == Win::Maximized;
                if maximized {
                    icon_button(ic::WIN_RESTORE, "restore", false, || store().set_win(Win::Normal))
                } else {
                    icon_button(ic::WIN_MAX, "maximize", false, || store().set_win(Win::Maximized))
                }
            }))
            .child(icon_button(ic::WIN_CLOSE, "close", true, move || close.set_win(Win::Closed))),
    )
    .into_element()
}
