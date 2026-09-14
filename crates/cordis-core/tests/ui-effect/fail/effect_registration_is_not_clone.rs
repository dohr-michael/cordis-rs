// The registration is a move-only exact capability: no Clone.

use cordis_core::effect::EffectRegistration;

fn requires_clone<T: Clone>() {}

fn main() {
    requires_clone::<EffectRegistration>();
}
