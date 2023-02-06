use std::{any::{TypeId, Any}, sync::Arc, collections::HashMap, hash::Hash};

use super::Registry;

struct EventRegistry {
  listeners: HashMap<TypeId, Vec<Arc<dyn Fn(dyn Any)>>>,
}

impl EventRegistry {
  pub fn new() -> Self {
    Self {
      listeners: HashMap::new(),
    }
  }

  pub fn subscribe<T>(&mut self, listener: dyn Fn(T)) where T: Any + Eq + Hash + 'static {
    let vec = self.listeners.entry(TypeId::of::<T>()).or_insert(Vec::new());
    vec.push(Arc::new(listener));
  }
}