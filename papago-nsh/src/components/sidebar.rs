//! `Sidebar`, ported from `prototype/src/components/Sidebar.tsx`: the
//! Project row IS its orchestrator — hollow ring dot, project path right;
//! each Workspace row IS its worker (1:1) — filled status dot + workspace
//! name left, PR/CI right. Project blocks are separated by a vertical gap
//! only; workspace rows carry tree connectors aligned with the caret, ending
//! in an elbow on the last row. Subagents are ephemeral: they never appear
//! here, the status bar owns them. All rows share `tree_row`; the footer
//! hosts + Project and settings.

use nsh_base::{button, input};
use nsh_core::prelude::*;

use crate::components::primitives::{dot, icon_button, overlay_panel, tree_connector, tree_row};
use crate::icons::{self as ic, icon};
use crate::store::{store, Sel};
use crate::theme::{pal, theme_scheme};
use crate::tokens::*;
use crate::types::{Agent, CiStatus};

/// Shared "add project / workspace" control (footer + collapsed rail share
/// one creation path and one popover, per the componentization rule).
fn add_project_control(icon_only: bool) -> AnyElement {
    let anchor = element_ref();
    let anchor2 = anchor.clone();
    let toggle = || store().toggle_popover("sel:project");
    let create = move || {
        let s = store();
        let v = s.draft("sel:project").trim().to_string();
        if v.is_empty() {
            return;
        }
        s.open_directory(&v);
        s.set_draft("sel:project", String::new());
        s.toggle_popover("sel:project");
    };
    let trigger: AnyElement = if icon_only {
        icon_button(ic::PLUS, "add project / workspace", false, move || toggle())
            .element_ref(&anchor)
            .into_element()
    } else {
        view()
            .h(px(H7))
            .pl(px(SP15))
            .pr(px(SP2))
            .flex_row()
            .items_center()
            .gap(px(SP15))
            .rounded(px(RADIUS_SM))
            .label("add project / workspace")
            .text_size(px(TEXT_XS))
            .text_color(move || pal().muted)
            .hover(|s| s.bg(white_a(ALPHA_HOVER)).text_color(move || pal().ink))
            .on_press(move || toggle())
            .child(icon(ic::PLUS, TEXT_XS))
            .child(text("Project"))
            .element_ref(&anchor)
            .into_element()
    };
    view()
        .relative()
        .flex_row()
        .child(trigger)
        .child(dynamic(move || {
            store().popover("sel:project").then(|| {
                overlay_panel(
                    &anchor2,
                    false,
                    rem(16.0),
                    || store().toggle_popover("sel:project"),
                    vec![
                        view()
                            .p(px(SP1))
                            .gap(px(SP15))
                            .child(
                                input()
                                    .value(move || store().draft("sel:project"))
                                    .on_change(move |t: &str| {
                                        store().set_draft("sel:project", t.to_string())
                                    })
                                    .on_submit(move |_: &str| create())
                                    .placeholder("directory path or name")
                                    .w_full()
                                    .h(px(H7))
                                    .rounded(px(RADIUS_SM))
                                    .border(px(HAIRLINE))
                                    .border_color(move || pal().line)
                                    .bg(clear())
                                    .px(px(SP2))
                                    .text_size(px(TEXT_XS))
                                    .focus(|s| s.border_color(move || pal().acc))
                                    .into_element(),
                            )
                            .child(
                                button()
                                    .w_full()
                                    .h(px(H7))
                                    .items_center()
                                    .justify_center()
                                    .rounded(px(RADIUS_SM))
                                    .bg(move || pal().acc)
                                    .text_size(px(TEXT_XS))
                                    .text_color(move || pal().bg)
                                    .on_press(move || create())
                                    .child(text("Create Project"))
                                    .into_element(),
                            )
                            .into_element(),
                    ],
                )
            })
        }))
        .into_element()
}

pub fn sidebar() -> AnyElement {
    dynamic(move || {
        if store().sidebar_collapsed.get() {
            rail()
        } else {
            full()
        }
    })
    .into_element()
}

