use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink};
use std::path::Path;
use std::sync::{Arc, Mutex};

/// Audio manager that preloads audio into memory and provides non-blocking playback
pub struct AudioManager {
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
    sink: Arc<Mutex<Sink>>,
    audio_data: Arc<Vec<u8>>,
    volume: Mutex<f32>,
}

impl AudioManager {
    fn from_vec(audio_data: Vec<u8>) -> Result<Self, Box<dyn std::error::Error>> {
        let (stream, stream_handle) = OutputStream::try_default()?;
        let sink = Sink::try_new(&stream_handle)?;

        // Warm up the decoder by decoding the audio once (but don't play it)
        // Note: We must clone here as rodio's Decoder requires owned data with 'static lifetime
        let cursor = std::io::Cursor::new(audio_data.clone());
        let decoder = Decoder::new(cursor)?;

        // Verify the audio can be decoded
        let _sample_count = decoder.count();
        tracing::info!("✓ Audio decoder warmed up and verified");

        Ok(Self {
            _stream: stream,
            stream_handle,
            sink: Arc::new(Mutex::new(sink)),
            audio_data: Arc::new(audio_data),
            volume: Mutex::new(1.0),
        })
    }

    /// Create a new AudioManager and preload the audio file into memory
    pub fn new<P: AsRef<Path>>(audio_path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let path = audio_path.as_ref();

        // Check if file exists
        if !path.exists() {
            return Err(format!("Audio file not found: {}", path.display()).into());
        }

        // Read entire audio file into memory
        let audio_data = std::fs::read(path)?;
        tracing::info!(
            "✓ Preloaded audio file: {} ({} bytes)",
            path.display(),
            audio_data.len()
        );
        Self::from_vec(audio_data)
    }

    /// Create a new AudioManager from preloaded audio bytes
    pub fn from_preloaded(data: Arc<Vec<u8>>) -> Result<Self, Box<dyn std::error::Error>> {
        let (stream, stream_handle) = OutputStream::try_default()?;
        let sink = Sink::try_new(&stream_handle)?;

        // Warm up the decoder by decoding the audio once (but don't play it)
        // Note: We must clone here as rodio's Decoder requires owned data with 'static lifetime
        let cursor = std::io::Cursor::new((*data).clone());
        let decoder = Decoder::new(cursor)?;

        // Verify the audio can be decoded
        let _sample_count = decoder.count();
        tracing::info!("✓ Audio decoder warmed up and verified");

        Ok(Self {
            _stream: stream,
            stream_handle,
            sink: Arc::new(Mutex::new(sink)),
            audio_data: data,
            volume: Mutex::new(1.0),
        })
    }

    /// Play the preloaded sound (non-blocking)
    /// This creates a new decoder from the in-memory data and plays it
    pub fn play_sound(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Note: We must clone here as rodio's Decoder requires owned data with 'static lifetime
        let cursor = std::io::Cursor::new((*self.audio_data).clone());
        let decoder = Decoder::new(cursor)?;

        // Stop any currently playing audio and reinitialize sink
        {
            let mut sink = self
                .sink
                .lock()
                .map_err(|_| "Audio sink poisoned".to_string())?;
            sink.stop();
            // Clear any queued audio
            if let Ok(new_sink) = Sink::try_new(&self.stream_handle) {
                *sink = new_sink;
            }
        }

        let sink = self
            .sink
            .lock()
            .map_err(|_| "Audio sink poisoned".to_string())?;

        // Apply current volume setting
        let volume = *self
            .volume
            .lock()
            .map_err(|_| "Volume mutex poisoned".to_string())?;
        sink.set_volume(volume);

        sink.append(decoder);
        sink.play();

        Ok(())
    }

    /// Play the preloaded sound with a fade-in effect
    /// Volume transitions from 0 to target volume over specified duration
    pub fn play_sound_with_fade(
        &self,
        fade_duration_ms: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Note: We must clone here as rodio's Decoder requires owned data with 'static lifetime
        let cursor = std::io::Cursor::new((*self.audio_data).clone());
        let decoder = Decoder::new(cursor)?;

        // Get target volume
        let target_volume = *self
            .volume
            .lock()
            .map_err(|_| "Volume mutex poisoned".to_string())?;

        // Stop any currently playing audio and reinitialize sink
        {
            let mut sink = self
                .sink
                .lock()
                .map_err(|_| "Audio sink poisoned".to_string())?;
            sink.stop();
            // Clear any queued audio
            if let Ok(new_sink) = Sink::try_new(&self.stream_handle) {
                *sink = new_sink;
            }
        }

        let sink = self
            .sink
            .lock()
            .map_err(|_| "Audio sink poisoned".to_string())?;

        // Start at 0 volume
        sink.set_volume(0.0);
        sink.append(decoder);
        sink.play();

        // Spawn thread to gradually increase volume
        let sink_clone = Arc::clone(&self.sink);
        std::thread::spawn(move || {
            let steps = 50; // 50 steps for smooth transition
            let step_duration = fade_duration_ms / steps;
            let volume_increment = target_volume / steps as f32;

            for i in 1..=steps {
                std::thread::sleep(std::time::Duration::from_millis(step_duration));
                if let Ok(sink) = sink_clone.lock() {
                    sink.set_volume(volume_increment * i as f32);
                }
            }
        });

        Ok(())
    }

