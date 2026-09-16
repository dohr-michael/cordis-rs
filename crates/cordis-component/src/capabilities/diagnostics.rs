//! Host-owned diagnostics capability.

use std::future::Future;

use cordis_core::Level;
use wasmtime::component::{Accessor, HasData, Linker};

use crate::HostState;
use crate::bindings;

/// Marker for the standard `cordis:plugin/diagnostics` host capability.
///
/// The adapter binds it to a component generation's host-assigned logger
/// channel. Guests can choose severity and message, but never a channel or a
/// logger exporter.
pub(crate) struct Diagnostics;

impl HasData for Diagnostics {
    type Data<'a> = &'a mut HostState;
}

impl bindings::cordis::plugin::diagnostics::Host for HostState {}

impl bindings::cordis::plugin::diagnostics::HostWithStore<HostState> for Diagnostics {
    fn emit(
        host: &Accessor<HostState, Self>,
        level: bindings::cordis::plugin::diagnostics::Level,
        message: String,
    ) -> impl Future<Output = ()> + Send {
        let logger = host.with(|mut access| access.get().logger.clone());
        async move {
            let level = match level {
                bindings::cordis::plugin::diagnostics::Level::Debug => Level::Debug,
                bindings::cordis::plugin::diagnostics::Level::Info => Level::Info,
                bindings::cordis::plugin::diagnostics::Level::Warn => Level::Warn,
                bindings::cordis::plugin::diagnostics::Level::Error => Level::Error,
            };
            logger.log(level, message);
        }
    }
}

/// Add the diagnostics import implementation to a component linker.
pub(crate) fn add_to_linker(linker: &mut Linker<HostState>) -> wasmtime::Result<()> {
    crate::bindings::CordisPlugin::add_to_linker::<_, crate::Diagnostics>(linker, |state| state)
}