/// Collapsed rail: one status dot per Agent (orchestrators + workers), click
/// to open.
fn rail() -> AnyElement {
    view()
        .relative()
        .z_index(20)
        .w(px(W8))
        .flex_shrink_0()
        .border_r(px(HAIRLINE))
        .border_color(white_a(ALPHA_BORDER))
        .bg(move || pal().glass1)
        .backdrop_blur(px(BLUR_XL))
        .flex_col()
        // top strip mirrors the pane header (h-8 + hairline)
        .child(
            view()
                .h(px(H8))
                .flex_shrink_0()
                .border_b(px(HAIRLINE))
                .border_color(white_a(ALPHA_BORDER))
                .items_center()
                .justify_center()
                .child(icon_button(ic::CARET_RIGHT, "expand sidebar", false, || {
                    store().toggle_sidebar()
                })),
        )
        .child(
            view().flex_1().min_h(px(0.0)).child(
                scroll()
                    .flex_1()
                    .w_full()
                    .flex_col()
                    .items_center()
                    .gap(px(SP15))
                    .py(px(SP15))
                    .child(each(
                        move || rail_rows(),
                        |r: &(Agent, String)| r.0.id.clone(),
                        |row, _| {
                            let label = { let r = row.get(); r.1.clone() };
                            view()
                                .w(px(W6))
                                .h(px(H6))
                                .flex_row()
                                .items_center()
                                .justify_center()
                                .rounded(px(RADIUS_SM))
                                .label(label.clone())
                                .hover(|s| s.bg(white_a(ALPHA_SELECTED)))
                                .bg(move || {
                                    let sel = store().sel.get();
                                    let r = row.get_untracked();
                                    if matches!(&sel, Some(Sel::Agent { id }) if *id == r.0.id) {
                                        white_a(0.10)
                                    } else {
                                        clear()
                                    }
                                })
                                .on_press(move || {
                                    let r = row.get();
                                    store().select(Sel::Agent { id: r.0.id.clone() });
                                })
                                .child(dot(row.get_untracked().0.status, row.get_untracked().0.role == crate::types::AgentRole::Orchestrator))
                                .into_element()
                        },
                    )),
            ),
        )
        // bottom strip mirrors the status bar (h-7 + hairline)
        .child(
            view()
                .h(px(H7))
                .flex_shrink_0()
                .border_t(px(HAIRLINE))
                .border_color(white_a(ALPHA_BORDER))
                .items_center()
                .justify_center()
                .child(add_project_control(true)),
        )
        .into_element()
}

fn rail_rows() -> Vec<(Agent, String)> {
    let fleet = store().fleet.get();
    let mut rows: Vec<(Agent, String)> = Vec::new();
    for p in &fleet.projects {
        rows.push((p.orchestrator.clone(), format!("{} · orchestrator", p.name)));
        for w in &p.workspaces {
            if !w.worker.archived {
                rows.push((w.worker.clone(), format!("{} / {} · {}", p.name, w.name, w.worker.title)));
            }
        }
    }
    rows.sort_by_key(|(a, _)| crate::data::urgency(a.status));
    rows
}

