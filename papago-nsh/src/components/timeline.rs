//! `Timeline`, ported from `prototype/src/components/Timeline.tsx`. Design
//! decision (Synth, 2026-10-27): every timeline item shares ONE visual
//! grammar — a borderless row: status icon · type label (Read / Edit / Bash
//! / Write / Subagent / Ask) · muted detail · right-aligned meta. Expandable
//! items (edit/write/bash) swap the status icon for a caret on hover; the
//! row itself never has a background or border — only the expanded body
//! does. All rows go through `row`; item kinds only supply label/detail/body.

use nsh_base::button;
use nsh_core::prelude::*;

use crate::icons::{self as ic, icon, NF};
use crate::store::{find_agent, store};
use crate::theme::pal;
use crate::tokens::*;
use crate::types::{AgentStatus, TimelineItem, ToolStatus};

pub fn timeline(agent_id: &str) -> AnyElement {
    let scroller = element_ref();
    let scroll_aid = agent_id.to_string();
    let effect_aid = agent_id.to_string();
    let tail_aid = agent_id.to_string();
    let each_aid = agent_id.to_string();
    let scroller2 = scroller.clone();
    // Tail-follow without effects (useEffect is banned in the prototype): a
    // MutationObserver pins the scroller to the bottom while followTail is
    // true — the port pins it from an effect over the timeline, which the
    // engine re-runs whenever this agent's timeline changes.
    effect(move || {
        store().fleet.get();
        if store().follow(&effect_aid) {
            scroller2.scroll_to_end(false);
        }
    });
    view()
        .relative()
        .flex_1()
        .min_h(px(0.0))
        .child(
            scroll()
                .element_ref(&scroller)
                .h_full()
                .on_scroll(move |e: &ScrollEvent| {
                    // followTail flips back on only near the bottom (24pt).
                    let at_bottom = e.content_size.height.0 - e.offset.y.0 - e.size.height.0 < 24.0;
                    store().set_follow_tail(&scroll_aid, at_bottom);
                })
                .px(px(SP4))
                .py(px(SP3))
                .child(
                    // The transcript selects as one page: a drag starts in
                    // one item and ends in another, Ctrl+C copies.
                    view()
                        .gap(px(SP15))
                        .selectable(true)
                        .child(each(
                            {
                                let each_aid = each_aid.clone();
                                move || {
                                    let fleet = store().fleet.get();
                                    find_agent(&fleet, &each_aid)
                                        .map(|f| f.agent.timeline.clone())
                                        .unwrap_or_default()
                                }
                            },
                            item_key,
                            move |item, i| item_view(&each_aid, &item, i),
                        ))
                        .child(view().h(px(SP1))),
                )
        )
        .child(dynamic(move || {
            (!store().follow(&tail_aid)).then(|| {
                let aid = tail_aid.clone();
                view()
                    .absolute()
                    .bottom(px(SP3))
                    .right(px(SP4))
                    .px(px(SP2))
                    .h(px(H6))
                    .flex_row()
                    .items_center()
                    .gap(px(SP1))
                    .rounded(px(RADIUS_FULL))
                    .bg(move || pal().surface)
                    .border(px(HAIRLINE))
                    .border_color(move || pal().line)
                    .text_size(px(TEXT_XS))
                    .text_color(move || pal().muted)
                    .hover(|s| s.text_color(move || pal().ink))
                    .label("tail")
                    .on_press(move || store().set_follow_tail(&aid, true))
                    .child(icon(ic::ARROW_DOWN, TEXT_XS))
                    .child(text("tail"))
                    .into_element()
            })
        }))
        .into_element()
}

/// React keyed the items by index; the port keys them by content, which is
/// stable for the append-only mock timelines (a resolved Ask or a running
/// tool's `took` does not re-key its row). The fold key keeps the store's
/// `${agentId}:${index}` shape, from each()'s row index.
fn item_key(it: &TimelineItem) -> String {
    match it {
        TimelineItem::User { text } => format!("user:{text}"),
        TimelineItem::Md { text } => format!("md:{text}"),
        TimelineItem::Tool { verb, target, .. } => format!("tool:{verb}:{target}"),
        TimelineItem::Sub { title, .. } => format!("sub:{title}"),
        TimelineItem::Ask { q, .. } => format!("ask:{q}"),
    }
}

