/// Detection module
///
/// Provides abstraction for different types of game event detection.
///
/// ## Architecture
///
/// ```text
/// DetectorPipeline
///   ├── Capture (screenshot)
///   ├── Preprocess (image processing)
///   ├── OCR (text extraction)
///   └── Detector (interpretation)
///       ├── GoalDetector
///       ├── KickoffDetector
///       └── MatchEndDetector
/// ```
///
/// ## Usage
///
/// ```rust,ignore
/// use detection::{DetectorPipeline, GoalDetector};
///
/// let detector = GoalDetector::new(phrases);
/// let mut pipeline = DetectorPipeline::new(Box::new(detector));
///
/// // Configure pipeline
/// pipeline.set_region([100, 100, 300, 50]);
/// pipeline.set_threshold(180);
///
/// // Run detection
/// match pipeline.detect()? {
///     DetectionResult::Goal { team } => {
///         println!("Goal scored by: {:?}", team);
///     }
///     DetectionResult::NoMatch => {
///         // No goal detected
///     }
/// }
/// ```

pub mod detector;
pub mod goal_detector;
pub mod kickoff_detector;
pub mod match_end_detector;
pub mod i18n;
pub mod i18n_loader;
// Pipeline module will be added in a later phase when capture/OCR are properly abstracted
// pub mod pipeline;

// Re-export commonly used types
pub use detector::{Detector, DetectionResult, DetectionContext};
pub use goal_detector::GoalDetector;
pub use kickoff_detector::KickoffDetector;
pub use match_end_detector::MatchEndDetector;
pub use i18n::{I18nPhrases, Language};
pub use i18n_loader::load_phrases;
// pub use pipeline::DetectorPipeline;
