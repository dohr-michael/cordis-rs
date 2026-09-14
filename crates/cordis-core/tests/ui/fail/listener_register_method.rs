use cordis_core::{Context, Event};
use cordis_core::event::Listener;
struct Ping; impl Event for Ping { const NAME: &'static str="ui/methodless"; type Args=(); type Output=(); }
fn probe<L: Listener<Ping>>(listener:L, ctx:&Context){ let _ = listener.register(ctx, Default::default()); }
fn main(){}
