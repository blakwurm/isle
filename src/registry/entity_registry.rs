use std::{
  any::{Any, TypeId},
  collections::{HashMap, HashSet},
  hash::Hash, rc::Rc, cell::RefCell, sync::{Arc, Mutex, mpsc::Sender},
};

#[macro_export]
macro_rules! filter {
  [$($t:ty),*] => {
    vec![$(std::any::TypeId::of::<$t>()),*]
  }
}

pub trait StateQueue {
  fn stage<F>(&self, f: F)
  where
    F: FnOnce(&mut Self) + Send + 'static;
  fn commit(&mut self);
}

impl<T: Component> StateQueue for T {
  queue: Arc<Mutex<Sender<Box<dyn FnOnce(&mut Self) + Send>>>>,
  fn stage<F>(&self, f: F)
  where
    F: FnOnce(&mut Self) + Send + 'static,
  {
  }

  fn commit(&mut self) {
  }
}

trait Component: Any + Send + Sync {
  fn as_any(&self) -> &dyn Any;
  fn as_any_mut(&mut self) -> &mut dyn Any;
}


#[derive(Default)]
pub struct EntityRegistry {
  entities: HashMap<(String, TypeId), Box<dyn Component>>,
  components: HashMap<TypeId, HashSet<String>>,
}

impl EntityRegistry {
  pub fn new() -> Self {
    Self {
      ..Default::default()
    }
  }

  pub fn add_component<T: Component>(&mut self, entity: String, component: T) {
    let type_id = TypeId::of::<T>();
    let components = self.components.entry(type_id).or_insert(HashSet::new());
    components.insert(entity.clone());

    self.entities.insert((entity, type_id), Box::new(component));
  }

  pub fn get_component<T: Component>(&self, entity: &String) -> Option<&T> {
    let type_id = TypeId::of::<T>();

    let entity = self.entities.get(&(entity.clone(), type_id))?;
    entity.as_ref().downcast_ref::<T>()
  }

  pub fn get_component_mut<T: Component>(&mut self, entity: &String) -> Option<&mut T> {
    let type_id = TypeId::of::<T>();

    let entity = self.entities.get_mut(&(entity.clone(), type_id))?;
    entity.downcast_mut::<T>()
  }

  pub fn get_components<T: Component>(&self) -> Option<Vec<&T>> {
    let type_id = TypeId::of::<T>();

    let entities = self.components.get(&type_id)?;
    Some(entities.iter().map(|entity| self.get_component::<T>(entity).unwrap()).collect())
  }

  pub fn get_entities_by_component<T: Component>(&self) -> Option<Vec<&String>> {
    let type_id = TypeId::of::<T>();

    let entities = self.components.get(&type_id)?;
    Some(entities.iter().collect())
  }

  pub fn get_entities_by_components(&self, components: &Vec<TypeId>) -> Option<HashSet<String>> {
    let mut set: HashSet<String> = self
      .components
      .get(&components[0])?
      .iter()
      .cloned()
      .collect();

    for component in &components[1..] {
      set = &set & self.components.get(&component)?;
    }

    Some(set)
  }
}

#[cfg(test)]
mod entity_registry_tests {
  use super::*;

  #[test]
  fn test_component_add() {
    let mut registry = EntityRegistry::new();

    registry.add_component(String::from("test_entity"), 1);
    registry.add_component(String::from("test_entity"), 2_i64);

    assert_eq!(
      registry.get_component::<i32>(&String::from("test_entity")),
      Some(&1)
    );
    assert_eq!(
      registry.get_component::<i64>(&String::from("test_entity")),
      Some(&2_i64)
    );
  }

  #[test]
  fn test_get_component_mut() {
    let mut registry = EntityRegistry::new();

    registry.add_component(String::from("test_entity"), 1);
    registry.add_component(String::from("test_entity"), 2_i64);

    assert_eq!(
      registry.get_component::<i32>(&String::from("test_entity")),
      Some(&1)
    );
    assert_eq!(
      registry.get_component::<i64>(&String::from("test_entity")),
      Some(&2_i64)
    );

    let entity = String::from("test_entity");

    assert_eq!(registry.get_component::<i32>(&entity), Some(&1));
    assert_eq!(registry.get_component::<i64>(&entity), Some(&2_i64));

    *registry.get_component_mut::<i32>(&entity).unwrap() = 3;
    *registry.get_component_mut::<i64>(&entity).unwrap() = 4_i64;

    assert_eq!(registry.get_component::<i32>(&entity), Some(&3));
    assert_eq!(registry.get_component::<i64>(&entity), Some(&4_i64));
  }

  #[test]
  fn test_get_entities_by_component() {
    let mut registry = EntityRegistry::new();

    registry.add_component(String::from("test_entity_1"), 1);
    registry.add_component(String::from("test_entity_2"), 2_i64);

    assert_eq!(
      registry.get_entities_by_component::<i32>(),
      Some(vec!(&String::from("test_entity_1")))
    );
    assert_eq!(
      registry.get_entities_by_component::<i64>(),
      Some(vec!(&String::from("test_entity_2")))
    );
  }

  #[test]
  fn test_get_entities_by_components() {
    let mut registry = EntityRegistry::new();

    registry.add_component(String::from("test_entity_1"), 1);
    registry.add_component(String::from("test_entity_1"), 2_i64);
    registry.add_component(String::from("test_entity_2"), 3);
    registry.add_component(String::from("test_entity_2"), 4_i64);
    registry.add_component(String::from("test_entity_3"), 5);
    registry.add_component(String::from("test_entity_4"), 6_i64);

    let entities = registry
      .get_entities_by_components(&filter![i32, i64])
      .unwrap();
    assert!(entities.contains(&String::from("test_entity_1")));
    assert!(entities.contains(&String::from("test_entity_2")));
    assert!(!entities.contains(&String::from("test_entity_3")));
    assert!(!entities.contains(&String::from("test_entity_4")));
  }
}
