//! Regression for the bit positions in pinned
//! `include/rive/animation/state_transition_flags.hpp`.
use nuxie_runtime::source::animation::state_transition::StateTransition;

#[test]
fn transition_predicates_match_every_authored_flag_combination() {
    for flags in 0_u32..64 {
        let mut transition = StateTransition::default();
        let mut base = std::mem::take(&mut transition.base);
        base.set_flags(flags, &mut transition);
        transition.base = base;

        // Independent pinned wire-bit oracle, not the enum used by the owner.
        assert_eq!(transition.is_disabled(), flags & 1 != 0, "flags={flags}");
        assert_eq!(
            transition.duration_is_percentage(),
            flags & 2 != 0,
            "flags={flags}"
        );
        assert_eq!(
            transition.enable_exit_time(),
            flags & 4 != 0,
            "flags={flags}"
        );
        assert_eq!(
            transition.exit_time_is_percentage(),
            flags & 8 != 0,
            "flags={flags}"
        );
        assert_eq!(transition.pause_on_exit(), flags & 16 != 0, "flags={flags}");
        assert_eq!(
            transition.enable_early_exit(),
            flags & 32 != 0,
            "flags={flags}"
        );
    }
}
