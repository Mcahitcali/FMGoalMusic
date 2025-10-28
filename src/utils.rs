use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Shared application state using atomic flags
/// Thread-safe flags for controlling the application
#[derive(Clone)]
pub struct AppState {
    /// Flag to indicate if the application should continue running
    pub is_running: Arc<AtomicBool>,
    /// Flag to indicate if detection is paused
    pub is_paused: Arc<AtomicBool>,
}

impl AppState {
    /// Create a new AppState with default values
    pub fn new() -> Self {
        Self {
            is_running: Arc::new(AtomicBool::new(true)),
            is_paused: Arc::new(AtomicBool::new(false)),
        }
    }
    
    /// Check if the application should continue running
    pub fn is_running(&self) -> bool {
        self.is_running.load(Ordering::Relaxed)
    }
    
    /// Set the running state
    pub fn set_running(&self, value: bool) {
        self.is_running.store(value, Ordering::Relaxed);
    }
    
    /// Check if detection is paused
    pub fn is_paused(&self) -> bool {
        self.is_paused.load(Ordering::Relaxed)
    }
    
    /// Set the paused state
    pub fn set_paused(&self, value: bool) {
        self.is_paused.store(value, Ordering::Relaxed);
    }
    
    /// Toggle pause state and return new state
    pub fn toggle_pause(&self) -> bool {
        let current = self.is_paused();
        let new_state = !current;
        self.set_paused(new_state);
        new_state
    }
    
    /// Stop the application
    pub fn stop(&self) {
        self.set_running(false);
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

/// Debounce helper to prevent rapid repeated triggers
pub struct Debouncer {
    last_trigger: Option<Instant>,
    debounce_duration: Duration,
}

impl Debouncer {
    /// Create a new debouncer with specified duration in milliseconds
    pub fn new(debounce_ms: u64) -> Self {
        Self {
            last_trigger: None,
            debounce_duration: Duration::from_millis(debounce_ms),
        }
    }
    
    /// Check if enough time has passed since last trigger
    /// Returns true if we should trigger, false if still in debounce period
    pub fn should_trigger(&mut self) -> bool {
        let now = Instant::now();
        
        match self.last_trigger {
            None => {
                // First trigger
                self.last_trigger = Some(now);
                true
            }
            Some(last) => {
                let elapsed = now.duration_since(last);
                if elapsed >= self.debounce_duration {
                    // Enough time has passed
                    self.last_trigger = Some(now);
                    true
                } else {
                    // Still in debounce period
                    false
                }
            }
        }
    }
    
    /// Reset the debouncer (clear last trigger time)
    pub fn reset(&mut self) {
        self.last_trigger = None;
    }
    
    /// Get remaining debounce time in milliseconds
    pub fn remaining_ms(&self) -> u64 {
        match self.last_trigger {
            None => 0,
            Some(last) => {
                let elapsed = Instant::now().duration_since(last);
                if elapsed >= self.debounce_duration {
                    0
                } else {
                    (self.debounce_duration - elapsed).as_millis() as u64
                }
            }
        }
    }
}

/// Performance metrics tracker
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub capture_time_ms: f64,
    pub ocr_time_ms: f64,
    pub total_time_ms: f64,
    pub fps: f64,
}

impl PerformanceMetrics {
    pub fn new() -> Self {
        Self {
            capture_time_ms: 0.0,
            ocr_time_ms: 0.0,
            total_time_ms: 0.0,
            fps: 0.0,
        }
    }
    
    /// Calculate FPS from total time
    pub fn calculate_fps(&mut self) {
        if self.total_time_ms > 0.0 {
            self.fps = 1000.0 / self.total_time_ms;
        }
    }
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple timer for measuring elapsed time
pub struct Timer {
    start: Instant,
}

impl Timer {
    /// Start a new timer
    pub fn start() -> Self {
        Self {
            start: Instant::now(),
        }
    }
    
    /// Get elapsed time in milliseconds
    pub fn elapsed_ms(&self) -> f64 {
        self.start.elapsed().as_secs_f64() * 1000.0
    }
    
    /// Get elapsed time as Duration
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    
    #[test]
    fn test_app_state() {
        let state = AppState::new();
        
        assert!(state.is_running());
        assert!(!state.is_paused());
        
        state.set_paused(true);
        assert!(state.is_paused());
        
        state.toggle_pause();
        assert!(!state.is_paused());
        
        state.stop();
        assert!(!state.is_running());
    }
    
    #[test]
    fn test_debouncer() {
        let mut debouncer = Debouncer::new(100); // 100ms debounce
        
        // First trigger should succeed
        assert!(debouncer.should_trigger());
        
        // Immediate second trigger should fail
        assert!(!debouncer.should_trigger());
        
        // Wait for debounce period
        thread::sleep(Duration::from_millis(110));
        
        // Should trigger again
        assert!(debouncer.should_trigger());
    }
    
    #[test]
    fn test_timer() {
        let timer = Timer::start();
        thread::sleep(Duration::from_millis(50));
        
        let elapsed = timer.elapsed_ms();
        assert!(elapsed >= 50.0 && elapsed < 100.0);
    }
    
    #[test]
    fn test_performance_metrics() {
        let mut metrics = PerformanceMetrics::new();
        metrics.total_time_ms = 10.0;
        metrics.calculate_fps();
        
        assert!((metrics.fps - 100.0).abs() < 0.1);
    }
}
