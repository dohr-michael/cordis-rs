use cordis_core::{Context, InjectSpec, Plugin};
use cordis_core::effect::CleanupResult;
use cordis_core::event::Event;
use std::{cell::Cell, error::Error, fmt, marker::PhantomData, rc::Rc};

#[derive(Debug)] struct E; impl fmt::Display for E { fn fmt(&self,f:&mut fmt::Formatter<'_>)->fmt::Result{f.write_str("e")} } impl Error for E {}
struct Config<'a> { _borrow: &'a Cell<u8>, _not_send_sync: Rc<()> }
struct Input { _not_clone_sync: Cell<u8> }
struct P;
impl Plugin for P {
    type Config = Config<'static>; type Input = Input; type PrepareError = E; type ApplyError = E;
    fn inject(&self)->InjectSpec{InjectSpec::none()}
    fn prepare(&self,_:Self::Config)->Result<Self::Input,E>{Ok(Input{_not_clone_sync:Cell::new(0)})}
    fn apply(&self,_:Context,_:&Self::Input)->impl std::future::Future<Output=Result<(),E>>+Send{async{Ok(())}}
}
struct Ev(PhantomData<Rc<()>>);
impl Event for Ev { const NAME:&'static str="facade58.bounds"; type Args=(); type Output=Cell<u8>; }
fn accepts_cleanup<F,Fut,R>(f:F) where F:FnOnce()->Fut+Send+'static, Fut:std::future::Future<Output=R>+Send, R:CleanupResult { drop(f); }
fn main(){ accepts_cleanup(|| async { () }); }
