/// Messaging module for Event/Command architecture
///
/// This module implements the Event/Command segregation pattern:
/// - **Events**: Notifications of things that happened (past tense, broadcast)
/// - **Commands**: Requests to perform actions (imperative, targeted)
///
/// ## Architecture
///
/// ```text
/// ┌─────────┐     Command      ┌──────────┐     Event      ┌─────────────┐
/// │ Module  │ ───────────────> │ Executor │ ─────────────> │  Event Bus  │
/// │  (GUI)  │                  │          │                │             │
/// └─────────┘                  └──────────┘                └─────────────┘
///                                                                 │
///                                                                 │ Publishes
///                                                                 ▼
///                                                           ┌──────────┐
///                                                           │ Handlers │
///                                                           │  (Audio, │
///                                                           │   etc.)  │
///                                                           └──────────┘
/// ```
///
/// ## Usage
///
/// ```rust,ignore
/// // Create event bus
/// let event_bus = EventBus::new();
///
/// // Subscribe to events
/// let (rx, _id) = event_bus.subscribe();
///
/// // Create command executor
/// let executor = CommandExecutor::new(event_bus.clone());
/// executor.start_processing();
///
/// // Execute commands
/// executor.execute(Command::StartDetection { ... });
///
/// // Handle events
/// while let Ok(event) = rx.recv() {
///     match event {
///         Event::GoalDetected { .. } => { /* handle goal */ },
///         _ => {}
///     }
/// }
/// ```

pub mod events;
pub mod commands;
pub mod bus;
pub mod executor;

// Re-export commonly used types
pub use events::{Event, ConfigField, AudioSource};
pub use commands::{Command, CommandResult, ConfigUpdate, AudioSourceType, CrowdCheerVariant};
pub use bus::{EventBus, SubscriberId};
pub use executor::CommandExecutor;
