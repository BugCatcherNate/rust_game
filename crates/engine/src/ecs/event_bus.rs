use std::any::{Any, TypeId};
use std::collections::HashMap;

/// Simple message bus that stores typed events until systems drain them.
///
/// Systems can publish any `'static + Send` type as an event. Consumers
/// request events of a specific type via [`drain`] which returns owned events
/// published since the last drain call for that type.
#[derive(Default)]
pub struct EventBus {
    events: HashMap<TypeId, Vec<Box<dyn Any + Send>>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            events: HashMap::new(),
        }
    }

    /// Publish a new event instance onto the bus.
    pub fn publish<E>(&mut self, event: E)
    where
        E: 'static + Send,
    {
        self.publish_boxed(Box::new(event));
    }

    pub fn publish_boxed(&mut self, event: Box<dyn Any + Send>) {
        let type_id = event.as_ref().type_id();
        self.events.entry(type_id).or_default().push(event);
    }

    /// Drain all pending events of the requested type.
    pub fn drain<E>(&mut self) -> Vec<E>
    where
        E: 'static + Send,
    {
        let events = self.events.remove(&TypeId::of::<E>()).unwrap_or_default();
        events
            .into_iter()
            .map(|event| {
                *event
                    .downcast::<E>()
                    .expect("type mismatch when downcasting queued events")
            })
            .collect()
    }

    /// Remove events that have not been consumed yet, regardless of type.
    pub fn clear(&mut self) {
        self.events.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq, Eq)]
    struct ExampleEvent(&'static str);

    #[test]
    fn publish_and_drain_by_type() {
        let mut bus = EventBus::new();
        bus.publish(ExampleEvent("first"));
        bus.publish(ExampleEvent("second"));

        let drained = bus.drain::<ExampleEvent>();
        assert_eq!(
            drained,
            vec![ExampleEvent("first"), ExampleEvent("second")]
        );
        assert!(bus.drain::<ExampleEvent>().is_empty());
    }

    #[test]
    fn events_of_different_types_are_isolated() {
        let mut bus = EventBus::new();
        bus.publish(ExampleEvent("only"));
        bus.publish(42u32);

        let strings = bus.drain::<ExampleEvent>();
        let numbers = bus.drain::<u32>();

        assert_eq!(strings, vec![ExampleEvent("only")]);
        assert_eq!(numbers, vec![42]);
    }
}
