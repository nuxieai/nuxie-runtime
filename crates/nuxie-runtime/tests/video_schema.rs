use nuxie_schema::{FieldKind, definition_by_name};

#[test]
fn video_schema_exposes_the_property_contract_to_compilers() {
    let video = definition_by_name("Video").unwrap();
    for (name, key, kind, writable) in [
        ("loop", 60004, FieldKind::Bool, true),
        ("muted", 60005, FieldKind::Bool, true),
        ("volume", 60006, FieldKind::Double, true),
        ("playbackRate", 60007, FieldKind::Double, true),
        ("loopStart", 60014, FieldKind::Double, true),
        ("loopEnd", 60015, FieldKind::Double, true),
        ("paused", 60016, FieldKind::Bool, true),
        ("position", 60017, FieldKind::Double, true),
        ("duration", 60018, FieldKind::Double, false),
        ("ended", 60019, FieldKind::Bool, false),
        ("state", 60020, FieldKind::Double, false),
    ] {
        let property = video
            .properties
            .iter()
            .find(|p| p.name == name)
            .unwrap_or_else(|| panic!("Video.{name} is missing from the compiler schema"));
        assert_eq!(property.key.int, key, "{name}");
        assert_eq!(property.runtime_type, kind, "{name}");
        assert!(property.bindable, "{name}");
        assert_eq!(property.animates, writable, "{name}");
        assert_eq!(property.computed, !writable, "{name}");
    }
}
