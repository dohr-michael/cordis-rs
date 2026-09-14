//! Issue 56 compile-time evidence that the retired List surface and compatibility replacements are gone (LG-07 / LK-07).

use trybuild::TestCases;

#[test]
fn list_interface() {
    let tests = TestCases::new();
    tests.compile_fail("tests/ui-list/fail/*.rs");
}
