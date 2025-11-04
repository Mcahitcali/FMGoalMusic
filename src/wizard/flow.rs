/// Wizard flow management
///
/// Manages navigation through wizard steps.

use super::state::WizardState;
use super::steps::WizardStep;

/// Navigation result
#[derive(Debug, Clone, PartialEq)]
pub enum NavigationResult {
    /// Navigation succeeded, now on new step
    Success(WizardStep),

    /// Navigation blocked (at boundary or validation failed)
    Blocked { reason: String },

    /// Wizard completed
    Completed,
}

/// Wizard flow manager
pub struct WizardFlow {
    state: WizardState,
}

impl WizardFlow {
    /// Create a new wizard flow
    pub fn new() -> Self {
        Self {
            state: WizardState::new(),
        }
    }

    /// Create a flow from existing state
    pub fn from_state(state: WizardState) -> Self {
        Self { state }
    }

    /// Get current step
    pub fn current_step(&self) -> WizardStep {
        self.state.current_step()
    }

    /// Get wizard state
    pub fn state(&self) -> &WizardState {
        &self.state
    }

    /// Get mutable wizard state
    pub fn state_mut(&mut self) -> &mut WizardState {
        &mut self.state
    }

    /// Check if wizard is completed
    pub fn is_completed(&self) -> bool {
        self.state.is_completed()
    }

    /// Navigate to next step
    pub fn next(&mut self) -> NavigationResult {
        if self.state.is_completed() {
            return NavigationResult::Completed;
        }

        let current = self.current_step();

        // Mark current step as completed
        self.state.mark_step_completed(current);

        // Get next step
        match current.next() {
            Some(next_step) => {
                self.state.set_current_step(next_step);

                // Check if we reached the end
                if next_step == WizardStep::Complete {
                    self.state.mark_completed();
                    NavigationResult::Completed
                } else {
                    NavigationResult::Success(next_step)
                }
            }
            None => {
                // Already at last step
                self.state.mark_completed();
                NavigationResult::Completed
            }
        }
    }

    /// Navigate to previous step
    pub fn back(&mut self) -> NavigationResult {
        let current = self.current_step();

        match current.previous() {
            Some(prev_step) => {
                self.state.set_current_step(prev_step);
                NavigationResult::Success(prev_step)
            }
            None => NavigationResult::Blocked {
                reason: "Already at first step".to_string(),
            },
        }
    }

    /// Skip current step (if allowed)
    pub fn skip(&mut self) -> NavigationResult {
        let current = self.current_step();

        if !current.is_skippable() {
            return NavigationResult::Blocked {
                reason: format!("Cannot skip step: {}", current.title()),
            };
        }

        self.next()
    }

    /// Jump to a specific step
    pub fn go_to(&mut self, step: WizardStep) -> NavigationResult {
        self.state.set_current_step(step);
        NavigationResult::Success(step)
    }

    /// Complete wizard immediately
    pub fn complete(&mut self) {
        self.state.mark_completed();
    }

    /// Reset wizard to beginning
    pub fn reset(&mut self) {
        self.state.reset();
    }

    /// Check if can go back
    pub fn can_go_back(&self) -> bool {
        self.current_step().previous().is_some()
    }

    /// Check if can go forward
    pub fn can_go_forward(&self) -> bool {
        !self.is_completed() && self.current_step().next().is_some()
    }

    /// Check if current step can be skipped
    pub fn can_skip(&self) -> bool {
        self.current_step().is_skippable()
    }
}

impl Default for WizardFlow {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_flow() {
        let flow = WizardFlow::new();
        assert_eq!(flow.current_step(), WizardStep::Welcome);
        assert!(!flow.is_completed());
        assert!(!flow.can_go_back());
        assert!(flow.can_go_forward());
    }

    #[test]
    fn test_next_navigation() {
        let mut flow = WizardFlow::new();

        let result = flow.next();
        assert_eq!(result, NavigationResult::Success(WizardStep::Permissions));
        assert_eq!(flow.current_step(), WizardStep::Permissions);
        assert!(flow.state().is_step_completed(WizardStep::Welcome));
    }

    #[test]
    fn test_back_navigation() {
        let mut flow = WizardFlow::new();
        flow.next(); // Go to Permissions

        let result = flow.back();
        assert_eq!(result, NavigationResult::Success(WizardStep::Welcome));
        assert_eq!(flow.current_step(), WizardStep::Welcome);
    }

    #[test]
    fn test_back_at_first_step() {
        let mut flow = WizardFlow::new();

        let result = flow.back();
        matches!(result, NavigationResult::Blocked { .. });
    }

    #[test]
    fn test_skip_allowed() {
        let mut flow = WizardFlow::new();

        // Navigate to a skippable step (TeamSelection)
        flow.go_to(WizardStep::TeamSelection);

        assert!(flow.can_skip());
        let result = flow.skip();
        assert_eq!(result, NavigationResult::Success(WizardStep::AudioSetup));
    }

    #[test]
    fn test_skip_blocked() {
        let mut flow = WizardFlow::new();
        // Welcome is not skippable

        assert!(!flow.can_skip());
        let result = flow.skip();
        matches!(result, NavigationResult::Blocked { .. });
    }

    #[test]
    fn test_completion() {
        let mut flow = WizardFlow::new();

        // Navigate through all steps
        while flow.can_go_forward() {
            flow.next();
        }

        assert!(flow.is_completed());
        assert_eq!(flow.current_step(), WizardStep::Complete);
    }

    #[test]
    fn test_immediate_completion() {
        let mut flow = WizardFlow::new();
        assert!(!flow.is_completed());

        flow.complete();
        assert!(flow.is_completed());
    }

    #[test]
    fn test_reset() {
        let mut flow = WizardFlow::new();
        flow.next();
        flow.next();

        assert_ne!(flow.current_step(), WizardStep::Welcome);

        flow.reset();
        assert_eq!(flow.current_step(), WizardStep::Welcome);
        assert!(!flow.is_completed());
    }

    #[test]
    fn test_go_to() {
        let mut flow = WizardFlow::new();

        let result = flow.go_to(WizardStep::AudioSetup);
        assert_eq!(result, NavigationResult::Success(WizardStep::AudioSetup));
        assert_eq!(flow.current_step(), WizardStep::AudioSetup);
    }
}
