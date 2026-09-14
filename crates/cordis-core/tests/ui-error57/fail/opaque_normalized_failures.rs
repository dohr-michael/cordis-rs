use cordis_core::{
    effect::EffectFailure,
    event::InvocationFailure,
    lifecycle::PluginFailure,
};

fn inspect_plugin(failure: &PluginFailure) {
    let _ = failure.downcast_ref::<std::io::Error>();
    let _ = PluginFailure::returned("boom".to_owned());
}
fn inspect_effect(failure: &EffectFailure) {
    let _ = failure.downcast_ref::<std::io::Error>();
    let _ = EffectFailure::returned("boom".to_owned());
}
fn inspect_invocation(failure: &InvocationFailure) {
    let _ = failure.downcast_ref::<std::io::Error>();
    let _ = InvocationFailure::returned("boom".to_owned());
}
fn main() {}
