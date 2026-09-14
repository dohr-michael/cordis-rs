use cordis_loader::EntryId;
fn assert_ord<T: Ord>() {}
fn main() { assert_ord::<EntryId>(); }
