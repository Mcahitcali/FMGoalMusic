/// Wizard state management
///
/// Tracks the current state and progress through the wizard.

use super::steps::WizardStep;
use std::collections::HashSet;

/// Wizard state
#[derive(Debug, Clone)]
pub struct WizardState {
    /// Current step
    current_step: WizardStep,

    /// Completed steps
    completed_steps: HashSet<WizardStep>,

    /// Whether the wizard has been fully completed
    is_completed: bool,

    /// Whether the wizard should be shown
    should_show: bool,
}

impl WizardState {
    /// Create a new wizard state (fresh start)
    pub fn new() -> Self {
        Self {
            current_step: WizardStep::Welcome,
            completed_steps: HashSet::new(),
            is_completed: false,
            should_show: true,
        }
    }

    /// Create a completed wizard state (skip wizard)
    pub fn completed() -> Self {
        let mut completed_steps = HashSet::new();
        for step in WizardStep::all_steps() {
            completed_steps.insert(step);
        }

        Self {
            current_step: WizardStep::Complete,
            completed_steps,
            is_completed: true,
            should_show: false,
        }
    }

    /// Get current step
    pub fn current_step(&self) -> WizardStep {
        self.current_step
    }

    /// Set current step
    pub fn set_current_step(&mut self, step: WizardStep) {
        self.current_step = step;
    }

    /// Check if wizard is completed
    pub fn is_completed(&self) -> bool {
        self.is_completed
    }

    /// Mark wizard as completed
    pub fn mark_completed(&mut self) {
        self.is_completed = true;
        self.should_show = false;
        self.current_step = WizardStep::Complete;

        // Mark all steps as completed
        for step in WizardStep::all_steps() {
            self.completed_steps.insert(step);
        }
    }

    /// Check if wizard should be shown
    pub fn should_show(&self) -> bool {
        self.should_show && !self.is_completed
    }

    /// Hide wizard (user dismissed it)
    pub fn hide(&mut self) {
        self.should_show = false;
    }

    /// Show wizard again (user requested it)
    pub fn show(&mut self) {
        self.should_show = true;
    }

    /// Mark a step as completed
    pub fn mark_step_completed(&mut self, step: WizardStep) {
        self.completed_steps.insert(step);
    }

    /// Check if a step is completed
    pub fn is_step_completed(&self, step: WizardStep) -> bool {
        self.completed_steps.contains(&step)
    }

    /// Get completion progress (0.0-1.0)
    pub fn progress(&self) -> f32 {
        if self.is_completed {
            return 1.0;
        }

        let total_steps = WizardStep::total_steps() as f32;
        let completed_count = self.completed_steps.len() as f32;

        (completed_count / total_steps).min(1.0)
    }

    /// Get number of completed steps
    pub fn completed_count(&self) -> usize {
        self.completed_steps.len()
    }

    /// Reset wizard to beginning
    pub fn reset(&mut self) {
        self.current_step = WizardStep::Welcome;
        self.completed_steps.clear();
        self.is_completed = false;
        self.should_show = true;
    }
}

impl Default for WizardState {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_wizard_state() {
        let state = WizardState::new();
        assert_eq!(state.current_step(), WizardStep::Welcome);
        assert!(!state.is_completed());
        assert!(state.should_show());
        assert_eq!(state.completed_count(), 0);
        assert_eq!(state.progress(), 0.0);
    }

    #[test]
    fn test_completed_wizard_state() {
        let state = WizardState::completed();
        assert!(state.is_completed());
        assert!(!state.should_show());
        assert_eq!(state.completed_count(), 6);
        assert_eq!(state.progress(), 1.0);
    }

    #[test]
    fn test_mark_step_completed() {
        let mut state = WizardState::new();
        assert!(!state.is_step_completed(WizardStep::Welcome));

        state.mark_step_completed(WizardStep::Welcome);
        assert!(state.is_step_completed(WizardStep::Welcome));
        assert_eq!(state.completed_count(), 1);
    }

    #[test]
    fn test_progress_calculation() {
        let mut state = WizardState::new();
        assert_eq!(state.progress(), 0.0);

        state.mark_step_completed(WizardStep::Welcome);
        assert!(state.progress() > 0.0 && state.progress() < 1.0);

        state.mark_completed();
        assert_eq!(state.progress(), 1.0);
    }

    #[test]
    fn test_mark_completed() {
        let mut state = WizardState::new();
        assert!(!state.is_completed());

        state.mark_completed();
        assert!(state.is_completed());
        assert!(!state.should_show());
        assert_eq!(state.current_step(), WizardStep::Complete);
        assert_eq!(state.completed_count(), 6);
    }

    #[test]
    fn test_show_hide() {
        let mut state = WizardState::new();
        assert!(state.should_show());

        state.hide();
        assert!(!state.should_show());

        state.show();
        assert!(state.should_show());
    }

    #[test]
    fn test_reset() {
        let mut state = WizardState::new();
        state.mark_step_completed(WizardStep::Welcome);
        state.mark_step_completed(WizardStep::Permissions);
        state.mark_completed();

        assert!(state.is_completed());
        assert_eq!(state.completed_count(), 6);

        state.reset();
        assert!(!state.is_completed());
        assert_eq!(state.completed_count(), 0);
        assert_eq!(state.current_step(), WizardStep::Welcome);
        assert!(state.should_show());
    }
}
