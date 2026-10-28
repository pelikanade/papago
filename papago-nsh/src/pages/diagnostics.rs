//! `Diagnostics`, ported from `prototype/src/pages/Diagnostics.tsx`.

use nsh_base::button;
use nsh_core::prelude::*;

use crate::store::store;
use crate::theme::pal;
use crate::tokens::*;

pub fn diagnostics() -> AnyElement {
    view()
        .flex_1()
        .flex_col()
        .items_center()
        .justify_center()
        .gap(px(SP3))
        .child(
            text("omp not found on PATH")
                .text_size(px(TEXT_SM))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(move || pal().warn),
        )
        .child(
            view()
                .text_size(px(TEXT_XS))
                .font_family("monospace")
                .text_color(move || pal().muted)
                .bg(move || pal().panel)
                .border(px(HAIRLINE))
                .border_color(move || pal().line)
                .rounded(px(RADIUS_MD))
                .p(px(SP3))
                .child(
                    text("curl -fsSL https://omp.sh/install | sh\n# or: bun install -g @oh-my-pi/pi-coding-agent"),
                ),
        )
        .child(
            text("or set an absolute path in ~/.papago/config.json → omp.path")
                .text_size(px(TEXT_XS))
                .text_color(move || pal().muted),
        )
        .child(
            button()
                .px(px(SP3))
                .h(px(H7))
                .items_center()
                .justify_center()
                .rounded(px(RADIUS_SM))
                .border(px(HAIRLINE))
                .border_color(move || pal().line)
                .text_size(px(TEXT_SM))
                .text_color(move || pal().muted)
                .hover(|s| s.text_color(move || pal().ink))
                .accessibility_label("Retry detection")
                .on_press(|| store().set_omp_missing(false))
                .child(text("Retry detection")),
        )
        .into_element()
}
