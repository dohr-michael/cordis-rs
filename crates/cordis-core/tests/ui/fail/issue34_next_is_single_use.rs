use cordis_core::event::{Next, around};
use cordis_core::{Context, Event};

struct Flow;
impl Event for Flow {
    const NAME: &'static str = "ui/issue34-next-single-use";
    type Args = String;
    type Output = String;
}

fn main() {
    let ctx = Context::new();
    let _ = ctx.on::<Flow, _>(around(|_, value: String, next: Next<Flow>| async move {
        let first = next.call(value.clone()).await?;
        let second = next.call(value).await?;
        Ok::<_, cordis_core::event::InvocationFailure>(format!("{first}{second}"))
    }));
}
