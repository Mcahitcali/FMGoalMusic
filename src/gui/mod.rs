mod actions;
mod controller;
mod global_hotkeys;
mod hotkeys;
mod state;
mod theme;
mod view;

use actions::*;
use controller::GuiController;
use global_hotkeys::{start_global_hotkey_listener, GlobalHotkeySystem};
use gpui::{
    px, size, App, AppContext, Application, Bounds, KeyBinding, WindowBounds, WindowOptions,
};
use hotkeys::{ActionId, HotkeyConfig};
use parking_lot::Mutex;
use std::sync::Arc;
use view::MainView;

/// Register keyboard shortcuts from the hotkey configuration
fn register_keybindings(cx: &mut App) {
    let config = HotkeyConfig::load().unwrap_or_default();

    // Register each keybinding from the config
    for (action_id, keybinding) in &config.bindings {
        let keystroke = keybinding.to_keystroke();

        match action_id {
            ActionId::ToggleMonitoring => {
                cx.bind_keys([KeyBinding::new(
                    keystroke.as_str(),
                    ToggleMonitoring,
                    Some("main_view"),
                )]);
            }
            ActionId::PreviewPlayPause => {
                cx.bind_keys([KeyBinding::new(
                    keystroke.as_str(),
                    PreviewPlayPause,
                    Some("main_view"),
                )]);
            }
            ActionId::StopGoalMusic => {
                cx.bind_keys([KeyBinding::new(
                    keystroke.as_str(),
                    StopGoalMusic,
                    Some("main_view"),
                )]);
            }
            ActionId::NextTab => {
                cx.bind_keys([KeyBinding::new(
                    keystroke.as_str(),
                    NextTab,
                    Some("main_view"),
                )]);
            }
            ActionId::PreviousTab => {
                cx.bind_keys([KeyBinding::new(
                    keystroke.as_str(),
                    PreviousTab,
                    Some("main_view"),
                )]);
            }
            ActionId::OpenHelp => {
                cx.bind_keys([KeyBinding::new(
                    keystroke.as_str(),
                    OpenHelp,
                    Some("main_view"),
                )]);
            }
            ActionId::OpenRegionSelector => {
                cx.bind_keys([KeyBinding::new(
                    keystroke.as_str(),
                    OpenRegionSelector,
                    Some("main_view"),
                )]);
            }
            ActionId::CapturePreview => {
                cx.bind_keys([KeyBinding::new(
                    keystroke.as_str(),
                    CapturePreview,
                    Some("main_view"),
                )]);
            }
            ActionId::AddMusicFile => {
                cx.bind_keys([KeyBinding::new(
                    keystroke.as_str(),
                    AddMusicFile,
                    Some("main_view"),
                )]);
            }
            ActionId::RemoveMusicFile => {
                cx.bind_keys([KeyBinding::new(
                    keystroke.as_str(),
                    RemoveMusicFile,
                    Some("main_view"),
                )]);
            }
            ActionId::IncreaseVolume => {
                cx.bind_keys([KeyBinding::new(
                    keystroke.as_str(),
                    IncreaseVolume,
                    Some("main_view"),
                )]);
            }
            ActionId::DecreaseVolume => {
                cx.bind_keys([KeyBinding::new(
                    keystroke.as_str(),
                    DecreaseVolume,
                    Some("main_view"),
                )]);
            }
            ActionId::IncreaseAmbianceVolume => {
                cx.bind_keys([KeyBinding::new(
                    keystroke.as_str(),
                    IncreaseAmbianceVolume,
                    Some("main_view"),
                )]);
            }
            ActionId::DecreaseAmbianceVolume => {
                cx.bind_keys([KeyBinding::new(
                    keystroke.as_str(),
                    DecreaseAmbianceVolume,
                    Some("main_view"),
                )]);
            }
            ActionId::OpenSettings => {
                cx.bind_keys([KeyBinding::new(
                    keystroke.as_str(),
                    OpenSettings,
                    Some("main_view"),
                )]);
            }
            ActionId::CheckForUpdates => {
                cx.bind_keys([KeyBinding::new(
                    keystroke.as_str(),
                    CheckForUpdates,
                    Some("main_view"),
                )]);
            }
        }
    }
}

pub fn run() -> anyhow::Result<()> {
    let controller = GuiController::new()?;

    // Initialize global hotkeys (work system-wide, even when app is not focused)
    let global_hotkeys = GlobalHotkeySystem::new(controller.clone())?;
    let global_hotkeys = Arc::new(Mutex::new(global_hotkeys));
    start_global_hotkey_listener(global_hotkeys.clone())?;

    let application = Application::new();

    application.run(move |cx: &mut App| {
        gpui_component::init(cx);
        theme::install(cx);

        // Register keyboard shortcuts (only work when app is focused)
        register_keybindings(cx);

        let bounds = Bounds::centered(None, size(px(1180.0), px(760.0)), cx);
        let controller = controller.clone();

        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                window_min_size: Some(size(px(1000.0), px(700.0))),
                ..Default::default()
            },
            move |window, cx| {
                let controller = controller.clone();
                let view = cx.new(|cx| MainView::new(window, cx, controller.clone()));
                cx.new(|cx| gpui_component::Root::new(view.into(), window, cx))
            },
        )
        .expect("failed to open GPUI window");

        cx.activate(true);
    });

    Ok(())
}
