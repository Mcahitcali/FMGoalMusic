/// Kickoff detector implementation
///
/// Detects match start events.
use super::detector::{DetectionContext, DetectionResult, Detector};
use super::i18n::I18nPhrases;

/// Kickoff detector
pub struct KickoffDetector {
    phrases: I18nPhrases,
    enabled: bool,
}

impl KickoffDetector {
    /// Create a new kickoff detector
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

    /// Calculate confidence based on text quality
    fn calculate_confidence(&self, text: &str) -> f32 {
        let mut confidence: f32 = 0.8; // Base confidence

        // Higher confidence for exact phrase match
        for phrase in &self.phrases.kickoff_phrases {
            if text.contains(phrase) {
                confidence = 0.95;
                break;
            }
        }

        confidence.min(1.0)
    }
}

impl Detector for KickoffDetector {
    fn detect(&self, context: &DetectionContext) -> DetectionResult {
        if !self.enabled {
            return DetectionResult::NoMatch;
        }

        // Check if text contains kickoff phrase
        if !self.phrases.contains_kickoff_phrase(&context.text) {
            return DetectionResult::NoMatch;
        }

        // Calculate confidence
        let confidence = self.calculate_confidence(&context.text);

        tracing::debug!(
            "Kickoff detected (confidence: {:.2}): text='{}'",
            confidence,
            context.text
        );

        DetectionResult::Kickoff { confidence }
    }

    fn name(&self) -> &'static str {
        "KickoffDetector"
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
    fn test_kickoff_detection_english() {
        let phrases = I18nPhrases::new(Language::English);
        let detector = KickoffDetector::new(phrases);

        let ctx = DetectionContext::new("Kick Off".to_string());
        let result = detector.detect(&ctx);

        match result {
            DetectionResult::Kickoff { confidence } => {
                assert!(confidence > 0.8);
            }
            _ => panic!("Expected Kickoff detection"),
        }
    }

    #[test]
    fn test_kickoff_detection_turkish() {
        let phrases = I18nPhrases::new(Language::Turkish);
        let detector = KickoffDetector::new(phrases);

        let ctx = DetectionContext::new("Başlangıç".to_string());
        let result = detector.detect(&ctx);

        match result {
            DetectionResult::Kickoff { confidence } => {
                assert!(confidence > 0.8);
            }
            _ => panic!("Expected Kickoff detection"),
        }
    }

    #[test]
    fn test_no_kickoff_detection() {
        let phrases = I18nPhrases::new(Language::English);
        let detector = KickoffDetector::new(phrases);

        let ctx = DetectionContext::new("Random text".to_string());
        let result = detector.detect(&ctx);

        assert_eq!(result, DetectionResult::NoMatch);
    }

    #[test]
    fn test_disabled_detector() {
        let phrases = I18nPhrases::new(Language::English);
        let mut detector = KickoffDetector::new(phrases);
        detector.set_enabled(false);

        let ctx = DetectionContext::new("Kick Off".to_string());
        let result = detector.detect(&ctx);

        assert_eq!(result, DetectionResult::NoMatch);
    }

    #[test]
    fn test_case_insensitive() {
        let phrases = I18nPhrases::new(Language::English);
        let detector = KickoffDetector::new(phrases);

        let ctx = DetectionContext::new("kick off".to_string());
        let result = detector.detect(&ctx);

        matches!(result, DetectionResult::Kickoff { .. });
    }
}
