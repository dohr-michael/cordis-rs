//! Issue 44 Logger public-interface compile evidence.

use trybuild::TestCases;

#[test]
fn logger_ui() {
    let t = TestCases::new();
    t.pass("tests/ui-logger/pass/*.rs");
    t.compile_fail("tests/ui-logger/fail/*.rs");
}
