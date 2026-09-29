use crate::*;

pub(crate) const FILE_EXTENSION: &str = "man";

#[derive(Debug, Clone, Default, Serialize)]
pub struct RuntimeManifest {
    pub names: BTreeMap<i32, StringValue>,
    pub paths: BTreeMap<i32, Vec<u32>>,
    pub has_watermark: bool,
    pub watermark_artboard_index: u32,
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
        Some(
            self.names
                .get(&cpp_manifest_resolver_key(id))
                .and_then(StringValue::as_str)
                .unwrap_or_default(),
        )
    }

    pub fn resolve_name_bytes(&self, id: u32) -> Option<&[u8]> {
        Some(
            self.names
                .get(&cpp_manifest_resolver_key(id))
                .map(StringValue::as_bytes)
                .unwrap_or_default(),
        )
    }

    pub fn resolve_path(&self, id: u32) -> Option<&[u32]> {
        Some(
            self.paths
                .get(&cpp_manifest_resolver_key(id))
                .map(Vec::as_slice)
                .unwrap_or_default(),
        )
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
        let Ok(section_size) = reader.read_var_uint() else {
            return manifest;
        };
        let section_start = reader.offset;

        let decoded = match section {
            0 => decode_cpp_manifest_names(&mut reader, &mut manifest),
            1 => decode_cpp_manifest_paths(&mut reader, &mut manifest),
            2 => decode_cpp_manifest_watermark(&mut reader, &mut manifest, section_size),
            _ => {
                let Some(section_size) = usize::try_from(section_size).ok() else {
                    return manifest;
                };
                if reader.read_bytes_exact(section_size).is_err() {
                    return manifest;
                }
                continue;
            }
        };

        if decoded.is_err() {
            return manifest;
        }

        let bytes_read = reader.offset - section_start;
        if u64::try_from(bytes_read).ok() != Some(section_size) {
            return manifest;
        }
    }

    manifest
}

