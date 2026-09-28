use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

fn constants(dir: &Path, output: &mut BTreeMap<String, u16>) {
    for entry in std::fs::read_dir(dir).expect("generated header directory") {
        let path = entry.expect("header entry").path();
        if path.is_dir() {
            constants(&path, output);
        } else if path.extension().is_some_and(|extension| extension == "hpp") {
            let source = std::fs::read_to_string(&path).expect("generated header");
            let Some(class) = source.lines().find_map(|line| {
                // Mixins such as ColorChannelsBase begin with `class Core;`.
                // Only the class definition owns the constants below.
                line.strip_prefix("class ")
                    .filter(|line| !line.trim_end().ends_with(';'))
                    .and_then(|line| line.split_whitespace().next())
            }) else {
                continue;
            };
            for line in source.lines().map(str::trim) {
                let Some(declaration) = line.strip_prefix("static const uint16_t ") else {
                    continue;
                };
                let Some((name, value)) = declaration.split_once(" = ") else {
                    continue;
                };
                if name.ends_with("PropertyKey") {
                    output.insert(
                        format!("{class}::{name}"),
                        value.trim_end_matches(';').parse().expect("property key"),
                    );
                }
            }
        }
    }
}

fn keys(source: &str, signature: &str, constants: &BTreeMap<String, u16>) -> BTreeSet<u16> {
    let body = source.split_once(signature).expect("registry method").1;
    let body = body.split("\n    static ").next().unwrap();
    body.split("case ")
        .skip(1)
        .map(|case| {
            let name = case.split_once("PropertyKey:").expect("property case").0;
            let name = format!("{}PropertyKey", name.split_whitespace().collect::<String>());
            *constants
                .get(&name)
                .unwrap_or_else(|| panic!("missing {name}"))
        })
        .collect()
}

#[test]
fn id_dispatch_matches_current_cpp_and_remains_uint_on_the_wire() {
    let runtime = std::env::var_os("RIVE_RUNTIME_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/Users/levi/dev/oss/rive-runtime"));
    let generated = runtime.join("include/rive/generated");
    let mut property_keys = BTreeMap::new();
    constants(&generated, &mut property_keys);
    let source =
        std::fs::read_to_string(generated.join("core_registry.hpp")).expect("current registry");
    let setters = keys(&source, "static void setId(", &property_keys);
    let getters = keys(&source, "static Id getId(", &property_keys);
    assert_eq!(setters, getters);
    let actual: BTreeSet<_> = (0..=u16::MAX)
        .filter(|key| nuxie_schema::is_id_property_key(*key))
        .collect();
    assert_eq!(
        actual, setters,
        "Id dispatch must follow current headers, not the historical seed alone"
    );
    let uint_setters = keys(&source, "static void setUint(", &property_keys);
    let uint_getters = keys(&source, "static uint32_t getUint(", &property_keys);
    assert!(actual.is_subset(&uint_setters));
    assert!(actual.is_subset(&uint_getters));
    for key in actual {
        assert_eq!(
            nuxie_schema::core_registry_field_kind_by_property_key(key),
            Some(nuxie_schema::CoreRegistryFieldKind::Uint)
        );
        assert_eq!(
            nuxie_schema::core_registry_setter_field_kind_by_property_key(key),
            Some(nuxie_schema::FieldKind::Uint)
        );
    }
    assert!(nuxie_schema::definition_by_type_key(102).is_none());
}
