//! EX-07 runnable migration evidence.

use std::process::{Command, Stdio};

#[test]
fn capstone_uses_only_final_v3_surface_and_self_terminates() {
    let source = include_str!("../src/main.rs");
    for stale in [
        ".plugin(",
        "waterfall_bail",
        ".serial::<",
        "InternalUpdate",
        "dyn Any",
        "scoped_",
        "live_fiber_count",
        "Registry",
        "EventCarrier",
        "CordisError",
        "emit_scoped",
        "query_scoped",
        "waterfall_scoped",
        "respawn",
    ] {
        assert!(!source.contains(stale), "stale surface remains: {stale}");
    }
    for required in [
        "Routing::",
        "waterfall_query",
        "on_update",
        "PreparedChange",
        ".era_swap(",
        ".id()",
        ".dispose().await",
        "responder",
        "mapper",
        "around",
        "observer_sync",
        "with_state",
    ] {
        assert!(
            source.contains(required),
            "missing final-v3 evidence: {required}"
        );
    }

    let output = Command::new(env!("CARGO_BIN_EXE_chat_capstone"))
        .stdin(Stdio::null())
        .output()
        .expect("run chat_capstone");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    for evidence in [
        "routing: scoped command reached registration context",
        "waterfall/query: mapped and answered",
        "update policy: target-scoped and same FiberId preserved",
        "registration context: callback kept registration Scope",
        "era replacement: new FiberId observed",
        "dependent convergence: chat active on successor",
        "notice: logout",
        "logout: ordinary dispose reached pending",
        "frontend one: terminated cleanly",
        "frontend two: terminated cleanly",
        "chat capstone complete",
    ] {
        assert!(
            stdout.contains(evidence),
            "missing evidence `{evidence}`\nstdout:\n{stdout}"
        );
    }
}
