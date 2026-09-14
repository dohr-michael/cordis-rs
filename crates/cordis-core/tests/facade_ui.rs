//! Issue 58 closure evidence for the complete cordis-core semantic facade.

use trybuild::TestCases;

#[test]
fn issue58_core_semantic_facade() {
    let t = TestCases::new();
    t.pass("tests/ui-facade58/pass/*.rs");

    // Global PI-08/PI-09 closure matrix: the earlier semantic tickets own the
    // local proofs; Issue 58 reruns representative witnesses together so the
    // final facade cannot regress by preserving a stale path or extra bound.
    t.pass("tests/ui-configuration/pass/minimal_bounds.rs");
    t.pass("tests/ui-effect/pass/effect_fnonce_minimal_bounds.rs");
    t.pass("tests/ui/pass/event_minimal_bounds.rs");
    t.pass("tests/ui-spawn/pass/opaque_fiber_identity.rs");
    t.pass("tests/ui-observation/pass/read_only_snapshot.rs");

    t.compile_fail("tests/ui-facade58/fail/*.rs");
    t.compile_fail("tests/ui-configuration/fail/removed_erased_plugin_surfaces.rs");
    t.compile_fail("tests/ui-configuration/fail/prepared_values_are_move_only.rs");
    t.compile_fail("tests/ui-context/fail/context_module_path_is_private.rs");
    t.compile_fail("tests/ui-context/fail/service_realm_has_no_representation_contract.rs");
    t.compile_fail("tests/ui-context/fail/service_realm_is_not_copy.rs");
    t.compile_fail("tests/ui-context/fail/service_realm_is_not_serializable.rs");
    t.compile_fail("tests/ui-spawn/fail/identity_and_registry_are_opaque.rs");
    t.compile_fail("tests/ui-registry/fail/removed_topology_and_old_removal.rs");
    t.compile_fail("tests/ui-service/fail/publication_is_move_only.rs");
    t.compile_fail("tests/ui/fail/old_events_module.rs");
    t.compile_fail("tests/ui/fail/issue33_removed_scoped_aliases.rs");
    t.compile_fail("tests/ui/fail/event_store_is_private.rs");
    t.compile_fail("tests/ui/fail/listener_registration_id_is_opaque.rs");
    t.compile_fail("tests/ui/fail/listener_registration_is_move_only.rs");
    t.compile_fail("tests/ui-effect/fail/effect_registration_is_move_only.rs");
    t.compile_fail("tests/ui-list/fail/removed_list_surfaces.rs");
    t.compile_fail("tests/ui-observation/fail/retired_internal_observation_surface.rs");
    t.compile_fail("tests/ui-error57/fail/global_error_surface_removed.rs");
}
