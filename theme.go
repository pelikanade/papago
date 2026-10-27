package main

import "github.com/egoist/mygo/ui"

// Theme layer: the base16 palettes and semantic tokens the prototype's Tailwind
// @theme block defines (prototype/src/index.css · prototype/src/themes.ts).
// The Go app paints the same values, so the port stays 1:1 with the design.
//
// Semantic mapping (dark schemes):
//
//	bg=base00 · panel=base01 · surface=base02 · line=base03 · mut=base04
//	ink=base05 · acc=base0C · work=base0D · ok=base0B · warn=base0A · err=base08
//
// Glass layers derive from the same slots (base00/base01/base04 with alpha).

// RGBA is a color as the design writes it: 0..1 components, straight alpha.
type RGBA struct {
	R, G, B, A float64
}

// Base16 slot indices.
const (
	Base00 = iota
	Base01
	Base02
	Base03
	Base04
	Base05
	Base06
	Base07
	Base08
	Base09
	Base0A
	Base0B
	Base0C
	Base0D
	Base0E
	Base0F
)

// Palette is one base16 scheme: sixteen #RRGGBB slots.
type Palette struct {
	Scheme string
	Slots  [16]string
}

func (p Palette) slot(i int) RGBA { return hexRGB(p.Slots[i]) }

// Surface opacities follow the desktop's own configuration (~/Projects/flake):
// stylix's opacity.terminal = 0.80 and noctalia's background_opacity = 0.80 for
// surfaces, 0.88 for raised cards, 1.0 for controls. The window surface sits at
// the app/terminal level, chrome strips one step up, overlays just under the
// controls' level — the prototype's web-page alphas (0.52/0.62/0.88) read as
// fog next to the rest of this desktop.
const (
	alphaSurface = 0.80 // stylix opacity.terminal · noctalia background_opacity
	alphaChrome  = 0.88 // noctalia card background_opacity
	alphaOverlay = 0.96 // one step under noctalia input_opacity = 1.0
)

// Theme is the semantic token set for one palette.
type Theme struct {
	Bg      RGBA // base00
	Panel   RGBA // base01
	Surface RGBA // base02
	Line    RGBA // base03
	Mut     RGBA // base04
	Ink     RGBA // base05
	Acc     RGBA // base0C
	Work    RGBA // base0D — working (blue)
	Ok      RGBA // base0B
	Warn    RGBA // base0A
	Err     RGBA // base08

	// Glass layers — one blur step per elevation (window → raised → panel → overlay).
	Glass0 RGBA // base00 · window base over the wallpaper
	Glass1 RGBA // base04 tint · raised surface: cards, inputs
	Glass2 RGBA // base01 · chrome strips: sidebar, status bar
	Glass3 RGBA // base01 · floating overlays: popovers, palettes, hub
}

// Palettes mirror THEMES in prototype/src/themes.ts, slot for slot.
var Palettes = map[string]Palette{
	"nord-dark": {
		Scheme: "Nord Dark",
		Slots: [16]string{
			"#2E3440", "#3B4252", "#434C5E", "#4C566A",
			"#D8DEE9", "#E5E9F0", "#ECEFF4", "#8FBCBB",
			"#BF616A", "#D08770", "#EBCB8B", "#A3BE8C",
			"#88C0D0", "#81A1C1", "#B48EAD", "#5E81AC",
		},
	},
	"gruvbox-dark": {
		Scheme: "Gruvbox Dark",
		Slots: [16]string{
			"#282828", "#3C3836", "#504945", "#665C54",
			"#BDAE93", "#D5C4A1", "#EBDBB2", "#FBF1C7",
			"#FB4934", "#FE8019", "#FABD2F", "#B8BB26",
			"#8EC07C", "#83A598", "#D3869B", "#D65D0E",
		},
	},
	"onedark-dark": {
		Scheme: "One Dark",
		Slots: [16]string{
			"#282C34", "#353B45", "#3E4451", "#545862",
			"#565C64", "#ABB2BF", "#B6BDCA", "#C8CCD4",
			"#E06C75", "#D19A66", "#E5C07B", "#98C379",
			"#56B6C2", "#61AFEF", "#C678DD", "#BE5046",
		},
	},
}

const defaultTheme = "nord-dark"

// themeOrder is the cycle order of the theme switch (THEME_ORDER in themes.ts).
var themeOrder = []string{"nord-dark", "gruvbox-dark", "onedark-dark"}

