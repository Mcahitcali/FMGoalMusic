/// Message handlers for MVU pattern
///
/// Handles all GUI messages and updates the model accordingly.
use super::messages::Message;
use super::model::Model;

/// Handle a message and update the model
///
/// Returns a vector of messages to process next (for chaining actions)
pub fn update(model: &mut Model, message: Message) -> Vec<Message> {
    tracing::debug!("Handling message: {}", message.description());

    match message {
        Message::TabChanged(tab) => {
            model.set_active_tab(tab);
            vec![]
        }

        Message::None => vec![],

        // Placeholder for other messages - will be implemented as we extract views
        _ => {
            tracing::warn!(
                "Message not yet handled in new MVU architecture: {}",
                message.description()
            );
            vec![]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::state::AppTab;
    use super::*;
    use crate::state::AppState;
    use parking_lot::Mutex;
    use std::sync::Arc;

    #[test]
    fn test_tab_changed() {
        let state = Arc::new(Mutex::new(AppState::default()));
        let mut model = Model::new(state);

        let messages = update(&mut model, Message::TabChanged(AppTab::Settings));
        assert_eq!(model.active_tab(), AppTab::Settings);
        assert!(messages.is_empty());
    }

    #[test]
    fn test_none_message() {
        let state = Arc::new(Mutex::new(AppState::default()));
        let mut model = Model::new(state);

        let messages = update(&mut model, Message::None);
        assert!(messages.is_empty());
    }
}
