use cordis_core::{Context, Event, Routing};
struct MoveOnly(Box<()>);
struct E; impl Event for E { const NAME: &'static str="ui/issue33/parallel-clone"; type Args=MoveOnly; type Output=(); }
fn main(){ let ctx=Context::new(); let _=ctx.emit_parallel::<E>(Routing::Unscoped, MoveOnly(Box::new(()))); }
