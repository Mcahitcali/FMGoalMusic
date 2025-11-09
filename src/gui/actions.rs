//! Keyboard actions for FM Goal Music
//!
//! This module defines all keyboard shortcuts using GPUI's action system.
//! Each action is a zero-sized type that can be dispatched through the focus chain.

use gpui::actions;

// Define all keyboard actions for the application
actions!(
    fm_goal_music,
    [
        // Primary monitoring control
        ToggleMonitoring,

        // Music preview control
        PreviewPlayPause,

        // Stop currently playing goal celebration music
        StopGoalMusic,

        // Navigation
        NextTab,
        PreviousTab,
        OpenHelp,

        // Region selection
        OpenRegionSelector,
        CapturePreview,

        // Music library
        AddMusicFile,
        RemoveMusicFile,

        // Volume control
        IncreaseVolume,
        DecreaseVolume,
        IncreaseAmbianceVolume,
        DecreaseAmbianceVolume,

        // Misc
        OpenSettings,
        CheckForUpdates,
    ]
);

/// Get the platform-specific modifier name for display
/// Returns "⌘" on macOS, "Ctrl" on other platforms
pub fn platform_modifier_symbol() -> &'static str {
    #[cfg(target_os = "macos")]
    {
        "⌘"
    }
    #[cfg(not(target_os = "macos"))]
    {
        "Ctrl"
    }
}

/// Get the platform-specific modifier name for display (full name)
/// Returns "Cmd" on macOS, "Ctrl" on other platforms
pub fn platform_modifier_name() -> &'static str {
    #[cfg(target_os = "macos")]
    {
        "Cmd"
    }
    #[cfg(not(target_os = "macos"))]
    {
        "Ctrl"
    }
}

/// Format a keybinding for display
/// Example: format_keybinding("1", true, false, false) -> "⌘1" on macOS, "Ctrl+1" on others
pub fn format_keybinding(key: &str, ctrl_cmd: bool, shift: bool, alt: bool) -> String {
    let mut parts = Vec::new();

    if ctrl_cmd {
        parts.push(platform_modifier_symbol().to_string());
    }

    if shift {
        parts.push("⇧".to_string());
    }

    if alt {
        #[cfg(target_os = "macos")]
        parts.push("⌥".to_string());
        #[cfg(not(target_os = "macos"))]
        parts.push("Alt".to_string());
    }

    parts.push(key.to_uppercase());

    // Use no separator on macOS (⌘1), use + on others (Ctrl+1)
    #[cfg(target_os = "macos")]
    {
        parts.join("")
    }
    #[cfg(not(target_os = "macos"))]
    {
        parts.join("+")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_keybinding() {
        // Test basic keybinding
        let result = format_keybinding("1", true, false, false);
        #[cfg(target_os = "macos")]
        assert_eq!(result, "⌘1");
        #[cfg(not(target_os = "macos"))]
        assert_eq!(result, "Ctrl+1");

        // Test with shift
        let result = format_keybinding("r", true, true, false);
        #[cfg(target_os = "macos")]
        assert_eq!(result, "⌘⇧R");
        #[cfg(not(target_os = "macos"))]
        assert_eq!(result, "Ctrl+⇧+R");
    }
}
