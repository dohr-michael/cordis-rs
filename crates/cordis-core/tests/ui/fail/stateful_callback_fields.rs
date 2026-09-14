use cordis_core::event::with_state;
fn main(){ let callback = with_state(|| 1usize, |_: cordis_core::Context, _: usize, _: ()| Ok::<(), std::convert::Infallible>(())); let _ = callback.factory; }
