use std::{collections::HashMap, any::{TypeId, Any}, rc::Rc};

pub struct EventSubscriptionArgs(pub Box<dyn FnMut(&dyn Any)>, pub Option<Vec<TypeId>>);

impl EventSubscriptionArgs {
  pub fn invoke(&mut self, event: Rc<Box::<dyn Any>>) {
    (self.0)(event.as_ref());
  }
}

pub struct EventRegistry {
  subscribers: HashMap<TypeId, Vec<EventSubscriptionArgs>>,
}

impl EventRegistry {
  pub fn new() -> Self {
    Self {
      subscribers: HashMap::new(),
    }
  }

  pub fn subscribe<E>(&mut self, mut callback: impl FnMut(&E) + 'static)
  where
    E: 'static + Any,
  {
    self.subscribe_with_filter(callback, None);
  }

  pub fn subscribe_with_filter<E>(&mut self, mut callback: impl FnMut(&E) + 'static, filter: Option<Vec<TypeId>>)
  where
  E: 'static + Any
  {
    let subscribers = self
      .subscribers
      .entry(TypeId::of::<E>())
      .or_insert_with(Vec::new);

    let boxed_callback = Box::new(move |event: &dyn Any| {
      if let Some(event) = event.downcast_ref::<E>() {
        callback(event);
      }
    });

    subscribers.push(EventSubscriptionArgs(boxed_callback, filter));
  }

  pub fn invoke(&mut self, event: Box<dyn Any>)
  {
    let event_type_id = (*event).type_id();
    if let Some(subscribers) = self.subscribers.get_mut(&event_type_id) {
      let rc = Rc::new(event);
      for subscriber in subscribers {
        subscriber.invoke(rc.clone());
      }
    }
  }

  pub fn get_subscriptions<E: 'static>(&mut self) -> Option<&mut Vec<EventSubscriptionArgs>> {
    self.subscribers.get_mut(&TypeId::of::<E>())
  }
}

#[cfg(test)]
mod event_registry_tests {
  use std::sync::{Arc, Mutex};

  use crate::filter;

use super::*;

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

    registry.invoke(Box::new(MyEvent {}));
    registry.invoke(Box::new(MyEvent {}));

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
