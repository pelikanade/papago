//! `Hub`, ported from `prototype/src/components/Hub.tsx`: the Agent Hub
//! panel window (floating): cross-Agent grouped live subagents. Read-only
//! rows + jump; pure view. `fixed top-10 right-6` becomes absolute
//! positioning against the app root, which the window container never
//! clips — the same escape the CSS `fixed` gives.

use nsh_core::prelude::*;

use crate::components::primitives::{dot, shadow_2xl};
use crate::icons::{self as ic, icon};
use crate::store::{collect_subs, store, Sel};
use crate::theme::pal;
use crate::tokens::*;
use crate::types::TimelineItem;

pub fn hub() -> AnyElement {
    dynamic(move || {
        store().hub_open.get().then(|| {
            view()
                .absolute()
                .top(px(H10))
                .right(px(SP6))
                .w(px(rem(32.5))) // w-130
                .max_h(Length::Definite(DefiniteLength::Fraction(Percentage(0.7)))) // max-h-[70vh]
                .bg(move || pal().glass3)
                .backdrop_blur(px(BLUR_2XL))
                .border(px(HAIRLINE))
                .border_color(white_a(ALPHA_BORDER))
                .rounded(px(RADIUS_LG))
                .shadow(shadow_2xl())
                .flex_col()
                .z_index(30)
                .child(
                    view()
                        .h(px(H10))
                        .px(px(SP3))
                        .flex_row()
                        .items_center()
                        .gap(px(SP2))
                        .border_b(px(HAIRLINE))
                        .border_color(white_a(ALPHA_BORDER))
                        .child(
                            text("Agent Hub")
                                .text_size(px(TEXT_SM))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(move || pal().ink),
                        )
                        .child(
                            text(move || {
                                let live = collect_subs(&store().fleet.get())
                                    .iter()
                                    .filter(|x| matches!(&x.sub, TimelineItem::Sub { status: AgentStatusE::Running, .. }))
                                    .count();
                                format!("{live} live")
                            })
                            .text_size(px(TEXT_XS))
                            .text_color(move || pal().muted.opacity(0.6)),
                        )
                        .child(
                            view()
                                .ml(Length::Auto)
                                .text_size(px(TEXT_SM))
                                .text_color(move || pal().muted)
                                .hover(|s| s.text_color(move || pal().ink))
                                .label("close hub")
                                .on_press(|| store().set_hub_open(false))
                                .child(icon(ic::X, TEXT_SM)),
                        ),
                )
                .child(
                    view().flex_1().min_h(px(0.0)).child(
                        scroll()
                            .flex_1()
                            .p(px(SP2))
                            .gap(px(SP2))
                            .child(dynamic(move || {
                                let subs = collect_subs(&store().fleet.get());
                                if subs.is_empty() {
                                    return vec![view()
                                        .p(px(SP3))
                                        .child(
                                            text("no live subagents")
                                                .text_size(px(TEXT_SM))
                                                .text_color(move || pal().muted.opacity(0.6)),
                                        )
                                        .into_element()];
                                }
                                // Groups keyed by agent title, insertion
                                // order (JS object key order).
                                let mut groups: Vec<(String, Vec<crate::store::SubRef>)> = Vec::new();
                                for x in subs {
                                    match groups.iter_mut().find(|(t, _)| *t == x.agent_title) {
                                        Some((_, rows)) => rows.push(x),
                                        None => groups.push((x.agent_title.clone(), vec![x])),
                                    }
                                }
                                groups
                                    .into_iter()
                                    .flat_map(|(title, rows)| {
                                        vec![
                                            view()
                                                .px(px(SP1))
                                                .py(px(0.5))
                                                .child(
                                                    text(title)
                                                        .text_size(px(TEXT_XS))
                                                        .text_color(move || pal().muted.opacity(0.7)),
                                                )
                                                .into_element(),
                                            view().gap(px(SP1)).child(
                                                rows.iter().map(hub_card).collect::<Vec<AnyElement>>()
                                            ).into_element(),
                                        ]
                                    })
                                    .collect()
                            })),
                    ),
                )
                .into_element()
        })
    })
    .into_element()
}

fn hub_card(x: &crate::store::SubRef) -> AnyElement {
    let (title, status, model, tool, tokens) = match &x.sub {
        TimelineItem::Sub { title, status, model, tool, tokens } => (title, *status, model, tool, tokens),
        _ => return view().into_element(),
    };
    let aid = x.agent_id.clone();
    view()
        .rounded(px(RADIUS_MD))
        .border(px(HAIRLINE))
        .border_color(white_a(ALPHA_BORDER))
        .bg(move || pal().glass1)
        .px(px(SP2))
        .py(px(SP15))
        .gap(px(SP1))
        .child(
            view()
                .flex_row()
                .items_center()
                .gap(px(SP2))
                .child(dot(status, false))
                .child(
                    text(title.clone())
                        .text_size(px(TEXT_SM))
                        .number_of_lines(1)
                        .text_color(move || pal().ink),
                )
                .child(
                    view()
                        .child(
                            text(format!("{model} · {tokens}"))
                                .text_size(px(TEXT_XS))
                                .number_of_lines(1)
                                .text_color(move || pal().muted.opacity(0.7)),
                        )
                        .into_element(),
                )
                .child(
                    view()
                        .ml(Length::Auto)
                        .text_size(px(TEXT_XS))
                        .text_color(move || pal().muted.opacity(0.6))
                        .hover(|s| s.text_color(move || pal().ink))
                        .label("open agent")
                        .on_press(move || store().select(Sel::Agent { id: aid.clone() }))
                        .child(icon(ic::ARROW_RIGHT, TEXT_XS)),
                ),
        )
        .child(
            view()
                .pl(px(SP4))
                .child(
                    text(tool.clone())
                        .text_size(px(TEXT_XS))
                        .number_of_lines(1)
                        .text_color(move || pal().muted.opacity(0.7)),
                ),
        )
        .into_element()
}

use crate::types::AgentStatus as AgentStatusE;
