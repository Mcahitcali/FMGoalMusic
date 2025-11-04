/// Wizard step definitions
///
/// Defines all steps in the first-run wizard flow.

/// Wizard step
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WizardStep {
    /// Welcome screen - Introduction to the application
    Welcome,

    /// Permission requests - Screen recording permission (macOS)
    Permissions,

    /// Region setup - Guide user through capture region selection
    RegionSetup,

    /// Team selection - Help user select their team
    TeamSelection,

    /// Audio setup - Configure and test audio
    AudioSetup,

    /// Complete - Finish setup
    Complete,
}

impl WizardStep {
    /// Get step title
    pub fn title(&self) -> &'static str {
        match self {
            WizardStep::Welcome => "Welcome to FM Goal Musics",
            WizardStep::Permissions => "Permissions",
            WizardStep::RegionSetup => "Screen Region Setup",
            WizardStep::TeamSelection => "Select Your Team",
            WizardStep::AudioSetup => "Audio Setup",
            WizardStep::Complete => "Setup Complete",
        }
    }

    /// Get step description
    pub fn description(&self) -> &'static str {
        match self {
            WizardStep::Welcome => "Let's get you set up to enjoy goal music in Football Manager!",
            WizardStep::Permissions => "Grant necessary permissions for screen capture",
            WizardStep::RegionSetup => "Select the screen region where goal notifications appear",
            WizardStep::TeamSelection => "Choose your team to play custom music when they score",
            WizardStep::AudioSetup => "Add music files and test audio playback",
            WizardStep::Complete => "You're all set! Enjoy your personalized goal music.",
        }
    }

    /// Get step number (1-indexed)
    pub fn number(&self) -> usize {
        match self {
            WizardStep::Welcome => 1,
            WizardStep::Permissions => 2,
            WizardStep::RegionSetup => 3,
            WizardStep::TeamSelection => 4,
            WizardStep::AudioSetup => 5,
            WizardStep::Complete => 6,
        }
    }

    /// Get total number of steps
    pub fn total_steps() -> usize {
        6
    }

    /// Check if this is the first step
    pub fn is_first(&self) -> bool {
        matches!(self, WizardStep::Welcome)
    }

    /// Check if this is the last step
    pub fn is_last(&self) -> bool {
        matches!(self, WizardStep::Complete)
    }

    /// Get next step
    pub fn next(&self) -> Option<WizardStep> {
        match self {
            WizardStep::Welcome => Some(WizardStep::Permissions),
            WizardStep::Permissions => Some(WizardStep::RegionSetup),
            WizardStep::RegionSetup => Some(WizardStep::TeamSelection),
            WizardStep::TeamSelection => Some(WizardStep::AudioSetup),
            WizardStep::AudioSetup => Some(WizardStep::Complete),
            WizardStep::Complete => None,
        }
    }

    /// Get previous step
    pub fn previous(&self) -> Option<WizardStep> {
        match self {
            WizardStep::Welcome => None,
            WizardStep::Permissions => Some(WizardStep::Welcome),
            WizardStep::RegionSetup => Some(WizardStep::Permissions),
            WizardStep::TeamSelection => Some(WizardStep::RegionSetup),
            WizardStep::AudioSetup => Some(WizardStep::TeamSelection),
            WizardStep::Complete => Some(WizardStep::AudioSetup),
        }
    }

    /// Check if this step can be skipped
    pub fn is_skippable(&self) -> bool {
        match self {
            WizardStep::Welcome => false,        // Must see welcome
            WizardStep::Permissions => false,    // Must handle permissions
            WizardStep::RegionSetup => false,    // Required for functionality
            WizardStep::TeamSelection => true,   // Can be set later
            WizardStep::AudioSetup => true,      // Can be set later
            WizardStep::Complete => false,       // Final step
        }
    }

    /// Get all steps in order
    pub fn all_steps() -> Vec<WizardStep> {
        vec![
            WizardStep::Welcome,
            WizardStep::Permissions,
            WizardStep::RegionSetup,
            WizardStep::TeamSelection,
            WizardStep::AudioSetup,
            WizardStep::Complete,
        ]
    }
}

impl Default for WizardStep {
    fn default() -> Self {
        WizardStep::Welcome
    }
}

impl std::fmt::Display for WizardStep {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.title())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_step_navigation() {
        let step = WizardStep::Welcome;
        assert!(step.is_first());
        assert!(!step.is_last());

        let next = step.next().unwrap();
        assert_eq!(next, WizardStep::Permissions);

        let complete = WizardStep::Complete;
        assert!(complete.is_last());
        assert!(complete.next().is_none());
    }

    #[test]
    fn test_step_numbers() {
        assert_eq!(WizardStep::Welcome.number(), 1);
        assert_eq!(WizardStep::Complete.number(), 6);
        assert_eq!(WizardStep::total_steps(), 6);
    }

    #[test]
    fn test_skippable_steps() {
        assert!(!WizardStep::Welcome.is_skippable());
        assert!(!WizardStep::RegionSetup.is_skippable());
        assert!(WizardStep::TeamSelection.is_skippable());
        assert!(WizardStep::AudioSetup.is_skippable());
    }

    #[test]
    fn test_all_steps() {
        let steps = WizardStep::all_steps();
        assert_eq!(steps.len(), 6);
        assert_eq!(steps[0], WizardStep::Welcome);
        assert_eq!(steps[5], WizardStep::Complete);
    }

    #[test]
    fn test_previous_navigation() {
        let step = WizardStep::Permissions;
        assert_eq!(step.previous(), Some(WizardStep::Welcome));

        let first = WizardStep::Welcome;
        assert_eq!(first.previous(), None);
    }
}
