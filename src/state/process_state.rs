/// Detection process state machine
///
/// Represents the lifecycle of the detection process with clear state transitions.

use std::time::Instant;

/// State of the detection process
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ProcessState {
    /// Detection is not running
    Stopped,

    /// Detection is starting (transitional state)
    Starting,

    /// Detection is actively running
    Running { since: Instant },

    /// Detection is stopping (transitional state)
    Stopping,
}

impl ProcessState {
    /// Check if detection is stopped
    pub fn is_stopped(&self) -> bool {
        matches!(self, ProcessState::Stopped)
    }

    /// Check if detection is running
    pub fn is_running(&self) -> bool {
        matches!(self, ProcessState::Running { .. })
    }

    /// Check if detection is in a transitional state
    pub fn is_transitioning(&self) -> bool {
        matches!(self, ProcessState::Starting | ProcessState::Stopping)
    }

    /// Get the time since detection started (if running)
    pub fn running_duration(&self) -> Option<std::time::Duration> {
        match self {
            ProcessState::Running { since } => Some(since.elapsed()),
            _ => None,
        }
    }

    /// Get a human-readable description of the state
    pub fn description(&self) -> &'static str {
        match self {
            ProcessState::Stopped => "Stopped",
            ProcessState::Starting => "Starting...",
            ProcessState::Running { .. } => "Running",
            ProcessState::Stopping => "Stopping...",
        }
    }
}

impl Default for ProcessState {
    fn default() -> Self {
        ProcessState::Stopped
    }
}

/// State transition results
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransitionError {
    /// Cannot start when already running
    AlreadyRunning,

    /// Cannot stop when already stopped
    AlreadyStopped,

    /// Cannot perform this action during a transition
    InTransition,
}

impl std::fmt::Display for TransitionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransitionError::AlreadyRunning => write!(f, "Detection is already running"),
            TransitionError::AlreadyStopped => write!(f, "Detection is already stopped"),
            TransitionError::InTransition => {
                write!(f, "Cannot perform action during state transition")
            }
        }
    }
}

impl std::error::Error for TransitionError {}

/// State machine for process transitions
pub struct ProcessStateMachine {
    state: ProcessState,
}

impl ProcessStateMachine {
    /// Create a new state machine in the Stopped state
    pub fn new() -> Self {
        Self {
            state: ProcessState::Stopped,
        }
    }

    /// Get the current state
    pub fn state(&self) -> ProcessState {
        self.state
    }

    /// Transition to Starting state
    pub fn start(&mut self) -> Result<(), TransitionError> {
        match self.state {
            ProcessState::Stopped => {
                self.state = ProcessState::Starting;
                Ok(())
            }
            ProcessState::Running { .. } => Err(TransitionError::AlreadyRunning),
            _ => Err(TransitionError::InTransition),
        }
    }

    /// Transition from Starting to Running
    pub fn mark_running(&mut self) -> Result<(), TransitionError> {
        match self.state {
            ProcessState::Starting => {
                self.state = ProcessState::Running {
                    since: Instant::now(),
                };
                Ok(())
            }
            _ => Err(TransitionError::InTransition),
        }
    }

    /// Transition to Stopping state
    pub fn stop(&mut self) -> Result<(), TransitionError> {
        match self.state {
            ProcessState::Running { .. } => {
                self.state = ProcessState::Stopping;
                Ok(())
            }
            ProcessState::Stopped => Err(TransitionError::AlreadyStopped),
            _ => Err(TransitionError::InTransition),
        }
    }

    /// Transition from Stopping to Stopped
    pub fn mark_stopped(&mut self) -> Result<(), TransitionError> {
        match self.state {
            ProcessState::Stopping => {
                self.state = ProcessState::Stopped;
                Ok(())
            }
            _ => Err(TransitionError::InTransition),
        }
    }

    /// Force stop (for error recovery)
    pub fn force_stop(&mut self) {
        self.state = ProcessState::Stopped;
    }
}

impl Default for ProcessStateMachine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_state_predicates() {
        let stopped = ProcessState::Stopped;
        assert!(stopped.is_stopped());
        assert!(!stopped.is_running());
        assert!(!stopped.is_transitioning());

        let running = ProcessState::Running {
            since: Instant::now(),
        };
        assert!(!running.is_stopped());
        assert!(running.is_running());
        assert!(!running.is_transitioning());

        let starting = ProcessState::Starting;
        assert!(!starting.is_stopped());
        assert!(!starting.is_running());
        assert!(starting.is_transitioning());
    }

    #[test]
    fn test_state_machine_transitions() {
        let mut sm = ProcessStateMachine::new();
        assert_eq!(sm.state(), ProcessState::Stopped);

        // Start transition
        assert!(sm.start().is_ok());
        assert_eq!(sm.state(), ProcessState::Starting);

        // Cannot start again while starting
        assert!(sm.start().is_err());

        // Mark running
        assert!(sm.mark_running().is_ok());
        assert!(sm.state().is_running());

        // Cannot start while running
        assert!(sm.start().is_err());

        // Stop transition
        assert!(sm.stop().is_ok());
        assert_eq!(sm.state(), ProcessState::Stopping);

        // Mark stopped
        assert!(sm.mark_stopped().is_ok());
        assert_eq!(sm.state(), ProcessState::Stopped);
    }

    #[test]
    fn test_force_stop() {
        let mut sm = ProcessStateMachine::new();
        sm.start().unwrap();

        // Force stop from Starting state
        sm.force_stop();
        assert_eq!(sm.state(), ProcessState::Stopped);
    }
}
