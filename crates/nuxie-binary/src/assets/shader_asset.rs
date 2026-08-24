pub(crate) const FILE_EXTENSION: &str = "rstb";

#[cfg(test)]
mod upstream_tests {
    use super::*;
    use nuxie_schema::{definition_by_name, definition_by_type_key};

    fn shader_asset_matches_type_key(type_key: u16) -> bool {
        let shader = *definition_by_name("ShaderAsset").unwrap();
        definition_by_type_key(type_key).is_some_and(|candidate| shader.is_a(candidate.name))
    }

    #[test]
    fn shader_asset_is_type_of() {
        assert_eq!(shader_asset_matches_type_key(970), true);
        assert_eq!(shader_asset_matches_type_key(103), true);
        assert_eq!(shader_asset_matches_type_key(99), true);
        assert_eq!(shader_asset_matches_type_key(0), false);
        assert_eq!(shader_asset_matches_type_key(999), false);
    }

    #[test]
    fn shader_asset_core_type() {
        let asset = definition_by_name("ShaderAsset").unwrap();
        assert_eq!(asset.type_key.int, 970);
    }

    #[test]
    fn shader_asset_file_extension() {
        assert_eq!(FILE_EXTENSION, "rstb");
    }
}
