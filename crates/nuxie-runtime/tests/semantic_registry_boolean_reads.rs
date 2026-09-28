use nuxie_runtime::source::{
    generated::core_registry::CoreRegistry, semantic::semantic_data::SemanticData,
};

// Upstream d4fe1022 adds these virtual-property getters to CoreRegistry::getBool.
// Binding reverse reads must see the flag that the matching setter changed.
#[test]
fn semantic_boolean_registry_reads_match_written_flags() {
    // 0d8ca59d moves checked (998) to uint and removes mixed (999).
    for key in (989..=1009).filter(|key| !matches!(key, 998 | 999)) {
        let mut semantic = SemanticData::default();
        for value in [true, false, true] {
            CoreRegistry::set_bool(&mut semantic, key, value);
            assert_eq!(
                CoreRegistry::get_bool(&mut semantic, key),
                value,
                "key {key}"
            );
        }
    }
}
