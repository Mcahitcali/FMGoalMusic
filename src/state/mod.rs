/// State management module
///
/// Centralized state management for the application with validation and persistence.

pub mod app_state;
pub mod process_state;

// Re-export commonly used types
pub use app_state::{AppState, MusicEntry, SelectedTeam, ValidationError};
pub use process_state::{ProcessState, ProcessStateMachine, TransitionError};
