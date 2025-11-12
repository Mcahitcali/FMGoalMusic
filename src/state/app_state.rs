/// Application state with validation
///
/// Contains all runtime state for the application with validation methods.
use std::path::PathBuf;

use super::process_state::ProcessState;

// Use SelectedTeam from config module
pub use crate::config::SelectedTeam;

/// Music entry with file path and optional keyboard shortcut
#[derive(Clone, Debug)]
pub struct MusicEntry {
    pub name: String,
    pub path: PathBuf,
    pub shortcut: Option<String>,
}

/// Shared state between GUI and background detection thread
#[derive(Clone, Debug)]
pub struct AppState {
    // Music library
    pub music_list: Vec<MusicEntry>,
    pub selected_music_index: Option<usize>,

    // Detection process
    pub process_state: ProcessState,
    pub detection_count: usize,
    pub status_message: String,

    // OCR settings
    pub capture_region: [u32; 4],
    pub ocr_threshold: u8,
    pub enable_morph_open: bool,

    // Debouncing
    pub debounce_ms: u64,

    // Team selection
    pub selected_team: Option<SelectedTeam>,

    // Audio settings
    pub music_volume: f32,
    pub ambiance_volume: f32,
    pub goal_ambiance_path: Option<String>,
    pub ambiance_enabled: bool,
    pub music_length_ms: u64,
    pub ambiance_length_ms: u64,

    // Update checker
    pub auto_check_updates: bool,
    pub skipped_version: Option<String>,

    // Multi-monitor support
    pub selected_monitor_index: usize,
    pub preview_image_path: Option<PathBuf>,
    pub preview_generation: u32,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            music_list: Vec::new(),
            selected_music_index: None,
            process_state: ProcessState::Stopped,
            detection_count: 0,
            status_message: "Ready".to_string(),
            capture_region: [0, 0, 200, 100],
            ocr_threshold: 0, // 0 = Otsu auto-thresholding
            enable_morph_open: false,
            debounce_ms: 8000, // 8 seconds
            selected_team: None,
            music_volume: 1.0,
            ambiance_volume: 0.6,
            goal_ambiance_path: None,
            ambiance_enabled: true,
            music_length_ms: 20_000,    // 20 seconds
            ambiance_length_ms: 20_000, // 20 seconds
            auto_check_updates: true,
            skipped_version: None,
            selected_monitor_index: 0, // Primary monitor
            preview_image_path: None,
            preview_generation: 0,
        }
    }
}

/// Validation errors
#[derive(Debug, Clone)]
pub enum ValidationError {
    InvalidVolume { field: String, value: f32 },
    InvalidRegion { region: [u32; 4] },
    InvalidDebounce { value: u64 },
    InvalidLength { field: String, value: u64 },
    NoMusicSelected,
    MusicFileNotFound { path: PathBuf },
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::InvalidVolume { field, value } => {
                write!(
                    f,
                    "Invalid volume for {}: {} (must be 0.0-1.0)",
                    field, value
                )
            }
            ValidationError::InvalidRegion { region } => {
                write!(
                    f,
                    "Invalid capture region: [{}, {}, {}, {}] (width and height must be > 0)",
                    region[0], region[1], region[2], region[3]
                )
            }
            ValidationError::InvalidDebounce { value } => {
                write!(f, "Invalid debounce: {}ms (must be 100-60000ms)", value)
            }
            ValidationError::InvalidLength { field, value } => {
                write!(
                    f,
                    "Invalid length for {}: {}ms (must be 1000-120000ms)",
                    field, value
                )
            }
            ValidationError::NoMusicSelected => write!(f, "No music file selected"),
            ValidationError::MusicFileNotFound { path } => {
                write!(f, "Music file not found: {}", path.display())
            }
        }
    }
}

impl std::error::Error for ValidationError {}

