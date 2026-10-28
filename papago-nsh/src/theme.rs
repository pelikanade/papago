//! The base16 theme system, ported one-to-one from `prototype/src/themes.ts`
//! and `prototype/src/index.css`: every semantic color token derives from a
//! base16 palette, so the app can swap themes without touching components.
//! React applies the palette by rewriting CSS custom properties
//! (`applyTheme`); the NSH port resolves the palette of the current
//! `theme_id` signal wherever a color is read, which keeps the same
//! one-file restyling property.

use nsh_core::prelude::*;

use crate::store::store;
use crate::types::AgentStatus;

/// The themes, in `THEME_ORDER` (cycling order).
pub const THEME_ORDER: [&str; 3] = ["nord-dark", "gruvbox-dark", "onedark-dark"];

pub const DEFAULT_THEME: &str = "nord-dark";

/// The scheme label the settings popover and the palette row show.
pub fn theme_scheme(id: &str) -> &'static str {
    match id {
        "gruvbox-dark" => "Gruvbox Dark",
        "onedark-dark" => "One Dark",
        _ => "Nord Dark",
    }
}

/// The resolved tokens of a palette (`themeVars` + the `@theme` block).
/// Semantic mapping (dark schemes):
/// bg=base00 · panel=base01 · surface=base02 · line=base03 · mut=base04
/// ink=base05 · acc=base0C · ok=base0B · warn=base0A · err=base08 ·
/// work=base0D. Glass layers derive from the same slots with alpha, one
/// blur step per elevation (window → raised → panel → overlay).
#[derive(Clone, Copy)]
pub struct Palette {
    /// The scheme label (`theme_scheme` reads it by id instead).
    #[allow(dead_code)]
    pub scheme: &'static str,
    pub bg: Rgba,
    pub panel: Rgba,
    pub surface: Rgba,
    pub line: Rgba,
    pub muted: Rgba,
    pub ink: Rgba,
    pub acc: Rgba,
    pub work: Rgba,
    pub warn: Rgba,
    pub ok: Rgba,
    pub err: Rgba,
    /// base00 · window base over the wallpaper.
    pub glass0: Rgba,
    /// base04 tint · raised surface: cards, inputs.
    pub glass1: Rgba,
    /// base01 · chrome strips: sidebar, status bar.
    pub glass2: Rgba,
    /// base01 · floating overlays: popovers, palettes, hub.
    pub glass3: Rgba,
}

pub fn palette_for(id: &str) -> Palette {
    // (scheme, base00, base01, base02, base03, base04, base05, base08, base0A, base0B, base0C, base0D)
    let (scheme, b00, b01, b02, b03, b04, b05, b08, b0a, b0b, b0c, b0d) = match id {
        "gruvbox-dark" => (
            "Gruvbox Dark",
            0x282828,
            0x3C3836,
            0x504945,
            0x665C54,
            0xBDAE93,
            0xD5C4A1,
            0xFB4934,
            0xFABD2F,
            0xB8BB26,
            0x8EC07C,
            0x83A598,
        ),
        "onedark-dark" => (
            "One Dark",
            0x282C34,
            0x353B45,
            0x3E4451,
            0x545862,
            0x565C64,
            0xABB2BF,
            0xE06C75,
            0xE5C07B,
            0x98C379,
            0x56B6C2,
            0x61AFEF,
        ),
        _ => (
            "Nord Dark",
            0x2E3440,
            0x3B4252,
            0x434C5E,
            0x4C566A,
            0xD8DEE9,
            0xE5E9F0,
            0xBF616A,
            0xEBCB8B,
            0xA3BE8C,
            0x88C0D0,
            0x81A1C1,
        ),
    };
    Palette {
        scheme,
        bg: rgb(b00),
        panel: rgb(b01),
        surface: rgb(b02),
        line: rgb(b03),
        muted: rgb(b04),
        ink: rgb(b05),
        err: rgb(b08),
        warn: rgb(b0a),
        ok: rgb(b0b),
        acc: rgb(b0c),
        work: rgb(b0d),
        glass0: rgb(b00).opacity(0.52),
        glass1: rgb(b04).opacity(0.05),
        glass2: rgb(b01).opacity(0.62),
        glass3: rgb(b01).opacity(0.88),
    }
}

/// The palette of the current theme, read reactively: a binding such as
/// `.bg(move || pal().glass1)` follows `cycleTheme` as CSS variables did.
/// Call it inside a closure; outside one it resolves the current value only.
pub fn pal() -> Palette {
    palette_for(store().theme_id.get())
}

/// STATUS_DOT (data.ts): the filled dot's color per agent status.
pub fn dot_color(p: &Palette, status: AgentStatus) -> Rgba {
    match status {
        AgentStatus::Running => p.work,
        AgentStatus::NeedsYou => p.warn,
        AgentStatus::Done => p.ok,
        AgentStatus::Idle => p.muted,
        AgentStatus::Error => p.err,
    }
}

/// STATUS_RING (data.ts): the same status colours in hollow-ring form, so an
/// orchestrator row never reads as a sibling of a worker row at a glance.
pub fn ring_color(p: &Palette, status: AgentStatus) -> Rgba {
    dot_color(p, status)
}
