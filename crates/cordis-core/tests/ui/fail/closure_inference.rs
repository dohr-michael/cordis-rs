use cordis_core::{Context, Event};
struct Ping; impl Event for Ping { const NAME: &'static str="ui/closure"; type Args=(); type Output=(); }
fn main(){ let ctx=Context::new(); let _=ctx.on::<Ping,_>(|_,_| {}); }
