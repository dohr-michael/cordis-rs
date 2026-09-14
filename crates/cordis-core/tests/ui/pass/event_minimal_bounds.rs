use cordis_core::event::{mapper_sync, observer_sync, with_state};
use cordis_core::{Context, Event};
use std::cell::Cell;
use std::convert::Infallible;
use std::rc::Rc;

struct Marker(Rc<()>);
struct MoveOnly(Cell<usize>);
impl Event for Marker {
    const NAME: &'static str = "ui/minimal-bounds";
    type Args = Cell<usize>;
    type Output = MoveOnly;
}
struct LocalState(Rc<()>);
fn main() {
    let ctx = Context::new();
    let _ = ctx.on::<Marker, _>(observer_sync(with_state(
        || LocalState(Rc::new(())),
        |_: Context, state: LocalState, _: Cell<usize>| { let _ = state.0; Ok::<(), Infallible>(()) },
    )));
    let _ = ctx.on::<Marker, _>(mapper_sync(|_: Context, value: Cell<usize>| Ok::<_, Infallible>(value)));
    let _ = MoveOnly(Cell::new(0));
}

struct OwnedFlow;
struct OwnedArg(String);
impl Event for OwnedFlow {
    const NAME: &'static str = "ui/owned-waterfall";
    type Args = OwnedArg;
    type Output = usize;
}

async fn owned_waterfall_accepts_move_only_args(ctx: Context) {
    let _ = ctx
        .waterfall::<OwnedFlow, _, _, Infallible>(
            cordis_core::Routing::Unscoped,
            OwnedArg(String::from("owned")),
            |arg| async move { Ok(arg.0.len()) },
        )
        .await;
}
