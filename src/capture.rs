use image::{ImageBuffer, Rgba};
use xcap::Monitor;

/// Screen capture manager using xcap for cross-platform GPU-accelerated capture
///
/// # Platform-Specific Implementation
///
/// ## macOS
/// - Uses ScreenCaptureKit (macOS 12.3+) or Core Graphics (older versions)
/// - Requires Screen Recording permission (System Preferences > Security & Privacy > Privacy > Screen Recording)
/// - Captures at native display resolution for best quality
/// - Supports Retina displays automatically
///
/// ## Windows
/// - Uses Windows.Graphics.Capture API (Windows 10 1903+)
/// - GPU-accelerated capture using DirectX
/// - No special permissions required
/// - Supports multi-monitor setups
///
/// ## Linux
/// - Uses X11 or Wayland (via pipewire/portals)
/// - Performance may vary based on compositor
///
/// # Performance Notes
/// - Monitor instance is reused across all captures for optimal performance
/// - Native resolution capture with efficient cropping
/// - Typical latency: 10-20ms on modern systems
pub struct CaptureManager {
    monitor: Monitor,
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
    /// Create a new CaptureManager with the specified region and monitor index
    /// The monitor is identified once and reused for all captures
    ///
    /// # Arguments
    /// * `region` - Screen region to capture
    /// * `monitor_index` - Monitor index (0 = primary, 1 = second, etc.)
    pub fn new(region: CaptureRegion, monitor_index: usize) -> Result<Self, Box<dyn std::error::Error>> {
        tracing::info!("Initializing screen capturer...");
        tracing::info!("  Region: x={}, y={}, w={}, h={}", region.x, region.y, region.width, region.height);
        tracing::info!("  Monitor index: {}", monitor_index);

        // Get all available monitors
        let monitors = Monitor::all()
            .map_err(|e| format!("Failed to enumerate monitors: {}", e))?;

        if monitors.is_empty() {
            return Err("No monitors found".into());
        }

        tracing::info!("  Available monitors: {}", monitors.len());

        // Get monitor by index, fallback to primary (0) if index out of bounds
        let monitor = monitors.into_iter()
            .nth(monitor_index)
            .or_else(|| {
                tracing::warn!("  WARNING: Monitor index {} not found, falling back to primary monitor (0)", monitor_index);
                Monitor::all().ok()?.into_iter().next()
            })
            .ok_or("Failed to get monitor")?;

        // Get monitor dimensions and name (handle Result return types in xcap v0.7)
        let monitor_width = monitor.width().unwrap_or(0);
        let monitor_height = monitor.height().unwrap_or(0);
        let monitor_name = monitor.name().unwrap_or_else(|_| "Unknown".to_string());

        tracing::info!("✓ Screen capturer initialized");
        tracing::info!("  Monitor: {}x{} ({})", monitor_width, monitor_height, monitor_name);

        // Validate region is within monitor bounds
        if monitor_width == 0 || monitor_height == 0 {
            return Err("Failed to get monitor dimensions".into());
        }

        if region.x >= monitor_width || region.y >= monitor_height {
            return Err(format!(
                "Capture region starting point ({}, {}) is outside monitor bounds (0, 0, {}, {})\n\
                \n\
                Please select a region within your screen.\n\
                Your monitor size: {}x{}",
                region.x, region.y, monitor_width, monitor_height,
                monitor_width, monitor_height
            ).into());
        }

        if region.x + region.width > monitor_width || region.y + region.height > monitor_height {
            return Err(format!(
                "Capture region ({}, {}, {}, {}) extends outside monitor bounds (0, 0, {}, {})\n\
                \n\
                Region ends at: ({}, {})\n\
                Monitor size: {}x{}\n\
                \n\
                Please select a smaller region or one that fits within your screen.",
                region.x, region.y, region.width, region.height,
                monitor_width, monitor_height,
                region.x + region.width, region.y + region.height,
                monitor_width, monitor_height
            ).into());
        }

        tracing::info!("  ✓ Region validated: within bounds");

        Ok(Self {
            monitor,
            region,
        })
    }

    /// Capture the configured screen region
    /// Returns an RGBA image buffer
    ///
    /// # Performance
    /// - Captures ONLY the configured region (not full screen)
    /// - Typical latency: 10-20ms on modern systems
    /// - GPU-accelerated on Windows and macOS
    ///
    /// # Platform Notes
    /// - Output format is RGBA8
    /// - Direct region capture is more efficient than full screen + crop
    /// - Monitor reuse is critical for performance
    ///
    /// # Permissions
    /// - macOS: Requires Screen Recording permission
    ///   If permission is denied, this will return an error with instructions
    pub fn capture_region(&mut self) -> Result<ImageBuffer<Rgba<u8>, Vec<u8>>, Box<dyn std::error::Error>> {
        // Capture ONLY the configured region using xcap's capture_region()
        // This is more efficient than capturing full screen and cropping
        // Platform-specific implementations:
        // - macOS: ScreenCaptureKit or Core Graphics
        // - Windows: Windows.Graphics.Capture API (DirectX)
        // - Linux: X11 or Wayland
        let image = self.monitor.capture_region(
            self.region.x,
            self.region.y,
            self.region.width,
            self.region.height,
        ).map_err(|e| -> Box<dyn std::error::Error> {
            // Provide helpful error messages based on the error type
            let error_msg = format!("{}", e);

            #[cfg(target_os = "macos")]
            if error_msg.contains("permission") || error_msg.contains("denied") || error_msg.contains("authorization") {
                return format!(
                    "Screen Recording permission denied.\n\
                    \n\
                    To grant permission on macOS:\n\
                    1. Open System Preferences/Settings > Privacy & Security\n\
                    2. Click 'Screen Recording' (or 'Screen & System Audio Recording')\n\
                    3. Enable permission for this application\n\
                    4. Restart the application\n\
                    \n\
                    Original error: {}", e
                ).into();
            }

            format!("Failed to capture screen region: {}", e).into()
        })?;

        Ok(image)
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
