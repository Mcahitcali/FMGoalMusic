//! Hotkey configuration system for FM Goal Music
//!
//! This module manages keyboard shortcut bindings and provides:
//! - Default keybindings
//! - Persistent storage of user-customized shortcuts
//! - Conflict detection
//! - Platform-specific modifier handling

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Represents a single key combination
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Keybinding {
    /// The key (e.g., "1", "r", "space", "escape")
    pub key: String,

    /// Whether Ctrl (Windows/Linux) or Cmd (macOS) is pressed
    pub ctrl_cmd: bool,

    /// Whether Shift is pressed
    pub shift: bool,

    /// Whether Alt (Windows/Linux) or Option (macOS) is pressed
    pub alt: bool,
}

impl Keybinding {
    /// Create a new keybinding
    pub fn new(key: impl Into<String>, ctrl_cmd: bool, shift: bool, alt: bool) -> Self {
        Self {
            key: key.into(),
            ctrl_cmd,
            shift,
            alt,
        }
    }

    /// Create a keybinding with just Ctrl/Cmd modifier
    pub fn ctrl_cmd(key: impl Into<String>) -> Self {
        Self::new(key, true, false, false)
    }

    /// Create a keybinding with Ctrl/Cmd + Shift
    pub fn ctrl_cmd_shift(key: impl Into<String>) -> Self {
        Self::new(key, true, true, false)
    }

    /// Create a keybinding with no modifiers
    pub fn plain(key: impl Into<String>) -> Self {
        Self::new(key, false, false, false)
    }

    /// Format the keybinding for display
    pub fn format(&self) -> String {
        crate::gui::actions::format_keybinding(&self.key, self.ctrl_cmd, self.shift, self.alt)
    }

    /// Convert to GPUI keystroke string format
    /// Examples: "cmd-1", "shift-cmd-r", "space"
    pub fn to_keystroke(&self) -> String {
        let mut parts = Vec::new();

        if self.shift {
            parts.push("shift");
        }

        if self.alt {
            #[cfg(target_os = "macos")]
            parts.push("alt"); // GPUI uses "alt" even on macOS
            #[cfg(not(target_os = "macos"))]
            parts.push("alt");
        }

        if self.ctrl_cmd {
            #[cfg(target_os = "macos")]
            parts.push("cmd");
            #[cfg(not(target_os = "macos"))]
            parts.push("ctrl");
        }

        parts.push(&self.key);
        parts.join("-")
    }
}

/// Action identifier for keybinding mapping
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ActionId {
    ToggleMonitoring,
    PreviewPlayPause,
    StopGoalMusic,
    NextTab,
    PreviousTab,
    OpenHelp,
    OpenRegionSelector,
    CapturePreview,
    AddMusicFile,
    RemoveMusicFile,
    IncreaseVolume,
    DecreaseVolume,
    IncreaseAmbianceVolume,
    DecreaseAmbianceVolume,
    OpenSettings,
    CheckForUpdates,
    QuitApp,
}

