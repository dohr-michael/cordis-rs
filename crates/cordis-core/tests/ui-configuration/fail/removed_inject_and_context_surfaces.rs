use cordis_core::{Context, InjectSpec};
use std::sync::Arc;

fn main() {
    let _ = InjectSpec::new();
    let _ = InjectSpec::none().requires(&["service"]);
    let _ = InjectSpec::none().require_with("service", Arc::new(()));
    let _ = InjectSpec::none().require_overriding("service", None);
    let _ = InjectSpec::none().iter();
    let _ = InjectSpec::none().is_empty();

    let ctx = Context::new();
    let _ = ctx.intercept("service");
    let _ = ctx.plugin_dyn;
    let _ = ctx.plugin;
}
