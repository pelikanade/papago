//! `WorkspaceGraph`, ported from
//! `prototype/src/components/WorkspaceGraph.tsx`: nodes = the workspace's
//! agents (1:1 today, its worker). Subagents are ephemeral and never become
//! nodes — the status bar shows them. Clicking a node opens the Agent view.

use nsh_core::prelude::*;

use crate::components::primitives::dot;
use crate::store::{store, Sel};
use crate::theme::pal;
use crate::tokens::*;
use crate::types::Agent;

const COL: f32 = 260.0;
const NODE_W: f32 = 212.0;
const NODE_H: f32 = 64.0;

pub fn workspace_graph(agents: &[Agent], selected_id: Option<&str>) -> AnyElement {
    if agents.is_empty() {
        return view()
            .py(px(SP6))
            .items_center()
            .child(
                text("no agents here yet — send a prompt below")
                    .text_size(px(TEXT_SM))
                    .text_color(move || pal().muted.opacity(0.6)),
            )
            .into_element();
    }
    let width = agents
        .iter()
        .enumerate()
        .map(|(i, _)| i as f32 * COL + NODE_W + SP15)
        .fold(f32::MIN, f32::max);
    let height = NODE_H + SP6;

    view()
        .relative()
        .w(px(width))
        .h(px(height))
        .child(
            agents
                .iter()
                .enumerate()
                .map(|(i, agent)| {
                    let aid = agent.id.clone();
                    let selected = selected_id == Some(agent.id.as_str());
                    view()
                        .absolute()
                        .left(px(i as f32 * COL + SP15))
                        .top(px(SP15))
                        .w(px(NODE_W))
                        .h(px(NODE_H))
                        .rounded(px(RADIUS_LG))
                        .border(px(HAIRLINE))
                        .bg(move || pal().glass1)
                        .backdrop_blur(px(BLUR_MD))
                        .px(px(SP3))
                        .py(px(SP2))
                        .label(aid.clone())
                        .border_color(move || {
                            if selected {
                                pal().acc
                            } else {
                                white_a(ALPHA_BORDER)
                            }
                        })
                        .hover(|s| s.border_color(move || pal().acc.opacity(0.7)))
                        .on_press(move || store().select(Sel::Agent { id: aid.clone() }))
                        .child(
                            view()
                                .flex_row()
                                .items_center()
                                .gap(px(SP2))
                                .child(dot(agent.status, false))
                                .child(
                                    text(agent.title.clone())
                                        .text_size(px(TEXT_SM))
                                        .number_of_lines(1)
                                        .text_color(move || pal().ink),
                                )
                                .child(
                                    view()
                                        .ml(Length::Auto)
                                        .child(
                                            text(agent.status.label())
                                                .text_size(px(TEXT_XS))
                                                .text_color(move || pal().muted.opacity(0.6)),
                                        )
                                        .into_element(),
                                ),
                        )
                        .child(
                            view()
                                .mt(px(SP1))
                                .child(
                                    text(agent.model.clone())
                                        .text_size(px(TEXT_XS))
                                        .number_of_lines(1)
                                        .text_color(move || pal().muted.opacity(0.7)),
                                ),
                        )
                        .into_element()
                })
                .collect::<Vec<AnyElement>>(),
        )
        .into_element()
}
