use cordis_core::{Context, Event};
struct E; impl Event for E { const NAME: &'static str="ui/issue33/scoped-aliases"; type Args=(); type Output=(); }
fn main(){ let ctx=Context::new(); let scope=ctx.scope(); let _=ctx.parallel_scoped::<E>(&scope, ()); let _=ctx.serial_scoped::<E>(&scope, ()); let _=ctx.bail_scoped::<E>(&scope, ()); }
