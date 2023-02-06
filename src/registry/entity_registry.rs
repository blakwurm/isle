use std::{any::{Any, TypeId}, collections::{HashSet, HashMap}, hash::Hash};

use super::Registry;

pub trait Component: Eq + Hash {}

pub struct EntityRegistry {
  component_registrations: Registry<String>,
  components: HashMap<(String, TypeId), Box<dyn Any>>,
}

impl EntityRegistry {
  pub fn new() -> Self {
    Self {
      component_registrations: Registry::new(),
      components: HashMap::new(),
    }
  }

  pub fn add_component<T>(&mut self, entity: &str, component: T) where T: Component + 'static {
    self.components.insert((String::from(entity), TypeId::of::<T>()), Box::new(component));
    self.component_registrations.register::<T>(Box::new(String::from(entity)));
  }

  pub fn get_entities_by_component<T>(&self) -> Option<Vec<Box<T>>> where T: Component + 'static {
    let set = self.component_registrations.get::<T>()?;

    Some(set.iter().map(|id| *self.components.get(&(String::from(*id.clone()), TypeId::of::<T>())).unwrap().downcast::<Box<T>>().unwrap()).collect())
  }
}