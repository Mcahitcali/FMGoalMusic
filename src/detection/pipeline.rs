/// Detection pipeline
///
/// Coordinates capture, preprocessing, OCR, and detection.

use super::detector::{Detector, DetectionContext, DetectionResult};
use crate::capture;
use crate::ocr::OcrEngine;
use std::sync::Arc;
use parking_lot::Mutex;

/// Detection pipeline configuration
#[derive(Debug, Clone)]
pub struct PipelineConfig {
    /// Screen region to capture [x, y, width, height]
    pub region: [u32; 4],
    /// OCR threshold (0-255)
    pub threshold: u8,
    /// Home team name (optional)
    pub home_team: Option<String>,
    /// Away team name (optional)
    pub away_team: Option<String>,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            region: [0, 0, 800, 100],
            threshold: 180,
            home_team: None,
            away_team: None,
        }
    }
}

/// Detection pipeline
///
/// Orchestrates the full detection process:
/// 1. Capture screenshot
/// 2. Preprocess image
/// 3. Run OCR
/// 4. Run detectors
pub struct DetectorPipeline {
    detector: Arc<Mutex<Box<dyn Detector>>>,
    ocr_engine: Arc<Mutex<OcrEngine>>,
    config: Arc<Mutex<PipelineConfig>>,
}

impl DetectorPipeline {
    /// Create a new detection pipeline
    pub fn new(detector: Box<dyn Detector>) -> Result<Self, Box<dyn std::error::Error>> {
        let ocr_engine = OcrEngine::new()?;

        Ok(Self {
            detector: Arc::new(Mutex::new(detector)),
            ocr_engine: Arc::new(Mutex::new(ocr_engine)),
            config: Arc::new(Mutex::new(PipelineConfig::default())),
        })
    }

    /// Update pipeline configuration
    pub fn set_config(&self, config: PipelineConfig) {
        *self.config.lock() = config;
    }

    /// Set capture region
    pub fn set_region(&self, region: [u32; 4]) {
        self.config.lock().region = region;
    }

    /// Set OCR threshold
    pub fn set_threshold(&self, threshold: u8) {
        self.config.lock().threshold = threshold;
    }

    /// Set team names
    pub fn set_teams(&self, home: Option<String>, away: Option<String>) {
        let mut config = self.config.lock();
        config.home_team = home;
        config.away_team = away;
    }

    /// Run detection pipeline
    pub fn detect(&self) -> Result<DetectionResult, Box<dyn std::error::Error>> {
        let config = self.config.lock().clone();

        // 1. Capture screenshot
        let image = capture::capture_region(
            config.region[0],
            config.region[1],
            config.region[2],
            config.region[3],
        )?;

        // 2. Preprocess image
        let processed = capture::preprocess_image(image, config.threshold);

        // 3. Run OCR
        let text = {
            let mut ocr = self.ocr_engine.lock();
            ocr.image_to_string(processed)?
        };

        // 4. Create context and run detector
        let context = DetectionContext::new(text)
            .with_teams(config.home_team, config.away_team);

        let detector = self.detector.lock();
        let result = detector.detect(&context);

        Ok(result)
    }

    /// Check if detector is enabled
    pub fn is_enabled(&self) -> bool {
        self.detector.lock().is_enabled()
    }

    /// Get detector name
    pub fn detector_name(&self) -> String {
        self.detector.lock().name().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::detection::goal_detector::GoalDetector;
    use crate::detection::i18n::{I18nPhrases, Language};

    // Note: Full pipeline tests require actual screen capture and OCR,
    // which are difficult to test in unit tests. These tests verify the
    // configuration and structure.

    #[test]
    fn test_pipeline_config_default() {
        let config = PipelineConfig::default();
        assert_eq!(config.region, [0, 0, 800, 100]);
        assert_eq!(config.threshold, 180);
        assert!(config.home_team.is_none());
        assert!(config.away_team.is_none());
    }

    #[test]
    fn test_pipeline_creation() {
        let phrases = I18nPhrases::new(Language::English);
        let detector = Box::new(GoalDetector::new(phrases));

        // Pipeline creation may fail if OCR engine can't initialize
        // (e.g., in CI without tesseract)
        if let Ok(pipeline) = DetectorPipeline::new(detector) {
            assert_eq!(pipeline.detector_name(), "GoalDetector");
            assert!(pipeline.is_enabled());
        }
    }

    #[test]
    fn test_pipeline_config_update() {
        let phrases = I18nPhrases::new(Language::English);
        let detector = Box::new(GoalDetector::new(phrases));

        if let Ok(pipeline) = DetectorPipeline::new(detector) {
            pipeline.set_region([100, 100, 500, 50]);
            pipeline.set_threshold(200);
            pipeline.set_teams(
                Some("Manchester United".to_string()),
                Some("Liverpool".to_string()),
            );

            let config = pipeline.config.lock().clone();
            assert_eq!(config.region, [100, 100, 500, 50]);
            assert_eq!(config.threshold, 200);
            assert_eq!(config.home_team, Some("Manchester United".to_string()));
            assert_eq!(config.away_team, Some("Liverpool".to_string()));
        }
    }
}
