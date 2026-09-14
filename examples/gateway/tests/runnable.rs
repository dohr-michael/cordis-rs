//! EX-03 executable evidence for the final v3 gateway consumer.

use std::fs;
use std::process::{Command, Stdio};

#[test]
fn gateway_uses_only_final_v3_facades() {
    let source = fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/main.rs"))
        .expect("gateway source should be readable");
    for stale in [
        "DynPlugin",
        "EntryTree",
        "by_resolve_key",
        "internal::",
        "waterfall_bail",
    ] {
        assert!(!source.contains(stale), "stale facade `{stale}` remains");
    }
    for required in [
        "LoadPlanBuilder",
        "prepare_plugin_json",
        "PreparedChange",
        "on_update",
        "Routing::Scoped",
        "waterfall_query",
        ".timeout(",
        "TimeoutOutcome::Completed",
        "TimeoutOutcome::Elapsed",
    ] {
        assert!(
            source.contains(required),
            "final facade `{required}` is not demonstrated"
        );
    }
}

#[test]
fn gateway_runbook_is_scripted_self_terminating_and_discriminating() {
    let output = Command::new(env!("CARGO_BIN_EXE_gateway"))
        .stdin(Stdio::null())
        .output()
        .expect("gateway should launch");
    assert!(
        output.status.success(),
        "gateway exited unsuccessfully:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    let stdout = String::from_utf8(output.stdout).expect("gateway output is UTF-8");
    for evidence in [
        "loader outcomes: complete",
        "resolver failure: deliberate-bad-config",
        "unresolved row: deliberate-unknown",
        "roster handoff:",
        "scoped waterfall/query: 200",
        "typed update veto: auth unchanged",
        "postcommit apply failure: not recoverable by control",
        "exact publication: ratelimit replaced",
        "timeout completed",
        "timeout elapsed",
        "timeout cancelled",
        "gateway down",
    ] {
        assert!(
            stdout.contains(evidence),
            "missing `{evidence}` in:\n{stdout}"
        );
    }
}
