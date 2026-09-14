// CleanupResult is sealed and covers exactly `()` and `Result<(), E>`:
// a foreign implementation is impossible, and other return types fail.

use cordis_core::Context;
use cordis_core::effect::CleanupResult;

struct Custom;

impl CleanupResult for Custom {}

fn main() {
    let ctx = Context::new();
    let _ = ctx.effect_sync(|| 7u32);
}
