/// Match end detector implementation
///
/// Detects match end events and extracts final scores.

use super::detector::{Detector, DetectionContext, DetectionResult};
use super::i18n::I18nPhrases;

/// Match end detector
pub struct MatchEndDetector {
    phrases: I18nPhrases,
    enabled: bool,
}

impl MatchEndDetector {
    /// Create a new match end detector
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

    /// Extract score from text (e.g., "3-1", "2:0")
    fn extract_score(&self, text: &str) -> Option<(u32, u32)> {
        // Common score patterns: "3-1", "2:0", "1 - 0", "4:2"
        let score_patterns = [
            regex::Regex::new(r"(\d+)\s*-\s*(\d+)").ok()?,
            regex::Regex::new(r"(\d+)\s*:\s*(\d+)").ok()?,
        ];

        for pattern in &score_patterns {
            if let Some(captures) = pattern.captures(text) {
                let home = captures.get(1)?.as_str().parse().ok()?;
                let away = captures.get(2)?.as_str().parse().ok()?;
                return Some((home, away));
            }
        }

        None
    }

    /// Calculate confidence based on text quality
    fn calculate_confidence(&self, text: &str, has_score: bool) -> f32 {
        let mut confidence: f32 = 0.7; // Base confidence

        // Higher confidence if score is present
        if has_score {
            confidence += 0.2;
        }

        // Higher confidence for exact phrase match
        for phrase in &self.phrases.match_end_phrases {
            if text.contains(phrase) {
                confidence += 0.1;
                break;
            }
        }

        confidence.min(1.0)
    }
}

impl Detector for MatchEndDetector {
    fn detect(&self, context: &DetectionContext) -> DetectionResult {
        if !self.enabled {
            return DetectionResult::NoMatch;
        }

        // Check if text contains match end phrase
        if !self.phrases.contains_match_end_phrase(&context.text) {
            return DetectionResult::NoMatch;
        }

        // Try to extract score
        let (home_score, away_score) = self
            .extract_score(&context.text)
            .unwrap_or((0, 0));

        // Calculate confidence
        let has_score = home_score > 0 || away_score > 0;
        let confidence = self.calculate_confidence(&context.text, has_score);

        log::debug!(
            "Match end detected (confidence: {:.2}): score={}-{}, text='{}'",
            confidence,
            home_score,
            away_score,
            context.text
        );

        DetectionResult::MatchEnd {
            home_score,
            away_score,
            confidence,
        }
    }

    fn name(&self) -> &'static str {
        "MatchEndDetector"
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
    fn test_match_end_detection_english() {
        let phrases = I18nPhrases::new(Language::English);
        let detector = MatchEndDetector::new(phrases);

        let ctx = DetectionContext::new("Full Time 3-1".to_string());
        let result = detector.detect(&ctx);

        match result {
            DetectionResult::MatchEnd {
                home_score,
                away_score,
                confidence,
            } => {
                assert_eq!(home_score, 3);
                assert_eq!(away_score, 1);
                assert!(confidence > 0.8);
            }
            _ => panic!("Expected MatchEnd detection"),
        }
    }

    #[test]
    fn test_match_end_detection_turkish() {
        let phrases = I18nPhrases::new(Language::Turkish);
        let detector = MatchEndDetector::new(phrases);

        let ctx = DetectionContext::new("Maç Sonu 2:0".to_string());
        let result = detector.detect(&ctx);

        match result {
            DetectionResult::MatchEnd {
                home_score,
                away_score,
                ..
            } => {
                assert_eq!(home_score, 2);
                assert_eq!(away_score, 0);
            }
            _ => panic!("Expected MatchEnd detection"),
        }
    }

    #[test]
    fn test_match_end_without_score() {
        let phrases = I18nPhrases::new(Language::English);
        let detector = MatchEndDetector::new(phrases);

        let ctx = DetectionContext::new("Full Time".to_string());
        let result = detector.detect(&ctx);

        match result {
            DetectionResult::MatchEnd {
                home_score,
                away_score,
                confidence,
            } => {
                assert_eq!(home_score, 0);
                assert_eq!(away_score, 0);
                assert!(confidence < 0.9); // Lower confidence without score
            }
            _ => panic!("Expected MatchEnd detection"),
        }
    }

    #[test]
    fn test_score_extraction_formats() {
        let phrases = I18nPhrases::new(Language::English);
        let detector = MatchEndDetector::new(phrases);

        let test_cases = vec![
            ("Full Time 3-1", (3, 1)),
            ("FT 2:0", (2, 0)),
            ("Full Time 1 - 0", (1, 0)),
            ("FT 4:2", (4, 2)),
        ];

        for (text, expected_score) in test_cases {
            let ctx = DetectionContext::new(text.to_string());
            let result = detector.detect(&ctx);

            match result {
                DetectionResult::MatchEnd {
                    home_score,
                    away_score,
                    ..
                } => {
                    assert_eq!(
                        (home_score, away_score),
                        expected_score,
                        "Failed for: {}",
                        text
                    );
                }
                _ => panic!("Expected MatchEnd detection for: {}", text),
            }
        }
    }

    #[test]
    fn test_no_match_end_detection() {
        let phrases = I18nPhrases::new(Language::English);
        let detector = MatchEndDetector::new(phrases);

        let ctx = DetectionContext::new("Random text".to_string());
        let result = detector.detect(&ctx);

        assert_eq!(result, DetectionResult::NoMatch);
    }

    #[test]
    fn test_disabled_detector() {
        let phrases = I18nPhrases::new(Language::English);
        let mut detector = MatchEndDetector::new(phrases);
        detector.set_enabled(false);

        let ctx = DetectionContext::new("Full Time 3-1".to_string());
        let result = detector.detect(&ctx);

        assert_eq!(result, DetectionResult::NoMatch);
    }
}
