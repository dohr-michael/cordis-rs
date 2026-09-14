use cordis_core::{Context, Event};

struct Flow;
impl Event for Flow {
    const NAME: &'static str = "ui/issue34-old-waterfall";
    type Args = String;
    type Output = String;
}

fn main() {
    let ctx = Context::new();
    let _ = ctx.waterfall_bail::<Flow, Flow, _>(String::new(), |_| Ok(String::new()));
    let _ = ctx.waterfall_scoped::<Flow, _>(&(), String::new(), |value| async move { Ok::<_, std::convert::Infallible>(value) });
}
