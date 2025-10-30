use image::{GrayImage, ImageBuffer, Luma, Rgba};
use leptess::{LepTess, Variable};

/// OCR manager that reuses Tesseract instance for optimal performance
pub struct OcrManager {
    tess: LepTess,
    manual_threshold: Option<u8>,
    enable_morph_open: bool,
}

impl OcrManager {
    /// Create a new OcrManager with optional manual threshold and morphological opening
    /// If threshold is 0, uses automatic Otsu thresholding
    pub fn new(_threshold: u8) -> Result<Self, Box<dyn std::error::Error>> {
        Self::new_with_options(_threshold, false)
    }
    
    /// Create OcrManager with full configuration options
    pub fn new_with_options(threshold: u8, enable_morph_open: bool) -> Result<Self, Box<dyn std::error::Error>> {
        println!("Initializing Tesseract OCR...");
        
        // Initialize Tesseract
        let mut tess = LepTess::new(None, "eng")?;
        
        // Set to auto page segmentation mode to search entire image
        // PSM 3 = Fully automatic page segmentation, but no OSD (best for text blocks)
        tess.set_variable(Variable::TesseditPagesegMode, "3")?;
        
        // Don't use whitelist - allow all characters so we can find "GOAL" in longer text
        // tess.set_variable(Variable::TesseditCharWhitelist, "GOAL")?;
        
        let manual_threshold = if threshold == 0 { None } else { Some(threshold) };
        
        println!("✓ Tesseract OCR initialized");
        println!("  Mode: PSM_AUTO (searches entire image)");
        println!("  Threshold: {}", if manual_threshold.is_some() { 
            format!("Manual ({})", threshold) 
        } else { 
            "Automatic (Otsu)".to_string() 
        });
        println!("  Morphological opening: {}", if enable_morph_open { "Enabled" } else { "Disabled" });
        
        Ok(Self { 
            tess,
            manual_threshold,
            enable_morph_open,
        })
    }
    
