use cordis_core::{Context, Event, Routing};
struct MoveOnly(Box<()>);
struct E; impl Event for E { const NAME: &'static str="ui/issue33/emit-clone"; type Args=MoveOnly; type Output=(); }
fn main(){ let ctx=Context::new(); let _=ctx.emit::<E>(Routing::Unscoped, MoveOnly(Box::new(()))); }
