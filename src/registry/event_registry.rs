use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

type EventCallback<'a, T> = Box<dyn FnMut(&T) + Send + Sync + 'a>;

#[derive(Default)]
struct EventRegistry<'a> {
    registry: HashMap<TypeId, Vec<Box<dyn FnMut(&dyn Any) + Send + Sync + 'a>>>,
}

impl<'a> EventRegistry<'a> {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }

    pub fn subscribe<T>(&mut self, callback: impl FnMut(&T) + Send + Sync + 'a)
    where
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
        callbacks.push(callback);
    }

    pub fn invoke<T>(&mut self, event: &T)
    where
        T: 'a + Any + Send + Sync,
    {
        let type_id = TypeId::of::<T>();

        if let Some(callbacks) = self.registry.get_mut(&type_id) {
            for callback in callbacks {
                callback(event);
            }
        }
    }
}

#[cfg(test)]
mod event_registry_tests {
    use std::sync::{Arc, Mutex};

    use super::*;

    struct MyEvent {}

    #[test]
    fn test_event_registry() {
        let mut registry = EventRegistry::new();

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
}
