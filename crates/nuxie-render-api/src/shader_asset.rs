//! Bounded decoder for the RSTB v4 payload of a Rive `ShaderAsset`.
//!
//! The native shader types, [`GpuCanvasAppleMetalShader`] and
//! [`GpuCanvasVulkanSpirVShader`], are built only here, by
//! [`GpuCanvasAppleMetalShader::decode`] and
//! [`GpuCanvasVulkanSpirVShader::decode`]. Each hashes the exact artifact
//! bytes, checks them against a [`GpuCanvasShaderProvenance`], and reads
//! every part it holds out of those bytes. A value of either type is
//! therefore proof that its code and metadata are the selected targets of
//! one trusted artifact, which is what the backends' unchecked native module
//! creation relies on.
//!
//! The WebGPU and WebGL2 decoders are here too, shared with the scripting
//! host, which reads those targets through the runtime's own index.

use std::collections::HashMap;
use std::sync::Arc;

use sha2::{Digest, Sha256};

use crate::{
    GpuCanvasAppleMetalShader, GpuCanvasShader, GpuCanvasShaderBinding,
    GpuCanvasShaderBindingReflection, GpuCanvasShaderBuiltin, GpuCanvasShaderEntry,
    GpuCanvasShaderEntryReflection, GpuCanvasShaderInterfaceBinding,
    GpuCanvasShaderInterfaceType, GpuCanvasShaderInterfaceVariable,
    GpuCanvasShaderInterpolation, GpuCanvasShaderProvenance, GpuCanvasShaderResourceKind,
    GpuCanvasShaderSampling, GpuCanvasShaderStage, GpuCanvasShaderTextureSampleType,
    GpuCanvasShaderTextureSamplerPair, GpuCanvasShaderTextureViewDimension,
    GpuCanvasVulkanSpirVShader,
};

pub const RSTB_MAGIC: u32 = 0x5253_5442;
pub const RSTB_VERSION: u16 = 4;
pub const WGSL_SOURCE_TARGET: u8 = 0;
pub const WGSL_BINDING_MAP_TARGET: u8 = 16;
pub const WEBGL2_SOURCE_TARGET: u8 = 1;
pub const WEBGL2_BINDING_MAP_TARGET: u8 = 11;
pub const WEBGL2_VERTEX_FIXUP_TARGET: u8 = 14;
pub const WEBGL2_FRAGMENT_FIXUP_TARGET: u8 = 15;
pub const VULKAN_SPIRV_SOURCE_TARGET: u8 = 5;
pub const VULKAN_SPIRV_BINDING_MAP_TARGET: u8 = 13;
pub const APPLE_METAL_SOURCE_TARGET: u8 = 2;
pub const APPLE_METAL_BINDING_MAP_TARGET: u8 = 10;
pub const TEXTURE_SAMPLER_PAIR_SECTION: u8 = 1;
pub const ENTRY_BINDINGS_SECTION: u8 = 2;
pub const SUPPLEMENTAL_REFLECTION_SECTION: u8 = 128;
pub const SUPPLEMENTAL_REFLECTION_VERSION: u8 = 1;
/// Largest RSTB table the host accepts.
pub const MAX_RSTB_BYTES: usize = 4 * 1024 * 1024;
pub const MAX_SHADER_MODULE_BYTES: usize = 1024 * 1024;
const BINDING_MAP_BLOB_VERSION: u8 = 3;
const BINDING_MAP_ALLOCATOR_VERSION: u8 = 2;
const BINDING_MAP_ENTRY_WIRE_SIZE: usize = 18;
const BINDING_MAP_LEGACY_ENTRY_WIRE_SIZE: usize = 14;
const BINDING_MAP_ABSENT: u16 = u16::MAX;
/// Signature bytes after a signed envelope's flags byte.
const ENVELOPE_SIGNATURE_BYTES: usize = 64;

/// Why a `ShaderAsset` payload did not decode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShaderAssetError(String);

impl ShaderAssetError {
    fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl std::fmt::Display for ShaderAssetError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for ShaderAssetError {}

type Result<T> = std::result::Result<T, ShaderAssetError>;

/// The RSTB v4 table inside one signed-content envelope (`[flags:1]
/// [signature:64 if signed] [content]`), read exactly as the runtime's
/// `ShaderAsset::index` reads it: the last descriptor of a target wins, and
/// only the winning descriptors must lie inside the blob data;
/// texture/sampler pair sections add up; the last entry-bindings section
/// wins. A second supplemental-reflection section is malformed.
pub struct RstbIndex<'a> {
    variants: HashMap<u8, &'a [u8]>,
    texture_sampler_pairs: Vec<GpuCanvasShaderTextureSamplerPair>,
    entry_bindings: &'a [u8],
    supplemental_reflection: Option<&'a [u8]>,
}