impl AppState {
    /// Validate volume (must be between 0.0 and 1.0)
    pub fn validate_volume(&self) -> Result<(), ValidationError> {
        if !(0.0..=1.0).contains(&self.music_volume) {
            return Err(ValidationError::InvalidVolume {
                field: "music_volume".to_string(),
                value: self.music_volume,
            });
        }

        if !(0.0..=1.0).contains(&self.ambiance_volume) {
            return Err(ValidationError::InvalidVolume {
                field: "ambiance_volume".to_string(),
                value: self.ambiance_volume,
            });
        }

        Ok(())
    }

    /// Validate capture region (width and height must be > 0)
    pub fn validate_region(&self) -> Result<(), ValidationError> {
        let [_, _, width, height] = self.capture_region;
        if width == 0 || height == 0 {
            return Err(ValidationError::InvalidRegion {
                region: self.capture_region,
            });
        }
        Ok(())
    }

    /// Validate debounce (must be reasonable range)
    pub fn validate_debounce(&self) -> Result<(), ValidationError> {
        if !(100..=60_000).contains(&self.debounce_ms) {
            return Err(ValidationError::InvalidDebounce {
                value: self.debounce_ms,
            });
        }
        Ok(())
    }

    /// Validate audio lengths
    pub fn validate_lengths(&self) -> Result<(), ValidationError> {
        if !(1000..=120_000).contains(&self.music_length_ms) {
            return Err(ValidationError::InvalidLength {
                field: "music_length".to_string(),
                value: self.music_length_ms,
            });
        }

        if !(1000..=120_000).contains(&self.ambiance_length_ms) {
            return Err(ValidationError::InvalidLength {
                field: "ambiance_length".to_string(),
                value: self.ambiance_length_ms,
            });
        }

        Ok(())
    }

    /// Validate music selection
    pub fn validate_music_selection(&self) -> Result<(), ValidationError> {
        let Some(index) = self.selected_music_index else {
            return Err(ValidationError::NoMusicSelected);
        };

        let Some(entry) = self.music_list.get(index) else {
            return Err(ValidationError::NoMusicSelected);
        };

        if !entry.path.exists() {
            return Err(ValidationError::MusicFileNotFound {
                path: entry.path.clone(),
            });
        }

        Ok(())
    }

    /// Validate all state fields
    pub fn validate_all(&self) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        if let Err(e) = self.validate_volume() {
            errors.push(e);
        }

        if let Err(e) = self.validate_region() {
            errors.push(e);
        }

        if let Err(e) = self.validate_debounce() {
            errors.push(e);
        }

        if let Err(e) = self.validate_lengths() {
            errors.push(e);
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Get the selected music entry
    pub fn selected_music(&self) -> Option<&MusicEntry> {
        self.selected_music_index
            .and_then(|i| self.music_list.get(i))
    }

    /// Check if detection can be started
    pub fn can_start_detection(&self) -> Result<(), ValidationError> {
        // Must have music selected
        self.validate_music_selection()?;

        // Must have valid region
        self.validate_region()?;

        // Must be stopped
        if !self.process_state.is_stopped() {
            return Err(ValidationError::InvalidRegion {
                region: self.capture_region,
            }); // TODO: Better error type
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_state_is_valid() {
        let state = AppState::default();
        assert!(state.validate_volume().is_ok());
        assert!(state.validate_region().is_ok());
        assert!(state.validate_debounce().is_ok());
        assert!(state.validate_lengths().is_ok());
    }

    #[test]
    fn test_invalid_volume() {
        let mut state = AppState::default();
        state.music_volume = 1.5;
        assert!(state.validate_volume().is_err());
    }

    #[test]
    fn test_invalid_region() {
        let mut state = AppState::default();
        state.capture_region = [0, 0, 0, 100]; // Zero width
        assert!(state.validate_region().is_err());
    }

    #[test]
    fn test_invalid_debounce() {
        let mut state = AppState::default();
        state.debounce_ms = 50; // Too short
        assert!(state.validate_debounce().is_err());
    }

    #[test]
    fn test_music_selection_validation() {
        let state = AppState::default();
        // No music selected
        assert!(state.validate_music_selection().is_err());
    }
}
