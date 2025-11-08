/// Event types for the application
///
/// Events represent things that have happened (past tense).
/// They are broadcast to all subscribers.
use std::path::PathBuf;
use std::time::Instant;

use crate::config::SelectedTeam;
use crate::state::ProcessState;

/// Application events
#[derive(Debug, Clone)]
pub enum Event {
    /// A goal was detected
    GoalDetected {
        team: Option<SelectedTeam>,
        timestamp: Instant,
    },

    /// Match has started (kickoff detected) - for v0.2
    MatchStarted { timestamp: Instant },

    /// Match has ended - for v0.3
    MatchEnded {
        timestamp: Instant,
        home_score: u32,
        away_score: u32,
    },

    /// Detection process state changed
    ProcessStateChanged {
        old_state: ProcessState,
        new_state: ProcessState,
    },

    /// Configuration was changed
    ConfigChanged { field: ConfigField },

    /// Music file was selected
    MusicSelected { path: PathBuf, name: String },

    /// Team was selected
    TeamSelected { team: SelectedTeam },

    /// Capture region was changed
    RegionChanged { region: [u32; 4] },

    /// Audio playback started
    AudioPlaybackStarted { source: AudioSource },

    /// Audio playback finished
    AudioPlaybackFinished { source: AudioSource },

    /// An error occurred
    ErrorOccurred { message: String, context: String },

    /// Application is shutting down
    Shutdown,
}

/// Configuration field that changed
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigField {
    MusicVolume,
    AmbianceVolume,
    OcrThreshold,
    DebounceMs,
    EnableMorphOpen,
    MusicLength,
    AmbianceLength,
    AmbianceEnabled,
    AutoCheckUpdates,
    SelectedMonitor,
}

/// Audio source types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AudioSource {
    GoalMusic,
    Ambiance,
    CrowdCheer,  // For v0.3
    Commentator, // Future
}

impl Event {
    /// Get a human-readable description of the event
    pub fn description(&self) -> String {
        match self {
            Event::GoalDetected { team, .. } => {
                if let Some(team) = team {
                    format!("Goal detected for {}", team.display_name)
                } else {
                    "Goal detected".to_string()
                }
            }
            Event::MatchStarted { .. } => "Match started".to_string(),
            Event::MatchEnded {
                home_score,
                away_score,
                ..
            } => {
                format!("Match ended {}-{}", home_score, away_score)
            }
            Event::ProcessStateChanged { new_state, .. } => {
                format!("Process state: {}", new_state.description())
            }
            Event::ConfigChanged { field } => {
                format!("Config changed: {:?}", field)
            }
            Event::MusicSelected { name, .. } => {
                format!("Music selected: {}", name)
            }
            Event::TeamSelected { team } => {
                format!("Team selected: {}", team.display_name)
            }
            Event::RegionChanged { region } => {
                format!("Region changed: {:?}", region)
            }
            Event::AudioPlaybackStarted { source } => {
                format!("Audio started: {:?}", source)
            }
            Event::AudioPlaybackFinished { source } => {
                format!("Audio finished: {:?}", source)
            }
            Event::ErrorOccurred { message, .. } => {
                format!("Error: {}", message)
            }
            Event::Shutdown => "Shutting down".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_description() {
        let event = Event::GoalDetected {
            team: None,
            timestamp: Instant::now(),
        };
        assert_eq!(event.description(), "Goal detected");

        let event = Event::MatchStarted {
            timestamp: Instant::now(),
        };
        assert_eq!(event.description(), "Match started");
    }
}