impl<'a> RstbIndex<'a> {
    /// Reads the table of the envelope `artifact`.
    pub fn parse(artifact: &'a [u8]) -> Result<Self> {
        let (&flags, rest) = artifact
            .split_first()
            .ok_or_else(|| ShaderAssetError::new("signed-content envelope is empty"))?;
        let content = if flags & 0x80 == 0 {
            rest
        } else {
            rest.get(ENVELOPE_SIGNATURE_BYTES..).ok_or_else(|| {
                ShaderAssetError::new("signed-content envelope truncated before signature end")
            })?
        };
        let malformed = || ShaderAssetError::new("ShaderAsset RSTB table is malformed");
        let mut cursor = Cursor::new(content);
        if cursor.read_u32("header").map_err(|_| malformed())? != RSTB_MAGIC
            || cursor.read_u16("header").map_err(|_| malformed())? != RSTB_VERSION
        {
            return Err(malformed());
        }
        let variant_count = cursor.read_u8("header").map_err(|_| malformed())?;
        let section_count = cursor.read_u8("header").map_err(|_| malformed())?;
        let mut descriptors = HashMap::new();
        for _ in 0..variant_count {
            let target = cursor.read_u8("variant").map_err(|_| malformed())?;
            let offset = cursor.read_u32("variant").map_err(|_| malformed())?;
            let size = cursor.read_u32("variant").map_err(|_| malformed())?;
            descriptors.insert(target, (offset, size));
        }
        let mut texture_sampler_pairs = Vec::new();
        let mut entry_bindings: &[u8] = &[];
        let mut supplemental_reflection = None;
        for _ in 0..section_count {
            let tag = cursor.read_u8("section").map_err(|_| malformed())?;
            let length = usize::from(cursor.read_u16("section").map_err(|_| malformed())?);
            let data = cursor.read_bytes(length, "section").map_err(|_| malformed())?;
            match tag {
                TEXTURE_SAMPLER_PAIR_SECTION => {
                    if let Some((&count, pairs)) = data.split_first() {
                        // A section too short for its count is skipped whole.
                        if let Some(pairs) = pairs.get(..usize::from(count) * 4) {
                            texture_sampler_pairs.extend(
                                pairs
                                    .chunks_exact(4)
                                    .filter_map(|pair| <[u8; 4]>::try_from(pair).ok())
                                    .map(|[texture_group, texture_binding, sampler_group, sampler_binding]| {
                                        GpuCanvasShaderTextureSamplerPair {
                                            texture_group,
                                            texture_binding,
                                            sampler_group,
                                            sampler_binding,
                                        }
                                    }),
                            );
                        }
                    }
                }
                ENTRY_BINDINGS_SECTION if !data.is_empty() => entry_bindings = data,
                SUPPLEMENTAL_REFLECTION_SECTION => {
                    if supplemental_reflection.replace(data).is_some() {
                        return Err(ShaderAssetError::new(
                            "duplicate supplemental reflection sections",
                        ));
                    }
                }
                _ => {}
            }
        }
        let blob_data = cursor.rest;
        let mut variants = HashMap::with_capacity(descriptors.len());
        for (target, (offset, size)) in descriptors {
            let start = usize::try_from(offset).map_err(|_| malformed())?;
            let size = usize::try_from(size).map_err(|_| malformed())?;
            let bytes = blob_data
                .get(start..)
                .and_then(|tail| tail.get(..size))
                .ok_or_else(malformed)?;
            variants.insert(target, bytes);
        }
        Ok(Self {
            variants,
            texture_sampler_pairs,
            entry_bindings,
            supplemental_reflection,
        })
    }

    /// The bytes of `target`; `None` when it is absent or empty.
    pub fn variant(&self, target: u8) -> Option<&'a [u8]> {
        self.variants
            .get(&target)
            .copied()
            .filter(|bytes| !bytes.is_empty())
    }

    pub fn texture_sampler_pairs(&self) -> &[GpuCanvasShaderTextureSamplerPair] {
        &self.texture_sampler_pairs
    }

    pub fn entry_bindings(&self) -> &'a [u8] {
        self.entry_bindings
    }

    pub fn supplemental_reflection(&self) -> Option<&'a [u8]> {
        self.supplemental_reflection
    }
}

