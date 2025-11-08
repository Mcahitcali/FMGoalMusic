pub mod flow;
pub mod persistence;
pub mod state;
/// First-run wizard module
///
/// Provides an onboarding experience for new users.
///
/// ## Architecture
///
/// ```text
/// WizardFlow
///   ├── WizardState (current step, completion status)
///   ├── WizardStep (enum of all steps)
///   └── Navigation (next, back, skip, complete)
/// ```
///
/// ## Usage
///
/// ```rust,ignore
/// use wizard::{WizardFlow, WizardStep};
///
/// let mut flow = WizardFlow::new();
///
/// // Check if wizard should be shown
/// if !flow.is_completed() {
///     // Show wizard UI
///     match flow.current_step() {
///         WizardStep::Welcome => {
///             // Render welcome screen
///         }
///         WizardStep::RegionSetup => {
///             // Guide user through region selection
///         }
///         // ... other steps
///     }
///
///     // Navigate
///     flow.next();
/// }
/// ```
///
/// ## Steps
///
/// 1. **Welcome** - Introduction to the application
/// 2. **Permissions** - Request screen recording permission (macOS)
/// 3. **RegionSetup** - Guide user through capture region selection
/// 4. **TeamSelection** - Help user select their team
/// 5. **AudioSetup** - Configure and test audio
/// 6. **Complete** - Finish setup and mark as completed
pub mod steps;

// Re-export commonly used types
pub use flow::WizardFlow;
pub use persistence::WizardPersistence;
pub use state::WizardState;
pub use steps::WizardStep;
