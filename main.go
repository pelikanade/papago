package main

import (
	"log"

	"github.com/egoist/mygo"
	"github.com/egoist/mygo/ui"
)

// papago's window: a frameless, transparent native surface. The desktop shows
// through the glass layers (no wallpaper is drawn by the app), and the app
// draws its own chrome — the design's window controls live in the pane header.
func main() {
	a := newApp()
	mygo.App.WhenReady(func() {
		mygo.NewWindow(mygo.WindowOptions{
			Title:          "papago",
			Width:          1280,
			Height:         720,
			UseContentSize: true,
			StateKey:       "main",
			Frameless:      true,
			Transparent:    true,
			// Vibrancy blurs what is behind the window where the platform has
			// it (macOS, Windows 11 22H2); elsewhere the glass layers are
			// plain translucency over the desktop.
			Vibrancy: mygo.VibrancyWindow,
			Content:  ui.View(a.view),
		})
	})
	if err := mygo.App.Run(); err != nil {
		log.Fatal(err)
	}
}
