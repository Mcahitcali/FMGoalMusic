/// Time limiter effect
///
/// Limits the playback duration of audio.

/// Limiter effect configuration
#[derive(Debug, Clone, Copy)]
pub struct LimiterEffect {
    /// Maximum playback duration in milliseconds
    limit_ms: u64,
}

impl LimiterEffect {
    /// Create a new limiter effect
    pub fn new(limit_ms: u64) -> Self {
        Self { limit_ms }
    }

    /// Get the limit in milliseconds
    pub fn limit_ms(&self) -> u64 {
        self.limit_ms
    }

    /// Get the limit as a Duration
    pub fn limit_duration(&self) -> std::time::Duration {
        std::time::Duration::from_millis(self.limit_ms)
    }

    /// Set the limit
    pub fn set_limit(&mut self, limit_ms: u64) {
        self.limit_ms = limit_ms;
    }
}

impl Default for LimiterEffect {
    fn default() -> Self {
        Self {
            limit_ms: 20_000, // 20 seconds
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_limiter_creation() {
        let limiter = LimiterEffect::new(10_000);
        assert_eq!(limiter.limit_ms(), 10_000);
    }

    #[test]
    fn test_limiter_duration() {
        let limiter = LimiterEffect::new(5_000);
        assert_eq!(limiter.limit_duration(), std::time::Duration::from_millis(5_000));
    }

    #[test]
    fn test_limiter_set_limit() {
        let mut limiter = LimiterEffect::new(10_000);
        limiter.set_limit(15_000);
        assert_eq!(limiter.limit_ms(), 15_000);
    }

    #[test]
    fn test_default_limiter() {
        let limiter = LimiterEffect::default();
        assert_eq!(limiter.limit_ms(), 20_000);
    }
}
