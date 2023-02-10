use std::any::TypeId;

use crate::registry::{entity_registry::EntityRegistry, event_registry::EventRegistry};

pub struct ECS<'a> {
  entities: EntityRegistry,
  events: EventRegistry<'a, Vec<TypeId>>,
}

impl ECS<'_> {

}

pub struct TickEvent<'a> {
  delta: f32,
  entity: EntityProxy<'a>,
}

pub struct EntityProxy<'a>(pub String, pub &'a mut EntityRegistry);

impl EntityProxy<'_> {
  pub fn get_component<T: 'static>(&self) -> Option<&T> {
    self.1.get_component(&self.0)
  }

  pub fn get_component_mut<T: 'static>(&mut self) -> Option<&mut T> {
    self.1.get_component_mut(&self.0)
  }
}