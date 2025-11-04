/// Command executor
///
/// Executes commands and emits events on completion.

use crossbeam_channel::{unbounded, Sender, Receiver};
use std::thread;

use super::commands::{Command, CommandResult};
use super::events::Event;
use super::bus::EventBus;

/// Command executor that processes commands and emits events
pub struct CommandExecutor {
    command_tx: Sender<Command>,
    command_rx: Receiver<Command>,
    event_bus: EventBus,
}

impl CommandExecutor {
    /// Create a new command executor with an event bus
    pub fn new(event_bus: EventBus) -> Self {
        let (tx, rx) = unbounded();

        Self {
            command_tx: tx,
            command_rx: rx,
            event_bus,
        }
    }

    /// Get a sender for submitting commands
    pub fn sender(&self) -> Sender<Command> {
        self.command_tx.clone()
    }

    /// Execute a command immediately (blocking)
    pub fn execute_sync(&self, command: Command) -> CommandResult {
        log::info!("Executing command: {}", command.description());

        // This is a placeholder - actual execution will be implemented
        // in the respective modules (audio, detection, config, etc.)
        match command {
            Command::StopDetection => {
                self.event_bus.publish(Event::ProcessStateChanged {
                    old_state: crate::state::ProcessState::Running {
                        since: std::time::Instant::now(),
                    },
                    new_state: crate::state::ProcessState::Stopped,
                });
                CommandResult::Success
            }
            Command::SaveConfig => {
                // Config saving will be handled by config module
                CommandResult::Success
            }
            Command::Quit => {
                self.event_bus.publish(Event::Shutdown);
                CommandResult::Success
            }
            _ => {
                // Other commands will be implemented in future phases
                log::warn!("Command not yet implemented: {}", command.description());
                CommandResult::SuccessWithValue(
                    "Command queued but not yet implemented".to_string()
                )
            }
        }
    }

    /// Execute a command asynchronously
    pub fn execute(&self, command: Command) {
        let _ = self.command_tx.send(command);
    }

    /// Start the command processing loop in a background thread
    pub fn start_processing(&self) {
        let rx = self.command_rx.clone();
        let executor = Self {
            command_tx: self.command_tx.clone(),
            command_rx: self.command_rx.clone(),
            event_bus: self.event_bus.clone(),
        };

        thread::spawn(move || {
            log::info!("Command executor thread started");

            while let Ok(command) = rx.recv() {
                match &command {
                    Command::Quit => {
                        log::info!("Quit command received, stopping executor");
                        executor.execute_sync(command);
                        break;
                    }
                    _ => {
                        executor.execute_sync(command);
                    }
                }
            }

            log::info!("Command executor thread stopped");
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_command_executor_creation() {
        let bus = EventBus::new();
        let executor = CommandExecutor::new(bus);

        let sender = executor.sender();
        assert!(sender.send(Command::SaveConfig).is_ok());
    }

    #[test]
    fn test_command_executor_sync() {
        let bus = EventBus::new();
        let executor = CommandExecutor::new(bus);

        let result = executor.execute_sync(Command::SaveConfig);
        match result {
            CommandResult::Success => {},
            _ => panic!("Expected success result"),
        }
    }

    #[test]
    fn test_command_executor_async() {
        let bus = EventBus::new();
        let executor = CommandExecutor::new(bus);

        executor.execute(Command::SaveConfig);
        // Command is queued, not executed yet
    }

    #[test]
    fn test_quit_command_emits_shutdown_event() {
        let bus = EventBus::new();
        let (rx, _id) = bus.subscribe();

        let executor = CommandExecutor::new(bus);
        executor.execute_sync(Command::Quit);

        let event = rx.try_recv().unwrap();
        match event {
            Event::Shutdown => {},
            _ => panic!("Expected Shutdown event"),
        }
    }
}