// themeFor returns the semantic tokens of a palette, defaulting to nord-dark.
func themeFor(name string) Theme {
	p, ok := Palettes[name]
	if !ok {
		p = Palettes[defaultTheme]
	}
	return Theme{
		Bg:      p.slot(Base00),
		Panel:   p.slot(Base01),
		Surface: p.slot(Base02),
		Line:    p.slot(Base03),
		Mut:     p.slot(Base04),
		Ink:     p.slot(Base05),
		Acc:     p.slot(Base0C),
		Work:    p.slot(Base0D),
		Ok:      p.slot(Base0B),
		Warn:    p.slot(Base0A),
		Err:     p.slot(Base08),

		Glass0: p.slot(Base00).withAlpha(alphaSurface),
		Glass1: p.slot(Base04).withAlpha(0.05),
		Glass2: p.slot(Base01).withAlpha(alphaChrome),
		Glass3: p.slot(Base01).withAlpha(alphaOverlay),
	}
}

// hexRGB parses #RRGGBB (the only form the palettes use).
func hexRGB(hex string) RGBA {
	v := hex
	if len(v) > 0 && v[0] == '#' {
		v = v[1:]
	}
	if len(v) != 6 {
		return RGBA{}
	}
	return RGBA{
		R: float64(hexPair(v[0:2])) / 255,
		G: float64(hexPair(v[2:4])) / 255,
		B: float64(hexPair(v[4:6])) / 255,
		A: 1,
	}
}

func hexPair(s string) int {
	n := 0
	for i := range len(s) {
		n *= 16
		switch c := s[i]; {
		case c >= '0' && c <= '9':
			n += int(c - '0')
		case c >= 'a' && c <= 'f':
			n += int(c-'a') + 10
		case c >= 'A' && c <= 'F':
			n += int(c-'A') + 10
		}
	}
	return n
}

func (c RGBA) withAlpha(a float64) RGBA {
	c.A = a
	return c
}

// mix lays this color over an opaque backdrop, as the browser composites a
// translucent fill: out = a*c + (1-a)*back.
func (c RGBA) mix(back RGBA) RGBA {
	a := c.A
	return RGBA{
		R: c.R*a + back.R*(1-a),
		G: c.G*a + back.G*(1-a),
		B: c.B*a + back.B*(1-a),
		A: 1,
	}
}

// white/black overlays the prototype uses as Tailwind's bg-white/[0.03] and
// friends: white with the given alpha, laid over whatever is beneath.
func white(alpha float64) RGBA { return RGBA{1, 1, 1, alpha} }

// col converts a design color to the toolkit's color.
func col(c RGBA) ui.Color {
	return ui.RGBA(
		uint8(c.R*255+0.5),
		uint8(c.G*255+0.5),
		uint8(c.B*255+0.5),
		float32(c.A),
	)
}

// whiteA is Tailwind's `white/<alpha>`: the border and hover overlays.
func whiteA(alpha float64) ui.Color { return col(white(alpha)) }

// uiTheme maps the design tokens onto MyGo's own theme, so the toolkit's
// widgets — the composer's text area, scrollbars, popovers, tooltips — take
// the same look as the hand-built surfaces around them.
//
// The window is transparent and frameless (a native app, not a page): nothing
// paints a background of its own, the desktop shows through the glass layers.
func uiTheme(t Theme) *ui.Theme {
	base := ui.DarkTheme()
	base.Background = ui.Transparent
	base.Surface = col(t.Panel)
	base.SurfaceHover = whiteA(alphaHover)
	base.SurfacePressed = whiteA(alphaSelected)
	base.Border = whiteA(alphaBorder)
	base.Text = col(t.Ink)
	base.TextMuted = col(t.Mut)
	base.Accent = col(t.Acc)
	base.AccentHover = col(t.Acc)
	base.AccentPressed = col(t.Acc)
	base.AccentText = col(t.Bg)
	base.Danger = col(t.Err)
	base.Warning = col(t.Warn)
	base.Success = col(t.Ok)
	base.Selection = col(t.Acc).Alpha(0.30)
	base.Focus = col(t.Acc)
	base.Scrollbar = col(t.Line)
	base.ScrollbarWidth = 8 // ::-webkit-scrollbar width in the prototype
	base.Radius = float32(radiusLG)
	base.Spacing = 4 // Tailwind's spacing unit
	base.FontSize = textSM
	base.Font = fontSans + ", " + fontIcons
	return base
}
