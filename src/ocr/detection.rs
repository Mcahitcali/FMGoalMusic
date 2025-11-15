/// Tesseract OCR detection
///
/// This module handles all Tesseract initialization and OCR operations.
/// It manages the Tesseract instance and performs OCR on preprocessed images.
use image::GrayImage;
use leptess::{LepTess, Variable};
use std::path::PathBuf;

/// Tesseract OCR detector
///
/// Manages Tesseract instance and performs OCR on binary (preprocessed) images
pub struct TesseractDetector {
    tess: LepTess,
}

impl TesseractDetector {
    /// Create a new Tesseract detector
    ///
    /// # Returns
    /// `Ok(TesseractDetector)` on success, or error if Tesseract initialization fails
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        tracing::info!("Initializing Tesseract OCR...");

        // Set up Tesseract data path for Windows bundled distribution
        let tessdata_path = Self::setup_tesseract_data_path()?;

        // Initialize Tesseract
        let mut tess = if let Some(path_buf) = tessdata_path.and_then(|p| p.to_str().map(|s| s.to_string())) {
            tracing::info!("Initializing Tesseract with datapath: {}", path_buf);
            LepTess::new(Some(&path_buf), "eng")?
        } else {
            LepTess::new(None, "eng")?
        };

        // Set to auto page segmentation mode
        // PSM 3 = Fully automatic page segmentation, but no OSD
        tess.set_variable(Variable::TesseditPagesegMode, "3")?;

        tracing::info!("✓ Tesseract OCR initialized");
        tracing::info!("  Mode: PSM_AUTO (searches entire image)");

        Ok(Self { tess })
    }

    /// Set up Tesseract data path for Windows bundled distribution
    fn setup_tesseract_data_path() -> Result<Option<PathBuf>, Box<dyn std::error::Error>> {
        #[cfg(target_os = "windows")]
        {
            // Set TESSDATA_PREFIX to the repo root so Tesseract can find ./tessdata
            let repo_root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            let tessdata_dir = repo_root.join("tessdata");
            if tessdata_dir.exists() {
                std::env::set_var("TESSDATA_PREFIX", &repo_root);
                tracing::info!(
                    "✓ Using bundled Tesseract data from {}",
                    tessdata_dir.display()
                );
                return Ok(Some(tessdata_dir));
            }

            tracing::info!(
                "⚠️  Bundled tessdata not found, falling back to system Tesseract"
            );
            return Ok(None);
        }

        #[cfg(not(target_os = "windows"))]
        {
            // On macOS, try to use tessdata bundled inside the .app Resources directory
            #[cfg(target_os = "macos")]
            {
                if let Some(dir) = Self::macos_bundled_tessdata_path() {
                    tracing::info!(
                        "✓ Using bundled Tesseract data from {}",
                        dir.display()
                    );
                    return Ok(Some(dir));
                }
                tracing::info!(
                    "⚠️  Bundled tessdata not found in app bundle, falling back to system Tesseract"
                );
            }

            // Other platforms (or macOS fallback): rely on system installation
            Ok(None)
        }
    }

    /// On macOS, compute the tessdata path inside the .app bundle, if present.
    #[cfg(target_os = "macos")]
    fn macos_bundled_tessdata_path() -> Option<PathBuf> {
        // Example executable path:
        //   /Applications/FM Goal Musics.app/Contents/MacOS/fm-goal-musics-gui
        let exe = std::env::current_exe().ok()?;
        let macos_dir = exe.parent()?; // .../Contents/MacOS
        let contents_dir = macos_dir.parent()?; // .../Contents
        let resources_dir = contents_dir.join("Resources");
        let tessdata_dir = resources_dir.join("tessdata");
        if tessdata_dir.exists() {
            Some(tessdata_dir)
        } else {
            None
        }
    }

    /// Perform OCR on a binary (preprocessed) image
    ///
    /// # Arguments
    /// * `binary_image` - Preprocessed binary image (black text on white background)
    ///
    /// # Returns
    /// Extracted text (uppercase, trimmed)
    pub fn detect_text(
        &mut self,
        binary_image: &GrayImage,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // Save to temp file (leptess requires file path)
        let temp_path = self.get_temp_path("ocr_temp.png");
        binary_image.save(&temp_path)?;

        // Set image and perform OCR
        self.tess.set_image(&temp_path)?;
        let text = self.tess.get_utf8_text()?;
        let text = text.trim().to_uppercase();

        // Log detected text for debugging
        if !text.is_empty() {
            tracing::info!("[fm-goal-musics][ocr-detect] {}", text);
        }

        // Clean up
        let _ = std::fs::remove_file(&temp_path);

        Ok(text)
    }

    /// Perform OCR on multiple preprocessed images and return first non-empty result
    ///
    /// Used for alternative preprocessing methods
    pub fn detect_text_multi(
        &mut self,
        images: Vec<GrayImage>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        for (i, image) in images.iter().enumerate() {
            let temp_path = self.get_temp_path(&format!("ocr_alt_{}.png", i));
            image.save(&temp_path)?;

            self.tess.set_image(&temp_path)?;
            let text = self.tess.get_utf8_text()?.trim().to_uppercase();

            // Log detected text
            if !text.is_empty() {
                tracing::info!("[fm-goal-musics][ocr-detect-alt-{}] {}", i, text);
            }

            // Clean up
            let _ = std::fs::remove_file(&temp_path);

            // Return first non-empty result
            if !text.is_empty() {
                return Ok(text);
            }
        }

        Ok(String::new())
    }

    /// Get temporary file path
    fn get_temp_path(&self, filename: &str) -> PathBuf {
        std::env::temp_dir().join(filename)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Luma;

    #[test]
    fn test_tesseract_creation() {
        let result = TesseractDetector::new();
        assert!(result.is_ok(), "Tesseract should initialize successfully");
    }

    #[test]
    fn test_detect_text_empty_image() {
        let mut detector = TesseractDetector::new().expect("Failed to create detector");

        // Create empty black image
        let img = GrayImage::from_pixel(100, 50, Luma([0]));

        let result = detector.detect_text(&img);

        // Should either succeed with empty text or fail gracefully
        match result {
            Ok(text) => assert!(text.is_empty() || text.len() < 10),
            Err(_) => {} // OCR error is acceptable for empty image
        }
    }

    #[test]
    fn test_detect_text_white_image() {
        let mut detector = TesseractDetector::new().expect("Failed to create detector");

        // Create white image
        let img = GrayImage::from_pixel(100, 50, Luma([255]));

        let result = detector.detect_text(&img);

        match result {
            Ok(text) => assert!(text.is_empty() || text.len() < 10),
            Err(_) => {}
        }
    }
}
