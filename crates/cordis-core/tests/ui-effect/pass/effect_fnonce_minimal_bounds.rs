//! The frozen effect surface: `FnOnce + Send` cleanups (never `Fn` or
//! `Sync`), `CleanupResult` covering `()` and `Result<(), E>`, and a
//! move-only registration.

use cordis_core::Context;
use cordis_core::effect::EffectRegistration;
use std::cell::Cell;
use std::fmt;
use std::rc::Rc;

/// `!Send + !Sync`: `CleanupResult` imposes no bounds beyond `Error`.
#[derive(Debug)]
struct LocalError(Rc<()>);

impl fmt::Display for LocalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let _ = &self.0;
        f.write_str("local")
    }
}

impl std::error::Error for LocalError {}

fn main() {
    let ctx = Context::new();

    // FnOnce-only: the cleanup moves a non-Clone capture out of itself.
    let owned = String::from("resource");
    let registration: EffectRegistration = ctx
        .effect_sync(move || drop(owned))
        .expect("root admits");

    // Move-only manual control: each operation consumes the registration.
    assert!(registration.disarm());

    // CleanupResult adapts `Result<(), E>` with no Send/Sync on E.
    let registration = ctx
        .effect_sync(|| -> Result<(), LocalError> {
            Err(LocalError(Rc::new(())))
        })
        .expect("root admits");
    let _pending = registration.dispose();

    // The closure captures a `Cell`: Send but not Sync.
    let cell = Cell::new(0u8);
    ctx.effect_sync(move || {
        cell.set(1);
    })
    .expect("no Sync bound");

    // Async cleanup: a Send future returned by an FnOnce closure.
    let ticket = String::from("ticket");
    ctx.effect(move || async move {
        drop(ticket);
    })
    .expect("async form");
}
