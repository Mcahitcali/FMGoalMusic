/// Goal detector implementation
///
/// Detects when a goal is scored and identifies the scoring team.

use super::detector::{Detector, DetectionContext, DetectionResult};
use super::i18n::I18nPhrases;

/// Goal detector
pub struct GoalDetector {
    phrases: I18nPhrases,
    enabled: bool,
}

impl GoalDetector {
    /// Create a new goal detector
    pub fn new(phrases: I18nPhrases) -> Self {
        Self {
            phrases,
            enabled: true,
        }
    }

    /// Enable/disable the detector
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Identify which team scored from the text
    fn identify_team(&self, text: &str, context: &DetectionContext) -> Option<String> {
        let text_lower = text.to_lowercase();

        // Check for home team name
        if let Some(ref home) = context.home_team {
            if text_lower.contains(&home.to_lowercase()) {
                return Some(home.clone());
            }
        }

        // Check for away team name
        if let Some(ref away) = context.away_team {
            if text_lower.contains(&away.to_lowercase()) {
                return Some(away.clone());
            }
        }

        // Check for generic "Home" or "Away" indicators
        if text_lower.contains(" home") || text_lower.starts_with("home") {
            return Some("Home".to_string());
        }
        if text_lower.contains(" away") || text_lower.starts_with("away") {
            return Some("Away".to_string());
        }

        None
    }

    /// Calculate confidence based on text quality
    fn calculate_confidence(&self, text: &str) -> f32 {
        let mut confidence: f32 = 0.7; // Base confidence

        // Higher confidence if team is identified
        let text_lower = text.to_lowercase();
        if text_lower.contains("home") || text_lower.contains("away") {
            confidence += 0.15;
        }

        // Higher confidence if goal phrase is exact match
        for phrase in &self.phrases.goal_phrases {
            if text.contains(phrase) {
                confidence += 0.15;
                break;
            }
        }

        confidence.min(1.0)
    }
}

impl Detector for GoalDetector {
    fn detect(&self, context: &DetectionContext) -> DetectionResult {
        if !self.enabled {
            return DetectionResult::NoMatch;
        }

        // Check if text contains goal phrase
        if !self.phrases.contains_goal_phrase(&context.text) {
            return DetectionResult::NoMatch;
        }

        // Identify team
        let team = self.identify_team(&context.text, context);

        // Calculate confidence
        let confidence = self.calculate_confidence(&context.text);

        tracing::debug!(
            "Goal detected (confidence: {:.2}): team={:?}, text='{}'",
            confidence,
            team,
            context.text
        );

        DetectionResult::Goal { team_name: team, confidence }
    }

    fn name(&self) -> &'static str {
        "GoalDetector"
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::detection::i18n::Language;

    #[test]
    fn test_goal_detection_english() {
        let phrases = I18nPhrases::new(Language::English);
        let detector = GoalDetector::new(phrases);

        let ctx = DetectionContext::new("GOAL! Home Team scores".to_string());
        let result = detector.detect(&ctx);

        match result {
            DetectionResult::Goal { team_name, confidence } => {
                assert_eq!(team_name, Some("Home".to_string()));
                assert!(confidence > 0.7);
            }
            _ => panic!("Expected Goal detection"),
        }
    }

    #[test]
    fn test_goal_detection_turkish() {
        let phrases = I18nPhrases::new(Language::Turkish);
        let detector = GoalDetector::new(phrases);

        let ctx = DetectionContext::new("GOL! Away Team".to_string());
        let result = detector.detect(&ctx);

        match result {
            DetectionResult::Goal { team_name, confidence } => {
                assert_eq!(team_name, Some("Away".to_string()));
                assert!(confidence > 0.7);
            }
            _ => panic!("Expected Goal detection"),
        }
    }

    #[test]
    fn test_goal_with_team_names() {
        let phrases = I18nPhrases::new(Language::English);
        let detector = GoalDetector::new(phrases);

        let ctx = DetectionContext::new("GOAL! Manchester United".to_string())
            .with_teams(Some("Manchester United".to_string()), Some("Liverpool".to_string()));

        let result = detector.detect(&ctx);

        match result {
            DetectionResult::Goal { team_name, .. } => {
                assert_eq!(team_name, Some("Manchester United".to_string()));
            }
            _ => panic!("Expected Goal detection"),
        }
    }

    #[test]
    fn test_no_goal_detection() {
        let phrases = I18nPhrases::new(Language::English);
        let detector = GoalDetector::new(phrases);

        let ctx = DetectionContext::new("Random text without goal phrase".to_string());
        let result = detector.detect(&ctx);

        assert_eq!(result, DetectionResult::NoMatch);
    }

    #[test]
    fn test_disabled_detector() {
        let phrases = I18nPhrases::new(Language::English);
        let mut detector = GoalDetector::new(phrases);
        detector.set_enabled(false);

        let ctx = DetectionContext::new("GOAL!".to_string());
        let result = detector.detect(&ctx);

        assert_eq!(result, DetectionResult::NoMatch);
    }

    #[test]
    fn test_confidence_calculation() {
        let phrases = I18nPhrases::new(Language::English);
        let detector = GoalDetector::new(phrases);

        // Goal with team indicator should have higher confidence
        let ctx1 = DetectionContext::new("GOAL! Home Team".to_string());
        let result1 = detector.detect(&ctx1);

        // Goal without team indicator should have lower confidence
        let ctx2 = DetectionContext::new("Goal!".to_string());
        let result2 = detector.detect(&ctx2);

        match (result1, result2) {
            (
                DetectionResult::Goal {
                    confidence: conf1, ..
                },
                DetectionResult::Goal {
                    confidence: conf2, ..
                },
            ) => {
                assert!(conf1 > conf2, "Expected higher confidence for clearer detection");
            }
            _ => panic!("Expected Goal detections"),
        }
    }
}
