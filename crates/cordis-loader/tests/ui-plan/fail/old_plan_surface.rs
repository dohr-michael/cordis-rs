use cordis_loader::{Entry, EntryOptions, EntryTree};
fn main() { let _ = (std::mem::size_of::<Entry>(), std::mem::size_of::<EntryOptions>(), EntryTree::new()); }
