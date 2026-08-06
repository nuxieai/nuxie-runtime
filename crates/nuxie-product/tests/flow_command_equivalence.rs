//! Evidence-backed comparison of the product Flow protocol and baseline CommandServer.

mod flow_command_equivalence_support;

use flow_command_equivalence_support::{
    Classification, RESPONSIBILITY_DECISIONS, compare_atomic_failure, compare_delivery_phases,
    compare_scalar_round_trip,
};

#[test]
fn every_audited_responsibility_has_an_explicit_decision() {
    let names = RESPONSIBILITY_DECISIONS
        .iter()
        .map(|decision| decision.responsibility)
        .collect::<Vec<_>>();

    assert_eq!(
        names,
        [
            "scalar value mutation",
            "output phases",
            "atomic rollback",
            "wake scheduling",
            "terminal errors",
            "wasm suitability",
            "latency",
            "allocations",
            "Flow-only graph and host-cycle machinery",
        ]
    );
    assert!(RESPONSIBILITY_DECISIONS.iter().all(|decision| {
        !decision.evidence.is_empty()
            && matches!(
                decision.classification,
                Classification::Equivalent
                    | Classification::NonEquivalent
                    | Classification::Deferred
            )
    }));
}

#[test]
fn scalar_round_trip_matches_but_delivery_phases_do_not() {
    let comparison = compare_scalar_round_trip();
    assert_eq!(comparison.flow_value, true);
    assert_eq!(comparison.command_value, true);

    let phases = compare_delivery_phases();
    assert_eq!(phases.flow_outputs_before_return, 1);
    assert_eq!(phases.command_events_before_server_poll, 0);
    assert_eq!(phases.command_events_before_message_dispatch, 0);
    assert!(phases.command_events_after_message_dispatch > 0);
}

#[test]
fn invalid_second_mutation_rolls_back_flow_but_not_prior_commands() {
    let comparison = compare_atomic_failure();
    assert!(!comparison.flow_value_after_failure);
    assert!(comparison.command_value_after_failure);
    assert_eq!(comparison.flow_error_class, "not_found");
    assert!(comparison.command_error_count > 0);
}
