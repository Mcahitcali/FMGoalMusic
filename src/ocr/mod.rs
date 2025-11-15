mod detection;
/// OCR module for goal detection in Football Manager
///
/// This module provides OCR-based goal detection from screen captures.
/// It combines image preprocessing, Tesseract OCR, and text extraction
/// to detect "GOAL FOR {team}" messages.
///
/// # Architecture
///
/// The module is split into focused submodules:
/// - `preprocessing`: Image transformations and thresholding
/// - `detection`: Tesseract OCR integration
/// - `text_extraction`: Parsing team names from OCR results
///
/// # Public API
///
/// The main interface is `OcrManager`, which provides:
/// - `new_with_options()`: Initialize with custom settings
/// - `detect_goal()`: Simple goal detection
/// - `detect_goal_with_team()`: Goal detection with team name extraction
mod preprocessing;
pub mod text_extraction;

use detection::TesseractDetector;
use image::{ImageBuffer, Rgba};
use preprocessing::ImagePreprocessor;

/// OCR manager for goal detection
///
/// Combines preprocessing, OCR, and text extraction into a simple API.
/// Maintains preprocessing settings and Tesseract instance for optimal performance.
///
/// # Example
/// ```no_run
/// use fm_goal_musics::ocr::OcrManager;
/// use image::RgbaImage;
///
/// let mut ocr = OcrManager::new_with_options(0, false)?;
/// let image: RgbaImage = /* captured screen */;
///
/// // Simple detection
/// if ocr.detect_goal(&image)? {
///     tracing::info!("Goal detected!");
/// }
///
/// // Detection with team name
/// if let Some(team) = ocr.detect_goal_with_team(&image)? {
///     tracing::info!("Goal for {}!", team);
/// }
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub struct OcrManager {
    preprocessor: ImagePreprocessor,
    detector: TesseractDetector,
}

impl OcrManager {
    /// Create a new OcrManager with configuration options
    ///
    /// # Arguments
    /// * `threshold` - Manual threshold value (0 = automatic Otsu thresholding)
    /// * `enable_morph_open` - Enable morphological opening for noise reduction
    ///
    /// # Returns
    /// `Ok(OcrManager)` on success, or error if Tesseract initialization fails
    ///
    /// # Threshold
    /// - `0`: Automatic threshold using Otsu's method (recommended)
    /// - `1-255`: Manual threshold value
    ///
    /// # Morphological Opening
    /// - `false`: Faster, suitable for clean screenshots
    /// - `true`: Slower, better for noisy screenshots (adds 5-10ms latency)
    pub fn new_with_options(
        threshold: u8,
        enable_morph_open: bool,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let preprocessor = ImagePreprocessor::new(threshold, enable_morph_open);
        let detector = TesseractDetector::new()?;

        tracing::info!(
            "  Threshold: {}",
            if threshold == 0 {
                "Automatic (Otsu)".to_string()
            } else {
                format!("Manual ({})", threshold)
            }
        );
        tracing::info!(
            "  Morphological opening: {}",
            if enable_morph_open {
                "Enabled"
            } else {
                "Disabled"
            }
        );

        Ok(Self {
            preprocessor,
            detector,
        })
    }

    /// Detect if "GOAL" text is present in the image
    ///
    /// Looks for "GOAL FOR {team}" or "GOL {team}" patterns.
    ///
    /// # Arguments
    /// * `image` - RGBA screen capture
    ///
    /// # Returns
    /// `Ok(true)` if goal detected, `Ok(false)` otherwise, or error on OCR failure
    pub fn detect_goal(
        &mut self,
        image: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        // Preprocess image
        let binary = self.preprocessor.preprocess(image);

        // Perform OCR
        let text = self.detector.detect_text(&binary)?;

        // Check for goal text
        if text_extraction::contains_goal_text(&text) {
            return Ok(true);
        }

        // If not detected, try alternative preprocessing methods
        let alt_images = self.preprocessor.try_alternative_methods(image);
        let alt_text = self.detector.detect_text_multi(alt_images)?;

        Ok(text_extraction::contains_goal_text(&alt_text))
    }

