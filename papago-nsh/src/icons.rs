//! Nerd Font icon glyphs (the Font Awesome set shipped in FiraCode Nerd Font
//! Mono), mirroring `prototype/src/icons.ts`: every visual icon in the
//! prototype comes from here — no emoji, no plain-text arrows, no SVGs. The
//! font itself ships as the crate's asset (`assets/fonts/`).

use nsh_core::prelude::*;

/// The FiraCode Nerd Font Mono face the `.nf` class asks for
/// (`prototype/src/index.css`).
pub const NF: FontAsset = asset!("fonts/FiraCodeNerdFontMono-Regular.ttf");

pub const CARET_DOWN: &str = "\u{f0d7}";
pub const CARET_RIGHT: &str = "\u{f0da}";
pub const CARET_LEFT: &str = "\u{f0d9}";
pub const ANGLE_RIGHT: &str = "\u{f105}";
pub const CHECK: &str = "\u{f00c}";
pub const CHECK_CIRCLE: &str = "\u{f058}";
pub const X_CIRCLE: &str = "\u{f057}";
pub const X: &str = "\u{f00d}";
pub const SPINNER: &str = "\u{f110}";
pub const FOLDER: &str = "\u{f07b}";
pub const FORK: &str = "\u{f126}";
pub const HOURGLASS: &str = "\u{f252}";
pub const STOP: &str = "\u{f04d}";
pub const SEND: &str = "\u{f1d8}";
pub const ARROW_RIGHT: &str = "\u{f061}";
pub const ARROW_UP: &str = "\u{f062}";
pub const ARROW_DOWN: &str = "\u{f063}";
pub const MODEL: &str = "\u{f2db}";
pub const FILTER: &str = "\u{f0b0}";
pub const PLUS: &str = "\u{f067}";
pub const GEAR: &str = "\u{f013}";
pub const TASKS: &str = "\u{f0ae}";
pub const SITEMAP: &str = "\u{f0e8}";
pub const WIN_MIN: &str = "\u{f2d1}";
pub const WIN_MAX: &str = "\u{f2d0}";
pub const WIN_CLOSE: &str = "\u{f2d3}";
pub const WIN_RESTORE: &str = "\u{f2d2}";
pub const SHIELD_SLASH: &str = "\u{f512}";

/// An icon: the glyph in the Nerd Font face with line-height 1 (the `.nf`
/// class). `size` is the surrounding text size it inherits in the prototype
/// (`text-xs` 12, `text-sm` 14).
pub fn icon(glyph: &str, size: f32) -> Text {
    text(glyph.to_owned())
        .font_family(NF.family)
        .text_size(px(size))
        .line_height(px(size))
        .flex_shrink_0()
}
