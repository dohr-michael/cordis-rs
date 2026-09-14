//! Compile evidence for the typed configuration firewall.

use trybuild::TestCases;

#[test]
fn configuration_interface() {
    let tests = TestCases::new();
    tests.pass("tests/ui-configuration/pass/*.rs");
    tests.compile_fail("tests/ui-configuration/fail/*.rs");
}
