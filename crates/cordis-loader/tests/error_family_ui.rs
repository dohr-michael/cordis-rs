//! Issue 57 compile evidence for Loader error families.

use trybuild::TestCases;
#[test]
fn issue57_loader_errors() {
    let t = TestCases::new();
    t.pass("tests/ui-error57/pass/*.rs");
    t.compile_fail("tests/ui-error57/fail/*.rs");
}