fn decode_cpp_manifest_watermark(
    reader: &mut BinaryReader<'_>,
    manifest: &mut RuntimeManifest,
    section_size: u64,
) -> Result<()> {
    let section_start = reader.offset;
    let version = reader.read_var_uint()?;
    if version == 1 {
        let flags = reader.read_var_uint()?;
        let artboard_index = u32::try_from(reader.read_var_uint()?)
            .map_err(|_| anyhow::anyhow!("watermark artboard index does not fit in uint32_t"))?;
        manifest.has_watermark = (flags & 1) != 0;
        manifest.watermark_artboard_index = artboard_index;
    }
    // Preserve upstream's mutation order: successfully read fields remain
    // visible even if the section's declared size or trailing bytes are bad.
    let bytes_read = (reader.offset - section_start) as u64;
    let remaining = section_size
        .checked_sub(bytes_read)
        .and_then(|value| usize::try_from(value).ok())
        .ok_or_else(|| anyhow::anyhow!("invalid watermark section size"))?;
    reader.read_bytes_exact(remaining)?;
    Ok(())
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
        // C++ stores this var-uint in an `int` before comparing it with the
        // unsigned loop counter. Values outside the positive `int` range can
        // request effectively unbounded work after integer promotion; reject
        // that malformed representation at the Rust allocation boundary.
        let path_len = i32::try_from(reader.read_var_uint()?)
            .map_err(|_| anyhow::anyhow!("manifest path length does not fit in C++ int"))?;
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

#[cfg(test)]
mod watermark_tests {
    use super::*;

    fn append_var_uint(out: &mut Vec<u8>, mut value: u64) {
        loop {
            let mut byte = (value & 0x7f) as u8;
            value >>= 7;
            if value != 0 {
                byte |= 0x80;
            }
            out.push(byte);
            if value == 0 {
                break;
            }
        }
    }

    fn section(id: u64, payload: &[u8]) -> Vec<u8> {
        let mut bytes = Vec::new();
        append_var_uint(&mut bytes, id);
        append_var_uint(&mut bytes, payload.len() as u64);
        bytes.extend_from_slice(payload);
        bytes
    }

    #[test]
    fn no_sections_has_no_watermark() {
        assert!(!parse_cpp_manifest_asset(&[]).has_watermark);
    }

    #[test]
    fn watermark_section_decodes() {
        let manifest = parse_cpp_manifest_asset(&section(2, &[1, 1, 7]));
        assert!(manifest.has_watermark);
        assert_eq!(manifest.watermark_artboard_index, 7);
    }

    #[test]
    fn cleared_flag_leaves_watermark_off() {
        let manifest = parse_cpp_manifest_asset(&section(2, &[1, 0, 7]));
        assert!(!manifest.has_watermark);
        assert_eq!(manifest.watermark_artboard_index, 7);
    }

    #[test]
    fn trailing_fields_are_skipped() {
        let manifest = parse_cpp_manifest_asset(&section(2, &[1, 1, 3, 0x2a, 0x2b]));
        assert!(manifest.has_watermark);
        assert_eq!(manifest.watermark_artboard_index, 3);
    }

    #[test]
    fn unknown_version_still_consumes_section() {
        let mut bytes = section(2, &[99, 1, 3]);
        bytes.extend(section(0, &[1, 4, 2, b'h', b'i']));
        let manifest = parse_cpp_manifest_asset(&bytes);
        assert!(!manifest.has_watermark);
        assert_eq!(manifest.resolve_name(4), Some("hi"));
    }

    #[test]
    fn unknown_section_keeps_parsing() {
        let mut bytes = section(99, &[1, 2, 3]);
        bytes.extend(section(2, &[1, 1, 5]));
        let manifest = parse_cpp_manifest_asset(&bytes);
        assert!(manifest.has_watermark);
        assert_eq!(manifest.watermark_artboard_index, 5);
    }

    #[test]
    fn out_of_range_index_stops_without_setting_watermark() {
        let mut payload = vec![1, 1];
        append_var_uint(&mut payload, 1u64 << 32);
        let mut bytes = section(2, &payload);
        bytes.extend(section(0, &[1, 4, 2, b'h', b'i']));
        let manifest = parse_cpp_manifest_asset(&bytes);
        assert!(!manifest.has_watermark);
        assert!(manifest.names.is_empty());

        let mut payload = vec![1, 1];
        append_var_uint(&mut payload, u32::MAX as u64);
        let manifest = parse_cpp_manifest_asset(&section(2, &payload));
        assert!(manifest.has_watermark);
        assert_eq!(manifest.watermark_artboard_index, u32::MAX);
    }

    #[test]
    fn section_ids_are_not_narrowed() {
        for id in [256, 257, 258] {
            let manifest = parse_cpp_manifest_asset(&section(id, &[1, 1, 7]));
            assert!(!manifest.has_watermark, "section id {id}");
            assert!(manifest.names.is_empty());
            assert!(manifest.paths.is_empty());
        }
        let manifest = parse_cpp_manifest_asset(&section(2, &[1, 1, 7]));
        assert!(manifest.has_watermark);
        assert_eq!(manifest.watermark_artboard_index, 7);
    }

    #[test]
    fn soft_failures_and_unknown_versions_preserve_decoded_state() {
        let mut bytes = section(0, &[1, 4, 2, b'h', b'i']);
        bytes.extend(section(2, &[1, 1, 7]));
        bytes.extend(section(2, &[99, 0, 0]));
        bytes.extend(section(2, &[1, 0])); // Missing index: no mutation.
        let manifest = parse_cpp_manifest_asset(&bytes);
        assert_eq!(manifest.resolve_name(4), Some("hi"));
        assert!(manifest.has_watermark);
        assert_eq!(manifest.watermark_artboard_index, 7);

        // A size mismatch is checked after storing recognized fields.
        for bytes in [vec![2, 2, 1, 1, 7], vec![2, 4, 1, 1, 7]] {
            let manifest = parse_cpp_manifest_asset(&bytes);
            assert!(manifest.has_watermark);
            assert_eq!(manifest.watermark_artboard_index, 7);
        }
    }
}
