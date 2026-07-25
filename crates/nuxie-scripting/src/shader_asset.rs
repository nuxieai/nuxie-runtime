//! Bounded decoder for the WebGPU RSTB v4 payload in `ShaderAsset`.
//!
//! Mirrors pinned C++ `src/assets/shader_asset.cpp::ShaderAsset::decode` plus
//! `src/lua/renderer/lua_gpu.cpp::buildShaderEntries`: WebGPU selects authored
//! whole-module WGSL target 0 and requires its `BindingMap` target 16 sidecar.

use nuxie_render_api::{
    GpuCanvasShader, GpuCanvasShaderBinding, GpuCanvasShaderEntry, GpuCanvasShaderResourceKind,
    GpuCanvasShaderStage, GpuCanvasShaderTextureSampleType, GpuCanvasShaderTextureViewDimension,
};

use crate::envelope::SignedContent;
use crate::vm::{Error, Result};

const RSTB_MAGIC: u32 = 0x5253_5442;
const RSTB_VERSION: u16 = 4;
const WGSL_SOURCE_TARGET: u8 = 0;
const WGSL_BINDING_MAP_TARGET: u8 = 16;
const MAX_RSTB_BYTES: usize = 4 * 1024 * 1024;
const MAX_SHADER_MODULE_BYTES: usize = 1024 * 1024;
const BINDING_MAP_BLOB_VERSION: u8 = 2;
const BINDING_MAP_ALLOCATOR_VERSION: u8 = 1;
const BINDING_MAP_ENTRY_WIRE_SIZE: usize = 14;
const BINDING_MAP_ABSENT: u16 = u16::MAX;

#[derive(Debug, Clone, Copy)]
struct VariantDescriptor {
    target: u8,
    offset: usize,
    size: usize,
}

struct Cursor<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Cursor<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn remaining(&self) -> usize {
        self.bytes.len().saturating_sub(self.offset)
    }

    fn read_bytes(&mut self, length: usize, label: &str) -> Result<&'a [u8]> {
        let end = self
            .offset
            .checked_add(length)
            .filter(|end| *end <= self.bytes.len())
            .ok_or_else(|| Error::runtime(format!("RSTB is truncated in {label}")))?;
        let value = &self.bytes[self.offset..end];
        self.offset = end;
        Ok(value)
    }

    fn read_u8(&mut self, label: &str) -> Result<u8> {
        Ok(self.read_bytes(1, label)?[0])
    }

    fn read_u16(&mut self, label: &str) -> Result<u16> {
        let bytes: [u8; 2] = self
            .read_bytes(2, label)?
            .try_into()
            .map_err(|_| Error::runtime(format!("RSTB is truncated in {label}")))?;
        Ok(u16::from_le_bytes(bytes))
    }

    fn read_u32(&mut self, label: &str) -> Result<u32> {
        let bytes: [u8; 4] = self
            .read_bytes(4, label)?
            .try_into()
            .map_err(|_| Error::runtime(format!("RSTB is truncated in {label}")))?;
        Ok(u32::from_le_bytes(bytes))
    }

    fn read_string(&mut self, label: &str) -> Result<String> {
        let length = usize::from(self.read_u16(label)?);
        let bytes = self.read_bytes(length, label)?;
        std::str::from_utf8(bytes)
            .map(str::to_owned)
            .map_err(|_| Error::runtime(format!("RSTB {label} is not UTF-8")))
    }
}

