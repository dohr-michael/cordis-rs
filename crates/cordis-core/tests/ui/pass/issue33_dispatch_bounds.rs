use cordis_core::event::{observer_sync, responder_sync};
use cordis_core::{Context, Event, Routing};
use std::cell::Cell;
use std::convert::Infallible;

struct NonCloneOutput(Cell<usize>);

struct Fanout;
impl Event for Fanout {
    const NAME: &'static str = "ui/issue33/fanout";
    type Args = String;
    type Output = NonCloneOutput;
}

struct MoveOnlyArgs(Box<usize>);
struct Owned;
impl Event for Owned {
    const NAME: &'static str = "ui/issue33/owned";
    type Args = MoveOnlyArgs;
    type Output = NonCloneOutput;
}

fn main() {
    let ctx = Context::new();
    let _ = ctx.on::<Fanout, _>(observer_sync(|_, _: String| Ok::<(), Infallible>(() )));
    let _ = ctx.on::<Fanout, _>(responder_sync(|_, _: String| {
        Ok::<_, Infallible>(Some(NonCloneOutput(Cell::new(1))))
    }));

    // The three fan-out/query primitives add Clone only to Args; Output stays move-only.
    let _ = ctx.emit::<Fanout>(Routing::Unscoped, String::new());
    let _ = ctx.emit_parallel::<Fanout>(Routing::Unscoped, String::new());
    let _ = ctx.query::<Fanout>(Routing::Unscoped, String::new());

    // Owned waterfall is the operation that transports a move-only Args value. Issue 34
    // owns its final runtime semantics; this fixture locks only the already-frozen bound.
    let owned = Context::new();
    let future = owned.waterfall::<Owned, _, _, Infallible>(
        Routing::Unscoped,
        MoveOnlyArgs(Box::new(7)),
        |args| async move { Ok(NonCloneOutput(Cell::new(*args.0))) },
    );
    drop(future);
    let output = NonCloneOutput(Cell::new(0));
    let _ = output.0;
}
