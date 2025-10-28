use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::env;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Screen region to capture [x, y, width, height]
    pub capture_region: [u32; 4],
    
    /// Path to the goal celebration audio file
    pub audio_file_path: String,
    
    /// Binary threshold for OCR preprocessing (0-255)
    pub ocr_threshold: u8,
    
    /// Debounce time in milliseconds to prevent duplicate triggers
    pub debounce_ms: u64,
    
    /// Enable morphological opening for noise reduction
    pub enable_morph_open: bool,
    
    /// Number of frames to run in benchmark mode
    pub bench_frames: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            capture_region: [0, 0, 200, 100],
            audio_file_path: "goal.mp3".to_string(),
            ocr_threshold: 150,
            debounce_ms: 8000, // 8 seconds between goal sounds
            enable_morph_open: false,
            bench_frames: 500,
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
            let config: Config = serde_json::from_str(&content)?;
            println!("✓ Loaded config from: {}", config_path.display());
            Ok(config)
        } else {
            // Create default config
            let config = Config::default();
            config.save()?;
            println!("✓ Created default config at: {}", config_path.display());
            println!("  Edit this file to customize settings.");
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
    
    /// Get the config file path (in app's base directory)
    fn config_path() -> Result<PathBuf, Box<dyn std::error::Error>> {
        let exe_path = env::current_exe()?;
        let exe_dir = exe_path.parent()
            .ok_or("Could not determine executable directory")?;
        
        let config_dir = exe_dir.join("config");
        Ok(config_dir.join("config.json"))
    }
    
    /// Get the config directory path (for display purposes)
    pub fn config_dir_display() -> String {
        Self::config_path()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| "unknown".to_string())
    }
    
    /// Get the full path to the audio file (relative to config directory)
    pub fn audio_file_full_path(&self) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let config_path = Self::config_path()?;
        let config_dir = config_path.parent()
            .ok_or("Could not determine config directory")?;
        
        Ok(config_dir.join(&self.audio_file_path))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.capture_region, [0, 0, 200, 100]);
        assert_eq!(config.ocr_threshold, 150);
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
