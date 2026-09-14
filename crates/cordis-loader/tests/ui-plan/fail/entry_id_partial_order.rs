use cordis_loader::EntryId;
fn assert_order<T: PartialOrd>() {}
fn main() { assert_order::<EntryId>(); }
