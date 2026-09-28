// Copyright 2026 Rive
//! The five semantic_check_state_test.cpp cases at
//! 0d8ca59dac1b67c35d03a79cb29c595ccddab8d9, plus registry dispatch coverage.

use nuxie_runtime::source::{
    generated::{core_registry::CoreRegistry, semantic::semantic_data_base::SemanticDataBase},
    semantic::{
        semantic_data::SemanticData,
        semantic_state::{SemanticCheckState, SemanticState, check_state_of, has_semantic_state},
    },
};

fn encode(value: u32) -> u32 {
    value << SemanticDataBase::IS_CHECKED_BIT_OFFSET
}

#[test]
fn check_state_of_decodes_each_authored_value() {
    assert_eq!(check_state_of(encode(0)), SemanticCheckState::Unchecked);
    assert_eq!(check_state_of(encode(1)), SemanticCheckState::Checked);
    assert_eq!(check_state_of(encode(2)), SemanticCheckState::Mixed);
}

#[test]
fn check_state_of_reads_the_unassigned_fourth_value_as_mixed() {
    assert_eq!(check_state_of(encode(3)), SemanticCheckState::Mixed);
}

#[test]
fn check_state_of_ignores_the_states_packed_around_the_field() {
    let neighbours = (SemanticState::SELECTED | SemanticState::TOGGLED | SemanticState::HIDDEN).0;
    assert_eq!(check_state_of(neighbours), SemanticCheckState::Unchecked);
    assert_eq!(
        check_state_of(neighbours | encode(2)),
        SemanticCheckState::Mixed
    );
}

#[test]
fn is_checked_writes_the_field_without_disturbing_its_neighbours() {
    let mut sd = SemanticData::default();
    sd.set_state_flags((SemanticState::SELECTED | SemanticState::TOGGLED).0);
    sd.set_is_checked(2);
    assert_eq!(sd.is_checked(), 2);
    assert_eq!(
        check_state_of(sd.base.state_flags()),
        SemanticCheckState::Mixed
    );
    assert!(has_semantic_state(
        sd.base.state_flags(),
        SemanticState::SELECTED
    ));
    assert!(has_semantic_state(
        sd.base.state_flags(),
        SemanticState::TOGGLED
    ));

    sd.set_is_checked(1);
    assert_eq!(sd.is_checked(), 1);
    assert_eq!(
        check_state_of(sd.base.state_flags()),
        SemanticCheckState::Checked
    );

    sd.set_is_checked(0);
    assert_eq!(sd.is_checked(), 0);
    assert_eq!(
        check_state_of(sd.base.state_flags()),
        SemanticCheckState::Unchecked
    );
    assert!(has_semantic_state(
        sd.base.state_flags(),
        SemanticState::SELECTED
    ));
    assert!(has_semantic_state(
        sd.base.state_flags(),
        SemanticState::TOGGLED
    ));
}

#[test]
fn check_field_occupies_the_bits_the_two_flags_used_to() {
    const _: () = assert!(SemanticDataBase::IS_CHECKED_BIT_OFFSET == 2);
    const _: () = assert!(SemanticDataBase::IS_CHECKED_FIELD_MASK == 0xc);
    assert_eq!(encode(1), 1 << 2);
    assert_eq!(encode(2), 1 << 3);
}

#[test]
fn check_state_registry_dispatch_is_uint_not_bool_and_mixed_key_is_removed() {
    let mut sd = SemanticData::default();
    let neighbours = (SemanticState::SELECTED | SemanticState::TOGGLED).0;
    sd.set_state_flags(neighbours);
    let checked_key = i32::from(SemanticDataBase::IS_CHECKED_PROPERTY_KEY);
    // Registry uint values first truncate to uint8, then the setter masks the
    // packed field to two bits. No range rejection or semantic clamping occurs.
    for (value, expected) in [
        (2, 2),
        (1, 1),
        (0, 0),
        (3, 3),
        (4, 0),
        (255, 3),
        (256, 0),
        (258, 2),
    ] {
        CoreRegistry::set_uint(&mut sd, checked_key, value);
        assert_eq!(CoreRegistry::get_uint(&mut sd, checked_key), expected);
        assert_eq!(u32::from(sd.is_checked()), expected);
        assert_eq!(sd.base.state_flags(), neighbours | encode(expected));
        for boolean in [true, false] {
            CoreRegistry::set_bool(&mut sd, checked_key, boolean);
            assert!(!CoreRegistry::get_bool(&mut sd, checked_key));
            CoreRegistry::set_bool(&mut sd, 999, boolean);
            assert!(!CoreRegistry::get_bool(&mut sd, 999));
            assert_eq!(sd.base.state_flags(), neighbours | encode(expected));
        }
        CoreRegistry::set_uint(&mut sd, 999, 2);
        assert_eq!(CoreRegistry::get_uint(&mut sd, 999), 0);
        assert_eq!(sd.base.state_flags(), neighbours | encode(expected));
    }
}
