/// Wizard state persistence
///
/// Saves and loads wizard completion state.

use super::state::WizardState;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Persisted wizard data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WizardPersistence {
    /// Whether wizard has been completed
    pub completed: bool,

    /// Version of wizard (for future migrations)
    pub version: u32,
}

impl WizardPersistence {
    /// Current wizard version
    const VERSION: u32 = 1;

    /// Create persistence data from wizard state
    pub fn from_state(state: &WizardState) -> Self {
        Self {
            completed: state.is_completed(),
            version: Self::VERSION,
        }
    }

    /// Convert to wizard state
    pub fn to_state(&self) -> WizardState {
        if self.completed {
            WizardState::completed()
        } else {
            WizardState::new()
        }
    }

    /// Get config file path
    pub fn config_file_path() -> Option<PathBuf> {
        dirs::config_dir().map(|dir| dir.join("FMGoalMusic").join("wizard.json"))
    }

    /// Save wizard state to disk
    pub fn save(state: &WizardState) -> Result<(), Box<dyn std::error::Error>> {
        let persistence = Self::from_state(state);

        let path = Self::config_file_path()
            .ok_or("Failed to get config directory")?;

        // Create parent directory if it doesn't exist
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let json = serde_json::to_string_pretty(&persistence)?;
        std::fs::write(&path, json)?;

        log::debug!("Saved wizard state to: {}", path.display());
        Ok(())
    }

    /// Load wizard state from disk
    pub fn load() -> Result<WizardState, Box<dyn std::error::Error>> {
        let path = Self::config_file_path()
            .ok_or("Failed to get config directory")?;

        if !path.exists() {
            log::debug!("No wizard state found, starting fresh");
            return Ok(WizardState::new());
        }

        let json = std::fs::read_to_string(&path)?;
        let persistence: WizardPersistence = serde_json::from_str(&json)?;

        log::debug!("Loaded wizard state from: {}", path.display());

        // Check version for future migrations
        if persistence.version != Self::VERSION {
            log::warn!(
                "Wizard config version mismatch: expected {}, found {}",
                Self::VERSION,
                persistence.version
            );
        }

        Ok(persistence.to_state())
    }

    /// Mark wizard as completed and save
    pub fn mark_completed() -> Result<(), Box<dyn std::error::Error>> {
        let mut state = WizardState::completed();
        Self::save(&state)
    }

    /// Reset wizard and save
    pub fn reset() -> Result<(), Box<dyn std::error::Error>> {
        let state = WizardState::new();
        Self::save(&state)
    }

    /// Delete wizard state file
    pub fn delete() -> Result<(), Box<dyn std::error::Error>> {
        if let Some(path) = Self::config_file_path() {
            if path.exists() {
                std::fs::remove_file(&path)?;
                log::debug!("Deleted wizard state file: {}", path.display());
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_state() {
        let state = WizardState::new();
        let persistence = WizardPersistence::from_state(&state);

        assert!(!persistence.completed);
        assert_eq!(persistence.version, WizardPersistence::VERSION);
    }

    #[test]
    fn test_from_completed_state() {
        let state = WizardState::completed();
        let persistence = WizardPersistence::from_state(&state);

        assert!(persistence.completed);
    }

    #[test]
    fn test_to_state() {
        let persistence = WizardPersistence {
            completed: false,
            version: 1,
        };

        let state = persistence.to_state();
        assert!(!state.is_completed());
        assert!(state.should_show());
    }

    #[test]
    fn test_to_completed_state() {
        let persistence = WizardPersistence {
            completed: true,
            version: 1,
        };

        let state = persistence.to_state();
        assert!(state.is_completed());
        assert!(!state.should_show());
    }

    #[test]
    fn test_serialization() {
        let persistence = WizardPersistence {
            completed: true,
            version: 1,
        };

        let json = serde_json::to_string(&persistence).unwrap();
        let deserialized: WizardPersistence = serde_json::from_str(&json).unwrap();

        assert_eq!(persistence.completed, deserialized.completed);
        assert_eq!(persistence.version, deserialized.version);
    }

    #[test]
    fn test_config_file_path() {
        let path = WizardPersistence::config_file_path();
        assert!(path.is_some());

        if let Some(path) = path {
            assert!(path.to_string_lossy().contains("FMGoalMusic"));
            assert!(path.to_string_lossy().ends_with("wizard.json"));
        }
    }
}
