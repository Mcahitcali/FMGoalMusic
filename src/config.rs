use display_info::DisplayInfo;
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;

fn default_music_volume() -> f32 {
    1.0
}

fn default_ambiance_volume() -> f32 {
    0.6
}

fn default_capture_region() -> [u32; 4] {
    let (screen_width, screen_height) = DisplayInfo::all()
        .ok()
        .and_then(|infos| {
            let display = infos
                .iter()
                .find(|d| d.is_primary)
                .or_else(|| infos.first());
            display.map(|d| (d.width as u32, d.height as u32))
        })
        .unwrap_or((1920, 1080));

    let screen_width = screen_width.max(1);
    let screen_height = screen_height.max(1);
    let capture_height = (screen_height / 4).max(1);
    let capture_y = screen_height.saturating_sub(capture_height);

    [0, capture_y, screen_width, capture_height]
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MusicEntry {
    pub name: String,
    pub path: String,
    pub shortcut: Option<String>,
}

/// Selected team information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SelectedTeam {
    pub league: String,
    pub team_key: String,
    pub display_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Screen region to capture [x, y, width, height]
    pub capture_region: [u32; 4],

    /// Binary threshold for OCR preprocessing (0-255)
    pub ocr_threshold: u8,

    /// Debounce time in milliseconds to prevent duplicate triggers
    pub debounce_ms: u64,

    /// Enable morphological opening for noise reduction
    pub enable_morph_open: bool,

    /// Number of frames to run in benchmark mode
    pub bench_frames: usize,

    /// List of music files added by the user
    #[serde(default)]
    pub music_list: Vec<MusicEntry>,

    /// Index of the selected music file
    pub selected_music_index: Option<usize>,

    /// Path to goal ambiance sound
    #[serde(default)]
    pub goal_ambiance_path: Option<String>,

    /// Whether ambiance sounds are enabled
    #[serde(default = "default_ambiance_enabled")]
    pub ambiance_enabled: bool,

    /// Music volume (0.0 to 1.0)
    #[serde(default = "default_music_volume")]
    pub music_volume: f32,

    /// Ambiance volume (0.0 to 1.0)
    #[serde(default = "default_ambiance_volume")]
    pub ambiance_volume: f32,

    /// Music length in milliseconds (0 = no limit)
    #[serde(default = "default_music_length")]
    pub music_length_ms: u64,

    /// Ambiance length in milliseconds (0 = no limit)  
    #[serde(default = "default_ambiance_length")]
    pub ambiance_length_ms: u64,

    /// Selected team for conditional audio playback
    #[serde(default)]
    pub selected_team: Option<SelectedTeam>,

    /// Automatically check for updates on startup
    #[serde(default = "default_auto_check_updates")]
    pub auto_check_updates: bool,

    /// Version to skip update notifications for
    #[serde(default)]
    pub skipped_version: Option<String>,

    /// Selected monitor index (0 = primary, 1 = second, etc.)
    #[serde(default)]
    pub selected_monitor_index: usize,
}

fn default_ambiance_enabled() -> bool {
    true
}

fn default_music_length() -> u64 {
    20000 // 20 seconds
}

fn default_ambiance_length() -> u64 {
    20000 // 20 seconds
}

fn default_auto_check_updates() -> bool {
    true // Enable by default so users discover updates
}

impl Default for Config {
    fn default() -> Self {
        Self {
            capture_region: default_capture_region(),
            ocr_threshold: 0, // 0 = automatic Otsu thresholding, or set 1-255 for manual
            debounce_ms: 8000, // 8 seconds between goal sounds
            enable_morph_open: false,
            bench_frames: 500,
            music_list: Vec::new(),
            selected_music_index: None,
            goal_ambiance_path: Some("goal_crowd_cheer.wav".to_string()),
            ambiance_enabled: default_ambiance_enabled(),
            music_volume: default_music_volume(),
            ambiance_volume: default_ambiance_volume(),
            music_length_ms: default_music_length(),
            ambiance_length_ms: default_ambiance_length(),
            selected_team: None,
            auto_check_updates: default_auto_check_updates(),
            skipped_version: None,
            selected_monitor_index: 0, // Primary monitor by default
        }
    }
}

impl Config {
    /// Load configuration from the platform-specific config directory.
    /// Creates default config if file doesn't exist.
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config_path = Self::config_path()?;

        if config_path.exists() {
            let content = fs::read_to_string(&config_path)?;
            let mut config: Config = serde_json::from_str(&content)?;

            if config.capture_region == [0, 0, 200, 100] {
                config.capture_region = default_capture_region();
                config.save()?;
                tracing::info!("✓ Migrated capture region to screen-based default");
            }

            tracing::info!("✓ Loaded config from: {}", config_path.display());
            Ok(config)
        } else {
            // Try migrating from legacy location (next to executable)
            let legacy_path = (|| -> Result<PathBuf, Box<dyn std::error::Error>> {
                let exe_path = env::current_exe()?;
                let exe_dir = exe_path
                    .parent()
                    .ok_or("Could not determine executable directory")?;
                Ok(exe_dir.join("config").join("config.json"))
            })();

            if let Ok(legacy) = legacy_path {
                if legacy.exists() {
                    if let Ok(content) = fs::read_to_string(&legacy) {
                        if let Ok(config) = serde_json::from_str::<Config>(&content) {
                            // Save migrated config to new path
                            config.save()?;
                            tracing::info!(
                                "✓ Migrated config from legacy path: {}",
                                legacy.display()
                            );
                            tracing::info!("✓ New config at: {}", config_path.display());
                            return Ok(config);
                        }
                    }
                }
            }

            // Create default config
            let config = Config::default();
            config.save()?;
            tracing::info!("✓ Created default config at: {}", config_path.display());
            tracing::info!("  Edit this file to customize settings.");
            Ok(config)
        }
    }

    /// Save configuration to disk
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

    /// Get the config file path in a user-writable config directory
    pub fn config_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
        let base = dirs::config_dir().ok_or("Could not determine user config directory")?;
        // Application-specific folder
        let app_dir = base.join("FMGoalMusic");
        // Ensure directory exists
        fs::create_dir_all(&app_dir)?;
        Ok(app_dir.join("config.json"))
    }

    /// Get the config directory path (for display purposes)
    pub fn config_dir_display() -> String {
        Self::config_path()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| "unknown".to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.capture_region, default_capture_region());
        assert_eq!(config.ocr_threshold, 0);
        assert_eq!(config.debounce_ms, 8000);
        assert_eq!(config.bench_frames, 500);
        assert!(!config.enable_morph_open);
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: Config = serde_json::from_str(&json).unwrap();

        assert_eq!(config.capture_region, deserialized.capture_region);
        assert_eq!(config.ocr_threshold, deserialized.ocr_threshold);
    }
}