    /// Detect goal and extract team name
    ///
    /// Returns the team name if "GOAL FOR {team}" or "GOL {team}" is detected.
    ///
    /// # Arguments
    /// * `image` - RGBA screen capture
    ///
    /// # Returns
    /// `Ok(Some(team_name))` if goal detected with team name,
    /// `Ok(None)` if no goal detected, or error on OCR failure
    pub fn detect_goal_with_team(
        &mut self,
        image: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    ) -> Result<Option<String>, Box<dyn std::error::Error>> {
        // Preprocess image
        let binary = self.preprocessor.preprocess(image);

        // Perform OCR
        let text = self.detector.detect_text(&binary)?;

        // Try to extract team name
        if let Some(team) = text_extraction::extract_team_name(&text) {
            return Ok(Some(team));
        }

        // If not detected, try alternative preprocessing
        let alt_images = self.preprocessor.try_alternative_methods(image);
        let alt_text = self.detector.detect_text_multi(alt_images)?;

        Ok(text_extraction::extract_team_name(&alt_text))
    }

    /// Detect if goal text is present using custom phrases
    ///
    /// Looks for custom goal phrases in addition to hardcoded patterns.
    ///
    /// # Arguments
    /// * `image` - RGBA screen capture
    /// * `custom_phrases` - List of custom goal detection phrases
    ///
    /// # Returns
    /// `Ok(true)` if goal detected (hardcoded or custom), `Ok(false)` otherwise, or error on OCR failure
    pub fn detect_goal_with_custom_phrases(
        &mut self,
        image: &ImageBuffer<Rgba<u8>, Vec<u8>>,
        custom_phrases: &[String],
    ) -> Result<bool, Box<dyn std::error::Error>> {
        // Preprocess image
        let binary = self.preprocessor.preprocess(image);

        // Perform OCR
        let text = self.detector.detect_text(&binary)?;

        // Check for goal text (hardcoded or custom)
        if text_extraction::contains_goal_text_with_custom(&text, custom_phrases) {
            return Ok(true);
        }

        // If not detected, try alternative preprocessing methods
        let alt_images = self.preprocessor.try_alternative_methods(image);
        let alt_text = self.detector.detect_text_multi(alt_images)?;

        Ok(text_extraction::contains_goal_text_with_custom(
            &alt_text,
            custom_phrases,
        ))
    }

    /// Get detected text (for debugging)
    ///
    /// Returns the raw OCR text without any filtering.
    #[allow(dead_code)]
    pub fn get_text(
        &mut self,
        image: &ImageBuffer<Rgba<u8>, Vec<u8>>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let binary = self.preprocessor.preprocess(image);
        let text = self.detector.detect_text(&binary)?;

        if !text.is_empty() {
            return Ok(text);
        }

        let alt_images = self.preprocessor.try_alternative_methods(image);
        let alt_text = self.detector.detect_text_multi(alt_images)?;

        Ok(alt_text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::RgbaImage;

    #[test]
    fn test_ocr_manager_creation() {
        // Test with auto threshold
        let result = OcrManager::new_with_options(0, false);
        assert!(result.is_ok());

        // Test with manual threshold
        let result = OcrManager::new_with_options(150, false);
        assert!(result.is_ok());

        // Test with morphological opening
        let result = OcrManager::new_with_options(0, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_detect_goal_empty_image() {
        let mut ocr = OcrManager::new_with_options(0, false).expect("Failed to create OCR manager");

        let img = RgbaImage::from_pixel(200, 100, image::Rgba([0, 0, 0, 255]));

        match ocr.detect_goal(&img) {
            Ok(detected) => assert!(!detected),
            Err(_) => {} // OCR error acceptable for empty image
        }
    }

    #[test]
    fn test_detect_goal_white_image() {
        let mut ocr = OcrManager::new_with_options(0, false).expect("Failed to create OCR manager");

        let img = RgbaImage::from_pixel(200, 100, image::Rgba([255, 255, 255, 255]));

        match ocr.detect_goal(&img) {
            Ok(detected) => assert!(!detected),
            Err(_) => {}
        }
    }

    #[test]
    fn test_detect_goal_with_team_empty() {
        let mut ocr = OcrManager::new_with_options(0, false).expect("Failed to create OCR manager");

        let img = RgbaImage::from_pixel(200, 100, image::Rgba([0, 0, 0, 255]));

        match ocr.detect_goal_with_team(&img) {
            Ok(team) => assert!(team.is_none()),
            Err(_) => {}
        }
    }
}
