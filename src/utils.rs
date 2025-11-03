use std::time::{Duration, Instant};

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
    #[allow(dead_code)]
    pub fn reset(&mut self) {
        self.last_trigger = None;
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
    
    #[allow(dead_code)]
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
    
    #[allow(dead_code)]
    pub fn len(&self) -> usize {
        self.timings.len()
    }
    
    #[allow(dead_code)]
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
            log::info!("No timing data collected");
            return;
        }
        
        log::info!("\n╔═══════════════════════════════════════════════════════════════╗");
        log::info!("║           FM Goal Musics - Latency Benchmark Report          ║");
        log::info!("╚═══════════════════════════════════════════════════════════════╝");
        log::info!("\nSample Size: {} iterations\n", self.timings.len());
        
        // Calculate stats for each stage
        let capture_stats = self.stage_stats(|t| t.capture_us);
        let preprocess_stats = self.stage_stats(|t| t.preprocess_us);
        let ocr_stats = self.stage_stats(|t| t.ocr_us);
        let audio_stats = self.stage_stats(|t| t.audio_trigger_us);
        let total_stats = self.stage_stats(|t| t.total_us);
        
        // Print table header
        log::info!("┌─────────────────┬──────────┬──────────┬──────────┬──────────┐");
        log::info!("│ Stage           │   Mean   │   p50    │   p95    │   p99    │");
        log::info!("├─────────────────┼──────────┼──────────┼──────────┼──────────┤");
        
        // Print each stage (in microseconds)
        Self::print_row("Capture", capture_stats);
        Self::print_row("Preprocess", preprocess_stats);
        Self::print_row("OCR", ocr_stats);
        Self::print_row("Audio Trigger", audio_stats);
        log::info!("├─────────────────┼──────────┼──────────┼──────────┼──────────┤");
        Self::print_row("TOTAL", total_stats);
        log::info!("└─────────────────┴──────────┴──────────┴──────────┴──────────┘");
        
        // Convert to milliseconds for summary
        let total_p95_ms = total_stats.2 / 1000.0;
        let total_p99_ms = total_stats.3 / 1000.0;
        
        log::info!("\n📊 Summary:");
        log::info!("  • Total p95 latency: {:.2} ms", total_p95_ms);
        log::info!("  • Total p99 latency: {:.2} ms", total_p99_ms);
        
        // Performance verdict
        if total_p95_ms < 100.0 {
            log::info!("  ✅ Performance target MET (p95 < 100ms)");
        } else {
            log::info!("  ❌ Performance target MISSED (p95 >= 100ms)");
            log::info!("     Target: < 100ms, Actual: {:.2}ms", total_p95_ms);
        }
        
        // Identify bottleneck
        let stages = [
            ("Capture", capture_stats.2),
            ("Preprocess", preprocess_stats.2),
            ("OCR", ocr_stats.2),
            ("Audio Trigger", audio_stats.2),
        ];
        
        let bottleneck = stages.iter().max_by(|a, b| a.1.partial_cmp(&b.1).unwrap()).unwrap();
        log::info!("\n🔍 Bottleneck: {} ({:.0} µs p95)", bottleneck.0, bottleneck.1);
        log::info!("");
    }
    
    fn print_row(name: &str, stats: (f64, f64, f64, f64)) {
        log::info!(
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
    fn test_debouncer_reset() {
        let mut debouncer = Debouncer::new(100);
        
        // Trigger once
        assert!(debouncer.should_trigger());
        
        // Should be in debounce period
        assert!(!debouncer.should_trigger());
        
        // Reset
        debouncer.reset();
        
        // Should be able to trigger immediately after reset
        assert!(debouncer.should_trigger());
    }
    
    #[test]
    fn test_iteration_timing() {
        let timing = IterationTiming::new();
        
        assert_eq!(timing.capture_us, 0.0);
        assert_eq!(timing.preprocess_us, 0.0);
        assert_eq!(timing.ocr_us, 0.0);
        assert_eq!(timing.audio_trigger_us, 0.0);
        assert_eq!(timing.total_us, 0.0);
    }
    
    #[test]
    fn test_iteration_timing_total_ms() {
        let mut timing = IterationTiming::new();
        timing.total_us = 50000.0; // 50ms in microseconds
        
        assert_eq!(timing.total_ms(), 50.0);
    }
    
    #[test]
    fn test_latency_stats_empty() {
        let stats = LatencyStats::new();
        
        assert_eq!(stats.len(), 0);
        assert!(stats.is_empty());
    }
    
    #[test]
    fn test_latency_stats_add() {
        let mut stats = LatencyStats::new();
        
        let timing = IterationTiming {
            capture_us: 10000.0,
            preprocess_us: 5000.0,
            ocr_us: 15000.0,
            audio_trigger_us: 100.0,
            total_us: 30100.0,
        };
        
        stats.add(timing);
        
        assert_eq!(stats.len(), 1);
        assert!(!stats.is_empty());
    }
    
    #[test]
    fn test_latency_stats_with_capacity() {
        let stats = LatencyStats::with_capacity(500);
        
        assert_eq!(stats.len(), 0);
        assert!(stats.is_empty());
    }

    #[test]
    fn test_debouncer_multiple_cycles() {
        let mut debouncer = Debouncer::new(50);
        
        // Cycle 1
        assert!(debouncer.should_trigger());
        assert!(!debouncer.should_trigger());
        
        thread::sleep(Duration::from_millis(60));
        
        // Cycle 2
        assert!(debouncer.should_trigger());
        assert!(!debouncer.should_trigger());
        
        thread::sleep(Duration::from_millis(60));
        
        // Cycle 3
        assert!(debouncer.should_trigger());
    }
}
