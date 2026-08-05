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

pub(crate) fn normalize_file_asset_ids(
    objects: &mut [Option<RuntimeObject>],
    import_statuses: &[RuntimeImportStatus],
) {
    let mut file_asset_ids = Vec::new();
    for index in 0..objects.len() {
        if import_statuses.get(index) != Some(&RuntimeImportStatus::Imported) {
            continue;
        }

        let Some(object) = objects[index].as_ref() else {
            continue;
        };
        // C++ dedupes through BackboardImporter::addFileAsset, which
        // FileAsset::import only reaches when addsToBackboard() is true;
        // ManifestAsset opts out.
        let is_backboard_file_asset =
            definition_by_type_key(object.type_key).is_some_and(|definition| {
                definition.is_a("FileAsset") && definition.name != "ManifestAsset"
            });
        if !is_backboard_file_asset {
            continue;
        }

        file_asset_ids.push(index);
        normalize_file_asset_ids_for_imported_assets(objects, &file_asset_ids);
    }
}

fn normalize_file_asset_ids_for_imported_assets(
    objects: &mut [Option<RuntimeObject>],
    file_asset_ids: &[usize],
) {
    let mut ids = std::collections::BTreeSet::new();
    let mut next_id = 1u32;

    for object_id in file_asset_ids {
        let object = objects[*object_id]
            .as_mut()
            .expect("file_asset_ids only contains present objects");
        let asset_id = object.uint_property("assetId").unwrap_or(0) as u32;
        if ids.contains(&asset_id) {
            set_runtime_uint_property(object, 204, "assetId", "FileAsset", u64::from(next_id));
        } else {
            ids.insert(asset_id);
            if asset_id >= next_id {
                next_id = asset_id.wrapping_add(1);
            }
        }
    }
}

fn set_runtime_uint_property(
    object: &mut RuntimeObject,
    key: u16,
    name: &'static str,
    owner: &'static str,
    value: u64,
) {
    upsert_runtime_property(
        &mut object.properties,
        RuntimeProperty {
            key,
            name,
            owner,
            value: FieldValue::Uint(value),
        },
    );
}
