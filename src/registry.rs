use std::{any::{TypeId}, collections::{HashMap, HashSet}, hash::Hash};

pub mod entity_registry;
pub mod event_registry;
pub mod system_registry;

pub struct Registry<T: Eq + Hash> {
  registry: HashMap<TypeId, HashSet<Box<T>>>,
}

impl<T: Eq + Hash> Registry<T> {
  pub fn new() -> Self {
    Self {
      registry: HashMap::new() 
    }
  }

  pub fn register<V: 'static>(&mut self, value: Box<T>) {
    let set = self.registry.entry(TypeId::of::<V>()).or_insert(HashSet::new());
    set.insert(value);
  }

  pub fn get<V: 'static>(&self) -> Option<&HashSet<Box<T>>> {
    self.registry.get(&TypeId::of::<V>())
  }
}