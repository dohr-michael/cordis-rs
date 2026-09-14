//! `Context::run` consumes the task's output inside the task: the output
//! carries no Send bound (only the future itself crosses to the
//! executor) and the framework retains nothing.

use cordis_core::Context;
use std::rc::Rc;

fn main() {
    // trybuild executes pass fixtures: `run` needs a current handle.
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let _entered = runtime.enter();

    let ctx = Context::new();

    // !Send output: the task resolves to a value only it may own.
    ctx.run(async move { Rc::new(()) }).expect("root admits");

    // !Send + !Clone output: consumed by the task, never retained.
    struct Precious(Rc<()>);
    ctx.run(async move { Precious(Rc::new(())) })
        .expect("root admits");
}
