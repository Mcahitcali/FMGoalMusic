/// Audio effects module
///
/// Provides effect decorators for audio playback: fade in/out, volume, time limiting.

pub mod fade;
pub mod volume;
pub mod limiter;

pub use fade::FadeEffect;
pub use volume::VolumeEffect;
pub use limiter::LimiterEffect;

/// Effect configuration that can be applied to audio playback
#[derive(Debug, Clone)]
pub struct EffectChain {
    /// Fade in duration in milliseconds
    pub fade_in_ms: Option<u64>,

    /// Fade out duration in milliseconds
    pub fade_out_ms: Option<u64>,

    /// Volume multiplier (0.0-1.0)
    pub volume: f32,

    /// Maximum playback duration in milliseconds
    pub limit_ms: Option<u64>,
}

impl Default for EffectChain {
    fn default() -> Self {
        Self {
            fade_in_ms: Some(200), // 200ms fade in
            fade_out_ms: Some(2000), // 2s fade out
            volume: 1.0,
            limit_ms: Some(20_000), // 20s limit
        }
    }
}

impl EffectChain {
    /// Create a new effect chain with no effects
    pub fn none() -> Self {
        Self {
            fade_in_ms: None,
            fade_out_ms: None,
            volume: 1.0,
            limit_ms: None,
        }
    }

    /// Set fade in duration
    pub fn with_fade_in(mut self, ms: u64) -> Self {
        self.fade_in_ms = Some(ms);
        self
    }

    /// Set fade out duration
    pub fn with_fade_out(mut self, ms: u64) -> Self {
        self.fade_out_ms = Some(ms);
        self
    }

    /// Set volume
    pub fn with_volume(mut self, volume: f32) -> Self {
        self.volume = volume.clamp(0.0, 1.0);
        self
    }

    /// Set time limit
    pub fn with_limit(mut self, ms: u64) -> Self {
        self.limit_ms = Some(ms);
        self
    }

    /// Remove all effects
    pub fn clear(mut self) -> Self {
        self.fade_in_ms = None;
        self.fade_out_ms = None;
        self.volume = 1.0;
        self.limit_ms = None;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_effect_chain_builder() {
        let chain = EffectChain::none()
            .with_fade_in(100)
            .with_fade_out(500)
            .with_volume(0.8)
            .with_limit(10_000);

        assert_eq!(chain.fade_in_ms, Some(100));
        assert_eq!(chain.fade_out_ms, Some(500));
        assert_eq!(chain.volume, 0.8);
        assert_eq!(chain.limit_ms, Some(10_000));
    }

    #[test]
    fn test_default_chain() {
        let chain = EffectChain::default();
        assert_eq!(chain.fade_in_ms, Some(200));
        assert_eq!(chain.fade_out_ms, Some(2000));
        assert_eq!(chain.volume, 1.0);
        assert_eq!(chain.limit_ms, Some(20_000));
    }

    #[test]
    fn test_volume_clamping() {
        let chain = EffectChain::none().with_volume(1.5);
        assert_eq!(chain.volume, 1.0); // Clamped

        let chain = EffectChain::none().with_volume(-0.5);
        assert_eq!(chain.volume, 0.0); // Clamped
    }
}
