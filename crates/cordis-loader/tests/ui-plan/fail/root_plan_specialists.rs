use cordis_loader::{IsolateEntry, PlanError, RealmPolicy};
fn main() { let _ = (std::mem::size_of::<IsolateEntry>(), std::mem::size_of::<PlanError>(), std::mem::size_of::<RealmPolicy>()); }
