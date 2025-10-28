use image::{GrayImage, ImageBuffer, Luma, Rgba};
use leptess::{LepTess, Variable};

/// OCR manager that reuses Tesseract instance for optimal performance
pub struct OcrManager {
    tess: LepTess,
    threshold: u8,
}

impl OcrManager {
    /// Create a new OcrManager with the specified binary threshold
    pub fn new(threshold: u8) -> Result<Self, Box<dyn std::error::Error>> {
        println!("Initializing Tesseract OCR...");
        
        // Initialize Tesseract
        let mut tess = LepTess::new(None, "eng")?;
        
        // Set to auto page segmentation mode to search entire image
        // PSM 3 = Fully automatic page segmentation, but no OSD (best for text blocks)
        tess.set_variable(Variable::TesseditPagesegMode, "3")?;
        
        // Don't use whitelist - allow all characters so we can find "GOAL" in longer text
        // tess.set_variable(Variable::TesseditCharWhitelist, "GOAL")?;
        
        println!("✓ Tesseract OCR initialized");
        println!("  Mode: PSM_AUTO (searches entire image)");
        println!("  Threshold: {}", threshold);
        
        Ok(Self { tess, threshold })
    }
    
    /// Detect if "GOAL" text is present in the image
    /// Returns true if "GOAL" is detected, false otherwise
    pub fn detect_goal(&mut self, image: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> Result<bool, Box<dyn std::error::Error>> {
        // Step 1: Convert RGBA to grayscale
        let gray = self.rgba_to_grayscale(image);
        
        // Step 2: Apply adaptive binary threshold
        let (width, height) = gray.dimensions();
        let mut binary_img = GrayImage::new(width, height);
        
        // Calculate automatic threshold using Otsu's method (simple approximation)
        // This works for any color combination
        let auto_threshold = self.calculate_otsu_threshold(&gray);
        
        // Use the automatic threshold for better results
        let threshold_to_use = auto_threshold;
        
        for (x, y, pixel) in gray.enumerate_pixels() {
            let value = if pixel[0] >= threshold_to_use { 255 } else { 0 };
            binary_img.put_pixel(x, y, Luma([value]));
        }
        
        // Check if we need to invert (text should be black, background white)
        // Count white vs black pixels - if more white, text is probably white, so invert
        let white_pixels = binary_img.pixels().filter(|p| p[0] > 127).count();
        let total_pixels = (width * height) as usize;
        
        if white_pixels > total_pixels / 2 {
            // More white than black = text is probably white, so invert
            image::imageops::invert(&mut binary_img);
        }
        
        // Step 3: Save to temp file (leptess requires file path)
        let temp_path = std::env::temp_dir().join("ocr_temp.png");
        binary_img.save(&temp_path)?;
        
        // Step 4: Set image for OCR
        self.tess.set_image(&temp_path)?;
        
        // Step 5: Get text
        let text = self.tess.get_utf8_text()?;
        let text = text.trim().to_uppercase();
        
        // Step 6: Check if "GOAL FOR" is detected exactly
        // Look for the exact phrase "GOAL FOR" which appears in Football Manager
        let detected = text.contains("GOAL FOR");
        
        // Clean up temp file
        let _ = std::fs::remove_file(&temp_path);
        
        Ok(detected)
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
    
    /// Convert RGBA image to grayscale
    fn rgba_to_grayscale(&self, image: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> GrayImage {
        let (width, height) = image.dimensions();
        let mut gray = GrayImage::new(width, height);
        
        for y in 0..height {
            for x in 0..width {
                let pixel = image.get_pixel(x, y);
                // Standard grayscale conversion: 0.299*R + 0.587*G + 0.114*B
                let gray_value = (0.299 * pixel[0] as f32 
                                + 0.587 * pixel[1] as f32 
                                + 0.114 * pixel[2] as f32) as u8;
                gray.put_pixel(x, y, Luma([gray_value]));
            }
        }
        
        gray
    }
    
    /// Update the binary threshold value
    pub fn set_threshold(&mut self, threshold: u8) {
        self.threshold = threshold;
    }
    
    /// Get the current threshold value
    pub fn threshold(&self) -> u8 {
        self.threshold
    }
    
    /// Get detected text (for debugging)
    pub fn get_text(&mut self, image: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> Result<String, Box<dyn std::error::Error>> {
        let gray = self.rgba_to_grayscale(image);
        
        // Apply adaptive binary threshold
        let (width, height) = gray.dimensions();
        let mut binary_img = GrayImage::new(width, height);
        
        let auto_threshold = self.calculate_otsu_threshold(&gray);
        
        for (x, y, pixel) in gray.enumerate_pixels() {
            let value = if pixel[0] >= auto_threshold { 255 } else { 0 };
            binary_img.put_pixel(x, y, Luma([value]));
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
    
    #[test]
    fn test_ocr_manager_creation() {
        let result = OcrManager::new(150);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_threshold_update() {
        let mut ocr = OcrManager::new(150).unwrap();
        assert_eq!(ocr.threshold(), 150);
        
        ocr.set_threshold(200);
        assert_eq!(ocr.threshold(), 200);
    }
}
