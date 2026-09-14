use cordis_core::{Context, Plugin};
use cordis_core::event::{Event, ListenerOptions, observer_sync};
use std::convert::Infallible;
struct P;
impl Plugin for P { type Config=(); type Input=(); type PrepareError=Infallible; type ApplyError=Infallible;
fn prepare(&self,_:())->Result<(),Infallible>{Ok(())}
async fn apply(&self,_:Context,_:&())->Result<(),Infallible>{Ok(())} }
struct E; impl Event for E { const NAME:&'static str="ui/update-observer"; type Args=(); type Output=(); }
fn main(){ let ctx=Context::new(); let listener=observer_sync(|_,_|Ok::<_,Infallible>(())); let _=ctx.on_update::<P,_>(listener,ListenerOptions::default()); }
