//! Compile evidence for the spawn surface: creation admits only a sealed
//! `PreparedPlugin`, and the normalized `PluginFailure` is opaque.

use trybuild::TestCases;

#[test]
fn spawn_interface() {
    let tests = TestCases::new();
    tests.pass("tests/ui-spawn/pass/*.rs");
    tests.compile_fail("tests/ui-spawn/fail/*.rs");
}
