use cordis_core::lifecycle::{PluginFailure, PluginFailureKind};

fn probe(failure: &PluginFailure) {
    // no public constructor
    let _ = PluginFailure::returned("boom".to_owned());
    // no original-error downcast or Any access
    let _ = failure.downcast_ref::<std::io::Error>();
    // kind is a value, not a type-level channel
    let _: PluginFailureKind = failure.kind();
}

// not Clone: a failure is delivered, never copied
fn clone_it(failure: &PluginFailure) -> PluginFailure {
    failure.clone()
}

fn main() {}
