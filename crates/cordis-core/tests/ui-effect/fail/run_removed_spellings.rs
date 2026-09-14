// The old global-error spellings of `Context::run` are gone: the
// operation reports its own TaskRegistrationError, and the global bucket
// lost its task arm.

use cordis_core::{Context, CordisError, CordisErrorCode, Result};

fn main() {
    let ctx = Context::new();

    // The return type is no longer the global Result alias.
    let _: Result<()> = ctx.run(async {});

    let err = ctx.run(async {}).unwrap_err();
    // No global-code accessor on the operation error…
    let _ = err.error_code();
    // …and no conversion into the global bucket.
    let _: CordisError = err;
    // The old code for this refusal is gone too.
    let _ = CordisErrorCode::OffRuntime;
}
