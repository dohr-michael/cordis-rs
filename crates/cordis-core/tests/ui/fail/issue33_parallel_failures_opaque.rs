use cordis_core::event::ParallelFailures;
fn inspect(failures: &ParallelFailures) { let _ = &failures.errors; let _ = failures.to_string(); }
fn main(){}
