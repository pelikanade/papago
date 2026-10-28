//! Shared shells, ported from `prototype/src/components/primitives.tsx`.
//! Design decision (Synth, 2026-10-27): status-bar items and tree rows are
//! shared shells — every trigger+popover goes through `status_menu`, every
//! tree row through `tree_row`. Extract a component the moment the pattern
//! appears twice (AGENTS.md).

use nsh_base::{button, ButtonElement};
use nsh_core::prelude::*;

use crate::icons::{self as ic, icon};
use crate::store::store;
use crate::theme::{dot_color, pal, ring_color};
use crate::tokens::*;
use crate::types::AgentStatus;

/// `Dot` — the status dot (see the semantics comment on STATUS_DOT).
/// `ring` draws the hollow orchestrator form (same status colour, no fill).
pub fn dot(status: AgentStatus, ring: bool) -> View {
    let s = store();
    let mut d = view().w(px(DOT)).h(px(DOT)).rounded(px(RADIUS_FULL)).flex_shrink_0();
    if ring {
        let pulse = s.ring_pulse.clone();
        d = d
            .border(px(2.0))
            .bg(clear())
            .border_color(move || ring_color(&pal(), status));
        if status == AgentStatus::Running {
            d = d.bind_opacity(&pulse);
        }
    } else {
        let pulse = s.pulse.clone();
        d = d.bg(move || dot_color(&pal(), status));
        if status == AgentStatus::Running {
            d = d.bind_opacity(&pulse);
        }
    }
    d
}

/// Tailwind's `shadow-2xl` (the window frame and the floating overlays).
pub fn shadow_2xl() -> Vec<BoxShadow> {
    vec![BoxShadow {
        offset_x: px(0.0),
        offset_y: px(SHADOW_2XL_Y),
        blur: px(SHADOW_2XL_BLUR),
        spread: px(SHADOW_2XL_SPREAD),
        color: rgb(0).opacity(0.25).into(),
        inset: false,
    }]
}

/// `Popover`'s panel: an overlay placed against the trigger
/// (`bottom-[calc(100%+6px)]`, `left-0`/`right-0`, `bg-panel border
/// border-white/10 rounded-md shadow-2xl p-2`). NSH's Overlay reports
/// presses outside itself and its anchor, which replaces the React
/// full-screen catcher div; Escape does not close it, as in the prototype.
pub fn overlay_panel(
    anchor: &ElementRef,
    align_end: bool,
    width: f32,
    close: impl Fn() + 'static,
    children: Vec<AnyElement>,
) -> Overlay {
    overlay()
        .anchor_content(anchor)
        .placement(Placement::Top)
        .offset(px(6.0))
        .align(if align_end { Align::End } else { Align::Start })
        .on_press_outside(move |_: &Point<Points>| close())
        .w(px(width))
        .bg(move || pal().panel)
        .border(px(1.0))
        .border_color(white_a(ALPHA_BORDER))
        .rounded(px(RADIUS_MD))
        .shadow(shadow_2xl())
        .p(px(2.0))
        .child(children)
}

/// `Seg` tones.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum SegTone {
    Default,
    Muted,
    Warn,
    Err,
    Brand,
}

fn seg_text_color(tone: SegTone) -> impl Fn() -> Rgba + 'static {
    move || {
        let p = pal();
        match tone {
            SegTone::Warn => p.warn,
            SegTone::Err => p.err,
            SegTone::Muted => p.muted.opacity(0.7),
            SegTone::Brand => p.acc,
            SegTone::Default => p.muted,
        }
    }
}

/// `Seg`'s shared look: `h-full px-3 flex items-center gap-1.5 text-xs`, a
/// hairline on the left unless `first`. Interactive segments are built on a
/// `button()`, quiet ones on a `view()` — the caller passes the base, which
/// keeps one style chain for both (the React `Tag = button | div`).
pub fn seg_styled<S: Element>(base: S, first: bool, tone: SegTone) -> S {
    let color = seg_text_color(tone);
    let brand_bg = tone == SegTone::Brand;
    let s = base
        .h_full()
        .px(px(SP3))
        .flex_row()
        .items_center()
        .gap(px(SP15))
        .text_size(px(TEXT_XS))
        .text_color(color);
    let s = if first { s } else { s.border_l(px(HAIRLINE)).border_color(move || pal().line.opacity(0.4)) };
    if brand_bg {
        s.bg(white_a(0.07))
    } else {
        s
    }
}

/// A quiet `Seg` (a `div` in the prototype).
pub fn seg(first: bool, tone: SegTone, title: &str, children: Vec<AnyElement>) -> AnyElement {
    seg_styled(view(), first, tone)
        .label(title.to_owned())
        .child(children)
        .into_element()
}

/// `MenuItem`: one popover row — `w-full flex items-center gap-2 px-2 py-2
/// rounded text-xs`, the check when active.
pub fn menu_item(active: bool, label: &str, on_press: impl Fn() + 'static) -> AnyElement {
    button()
        .w_full()
        .flex_row()
        .items_center()
        .gap(px(SP2))
        .px(px(SP2))
        .py(px(SP2))
        .rounded(px(RADIUS_SM))
        .text_size(px(TEXT_XS))
        .text_color(move || if active { pal().acc } else { pal().ink })
        .hover(|s| s.bg(white_a(ALPHA_HOVER_ALT)))
        .on_press(on_press)
        .child(if active {
            icon(ic::CHECK, TEXT_XS).into_element()
        } else {
            view().w(px(SP4)).into_element()
        })
        .child(text(label.to_owned()).text_size(px(TEXT_XS)).number_of_lines(1))
        .into_element()
}

