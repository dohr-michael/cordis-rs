//! Standard host capabilities exposed to Cordis Components.

mod diagnostics;

pub(crate) use diagnostics::Diagnostics;
pub(crate) use diagnostics::add_to_linker;
