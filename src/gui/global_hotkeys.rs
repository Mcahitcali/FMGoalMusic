//! Global hotkey system for FM Goal Music
//!
//! Provides system-wide keyboard shortcuts that work even when the app is not focused.
//! This is essential for controlling goal music while playing games in fullscreen.

use anyhow::{anyhow, Result};
use global_hotkey::{
    hotkey::{Code, HotKey, Modifiers},
    GlobalHotKeyEvent, GlobalHotKeyManager,
};
use parking_lot::Mutex;
use std::sync::Arc;

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

    /// Process a global hotkey event
    pub fn handle_event(&self, event: GlobalHotKeyEvent) {
        // Map the hotkey to its ID
        let hotkey_id = match self.hotkeys.iter().position(|h| h.id() == event.id) {
            Some(0) => GlobalHotkeyId::ToggleMonitoring,
            Some(1) => GlobalHotkeyId::StopGoalMusic,
            _ => {
                tracing::warn!("Unknown global hotkey event: {:?}", event.id);
                return;
            }
        };

        tracing::debug!("Global hotkey triggered: {:?}", hotkey_id);

        // Execute the corresponding action
        match hotkey_id {
            GlobalHotkeyId::ToggleMonitoring => {
                self.handle_toggle_monitoring();
            }
            GlobalHotkeyId::StopGoalMusic => {
                self.handle_stop_goal_music();
            }
        }
    }

    /// Handle toggle monitoring hotkey
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

    /// Handle stop goal music hotkey
    fn handle_stop_goal_music(&self) {
        if let Err(err) = self.controller.stop_goal_music() {
            tracing::error!("Failed to stop goal music via global hotkey: {}", err);
        } else {
            tracing::info!("✓ Goal music stopped via global hotkey");
        }
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

/// Start listening for global hotkey events in a background thread
pub fn start_global_hotkey_listener(system: Arc<Mutex<GlobalHotkeySystem>>) -> Result<()> {
    let receiver = GlobalHotKeyEvent::receiver();

    std::thread::spawn(move || loop {
        if let Ok(event) = receiver.recv() {
            let system = system.lock();
            system.handle_event(event);
        }
    });

    tracing::info!("✓ Global hotkey listener started");
    Ok(())
}
