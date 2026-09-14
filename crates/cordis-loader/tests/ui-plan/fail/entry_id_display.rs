use cordis_loader::{EntryGroup, LoadPlanBuilder};
fn main() {
    let mut builder = LoadPlanBuilder::new();
    let id = builder.add_group(None, EntryGroup { name: "root".into() }).unwrap();
    let _ = format!("{id}");
}