fn item_view(agent_id: &str, item: &ReadSignal<TimelineItem>, i: ReadSignal<usize>) -> AnyElement {
    match item.get() {
        TimelineItem::User { text: body } => view()
            .flex_row()
            .justify_end()
            .child(
                view()
                    .max_w(Length::Definite(DefiniteLength::Fraction(Percentage(0.7))))
                    .rounded(px(RADIUS_LG))
                    .bg(move || pal().glass1)
                    .backdrop_blur(px(BLUR_MD))
                    .border(px(HAIRLINE))
                    .border_color(white_a(ALPHA_BORDER))
                    .px(px(SP3))
                    .py(px(SP2))
                    .text_size(px(TEXT_SM))
                    .text_color(move || pal().ink)
                    .child(text(body)),
            )
            .into_element(),
        TimelineItem::Md { text } => {
            md_text(&text).text_size(px(TEXT_SM)).line_height(px(TEXT_SM_LINE)).into_element()
        }
        TimelineItem::Tool { .. } => tool_line(agent_id, item, i),
        TimelineItem::Sub { .. } => sub_line(item),
        TimelineItem::Ask { .. } => ask_line(item),
    }
}

/// The shared timeline row shell (see the design decision above): icon ·
/// label · detail · meta; hover swaps the status icon for a caret when
/// expandable — a row-local hover signal stands in for the CSS
/// `group-hover:`. The expanded body follows the row, inside the wrapper
/// `div` the React component renders.
#[allow(clippy::too_many_arguments)]
fn row(
    glyph: &'static str,
    tone: impl Fn() -> Rgba + 'static,
    label: &str,
    detail: Vec<AnyElement>,
    meta: Vec<AnyElement>,
    expandable: bool,
    open: bool,
    on_toggle: Option<Box<dyn Fn() + 'static>>,
    body: Vec<AnyElement>,
) -> AnyElement {
    let hover = signal(false);
    let enter_hover = hover.clone();
    let leave_hover = hover.clone();
    let glyph_hover = hover.clone();
    let caret_hover = hover.clone();

    // The icon slot: the caret down when open; else the glyph, which hides
    // for the caret while the row is hovered (expandable rows only).
    let icon_slot: View = if expandable && open {
        view()
            .w(px(SP4))
            .h(px(SP4))
            .flex_row()
            .items_center()
            .justify_center()
            .flex_shrink_0()
            .child(icon(ic::CARET_DOWN, TEXT_SM).text_color(move || pal().muted.opacity(0.7)))
    } else {
        let slot = view()
            .relative()
            .w(px(SP4))
            .h(px(SP4))
            .flex_row()
            .items_center()
            .justify_center()
            .flex_shrink_0()
            .child(
                view()
                    .opacity(move || if glyph_hover.get() && expandable { 0.0f32 } else { 1.0f32 })
                    .child(text(glyph.to_owned()).font_family(NF.family).text_color(tone)),
            );
        if expandable {
            slot.child(
                view()
                    .absolute()
                    .inset(px(0.0))
                    .flex_row()
                    .items_center()
                    .justify_center()
                    .opacity(move || if caret_hover.get() { 1.0f32 } else { 0.0f32 })
                    .child(icon(ic::CARET_RIGHT, TEXT_SM).text_color(move || pal().muted.opacity(0.7))),
            )
        } else {
            slot
        }
    };

    let mut content: Vec<AnyElement> = vec![
        icon_slot.into_element(),
        text(label.to_owned()).text_size(px(TEXT_SM)).text_color(move || pal().ink).into_element(),
    ];
    content.extend(detail);
    if !meta.is_empty() {
        content.push(
            view()
                .ml(Length::Auto)
                .flex_row()
                .items_center()
                .gap(px(SP2))
                .text_size(px(TEXT_XS))
                .child(meta)
                .into_element(),
        );
    }

    let row = if expandable {
        view()
            .on_hover_in(move || enter_hover.set(true))
            .on_hover_out(move || leave_hover.set(false))
            .child(
                button()
                    .on_press(on_toggle.unwrap_or_else(|| Box::new(|| {})))
                    .accessibility_label(label.to_owned())
                    .w_full()
                    .flex_row()
                    .items_center()
                    .gap(px(SP2))
                    .px(px(SP1))
                    .py(px(0.5))
                    .child(content),
            )
            .into_element()
    } else {
        view()
            .label(label.to_owned())
            .w_full()
            .flex_row()
            .items_center()
            .gap(px(SP2))
            .px(px(SP1))
            .py(px(0.5))
            .child(content)
            .into_element()
    };
    view()
        .child(row)
        .child(if expandable && open { body } else { vec![] })
        .into_element()
}

fn status_glyph(status: ToolStatus) -> &'static str {
    match status {
        ToolStatus::Done => ic::CHECK,
        ToolStatus::Error => ic::X,
        ToolStatus::Running => ic::SPINNER,
    }
}

fn status_tone(status: ToolStatus) -> impl Fn() -> Rgba + 'static {
    move || {
        let p = pal();
        match status {
            ToolStatus::Done => p.ok,
            ToolStatus::Error => p.err,
            ToolStatus::Running => p.acc,
        }
    }
}

