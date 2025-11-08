/// GUI Messages for MVU pattern
///
/// All possible user actions and system events in the GUI.
use std::path::PathBuf;

use crate::config::SelectedTeam;
use crate::update_checker::UpdateCheckResult;

use super::state::AppTab;

/// GUI Messages - represent all possible state changes
#[derive(Debug, Clone)]
pub enum Message {
    // Tab navigation
    TabChanged(AppTab),

    // Library tab actions
    AddMusicClicked,
    MusicFilesSelected(Vec<PathBuf>),
    MusicSelected(usize),
    RemoveMusicClicked(usize),
    PreviewAudioClicked,
    StopPreviewClicked,

    // Detection actions
    StartDetectionClicked,
    StopDetectionClicked,
    SelectRegionClicked,
    RegionSelected([u32; 4]),
    RegionSelectionCancelled,

    // Team selection actions
    LeagueSelected(String),
    TeamSelected {
        league: String,
        team_key: String,
        team: SelectedTeam,
    },
    ClearTeamSelection,

    // Settings actions
    MusicVolumeChanged(f32),
    AmbianceVolumeChanged(f32),
    OcrThresholdChanged(u8),
    DebounceChanged(u64),
    MorphOpenToggled(bool),
    MusicLengthChanged(u64),
    AmbianceLengthChanged(u64),
    AmbianceEnabledToggled(bool),
    AmbiancePathChanged(String),
    AutoCheckUpdatesToggled(bool),
    MonitorSelected(usize),

    // Update checker actions
    CheckForUpdatesClicked,
    UpdateCheckResult(UpdateCheckResult),
    DownloadUpdateClicked(String), // URL
    SkipVersionClicked(String),    // Version
    CloseUpdateModal,

    // Help tab actions
    OpenDocumentationClicked,
    OpenGitHubClicked,

    // System events (from external sources)
    CapturePreviewUpdated,
    DetectionThreadFinished,

    // Error handling
    ErrorOccurred(String),

    // No-op (for views that don't emit messages)
    None,
}

impl Message {
    /// Get a human-readable description of the message
    pub fn description(&self) -> &str {
        match self {
            Message::TabChanged(_) => "Tab changed",
            Message::AddMusicClicked => "Add music clicked",
            Message::MusicFilesSelected(_) => "Music files selected",
            Message::MusicSelected(_) => "Music selected",
            Message::RemoveMusicClicked(_) => "Remove music clicked",
            Message::PreviewAudioClicked => "Preview audio clicked",
            Message::StopPreviewClicked => "Stop preview clicked",
            Message::StartDetectionClicked => "Start detection clicked",
            Message::StopDetectionClicked => "Stop detection clicked",
            Message::SelectRegionClicked => "Select region clicked",
            Message::RegionSelected(_) => "Region selected",
            Message::RegionSelectionCancelled => "Region selection cancelled",
            Message::LeagueSelected(_) => "League selected",
            Message::TeamSelected { .. } => "Team selected",
            Message::ClearTeamSelection => "Clear team selection",
            Message::MusicVolumeChanged(_) => "Music volume changed",
            Message::AmbianceVolumeChanged(_) => "Ambiance volume changed",
            Message::OcrThresholdChanged(_) => "OCR threshold changed",
            Message::DebounceChanged(_) => "Debounce changed",
            Message::MorphOpenToggled(_) => "Morph open toggled",
            Message::MusicLengthChanged(_) => "Music length changed",
            Message::AmbianceLengthChanged(_) => "Ambiance length changed",
            Message::AmbianceEnabledToggled(_) => "Ambiance enabled toggled",
            Message::AmbiancePathChanged(_) => "Ambiance path changed",
            Message::AutoCheckUpdatesToggled(_) => "Auto check updates toggled",
            Message::MonitorSelected(_) => "Monitor selected",
            Message::CheckForUpdatesClicked => "Check for updates clicked",
            Message::UpdateCheckResult(_) => "Update check result received",
            Message::DownloadUpdateClicked(_) => "Download update clicked",
            Message::SkipVersionClicked(_) => "Skip version clicked",
            Message::CloseUpdateModal => "Close update modal",
            Message::OpenDocumentationClicked => "Open documentation clicked",
            Message::OpenGitHubClicked => "Open GitHub clicked",
            Message::CapturePreviewUpdated => "Capture preview updated",
            Message::DetectionThreadFinished => "Detection thread finished",
            Message::ErrorOccurred(_) => "Error occurred",
            Message::None => "No-op",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_description() {
        let msg = Message::StartDetectionClicked;
        assert_eq!(msg.description(), "Start detection clicked");

        let msg = Message::TabChanged(AppTab::Settings);
        assert_eq!(msg.description(), "Tab changed");
    }
}
