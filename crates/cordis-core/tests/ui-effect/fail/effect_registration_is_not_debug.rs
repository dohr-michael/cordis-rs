// The registration exposes no Debug contract.

use cordis_core::effect::EffectRegistration;

fn requires_debug<T: std::fmt::Debug>() {}

fn main() {
    requires_debug::<EffectRegistration>();
}
