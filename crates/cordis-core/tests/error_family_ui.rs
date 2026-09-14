//! Issue 57 compile evidence for operation-specific error families.

use trybuild::TestCases;

#[test]
fn issue57_error_surface() {
    let t = TestCases::new();
    t.pass("tests/ui-error57/pass/*.rs");
    t.compile_fail("tests/ui-error57/fail/*.rs");
}
