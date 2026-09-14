fn assert_debug<T: std::fmt::Debug>() {}
fn main() { assert_debug::<cordis_timer::Timeout<std::future::Ready<()>>>(); }