pub(crate) fn decode_shader_asset(name: &str, payload: &[u8]) -> Result<GpuCanvasShader> {
    let envelope = SignedContent::parse(payload)
        .map_err(|error| Error::runtime(format!("ShaderAsset '{name}': {error}")))?;
    let rstb = envelope.content;
    if rstb.len() > MAX_RSTB_BYTES {
        return Err(Error::runtime(format!(
            "ShaderAsset '{name}' RSTB exceeds {MAX_RSTB_BYTES} bytes"
        )));
    }

    let mut cursor = Cursor::new(rstb);
    if cursor.read_u32("magic")? != RSTB_MAGIC {
        return Err(Error::runtime(format!(
            "ShaderAsset '{name}' has invalid RSTB magic"
        )));
    }
    if cursor.read_u16("version")? != RSTB_VERSION {
        return Err(Error::runtime(format!(
            "ShaderAsset '{name}' must use RSTB version {RSTB_VERSION}"
        )));
    }
    let variant_count = usize::from(cursor.read_u8("variant count")?);
    let section_count = usize::from(cursor.read_u8("section count")?);

    let mut descriptors = Vec::with_capacity(variant_count);
    for _ in 0..variant_count {
        descriptors.push(VariantDescriptor {
            target: cursor.read_u8("variant target")?,
            offset: usize::try_from(cursor.read_u32("variant offset")?)
                .map_err(|_| Error::runtime("RSTB variant offset is not addressable"))?,
            size: usize::try_from(cursor.read_u32("variant size")?)
                .map_err(|_| Error::runtime("RSTB variant size is not addressable"))?,
        });
    }
    for _ in 0..section_count {
        let _tag = cursor.read_u8("section tag")?;
        let length = usize::from(cursor.read_u16("section length")?);
        cursor.read_bytes(length, "section payload")?;
    }

    let blob_data = cursor.read_bytes(cursor.remaining(), "blob data")?;
    // `ShaderAsset::decode` first indexes descriptors by target, so a later
    // duplicate replaces an earlier descriptor before any range is checked.
    // Validate the final descriptor for every target (including retired
    // targets) after that coalescing step.
    let mut final_descriptors = [None; 256];
    for descriptor in descriptors {
        final_descriptors[usize::from(descriptor.target)] = Some(descriptor);
    }
    let mut wgsl = None;
    let mut binding_map = None;
    for descriptor in final_descriptors.into_iter().flatten() {
        let end = descriptor
            .offset
            .checked_add(descriptor.size)
            .filter(|end| *end <= blob_data.len())
            .ok_or_else(|| {
                Error::runtime(format!("ShaderAsset '{name}' RSTB variant is truncated"))
            })?;
        let bytes = &blob_data[descriptor.offset..end];
        match descriptor.target {
            // `ShaderAsset::decode` indexes by target and the last descriptor
            // wins. Preserve that deterministic replacement behavior.
            WGSL_SOURCE_TARGET => wgsl = Some(bytes),
            WGSL_BINDING_MAP_TARGET => binding_map = Some(bytes),
            _ => {}
        }
    }
    let wgsl = wgsl.ok_or_else(|| {
        Error::runtime(format!(
            "ShaderAsset '{name}' has no WebGPU RSTB target-0 WGSL source"
        ))
    })?;
    let binding_map = binding_map.ok_or_else(|| {
        Error::runtime(format!(
            "ShaderAsset '{name}' has no mandatory WebGPU RSTB target-16 binding map"
        ))
    })?;
    decode_whole_module_wgsl(name, wgsl, binding_map)
}

fn decode_whole_module_wgsl(
    name: &str,
    source_container: &[u8],
    binding_map: &[u8],
) -> Result<GpuCanvasShader> {
    let mut cursor = Cursor::new(source_container);
    let entry_count = usize::from(cursor.read_u8("WGSL entry count")?);
    if entry_count == 0 {
        return Err(Error::runtime(format!(
            "ShaderAsset '{name}' WGSL entry table is empty"
        )));
    }
    let mut entries = Vec::with_capacity(entry_count);
    for _ in 0..entry_count {
        let stage = match cursor.read_u8("WGSL stage")? {
            0 => GpuCanvasShaderStage::Vertex,
            1 => GpuCanvasShaderStage::Fragment,
            2 => GpuCanvasShaderStage::Compute,
            other => {
                return Err(Error::runtime(format!(
                    "ShaderAsset '{name}' WGSL stage {other} is unsupported"
                )));
            }
        };
        entries.push(GpuCanvasShaderEntry {
            stage,
            logical_entry_point: cursor.read_string("WGSL logical entry point")?,
            physical_entry_point: cursor.read_string("WGSL physical entry point")?,
        });
    }

    let source_length = usize::try_from(cursor.read_u32("WGSL source length")?)
        .map_err(|_| Error::runtime("WGSL source length is not addressable"))?;
    if source_length == 0 || source_length > MAX_SHADER_MODULE_BYTES {
        return Err(Error::runtime(format!(
            "ShaderAsset '{name}' WGSL module size must be between 1 and {MAX_SHADER_MODULE_BYTES} bytes"
        )));
    }
    let source = std::str::from_utf8(cursor.read_bytes(source_length, "WGSL source")?)
        .map_err(|_| Error::runtime(format!("ShaderAsset '{name}' WGSL source is not UTF-8")))?
        .to_owned();
    Ok(GpuCanvasShader {
        source,
        entries,
        bindings: decode_binding_map(name, binding_map)?,
    })
}

