//! `Welcome`, ported from `prototype/src/pages/Welcome.tsx`.

use nsh_base::button;
use nsh_core::prelude::*;

use crate::store::store;
use crate::theme::pal;
use crate::tokens::*;

pub fn welcome() -> AnyElement {
    view()
        .flex_1()
        .flex_col()
        .items_center()
        .justify_center()
        .gap(px(SP3))
        .child(
            text("papago")
                .text_size(px(TEXT_BASE))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(move || pal().ink),
        )
        .child(
            text("run and review your omp agents — no editor, no terminal, no ceremony")
                .text_size(px(TEXT_SM))
                .text_color(move || pal().muted),
        )
        .child(
            button()
                .mt(px(SP2))
                .px(px(SP3))
                .h(px(H7))
                .items_center()
                .justify_center()
                .rounded(px(RADIUS_SM))
                .bg(move || pal().acc)
                .text_color(move || pal().bg)
                .text_size(px(TEXT_SM))
                .accessibility_label("Open directory…")
                .on_press(|| store().open_directory("myrepo"))
                .child(text("Open directory…")),
        )
        .child(
            text("git root becomes a Project; a local Workspace appears with it")
                .text_size(px(TEXT_XS))
                .text_color(move || pal().muted.opacity(0.6)),
        )
        .into_element()
}
