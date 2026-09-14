use cordis_core::Context;
fn main() { let scope = Context::new().scope(); let _ = scope.with_child_scope(); }
