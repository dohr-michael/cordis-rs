//! EX-05 executable evidence for the final-v3 scopes/tenants consumer.

use std::fs;
use std::process::{Command, Stdio};

#[test]
fn scopes_tenants_uses_only_final_v3_facades() {
    let source = fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/main.rs"))
        .expect("scopes_tenants source should be readable");
    for stale in [
        "EventCarrier",
        ".plugin(",
        "emit_scoped",
        "bail_scoped",
        "services_snapshot",
        "remove_service",
        "provide(&self)",
        "IsolateId",
        "IsolateRealm",
        "as_u64",
        "structural-group",
        "Context hierarchy",
    ] {
        assert!(!source.contains(stale), "stale mechanism `{stale}` remains");
    }
    for required in [
        "new_service_realm()",
        "with_service_realms",
        "with_child_scope()",
        "Routing::Scoped",
        "Routing::Unscoped",
        "ListenerOptions::default().global()",
        "PreparedPlugin",
        "Roster",
    ] {
        assert!(
            source.contains(required),
            "final-v3 mechanism `{required}` is not demonstrated"
        );
    }
}

#[test]
fn scopes_tenants_runbook_is_scripted_self_terminating_and_discriminating() {
    let output = Command::new(env!("CARGO_BIN_EXE_scopes_tenants"))
        .stdin(Stdio::null())
        .output()
        .expect("scopes_tenants should launch");
    assert!(
        output.status.success(),
        "scopes_tenants exited unsuccessfully:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    let stdout = String::from_utf8(output.stdout).expect("scopes_tenants output is UTF-8");
    for evidence in [
        "placement: private realms stay isolated",
        "placement: explicit shared realm joins only trail",
        "routing: ancestor reached",
        "routing: sibling excluded",
        "routing: global listener reached",
        "axes: isolate derivation kept the same Scope reachability",
        "scopes_tenants tour complete",
    ] {
        assert!(
            stdout.contains(evidence),
            "missing `{evidence}` in:\n{stdout}"
        );
    }
}
