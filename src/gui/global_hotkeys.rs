//! Global hotkey system for FM Goal Music
//!
//! Provides system-wide keyboard shortcuts that work even when the app is not focused.
//! This is essential for controlling goal music while playing games in fullscreen.

use anyhow::{anyhow, Result};
use global_hotkey::{
    hotkey::{Code, HotKey, Modifiers},
    GlobalHotKeyEvent, GlobalHotKeyManager,
};

use super::controller::GuiController;

/// Global hotkey IDs
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GlobalHotkeyId {
    ToggleMonitoring = 1,
    StopGoalMusic = 2,
}

/// Manages system-wide keyboard shortcuts
pub struct GlobalHotkeySystem {
    manager: GlobalHotKeyManager,
    controller: GuiController,
    hotkeys: Vec<HotKey>,
}

/// Send-safe handler that processes global hotkey events on a background thread.
///
/// This intentionally does NOT contain the platform-specific GlobalHotKeyManager,
/// which is not Send on Windows. Instead, it only keeps the IDs of the registered
/// hotkeys and a clone of the GUI controller.
#[derive(Clone)]
pub struct GlobalHotkeyHandler {
    controller: GuiController,
    toggle_id: u32,
    stop_id: u32,
}

impl GlobalHotkeySystem {
    /// Create a new global hotkey system
    ///
    /// Uses more unique key combinations to avoid conflicts:
    /// - Cmd/Ctrl + Shift + 1: Toggle monitoring
    /// - Cmd/Ctrl + Shift + S: Stop goal music
    pub fn new(controller: GuiController) -> Result<Self> {
        let manager = GlobalHotKeyManager::new()
            .map_err(|e| anyhow!("Failed to create global hotkey manager: {}", e))?;

        let mut system = Self {
            manager,
            controller,
            hotkeys: Vec::new(),
        };

        // Register the two critical global hotkeys
        system.register_hotkeys()?;

        Ok(system)
    }

    /// Create a lightweight handler that can be sent across threads safely.
    pub fn create_handler(&self) -> GlobalHotkeyHandler {
        let toggle_id = self.hotkeys.get(0).map(|h| h.id()).unwrap_or(0);
        let stop_id = self.hotkeys.get(1).map(|h| h.id()).unwrap_or(0);

        GlobalHotkeyHandler {
            controller: self.controller.clone(),
            toggle_id,
            stop_id,
        }
    }

    /// Register global hotkeys with the system
    fn register_hotkeys(&mut self) -> Result<()> {
        // Platform-specific modifier (Cmd on macOS, Ctrl on Windows/Linux)
        #[cfg(target_os = "macos")]
        let platform_modifier = Modifiers::SUPER; // Cmd key
        #[cfg(not(target_os = "macos"))]
        let platform_modifier = Modifiers::CONTROL;

        // Hotkey 1: Cmd/Ctrl + Shift + 1 - Toggle Monitoring
        let toggle_monitoring =
            HotKey::new(Some(platform_modifier | Modifiers::SHIFT), Code::Digit1);

        // Hotkey 2: Cmd/Ctrl + Shift + S - Stop Goal Music
        let stop_music = HotKey::new(Some(platform_modifier | Modifiers::SHIFT), Code::KeyS);

        // Register with the system
        self.manager
            .register(toggle_monitoring)
            .map_err(|e| anyhow!("Failed to register toggle monitoring hotkey: {}", e))?;

        self.manager
            .register(stop_music)
            .map_err(|e| anyhow!("Failed to register stop music hotkey: {}", e))?;

        self.hotkeys.push(toggle_monitoring);
        self.hotkeys.push(stop_music);

        tracing::info!("✓ Global hotkeys registered:");
        #[cfg(target_os = "macos")]
        {
            tracing::info!("  ⌘⇧1 - Toggle goal monitoring");
            tracing::info!("  ⌘⇧S - Stop goal music");
        }
        #[cfg(not(target_os = "macos"))]
        {
            tracing::info!("  Ctrl+Shift+1 - Toggle goal monitoring");
            tracing::info!("  Ctrl+Shift+S - Stop goal music");
        }

        Ok(())
    }

    /// Get human-readable descriptions of registered hotkeys
    pub fn get_hotkey_descriptions() -> Vec<(&'static str, &'static str)> {
        #[cfg(target_os = "macos")]
        {
            vec![
                ("⌘⇧1", "Toggle goal monitoring (works globally)"),
                ("⌘⇧S", "Stop goal music (works globally)"),
            ]
        }
        #[cfg(not(target_os = "macos"))]
        {
            vec![
                ("Ctrl+Shift+1", "Toggle goal monitoring (works globally)"),
                ("Ctrl+Shift+S", "Stop goal music (works globally)"),
            ]
        }
    }
}

impl Drop for GlobalHotkeySystem {
    fn drop(&mut self) {
        // Unregister all hotkeys on drop
        for hotkey in &self.hotkeys {
            if let Err(e) = self.manager.unregister(*hotkey) {
                tracing::error!("Failed to unregister hotkey: {}", e);
            }
        }
        tracing::info!("Global hotkeys unregistered");
    }
}

impl GlobalHotkeyHandler {
    /// Process a global hotkey event using stored IDs and controller.
    pub fn handle_event(&self, event: GlobalHotKeyEvent) {
        let hotkey_id = if event.id == self.toggle_id {
            Some(GlobalHotkeyId::ToggleMonitoring)
        } else if event.id == self.stop_id {
            Some(GlobalHotkeyId::StopGoalMusic)
        } else {
            None
        };

        let hotkey_id = match hotkey_id {
            Some(id) => id,
            None => {
                tracing::warn!("Unknown global hotkey event: {:?}", event.id);
                return;
            }
        };

        tracing::debug!("Global hotkey triggered: {:?}", hotkey_id);

        match hotkey_id {
            GlobalHotkeyId::ToggleMonitoring => self.handle_toggle_monitoring(),
            GlobalHotkeyId::StopGoalMusic => self.handle_stop_goal_music(),
        }
    }

    fn handle_toggle_monitoring(&self) {
        use crate::state::ProcessState;

        let state = self.controller.state();
        let is_running = matches!(state.lock().process_state, ProcessState::Running { .. });

        if is_running {
            if let Err(err) = self.controller.stop_monitoring() {
                tracing::error!("Failed to stop monitoring via global hotkey: {}", err);
            } else {
                tracing::info!("✓ Monitoring stopped via global hotkey");
            }
        } else {
            if let Err(err) = self.controller.start_monitoring() {
                tracing::error!("Failed to start monitoring via global hotkey: {}", err);
            } else {
                tracing::info!("✓ Monitoring started via global hotkey");
            }
        }
    }

    fn handle_stop_goal_music(&self) {
        if let Err(err) = self.controller.stop_goal_music() {
            tracing::error!("Failed to stop goal music via global hotkey: {}", err);
        } else {
            tracing::info!("✓ Goal music stopped via global hotkey");
        }
    }
}

/// Start listening for global hotkey events in a background thread.
///
/// Only the Send-safe GlobalHotkeyHandler is moved into the thread, avoiding
/// sending the non-Send GlobalHotKeyManager across threads on Windows.
pub fn start_global_hotkey_listener(handler: GlobalHotkeyHandler) -> Result<()> {
    let receiver = GlobalHotKeyEvent::receiver();

    std::thread::spawn(move || loop {
        if let Ok(event) = receiver.recv() {
            handler.handle_event(event);
        }
    });

    tracing::info!("✓ Global hotkey listener started");
    Ok(())
}
