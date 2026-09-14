use cordis_core::event::{around, mapper, mapper_sync, observer, observer_sync, responder, responder_sync};
use cordis_core::{Context, Event};
use std::convert::Infallible;

struct Flow;
impl Event for Flow { const NAME: &'static str = "ui/flow"; type Args = String; type Output = String; }

fn main() {
    let ctx = Context::new();
    let _ = ctx.on::<Flow, _>(observer(|_: Context, _: String| async { Ok::<(), Infallible>(()) }));
    let _ = ctx.on::<Flow, _>(observer_sync(|_: Context, _: String| Ok::<(), Infallible>(() )));
    let _ = ctx.on::<Flow, _>(responder(|_: Context, value: String| async move { Ok::<_, Infallible>(Some(value)) }));
    let _ = ctx.on::<Flow, _>(responder_sync(|_: Context, value: String| Ok::<_, Infallible>(Some(value))));
    let _ = ctx.on::<Flow, _>(mapper(|_: Context, value: String| async move { Ok::<_, Infallible>(value) }));
    let _ = ctx.on::<Flow, _>(mapper_sync(|_: Context, value: String| Ok::<_, Infallible>(value)));
    let _ = ctx.on::<Flow, _>(around(|_: Context, value: String, next: cordis_core::event::Next<Flow>| async move { next.call(value).await }));
}
