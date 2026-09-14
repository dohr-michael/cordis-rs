use cordis_core::FiberState;

fn main() {
    let _: u8 = FiberState::Active.into();
    let _ = FiberState::try_from(1u8);
}
