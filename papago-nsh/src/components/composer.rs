//! `ComposerBar`, ported from `prototype/src/components/ComposerBar.tsx`.
//! DEC-015 Composer Bar: clean text box (no chrome inside) above the pane's
//! bottom-edge status bar — a full-bleed footer. Tern-style flat segments.
//! Every interactive segment is a `status_menu` (shared trigger+popover
//! shell); vitals are plain segments. Icons are Nerd Font glyphs.
//! DEC-014: Enter = send/steer, Alt+Enter = queue, Shift+Enter = newline.
//! Pure view: text/drafts live in the store (AGENTS.md).

use nsh_base::{button, input};
use nsh_core::prelude::*;

use crate::components::ask::ask_form;
use crate::components::primitives::{dot, menu_item, seg, seg_styled, status_menu, SegTone};
use crate::data::{approval_label, think_label, APPROVALS, BRANCHES, MODELS, THINK};
use crate::icons::{self as ic, icon};
use crate::store::{chips_for, find_agent, find_project, find_workspace, store};
use crate::theme::pal;
use crate::tokens::*;
use crate::types::{AgentRole, TimelineItem};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Variant {
    Project,
    Agent,
}

/// DEC-014 dispatch: Enter = prompt/steer, Alt+Enter = queue.
fn send_dispatch(variant: Variant, project_id: &str, agent_id: &str, alt: bool, text: &str) {
    let t = text.trim();
    if t.is_empty() {
        return;
    }
    let s = store();
    match variant {
        Variant::Project => s.spawn_from_project(project_id, t),
        Variant::Agent => s.send_to_agent(agent_id, t, alt),
    }
}

