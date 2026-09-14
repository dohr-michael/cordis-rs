//! EX-06 executable evidence for the final-v3 logging/exporters consumer.

use std::fs;
use std::process::{Command, Stdio};

#[test]
fn logging_exporters_uses_only_final_v3_facades() {
    let source = fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/main.rs"))
        .expect("logging_exporters source should be readable");
    for stale in [
        "Message",
        "ExporterId",
        "remove_exporter",
        "InternalDispatch",
        "InternalListener",
        "InternalPlugin",
        "InternalService",
        "InternalStatus",
        "DispatchMode",
        "EventCarrier",
        "services_snapshot",
        "live_fiber_count",
        "pending_missing()\n            .iter",
        ".plugin(",
        "boot_report(&ctx",
        ".report(&ctx)",
    ] {
        assert!(!source.contains(stale), "stale mechanism `{stale}` remains");
    }
    for required in [
        "LogRecord",
        "BufferExporter",
        "ExporterRegistration",
        "with_name",
        "runtime_snapshot()",
        "observe_runtime",
        "RuntimeObservation",
        "remove_plugins::<Canary>",
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
fn logging_exporters_runbook_is_scripted_self_terminating_and_discriminating() {
    let output = Command::new(env!("CARGO_BIN_EXE_logging_exporters"))
        .stdin(Stdio::null())
        .output()
        .expect("logging_exporters should launch");
    assert!(
        output.status.success(),
        "logging_exporters exited unsuccessfully:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).expect("logging_exporters output is UTF-8");
    for evidence in [
        "buffer: pre-registration record absent",
        "buffering: explicit bounded buffer evicted oldest receipt",
        "filtering: channel override and default threshold distinguished",
        "exact removal: removed occurrence stayed quiet while sibling survived",
        "record accessors: sequence/timestamp/channel/level/text observed",
        "snapshot: flat fiber rows observed",
        "snapshot/observation: opaque FiberId correlates current state",
        "observation: failing observer did not fail source operation",
        "typed removal: old rows disappeared",
        "fresh spawn: same Plugin received a fresh FiberId",
        "logging_exporters tour complete",
    ] {
        assert!(
            stdout.contains(evidence),
            "missing `{evidence}` in:\n{stdout}"
        );
    }
}