impl GpuCanvasShaderProvenance {
    /// Whether this provenance was minted for exactly `artifact`.
    pub fn authorizes_artifact(&self, artifact: &[u8]) -> bool {
        u64::try_from(artifact.len()).is_ok_and(|size| size == self.artifact_size)
            && <[u8; 32]>::from(Sha256::digest(artifact)) == self.artifact_sha256
    }
}

impl GpuCanvasVulkanSpirVShader {
    /// Decodes the Vulkan SPIR-V of the `ShaderAsset` whose exact envelope
    /// bytes are `artifact`: whole-module SPIR-V target 5, its target-13
    /// binding map, the entry-binding bytes and texture/sampler pairs.
    /// `provenance` must authorize these bytes. No source translation occurs.
    pub fn decode(
        name: &str,
        artifact: &[u8],
        provenance: GpuCanvasShaderProvenance,
        shader_asset_id: u32,
    ) -> Result<Self> {
        if !provenance.authorizes_artifact(artifact) {
            return Err(ShaderAssetError::new(format!(
                "ShaderAsset '{name}' Vulkan SPIR-V provenance does not authorize this artifact"
            )));
        }
        let rstb = RstbIndex::parse(artifact)?;
        let source_container = rstb.variant(VULKAN_SPIRV_SOURCE_TARGET).ok_or_else(|| {
            ShaderAssetError::new(format!(
                "ShaderAsset '{name}' has no Vulkan RSTB target-5 SPIR-V source"
            ))
        })?;
        let binding_map = rstb
            .variant(VULKAN_SPIRV_BINDING_MAP_TARGET)
            .ok_or_else(|| {
                ShaderAssetError::new(format!(
                    "ShaderAsset '{name}' has no mandatory Vulkan RSTB target-13 binding map"
                ))
            })?;
        let (code, entries) = decode_whole_module_bytes(name, "SPIR-V", source_container)?;
        Ok(Self {
            code: Arc::from(code),
            entries,
            bindings: decode_binding_map(name, binding_map)?,
            binding_map_bytes: Arc::from(binding_map),
            entry_binding_bytes: Arc::from(rstb.entry_bindings()),
            shader_asset_id,
            texture_sampler_pairs: rstb.texture_sampler_pairs().to_vec(),
            provenance,
        })
    }
}

impl GpuCanvasAppleMetalShader {
    /// Decodes the Apple Metal shader of the `ShaderAsset` whose exact
    /// envelope bytes are `artifact`: whole-module MSL target 2, its
    /// target-10 binding map, and the supplemental reflection that carries
    /// the digests of both and is checked against their decoded tables.
    /// `provenance` must authorize these bytes. No source translation occurs.
    pub fn decode(
        name: &str,
        artifact: &[u8],
        provenance: GpuCanvasShaderProvenance,
        shader_asset_id: u32,
    ) -> Result<Self> {
        if !provenance.authorizes_artifact(artifact) {
            return Err(ShaderAssetError::new(format!(
                "ShaderAsset '{name}' Apple Metal provenance does not authorize this artifact"
            )));
        }
        let rstb = RstbIndex::parse(artifact)?;
        let source_container = rstb.variant(APPLE_METAL_SOURCE_TARGET).ok_or_else(|| {
            ShaderAssetError::new(format!(
                "ShaderAsset '{name}' has no Apple Metal RSTB target-2 MSL source"
            ))
        })?;
        let binding_map = rstb
            .variant(APPLE_METAL_BINDING_MAP_TARGET)
            .ok_or_else(|| {
                ShaderAssetError::new(format!(
                    "ShaderAsset '{name}' has no mandatory Apple Metal RSTB target-10 binding map"
                ))
            })?;
        let reflection = rstb.supplemental_reflection().ok_or_else(|| {
            ShaderAssetError::new(format!(
                "ShaderAsset '{name}' has no supplemental reflection section tag 2"
            ))
        })?;
        let (source, entries) = decode_whole_module_source(name, "MSL", source_container)?;
        let bindings = decode_binding_map(name, binding_map)?;
        let (entry_reflection, binding_reflection) = decode_supplemental_reflection(
            name,
            reflection,
            source_container,
            binding_map,
            &entries,
            &bindings,
        )?;
        Ok(Self {
            source,
            entries,
            bindings,
            binding_map_bytes: Arc::from(binding_map),
            entry_binding_bytes: Arc::from(rstb.entry_bindings()),
            entry_reflection,
            binding_reflection,
            shader_asset_id,
            texture_sampler_pairs: rstb.texture_sampler_pairs().to_vec(),
            provenance,
        })
    }
}