pub fn composer_bar(variant: Variant, project_id: &str, workspace_id: Option<&str>, agent_id: Option<&str>) -> AnyElement {
    let agent_id = agent_id.unwrap_or_default();
    let draft_key = match variant {
        Variant::Project => format!("project:{project_id}"),
        Variant::Agent => format!("agent:{agent_id}"),
    };
    let placeholder = match variant {
        Variant::Project => "New Agent in a fresh worktree…",
        Variant::Agent => "What are we cooking?",
    };

    // Facts fixed for the route (the role and workspace never change while
    // the page is up) — read untracked, so the region that mounts this
    // composer does not subscribe to the fleet and rebuild on every tick
    // (zustand's selectors do the same for the React app; a memo would leak
    // its eager first read into this region's scope).
    let agent_role = untracked(|| {
        let fleet = store().fleet.get();
        find_agent(&fleet, agent_id).map(|f| f.agent.role)
    });
    let workspace = untracked(|| {
        let fleet = store().fleet.get();
        workspace_id.and_then(|id| find_workspace(&fleet, id).map(|(_, w)| w.clone()))
    });
    let project_name = untracked(|| {
        let fleet = store().fleet.get();
        find_project(&fleet, project_id).map(|p| p.name.clone())
    });

    // Pending Ask: a memo, so the writing surface swaps only when an ask
    // appears or resolves — not on every fleet tick (React's conditional
    // render is equally stable).
    let pending_aid = agent_id.to_string();
    let pending = computed(move || {
        let fleet = store().fleet.get();
        let found = find_agent(&fleet, &pending_aid)?;
        found
            .agent
            .timeline
            .iter()
            .position(|it| matches!(it, TimelineItem::Ask { resolved: None, .. }))
            .map(|i| (i, found.agent.timeline[i].clone()))
    });
    // Queue / subs / streaming / todo memos: the segments appear and leave
    // as these change; nothing else rebuilds.
    let queue_aid = agent_id.to_string();
    let queue = computed(move || {
        let fleet = store().fleet.get();
        find_agent(&fleet, &queue_aid).map(|f| f.agent.queue.clone()).unwrap_or_default()
    });
    let subs_aid = agent_id.to_string();
    let subs = computed(move || {
        let fleet = store().fleet.get();
        match find_agent(&fleet, &subs_aid) {
            Some(f) => f
                .agent
                .timeline
                .iter()
                .filter(|it| matches!(it, TimelineItem::Sub { .. }))
                .cloned()
                .collect::<Vec<TimelineItem>>(),
            None => Vec::new(),
        }
    });
    let streaming_aid = agent_id.to_string();
    let streaming = computed(move || {
        let fleet = store().fleet.get();
        find_agent(&fleet, &streaming_aid).map(|f| f.agent.streaming).unwrap_or(false)
    });
    let todo_aid = agent_id.to_string();
    let todo = computed(move || {
        let fleet = store().fleet.get();
        find_agent(&fleet, &todo_aid).and_then(|f| f.agent.todo)
    });

    // ---- writing surface: clean box — or the pending Ask, which replaces it ----
    let surface_key = draft_key.clone();
    let surface_aid = agent_id.to_string();
    let surface_pid = project_id.to_string();
    let surface = view()
        .px(px(SP3))
        .pt(px(SP15 / 2.0))
        .pb(px(SP15 / 2.0))
        .child(dynamic(move || match pending.get() {
            Some((index, item)) => ask_form(&surface_aid, index, &item),
            None => composer_input(variant, &surface_pid, &surface_aid, &surface_key, placeholder),
        }));

    // ---- pane bottom edge: full-bleed status bar, aligned with the sidebar footer ----
    let mut bar: Vec<AnyElement> = Vec::new();

    // Model chip (per-Project last-used).
    let model_id = format!("chip:model:{project_id}");
    let model_label_pid = project_id.to_string();
    let model_pid = project_id.to_string();
    let model_close = model_id.clone();
    bar.push(status_menu(
        &model_id,
        true,
        Some(ic::MODEL),
        vec![text(move || chips_for(&store(), &model_label_pid).model).into_element()],
        "model (per-Project last-used)",
        rem(18.0),
        move || {
            MODELS
                .iter()
                .map(|m| {
                    let m = m.to_string();
                    let pid = model_pid.clone();
                    let close = model_close.clone();
                    let label = m.clone();
                    menu_item(chips_for(&store(), &pid).model == m, &label, move || {
                        store().set_chip(&pid, "model", m.clone());
                        store().toggle_popover(&close);
                    })
                })
                .collect()
        },
    ));

    // Thinking level.
    let th_id = format!("chip:th:{project_id}");
    let th_label_pid = project_id.to_string();
    let th_pid = project_id.to_string();
    let th_close = th_id.clone();
    bar.push(status_menu(
        &th_id,
        false,
        Some(ic::FILTER),
        vec![text(move || think_label(&chips_for(&store(), &th_label_pid).thinking)).into_element()],
        "thinking level",
        rem(12.0),
        move || {
            THINK
                .iter()
                .map(|t| {
                    let t = t.to_string();
                    let pid = th_pid.clone();
                    let close = th_close.clone();
                    menu_item(chips_for(&store(), &pid).thinking == t, think_label(&t), move || {
                        let t = t.clone();
                        store().set_chip(&pid, "thinking", t.clone());
                        store().toggle_popover(&close);
                    })
                })
                .collect()
        },
    ));

    // Approval (ADR-0004) — or the orchestrator's read-only mark.
    if agent_role == Some(AgentRole::Orchestrator) {
        let mut children = vec![icon(ic::SHIELD_SLASH, TEXT_XS).into_element()];
        children.push(text("Plan · read-only").into_element());
        bar.push(seg(false, SegTone::Muted, "orchestrator: read + plan + dispatch only — workers own writes", children));
    } else {
        let approval_id = format!("chip:approval:{project_id}");
        let approval_label_pid = project_id.to_string();
        let approval_pid = project_id.to_string();
        let approval_close = approval_id.clone();
        bar.push(status_menu(
            &approval_id,
            false,
            Some(ic::SHIELD_SLASH),
            vec![text(move || approval_label(&chips_for(&store(), &approval_label_pid).approval)).into_element()],
            "approval mode (ADR-0004)",
            rem(14.0),
            move || {
                APPROVALS
                    .iter()
                    .map(|a| {
                        let a = a.to_string();
                        let pid = approval_pid.clone();
                        let close = approval_close.clone();
                        menu_item(chips_for(&store(), &pid).approval == a, approval_label(&a), move || {
                            let a = a.clone();
                            store().set_chip(&pid, "approval", a.clone());
                            store().toggle_popover(&close);
                        })
                    })
                    .collect()
            },
        ));
    }

    // The variant's middle segment.
    match variant {
        Variant::Project => {
            // Base branch + the random slug preview (display-only, DEC-002).
            let base_id = format!("base:{project_id}");
            let base_label_pid = project_id.to_string();
            let base_pid = project_id.to_string();
            let base_close = base_id.clone();
            bar.push(status_menu(
                &base_id,
                false,
                Some(ic::FORK),
                vec![text(move || {
                    let fleet = store().fleet.get();
                    let branch = find_project(&fleet, &base_label_pid).map(|p| p.branch.clone()).unwrap_or_default();
                    format!("base: {branch}")
                })
                .into_element()],
                "base branch",
                rem(11.0),
                move || {
                    BRANCHES
                        .iter()
                        .map(|b| {
                            let b = b.to_string();
                            let pid = base_pid.clone();
                            let close = base_close.clone();
                            let fleet = store().fleet.get();
                            let active = find_project(&fleet, &pid).map(|p| p.branch == b).unwrap_or(false);
                            let b_label = b.clone();
                            menu_item(active, &b_label, move || {
                                store().set_project_branch(&pid, &b);
                                store().toggle_popover(&close);
                            })
                        })
                        .collect()
                },
            ));
            bar.push(seg(
                false,
                SegTone::Muted,
                "random slug, display-only (DEC-002)",
                vec![text(move || format!("slug: {}", store().slug_preview.get())).into_element()],
            ));
        }
        Variant::Agent => {
            if let Some(w) = &workspace {
                // The workspace the agent works in.
                let mut ws_children = vec![icon(ic::FOLDER, TEXT_XS).into_element()];
                ws_children.push(text(w.name.clone()).into_element());
                bar.push(seg(false, SegTone::Muted, &w.path, ws_children));
                let mut branch_children = vec![icon(ic::FORK, TEXT_XS).into_element()];
                branch_children.push(text(w.branch.clone()).into_element());
                if let Some(dirty) = &w.dirty {
                    branch_children.push(text(dirty.clone()).text_color(move || pal().warn).into_element());
                }
                if let Some(untracked) = &w.untracked {
                    branch_children.push(
                        text(untracked.clone()).text_color(move || pal().acc).into_element(),
                    );
                }
                bar.push(seg(false, SegTone::Warn, &w.path, branch_children));
            } else if let Some(name) = &project_name {
                let mut p_children = vec![icon(ic::FOLDER, TEXT_XS).into_element()];
                p_children.push(text(name.clone()).into_element());
                bar.push(seg(false, SegTone::Muted, project_id, p_children));
            }
        }
    }

    if variant == Variant::Agent {
        // Context usage: the 32pt track with the acc fill at ctx%.
        let ctx_aid = agent_id.to_string();
        let mut ctx_children: Vec<AnyElement> = vec![view()
            .w(px(W8))
            .h(px(SP2))
            .rounded(px(2.0))
            .overflow_hidden()
            .bg(move || pal().line)
            .child(
                view()
                    .absolute()
                    .left(px(0.0))
                    .top(px(0.0))
                    .bottom(px(0.0))
                    .w({
                        let ctx_aid = ctx_aid.clone();
                        move || {
                            let fleet = store().fleet.get();
                            let ctx = find_agent(&fleet, &ctx_aid).map(|f| f.agent.ctx).unwrap_or(0);
                            px(ctx as f32 / 100.0 * W8)
                        }
                    })
                    .bg(move || pal().acc),
            )
            .into_element()];
        let ctx_aid2 = ctx_aid.clone();
        ctx_children.push(
            text(move || {
                let fleet = store().fleet.get();
                let ctx = find_agent(&fleet, &ctx_aid2).map(|f| f.agent.ctx).unwrap_or(0);
                format!("{ctx}%")
            })
            .into_element(),
        );
        bar.push(seg(false, SegTone::Muted, "context usage", ctx_children));

        // Todo phases.
        let todo_memo = todo.clone();
        bar.push(dynamic(move || {
            todo_memo.get().map(|(done, total)| {
                seg(
                    false,
                    SegTone::Muted,
                    "todo phases",
                    vec![text(format!("todo {done}/{total}")).into_element()],
                )
            })
        }));

        // Tokens per second while streaming.
        let tps_aid = agent_id.to_string();
        let streaming_memo = streaming.clone();
        bar.push(dynamic(move || {
            streaming_memo.get().then(|| {
                seg(
                    false,
                    SegTone::Muted,
                    "tokens per second",
                    vec![text({
                        let tps_aid = tps_aid.clone();
                        move || {
                            let fleet = store().fleet.get();
                            let tps = find_agent(&fleet, &tps_aid).map(|f| f.agent.tps).unwrap_or(0);
                            format!("{tps} t/s")
                        }
                    })
                    .into_element()],
                )
            })
        }));
    }

    // Queued follow-ups (Alt+Enter).
    if variant == Variant::Agent {
        let queue_id = format!("queue:{agent_id}");
        let queue_memo = queue.clone();
        let queue_aid = agent_id.to_string();
        let queue_close = queue_id.clone();
        let queue_id_dyn = queue_id.clone();
        bar.push(dynamic(move || {
            let q = queue_memo.get();
            (!q.is_empty()).then(|| {
                let qm = queue_memo.clone();
                let queue_id_menu = queue_id_dyn.clone();
                let aid_base = queue_aid.clone();
                let close_base = queue_close.clone();
                status_menu(
                    &queue_id_menu,
                    false,
                    Some(ic::HOURGLASS),
                    vec![text(move || qm.get().len().to_string()).into_element()],
                    "queued follow-ups (Alt+Enter)",
                    rem(20.0),
                    move || {
                        let qm = qm.clone();
                        let aid_base = aid_base.clone();
                        let close_base = close_base.clone();
                        let q = qm.get();
                        q.iter()
                            .enumerate()
                            .map(|(i, item)| {
                                let item = item.clone();
                                let aid = aid_base.clone();
                                let _ = close_base.clone();
                                view()
                                    .flex_row()
                                    .items_center()
                                    .gap(px(SP15))
                                    .px(px(SP15))
                                    .py(px(SP1))
                                    .rounded(px(RADIUS_SM))
                                    .text_size(px(TEXT_XS))
                                    .text_color(move || pal().ink)
                                    .hover(|s| s.bg(white_a(ALPHA_HOVER_ALT)))
                                    .child(
                                        text(item.clone())
                                            .number_of_lines(1)
                                            .flex_1()
                                            .into_element(),
                                    )
                                    .child({
                                        let aid = aid.clone();
                                        view()
                                            .text_color(move || pal().muted)
                                            .hover(|s| s.text_color(move || pal().acc))
                                            .label("promote")
                                            .on_press(move || store().promote_queued(&aid, i))
                                            .child(icon(ic::ARROW_UP, TEXT_XS))
                                            .into_element()
                                    })
                                    .child({
                                        let aid = aid.clone();
                                        view()
                                            .text_color(move || pal().muted)
                                            .hover(|s| s.text_color(move || pal().err))
                                            .label("remove")
                                            .on_press(move || store().remove_queued(&aid, i))
                                            .child(icon(ic::X, TEXT_XS))
                                            .into_element()
                                    })
                                    .into_element()
                            })
                            .collect()
                    },
                )
            })
        }));
    }

    // Background tasks (ephemeral subagents).
    if variant == Variant::Agent {
        let bg_id = format!("bg:{agent_id}");
        let subs_memo = subs.clone();
        bar.push(dynamic(move || {
            let sub_items = subs_memo.get();
            (!sub_items.is_empty()).then(|| {
                let label_memo = subs_memo.clone();
                let content_memo = subs_memo.clone();
                status_menu(
                    &bg_id,
                    false,
                    Some(ic::TASKS),
                    vec![text(move || {
                        let items = label_memo.get();
                        let running = items
                            .iter()
                            .filter(|it| matches!(it, TimelineItem::Sub { status: crate::types::AgentStatus::Running, .. }))
                            .count();
                        format!("{running}/{}", items.len())
                    })
                    .into_element()],
                    "background tasks (ephemeral subagents)",
                    rem(20.0),
                    move || {
                        content_memo
                            .get()
                            .iter()
                            .map(|it| match it {
                                TimelineItem::Sub { title, status, .. } => {
                                    let mut children = vec![dot(*status, false).into_element()];
                                    children.push(
                                        text(title.clone())
                                            .number_of_lines(1)
                                            .flex_1()
                                            .into_element(),
                                    );
                                    children.push(
                                        text(status.label())
                                            .text_color(move || pal().muted.opacity(0.6))
                                            .into_element(),
                                    );
                                    view()
                                        .flex_row()
                                        .items_center()
                                        .gap(px(SP15))
                                        .px(px(SP15))
                                        .py(px(SP1))
                                        .rounded(px(RADIUS_SM))
                                        .text_size(px(TEXT_XS))
                                        .text_color(move || pal().ink)
                                        .child(children)
                                        .into_element()
                                }
                                _ => view().into_element(),
                            })
                            .collect()
                    },
                )
            })
        }));
    }

    // The spring, then abort (while streaming) and send.
    bar.push(view().flex_1().into_element());

    if variant == Variant::Agent {
        // The abort press target: the React Seg's onClick aborts.
        let abort_aid = agent_id.to_string();
        let streaming_memo = streaming.clone();
        bar.push(dynamic(move || {
            streaming_memo.get().then(|| {
                let abort_aid = abort_aid.clone();
                seg_styled(button(), false, SegTone::Err)
                    .accessibility_label("abort (immediate, no confirm)")
                    .hover(|s| s.bg(white_a(ALPHA_HOVER_ALT)))
                    .on_press(move || store().abort_agent(&abort_aid))
                    .child(icon(ic::STOP, TEXT_XS).text_color(move || pal().err))
                    .into_element()
            })
        }));
    }

    // Send: Enter steers while streaming, Alt+Enter queues. The icon dims
    // while the draft is empty (hasText in the prototype).
    let send_color_key = draft_key.clone();
    let send_icon_key = draft_key.clone();
    let send_press_key = draft_key.clone();
    let send_pid = project_id.to_string();
    let send_aid = agent_id.to_string();
    bar.push(
        seg_styled(button(), false, SegTone::Default)
            .accessibility_label("Send · Enter steers while streaming · Alt+Enter queues")
            .hover(|s| s.bg(white_a(ALPHA_HOVER_ALT)))
            .on_press(move || {
                let t = store().draft(&send_press_key);
                send_dispatch(variant, &send_pid, &send_aid, false, &t);
            })
            .child(
                icon(ic::SEND, TEXT_XS).text_color(move || {
                    let p = pal();
                    if store().draft(&send_color_key).trim().is_empty() {
                        p.muted.opacity(0.4)
                    } else {
                        p.acc
                    }
                }),
            )
            .into_element(),
    );
    let _ = send_icon_key;

    view()
        .flex_shrink_0()
        .child(surface)
        .child(
            view()
                .h(px(H7))
                .bg(move || pal().glass2)
                .backdrop_blur(px(BLUR_XL))
                .border_t(px(HAIRLINE))
                .border_color(white_a(ALPHA_BORDER))
                .flex_row()
                .items_stretch()
                .child(bar),
        )
        .into_element()
}

