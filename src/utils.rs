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
    
    /// Reset the debouncer
    pub fn reset(&mut self) {
        self.last_trigger = None;
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
    
    /// Get elapsed time in microseconds
    pub fn elapsed_us(&self) -> f64 {
        self.start.elapsed().as_secs_f64() * 1_000_000.0
    }
}

/// Timing measurements for a single iteration
#[derive(Debug, Clone, Copy)]
pub struct IterationTiming {
    pub capture_us: f64,
    pub preprocess_us: f64,
    pub ocr_us: f64,
    pub audio_trigger_us: f64,
    pub total_us: f64,
}

impl IterationTiming {
    pub fn new() -> Self {
        Self {
            capture_us: 0.0,
            preprocess_us: 0.0,
            ocr_us: 0.0,
            audio_trigger_us: 0.0,
            total_us: 0.0,
        }
    }
    
    pub fn total_ms(&self) -> f64 {
        self.total_us / 1000.0
    }
}

impl Default for IterationTiming {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics collector for latency measurements
pub struct LatencyStats {
    timings: Vec<IterationTiming>,
}

impl LatencyStats {
    pub fn new() -> Self {
        Self {
            timings: Vec::new(),
        }
    }
    
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            timings: Vec::with_capacity(capacity),
        }
    }
    
    pub fn add(&mut self, timing: IterationTiming) {
        self.timings.push(timing);
    }
    
    pub fn len(&self) -> usize {
        self.timings.len()
    }
    
    pub fn is_empty(&self) -> bool {
        self.timings.is_empty()
    }
    
    /// Calculate percentile from sorted data
    fn percentile(sorted: &[f64], p: f64) -> f64 {
        if sorted.is_empty() {
            return 0.0;
        }
        
        let idx = (p / 100.0 * (sorted.len() - 1) as f64).round() as usize;
        sorted[idx]
    }
    
    /// Calculate statistics for a specific stage
    fn stage_stats(&self, extract: impl Fn(&IterationTiming) -> f64) -> (f64, f64, f64, f64) {
        if self.timings.is_empty() {
            return (0.0, 0.0, 0.0, 0.0);
        }
        
        let mut values: Vec<f64> = self.timings.iter().map(&extract).collect();
        values.sort_by(|a, b| a.partial_cmp(b).unwrap());
        
        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let p50 = Self::percentile(&values, 50.0);
        let p95 = Self::percentile(&values, 95.0);
        let p99 = Self::percentile(&values, 99.0);
        
        (mean, p50, p95, p99)
    }
    
    /// Print comprehensive benchmark report
    pub fn print_report(&self) {
        if self.timings.is_empty() {
            println!("No timing data collected");
            return;
        }
        
        println!("\n╔═══════════════════════════════════════════════════════════════╗");
        println!("║           FM Goal Musics - Latency Benchmark Report          ║");
        println!("╚═══════════════════════════════════════════════════════════════╝");
        println!("\nSample Size: {} iterations\n", self.timings.len());
        
        // Calculate stats for each stage
        let capture_stats = self.stage_stats(|t| t.capture_us);
        let preprocess_stats = self.stage_stats(|t| t.preprocess_us);
        let ocr_stats = self.stage_stats(|t| t.ocr_us);
        let audio_stats = self.stage_stats(|t| t.audio_trigger_us);
        let total_stats = self.stage_stats(|t| t.total_us);
        
        // Print table header
        println!("┌─────────────────┬──────────┬──────────┬──────────┬──────────┐");
        println!("│ Stage           │   Mean   │   p50    │   p95    │   p99    │");
        println!("├─────────────────┼──────────┼──────────┼──────────┼──────────┤");
        
        // Print each stage (in microseconds)
        Self::print_row("Capture", capture_stats);
        Self::print_row("Preprocess", preprocess_stats);
        Self::print_row("OCR", ocr_stats);
        Self::print_row("Audio Trigger", audio_stats);
        println!("├─────────────────┼──────────┼──────────┼──────────┼──────────┤");
        Self::print_row("TOTAL", total_stats);
        println!("└─────────────────┴──────────┴──────────┴──────────┴──────────┘");
        
        // Convert to milliseconds for summary
        let total_p95_ms = total_stats.2 / 1000.0;
        let total_p99_ms = total_stats.3 / 1000.0;
        
        println!("\n📊 Summary:");
        println!("  • Total p95 latency: {:.2} ms", total_p95_ms);
        println!("  • Total p99 latency: {:.2} ms", total_p99_ms);
        
        // Performance verdict
        if total_p95_ms < 100.0 {
            println!("  ✅ Performance target MET (p95 < 100ms)");
        } else {
            println!("  ❌ Performance target MISSED (p95 >= 100ms)");
            println!("     Target: < 100ms, Actual: {:.2}ms", total_p95_ms);
        }
        
        // Identify bottleneck
        let stages = [
            ("Capture", capture_stats.2),
            ("Preprocess", preprocess_stats.2),
            ("OCR", ocr_stats.2),
            ("Audio Trigger", audio_stats.2),
        ];
        
        let bottleneck = stages.iter().max_by(|a, b| a.1.partial_cmp(&b.1).unwrap()).unwrap();
        println!("\n🔍 Bottleneck: {} ({:.0} µs p95)", bottleneck.0, bottleneck.1);
        println!();
    }
    
    fn print_row(name: &str, stats: (f64, f64, f64, f64)) {
        println!(
            "│ {:<15} │ {:>6.0} µs │ {:>6.0} µs │ {:>6.0} µs │ {:>6.0} µs │",
            name, stats.0, stats.1, stats.2, stats.3
        );
    }
}

impl Default for LatencyStats {
    fn default() -> Self {
        Self::new()
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
}