    /// Detect if "GOAL" text is present in the image
    /// Returns true if "GOAL" is detected, false otherwise
    pub fn detect_goal(&mut self, image: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> Result<bool, Box<dyn std::error::Error>> {
        // Step 1: Convert RGBA to grayscale
        let gray = self.rgba_to_grayscale(image);
        
        // Step 2: Apply binary threshold (manual or automatic)
        let (width, height) = gray.dimensions();
        let mut binary_img = GrayImage::new(width, height);
        
        // Determine threshold to use
        let threshold_to_use = match self.manual_threshold {
            Some(t) => t,
            None => self.calculate_otsu_threshold(&gray),
        };
        
        // Apply threshold
        for (x, y, pixel) in gray.enumerate_pixels() {
            let value = if pixel[0] >= threshold_to_use { 255 } else { 0 };
            binary_img.put_pixel(x, y, Luma([value]));
        }
        
        // Step 3: Apply morphological opening if enabled (noise reduction)
        if self.enable_morph_open {
            binary_img = self.morphological_opening(&binary_img);
        }
        
        // Step 4: Check if we need to invert (text should be black, background white)
        // Count white vs black pixels - if more white, text is probably white, so invert
        let white_pixels = binary_img.pixels().filter(|p| p[0] > 127).count();
        let total_pixels = (width * height) as usize;
        
        if white_pixels > total_pixels / 2 {
            // More white than black = text is probably white, so invert
            image::imageops::invert(&mut binary_img);
        }
        
        // Step 5: Save to temp file (leptess requires file path)
        let temp_path = std::env::temp_dir().join("ocr_temp.png");
        binary_img.save(&temp_path)?;
        
        // Step 6: Set image for OCR
        self.tess.set_image(&temp_path)?;
        
        // Step 7: Get text
        let text = self.tess.get_utf8_text()?;
        let text = text.trim().to_uppercase();
        
        // Step 8: Check if "GOAL FOR" is detected exactly
        // Look for the exact phrase "GOAL FOR" which appears in Football Manager
        let detected = text.contains("GOAL FOR");
        
                
        // Clean up temp file
        let _ = std::fs::remove_file(&temp_path);
        
        // If not detected with standard method, try alternative preprocessing for colored text
        if !detected {
            if let Ok(alt_detected) = self.try_alternative_preprocessing(image) {
                return Ok(alt_detected);
            }
        }
        
        Ok(detected)
    }
    
    /// Alternative preprocessing method for colored text on colored backgrounds
    fn try_alternative_preprocessing(&mut self, image: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> Result<bool, Box<dyn std::error::Error>> {
        let (width, height) = image.dimensions();
        let mut binary_img = GrayImage::new(width, height);
        
        // Method 1: Use color channel separation - try each RGB channel separately
        for channel in 0..3 {
            binary_img = GrayImage::new(width, height);
            
            for (x, y, pixel) in image.enumerate_pixels() {
                // Use only one color channel
                let channel_value = pixel[channel];
                // Apply aggressive threshold for this channel
                let value = if channel_value > 128 { 255 } else { 0 };
                binary_img.put_pixel(x, y, Luma([value]));
            }
            
            // Apply morphological opening if enabled
            if self.enable_morph_open {
                binary_img = self.morphological_opening(&binary_img);
            }
            
            // Try OCR with this channel
            let temp_path = std::env::temp_dir().join(format!("ocr_temp_ch{}.png", channel));
            binary_img.save(&temp_path)?;
            self.tess.set_image(&temp_path)?;
            
            let text = self.tess.get_utf8_text()?;
            let text = text.trim().to_uppercase();
            
            // Clean up
            let _ = std::fs::remove_file(&temp_path);
            
            if text.contains("GOAL FOR") {
                return Ok(true);
            }
        }
        
        // Method 2: Use edge detection approach
        binary_img = self.edge_based_preprocessing(image);
        
        let temp_path = std::env::temp_dir().join("ocr_temp_edge.png");
        binary_img.save(&temp_path)?;
        self.tess.set_image(&temp_path)?;
        
        let text = self.tess.get_utf8_text()?;
        let text = text.trim().to_uppercase();
        
        // Clean up
        let _ = std::fs::remove_file(&temp_path);
        
        if text.contains("GOAL FOR") {
            return Ok(true);
        }
        
        Ok(false)
    }
    
    /// Edge-based preprocessing for text on colored backgrounds
    fn edge_based_preprocessing(&self, image: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> GrayImage {
        let (width, height) = image.dimensions();
        let mut gray = GrayImage::new(width, height);
        let mut edges = GrayImage::new(width, height);
        
        // Convert to grayscale first
        for y in 0..height {
            for x in 0..width {
                let pixel = image.get_pixel(x, y);
                let gray_val = (0.299 * pixel[0] as f32 + 0.587 * pixel[1] as f32 + 0.114 * pixel[2] as f32) as u8;
                gray.put_pixel(x, y, Luma([gray_val]));
            }
        }
        
        // Apply Sobel edge detection
        for y in 1..height-1 {
            for x in 1..width-1 {
                let tl = gray.get_pixel(x-1, y-1)[0] as i32;
                let tm = gray.get_pixel(x, y-1)[0] as i32;
                let tr = gray.get_pixel(x+1, y-1)[0] as i32;
                let ml = gray.get_pixel(x-1, y)[0] as i32;
                let mr = gray.get_pixel(x+1, y)[0] as i32;
                let bl = gray.get_pixel(x-1, y+1)[0] as i32;
                let bm = gray.get_pixel(x, y+1)[0] as i32;
                let br = gray.get_pixel(x+1, y+1)[0] as i32;
                
                let gx = -tl - 2*ml - bl + tr + 2*mr + br;
                let gy = -tl - 2*tm - tr + bl + 2*bm + br;
                let magnitude = ((gx*gx + gy*gy) as f32).sqrt() as u8;
                
                edges.put_pixel(x, y, Luma([magnitude]));
            }
        }
        
        // Threshold edges
        let mut thresholded = GrayImage::new(width, height);
        for (x, y, pixel) in edges.enumerate_pixels() {
            let value = if pixel[0] > 50 { 255 } else { 0 };
            thresholded.put_pixel(x, y, Luma([value]));
        }
        
        thresholded
    }
    
    /// Detect goal and extract team name from "GOAL FOR [team_name]" pattern
    /// Returns Some(team_name) if goal detected, None otherwise
    pub fn detect_goal_with_team(&mut self, image: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> Result<Option<String>, Box<dyn std::error::Error>> {
        // Use same preprocessing as detect_goal
        let gray = self.rgba_to_grayscale(image);
        let (width, height) = gray.dimensions();
        let mut binary_img = GrayImage::new(width, height);
        
        let threshold_to_use = match self.manual_threshold {
            Some(t) => t,
            None => self.calculate_otsu_threshold(&gray),
        };
        
        for (x, y, pixel) in gray.enumerate_pixels() {
            let value = if pixel[0] >= threshold_to_use { 255 } else { 0 };
            binary_img.put_pixel(x, y, Luma([value]));
        }
        
        if self.enable_morph_open {
            binary_img = self.morphological_opening(&binary_img);
        }
        
        let white_pixels = binary_img.pixels().filter(|p| p[0] > 127).count();
        let total_pixels = (width * height) as usize;
        
        if white_pixels > total_pixels / 2 {
            image::imageops::invert(&mut binary_img);
        }
        
        let temp_path = std::env::temp_dir().join("ocr_temp.png");
        binary_img.save(&temp_path)?;
        self.tess.set_image(&temp_path)?;
        
        let text = self.tess.get_utf8_text()?;
        let text = text.trim().to_uppercase();
        
                
        // Clean up temp file
        let _ = std::fs::remove_file(&temp_path);
        
        // Extract team name from "GOAL FOR [team_name]" pattern
        if let Some(pos) = text.find("GOAL FOR") {
            let after_goal_for = &text[pos + 8..]; // "GOAL FOR" is 8 characters
            let team_name = after_goal_for.trim();
            
            if !team_name.is_empty() {
                return Ok(Some(team_name.to_string()));
            }
        }
        
        // If not detected with standard method, try alternative preprocessing for colored text
        if text.is_empty() || !text.contains("GOAL FOR") {
            if let Ok(alt_detected) = self.try_alternative_preprocessing(image) {
                if alt_detected {
                    // Try to extract team name with alternative preprocessing
                    if let Ok(team_name) = self.extract_team_with_alternative(image) {
                        return Ok(Some(team_name));
                    }
                }
            }
        }
        
        Ok(None)
    }
    
    /// Extract team name using alternative preprocessing methods
    fn extract_team_with_alternative(&mut self, image: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> Result<String, Box<dyn std::error::Error>> {
        let (width, height) = image.dimensions();
        let mut binary_img = GrayImage::new(width, height);
        
        // Try each RGB channel separately for team extraction
        for channel in 0..3 {
            binary_img = GrayImage::new(width, height);
            
            for (x, y, pixel) in image.enumerate_pixels() {
                let channel_value = pixel[channel];
                let value = if channel_value > 128 { 255 } else { 0 };
                binary_img.put_pixel(x, y, Luma([value]));
            }
            
            if self.enable_morph_open {
                binary_img = self.morphological_opening(&binary_img);
            }
            
            let temp_path = std::env::temp_dir().join(format!("ocr_team_ch{}.png", channel));
            binary_img.save(&temp_path)?;
            self.tess.set_image(&temp_path)?;
            
            let text = self.tess.get_utf8_text()?;
            let text = text.trim().to_uppercase();
            
            let _ = std::fs::remove_file(&temp_path);
            
            if let Some(pos) = text.find("GOAL FOR") {
                let after_goal_for = &text[pos + 8..];
                let team_name = after_goal_for.trim();
                
                if !team_name.is_empty() {
                    return Ok(team_name.to_string());
                }
            }
        }
        
        Err("No team detected with alternative methods".into())
    }
    
    /// Apply morphological opening (erosion followed by dilation)
    /// This removes small noise while preserving larger text structures
    fn morphological_opening(&self, image: &GrayImage) -> GrayImage {
        let (width, height) = image.dimensions();
        
        // Use a 3x3 structuring element (cross shape)
        // Erosion first - removes small white noise
        let mut eroded = GrayImage::new(width, height);
        for y in 1..height-1 {
            for x in 1..width-1 {
                // Check if all neighbors in cross pattern are white
                let center = image.get_pixel(x, y)[0];
                let top = image.get_pixel(x, y-1)[0];
                let bottom = image.get_pixel(x, y+1)[0];
                let left = image.get_pixel(x-1, y)[0];
                let right = image.get_pixel(x+1, y)[0];
                
                // Erosion: pixel is white only if all neighbors are white
                let value = if center > 127 && top > 127 && bottom > 127 && left > 127 && right > 127 {
                    255
                } else {
                    0
                };
                eroded.put_pixel(x, y, Luma([value]));
            }
        }
        
        // Dilation - restores the size of remaining structures
        let mut dilated = GrayImage::new(width, height);
        for y in 1..height-1 {
            for x in 1..width-1 {
                // Check if any neighbor in cross pattern is white
                let center = eroded.get_pixel(x, y)[0];
                let top = eroded.get_pixel(x, y-1)[0];
                let bottom = eroded.get_pixel(x, y+1)[0];
                let left = eroded.get_pixel(x-1, y)[0];
                let right = eroded.get_pixel(x+1, y)[0];
                
                // Dilation: pixel is white if any neighbor is white
                let value = if center > 127 || top > 127 || bottom > 127 || left > 127 || right > 127 {
                    255
                } else {
                    0
                };
                dilated.put_pixel(x, y, Luma([value]));
            }
        }
        
        dilated
    }
    
    /// Calculate optimal threshold using Otsu's method
    /// This automatically finds the best threshold for any image
    fn calculate_otsu_threshold(&self, gray: &GrayImage) -> u8 {
        // Build histogram
        let mut histogram = [0u32; 256];
        for pixel in gray.pixels() {
            histogram[pixel[0] as usize] += 1;
        }
        
        let total_pixels = gray.width() * gray.height();
        
        // Calculate Otsu threshold
        let mut sum = 0u64;
        for i in 0..256 {
            sum += (i as u64) * (histogram[i] as u64);
        }
        
        let mut sum_background = 0u64;
        let mut weight_background = 0u32;
        let mut max_variance = 0.0;
        let mut threshold = 0u8;
        
        for i in 0..256 {
            weight_background += histogram[i];
            if weight_background == 0 {
                continue;
            }
            
            let weight_foreground = total_pixels - weight_background;
            if weight_foreground == 0 {
                break;
            }
            
            sum_background += (i as u64) * (histogram[i] as u64);
            
            let mean_background = sum_background as f64 / weight_background as f64;
            let mean_foreground = (sum - sum_background) as f64 / weight_foreground as f64;
            
            let variance = (weight_background as f64) * (weight_foreground as f64) 
                         * (mean_background - mean_foreground).powi(2);
            
            if variance > max_variance {
                max_variance = variance;
                threshold = i as u8;
            }
        }
        
        threshold
    }
    
    /// Convert RGBA image to grayscale with enhanced contrast for colored text
    fn rgba_to_grayscale(&self, image: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> GrayImage {
        let (width, height) = image.dimensions();
        let mut gray = GrayImage::new(width, height);
        
        for y in 0..height {
            for x in 0..width {
                let pixel = image.get_pixel(x, y);
                let r = pixel[0] as f32;
                let g = pixel[1] as f32;
                let b = pixel[2] as f32;
                let a = pixel[3] as f32;
                
                // Enhanced grayscale conversion that better handles colored text
                // Use maximum channel value for better contrast with colored text
                let max_channel = r.max(g).max(b);
                let min_channel = r.min(g).min(b);
                let saturation = if max_channel > 0.0 { (max_channel - min_channel) / max_channel } else { 0.0 };
                
                // Combine standard grayscale with saturation enhancement
                let standard_gray = 0.299 * r + 0.587 * g + 0.114 * b;
                let enhanced_gray = if saturation > 0.3 {
                    // High saturation - use max channel for better text contrast
                    max_channel
                } else {
                    // Low saturation - use standard grayscale
                    standard_gray
                };
                
                // Apply contrast enhancement
                let contrast_enhanced = ((enhanced_gray - 128.0) * 1.5 + 128.0).max(0.0).min(255.0);
                gray.put_pixel(x, y, Luma([contrast_enhanced as u8]));
            }
        }
        
        gray
    }
    
    /// Get detected text (for debugging)
    pub fn get_text(&mut self, image: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> Result<String, Box<dyn std::error::Error>> {
        let gray = self.rgba_to_grayscale(image);
        
        // Apply binary threshold (manual or automatic)
        let (width, height) = gray.dimensions();
        let mut binary_img = GrayImage::new(width, height);
        
        // Determine threshold to use
        let threshold_to_use = match self.manual_threshold {
            Some(t) => t,
            None => self.calculate_otsu_threshold(&gray),
        };
        
        for (x, y, pixel) in gray.enumerate_pixels() {
            let value = if pixel[0] >= threshold_to_use { 255 } else { 0 };
            binary_img.put_pixel(x, y, Luma([value]));
        }
        
        // Apply morphological opening if enabled
        if self.enable_morph_open {
            binary_img = self.morphological_opening(&binary_img);
        }
        
        // Auto-invert if needed
        let white_pixels = binary_img.pixels().filter(|p| p[0] > 127).count();
        let total_pixels = (width * height) as usize;
        
        if white_pixels > total_pixels / 2 {
            image::imageops::invert(&mut binary_img);
        }
        
        // Save to temp file
        let temp_path = std::env::temp_dir().join("ocr_temp_debug.png");
        binary_img.save(&temp_path)?;
        
        // Set image and get text
        self.tess.set_image(&temp_path)?;
        let text = self.tess.get_utf8_text()?;
        
        // Clean up
        let _ = std::fs::remove_file(&temp_path);
        
        Ok(text.trim().to_uppercase())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgba, RgbaImage};
    
    /// Helper function to create a test image with text
    fn create_test_image_with_text(width: u32, height: u32, text: &str, text_color: Rgba<u8>, bg_color: Rgba<u8>) -> RgbaImage {
        // Create a simple test image
        // In real testing, you'd use imageproc or similar to render actual text
        // For now, we create a basic image that can be used for testing the pipeline
        let mut img = RgbaImage::from_pixel(width, height, bg_color);
        
        // Add some "text-like" patterns in the center
        // This is a simplified version - real text rendering would use a font library
        if text.contains("GOAL") {
            // Draw some rectangular patterns to simulate text
            for y in height / 3..2 * height / 3 {
                for x in width / 4..3 * width / 4 {
                    if (x / 10) % 2 == 0 {
                        img.put_pixel(x, y, text_color);
                    }
                }
            }
        }
        
        img
    }
    
    #[test]
    fn test_ocr_manager_creation() {
        // Test with auto threshold
        let result = OcrManager::new(0);
        assert!(result.is_ok());
        
        // Test with manual threshold
        let result = OcrManager::new_with_options(150, false);
        assert!(result.is_ok());
        
        // Test with morphological opening
        let result = OcrManager::new_with_options(0, true);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_rgba_to_grayscale() {
        let manager = OcrManager::new(0).expect("Failed to create OCR manager");
        
        // Create a simple RGBA image
        let mut img = RgbaImage::new(10, 10);
        img.put_pixel(0, 0, Rgba([255, 0, 0, 255])); // Red
        img.put_pixel(1, 0, Rgba([0, 255, 0, 255])); // Green
        img.put_pixel(2, 0, Rgba([0, 0, 255, 255])); // Blue
        img.put_pixel(3, 0, Rgba([255, 255, 255, 255])); // White
        img.put_pixel(4, 0, Rgba([0, 0, 0, 255])); // Black
        
        let gray = manager.rgba_to_grayscale(&img);
        
        // Check dimensions
        assert_eq!(gray.width(), 10);
        assert_eq!(gray.height(), 10);
        
        // Check grayscale conversion (approximate values)
        // Red: 0.299*255 ≈ 76
        assert!(gray.get_pixel(0, 0)[0] > 70 && gray.get_pixel(0, 0)[0] < 82);
        
        // Green: 0.587*255 ≈ 150
        assert!(gray.get_pixel(1, 0)[0] > 145 && gray.get_pixel(1, 0)[0] < 155);
        
        // Blue: 0.114*255 ≈ 29
        assert!(gray.get_pixel(2, 0)[0] > 25 && gray.get_pixel(2, 0)[0] < 35);
        
        // White should be 255
        assert_eq!(gray.get_pixel(3, 0)[0], 255);
        
        // Black should be 0
        assert_eq!(gray.get_pixel(4, 0)[0], 0);
    }
    
    #[test]
    fn test_calculate_otsu_threshold() {
        let manager = OcrManager::new(0).expect("Failed to create OCR manager");
        
        // Create a bimodal image with two peaks (dark and light)
        let mut gray = GrayImage::new(100, 100);
        for y in 0..100 {
            for x in 0..100 {
                if x < 50 {
                    gray.put_pixel(x, y, Luma([50])); // Dark gray
                } else {
                    gray.put_pixel(x, y, Luma([200])); // Light gray
                }
            }
        }
        
        let threshold = manager.calculate_otsu_threshold(&gray);
        
        // For a bimodal distribution with peaks at 50 and 200, 
        // threshold should be somewhere in between
        assert!(threshold > 40 && threshold < 210, "Threshold was {}", threshold);
    }
    
    #[test]
    fn test_morphological_opening() {
        let manager = OcrManager::new_with_options(0, true).expect("Failed to create OCR manager");
        
        // Create an image with noise
        let mut img = GrayImage::new(10, 10);
        
        // Fill with black
        for y in 0..10 {
            for x in 0..10 {
                img.put_pixel(x, y, Luma([0]));
            }
        }
        
        // Add a single white pixel (noise)
        img.put_pixel(5, 5, Luma([255]));
        
        // Add a larger white structure
        for x in 2..5 {
            for y in 2..5 {
                img.put_pixel(x, y, Luma([255]));
            }
        }
        
        let result = manager.morphological_opening(&img);
        
        // The single pixel should be removed (or reduced)
        // The larger structure should remain (though possibly smaller)
        assert_eq!(result.width(), 10);
        assert_eq!(result.height(), 10);
        
        // Single pixel should be gone or reduced
        let center_pixel = result.get_pixel(5, 5)[0];
        assert!(center_pixel < 255, "Noise pixel should be removed or reduced");
    }
    
    #[test]
    fn test_empty_image_detection() {
        let mut manager = OcrManager::new(0).expect("Failed to create OCR manager");
        
        // Create an empty black image
        let img = RgbaImage::from_pixel(200, 100, Rgba([0, 0, 0, 255]));
        
        // Should not detect "GOAL FOR" in empty image
        let result = manager.detect_goal(&img);
        
        // This might fail or return false - either is acceptable
        match result {
            Ok(detected) => assert!(!detected, "Should not detect GOAL in empty image"),
            Err(_) => {} // OCR error is acceptable for empty image
        }
    }
    
    #[test]
    fn test_white_image_detection() {
        let mut manager = OcrManager::new(0).expect("Failed to create OCR manager");
        
        // Create a white image
        let img = RgbaImage::from_pixel(200, 100, Rgba([255, 255, 255, 255]));
        
        // Should not detect "GOAL FOR" in white image
        let result = manager.detect_goal(&img);
        
        match result {
            Ok(detected) => assert!(!detected, "Should not detect GOAL in white image"),
            Err(_) => {} // OCR error is acceptable
        }
    }
    
    #[test]
    fn test_manual_vs_auto_threshold() {
        // Test that both manual and auto threshold modes work
        let manager_auto = OcrManager::new_with_options(0, false);
        let manager_manual = OcrManager::new_with_options(128, false);
        
        assert!(manager_auto.is_ok());
        assert!(manager_manual.is_ok());
        
        let manager_auto = manager_auto.unwrap();
        let manager_manual = manager_manual.unwrap();
        
        // Verify threshold settings
        assert_eq!(manager_auto.manual_threshold, None);
        assert_eq!(manager_manual.manual_threshold, Some(128));
    }
    
    #[test]
    fn test_morphological_opening_flag() {
        let manager_disabled = OcrManager::new_with_options(0, false).unwrap();
        let manager_enabled = OcrManager::new_with_options(0, true).unwrap();
        
        assert!(!manager_disabled.enable_morph_open);
        assert!(manager_enabled.enable_morph_open);
    }
}