/// Reads an RSTB byte sequence front to back. Every read either returns the
/// bytes it asked for or a truncation error; nothing indexes the input.
struct Cursor<'a> {
    rest: &'a [u8],
}

impl<'a> Cursor<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { rest: bytes }
    }

    fn remaining(&self) -> usize {
        self.rest.len()
    }

    fn truncated(label: &str) -> ShaderAssetError {
        ShaderAssetError::new(format!("RSTB is truncated in {label}"))
    }

    fn read_bytes(&mut self, length: usize, label: &str) -> Result<&'a [u8]> {
        let (value, rest) = self
            .rest
            .split_at_checked(length)
            .ok_or_else(|| Self::truncated(label))?;
        self.rest = rest;
        Ok(value)
    }

    fn read_array<const N: usize>(&mut self, label: &str) -> Result<[u8; N]> {
        let (value, rest) = self
            .rest
            .split_first_chunk::<N>()
            .ok_or_else(|| Self::truncated(label))?;
        self.rest = rest;
        Ok(*value)
    }

    fn read_u8(&mut self, label: &str) -> Result<u8> {
        let [value] = self.read_array(label)?;
        Ok(value)
    }

    fn read_u16(&mut self, label: &str) -> Result<u16> {
        self.read_array(label).map(u16::from_le_bytes)
    }

    fn read_u32(&mut self, label: &str) -> Result<u32> {
        self.read_array(label).map(u32::from_le_bytes)
    }

        fn read_u64(&mut self, label: &str) -> Result<u64> {
        self.read_array(label).map(u64::from_le_bytes)
    }

    fn read_string(&mut self, label: &str) -> Result<String> {
        let length = usize::from(self.read_u16(label)?);
        let bytes = self.read_bytes(length, label)?;
        std::str::from_utf8(bytes)
            .map(str::to_owned)
            .map_err(|_| ShaderAssetError::new(format!("RSTB {label} is not UTF-8")))
    }
}

pub fn decode_whole_module_wgsl(
    name: &str,
    source_container: &[u8],
    binding_map: &[u8],
    shader_asset_id: u32,
    texture_sampler_pairs: Vec<GpuCanvasShaderTextureSamplerPair>,
    entry_binding_bytes: std::sync::Arc<[u8]>,
) -> Result<GpuCanvasShader> {
    let (source, entries) = decode_whole_module_source(name, "WGSL", source_container)?;
    Ok(GpuCanvasShader {
        source,
        entries,
        bindings: decode_binding_map(name, &binding_map)?,
        binding_map_bytes: Arc::from(binding_map),
        entry_binding_bytes,
        shader_asset_id,
        texture_sampler_pairs,
    })
}

pub fn decode_per_entry_glsl(
    name: &str,
    source_container: &[u8],
) -> Result<(Vec<GpuCanvasShaderEntry>, Vec<String>)> {
    let mut cursor = Cursor::new(source_container);
    let entry_count = usize::from(cursor.read_u8("GLSL entry count")?);
    if entry_count == 0 {
        return Err(ShaderAssetError::new(format!(
            "ShaderAsset '{name}' GLSL entry table is empty"
        )));
    }
    let mut entries = Vec::with_capacity(entry_count);
    let mut sources = Vec::with_capacity(entry_count);
    for _ in 0..entry_count {
        let stage = match cursor.read_u8("GLSL shader stage")? {
            0 => GpuCanvasShaderStage::Vertex,
            1 => GpuCanvasShaderStage::Fragment,
            2 => GpuCanvasShaderStage::Compute,
            other => {
                return Err(ShaderAssetError::new(format!(
                    "ShaderAsset '{name}' WebGL2 stage {other} is unsupported"
                )));
            }
        };
        let logical_entry_point = cursor.read_string("GLSL logical entry point")?;
        let physical_entry_point = cursor.read_string("GLSL physical entry point")?;
        let source_length = usize::try_from(cursor.read_u32("GLSL source length")?)
            .map_err(|_| ShaderAssetError::new("GLSL source length is not addressable"))?;
        let source_bytes = cursor.read_bytes(source_length, "GLSL source")?;
        // visitShaderAssetModules parses the whole per-entry container, but
        // only vertex/fragment entries create GLSL modules.
        if stage == GpuCanvasShaderStage::Compute {
            continue;
        }
        if logical_entry_point.is_empty() || physical_entry_point != "main" {
            return Err(ShaderAssetError::new(format!(
                "ShaderAsset '{name}' WebGL2 entries require a logical name and physical entry 'main'"
            )));
        }
        if source_length == 0 || source_length > MAX_SHADER_MODULE_BYTES {
            return Err(ShaderAssetError::new(format!(
                "ShaderAsset '{name}' GLSL module size must be between 1 and {MAX_SHADER_MODULE_BYTES} bytes"
            )));
        }
        let source = std::str::from_utf8(source_bytes)
            .map_err(|_| ShaderAssetError::new(format!("ShaderAsset '{name}' GLSL source is not UTF-8")))?
            .to_owned();
        if !source.starts_with("#version 300 es") {
            return Err(ShaderAssetError::new(format!(
                "ShaderAsset '{name}' WebGL2 source is not GLSL ES 3.00"
            )));
        }
        entries.push(GpuCanvasShaderEntry {
            stage,
            logical_entry_point,
            physical_entry_point,
        });
        sources.push(source);
    }
    if cursor.remaining() != 0 {
        return Err(ShaderAssetError::new(format!(
            "ShaderAsset '{name}' GLSL entry container has trailing bytes"
        )));
    }
    Ok((entries, sources))
}

