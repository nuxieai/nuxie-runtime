use crate::*;

impl RuntimeFile {
    pub fn file_assets(&self) -> Vec<&RuntimeObject> {
        self.file_asset_object_ids
            .iter()
            .filter_map(|object_id| self.object(*object_id))
            .collect()
    }

    pub fn file_asset(&self, index: usize) -> Option<&RuntimeObject> {
        self.file_asset_object_ids
            .get(index)
            .and_then(|object_id| self.object(*object_id))
    }
}

impl RuntimeObject {
    pub fn file_asset_cdn_uuid_string(&self) -> Option<String> {
        let definition = definition_by_type_key(self.type_key)?;
        if !definition.is_a("FileAsset") {
            return None;
        }

        Some(format_cpp_file_asset_cdn_uuid(
            self.bytes_property("cdnUuid").unwrap_or(&[]),
        ))
    }

    pub fn file_asset_extension(&self) -> Option<&'static str> {
        cpp_file_asset_extension(self.type_name)
    }

    pub fn file_asset_unique_name(&self) -> Option<String> {
        String::from_utf8(self.file_asset_unique_name_bytes()?).ok()
    }

    pub fn file_asset_unique_name_bytes(&self) -> Option<Vec<u8>> {
        self.file_asset_extension()?;
        let name = self.string_property_bytes("name").unwrap_or_default();
        let stem_end = name
            .iter()
            .rposition(|byte| *byte == b'.')
            .unwrap_or(name.len());
        let mut unique_name = name[..stem_end].to_vec();
        unique_name.extend_from_slice(b"-");
        unique_name.extend_from_slice(
            self.uint_property("assetId")
                .unwrap_or(0)
                .to_string()
                .as_bytes(),
        );
        Some(unique_name)
    }

    pub fn file_asset_unique_filename(&self) -> Option<String> {
        String::from_utf8(self.file_asset_unique_filename_bytes()?).ok()
    }

    pub fn file_asset_unique_filename_bytes(&self) -> Option<Vec<u8>> {
        let extension = self.file_asset_extension()?;
        let mut filename = self.file_asset_unique_name_bytes()?;
        filename.extend_from_slice(b".");
        filename.extend_from_slice(extension.as_bytes());
        Some(filename)
    }
}

fn cpp_file_asset_extension(type_name: &str) -> Option<&'static str> {
    match type_name {
        "ImageAsset" => Some("png"),
        "FontAsset" => Some("ttf"),
        "AudioAsset" => Some("wav"),
        "BlobAsset" => Some(super::blob_asset::FILE_EXTENSION),
        "ScriptAsset" => Some("lua"),
        "ShaderAsset" => Some(super::shader_asset::FILE_EXTENSION),
        "ManifestAsset" => Some(super::manifest_asset::FILE_EXTENSION),
        _ => None,
    }
}

pub(crate) fn cpp_file_assets_contains(object: &RuntimeObject) -> bool {
    definition_by_type_key(object.type_key).is_some_and(|definition| {
        definition.is_a("FileAsset") && definition.name != "ManifestAsset"
    })
}

fn format_cpp_file_asset_cdn_uuid(bytes: &[u8]) -> String {
    if bytes.len() != 16 {
        return String::new();
    }

    const INDICES: [usize; 16] = [3, 2, 1, 0, 5, 4, 7, 6, 9, 8, 15, 14, 13, 12, 11, 10];

    let mut uuid = String::with_capacity(36);
    for index in INDICES {
        uuid.push_str(&format!("{:02x}", bytes[index]));
        if matches!(index, 0 | 4 | 6 | 8) {
            uuid.push('-');
        }
    }
    uuid
}
