use image::{ImageBuffer, Rgba};
use scap::capturer::{Capturer, Options, Area, Point, Size, Resolution};
use scap::frame::{Frame, FrameType};

/// Screen capture manager that reuses the capturer instance for optimal performance
pub struct CaptureManager {
    capturer: Capturer,
    region: CaptureRegion,
}

/// Represents a screen region to capture
#[derive(Debug, Clone, Copy)]
pub struct CaptureRegion {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl CaptureRegion {
    pub fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self { x, y, width, height }
    }
    
    pub fn from_array(arr: [u32; 4]) -> Self {
        Self::new(arr[0], arr[1], arr[2], arr[3])
    }
}

impl CaptureManager {
    /// Create a new CaptureManager with the specified region
    /// The capturer is initialized once and reused for all captures
    pub fn new(region: CaptureRegion) -> Result<Self, Box<dyn std::error::Error>> {
        println!("Initializing screen capturer...");
        println!("  Region: x={}, y={}, w={}, h={}", region.x, region.y, region.width, region.height);
        
        // Check platform support
        if !scap::is_supported() {
            return Err("Platform not supported".into());
        }
        
        // Check permissions
        if !scap::has_permission() {
            println!("  Requesting screen recording permission...");
            if !scap::request_permission() {
                return Err("Screen recording permission denied".into());
            }
        }
        
        // Create options with crop area matching our region
        let options = Options {
            fps: 30,
            target: None, // Primary display
            show_cursor: false,
            show_highlight: false,
            excluded_targets: None,
            output_type: FrameType::BGRAFrame,
            output_resolution: Resolution::Captured, // Use native resolution
            crop_area: Some(Area {
                origin: Point {
                    x: region.x as f64,
                    y: region.y as f64,
                },
                size: Size {
                    width: region.width as f64,
                    height: region.height as f64,
                },
            }),
            captures_audio: false, // We don't need audio
            ..Default::default()
        };
        
        // Build capturer
        let capturer = Capturer::build(options)
            .map_err(|e| {
                format!(
                    "Failed to build capturer: {:?}\n\
                    \n\
                    This usually means the capture region is outside screen bounds.\n\
                    Your region: [x={}, y={}, w={}, h={}] ends at ({}, {})\n\
                    \n\
                    To find your screen size:\n\
                    1. Take a full screenshot\n\
                    2. Check its dimensions (e.g., 1920x1080)\n\
                    3. Update config.json with a valid region\n\
                    \n\
                    Example for 1920x1080 screen (bottom 100px):\n\
                    \"capture_region\": [0, 980, 1920, 100]",
                    e,
                    region.x, region.y, region.width, region.height,
                    region.x + region.width, region.y + region.height
                )
            })?;
        
        println!("✓ Screen capturer initialized");
        
        Ok(Self {
            capturer,
            region,
        })
    }
    
    /// Capture the configured screen region
    /// Returns an RGBA image buffer
    pub fn capture_region(&mut self) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>, Box<dyn std::error::Error>> {
        // Start capture
        self.capturer.start_capture();
        
        // Get next video frame
        let video_frame = loop {
            match self.capturer.get_next_frame()? {
                Frame::Video(frame) => break frame,
                Frame::Audio(_) => continue, // Skip audio frames
            }
        };
        
        // Stop capture
        self.capturer.stop_capture();
        
        // Handle different video frame types
        use scap::frame::VideoFrame;
        
        let img_buffer = match video_frame {
            VideoFrame::BGRA(frame) => {
                // BGRA format - convert to RGBA by swapping B and R channels
                let mut pixels = frame.data.clone();
                for chunk in pixels.chunks_exact_mut(4) {
                    chunk.swap(0, 2); // B <-> R
                }
                
                ImageBuffer::from_raw(frame.width as u32, frame.height as u32, pixels)
                    .ok_or("Failed to create ImageBuffer from BGRA frame")?
            }
            VideoFrame::BGR0(frame) => {
                // BGR0 format - convert to RGBA (swap B and R, set alpha to 255)
                let mut pixels = Vec::with_capacity((frame.width * frame.height * 4) as usize);
                for chunk in frame.data.chunks_exact(4) {
                    pixels.push(chunk[2]); // R
                    pixels.push(chunk[1]); // G
                    pixels.push(chunk[0]); // B
                    pixels.push(255);      // A
                }
                
                ImageBuffer::from_raw(frame.width as u32, frame.height as u32, pixels)
                    .ok_or("Failed to create ImageBuffer from BGR0 frame")?
            }
            _ => {
                return Err("Unsupported video frame format. Please use BGRAFrame output type.".into());
            }
        };
        
        Ok(img_buffer)
    }
    
    /// Update the capture region
    pub fn set_region(&mut self, region: CaptureRegion) {
        self.region = region;
    }
    
    /// Get the current capture region
    pub fn region(&self) -> CaptureRegion {
        self.region
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_capture_region_creation() {
        let region = CaptureRegion::new(0, 0, 200, 100);
        assert_eq!(region.x, 0);
        assert_eq!(region.y, 0);
        assert_eq!(region.width, 200);
        assert_eq!(region.height, 100);
    }
    
    #[test]
    fn test_capture_region_from_array() {
        let region = CaptureRegion::from_array([10, 20, 300, 150]);
        assert_eq!(region.x, 10);
        assert_eq!(region.y, 20);
        assert_eq!(region.width, 300);
        assert_eq!(region.height, 150);
    }
}
