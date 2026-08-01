use crate::*;

pub(crate) const FILE_EXTENSION: &str = "man";

#[derive(Debug, Clone, Default, Serialize)]
pub struct RuntimeManifest {
    pub names: BTreeMap<i32, StringValue>,
    pub paths: BTreeMap<i32, Vec<u32>>,
}

impl RuntimeFile {
    pub fn manifest(&self) -> Option<RuntimeManifest> {
        self.manifest_with_script_assets(false)
    }

    pub fn scripting_manifest(&self) -> Option<RuntimeManifest> {
        self.manifest_with_script_assets(true)
    }

    fn manifest_with_script_assets(
        &self,
        script_assets_create_importers: bool,
    ) -> Option<RuntimeManifest> {
        let mut latest_file_asset = None;
        let mut manifest = None;

        for (index, object) in self.objects.iter().enumerate() {
            if self.import_status(index) != Some(RuntimeImportStatus::Imported) {
                continue;
            }

            let Some(object) = object.as_ref() else {
                continue;
            };
            let Some(definition) = definition_by_type_key(object.type_key) else {
                continue;
            };

            if file_asset_creates_importer(definition.name, script_assets_create_importers) {
                latest_file_asset = Some(object);
                if definition.name == "ManifestAsset" {
                    manifest = Some(RuntimeManifest::default());
                }
            }

            if definition.name == "FileAssetContents"
                && latest_file_asset.is_some_and(|asset| asset.type_name == "ManifestAsset")
            {
                manifest = Some(parse_cpp_manifest_asset(
                    object.bytes_property("bytes").unwrap_or(&[]),
                ));
            }
        }

        manifest
    }
}

impl RuntimeManifest {
    pub fn resolve_name(&self, id: u32) -> Option<&str> {
        self.names
            .get(&cpp_manifest_resolver_key(id))
            .and_then(StringValue::as_str)
    }

    pub fn resolve_name_bytes(&self, id: u32) -> Option<&[u8]> {
        self.names
            .get(&cpp_manifest_resolver_key(id))
            .map(StringValue::as_bytes)
    }

    pub fn resolve_path(&self, id: u32) -> Option<&[u32]> {
        self.paths
            .get(&cpp_manifest_resolver_key(id))
            .map(Vec::as_slice)
    }
}

pub(crate) fn validate_cpp_manifest_assets_with_budget(
    file: &RuntimeFile,
    script_assets_create_importers: bool,
    property_budget: &mut RuntimePropertyBudget,
) -> Result<()> {
    if property_budget.maximum.is_none() {
        return Ok(());
    }

    let mut latest_file_asset_is_manifest = false;
    for (index, object) in file.objects.iter().enumerate() {
        if file.import_status(index) != Some(RuntimeImportStatus::Imported) {
            continue;
        }
        let Some(object) = object.as_ref() else {
            continue;
        };
        let Some(definition) = definition_by_type_key(object.type_key) else {
            continue;
        };

        if file_asset_creates_importer(definition.name, script_assets_create_importers) {
            latest_file_asset_is_manifest = definition.name == "ManifestAsset";
            continue;
        }
        if definition.name == "FileAssetContents" && latest_file_asset_is_manifest {
            validate_cpp_manifest_asset_with_budget(
                object.bytes_property("bytes").unwrap_or(&[]),
                property_budget,
            )?;
        }
    }
    Ok(())
}

