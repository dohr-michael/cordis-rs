// ADR 0037's one-canonical-path rule: `ConfigResolutionError` lives in the
// `service` module only; the crate root re-export whitelist does not carry
// it.
use cordis_core::ConfigResolutionError;

fn main() {}
