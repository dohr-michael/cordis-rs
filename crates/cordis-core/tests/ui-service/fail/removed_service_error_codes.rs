use cordis_core::CordisErrorCode;

fn main() {
    let _ = CordisErrorCode::ServiceUnavailable;
    let _ = CordisErrorCode::DuplicateService;
    let _ = CordisErrorCode::ServiceTypeMismatch;
}
