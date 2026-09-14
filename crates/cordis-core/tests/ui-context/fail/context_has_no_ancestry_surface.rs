use cordis_core::Context;

fn main() {
    let ctx = Context::new();
    let _ = ctx.parent();
    let _ = ctx.layers();
    let _ = ctx.ancestors();
}
