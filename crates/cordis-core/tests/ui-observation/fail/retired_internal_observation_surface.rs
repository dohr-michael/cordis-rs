use cordis_core::event::{DispatchOutcomeKind, EventOperation};
use cordis_core::observation::RuntimeObservation;
use cordis_core::{Context, Event, Plugin};

fn require_event<E: Event>() {}
fn require_plugin<P: Plugin>() {}

fn main() {
    let _ctx = Context::new();
    let _ = cordis_core::internal::InternalPlugin;
    let _ = cordis_core::InternalDispatch;
    let _ = cordis_core::DispatchMode::Serial;
    require_event::<RuntimeObservation>();
    require_plugin::<RuntimeObservation>();
    let _ = EventOperation::Emit.to_string();
    let _ = DispatchOutcomeKind::Completed.to_string();
}