impl ActionId {
    /// Get human-readable description of the action
    pub fn description(&self) -> &'static str {
        match self {
            ActionId::ToggleMonitoring => "Start/stop goal monitoring",
            ActionId::PreviewPlayPause => "Play/pause music preview",
            ActionId::StopGoalMusic => "Stop goal celebration music",
            ActionId::NextTab => "Navigate to next tab",
            ActionId::PreviousTab => "Navigate to previous tab",
            ActionId::OpenHelp => "Open help tab",
            ActionId::OpenRegionSelector => "Open region selector",
            ActionId::CapturePreview => "Capture preview screenshot",
            ActionId::AddMusicFile => "Add music file to library",
            ActionId::RemoveMusicFile => "Remove selected music",
            ActionId::IncreaseVolume => "Increase music volume",
            ActionId::DecreaseVolume => "Decrease music volume",
            ActionId::IncreaseAmbianceVolume => "Increase ambiance volume",
            ActionId::DecreaseAmbianceVolume => "Decrease ambiance volume",
            ActionId::OpenSettings => "Open settings",
            ActionId::CheckForUpdates => "Check for updates",
            ActionId::QuitApp => "Quit application",
        }
    }

    /// Get the category of this action
    pub fn category(&self) -> &'static str {
        match self {
            ActionId::ToggleMonitoring
            | ActionId::StopGoalMusic
            | ActionId::OpenRegionSelector
            | ActionId::CapturePreview => "Monitoring",
            ActionId::PreviewPlayPause | ActionId::AddMusicFile | ActionId::RemoveMusicFile => {
                "Music Library"
            }
            ActionId::IncreaseVolume
            | ActionId::DecreaseVolume
            | ActionId::IncreaseAmbianceVolume
            | ActionId::DecreaseAmbianceVolume => "Volume",
            ActionId::NextTab | ActionId::PreviousTab | ActionId::OpenHelp => "Navigation",
            ActionId::OpenSettings | ActionId::CheckForUpdates | ActionId::QuitApp => "Application",
        }
    }

    /// Returns all action IDs
    pub fn all() -> Vec<ActionId> {
        vec![
            ActionId::ToggleMonitoring,
            ActionId::PreviewPlayPause,
            ActionId::StopGoalMusic,
            ActionId::NextTab,
            ActionId::PreviousTab,
            ActionId::OpenHelp,
            ActionId::OpenRegionSelector,
            ActionId::CapturePreview,
            ActionId::AddMusicFile,
            ActionId::RemoveMusicFile,
            ActionId::IncreaseVolume,
            ActionId::DecreaseVolume,
            ActionId::IncreaseAmbianceVolume,
            ActionId::DecreaseAmbianceVolume,
            ActionId::OpenSettings,
            ActionId::CheckForUpdates,
            ActionId::QuitApp,
        ]
    }
}

/// Hotkey configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyConfig {
    /// Map of actions to their keybindings
    pub bindings: HashMap<ActionId, Keybinding>,
}

impl Default for HotkeyConfig {
    fn default() -> Self {
        let mut bindings = HashMap::new();

        // Primary actions (matching documented shortcuts)
        bindings.insert(ActionId::ToggleMonitoring, Keybinding::ctrl_cmd("1"));
        bindings.insert(
            ActionId::OpenRegionSelector,
            Keybinding::ctrl_cmd_shift("r"),
        );
        bindings.insert(ActionId::OpenHelp, Keybinding::ctrl_cmd("k"));

        // Music preview (Space bar, no modifiers)
        bindings.insert(ActionId::PreviewPlayPause, Keybinding::plain("space"));

        // Stop goal music (Ctrl/Cmd + S)
        bindings.insert(ActionId::StopGoalMusic, Keybinding::ctrl_cmd("s"));

        // Navigation
        bindings.insert(
            ActionId::NextTab,
            Keybinding::new("right", true, false, true), // Ctrl/Cmd + Alt + →
        );
        bindings.insert(
            ActionId::PreviousTab,
            Keybinding::new("left", true, false, true), // Ctrl/Cmd + Alt + ←
        );

        // Music library
        bindings.insert(ActionId::AddMusicFile, Keybinding::ctrl_cmd("o"));
        bindings.insert(ActionId::RemoveMusicFile, Keybinding::plain("delete"));

        // Volume control
        bindings.insert(ActionId::IncreaseVolume, Keybinding::ctrl_cmd("up"));
        bindings.insert(ActionId::DecreaseVolume, Keybinding::ctrl_cmd("down"));
        bindings.insert(
            ActionId::IncreaseAmbianceVolume,
            Keybinding::ctrl_cmd_shift("up"),
        );
        bindings.insert(
            ActionId::DecreaseAmbianceVolume,
            Keybinding::ctrl_cmd_shift("down"),
        );

        // Application
        bindings.insert(ActionId::OpenSettings, Keybinding::ctrl_cmd(","));
        bindings.insert(ActionId::CapturePreview, Keybinding::ctrl_cmd("p"));
        bindings.insert(ActionId::CheckForUpdates, Keybinding::ctrl_cmd("u"));
        bindings.insert(ActionId::QuitApp, Keybinding::ctrl_cmd("q"));

        Self { bindings }
    }
}

