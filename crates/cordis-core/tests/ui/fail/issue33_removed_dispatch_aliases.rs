use cordis_core::{Context, Event};
struct E; impl Event for E { const NAME: &'static str="ui/issue33/aliases"; type Args=(); type Output=(); }
fn main(){ let ctx=Context::new(); let _=ctx.parallel::<E>(()); let _=ctx.serial::<E>(()); let _=ctx.bail::<E>(()); }
