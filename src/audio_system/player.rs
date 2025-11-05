/// Audio player for individual sound playback
///
/// Handles playback of a single audio source with effects.

use std::sync::Arc;
use std::time::Duration;

use rodio::{Decoder, OutputStream, OutputStreamHandle, Sink, Source};

use super::effects::EffectChain;
use super::source::AudioSourceType;

/// Individual audio player
pub struct AudioPlayer {
    source_type: AudioSourceType,
    _stream: OutputStream,
    stream_handle: OutputStreamHandle,
    sink: Sink,
    audio_data: Arc<Vec<u8>>,
    effects: EffectChain,
}

impl AudioPlayer {
    /// Create a new audio player
    pub fn new(
        source_type: AudioSourceType,
        audio_data: Arc<Vec<u8>>,
        effects: EffectChain,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let (stream, stream_handle) = OutputStream::try_default()?;
        let sink = Sink::try_new(&stream_handle)?;

        // Verify the audio can be decoded
        let cursor = std::io::Cursor::new((*audio_data).clone());
        let decoder = Decoder::new(cursor)?;
        let _ = decoder.count(); // Warm up decoder

        tracing::debug!(
            "Created audio player for {} with effects: fade_in={:?}ms, fade_out={:?}ms, volume={}, limit={:?}ms",
            source_type,
            effects.fade_in_ms,
            effects.fade_out_ms,
            effects.volume,
            effects.limit_ms
        );

        Ok(Self {
            source_type,
            _stream: stream,
            stream_handle,
            sink,
            audio_data,
            effects,
        })
    }

    /// Play the audio with configured effects
    pub fn play(&self) -> Result<(), Box<dyn std::error::Error>> {
        tracing::info!("Playing audio: {}", self.source_type);

        // Create decoder from preloaded data
        let cursor = std::io::Cursor::new((*self.audio_data).clone());
        let source = Decoder::new(cursor)?;

        // Build the source chain with effects using trait objects
        // Each transformation returns a different type, so we use dynamic dispatch
        let source: Box<dyn Source<Item = i16> + Send> = {
            let mut boxed_source: Box<dyn Source<Item = i16> + Send> = Box::new(source);

            // Apply time limit if specified
            if let Some(limit_ms) = self.effects.limit_ms {
                let duration = Duration::from_millis(limit_ms);
                boxed_source = Box::new(boxed_source.take_duration(duration));
            }

            // Apply fade in if specified
            if let Some(fade_in_ms) = self.effects.fade_in_ms {
                let duration = Duration::from_millis(fade_in_ms);
                boxed_source = Box::new(boxed_source.fade_in(duration));
            }

            boxed_source
        };

        // Add to sink
        self.sink.append(source);

        // Apply volume
        self.sink.set_volume(self.effects.volume);

        // Start playback
        self.sink.play();

        // Schedule fade out if specified
        // Note: rodio doesn't have built-in fade-out, so we'd need to implement this
        // For now, just log it
        if let Some(fade_out_ms) = self.effects.fade_out_ms {
            tracing::debug!("Fade out of {}ms will be applied near end", fade_out_ms);
            // TODO: Implement fade-out scheduling
        }

        Ok(())
    }

    /// Stop playback
    pub fn stop(&self) {
        tracing::debug!("Stopping audio: {}", self.source_type);
        self.sink.stop();
    }

    /// Pause playback
    pub fn pause(&self) {
        self.sink.pause();
    }

    /// Resume playback
    pub fn resume(&self) {
        self.sink.play();
    }

    /// Check if audio is playing
    pub fn is_playing(&self) -> bool {
        !self.sink.empty()
    }

    /// Set volume (0.0-1.0)
    pub fn set_volume(&self, volume: f32) {
        self.sink.set_volume(volume.clamp(0.0, 1.0));
    }

    /// Get source type
    pub fn source_type(&self) -> AudioSourceType {
        self.source_type
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests are limited because rodio requires actual audio hardware
    // In a real test environment, you'd use mocks or integration tests

    #[test]
    fn test_audio_player_source_type() {
        // This test just verifies the type system works
        let source_type = AudioSourceType::GoalMusic;
        assert_eq!(source_type.to_string(), "Goal Music");
    }
}
