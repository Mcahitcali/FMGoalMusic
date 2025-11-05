/// Audio system manager
///
/// Coordinates multiple audio players for simultaneous playback.

use std::collections::HashMap;
use std::sync::Arc;
use std::path::Path;

use parking_lot::Mutex;

use super::player::AudioPlayer;
use super::effects::EffectChain;
use super::source::AudioSourceType;

/// Audio system manager
///
/// Manages multiple audio sources playing simultaneously.
pub struct AudioSystemManager {
    players: Arc<Mutex<HashMap<AudioSourceType, AudioPlayer>>>,
}

impl AudioSystemManager {
    /// Create a new audio system manager
    pub fn new() -> Self {
        Self {
            players: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Load audio data from a file
    pub fn load_audio(
        &self,
        source_type: AudioSourceType,
        audio_path: &Path,
        effects: EffectChain,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Check if file exists
        if !audio_path.exists() {
            return Err(format!("Audio file not found: {}", audio_path.display()).into());
        }

        // Read audio data into memory
        let audio_data = std::fs::read(audio_path)?;
        tracing::info!(
            "Loaded audio for {}: {} ({} bytes)",
            source_type,
            audio_path.display(),
            audio_data.len()
        );

        self.load_audio_from_memory(source_type, Arc::new(audio_data), effects)
    }

    /// Load audio data from memory
    pub fn load_audio_from_memory(
        &self,
        source_type: AudioSourceType,
        audio_data: Arc<Vec<u8>>,
        effects: EffectChain,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Create player
        let player = AudioPlayer::new(source_type, audio_data, effects)?;

        // Store player
        let mut players = self.players.lock();
        players.insert(source_type, player);

        tracing::debug!("Audio player ready for {}", source_type);
        Ok(())
    }

    /// Play audio from a source
    pub fn play(&self, source_type: AudioSourceType) -> Result<(), Box<dyn std::error::Error>> {
        let players = self.players.lock();

        let player = players
            .get(&source_type)
            .ok_or_else(|| format!("No audio loaded for {}", source_type))?;

        // Check if source is exclusive and stop others if needed
        if source_type.is_exclusive() {
            drop(players); // Release lock before calling stop_all
            self.stop_all();
            let players = self.players.lock();
            if let Some(player) = players.get(&source_type) {
                player.play()?;
            }
        } else {
            player.play()?;
        }

        Ok(())
    }

    /// Stop audio from a specific source
    pub fn stop(&self, source_type: AudioSourceType) {
        let players = self.players.lock();
        if let Some(player) = players.get(&source_type) {
            player.stop();
        }
    }

    /// Stop all audio
    pub fn stop_all(&self) {
        let players = self.players.lock();
        for player in players.values() {
            player.stop();
        }
        tracing::debug!("Stopped all audio sources");
    }

    /// Pause audio from a specific source
    pub fn pause(&self, source_type: AudioSourceType) {
        let players = self.players.lock();
        if let Some(player) = players.get(&source_type) {
            player.pause();
        }
    }

    /// Resume audio from a specific source
    pub fn resume(&self, source_type: AudioSourceType) {
        let players = self.players.lock();
        if let Some(player) = players.get(&source_type) {
            player.resume();
        }
    }

    /// Check if audio is playing from a specific source
    pub fn is_playing(&self, source_type: AudioSourceType) -> bool {
        let players = self.players.lock();
        players
            .get(&source_type)
            .map(|p| p.is_playing())
            .unwrap_or(false)
    }

    /// Set volume for a specific source
    pub fn set_volume(&self, source_type: AudioSourceType, volume: f32) {
        let players = self.players.lock();
        if let Some(player) = players.get(&source_type) {
            player.set_volume(volume);
        }
    }

    /// Get number of loaded audio sources
    pub fn loaded_count(&self) -> usize {
        self.players.lock().len()
    }

    /// Unload audio for a specific source
    pub fn unload(&self, source_type: AudioSourceType) {
        let mut players = self.players.lock();
        if players.remove(&source_type).is_some() {
            tracing::debug!("Unloaded audio for {}", source_type);
        }
    }

    /// Unload all audio
    pub fn unload_all(&self) {
        let mut players = self.players.lock();
        players.clear();
        tracing::debug!("Unloaded all audio sources");
    }
}

impl Default for AudioSystemManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manager_creation() {
        let manager = AudioSystemManager::new();
        assert_eq!(manager.loaded_count(), 0);
    }

    #[test]
    fn test_manager_default() {
        let manager = AudioSystemManager::default();
        assert_eq!(manager.loaded_count(), 0);
    }

    // Note: More comprehensive tests would require audio test files
}
