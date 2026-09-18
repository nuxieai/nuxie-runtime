use nuxie_schema::{
    CoreRegistryFieldKind, StoredFieldInitializer, core_registry_field_kind_by_property_key,
    definition_by_type_key, object_supports_property,
};

#[test]
fn collection_metadata_extends_upstream_semantics_with_unknown_defaults() {
    let collection = definition_by_type_key(60002).expect("collection semantic definition");
    assert!(collection.is_a("SemanticData"));
    assert!(collection.is_a("Component"));
    // Upstream role, name and state remain on the same semantic object.
    for key in [982, 983, 988] {
        assert!(object_supports_property(60002, key));
    }
    for key in [60016, 60017] {
        assert_eq!(
            core_registry_field_kind_by_property_key(key),
            Some(CoreRegistryFieldKind::Uint)
        );
        let property = collection.property_by_key(key).unwrap();
        assert_eq!(
            property.stored_field_initializer(),
            Some(StoredFieldInitializer::Uint(u32::MAX.into()))
        );
        assert!(object_supports_property(60002, key));
        assert!(!object_supports_property(668, key));
    }
}
