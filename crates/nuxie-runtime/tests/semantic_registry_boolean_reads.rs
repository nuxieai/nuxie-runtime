use nuxie_runtime::source::{
    generated::core_registry::CoreRegistry, semantic::semantic_data::SemanticData,
};

// Upstream d4fe1022 adds these virtual-property getters to CoreRegistry::getBool.
// Binding reverse reads must see the flag that the matching setter changed.
#[test]
fn semantic_boolean_registry_reads_match_written_flags() {
    for key in 989..=1009 {
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