pub fn validate_gl_fixup(name: &str, stage: &str, bytes: &[u8]) -> Result<()> {
    let mut cursor = Cursor::new(bytes);
    if cursor.read_u8("GL fixup version")? != 1 {
        return Err(ShaderAssetError::new(format!(
            "ShaderAsset '{name}' {stage} GL fixup has unsupported version"
        )));
    }
    let count = usize::from(cursor.read_u16("GL fixup count")?);
    for _ in 0..count {
        let kind = cursor.read_u8("GL fixup kind")?;
        if kind > 1 {
            return Err(ShaderAssetError::new(format!(
                "ShaderAsset '{name}' {stage} GL fixup has unknown kind {kind}"
            )));
        }
        let _slot = cursor.read_u8("GL fixup slot")?;
        let entry_name = cursor.read_string("GL fixup name")?;
        if entry_name.is_empty() {
            return Err(ShaderAssetError::new(format!(
                "ShaderAsset '{name}' {stage} GL fixup has an empty name"
            )));
        }
    }
    if cursor.remaining() != 0 {
        return Err(ShaderAssetError::new(format!(
            "ShaderAsset '{name}' {stage} GL fixup has trailing bytes"
        )));
    }
    Ok(())
}

fn decode_whole_module_source(
    name: &str,
    language: &str,
    source_container: &[u8],
) -> Result<(String, Vec<GpuCanvasShaderEntry>)> {
    let mut cursor = Cursor::new(source_container);
    let entry_count = usize::from(cursor.read_u8("shader entry count")?);
    if entry_count == 0 {
        return Err(ShaderAssetError::new(format!(
            "ShaderAsset '{name}' {language} entry table is empty"
        )));
    }
    let mut entries = Vec::with_capacity(entry_count);
    for _ in 0..entry_count {
        let stage = match cursor.read_u8("shader stage")? {
            0 => GpuCanvasShaderStage::Vertex,
            1 => GpuCanvasShaderStage::Fragment,
            2 => GpuCanvasShaderStage::Compute,
            other => {
                return Err(ShaderAssetError::new(format!(
                    "ShaderAsset '{name}' {language} stage {other} is unsupported"
                )));
            }
        };
        entries.push(GpuCanvasShaderEntry {
            stage,
            logical_entry_point: cursor.read_string("shader logical entry point")?,
            physical_entry_point: cursor.read_string("shader physical entry point")?,
        });
    }

    let source_length = usize::try_from(cursor.read_u32("shader source length")?)
        .map_err(|_| ShaderAssetError::new("shader source length is not addressable"))?;
    if source_length == 0 || source_length > MAX_SHADER_MODULE_BYTES {
        return Err(ShaderAssetError::new(format!(
            "ShaderAsset '{name}' {language} module size must be between 1 and {MAX_SHADER_MODULE_BYTES} bytes"
        )));
    }
    let source = std::str::from_utf8(cursor.read_bytes(source_length, "shader source")?)
        .map_err(|_| {
            ShaderAssetError::new(format!(
                "ShaderAsset '{name}' {language} source is not UTF-8"
            ))
        })?
        .to_owned();
    Ok((source, entries))
}

