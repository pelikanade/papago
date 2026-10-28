//! `ProjectView`, ported from `prototype/src/pages/ProjectView.tsx`: the
//! orchestrator row (read/plan/dispatch) + its workspaces. Sending from the
//! composer below goes to the project orchestrator, which creates a
//! workspace + worktree + worker for the prompt.

use nsh_base::button;
use nsh_core::prelude::*;

use crate::components::composer::{composer_bar, Variant};
use crate::components::header::header;
use crate::components::primitives::dot;
use crate::icons::{self as ic, NF};
use crate::store::{find_project, store, Sel};
use crate::theme::pal;
use crate::tokens::*;
use crate::types::{Isolation, Project};

/// One card row for every entry in the list (orchestrator, then
/// workspaces): `w-full rounded-md border border-line px-3 py-2 flex
/// items-center gap-2 hover:border-mut/60`.
fn card_row(title: &str, on_press: Box<dyn Fn() + 'static>, children: Vec<AnyElement>) -> AnyElement {
    button()
        .w_full()
        .rounded(px(RADIUS_MD))
        .border(px(HAIRLINE))
        .border_color(move || pal().line)
        .hover(|s| s.border_color(move || pal().muted.opacity(0.6)))
        .px(px(SP3))
        .py(px(SP2))
        .flex_row()
        .items_center()
        .gap(px(SP2))
        .accessibility_label(title.to_owned())
        .on_press(on_press)
        .child(children)
        .into_element()
}

fn project_rows(p: &Project) -> Vec<AnyElement> {
    let mut rows = Vec::new();
    let oid = p.orchestrator.id.clone();
    let press_oid = oid.clone();
    let o_children: Vec<AnyElement> = vec![
        dot(p.orchestrator.status, true).into_element(),
        text(ic::SITEMAP)
            .font_family(NF.family)
            .text_size(px(TEXT_XS))
            .text_color(move || pal().muted.opacity(0.6))
            .into_element(),
        text("orchestrator")
            .text_size(px(TEXT_SM))
            .font_weight(FontWeight::MEDIUM)
            .text_color(move || pal().ink)
            .into_element(),
        text("read · plan · dispatch — workers own writes")
            .text_size(px(TEXT_XS))
            .text_color(move || pal().muted)
            .into_element(),
        view()
            .ml(Length::Auto)
            .child(
                text(p.orchestrator.model.clone())
                    .text_size(px(TEXT_XS))
                    .text_color(move || pal().muted.opacity(0.6)),
            )
            .into_element(),
    ];
    rows.push(card_row("orchestrator", Box::new(move || store().select(Sel::Agent { id: press_oid.clone() })), o_children));

    for w in &p.workspaces {
        if w.worker.archived {
            continue;
        }
        let wid = w.id.clone();
        let press_wid = wid.clone();
        let isolation = match w.isolation {
            Isolation::Worktree => format!("worktree · {}", w.branch),
            Isolation::Local => "local checkout".to_string(),
        };
        let children: Vec<AnyElement> = vec![
            dot(w.worker.status, false).into_element(),
            text(w.name.clone())
                .text_size(px(TEXT_SM))
                .font_weight(FontWeight::MEDIUM)
                .text_color(move || pal().ink)
                .into_element(),
            text(isolation)
                .text_size(px(TEXT_XS))
                .text_color(move || pal().muted)
                .into_element(),
            view()
                .ml(Length::Auto)
                .child(
                    text(w.worker.title.clone())
                        .text_size(px(TEXT_XS))
                        .text_color(move || pal().muted.opacity(0.6))
                        .number_of_lines(1),
                )
                .into_element(),
        ];
        rows.push(card_row(&w.name, Box::new(move || store().select(Sel::Workspace { id: press_wid.clone() })), children));
    }
    rows
}

pub fn project_view(id: &str) -> AnyElement {
    let pid = id.to_string();
    let pane_pid = pid.clone();
    view()
        .flex_1()
        .min_w(px(0.0))
        .flex_col()
        .child(
            // The header (its sub counts the live workspaces) and the card
            // rows follow the fleet; the composer below stays mounted.
            dynamic(move || {
                let fleet = store().fleet.get();
                match find_project(&fleet, &pane_pid) {
                    None => vec![],
                    Some(p) => {
                        let count = p.workspaces.iter().filter(|w| !w.worker.archived).count();
                        vec![
                            header(
                                &p.name,
                                Some(&format!("{} · {} · {} workspaces", p.path, p.branch, count)),
                                None,
                            ),
                            view()
                                .flex_1()
                                .min_h(px(0.0))
                                .child(
                                    scroll()
                                        .flex_1()
                                        .px(px(SP4))
                                        .py(px(SP3))
                                        .gap(px(SP2))
                                        .child(project_rows(p)),
                                )
                                .into_element(),
                        ]
                    }
                }
            }),
        )
        .child(composer_bar(Variant::Project, &pid, None, None))
        .into_element()
}