fn decode_binding_map(name: &str, bytes: &[u8]) -> Result<Vec<GpuCanvasShaderBinding>> {
    let mut cursor = Cursor::new(bytes);
    if cursor.read_u8("binding-map blob version")? != BINDING_MAP_BLOB_VERSION {
        return Err(Error::runtime(format!(
            "ShaderAsset '{name}' has unsupported WGSL binding-map blob version"
        )));
    }
    if cursor.read_u8("binding-map allocator version")? != BINDING_MAP_ALLOCATOR_VERSION {
        return Err(Error::runtime(format!(
            "ShaderAsset '{name}' has unsupported WGSL binding-map allocator version"
        )));
    }
    let entry_size = usize::from(cursor.read_u16("binding-map entry size")?);
    if entry_size < BINDING_MAP_ENTRY_WIRE_SIZE {
        return Err(Error::runtime(format!(
            "ShaderAsset '{name}' WGSL binding-map entries are too small"
        )));
    }
    let entry_count = usize::try_from(cursor.read_u32("binding-map entry count")?)
        .map_err(|_| Error::runtime("binding-map entry count is not addressable"))?;
    let required_bytes = entry_count
        .checked_mul(entry_size)
        .ok_or_else(|| Error::runtime("binding-map byte length overflow"))?;
    if cursor.remaining() < required_bytes {
        return Err(Error::runtime(format!(
            "ShaderAsset '{name}' WGSL binding map is truncated"
        )));
    }

    let mut bindings = Vec::with_capacity(entry_count);
    for _ in 0..entry_count {
        let row = cursor.read_bytes(entry_size, "binding-map entry")?;
        let read_slot = |offset| {
            let raw = u16::from_le_bytes([row[offset], row[offset + 1]]);
            (raw != BINDING_MAP_ABSENT).then_some(raw)
        };
        bindings.push(GpuCanvasShaderBinding {
            group: row[0],
            binding: row[1],
            kind: decode_resource_kind(name, row[2])?,
            stage_mask: row[3],
            backend_space: row[4],
            backend_slots: [read_slot(5), read_slot(7), read_slot(9)],
            texture_view_dimension: decode_texture_view_dimension(name, row[11])?,
            texture_sample_type: decode_texture_sample_type(name, row[12])?,
            texture_multisampled: row[13] != 0,
        });
    }
    Ok(bindings)
}

fn decode_resource_kind(name: &str, value: u8) -> Result<GpuCanvasShaderResourceKind> {
    match value {
        0 => Ok(GpuCanvasShaderResourceKind::UniformBuffer),
        1 => Ok(GpuCanvasShaderResourceKind::StorageBufferReadOnly),
        2 => Ok(GpuCanvasShaderResourceKind::StorageBufferReadWrite),
        3 => Ok(GpuCanvasShaderResourceKind::SampledTexture),
        4 => Ok(GpuCanvasShaderResourceKind::StorageTexture),
        5 => Ok(GpuCanvasShaderResourceKind::Sampler),
        6 => Ok(GpuCanvasShaderResourceKind::ComparisonSampler),
        other => Err(Error::runtime(format!(
            "ShaderAsset '{name}' WGSL binding map has unknown resource kind {other}"
        ))),
    }
}

fn decode_texture_view_dimension(
    name: &str,
    value: u8,
) -> Result<GpuCanvasShaderTextureViewDimension> {
    match value {
        0 => Ok(GpuCanvasShaderTextureViewDimension::Undefined),
        1 => Ok(GpuCanvasShaderTextureViewDimension::D1),
        2 => Ok(GpuCanvasShaderTextureViewDimension::D2),
        3 => Ok(GpuCanvasShaderTextureViewDimension::D2Array),
        4 => Ok(GpuCanvasShaderTextureViewDimension::Cube),
        5 => Ok(GpuCanvasShaderTextureViewDimension::CubeArray),
        6 => Ok(GpuCanvasShaderTextureViewDimension::D3),
        other => Err(Error::runtime(format!(
            "ShaderAsset '{name}' WGSL binding map has unknown texture view dimension {other}"
        ))),
    }
}

