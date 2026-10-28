//! `WorkspaceView`, ported from `prototype/src/pages/WorkspaceView.tsx`:
//! graph (its single worker agent; subagents are ephemeral) + the worker's
//! composer, so you can prompt it straight from the graph.

use nsh_core::prelude::*;

use crate::components::composer::{composer_bar, Variant};
use crate::components::graph::workspace_graph;
use crate::components::header::header;
use crate::store::{find_workspace, store};
use crate::theme::pal;
use crate::tokens::*;
use crate::types::Isolation;

pub fn workspace_view(id: &str) -> AnyElement {
    let wid = id.to_string();
    view()
        .flex_1()
        .min_w(px(0.0))
        .flex_col()
        .child(
            // The graph follows the fleet (the worker's status, archived);
            // the composer below stays mounted.
            dynamic(move || {
                let fleet = store().fleet.get();
                match find_workspace(&fleet, &wid) {
                    None => vec![],
                    Some((p, w)) => {
                        let sub = match w.isolation {
                            Isolation::Worktree => format!("{} · worktree · base {}", w.path, p.branch),
                            Isolation::Local => format!("{} · local checkout", w.path),
                        };
                        let crumbs = format!("{} ▸ ", p.name);
                        vec![
                            header(&w.name, Some(&sub), Some(vec![text(crumbs)
                                .text_size(px(TEXT_XS))
                                .text_color(move || pal().muted.opacity(0.6))
                                .into_element()])),
                            view()
                                .flex_1()
                                .min_h(px(0.0))
                                .child(
                                    scroll().flex_1().px(px(SP4)).py(px(SP3)).child(
                                        // The graph's node list is the
                                        // worker unless archived.
                                        workspace_graph(
                                            if w.worker.archived { &[] } else {
                                                std::slice::from_ref(&w.worker)
                                            },
                                            sel_agent_id().as_deref(),
                                        ),
                                    ),
                                )
                                .into_element(),
                        ]
                    }
                }
            }),
        )
        .child({
            // The composer's fixed facts, read untracked (see agent.rs).
            let ids = untracked(|| {
                let fleet = store().fleet.get();
                find_workspace(&fleet, id).map(|(p, w)| (p.id.clone(), w.id.clone(), w.worker.id.clone()))
            });
            match ids {
                Some((pid, wid, aid)) => composer_bar(Variant::Agent, &pid, Some(&wid), Some(&aid)),
                None => view().into_element(),
            }
        })
        .into_element()
}

/// The workspace graph highlights the selected agent.
fn sel_agent_id() -> Option<String> {
    match store().sel.get() {
        Some(crate::types::Sel::Agent { id }) => Some(id),
        _ => None,
    }
}
