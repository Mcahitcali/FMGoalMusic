/// Fade in/out effect
///
/// Provides smooth volume transitions at the start and end of audio playback.

/// Fade effect configuration
#[derive(Debug, Clone, Copy)]
pub struct FadeEffect {
    /// Fade in duration in milliseconds
    pub fade_in_ms: u64,

    /// Fade out duration in milliseconds
    pub fade_out_ms: u64,
}

impl FadeEffect {
    /// Create a new fade effect
    pub fn new(fade_in_ms: u64, fade_out_ms: u64) -> Self {
        Self {
            fade_in_ms,
            fade_out_ms,
        }
    }

    /// Create a fade effect with only fade in
    pub fn fade_in(ms: u64) -> Self {
        Self {
            fade_in_ms: ms,
            fade_out_ms: 0,
        }
    }

    /// Create a fade effect with only fade out
    pub fn fade_out(ms: u64) -> Self {
        Self {
            fade_in_ms: 0,
            fade_out_ms: ms,
        }
    }

    /// Get fade in duration in seconds (for rodio)
    pub fn fade_in_duration(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.fade_in_ms)
    }

    /// Get fade out duration in seconds (for rodio)
    pub fn fade_out_duration(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.fade_out_ms)
    }
}

impl Default for FadeEffect {
    fn default() -> Self {
        Self {
            fade_in_ms: 200,   // 200ms fade in
            fade_out_ms: 2000, // 2s fade out
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fade_effect_creation() {
        let fade = FadeEffect::new(100, 500);
        assert_eq!(fade.fade_in_ms, 100);
        assert_eq!(fade.fade_out_ms, 500);
    }

    #[test]
    fn test_fade_in_only() {
        let fade = FadeEffect::fade_in(300);
        assert_eq!(fade.fade_in_ms, 300);
        assert_eq!(fade.fade_out_ms, 0);
    }

    #[test]
    fn test_fade_out_only() {
        let fade = FadeEffect::fade_out(1000);
        assert_eq!(fade.fade_in_ms, 0);
        assert_eq!(fade.fade_out_ms, 1000);
    }

    #[test]
    fn test_default_fade() {
        let fade = FadeEffect::default();
        assert_eq!(fade.fade_in_ms, 200);
        assert_eq!(fade.fade_out_ms, 2000);
    }
}