fn decode_whole_module_bytes(
    name: &str,
    language: &str,
    source_container: &[u8],
) -> Result<(Vec<u8>, Vec<GpuCanvasShaderEntry>)> {
    let mut cursor = Cursor::new(source_container);
    let entry_count = usize::from(cursor.read_u8("shader entry count")?);
    if entry_count == 0 {
        return Err(ShaderAssetError::new(format!(
            "ShaderAsset '{name}' {language} entry table is empty"
        )));
    }
    let mut entries = Vec::with_capacity(entry_count);
    for _ in 0..entry_count {
        let stage = match cursor.read_u8("shader stage")? {
            0 => GpuCanvasShaderStage::Vertex,
            1 => GpuCanvasShaderStage::Fragment,
            2 => GpuCanvasShaderStage::Compute,
            other => {
                return Err(ShaderAssetError::new(format!(
                    "ShaderAsset '{name}' {language} stage {other} is unsupported"
                )));
            }
        };
        entries.push(GpuCanvasShaderEntry {
            stage,
            logical_entry_point: cursor.read_string("shader logical entry point")?,
            physical_entry_point: cursor.read_string("shader physical entry point")?,
        });
    }

    let source_length = usize::try_from(cursor.read_u32("shader source length")?)
        .map_err(|_| ShaderAssetError::new("shader source length is not addressable"))?;
    let source = cursor.read_bytes(source_length, "shader source")?.to_vec();
    Ok((source, entries))
}

pub fn decode_binding_map(name: &str, bytes: &[u8]) -> Result<Vec<GpuCanvasShaderBinding>> {
    let mut cursor = Cursor::new(bytes);
    if cursor.read_u8("binding-map blob version")? != BINDING_MAP_BLOB_VERSION {
        return Err(ShaderAssetError::new(format!(
            "ShaderAsset '{name}' has unsupported WGSL binding-map blob version"
        )));
    }
    if cursor.read_u8("binding-map allocator version")? != BINDING_MAP_ALLOCATOR_VERSION {
        return Err(ShaderAssetError::new(format!(
            "ShaderAsset '{name}' has unsupported WGSL binding-map allocator version"
        )));
    }
    let entry_size = usize::from(cursor.read_u16("binding-map entry size")?);
    if entry_size < BINDING_MAP_LEGACY_ENTRY_WIRE_SIZE {
        return Err(ShaderAssetError::new(format!(
            "ShaderAsset '{name}' WGSL binding-map entries are too small"
        )));
    }
    let entry_count = usize::try_from(cursor.read_u32("binding-map entry count")?)
        .map_err(|_| ShaderAssetError::new("binding-map entry count is not addressable"))?;
    let group_size = usize::from(cursor.read_u16("binding-map group size")?);
    let group_count = usize::from(cursor.read_u16("binding-map group count")?);
    if group_count != 0 && group_size < 9 {
        return Err(ShaderAssetError::new("binding-map group rows are too small"));
    }
    let required_bytes = entry_count
        .checked_mul(entry_size)
        .and_then(|entries| group_count.checked_mul(group_size)?.checked_add(entries))
        .ok_or_else(|| ShaderAssetError::new("binding-map byte length overflow"))?;
    if cursor.remaining() < required_bytes {
        return Err(ShaderAssetError::new(format!(
            "ShaderAsset '{name}' WGSL binding map is truncated"
        )));
    }

    let mut bindings = Vec::with_capacity(entry_count);
    let slot = |raw: u16| (raw != BINDING_MAP_ABSENT).then_some(raw);
    for _ in 0..entry_count {
        // A row is at least the legacy 14 bytes (checked above); bytes past the
        // fields this reader knows are append-only extensions and are skipped.
        let mut row = Cursor::new(cursor.read_bytes(entry_size, "binding-map entry")?);
        bindings.push(GpuCanvasShaderBinding {
            group: row.read_u8("binding-map entry")?,
            binding: row.read_u8("binding-map entry")?,
            kind: decode_resource_kind(name, row.read_u8("binding-map entry")?)?,
            stage_mask: row.read_u8("binding-map entry")?,
            backend_space: row.read_u8("binding-map entry")?,
            backend_slots: [
                slot(row.read_u16("binding-map entry")?),
                slot(row.read_u16("binding-map entry")?),
                slot(row.read_u16("binding-map entry")?),
            ],
            texture_view_dimension: decode_texture_view_dimension(
                name,
                row.read_u8("binding-map entry")?,
            )?,
            texture_sample_type: decode_texture_sample_type(
                name,
                row.read_u8("binding-map entry")?,
            )?,
            texture_multisampled: row.read_u8("binding-map entry")? != 0,
            min_binding_size: if entry_size >= BINDING_MAP_ENTRY_WIRE_SIZE {
                row.read_u32("binding-map entry")?
            } else {
                0
            },
        });
    }
    Ok(bindings)
}

