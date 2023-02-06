use std::{any::{Any, TypeId}, collections::{HashSet, HashMap}, hash::Hash, sync::Arc};

use super::Registry;

pub struct EntityRegistry {
  component_registrations: HashMap<TypeId, HashSet<String>>,
  components: HashMap<(String, TypeId), Arc<dyn Any>>,
}

impl EntityRegistry {
  pub fn new() -> Self {
    Self {
      component_registrations: HashMap::new(),
      components: HashMap::new(),
    }
  }

  pub fn add_component<T: Eq + Hash + 'static>(&mut self, entity: &str, component: T) {
    let set = self.component_registrations.entry(TypeId::of::<T>()).or_insert(HashSet::new());
    set.insert(String::from(entity));

    self.components.insert((String::from(entity), TypeId::of::<T>()), Arc::new(component));
  }

  pub fn get_entities_by_component<T: Eq + Hash + 'static>(&self) -> Option<Vec<Arc<T>>> {
    let set = self.component_registrations.get(&TypeId::of::<T>())?;
    let vec: Vec<Arc<T>> = set.iter().map(|entity| {
      let component = self.components.get(&(entity.clone(), TypeId::of::<T>())).unwrap();
      component.downcast_ref::<Arc<T>>().unwrap().clone()
    }).collect();

    Some(vec)
  }

  pub fn get_entities_by_components<T>(&self, types: Vec<T>) -> Option<Vec<HashMap<TypeId, Arc<dyn Any>>>>  where T: Any + 'static {
    let mut set: HashSet<String> = HashSet::new();
    for type_id in &types {
      let type_id = TypeId::of::<T>();
      set = &set & self.component_registrations.get(&type_id)?;
    }

    Some(set.into_iter().map(|entity| {
      types.iter().map(|type_id| {
        let type_id = TypeId::of::<T>();
        let component = self.components.get(&(entity.clone(), type_id)).unwrap();
        (type_id, component.clone())
      }).collect()
    }).collect())
  }
}

#[cfg(test)]
mod test_entity_registry {
  use super::*;

  #[derive(Eq, Hash, PartialEq)]
  struct TestComponentOne { data: String, }

  #[derive(Eq, Hash, PartialEq)]
  struct TestComponentTwo { }

  #[derive(Eq, Hash, PartialEq)]
  struct TestComponentThree { }

  #[test]
  fn test_add_component() {
    let mut registry = EntityRegistry::new();
    
    registry.add_component("test_entity", TestComponentOne{ data: "test".to_string() });

    let data = registry.get_entities_by_component::<TestComponentOne>().unwrap()[0].data.clone();

    assert_eq!(data, "test".to_string());
  }
}