use std::{
  any::{Any, TypeId},
  collections::HashMap,
};

type EventCallback<'a, T> = Box<dyn FnMut(&T) + Send + Sync + 'a>;

struct EventSubscriptionArgs<'a, F>(Box<dyn FnMut(&dyn Any) + Send + Sync + 'a>, Option<F>);

#[derive(Default)]
pub struct EventRegistry<'a, F> {
  registry: HashMap<TypeId, Vec<EventSubscriptionArgs<'a, F>>>,
}

impl<'a, F> EventRegistry<'a, F>
where
  F: Default,
{
  pub fn new() -> Self {
    Self {
      ..Default::default()
    }
  }

  pub fn subscribe<T>(&mut self, callback: impl FnMut(&T) + Send + Sync + 'a)
  where
    T: 'a + Any + Send + Sync,
  {
    self.subscribe_with_filter(callback, None)
  }

  pub fn subscribe_with_filter<T>(
    &mut self,
    callback: impl FnMut(&T) + Send + Sync + 'a,
    filter: Option<F>,
  ) where
    T: 'a + Any + Send + Sync,
  {
    let type_id = TypeId::of::<T>();

    let mut boxed_callback = Box::new(callback) as EventCallback<'a, T>;
    let callback = Box::new(move |data: &dyn Any| {
      if let Some(data) = data.downcast_ref::<T>() {
        boxed_callback(data);
      }
    }) as Box<dyn FnMut(&dyn Any) + Send + Sync + 'a>;

    let callbacks = self.registry.entry(type_id).or_insert(Vec::new());
    callbacks.push(EventSubscriptionArgs(callback, filter));
  }

  pub fn invoke<T>(&mut self, event: &T)
  where
    T: 'a + Any + Send + Sync,
  {
    let type_id = TypeId::of::<T>();

    if let Some(callbacks) = self.registry.get_mut(&type_id) {
      for callback in callbacks {
        callback.0(event);
      }
    }
  }

  pub fn get_subscriptions<T>(&self) -> Option<&Vec<EventSubscriptionArgs<'a, F>>>
  where
    T: 'a + Any + Send + Sync,
  {
    let type_id = TypeId::of::<T>();

    self.registry.get(&type_id)
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
    let mut registry: EventRegistry<f32> = EventRegistry::new();

    let counter: Arc<Mutex<u32>> = Arc::new(Mutex::new(0));
    let c_ref = counter.clone();

    registry.subscribe(move |_: &MyEvent| {
      let mut v = c_ref.lock().unwrap();
      *v += 1;
    });

    registry.invoke(&MyEvent {});
    registry.invoke(&MyEvent {});

    assert_eq!(2, *counter.lock().unwrap());
  }

  #[test]
  fn test_filter_registry() {
    let mut registry: EventRegistry<Vec<TypeId>> = EventRegistry::new();

    registry.subscribe_with_filter::<i32>(|_| {}, Some(filter![i32, i64]));
    let subscriptions = registry.get_subscriptions::<i32>().unwrap();
    assert_eq!(subscriptions.len(), 1);

    let filter = subscriptions[0].1.as_ref().unwrap();
    assert_eq!(filter.len(), 2);
    assert_eq!(filter, &filter![i32, i64])
  }
}