fn decode_supplemental_reflection(
    name: &str,
    bytes: &[u8],
    source_container: &[u8],
    binding_map: &[u8],
    entries: &[GpuCanvasShaderEntry],
    bindings: &[GpuCanvasShaderBinding],
) -> Result<(
    Vec<GpuCanvasShaderEntryReflection>,
    Vec<GpuCanvasShaderBindingReflection>,
)> {
    let mut cursor = Cursor::new(bytes);
    if cursor.read_u8("reflection version")? != SUPPLEMENTAL_REFLECTION_VERSION {
        return Err(ShaderAssetError::new(format!(
            "ShaderAsset '{name}' has unsupported supplemental reflection version"
        )));
    }
    let expected_source: [u8; 32] = cursor.read_array("reflection source digest")?;
    let expected_map: [u8; 32] = cursor.read_array("reflection binding-map digest")?;
    if expected_source != <[u8; 32]>::from(Sha256::digest(source_container))
        || expected_map != <[u8; 32]>::from(Sha256::digest(binding_map))
    {
        return Err(ShaderAssetError::new(format!(
            "ShaderAsset '{name}' supplemental reflection does not match target 2/10 bytes"
        )));
    }

    let entry_count = usize::from(cursor.read_u8("reflection entry count")?);
    if entry_count != entries.len() {
        return Err(ShaderAssetError::new(format!(
            "ShaderAsset '{name}' supplemental reflection entry count does not match MSL"
        )));
    }
    let mut entry_reflection = Vec::with_capacity(entry_count);
    for expected in entries {
        let stage = decode_stage(
            name,
            cursor.read_u8("reflection entry stage")?,
            "reflection",
        )?;
        let logical_entry_point = cursor.read_string("reflection logical entry point")?;
        let physical_entry_point = cursor.read_string("reflection physical entry point")?;
        if stage != expected.stage
            || logical_entry_point != expected.logical_entry_point
            || physical_entry_point != expected.physical_entry_point
        {
            return Err(ShaderAssetError::new(format!(
                "ShaderAsset '{name}' supplemental reflection entry does not match MSL entry table"
            )));
        }
        let workgroup_size = [
            cursor.read_u32("reflection workgroup x")?,
            cursor.read_u32("reflection workgroup y")?,
            cursor.read_u32("reflection workgroup z")?,
        ];
        if workgroup_size.contains(&0)
            || (stage != GpuCanvasShaderStage::Compute && workgroup_size != [1, 1, 1])
        {
            return Err(ShaderAssetError::new(format!(
                "ShaderAsset '{name}' has invalid reflected workgroup size"
            )));
        }
        let input_count = usize::from(cursor.read_u8("reflection input count")?);
        let output_count = usize::from(cursor.read_u8("reflection output count")?);
        let inputs = decode_interface_variables(name, &mut cursor, input_count)?;
        let outputs = decode_interface_variables(name, &mut cursor, output_count)?;
        entry_reflection.push(GpuCanvasShaderEntryReflection {
            stage,
            logical_entry_point,
            physical_entry_point,
            workgroup_size,
            inputs,
            outputs,
        });
    }

    let binding_count = usize::from(cursor.read_u16("reflection binding count")?);
    if binding_count != bindings.len() {
        return Err(ShaderAssetError::new(format!(
            "ShaderAsset '{name}' supplemental reflection binding count does not match target 10"
        )));
    }
    let mut binding_reflection = Vec::with_capacity(binding_count);
    for expected in bindings {
        let group = cursor.read_u8("reflection binding group")?;
        let binding = cursor.read_u8("reflection binding index")?;
        let array_count = cursor.read_u16("reflection binding array count")?;
        let min_buffer_size = cursor.read_u64("reflection minimum buffer size")?;
        if (group, binding) != (expected.group, expected.binding) || array_count == 0 {
            return Err(ShaderAssetError::new(format!(
                "ShaderAsset '{name}' supplemental reflection binding does not match target 10"
            )));
        }
        let is_buffer = matches!(
            expected.kind,
            GpuCanvasShaderResourceKind::UniformBuffer
                | GpuCanvasShaderResourceKind::StorageBufferReadOnly
                | GpuCanvasShaderResourceKind::StorageBufferReadWrite
        );
        if is_buffer == (min_buffer_size == 0) {
            return Err(ShaderAssetError::new(format!(
                "ShaderAsset '{name}' supplemental reflection has invalid minimum buffer size"
            )));
        }
        binding_reflection.push(GpuCanvasShaderBindingReflection {
            group,
            binding,
            array_count,
            min_buffer_size,
        });
    }
    if cursor.remaining() != 0 {
        return Err(ShaderAssetError::new(format!(
            "ShaderAsset '{name}' supplemental reflection has trailing bytes"
        )));
    }
    Ok((entry_reflection, binding_reflection))
}

