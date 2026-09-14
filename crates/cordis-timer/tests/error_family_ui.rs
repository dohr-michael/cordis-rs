//! Issue 57 compile evidence for Timer error families.

use trybuild::TestCases;
#[test]
fn issue57_timer_errors() {
    let t = TestCases::new();
    t.pass("tests/ui-error57/pass/*.rs");
}
