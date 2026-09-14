use cordis_core::{FiberId, Fork};

fn same_fiber(left: &Fork, right: &Fork) -> bool {
    left.id() == right.id()
}

fn correlate(id: &FiberId) -> (FiberId, String) {
    (id.clone(), format!("{id:?}"))
}

fn main() {
    let _ = same_fiber as fn(&Fork, &Fork) -> bool;
    let _ = correlate as fn(&FiberId) -> (FiberId, String);
}
