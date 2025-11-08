use crossbeam_channel::{unbounded, Receiver, Sender};
use parking_lot::RwLock;
/// Event bus for pub/sub messaging
///
/// Allows modules to subscribe to events and broadcast events to all subscribers.
use std::sync::Arc;

use super::events::Event;

/// Subscriber ID for tracking subscriptions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SubscriberId(usize);

/// Event subscriber
struct Subscriber {
    id: SubscriberId,
    sender: Sender<Event>,
}

/// Event bus for broadcasting events to subscribers
pub struct EventBus {
    subscribers: Arc<RwLock<Vec<Subscriber>>>,
    next_id: Arc<RwLock<usize>>,
}

impl EventBus {
    /// Create a new event bus
    pub fn new() -> Self {
        Self {
            subscribers: Arc::new(RwLock::new(Vec::new())),
            next_id: Arc::new(RwLock::new(0)),
        }
    }

    /// Subscribe to events, returns a receiver and subscription ID
    pub fn subscribe(&self) -> (Receiver<Event>, SubscriberId) {
        let (tx, rx) = unbounded();

        let mut next_id = self.next_id.write();
        let id = SubscriberId(*next_id);
        *next_id += 1;
        drop(next_id);

        let subscriber = Subscriber { id, sender: tx };

        self.subscribers.write().push(subscriber);

        (rx, id)
    }

    /// Unsubscribe from events
    pub fn unsubscribe(&self, id: SubscriberId) {
        self.subscribers.write().retain(|s| s.id != id);
    }

    /// Publish an event to all subscribers
    pub fn publish(&self, event: Event) {
        let subscribers = self.subscribers.read();

        // Send to all subscribers (non-blocking)
        for subscriber in subscribers.iter() {
            // If send fails, subscriber channel is closed - that's ok
            let _ = subscriber.sender.try_send(event.clone());
        }
    }

    /// Get number of active subscribers
    pub fn subscriber_count(&self) -> usize {
        self.subscribers.read().len()
    }

    /// Clear all subscribers
    pub fn clear(&self) {
        self.subscribers.write().clear();
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for EventBus {
    fn clone(&self) -> Self {
        Self {
            subscribers: Arc::clone(&self.subscribers),
            next_id: Arc::clone(&self.next_id),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_event_bus_subscribe() {
        let bus = EventBus::new();
        let (_rx, _id) = bus.subscribe();
        assert_eq!(bus.subscriber_count(), 1);
    }

    #[test]
    fn test_event_bus_unsubscribe() {
        let bus = EventBus::new();
        let (_rx, id) = bus.subscribe();
        assert_eq!(bus.subscriber_count(), 1);

        bus.unsubscribe(id);
        assert_eq!(bus.subscriber_count(), 0);
    }

    #[test]
    fn test_event_bus_publish() {
        let bus = EventBus::new();
        let (rx, _id) = bus.subscribe();

        let event = Event::GoalDetected {
            team: None,
            timestamp: Instant::now(),
        };

        bus.publish(event.clone());

        let received = rx.try_recv().unwrap();
        match received {
            Event::GoalDetected { .. } => {}
            _ => panic!("Wrong event type received"),
        }
    }

    #[test]
    fn test_event_bus_multiple_subscribers() {
        let bus = EventBus::new();
        let (rx1, _id1) = bus.subscribe();
        let (rx2, _id2) = bus.subscribe();

        assert_eq!(bus.subscriber_count(), 2);

        let event = Event::Shutdown;
        bus.publish(event);

        assert!(rx1.try_recv().is_ok());
        assert!(rx2.try_recv().is_ok());
    }

    #[test]
    fn test_event_bus_clear() {
        let bus = EventBus::new();
        let (_rx1, _id1) = bus.subscribe();
        let (_rx2, _id2) = bus.subscribe();

        assert_eq!(bus.subscriber_count(), 2);

        bus.clear();
        assert_eq!(bus.subscriber_count(), 0);
    }

    #[test]
    fn test_event_bus_clone() {
        let bus1 = EventBus::new();
        let bus2 = bus1.clone();

        let (_rx, _id) = bus1.subscribe();
        assert_eq!(bus1.subscriber_count(), 1);
        assert_eq!(bus2.subscriber_count(), 1); // Shared state
    }
}
