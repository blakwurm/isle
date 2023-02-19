use std::{collections::HashMap, any::{TypeId, Any}};
use isle_traits::event::Event;

pub struct EventSubscriptionArgs(Box<dyn FnMut(&dyn Event)>, Option<Vec<TypeId>>);
struct EventRegistry {
  subscribers: HashMap<TypeId, Vec<EventSubscriptionArgs>>,
}

impl EventRegistry {
  fn new() -> Self {
    Self {
      subscribers: HashMap::new(),
    }
  }

  fn subscribe<E>(&mut self, mut callback: impl FnMut(&E) + 'static)
  where
    E: Event + 'static,
  {
    self.subscribe_with_filter(callback, None);
  }

  fn subscribe_with_filter<E>(&mut self, mut callback: impl FnMut(&E) + 'static, filter: Option<Vec<TypeId>>)
  where
  E: Event + 'static
  {
    let subscribers = self
      .subscribers
      .entry(TypeId::of::<E>())
      .or_insert_with(Vec::new);

    let boxed_callback = Box::new(move |event: &dyn Event| {
      if let Some(event) = event.as_any().downcast_ref::<E>() {
        callback(event);
      }
    });

    subscribers.push(EventSubscriptionArgs(boxed_callback, filter));
  }

  fn invoke<E: 'static>(&mut self, event: E)
  where
    E: Event,
  {
    let event_ref = &event;
    if let Some(subscribers) = self.subscribers.get_mut(&TypeId::of::<E>()) {
      for subscriber in subscribers {
        subscriber.0(event_ref);
      }
    }
  }

  fn get_subscriptions<E: 'static>(&self) -> Option<&Vec<EventSubscriptionArgs>> {
    self.subscribers.get(&TypeId::of::<E>())
  }
}

#[cfg(test)]
mod event_registry_tests {
  use std::sync::{Arc, Mutex};

  use isle_macros::Event;

  use crate::filter;

use super::*;

  #[derive(Event)]
  struct MyEvent {}

  #[test]
  fn test_event_registry() {
    let mut registry: EventRegistry = EventRegistry::new();

    let counter: Arc<Mutex<u32>> = Arc::new(Mutex::new(0));
    let c_ref = counter.clone();

    registry.subscribe(move |_: &MyEvent| {
      let mut v = c_ref.lock().unwrap();
      *v += 1;
    });

    registry.invoke(MyEvent {});
    registry.invoke(MyEvent {});

    assert_eq!(2, *counter.lock().unwrap());
  }

  #[test]
  fn test_filter_registry() {
    let mut registry: EventRegistry = EventRegistry::new();

    registry.subscribe_with_filter::<MyEvent>(|_| {}, Some(filter![i32, i64]));
    let subscriptions = registry.get_subscriptions::<MyEvent>().unwrap();
    assert_eq!(subscriptions.len(), 1);

    let filter = subscriptions[0].1.as_ref().unwrap();
    assert_eq!(filter.len(), 2);
    assert_eq!(filter, &filter![i32, i64])
  }
}