/// The full sidebar: header, the project tree, the footer.
fn full() -> AnyElement {
    let anchor = element_ref();
    let anchor2 = anchor.clone();
    view()
        .relative()
        .z_index(20)
        .w(px(W72))
        .flex_shrink_0()
        .border_r(px(HAIRLINE))
        .border_color(white_a(ALPHA_BORDER))
        .bg(move || pal().glass1)
        .backdrop_blur(px(BLUR_XL))
        .flex_col()
        .child(
            view()
                .h(px(H8))
                .px(px(SP3))
                .flex_row()
                .items_center()
                .gap(px(SP2))
                .border_b(px(HAIRLINE))
                .border_color(white_a(ALPHA_BORDER))
                .child(
                    text("papago")
                        .text_size(px(TEXT_SM))
                        .font_weight(FontWeight::SEMIBOLD)
                        .letter_spacing(px(0.4))
                        .text_color(move || pal().ink),
                )
                .child(
                    text("prototype")
                        .text_size(px(TEXT_XS))
                        .text_color(move || pal().muted.opacity(0.6)),
                )
                .child(
                    view()
                        .label("palette")
                        .text_size(px(TEXT_XS))
                        .text_color(move || pal().muted)
                        .hover(|s| s.text_color(move || pal().ink))
                        .on_press(|| store().set_palette_open(true))
                        .child(text("⌘K")),
                )
                .child(
                    view().ml(Length::Auto).child(icon_button(ic::CARET_LEFT, "collapse sidebar", false, || {
                        store().toggle_sidebar()
                    })),
                ),
        )
        .child(
            view().flex_1().min_h(px(0.0)).child(
                scroll()
                    .flex_1()
                    .py(px(SP15))
                    // React keyed the blocks per project: rows keep their
                    // identity (and their press targets) across the fleet
                    // ticks, and the fold state re-renders the tree.
                    .child(each(
                        move || {
                            let _folds = store().popovers.get();
                            store().fleet.get().projects.clone()
                        },
                        |p: &crate::types::Project| p.id.clone(),
                        |p, i| project_block(p, i.get()),
                    )),
            ),
        )
        .child(
            view()
                .h(px(H7))
                .px(px(SP15))
                .border_t(px(HAIRLINE))
                .border_color(white_a(ALPHA_BORDER))
                .flex_row()
                .items_center()
                .child(add_project_control(false))
                .child(view().flex_1())
                .child(
                    view()
                        .relative()
                        .flex_row()
                        .child(
                            icon_button(ic::GEAR, "settings", false, move || {
                                store().toggle_popover("footer:settings")
                            })
                            .element_ref(&anchor),
                        )
                        .child(dynamic(move || {
                            store().popover("footer:settings").then(|| {
                                overlay_panel(
                                    &anchor2,
                                    true,
                                    rem(15.0),
                                    || store().toggle_popover("footer:settings"),
                                    vec![
                                        button()
                                            .w_full()
                                            .flex_row()
                                            .items_center()
                                            .gap(px(SP2))
                                            .px(px(SP2))
                                            .py(px(SP2))
                                            .rounded(px(RADIUS_SM))
                                            .text_size(px(TEXT_XS))
                                            .text_color(move || pal().ink)
                                            .hover(|s| s.bg(white_a(ALPHA_HOVER_ALT)))
                                            .on_press(|| store().cycle_theme())
                                            .child(text(move || {
                                                format!("Theme: {}", theme_scheme(store().theme_id.get()))
                                            }))
                                            .into_element(),
                                        view()
                                            .px(px(SP2))
                                            .py(px(SP15))
                                            .text_size(px(TEXT_XS))
                                            .text_color(move || pal().muted.opacity(0.7))
                                            .child(text("daemon · local"))
                                            .into_element(),
                                        view()
                                            .px(px(SP2))
                                            .py(px(SP15))
                                            .text_size(px(TEXT_XS))
                                            .text_color(move || pal().muted.opacity(0.7))
                                            .child(text("~/.papago/config.json"))
                                            .into_element(),
                                    ],
                                )
                            })
                        })),
                ),
        )
        .into_element()
}

/// One Project block: the orchestrator row + its workspace rows. Each
/// Project is its own block: a larger vertical gap separates the blocks, so
/// an orchestrator header never reads as a sibling row. Keyed per project
/// (`each`): the block keeps its identity — and its press targets — across
/// fleet ticks, as React's key does.
fn project_block(p: ReadSignal<crate::types::Project>, pi: usize) -> AnyElement {
    let project = p.get();
    let pid = project.id.clone();
    let oid = project.orchestrator.id.clone();
    let press_oid = oid.clone();
    let open_key = format!("tree:{pid}");
    let open_key2 = open_key.clone();
    let ws_signal = p.clone();
    let mut block = view();
    if pi > 0 {
        block = block.mt(px(SP10));
    }
    block
        .child(tree_row(
            0,
            move || matches!(&store().sel.get(), Some(Sel::Agent { id }) if *id == oid),
            &format!("{} · orchestrator", project.name),
            move || store().select(Sel::Agent { id: press_oid.clone() }),
            Some(Box::new(move || store().toggle_popover(&open_key2))),
            move || store().popovers.get().get(&open_key).copied().unwrap_or(true),
            vec![
                dot(project.orchestrator.status, true).into_element(),
                text(project.name.clone())
                    .text_size(px(TEXT_XS))
                    .font_weight(FontWeight::MEDIUM)
                    .number_of_lines(1)
                    .text_color(move || pal().ink)
                    .into_element(),
                view()
                    .ml(Length::Auto)
                    .child(
                        text(project.path.clone())
                            .text_size(px(TEXT_XS))
                            .number_of_lines(1)
                            .text_color(move || pal().muted.opacity(0.5)),
                    )
                    .into_element(),
            ],
        ))
        .child(
            // The workspace rows, keyed per workspace, shown while the
            // fold is open (React: `{pOpen && workspaces.map(…)}`).
            dynamic({
                let fold_key = pid.clone();
                move || {
                let open = store().popovers.get().get(&fold_key).copied().unwrap_or(true);
                open.then(|| {
                    each(
                        move || {
                            let ws = ws_signal.get().workspaces;
                            ws.into_iter().filter(|w| !w.worker.archived).collect::<Vec<_>>()
                        },
                        |w: &crate::types::Workspace| w.id.clone(),
                        |w, _i| workspace_row(w),
                    )
                })
                }
            }),
        )
        .into_element()
}