/// `StatusMenu`: a `Seg` trigger plus its anchored popover, both driven by
/// the `popovers` map. The trigger carries icon + label + the caret.
/// `first` carries the React `first:border-l-0` (the bar's left edge).
pub fn status_menu(
    id: &str,
    first: bool,
    glyph: Option<&'static str>,
    label: Vec<AnyElement>,
    title: &str,
    width: f32,
    content: impl Fn() -> Vec<AnyElement> + 'static,
) -> AnyElement {
    let sid = id.to_string();
    let anchor = element_ref();
    let toggle_sid = sid.clone();
    let popover_sid = sid.clone();
    let close_sid = sid;
    let anchor2 = anchor.clone();
    let mut trigger = label;
    if let Some(g) = glyph {
        trigger.insert(0, icon(g, TEXT_SM).into_element());
    }
    trigger.push(icon(ic::CARET_DOWN, TEXT_XS).into_element());
    view()
        .relative()
        .flex_row()
        .child(
            seg_styled(button(), first, SegTone::Default)
                .accessibility_label(title.to_owned())
                .hover(|s| s.bg(white_a(ALPHA_HOVER_ALT)))
                .on_press(move || store().toggle_popover(&toggle_sid))
                .child(trigger)
                .element_ref(&anchor)
                .into_element(),
        )
        .child(dynamic(move || {
            store().popover(&popover_sid).then(|| {
                let close_sid = close_sid.clone();
                let anchor2 = anchor2.clone();
                overlay_panel(
                    &anchor2,
                    false,
                    width,
                    move || store().toggle_popover(&close_sid),
                    content(),
                )
            })
        }))
        .into_element()
}

/// `TreeRow`: `w-full flex items-center gap-2 py-1.5` with the indent pads
/// of the prototype (`px-2`, `pl-6 pr-2`, …). The fold toggle is its own
/// hit target; the row click selects the row's agent.
pub fn tree_row(
    indent: u8,
    selected: impl Fn() -> bool + 'static,
    title: &str,
    on_click: impl Fn() + 'static,
    on_caret: Option<Box<dyn Fn() + 'static>>,
    open: impl Fn() -> bool + 'static,
    children: Vec<AnyElement>,
) -> AnyElement {
    let mut row = button()
        .on_press(on_click)
        .accessibility_label(title.to_owned())
        .w_full()
        .flex_row()
        .items_center()
        .gap(px(SP2))
        .py(px(SP15))
        .bg(move || {
            if selected() {
                white_a(ALPHA_SELECTED)
            } else {
                clear()
            }
        })
        .hover(|s| s.bg(white_a(ALPHA_HOVER)));
    row = match indent {
        0 => row.px(px(SP2)),
        1 => row.pl(px(SP6)).pr(px(SP2)),
        2 => row.pl(px(SP10)).pr(px(SP2)),
        _ => row.pl(px(SP14)).pr(px(SP2)),
    };
    if let Some(f) = on_caret {
        row = row.child(
            view()
                .on_press(f)
                .label("fold")
                .px(px(SP1))
                .flex_shrink_0()
                .text_color(move || pal().muted.opacity(0.6))
                .child(dynamic(move || {
                    icon(if open() { ic::CARET_DOWN } else { ic::CARET_RIGHT }, TEXT_XS)
                }))
                .into_element(),
        );
    }
    row.child(children).into_element()
}

/// `IconButton`: icon-only toolbar button (window controls, panel header
/// actions) — one shell so the size/hover/tone behaviour never drifts
/// between surfaces. `h-7 w-7 rounded hover:bg-white/[0.07]`; the err tone
/// hovers red text.
pub fn icon_button(glyph: &str, title: &str, tone_err: bool, on_press: impl Fn() + 'static) -> ButtonElement {
    let mut b = button()
        .h(px(H7))
        .w(px(H7))
        .items_center()
        .justify_center()
        .rounded(px(RADIUS_SM))
        .accessibility_label(title.to_owned())
        .text_color(move || pal().muted)
        .on_press(on_press)
        .child(icon(glyph, TEXT_SM));
    b = if tone_err {
        b.hover(|s| s.text_color(move || pal().err))
    } else {
        b.hover(|s| s.bg(white_a(ALPHA_HOVER_ALT)))
    };
    b
}

/// `TreeConnector`: the hairline that ties a child row to its parent. A
/// vertical runs at the parent caret's centre (left-2.5 = 10px) down the
/// group; on the last row it stops at the row's midline and turns right into
/// an elbow that meets the child's status dot. Rows are 28pt tall (py-1.5 +
/// a 16pt line), so the midline is 14pt — the fixed stand-in for Tailwind's
/// `top-1/2`. Rendered after the row inside a `relative` wrapper.
pub fn tree_connector(last: bool) -> Vec<AnyElement> {
    let vertical = if last {
        view()
            .absolute()
            .left(px(SP25))
            .top(px(0.0))
            .h(px(H7 / 2.0))
            .w(px(HAIRLINE))
    } else {
        view().absolute().left(px(SP25)).top(px(0.0)).bottom(px(0.0)).w(px(HAIRLINE))
    }
    .bg(white_a(ALPHA_BORDER))
    .pointer_events(PointerEvents::None)
    .into_element();
    if last {
        vec![
            vertical,
            view()
                .absolute()
                .left(px(SP25))
                .top(px(H7 / 2.0))
                .w(px(SP35))
                .h(px(HAIRLINE))
                .bg(white_a(ALPHA_BORDER))
                .pointer_events(PointerEvents::None)
                .into_element(),
        ]
    } else {
        vec![vertical]
    }
}
