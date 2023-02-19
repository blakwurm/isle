pub trait Event {
  fn as_any(&self) -> &dyn std::any::Any;
}