/// One workspace row (double as its worker agent row, 1:1): status dot +
/// workspace name on the left, PR/CI on the right, plus the tree connector
/// that wraps the group (elbow on the last row). Keyed per workspace.
fn workspace_row(w: ReadSignal<crate::types::Workspace>) -> AnyElement {
    let ws = w.get();
    let aid = ws.worker.id.clone();
    let press_aid = aid.clone();
    let title = format!("{} · {} · {} · {}", ws.name, ws.branch, ws.worker.title, ws.path);
    let last = w_signal_last(&ws);
    let mut pr_ci: Vec<AnyElement> = Vec::new();
    if let Some(pr) = ws.pr {
        pr_ci.push(icon(ic::FORK, TEXT_XS).into_element());
        pr_ci.push(
            text(pr.to_string())
                .text_size(px(TEXT_XS))
                .text_color(move || pal().muted.opacity(0.6))
                .into_element(),
        );
        if let Some(ci) = ws.ci {
            pr_ci.push(
                text("·")
                    .text_size(px(TEXT_XS))
                    .text_color(move || pal().muted.opacity(0.4))
                    .into_element(),
            );
            let glyph = match ci {
                CiStatus::Passed => ic::CHECK_CIRCLE,
                CiStatus::Failed => ic::X_CIRCLE,
                CiStatus::Running => ic::SPINNER,
            };
            pr_ci.push(
                icon(glyph, TEXT_XS)
                    .text_color(move || {
                        let p = pal();
                        match ci {
                            CiStatus::Passed => p.ok,
                            CiStatus::Failed => p.err,
                            CiStatus::Running => p.acc,
                        }
                    })
                    .into_element(),
            );
            pr_ci.push(
                text(ci.label())
                    .text_size(px(TEXT_XS))
                    .text_color(move || {
                        let p = pal();
                        match ci {
                            CiStatus::Passed => p.ok,
                            CiStatus::Failed => p.err,
                            CiStatus::Running => p.acc,
                        }
                    })
                    .into_element(),
            );
        }
    }
    let mut children = vec![
        dot(ws.worker.status, false).into_element(),
        text(ws.name.clone())
            .text_size(px(TEXT_XS))
            .number_of_lines(1)
            .text_color(move || pal().ink)
            .into_element(),
    ];
    if !pr_ci.is_empty() {
        children.push(
            view()
                .ml(Length::Auto)
                .flex_row()
                .items_center()
                .gap(px(SP1))
                .child(pr_ci)
                .into_element(),
        );
    }
    let row = tree_row(
        1,
        move || matches!(&store().sel.get(), Some(Sel::Agent { id }) if *id == aid),
        &title,
        move || store().select(Sel::Agent { id: press_aid.clone() }),
        None,
        || true,
        children,
    );
    view()
        .relative()
        .child(row)
        .child(tree_connector(last))
        .into_element()
}

/// The connector's `last` flag: no later unarchived sibling follows —
/// computed from the fleet at build time; the row is rebuilt only when its
/// workspace is re-keyed.
fn w_signal_last(ws: &crate::types::Workspace) -> bool {
    let fleet = store().fleet.get();
    for p in &fleet.projects {
        if let Some(i) = p.workspaces.iter().position(|w| w.id == ws.id) {
            return p.workspaces[i + 1..].iter().all(|x| x.worker.archived);
        }
    }
    true
}
