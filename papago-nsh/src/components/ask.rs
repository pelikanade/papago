//! `AskForm`, ported from `prototype/src/components/Ask.tsx`: the
//! interactive Ask form, rendered in the composer slot (it replaces the
//! text box while a pending Ask exists). Inline timeline entries stay
//! read-only references.

use nsh_base::{button, input};
use nsh_core::prelude::*;

use crate::store::store;
use crate::theme::pal;
use crate::tokens::*;
use crate::types::TimelineItem;

pub fn ask_form(agent_id: &str, index: usize, it: &TimelineItem) -> AnyElement {
    let (kind, q, options) = match it {
        TimelineItem::Ask { kind, q, options, .. } => (*kind, q.clone(), options.clone()),
        _ => return view().into_element(),
    };
    let draft_key = format!("ask:{agent_id}:{index}");

    let mut box_ = view()
        .rounded(px(RADIUS_LG))
        .border(px(HAIRLINE))
        .border_color(move || pal().warn.opacity(0.5))
        .bg(move || pal().warn.opacity(0.05))
        .backdrop_blur(px(BLUR_MD))
        .p(px(SP15 / 2.0 + SP1)) // p-2.5
        .gap(px(SP2));

    // badge + hint
    box_ = box_.child(
        view()
            .flex_row()
            .items_center()
            .gap(px(SP2))
            .child(
                text(format!("Ask · {}", kind.label()))
                    .text_size(px(TEXT_XS))
                    .px(px(SP15 / 2.0 + 0.5)) // px-1.5
                    .rounded(px(RADIUS_SM))
                    .bg(move || pal().warn.opacity(0.2))
                    .text_color(move || pal().warn),
            )
            .child(
                text("blocks this step until answered · never expires")
                    .text_size(px(TEXT_XS))
                    .text_color(move || pal().muted),
            ),
    );
    box_ = box_.child(text(q).text_size(px(TEXT_SM)).text_color(move || pal().ink));

    match kind {
        crate::types::AskKind::Select => {
            box_ = box_.child(
                view()
                    .flex_row()
                    .flex_wrap()
                    .gap(px(SP15))
                    .child(
                        options
                            .unwrap_or_default()
                            .iter()
                            .enumerate()
                            .map(|(i, o)| {
                                let o = o.clone();
                                let o_label = o.clone();
                                let aid = agent_id.to_string();
                                button()
                                    .px(px(SP2))
                                    .h(px(H7))
                                    .items_center()
                                    .rounded(px(RADIUS_SM))
                                    .border(px(HAIRLINE))
                                    .border_color(move || pal().line)
                                    .text_size(px(TEXT_SM))
                                    .text_color(move || pal().ink)
                                    .hover(|s| {
                                        s.border_color(move || pal().acc).text_color(move || pal().acc)
                                    })
                                    .on_press(move || store().answer_ask(&aid, index, &o))
                                    .child(
                                        text("")
                                            .child(
                                                text(format!("{}", i + 1))
                                                    .text_color(move || pal().muted.opacity(0.6))
                                                    .mr(px(SP1))
                                                    .into_element(),
                                            )
                                            .child(o_label)
                                            .into_element(),
                                    )
                                    .into_element()
                            })
                            .collect::<Vec<AnyElement>>(),
                    ),
            );
        }
        crate::types::AskKind::Confirm => {
            let approve_id = agent_id.to_string();
            let cancel_id = agent_id.to_string();
            box_ = box_.child(
                view()
                    .flex_row()
                    .gap(px(SP15))
                    .child(
                        button()
                            .px(px(SP2 + SP1)) // px-2.5
                            .h(px(H7))
                            .items_center()
                            .justify_center()
                            .rounded(px(RADIUS_SM))
                            .bg(move || pal().acc)
                            .text_color(move || pal().bg)
                            .text_size(px(TEXT_SM))
                            .on_press(move || store().answer_ask(&approve_id, index, "Approve"))
                            .child(text("Approve")),
                    )
                    .child(
                        button()
                            .px(px(SP2 + SP1))
                            .h(px(H7))
                            .items_center()
                            .justify_center()
                            .rounded(px(RADIUS_SM))
                            .border(px(HAIRLINE))
                            .border_color(move || pal().line)
                            .text_size(px(TEXT_SM))
                            .text_color(move || pal().muted)
                            .hover(|s| {
                                s.text_color(move || pal().err).border_color(move || pal().err)
                            })
                            .on_press(move || store().answer_ask(&cancel_id, index, "Cancel"))
                            .child(text("Cancel")),
                    ),
            );
        }
        crate::types::AskKind::Input => {
            let submit_id = agent_id.to_string();
            let submit_key = draft_key.clone();
            let change_key = draft_key.clone();
            box_ = box_.child(
                view()
                    .flex_row()
                    .gap(px(SP15))
                    .child(
                        input()
                            .value({
                                let change_key = change_key.clone();
                                move || store().draft(&change_key)
                            })
                            .on_change({
                                let change_key = change_key.clone();
                                move |t: &str| store().set_draft(&change_key, t.to_string())
                            })
                            .on_submit({
                                let submit_key = draft_key.clone();
                                let submit_id = agent_id.to_string();
                                move |_: &str| {
                                    let v = store().draft(&submit_key).trim().to_string();
                                    if !v.is_empty() {
                                        store().answer_ask(&submit_id, index, &v);
                                    }
                                }
                            })
                            .flex_1()
                            .h(px(H7))
                            .rounded(px(RADIUS_SM))
                            .border(px(HAIRLINE))
                            .border_color(move || pal().line)
                            .bg(clear())
                            .px(px(SP2))
                            .text_size(px(TEXT_SM))
                            .focus(|s| s.border_color(move || pal().acc)),
                    )
                    .child(
                        button()
                            .px(px(SP2 + SP1))
                            .h(px(H7))
                            .items_center()
                            .justify_center()
                            .rounded(px(RADIUS_SM))
                            .bg(move || pal().acc)
                            .text_color(move || pal().bg)
                            .text_size(px(TEXT_SM))
                            .on_press(move || {
                                let v = store().draft(&submit_key).trim().to_string();
                                if !v.is_empty() {
                                    store().answer_ask(&submit_id, index, &v);
                                }
                            })
                            .child(text("Submit")),
                    ),
            );
        }
    }
    box_.into_element()
}
