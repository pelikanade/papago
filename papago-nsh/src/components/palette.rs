//! `Palette`, ported from `prototype/src/components/Palette.tsx`: the
//! command palette (Ctrl+K): jumps + actions. The only shortcut surface.
//! Pure view: query + open state live in the store. `fixed top-16
//! left-1/2 -translate-x-1/2` becomes a full-width absolute row that
//! centers its w-160 panel, against the app root like the CSS `fixed`.

use nsh_base::{button, input};
use nsh_core::prelude::*;

use crate::components::primitives::shadow_2xl;
use crate::icons::{self as ic, icon};
use crate::store::{collect_asks, store, Sel};
use crate::theme::{pal, theme_scheme};
use crate::tokens::*;

/// top-16.
const TOP_16: f32 = 64.0;

pub fn palette() -> AnyElement {
    dynamic(move || {
        if store().palette_open.get() {
            vec![dim_catcher(), panel()]
        } else {
            vec![]
        }
    })
    .into_element()
}

/// The dim catcher: `fixed inset-0 z-40 bg-black/40`, closing on a press.
fn dim_catcher() -> AnyElement {
    view()
        .absolute()
        .inset(px(0.0))
        .bg(rgb(0).opacity(0.40))
        .z_index(40)
        .label("close palette")
        .on_press(|| store().set_palette_open(false))
        .into_element()
}

fn panel() -> AnyElement {
    view()
        .absolute()
        .top(px(TOP_16))
        .left(px(0.0))
        .right(px(0.0))
        .flex_row()
        .justify_center()
        .z_index(50)
        .child(
            view()
                .w(px(rem(40.0))) // w-160
                .bg(move || pal().glass3)
                .backdrop_blur(px(BLUR_2XL))
                .border(px(HAIRLINE))
                .border_color(white_a(ALPHA_BORDER))
                .rounded(px(RADIUS_LG))
                .shadow(shadow_2xl())
                .overflow_hidden()
                .flex_col()
                .child(
                    input()
                        .value(move || store().palette_query.get())
                        .on_change(move |t: &str| store().set_palette_query(t.to_string()))
                        .placeholder("jump to Agent / Workspace · run action…")
                        .auto_focus(true)
                        .w_full()
                        .h(px(H9))
                        .px(px(SP3))
                        .bg(clear())
                        .text_size(px(TEXT_SM))
                        .text_color(move || pal().ink)
                        .border_b(px(HAIRLINE))
                        .border_color(white_a(ALPHA_BORDER)),
                )
                .child(
                    view().max_h(px(rem(26.25))).child(
                        scroll()
                            .flex_1()
                            .py(px(SP1))
                            // React keyed the rows by their id: a row keeps
                            // its nodes (and its press target) across the
                            // fleet ticks that rebuild the list.
                            .child(each(
                                filtered_rows,
                                |r: &PalRow| r.id.clone(),
                                |row, _| pal_row_element(row),
                            ))
                            .child(dynamic(move || {
                                if filtered_rows().is_empty() {
                                    vec![no_match()]
                                } else {
                                    vec![]
                                }
                            })),
                    ),
                )
                .into_element(),
        )
        .into_element()
}

#[derive(Clone)]
struct PalRow {
    id: String,
    label: String,
    hint: Option<String>,
    icon: Option<&'static str>,
    run: std::sync::Arc<dyn Fn() + 'static>,
}