fn composer_input(variant: Variant, project_id: &str, agent_id: &str, draft_key: &str, placeholder: &str) -> AnyElement {
    let project_id = project_id.to_string();
    let agent_id = agent_id.to_string();
    let placeholder = placeholder.to_string();
    // The growing multiline input (field-sizing-content in the prototype):
    // its height follows the content, clamped at max-h-40.
    let height = signal(px(rem(2.5))); // rows=1 → ~40pt
    let key = draft_key.to_string();
    let submit_variant = variant;
    let submit_project = project_id.clone();
    let submit_agent = agent_id.clone();
    input()
        .multiline(true)
        .value({
            let key = key.clone();
            move || store().draft(&key)
        })
        .on_change({
            let key = key.clone();
            move |t: &str| store().set_draft(&key, t.to_string())
        })
        .on_submit(move |t: &str| send_dispatch(submit_variant, &submit_project, &submit_agent, false, t))
        .submit_behavior(SubmitBehavior::Submit)
        .placeholder(placeholder.as_str())
        .w_full()
        .max_h(px(rem(10.0))) // max-h-40
        .on_content_size_change(move |size: &Size<Points>| {
            height.set(px((size.height.0 + 16.0).clamp(rem(2.5), rem(10.0))));
        })
        .h(move || height.get())
        .px(px(SP3 + SP1)) // px-3.5
        .py(px(SP2))
        .text_size(px(TEXT_SM))
        .line_height(px(TEXT_SM_LINE))
        .bg(move || pal().glass1)
        .backdrop_blur(px(BLUR_MD))
        .rounded(px(RADIUS_LG))
        .border(px(HAIRLINE))
        .border_color(white_a(ALPHA_BORDER))
        .focus(|s| s.border_color(move || pal().acc.opacity(0.6)))
        .into_element()
}
