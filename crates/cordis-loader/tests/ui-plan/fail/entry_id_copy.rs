use cordis_loader::EntryId;
fn assert_copy<T: Copy>() {}
fn main() { assert_copy::<EntryId>(); }
