use cordis_core::{Context, Event};
use cordis_core::event::Listener;
struct Ping; impl Event for Ping { const NAME: &'static str="ui/custom"; type Args=(); type Output=(); }
struct Mine;
impl Listener<Ping> for Mine {}
fn main(){ let _=Context::new(); }
