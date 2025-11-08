/// Detector trait and common types
///
/// Defines the interface for all game event detectors.
use std::time::Instant;

/// Detection result from a detector
#[derive(Debug, Clone, PartialEq)]
pub enum DetectionResult {
    /// Goal detected
    Goal {
        /// Team name (if identified from text)
        team_name: Option<String>,
        /// Confidence level (0.0-1.0)
        confidence: f32,
    },
    /// Match kickoff detected
    Kickoff {
        /// Confidence level (0.0-1.0)
        confidence: f32,
    },
    /// Match end detected
    MatchEnd {
        /// Home team score
        home_score: u32,
        /// Away team score
        away_score: u32,
        /// Confidence level (0.0-1.0)
        confidence: f32,
    },
    /// No match found
    NoMatch,
}

/// Context passed to detectors
#[derive(Debug, Clone)]
pub struct DetectionContext {
    /// OCR extracted text
    pub text: String,
    /// Detection timestamp
    pub timestamp: Instant,
    /// Configured home team name (if any)
    pub home_team: Option<String>,
    /// Configured away team name (if any)
    pub away_team: Option<String>,
}

impl DetectionContext {
    /// Create a new detection context
    pub fn new(text: String) -> Self {
        Self {
            text,
            timestamp: Instant::now(),
            home_team: None,
            away_team: None,
        }
    }

    /// Set team names
    pub fn with_teams(mut self, home: Option<String>, away: Option<String>) -> Self {
        self.home_team = home;
        self.away_team = away;
        self
    }
}

/// Detector trait
///
/// Implement this trait to create custom detectors for different game events.
pub trait Detector: Send + Sync {
    /// Detect game events from OCR text
    ///
    /// Returns a DetectionResult indicating what was detected.
    fn detect(&self, context: &DetectionContext) -> DetectionResult;

    /// Get detector name (for logging)
    fn name(&self) -> &'static str;

    /// Check if detector is enabled
    fn is_enabled(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detection_context_creation() {
        let ctx = DetectionContext::new("test text".to_string());
        assert_eq!(ctx.text, "test text");
        assert!(ctx.home_team.is_none());
        assert!(ctx.away_team.is_none());
    }

    #[test]
    fn test_detection_context_with_teams() {
        let ctx = DetectionContext::new("test".to_string())
            .with_teams(Some("Home".to_string()), Some("Away".to_string()));

        assert_eq!(ctx.home_team, Some("Home".to_string()));
        assert_eq!(ctx.away_team, Some("Away".to_string()));
    }

    #[test]
    fn test_detection_result_equality() {
        let result1 = DetectionResult::Goal {
            team_name: Some("Home".to_string()),
            confidence: 0.95,
        };
        let result2 = DetectionResult::Goal {
            team_name: Some("Home".to_string()),
            confidence: 0.95,
        };
        assert_eq!(result1, result2);
    }
}