/// Rows are equal when their ids are: `each` reconciles by key, and the
/// label may change under the same key (the theme row's, as it cycles).
impl PartialEq for PalRow {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

/// The palette's rows, filtered by the query.
fn filtered_rows() -> Vec<PalRow> {
    let q = store().palette_query.get().to_lowercase();
    palette_rows().into_iter().filter(|r| r.label.to_lowercase().contains(&q)).collect()
}

/// One palette row, built from its row signal (the label follows the row's
/// value: the theme row's label changes as the theme cycles).
fn pal_row_element(row: ReadSignal<PalRow>) -> AnyElement {
    let mut children: Vec<AnyElement> = Vec::new();
    if let Some(g) = row.get().icon {
        children.push(icon(g, TEXT_SM).text_color(move || pal().muted.opacity(0.6)).into_element());
    }
    children.push(
        text(move || row.get().label)
            .text_size(px(TEXT_SM))
            .number_of_lines(1)
            .text_color(move || pal().ink)
            .into_element(),
    );
    if let Some(h) = &row.get().hint {
        children.push(
            view()
                .ml(Length::Auto)
                .child(
                    text(h.clone())
                        .text_size(px(TEXT_XS))
                        .text_color(move || pal().muted.opacity(0.6))
                        .into_element(),
                )
                .into_element(),
        );
    }
    let run = row.get().run.clone();
    let id = row.get().id;
    button()
        .w_full()
        .flex_row()
        .items_center()
        .gap(px(SP2))
        .px(px(SP3))
        .py(px(SP15))
        .accessibility_label(id)
        .hover(|s| s.bg(white_a(ALPHA_HOVER_ALT)))
        .on_press(move || {
            run();
            store().set_palette_open(false);
        })
        .child(children)
        .into_element()
}

fn no_match() -> AnyElement {
    view()
        .px(px(SP3))
        .py(px(SP2))
        .child(
            text("no match")
                .text_size(px(TEXT_SM))
                .text_color(move || pal().muted.opacity(0.6)),
        )
        .into_element()
}

/// The palette's rows, from the current state (jumps first, then actions).
fn palette_rows() -> Vec<PalRow> {
    let s = store();
    let fleet = s.fleet.get();
    let mut rows: Vec<PalRow> = Vec::new();

    for a in collect_asks(&fleet) {
        let aid = a.agent_id.clone();
        rows.push(PalRow {
            id: format!("ask:{}:{}", a.agent_id, a.index),
            label: format!("Next Ask — {}", a.q),
            hint: Some("jump".into()),
            icon: None,
            run: std::sync::Arc::new(move || store().select(Sel::Agent { id: aid.clone() })),
        });
    }
    for p in &fleet.projects {
        let pid = p.id.clone();
        rows.push(PalRow {
            id: format!("p:{}", p.id),
            label: p.name.clone(),
            hint: Some("project".into()),
            icon: Some(ic::ARROW_RIGHT),
            run: std::sync::Arc::new(move || store().select(Sel::Project { id: pid.clone() })),
        });
        let oid = p.orchestrator.id.clone();
        rows.push(PalRow {
            id: format!("o:{}", p.orchestrator.id),
            label: format!("{} / orchestrator", p.name),
            hint: Some(p.orchestrator.status.label().into()),
            icon: Some(ic::SITEMAP),
            run: std::sync::Arc::new(move || store().select(Sel::Agent { id: oid.clone() })),
        });
        for w in &p.workspaces {
            let wid = w.id.clone();
            rows.push(PalRow {
                id: format!("w:{}", w.id),
                label: format!("{} / {}", p.name, w.name),
                hint: Some(w.isolation.label().into()),
                icon: Some(ic::ARROW_RIGHT),
                run: std::sync::Arc::new(move || store().select(Sel::Workspace { id: wid.clone() })),
            });
            let aid = w.worker.id.clone();
            rows.push(PalRow {
                id: format!("a:{}", w.worker.id),
                label: w.worker.title.clone(),
                hint: Some(w.worker.status.label().into()),
                icon: Some(ic::ARROW_RIGHT),
                run: std::sync::Arc::new(move || store().select(Sel::Agent { id: aid.clone() })),
            });
        }
        let prune_id = p.id.clone();
        rows.push(PalRow {
            id: format!("prune:{}", p.id),
            label: format!("Prune worktrees — {}", p.name),
            hint: Some("destructive".into()),
            icon: None,
            run: std::sync::Arc::new(move || store().prune_worktrees(&prune_id)),
        });
    }
    rows.push(PalRow {
        id: "hub".into(),
        label: "Open Agent Hub".into(),
        hint: None,
        icon: None,
        run: std::sync::Arc::new(|| store().set_hub_open(true)),
    });
    rows.push(PalRow {
        id: "theme".into(),
        label: format!("Theme: {} → next", theme_scheme(s.theme_id.get())),
        hint: Some("base16".into()),
        icon: None,
        run: std::sync::Arc::new(|| store().cycle_theme()),
    });
    rows.push(PalRow {
        id: "open".into(),
        label: "Open directory…".into(),
        hint: None,
        icon: None,
        run: std::sync::Arc::new(|| store().open_directory("newrepo")),
    });
    let missing = s.omp_missing.get();
    rows.push(PalRow {
        id: "diag".into(),
        label: "Toggle omp diagnostics view".into(),
        hint: Some("debug".into()),
        icon: None,
        run: std::sync::Arc::new(move || store().set_omp_missing(!missing)),
    });
    rows
}
