//! Tailwind's spacing and type scale, as the prototype's design uses it
//! (`prototype/src/index.css` + the mygo port's `tokens.go`). The React
//! prototype is written in Tailwind classes; this file gives the NSH port the
//! same numbers so a class like `gap-2` or `text-sm` reads the same on both
//! sides. Values are CSS pixels == NSH points.

use nsh_core::prelude::*;

/// Spacing scale (1 unit = 4px): `gap-1`, `p-1` …
pub const SP1: f32 = 4.0;
pub const SP15: f32 = 6.0;
pub const SP2: f32 = 8.0;
pub const SP25: f32 = 10.0;
pub const SP3: f32 = 12.0;
pub const SP35: f32 = 14.0;
pub const SP4: f32 = 16.0;
pub const SP6: f32 = 24.0;
pub const SP10: f32 = 40.0;
pub const SP14: f32 = 56.0;

/// Type scale (font size / line height).
pub const TEXT_XS: f32 = 12.0;
pub const TEXT_XS_LINE: f32 = 16.0;
pub const TEXT_SM: f32 = 14.0;
pub const TEXT_SM_LINE: f32 = 20.0;
pub const TEXT_BASE: f32 = 16.0;
/// text-base line height (unused by the current design).
#[allow(dead_code)]
pub const TEXT_BASE_LINE: f32 = 24.0;

/// Radii: `rounded`, `rounded-md`, `rounded-lg`, `rounded-full`.
pub const RADIUS_SM: f32 = 4.0;
pub const RADIUS_MD: f32 = 6.0;
pub const RADIUS_LG: f32 = 8.0;
pub const RADIUS_FULL: f32 = 9999.0;

/// Control heights & strip heights.
/// h-5, chips and badges (unused by the current design).
#[allow(dead_code)]
pub const H5: f32 = 20.0;
pub const H6: f32 = 24.0;
pub const H7: f32 = 28.0;
pub const H8: f32 = 32.0;
pub const H9: f32 = 36.0;
pub const H10: f32 = 40.0;

/// Fixed widths.
pub const W6: f32 = 24.0;
pub const W8: f32 = 32.0;
pub const W72: f32 = 288.0;
pub const DOT: f32 = 10.0;

/// Hairlines.
pub const HAIRLINE: f32 = 1.0;

/// Alpha tints the prototype writes as Tailwind opacity modifiers over white
/// (`bg-white/[0.x]`) and `border-white/10`.
pub const ALPHA_BORDER: f32 = 0.10;
pub const ALPHA_HOVER: f32 = 0.06;
pub const ALPHA_HOVER_ALT: f32 = 0.07;
pub const ALPHA_SELECTED: f32 = 0.08;

/// Backdrop blur radii, one blur step per glass elevation
/// (`backdrop-blur-sm … 3xl`).
pub const BLUR_SM: f32 = 4.0;
pub const BLUR_MD: f32 = 12.0;
pub const BLUR_XL: f32 = 24.0;
pub const BLUR_2XL: f32 = 40.0;
pub const BLUR_3XL: f32 = 64.0;

/// rem units the prototype writes for panels: popovers 15–20rem, the Hub
/// w-130, the palette w-160 and its max-h-105.
pub fn rem(n: f32) -> f32 {
    n * 16.0
}

/// The body's backdrop color behind the glass window (`index.css`: the
/// wallpaper the translucency samples; opaque here because NSH windows on
/// GTK draw over the desktop without vibrancy).
pub const BODY: u32 = 0x0b0d12;

/// Tailwind's `shadow-2xl` for the window frame and the floating overlays.
pub const SHADOW_2XL_BLUR: f32 = 50.0;
pub const SHADOW_2XL_Y: f32 = 25.0;
pub const SHADOW_2XL_SPREAD: f32 = -12.0;

/// `bg-white/[0.x]`: white with alpha, as an `Rgba` matching the palette.
pub fn white_a(alpha: f32) -> Rgba {
    rgb(0xffffff).opacity(alpha)
}

/// `bg-transparent`, as an `Rgba` matching the palette.
pub fn clear() -> Rgba {
    rgb(0).opacity(0.0)
}
