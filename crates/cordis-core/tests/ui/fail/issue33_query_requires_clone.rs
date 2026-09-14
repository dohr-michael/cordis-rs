use cordis_core::{Context, Event, Routing};
struct MoveOnly(Box<()>);
struct E; impl Event for E { const NAME: &'static str="ui/issue33/query-clone"; type Args=MoveOnly; type Output=(); }
fn main(){ let ctx=Context::new(); let _=ctx.query::<E>(Routing::Unscoped, MoveOnly(Box::new(()))); }
