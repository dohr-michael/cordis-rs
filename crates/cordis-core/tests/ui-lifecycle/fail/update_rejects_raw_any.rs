use cordis_core::Fork;
use std::any::Any;
fn raw(fork: &Fork, value: Box<dyn Any + Send>) { let _ = fork.update(value); }
fn main() {}
