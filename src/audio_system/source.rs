/// Audio source types
///
/// Defines different categories of audio that can be played simultaneously.
use std::fmt;

/// Audio source categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AudioSourceType {
    /// Goal music (primary)
    GoalMusic,

    /// Goal ambiance/crowd sound
    GoalAmbiance,

    /// Match start crowd sound (v0.2)
    MatchStart,

    /// Match end crowd reaction (v0.3)
    MatchEnd,

    /// Preview playback (UI)
    Preview,
}

impl fmt::Display for AudioSourceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AudioSourceType::GoalMusic => write!(f, "Goal Music"),
            AudioSourceType::GoalAmbiance => write!(f, "Goal Ambiance"),
            AudioSourceType::MatchStart => write!(f, "Match Start"),
            AudioSourceType::MatchEnd => write!(f, "Match End"),
            AudioSourceType::Preview => write!(f, "Preview"),
        }
    }
}

impl AudioSourceType {
    /// Check if this source should stop others when playing
    pub fn is_exclusive(&self) -> bool {
        match self {
            AudioSourceType::GoalMusic => false,    // Can play with ambiance
            AudioSourceType::GoalAmbiance => false, // Can play with music
            AudioSourceType::MatchStart => false,
            AudioSourceType::MatchEnd => false,
            AudioSourceType::Preview => true, // Preview stops everything
        }
    }

    /// Get default priority (higher = more important)
    pub fn priority(&self) -> u8 {
        match self {
            AudioSourceType::GoalMusic => 10,
            AudioSourceType::GoalAmbiance => 9,
            AudioSourceType::MatchStart => 8,
            AudioSourceType::MatchEnd => 8,
            AudioSourceType::Preview => 5,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_source_type_display() {
        assert_eq!(AudioSourceType::GoalMusic.to_string(), "Goal Music");
        assert_eq!(AudioSourceType::GoalAmbiance.to_string(), "Goal Ambiance");
    }

    #[test]
    fn test_source_exclusivity() {
        assert!(!AudioSourceType::GoalMusic.is_exclusive());
        assert!(!AudioSourceType::GoalAmbiance.is_exclusive());
        assert!(AudioSourceType::Preview.is_exclusive());
    }

    #[test]
    fn test_source_priority() {
        assert!(AudioSourceType::GoalMusic.priority() > AudioSourceType::Preview.priority());
    }
}