/// Validate every count that the lazy manifest decoder will later use to
/// allocate map entries or path vectors. Malformed manifests intentionally
/// remain a soft failure, matching `parse_cpp_manifest_asset`; declared work
/// above the import budget is the only error promoted to the file boundary.
pub(crate) fn validate_cpp_manifest_asset_with_budget(
    bytes: &[u8],
    property_budget: &mut RuntimePropertyBudget,
) -> Result<()> {
    let mut reader = BinaryReader::new(bytes);
    while !reader.reached_end() {
        let Ok(section) = reader.read_var_uint() else {
            return Ok(());
        };
        let Some(section_size) = reader
            .read_var_uint()
            .ok()
            .and_then(|value| usize::try_from(value).ok())
        else {
            return Ok(());
        };
        let Ok(section_bytes) = reader.read_bytes_exact(section_size) else {
            return Ok(());
        };
        let mut section_reader = BinaryReader::new(section_bytes);

        match section {
            0 => {
                let Ok(count) = section_reader.read_var_uint() else {
                    return Ok(());
                };
                let count = property_budget.reserve_declared(count, "manifest name entries")?;
                for _ in 0..count {
                    if section_reader.read_var_uint().is_err()
                        || section_reader.read_length_prefixed_bytes().is_err()
                    {
                        return Ok(());
                    }
                }
            }
            1 => {
                let Ok(count) = section_reader.read_var_uint() else {
                    return Ok(());
                };
                let count = property_budget.reserve_declared(count, "manifest path entries")?;
                for _ in 0..count {
                    if section_reader.read_var_uint().is_err() {
                        return Ok(());
                    }
                    let Ok(path_len) = section_reader.read_var_uint() else {
                        return Ok(());
                    };
                    let path_len =
                        property_budget.reserve_declared(path_len, "manifest path components")?;
                    for _ in 0..path_len {
                        if section_reader.read_var_uint().is_err() {
                            return Ok(());
                        }
                    }
                }
            }
            _ => continue,
        }

        if !section_reader.reached_end() {
            return Ok(());
        }
    }
    Ok(())
}

fn parse_cpp_manifest_asset(bytes: &[u8]) -> RuntimeManifest {
    let mut manifest = RuntimeManifest::default();
    if bytes.is_empty() {
        return manifest;
    }

    let mut reader = BinaryReader::new(bytes);
    while !reader.reached_end() {
        let Ok(section) = reader.read_var_uint() else {
            return manifest;
        };
        let section_size = match reader
            .read_var_uint()
            .ok()
            .and_then(|value| usize::try_from(value).ok())
        {
            Some(value) => value,
            None => return manifest,
        };
        let section_bytes = match reader.read_bytes_exact(section_size) {
            Ok(bytes) => bytes,
            Err(_) => return manifest,
        };
        let mut section_reader = BinaryReader::new(section_bytes);

        let decoded = match section {
            0 => decode_cpp_manifest_names(&mut section_reader, &mut manifest),
            1 => decode_cpp_manifest_paths(&mut section_reader, &mut manifest),
            _ => continue,
        };

        if decoded.is_err() {
            return manifest;
        }

        if !section_reader.reached_end() {
            return manifest;
        }
    }

    manifest
}

fn decode_cpp_manifest_names(
    reader: &mut BinaryReader<'_>,
    manifest: &mut RuntimeManifest,
) -> Result<()> {
    let count = reader.read_var_uint()?;
    for _ in 0..count {
        let id = cpp_manifest_key(reader.read_var_uint()?);
        let value = reader.read_string()?;
        manifest.names.insert(id, value);
    }
    Ok(())
}

fn decode_cpp_manifest_paths(
    reader: &mut BinaryReader<'_>,
    manifest: &mut RuntimeManifest,
) -> Result<()> {
    let count = reader.read_var_uint()?;
    for _ in 0..count {
        let id = cpp_manifest_key(reader.read_var_uint()?);
        let path_len = reader.read_var_uint()?;
        let mut path = Vec::new();
        for _ in 0..path_len {
            path.push(read_cpp_manifest_path_id(reader));
        }
        manifest.paths.insert(id, path);
    }
    Ok(())
}

// Manifest name/path maps are keyed by a *signed* int in C++
// (`DataResolver::resolveName(int id)`, include/rive/data_resolver.hpp). The
// runtime id arrives as an unsigned var-uint, so we deliberately reinterpret the
// low 32 bits as i32 (`as i32` is a bit-preserving truncate/reinterpret in Rust,
// NOT saturating) to match C++'s key space exactly -- an id above i32::MAX must
// wrap to the same negative key on both insert and lookup. Insert path.
pub(crate) fn cpp_manifest_key(value: u64) -> i32 {
    value as i32
}

// Lookup counterpart to cpp_manifest_key: same intentional u32->i32
// reinterpret, so `resolve_*` finds keys inserted by decode_cpp_manifest_*.
// See cpp_manifest_key above and the pinning test cpp_manifest_key_reinterpret.
pub(crate) fn cpp_manifest_resolver_key(value: u32) -> i32 {
    value as i32
}

fn read_cpp_manifest_path_id(reader: &mut BinaryReader<'_>) -> u32 {
    match reader.read_var_uint() {
        Ok(value) => value as u32,
        Err(_) => {
            reader.offset = reader.bytes.len();
            0
        }
    }
}
