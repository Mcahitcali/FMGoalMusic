/// Command types for the application
///
/// Commands represent requests to perform actions (imperative).
/// They are executed by the command executor.
use std::path::PathBuf;

use crate::config::SelectedTeam;

/// Application commands
#[derive(Debug, Clone)]
pub enum Command {
    /// Start detection with the specified settings
    StartDetection {
        music_path: PathBuf,
        music_name: String,
        team: Option<SelectedTeam>,
    },

    /// Stop detection
    StopDetection,

    /// Play audio from the specified source
    PlayAudio {
        source: AudioSourceType,
        volume: f32,
    },

    /// Stop all audio playback
    StopAudio,

    /// Save configuration to disk
    SaveConfig,

    /// Load configuration from disk
    LoadConfig,

    /// Select a music file
    SelectMusic { path: PathBuf, name: String },

    /// Select a team
    SelectTeam { team: SelectedTeam },

    /// Change capture region
    ChangeRegion { region: [u32; 4] },

    /// Update a configuration value
    UpdateConfig { update: ConfigUpdate },

    /// Check for application updates
    CheckForUpdates,

    /// Quit the application
    Quit,
}

/// Audio source type for playback
#[derive(Debug, Clone)]
pub enum AudioSourceType {
    /// Play the selected goal music
    GoalMusic { path: PathBuf },

    /// Play goal ambiance
    Ambiance { path: PathBuf },

    /// Play crowd cheer (for v0.3)
    CrowdCheer {
        path: PathBuf,
        variant: CrowdCheerVariant,
    },
}

/// Crowd cheer variants based on result (for v0.3)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrowdCheerVariant {
    Win,
    Draw,
    Loss,
}

/// Configuration update types
#[derive(Debug, Clone)]
pub enum ConfigUpdate {
    MusicVolume(f32),
    AmbianceVolume(f32),
    OcrThreshold(u8),
    DebounceMs(u64),
    EnableMorphOpen(bool),
    MusicLength(u64),
    AmbianceLength(u64),
    AmbianceEnabled(bool),
    AutoCheckUpdates(bool),
    SelectedMonitor(usize),
}

/// Result of command execution
#[derive(Debug)]
pub enum CommandResult {
    /// Command executed successfully
    Success,

    /// Command executed with a specific result
    SuccessWithValue(String),

    /// Command failed with an error
    Error(String),
}

impl Command {
    /// Get a human-readable description of the command
    pub fn description(&self) -> String {
        match self {
            Command::StartDetection {
                music_name, team, ..
            } => {
                if let Some(team) = team {
                    format!("Start detection: {} ({})", music_name, team.display_name)
                } else {
                    format!("Start detection: {}", music_name)
                }
            }
            Command::StopDetection => "Stop detection".to_string(),
            Command::PlayAudio { source, .. } => {
                format!("Play audio: {:?}", source)
            }
            Command::StopAudio => "Stop audio".to_string(),
            Command::SaveConfig => "Save configuration".to_string(),
            Command::LoadConfig => "Load configuration".to_string(),
            Command::SelectMusic { name, .. } => {
                format!("Select music: {}", name)
            }
            Command::SelectTeam { team } => {
                format!("Select team: {}", team.display_name)
            }
            Command::ChangeRegion { region } => {
                format!("Change region: {:?}", region)
            }
            Command::UpdateConfig { update } => {
                format!("Update config: {:?}", update)
            }
            Command::CheckForUpdates => "Check for updates".to_string(),
            Command::Quit => "Quit application".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_description() {
        let cmd = Command::StopDetection;
        assert_eq!(cmd.description(), "Stop detection");

        let cmd = Command::SaveConfig;
        assert_eq!(cmd.description(), "Save configuration");
    }
}
