use std::any::Any;

pub mod event;

pub trait StateQueue {
  fn stage<F>(&self, f: F)
  where
    F: FnOnce(&mut Self) + Send + 'static;
  fn commit(&mut self);
}

pub trait Anyable {
  fn as_any(&self) -> &dyn Any;
  fn as_any_mut(&mut self) -> &mut dyn Any;
}

impl dyn Anyable {
  fn downcast_ref<T: Anyable + 'static>(&self) -> Option<&T> {
    self.as_any().downcast_ref::<T>()
  }
  fn downcast_mut<T: Anyable + 'static>(&mut self) -> Option<&mut T> {
    self.as_any_mut().downcast_mut::<T>()
  }
}