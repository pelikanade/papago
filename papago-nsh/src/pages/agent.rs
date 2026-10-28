//! `AgentView`, ported from `prototype/src/pages/AgentView.tsx`: one agent's
//! timeline + its composer. Workers sit inside a workspace; the
//! orchestrator sits directly under the project (no workspace).

use nsh_core::prelude::*;

use crate::components::composer::{composer_bar, Variant};
use crate::components::header::header;
use crate::components::timeline::timeline;
use crate::icons::{self as ic, icon};
use crate::store::{find_agent, store};
use crate::theme::pal;
use crate::tokens::*;
use crate::types::AgentRole;

pub fn agent_view(id: &str) -> AnyElement {
    // Route-fixed facts: the title, the role, the project, the workspace —
    // read untracked, so the routing region that mounts this page does not
    // subscribe to the fleet and rebuild it on every tick (zustand's
    // selectors do the same for the React app; a memo would leak its eager
    // first read into the region's scope).
    let found = untracked(|| {
        let fleet = store().fleet.get();
        find_agent(&fleet, id).map(|f| {
            (
                f.agent.title.clone(),
                f.agent.role,
                f.project.name.clone(),
                f.project.id.clone(),
                f.workspace.map(|w| (w.name.clone(), w.id.clone())),
            )
        })
    });

    let (title, crumbs, project_id, workspace_id): (String, Vec<AnyElement>, String, Option<String>) =
        match found {
            None => (String::new(), vec![], String::new(), None),
            Some((agent_title, role, project_name, pid, workspace)) => {
                let title = match role {
                    AgentRole::Orchestrator => "orchestrator".to_string(),
                    AgentRole::Worker => agent_title,
                };
                let mut crumbs: Vec<AnyElement> = vec![
                    text(project_name)
                        .text_size(px(TEXT_XS))
                        .text_color(move || pal().muted.opacity(0.6))
                        .into_element(),
                    icon(ic::ANGLE_RIGHT, TEXT_XS)
                        .text_color(move || pal().muted.opacity(0.6))
                        .into_element(),
                ];
                if let Some((wname, _)) = &workspace {
                    crumbs.push(
                        text(wname.clone())
                            .text_size(px(TEXT_XS))
                            .text_color(move || pal().muted.opacity(0.6))
                            .into_element(),
                    );
                    crumbs.push(
                        icon(ic::ANGLE_RIGHT, TEXT_XS)
                            .text_color(move || pal().muted.opacity(0.6))
                            .into_element(),
                    );
                }
                (title, crumbs, pid, workspace.map(|(_, wid)| wid))
            }
        };

    view()
        .flex_1()
        .min_w(px(0.0))
        .flex_col()
        .child(header(&title, None, Some(crumbs)))
        .child(timeline(id))
        .child(match workspace_id {
            Some(wid) => composer_bar(Variant::Agent, &project_id, Some(&wid), Some(id)),
            None => composer_bar(Variant::Agent, &project_id, None, Some(id)),
        })
        .into_element()
}
