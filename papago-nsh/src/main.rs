//! Runs the Rust application directly on the machine's native backend.

#![cfg_attr(target_os = "windows", windows_subsystem = "windows")]

#[cfg(target_os = "linux")]
use nsh_platform_linux as platform;
#[cfg(target_os = "macos")]
use nsh_platform_macos as platform;
#[cfg(target_os = "windows")]
use nsh_platform_windows as platform;

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "windows"))]
fn main() -> impl std::process::Termination {
    env_logger::init();
    platform::run_app(
        platform::AppSource::Rust(
            // The Nerd Font the icons draw with, as bytes: a standalone
            // run has no asset location, so the registry's font file
            // cannot be read from one — the root carries it instead.
            platform::Root::of::<nsh_app_papago::App>().font(
                include_bytes!("../assets/fonts/FiraCodeNerdFontMono-Regular.ttf").as_slice().to_vec(),
            ),
        ),
        platform::AppOptions {
            title: "papago".to_owned(),
            #[cfg(any(target_os = "linux", target_os = "windows"))]
            app_id: "com.example.papago".to_owned(),
            // The prototype's window: 1280×720 content size (mygo
            // UseContentSize), frameless with the app's own chrome.
            size: (1280, 720),
            ..Default::default()
        },
    )
}

#[cfg(not(any(target_os = "linux", target_os = "macos", target_os = "windows")))]
compile_error!(
    "Run this binary on macOS, Windows or Linux; use the TSX entry with nsh run for iOS and Android."
);