fn decode_interface_variables(
    name: &str,
    cursor: &mut Cursor<'_>,
    count: usize,
) -> Result<Vec<GpuCanvasShaderInterfaceVariable>> {
    let mut result = Vec::with_capacity(count);
    for _ in 0..count {
        let binding_kind = cursor.read_u8("interface binding kind")?;
        let value = cursor.read_u16("interface binding value")?;
        let interface_type = decode_interface_type(name, cursor.read_u8("interface type")?)?;
        let interpolation = cursor.read_u8("interface interpolation")?;
        let sampling = cursor.read_u8("interface sampling")?;
        let binding = match binding_kind {
            0 => GpuCanvasShaderInterfaceBinding::Location {
                location: value,
                interpolation: decode_optional_interpolation(name, interpolation)?,
                sampling: decode_optional_sampling(name, sampling)?,
            },
            1 if interpolation == u8::MAX && sampling == u8::MAX => {
                GpuCanvasShaderInterfaceBinding::Builtin(decode_builtin(name, value)?)
            }
            _ => {
                return Err(ShaderAssetError::new(format!(
                    "ShaderAsset '{name}' has invalid reflected interface binding"
                )));
            }
        };
        if result
            .iter()
            .any(|variable: &GpuCanvasShaderInterfaceVariable| variable.binding == binding)
        {
            return Err(ShaderAssetError::new(format!(
                "ShaderAsset '{name}' has duplicate reflected interface binding"
            )));
        }
        result.push(GpuCanvasShaderInterfaceVariable {
            binding,
            interface_type,
        });
    }
    Ok(result)
}

fn decode_stage(name: &str, value: u8, label: &str) -> Result<GpuCanvasShaderStage> {
    match value {
        0 => Ok(GpuCanvasShaderStage::Vertex),
        1 => Ok(GpuCanvasShaderStage::Fragment),
        2 => Ok(GpuCanvasShaderStage::Compute),
        other => Err(ShaderAssetError::new(format!(
            "ShaderAsset '{name}' {label} stage {other} is unsupported"
        ))),
    }
}

fn decode_interface_type(name: &str, value: u8) -> Result<GpuCanvasShaderInterfaceType> {
    use GpuCanvasShaderInterfaceType::*;
    [
        Float, Float2, Float3, Float4, Sint, Sint2, Sint3, Sint4, Uint, Uint2, Uint3, Uint4, Bool,
    ]
    .get(usize::from(value))
    .copied()
    .ok_or_else(|| {
        ShaderAssetError::new(format!(
            "ShaderAsset '{name}' has invalid reflected interface type"
        ))
    })
}

fn decode_optional_interpolation(
    name: &str,
    value: u8,
) -> Result<Option<GpuCanvasShaderInterpolation>> {
    use GpuCanvasShaderInterpolation::*;
    if value == u8::MAX {
        return Ok(None);
    }
    [Perspective, Linear, Flat]
        .get(usize::from(value))
        .copied()
        .map(Some)
        .ok_or_else(|| {
            ShaderAssetError::new(format!(
                "ShaderAsset '{name}' has invalid reflected interpolation"
            ))
        })
}

fn decode_optional_sampling(name: &str, value: u8) -> Result<Option<GpuCanvasShaderSampling>> {
    use GpuCanvasShaderSampling::*;
    if value == u8::MAX {
        return Ok(None);
    }
    [Center, Centroid, Sample, First, Either]
        .get(usize::from(value))
        .copied()
        .map(Some)
        .ok_or_else(|| {
            ShaderAssetError::new(format!(
                "ShaderAsset '{name}' has invalid reflected sampling"
            ))
        })
}

fn decode_builtin(name: &str, value: u16) -> Result<GpuCanvasShaderBuiltin> {
    use GpuCanvasShaderBuiltin::*;
    [
        VertexIndex,
        InstanceIndex,
        Position,
        FrontFacing,
        FragDepth,
        SampleIndex,
        SampleMask,
    ]
    .get(usize::from(value))
    .copied()
    .ok_or_else(|| {
        ShaderAssetError::new(format!(
            "ShaderAsset '{name}' has invalid reflected builtin"
        ))
    })
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
        other => Err(ShaderAssetError::new(format!(
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
        other => Err(ShaderAssetError::new(format!(
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
        other => Err(ShaderAssetError::new(format!(
            "ShaderAsset '{name}' WGSL binding map has unknown texture sample type {other}"
        ))),
    }
}