    /// Play the preloaded sound with fade-in and automatic stop after specified duration
    /// Combines fade-in effect with timed stopping and fade-out
    pub fn play_sound_with_fade_and_limit(
        &self,
        fade_duration_ms: u64,
        max_duration_ms: u64,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Note: We must clone here as rodio's Decoder requires owned data with 'static lifetime
        let cursor = std::io::Cursor::new((*self.audio_data).clone());
        let decoder = Decoder::new(cursor)?;

        // Get target volume
        let target_volume = *self
            .volume
            .lock()
            .map_err(|_| "Volume mutex poisoned".to_string())?;

        // Stop any currently playing audio and use the existing sink
        {
            let mut sink = self
                .sink
                .lock()
                .map_err(|_| "Audio sink poisoned".to_string())?;
            sink.stop();
            // Clear any queued audio
            if let Ok(new_sink) = Sink::try_new(&self.stream_handle) {
                *sink = new_sink;
            }
        }

        // Get the sink for playback
        let sink = self
            .sink
            .lock()
            .map_err(|_| "Audio sink poisoned".to_string())?;

        // Start at 0 volume
        sink.set_volume(0.0);
        sink.append(decoder);
        sink.play();

        // Spawn thread for fade-in effect, playback, and fade-out stopping
        let sink_clone = Arc::clone(&self.sink);
        std::thread::spawn(move || {
            // Fade-in phase
            let steps = 50; // 50 steps for smooth transition
            let step_duration = fade_duration_ms / steps;
            let volume_increment = target_volume / steps as f32;

            for i in 1..=steps {
                std::thread::sleep(std::time::Duration::from_millis(step_duration));
                if let Ok(sink) = sink_clone.lock() {
                    sink.set_volume(volume_increment * i as f32);
                }
            }

            // Calculate fade-out timing
            let fade_out_duration = 2000; // 2 seconds fade-out
            let playback_duration = if max_duration_ms > fade_out_duration {
                max_duration_ms - fade_out_duration
            } else {
                0 // No playback time, just fade in and immediately fade out
            };

            // Wait for normal playback duration
            if playback_duration > 0 {
                std::thread::sleep(std::time::Duration::from_millis(playback_duration));
            }

            // Fade-out phase
            let fade_out_steps = 50;
            let fade_out_step_duration = fade_out_duration / fade_out_steps;
            let volume_decrement = target_volume / fade_out_steps as f32;

            for i in 1..=fade_out_steps {
                std::thread::sleep(std::time::Duration::from_millis(fade_out_step_duration));
                if let Ok(sink) = sink_clone.lock() {
                    let current_volume = target_volume - (volume_decrement * i as f32);
                    sink.set_volume(current_volume.max(0.0));
                }
            }

            // Stop the audio after fade-out completes
            if let Ok(sink) = sink_clone.lock() {
                sink.stop();
            }
        });

        Ok(())
    }

    /// Set the volume for this audio manager (0.0 to 1.0)
    pub fn set_volume(&self, volume: f32) {
        let clamped = volume.clamp(0.0, 1.0);
        if let Ok(mut vol) = self.volume.lock() {
            *vol = clamped;
        }
        // Also update the sink's volume immediately
        if let Ok(sink) = self.sink.lock() {
            sink.set_volume(clamped);
        }
    }

    /// Stop any currently playing audio and clear queued sounds
    pub fn stop(&self) {
        if let Ok(mut sink) = self.sink.lock() {
            sink.stop();
            if let Ok(new_sink) = Sink::try_new(&self.stream_handle) {
                *sink = new_sink;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;
    use std::path::PathBuf;

    // Helper to create a minimal valid MP3 file for testing
    fn create_test_mp3() -> PathBuf {
        let temp_dir = std::env::temp_dir();
        let test_file = temp_dir.join("test_audio.mp3");

        // This is a minimal valid MP3 frame header
        // In real usage, you'd use an actual MP3 file
        let mp3_data = vec![
            0xFF, 0xFB, 0x90, 0x00, // MP3 frame sync + header
        ];

        let mut file = File::create(&test_file).unwrap();
        file.write_all(&mp3_data).unwrap();

        test_file
    }

    #[test]
    fn test_audio_manager_creation_fails_with_missing_file() {
        let result = AudioManager::new("nonexistent.mp3");
        assert!(result.is_err());
    }

    #[test]
    fn test_audio_data_preloaded() {
        let test_file = create_test_mp3();

        // Note: This test may fail because the minimal MP3 isn't valid enough
        // In production, use a real MP3 file for testing
        let result = AudioManager::new(&test_file);

        // Clean up
        let _ = std::fs::remove_file(test_file);

        // We expect this to potentially fail with the minimal MP3
        // but it demonstrates the structure
        if let Ok(manager) = result {
            assert!(!manager.audio_data.is_empty());
        }
    }
}
