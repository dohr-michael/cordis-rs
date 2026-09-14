use cordis_core::event::{around, mapper, mapper_sync, observer, observer_sync, responder, responder_sync, with_state};
use cordis_core::{Context, Event};
use std::convert::Infallible;
use std::rc::Rc;

struct Flow;
impl Event for Flow { const NAME: &'static str = "ui/stateful-flow"; type Args = String; type Output = String; }
struct LocalOnly(Rc<()>);

fn main() {
    let ctx = Context::new();
    let _ = ctx.on::<Flow, _>(observer_sync(with_state(|| LocalOnly(Rc::new(())), |_: Context, state: LocalOnly, _: String| { let _ = state.0; Ok::<(), Infallible>(()) })));
    let _ = ctx.on::<Flow, _>(observer(with_state(String::new, |_: Context, state: String, _: String| async move { drop(state); Ok::<(), Infallible>(()) })));
    let _ = ctx.on::<Flow, _>(responder_sync(with_state(String::new, |_: Context, state: String, _: String| Ok::<_, Infallible>(Some(state)))));
    let _ = ctx.on::<Flow, _>(responder(with_state(String::new, |_: Context, state: String, _: String| async move { Ok::<_, Infallible>(Some(state)) })));
    let _ = ctx.on::<Flow, _>(mapper_sync(with_state(String::new, |_: Context, mut state: String, value: String| { state.push_str(&value); Ok::<_, Infallible>(state) })));
    let _ = ctx.on::<Flow, _>(mapper(with_state(String::new, |_: Context, mut state: String, value: String| async move { state.push_str(&value); Ok::<_, Infallible>(state) })));
    let _ = ctx.on::<Flow, _>(around(with_state(String::new, |_: Context, state: String, value: String, next: cordis_core::event::Next<Flow>| async move { drop(state); next.call(value).await })));
}
