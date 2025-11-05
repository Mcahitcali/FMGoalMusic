/// Volume control effect
///
/// Provides volume adjustment for audio playback.

/// Volume effect configuration
#[derive(Debug, Clone, Copy)]
pub struct VolumeEffect {
    /// Volume multiplier (0.0-1.0)
    level: f32,
}

impl VolumeEffect {
    /// Create a new volume effect
    pub fn new(level: f32) -> Self {
        Self {
            level: level.clamp(0.0, 1.0),
        }
    }

    /// Get the volume level
    pub fn level(&self) -> f32 {
        self.level
    }

    /// Set the volume level
    pub fn set_level(&mut self, level: f32) {
        self.level = level.clamp(0.0, 1.0);
    }

    /// Check if muted
    pub fn is_muted(&self) -> bool {
        self.level == 0.0
    }

    /// Mute
    pub fn mute(&mut self) {
        self.level = 0.0;
    }

    /// Unmute to a specific level
    pub fn unmute(&mut self, level: f32) {
        self.level = level.clamp(0.0, 1.0);
    }
}

impl Default for VolumeEffect {
    fn default() -> Self {
        Self { level: 1.0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_volume_creation() {
        let volume = VolumeEffect::new(0.5);
        assert_eq!(volume.level(), 0.5);
    }

    #[test]
    fn test_volume_clamping() {
        let volume = VolumeEffect::new(1.5);
        assert_eq!(volume.level(), 1.0);

        let volume = VolumeEffect::new(-0.5);
        assert_eq!(volume.level(), 0.0);
    }

    #[test]
    fn test_volume_set_level() {
        let mut volume = VolumeEffect::new(0.5);
        volume.set_level(0.8);
        assert_eq!(volume.level(), 0.8);

        volume.set_level(2.0);
        assert_eq!(volume.level(), 1.0); // Clamped
    }

    #[test]
    fn test_volume_mute() {
        let mut volume = VolumeEffect::new(0.7);
        assert!(!volume.is_muted());

        volume.mute();
        assert!(volume.is_muted());
        assert_eq!(volume.level(), 0.0);

        volume.unmute(0.5);
        assert!(!volume.is_muted());
        assert_eq!(volume.level(), 0.5);
    }

    #[test]
    fn test_default_volume() {
        let volume = VolumeEffect::default();
        assert_eq!(volume.level(), 1.0);
        assert!(!volume.is_muted());
    }
}