fn decode_texture_sample_type(name: &str, value: u8) -> Result<GpuCanvasShaderTextureSampleType> {
    match value {
        0 => Ok(GpuCanvasShaderTextureSampleType::Undefined),
        1 => Ok(GpuCanvasShaderTextureSampleType::Float),
        2 => Ok(GpuCanvasShaderTextureSampleType::UnfilterableFloat),
        3 => Ok(GpuCanvasShaderTextureSampleType::Depth),
        4 => Ok(GpuCanvasShaderTextureSampleType::Sint),
        5 => Ok(GpuCanvasShaderTextureSampleType::Uint),
        other => Err(Error::runtime(format!(
            "ShaderAsset '{name}' WGSL binding map has unknown texture sample type {other}"
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha256};

    const LOC_009_UBO_WGSL: &str = include_str!("../tests/fixtures/loc009-ubo-triangle.wgsl");
    const LOC_009_BINDING_MAP: &[u8] = &[
        0x02, 0x01, 0x0e, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x02, 0x00, 0xff, 0xff,
        0x00, 0x00, 0xff, 0xff, 0x00, 0x00, 0x00,
    ];

    fn put_u16(bytes: &mut Vec<u8>, value: u16) {
        bytes.extend_from_slice(&value.to_le_bytes());
    }

    fn put_u32(bytes: &mut Vec<u8>, value: u32) {
        bytes.extend_from_slice(&value.to_le_bytes());
    }

    fn put_string(bytes: &mut Vec<u8>, value: &str) {
        put_u16(bytes, value.len() as u16);
        bytes.extend_from_slice(value.as_bytes());
    }

    fn source_container(entries: &[(u8, &str, &str)], wgsl: &str) -> Vec<u8> {
        let mut source = vec![entries.len() as u8];
        for (stage, logical, physical) in entries {
            source.push(*stage);
            put_string(&mut source, logical);
            put_string(&mut source, physical);
        }
        put_u32(&mut source, wgsl.len() as u32);
        source.extend_from_slice(wgsl.as_bytes());
        source
    }

    fn loc_009_source_container() -> Vec<u8> {
        source_container(
            &[(0, "vs_main", "vs_main"), (1, "fs_main", "fs_main")],
            LOC_009_UBO_WGSL,
        )
    }

    fn rstb_payload_with_blob(descriptors: &[(u8, usize, usize)], blob_data: &[u8]) -> Vec<u8> {
        let mut payload = vec![0];
        put_u32(&mut payload, RSTB_MAGIC);
        put_u16(&mut payload, RSTB_VERSION);
        payload.extend_from_slice(&[descriptors.len() as u8, 0]);
        for (target, offset, size) in descriptors {
            payload.push(*target);
            put_u32(&mut payload, *offset as u32);
            put_u32(&mut payload, *size as u32);
        }
        payload.extend_from_slice(blob_data);
        payload
    }

    fn rstb_payload(variants: &[(u8, Vec<u8>)]) -> Vec<u8> {
        let mut offset = 0u32;
        let mut descriptors = Vec::with_capacity(variants.len());
        for (target, blob) in variants {
            descriptors.push((*target, offset as usize, blob.len()));
            offset += blob.len() as u32;
        }
        let mut blob_data = Vec::with_capacity(offset as usize);
        for (_, blob) in variants {
            blob_data.extend_from_slice(blob);
        }
        rstb_payload_with_blob(&descriptors, &blob_data)
    }

    fn loc_009_webgpu_payload() -> Vec<u8> {
        rstb_payload(&[
            (WGSL_SOURCE_TARGET, loc_009_source_container()),
            (WGSL_BINDING_MAP_TARGET, LOC_009_BINDING_MAP.to_vec()),
        ])
    }

    #[test]
    fn decodes_pinned_cpp_webgpu_whole_module_and_binding_map() {
        let payload = loc_009_webgpu_payload();
        assert_eq!(
            format!("{:x}", Sha256::digest(&payload[1..])),
            "546517d0dc9fbdaf9585f3daa6e440628e62292d7cb8aa7253fd3019aa35713d",
            "fixture must remain byte-identical to pinned C++ f4bb3025e263",
        );
        let shader = decode_shader_asset("scene", &payload)
            .expect("WebGPU selects target-0 WGSL and mandatory target-16 binding map");
        assert_eq!(shader.source, LOC_009_UBO_WGSL);
        assert_eq!(
            shader
                .entries
                .iter()
                .map(|entry| (
                    entry.stage,
                    entry.logical_entry_point.as_str(),
                    entry.physical_entry_point.as_str(),
                ))
                .collect::<Vec<_>>(),
            vec![
                (GpuCanvasShaderStage::Vertex, "vs_main", "vs_main"),
                (GpuCanvasShaderStage::Fragment, "fs_main", "fs_main"),
            ],
        );
        assert_eq!(shader.bindings.len(), 1);
        assert_eq!(
            shader.bindings[0].kind,
            GpuCanvasShaderResourceKind::UniformBuffer,
        );
        assert_eq!(shader.bindings[0].stage_mask, 1 << 1);
        assert_eq!(shader.bindings[0].backend_slots, [None, Some(0), None]);
    }

    #[test]
    fn rejects_out_of_bounds_variants_and_missing_entries() {
        let mut payload = loc_009_webgpu_payload();
        payload[10..14].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(decode_shader_asset("scene", &payload).is_err());

        let mut payload = loc_009_webgpu_payload();
        let blob_start = 1 + 8 + 2 * 9;
        payload[blob_start] = 0;
        assert!(decode_shader_asset("scene", &payload).is_err());
    }

    #[test]
    fn selects_webgpu_targets_deterministically_among_retired_variants() {
        let shader = decode_shader_asset(
            "scene",
            &rstb_payload(&[
                (1, b"retired GLSL".to_vec()),
                (WGSL_SOURCE_TARGET, b"superseded WGSL".to_vec()),
                (11, b"retired GLSL binding map".to_vec()),
                (WGSL_BINDING_MAP_TARGET, b"superseded binding map".to_vec()),
                (14, b"retired vertex fixup".to_vec()),
                (WGSL_SOURCE_TARGET, loc_009_source_container()),
                (15, b"retired fragment fixup".to_vec()),
                (WGSL_BINDING_MAP_TARGET, LOC_009_BINDING_MAP.to_vec()),
            ]),
        )
        .expect("the final target-0 descriptor wins exactly as in ShaderAsset::decode");
        assert_eq!(shader.source, LOC_009_UBO_WGSL);
        assert_eq!(shader.bindings.len(), 1);
    }

    #[test]
    fn validates_only_final_last_wins_descriptors_like_cpp() {
        let source = loc_009_source_container();
        let mut blob_data = source.clone();
        let binding_map_offset = blob_data.len();
        blob_data.extend_from_slice(LOC_009_BINDING_MAP);
        let payload = rstb_payload_with_blob(
            &[
                (WGSL_SOURCE_TARGET, u32::MAX as usize, 1),
                (WGSL_BINDING_MAP_TARGET, u32::MAX as usize, 1),
                (WGSL_SOURCE_TARGET, 0, source.len()),
                (
                    WGSL_BINDING_MAP_TARGET,
                    binding_map_offset,
                    LOC_009_BINDING_MAP.len(),
                ),
            ],
            &blob_data,
        );

        let shader = decode_shader_asset("scene", &payload)
            .expect("C++ overwrites duplicate targets before validating final ranges");
        assert_eq!(shader.source, LOC_009_UBO_WGSL);
        assert_eq!(shader.bindings.len(), 1);
    }

    #[test]
    fn preserves_arbitrary_logical_and_physical_entry_names_in_declaration_order() {
        let source = source_container(
            &[
                (0, "alternate_vertex", "vs_main"),
                (0, "default_vertex", "vs_main"),
                (1, "alternate_fragment", "fs_main"),
                (1, "default_fragment", "fs_main"),
            ],
            LOC_009_UBO_WGSL,
        );
        let payload = rstb_payload(&[
            (WGSL_SOURCE_TARGET, source),
            (WGSL_BINDING_MAP_TARGET, LOC_009_BINDING_MAP.to_vec()),
        ]);

        let shader = decode_shader_asset("scene", &payload)
            .expect("entry names are records, not a vs_main/fs_main schema");
        assert_eq!(
            shader
                .entries
                .iter()
                .map(|entry| entry.logical_entry_point.as_str())
                .collect::<Vec<_>>(),
            vec![
                "alternate_vertex",
                "default_vertex",
                "alternate_fragment",
                "default_fragment",
            ],
        );
    }

    #[test]
    fn accepts_cpp_descriptor_aliases_gaps_and_trailing_bytes() {
        let mut source = loc_009_source_container();
        source.extend_from_slice(b"source-extension");
        let source_offset = 3;
        let map_offset = source_offset + source.len() + 5;
        let mut blob_data = b"gap".to_vec();
        blob_data.extend_from_slice(&source);
        blob_data.extend_from_slice(b"gap!!");
        blob_data.extend_from_slice(LOC_009_BINDING_MAP);
        blob_data.extend_from_slice(b"unreferenced-trailing-bytes");
        let payload = rstb_payload_with_blob(
            &[
                // Rive's descriptor table is an index, not a packed stream:
                // retired variants may alias a live range.
                (1, source_offset, source.len()),
                (WGSL_SOURCE_TARGET, source_offset, source.len()),
                (
                    WGSL_BINDING_MAP_TARGET,
                    map_offset,
                    LOC_009_BINDING_MAP.len(),
                ),
            ],
            &blob_data,
        );

        let shader = decode_shader_asset("scene", &payload)
            .expect("pinned C++ accepts aliases, gaps, and unreferenced trailing bytes");
        assert_eq!(shader.source, LOC_009_UBO_WGSL);
        assert_eq!(shader.bindings.len(), 1);
    }

    #[test]
    fn requires_wgsl_source_and_binding_map_targets() {
        let source = loc_009_source_container();
        let missing_source =
            rstb_payload(&[(WGSL_BINDING_MAP_TARGET, LOC_009_BINDING_MAP.to_vec())]);
        assert!(
            decode_shader_asset("scene", &missing_source)
                .expect_err("target 0 is mandatory")
                .to_string()
                .contains("target-0"),
        );

        let missing_map = rstb_payload(&[(WGSL_SOURCE_TARGET, source)]);
        assert!(
            decode_shader_asset("scene", &missing_map)
                .expect_err("target 16 is mandatory")
                .to_string()
                .contains("target-16"),
        );
    }

    #[test]
    fn malformed_binding_maps_fail_closed() {
        let source = loc_009_source_container();
        for malformed in [
            vec![2, 1, 14, 0, 1, 0, 0, 0],
            vec![3, 1, 14, 0, 0, 0, 0, 0],
            vec![2, 1, 13, 0, 0, 0, 0, 0],
            vec![2, 1, 14, 0, 1, 0, 0, 0],
        ] {
            let payload = rstb_payload(&[
                (WGSL_SOURCE_TARGET, source.clone()),
                (WGSL_BINDING_MAP_TARGET, malformed),
            ]);
            assert!(decode_shader_asset("scene", &payload).is_err());
        }
    }

    #[test]
    fn accepts_append_only_binding_map_rows_and_trailing_extension_data() {
        let mut extended_map = LOC_009_BINDING_MAP.to_vec();
        extended_map[2..4].copy_from_slice(&15u16.to_le_bytes());
        extended_map.extend_from_slice(&[0xa5, 0x5a]);
        let payload = rstb_payload(&[
            (WGSL_SOURCE_TARGET, loc_009_source_container()),
            (WGSL_BINDING_MAP_TARGET, extended_map),
        ]);

        let shader = decode_shader_asset("scene", &payload)
            .expect("BindingMap v2 is append-only like the pinned C++ decoder");
        assert_eq!(shader.bindings.len(), 1);
        assert_eq!(shader.bindings[0].backend_slots, [None, Some(0), None]);
    }
}