fn as_tool_status(s: AgentStatus) -> ToolStatus {
    match s {
        AgentStatus::Done => ToolStatus::Done,
        AgentStatus::Error => ToolStatus::Error,
        _ => ToolStatus::Running,
    }
}

fn type_label(verb: &str) -> String {
    let mut c = verb.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

fn tool_line(agent_id: &str, item: &ReadSignal<TimelineItem>, i: ReadSignal<usize>) -> AnyElement {
    let (verb, target, status, diff, before, after) = match item.get() {
        TimelineItem::Tool { verb, target, status, diff, before, after, .. } => {
            (verb, target, status, diff, before, after)
        }
        _ => return view().into_element(),
    };
    let expandable = verb == "bash" || before.is_some() || after.is_some();
    // Synth-confirmed (2026-10-27): the running-time number MUST move at
    // 0.1s resolution — the stopwatch feel is the intended design, not
    // jitter. Elapsed time advances from the run source (mock: 100ms store
    // clock, tickClock); no local timers.
    let took_item = item.clone();
    // The fold re-renders the row: a keyed row is built once, so the open
    // state cannot be captured at build time (React re-renders on the fold
    // map; the port re-renders this row alone).
    let fold_key = format!("{}:{}", agent_id, i.get());
    dynamic(move || {
        let open = store().fold(&fold_key);
        let toggle_key = fold_key.clone();
        let mut meta: Vec<AnyElement> = Vec::new();
        if let Some(d) = diff {
            meta.push(
                text("")
                    .text_color(move || pal().muted.opacity(0.7))
                    .child(
                        text(format!("+{}", d.add))
                            .text_color(move || pal().ok)
                            .into_element(),
                    )
                    .child(" ")
                    .child(
                        text(format!("−{}", d.del))
                            .text_color(move || pal().err)
                            .into_element(),
                    )
                    .into_element(),
            );
        }
        meta.push(
            text(move || match took_item.get() {
                TimelineItem::Tool { took, .. } => took,
                _ => String::new(),
            })
            .text_color(move || pal().muted.opacity(0.6))
            .tabular_nums()
            .into_element(),
        );

        let body: Vec<AnyElement> = vec![
            view()
                .mt(px(SP1))
                .ml(px(SP6))
                .rounded(px(RADIUS_MD))
                .border(px(HAIRLINE))
                .border_color(white_a(ALPHA_BORDER))
                .bg(move || pal().glass1)
                .backdrop_blur(px(BLUR_SM))
                .px(px(SP3))
                .py(px(SP2))
                .child(if before.is_some() || after.is_some() {
                    split_diff(before.as_deref().unwrap_or(&[]), after.as_deref().unwrap_or(&[]))
                } else if verb == "bash" {
                    mono_pre("$ go test ./... -run Rate\nok  api/rate  8.1s\nok  api/auth  0.9s")
                } else {
                    mono_pre("…output preview (tail)…")
                })
                .into_element(),
        ];

        row(
            status_glyph(status),
            status_tone(status),
            &type_label(&verb),
            vec![
                text(target.clone())
                    .text_size(px(TEXT_SM))
                    .font_family("monospace")
                    .number_of_lines(1)
                    .text_color(move || pal().muted)
                    .into_element(),
            ],
            meta,
            expandable,
            open,
            Some(Box::new(move || store().toggle_fold(&toggle_key))),
            body,
        )
    })
    .into_element()
}

fn mono_pre(source: &'static str) -> AnyElement {
    text(source)
        .text_size(px(TEXT_XS))
        .font_family("monospace")
        .line_height(px(TEXT_XS_LINE))
        .text_color(move || pal().muted)
        .into_element()
}

fn sub_line(item: &ReadSignal<TimelineItem>) -> AnyElement {
    let (title, status, model, tool, tokens) = match item.get() {
        TimelineItem::Sub { title, status, model, tool, tokens } => (title, status, model, tool, tokens),
        _ => return view().into_element(),
    };
    let tool_status = as_tool_status(status);
    row(
        status_glyph(tool_status),
        status_tone(tool_status),
        "Subagent",
        vec![
            text(title.clone())
                .text_size(px(TEXT_SM))
                .number_of_lines(1)
                .text_color(move || pal().muted)
                .into_element(),
        ],
        vec![
            text(format!("{model} · {tool} · {tokens}"))
                .text_color(move || pal().muted.opacity(0.7))
                .number_of_lines(1)
                .into_element(),
        ],
        false,
        false,
        None,
        vec![],
    )
}

fn ask_line(item: &ReadSignal<TimelineItem>) -> AnyElement {
    let (q, resolved) = match item.get() {
        TimelineItem::Ask { q, resolved, .. } => (q, resolved),
        _ => return view().into_element(),
    };
    let done = resolved.is_some();
    let glyph: &'static str = if done { ic::CHECK } else { "?" };
    let detail = if done {
        text(q.clone())
            .text_size(px(TEXT_SM))
            .number_of_lines(1)
            .line_through()
            .text_decoration_color(move || pal().muted.opacity(0.4))
            .text_color(move || pal().muted)
            .into_element()
    } else {
        text(q.clone())
            .text_size(px(TEXT_SM))
            .number_of_lines(1)
            .text_color(move || pal().muted)
            .into_element()
    };
    let meta = if done {
        text(resolved.clone().unwrap_or_default())
            .text_color(move || pal().ink)
            .into_element()
    } else {
        text("answer below")
            .text_color(move || pal().warn.opacity(0.8))
            .into_element()
    };
    row(
        glyph,
        move || {
            let p = pal();
            if done {
                p.ok
            } else {
                p.warn
            }
        },
        "Ask",
        vec![detail],
        vec![meta],
        false,
        false,
        None,
        vec![],
    )
}

/// side-by-side diff: old lines left, new lines right; compact mono at
/// text-xs.
fn split_diff(before: &[String], after: &[String]) -> AnyElement {
    view()
        .flex_row()
        .gap(px(SP3))
        .text_size(px(TEXT_XS))
        .font_family("monospace")
        .line_height(px(TEXT_XS_LINE))
        .child(diff_column("before", before, false))
        .child(diff_column("after", after, true))
        .into_element()
}

fn diff_column(label: &str, lines: &[String], added: bool) -> AnyElement {
    let label = label.to_owned();
    let label = label.to_owned();
    let marker = if added { "+" } else { "−" };
    view()
        .flex_1()
        .min_w(px(0.0))
        .flex_col()
        .child(
            text(label)
                .text_size(px(TEXT_XS))
                .text_color(move || pal().muted.opacity(0.6))
                .mb(px(SP1))
                .into_element(),
        )
        .child(
            lines
                .iter()
                .map(|l| {
                    text("")
                        .child(
                            text(marker)
                                .text_color(move || pal().muted.opacity(0.4))
                                .mr(px(SP2))
                                .into_element(),
                        )
                        .child(l.clone())
                        .number_of_lines(1)
                        .text_color(move || {
                            let p = pal();
                            if added {
                                p.ok.opacity(0.9)
                            } else {
                                p.err.opacity(0.9)
                            }
                        })
                        .into_element()
                })
                .collect::<Vec<AnyElement>>(),
        )
        .into_element()
}

/// `renderMd`: `code` spans in the mono face with a surface chip, **bold**
/// spans semibold, everything else plain — whitespace-pre-wrap.
fn md_text(source: &str) -> Text {
    let mut out = text("");
    for part in split_md(source) {
        if part.starts_with('`') && part.ends_with('`') && part.len() >= 2 {
            out = out.child(
                text(part[1..part.len() - 1].to_string())
                    .font_family("monospace")
                    .text_size(px(TEXT_XS))
                    .bg(move || pal().surface)
                    .border(px(HAIRLINE))
                    .border_color(move || pal().line.opacity(0.6))
                    .rounded(px(RADIUS_SM))
                    .px(px(SP1))
                    .text_color(move || pal().acc.opacity(0.9))
                    .into_element(),
            );
        } else if part.starts_with("**") && part.ends_with("**") && part.len() >= 4 {
            out = out.child(text(part[2..part.len() - 2].to_string()).font_weight(FontWeight::SEMIBOLD).into_element());
        } else if !part.is_empty() {
            out = out.child(part);
        }
    }
    out
}

/// JS `text.split(/(`[^`]+`|\*\*[^*]+\*\*)/g)` with the separators kept.
fn split_md(source: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut plain = String::new();
    let mut i = 0;
    while i < source.len() {
        let rest = &source[i..];
        if rest.starts_with('`') {
            if let Some(end) = rest[1..].find('`') {
                if !plain.is_empty() {
                    out.push(std::mem::take(&mut plain));
                }
                out.push(rest[..end + 2].to_string());
                i += end + 2;
                continue;
            }
        }
        if rest.starts_with("**") {
            if let Some(end) = rest[2..].find("**") {
                if !plain.is_empty() {
                    out.push(std::mem::take(&mut plain));
                }
                out.push(rest[..end + 4].to_string());
                i += end + 4;
                continue;
            }
        }
        let ch = rest.chars().next().unwrap();
        plain.push(ch);
        i += ch.len_utf8();
    }
    if !plain.is_empty() {
        out.push(plain);
    }
    out
}