impl HotkeyConfig {
    /// Load hotkey configuration from disk, creating default if it doesn't exist
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = Self::config_path()?;

        if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            let loaded: HotkeyConfig = serde_json::from_str(&content)?;

            // Merge loaded bindings over defaults so new actions (like QuitApp)
            // automatically get default shortcuts if they were not present
            // when the config file was first created.
            let mut config = HotkeyConfig::default();
            for (action, binding) in loaded.bindings {
                config.bindings.insert(action, binding);
            }

            tracing::info!("✓ Loaded hotkey config from: {}", config_path.display());
            Ok(config)
        } else {
            // Create default config
            let config = HotkeyConfig::default();
            config.save()?;
            tracing::info!(
                "✓ Created default hotkey config at: {}",
                config_path.display()
            );
            Ok(config)
        }
    }

    /// Save hotkey configuration to disk
    pub fn save(&self) -> Result<(), Box<dyn std::error::Error>> {
        let config_path = Self::config_path()?;

        // Ensure parent directory exists
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string_pretty(self)?;
        fs::write(&config_path, json)?;

        Ok(())
    }

    /// Get the hotkey config file path
    pub fn config_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
        let base = dirs::config_dir().ok_or("Could not determine user config directory")?;
        let app_dir = base.join("FMGoalMusic");
        fs::create_dir_all(&app_dir)?;
        Ok(app_dir.join("hotkeys.json"))
    }

    /// Get keybinding for a specific action
    pub fn get(&self, action: ActionId) -> Option<&Keybinding> {
        self.bindings.get(&action)
    }

    /// Set keybinding for an action
    pub fn set(&mut self, action: ActionId, binding: Keybinding) {
        self.bindings.insert(action, binding);
    }

    /// Check if a keybinding conflicts with any existing binding
    pub fn has_conflict(
        &self,
        binding: &Keybinding,
        exclude: Option<ActionId>,
    ) -> Option<ActionId> {
        for (action, existing) in &self.bindings {
            if Some(*action) == exclude {
                continue;
            }
            if existing == binding {
                return Some(*action);
            }
        }
        None
    }

    /// Reset all keybindings to defaults
    pub fn reset_to_defaults(&mut self) {
        *self = Self::default();
    }

    /// Reset a specific keybinding to default
    pub fn reset_action(&mut self, action: ActionId) {
        if let Some(default_binding) = Self::default().bindings.get(&action) {
            self.bindings.insert(action, default_binding.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keybinding_format() {
        let binding = Keybinding::ctrl_cmd("1");
        let formatted = binding.format();
        #[cfg(target_os = "macos")]
        assert_eq!(formatted, "⌘1");
        #[cfg(not(target_os = "macos"))]
        assert_eq!(formatted, "Ctrl+1");
    }

    #[test]
    fn test_keybinding_to_keystroke() {
        let binding = Keybinding::ctrl_cmd_shift("r");
        let keystroke = binding.to_keystroke();
        #[cfg(target_os = "macos")]
        assert_eq!(keystroke, "shift-cmd-r");
        #[cfg(not(target_os = "macos"))]
        assert_eq!(keystroke, "shift-ctrl-r");
    }

    #[test]
    fn test_default_hotkeys() {
        let config = HotkeyConfig::default();
        assert_eq!(config.get(ActionId::ToggleMonitoring).unwrap().key, "1");
        assert!(config.get(ActionId::ToggleMonitoring).unwrap().ctrl_cmd);
    }

    #[test]
    fn test_conflict_detection() {
        let config = HotkeyConfig::default();
        let existing = Keybinding::ctrl_cmd("1");
        let conflict = config.has_conflict(&existing, None);
        assert_eq!(conflict, Some(ActionId::ToggleMonitoring));
    }

    #[test]
    fn test_action_categories() {
        assert_eq!(ActionId::ToggleMonitoring.category(), "Monitoring");
        assert_eq!(ActionId::PreviewPlayPause.category(), "Music Library");
        assert_eq!(ActionId::NextTab.category(), "Navigation");
    }
}
