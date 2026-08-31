//! Bounded decoder for the backend-neutral RSTB v4 payload in `ShaderAsset`.
//!
//! Mirrors pinned C++ `src/assets/shader_asset.cpp::ShaderAsset::decode` at
//! registration, then `src/lua/renderer/lua_gpu.cpp::buildShaderEntries` when
//! WebGPU selects authored whole-module WGSL target 0 and its mandatory
//! `BindingMap` target 16 sidecar. Vulkan selects authored whole-module SPIR-V
//! target 5 and its target 13 binding-map sidecar.

#[cfg(any(feature = "android-authored-wgsl", test))]
use nuxie_render_api::GpuCanvasVulkanSpirVShader;
#[cfg(any(feature = "apple-authored-msl", test))]
use nuxie_render_api::{
    GpuCanvasAppleMetalShader, GpuCanvasShaderBindingReflection, GpuCanvasShaderBuiltin,
    GpuCanvasShaderEntryReflection, GpuCanvasShaderInterfaceBinding, GpuCanvasShaderInterfaceType,
    GpuCanvasShaderInterfaceVariable, GpuCanvasShaderInterpolation, GpuCanvasShaderSampling,
};
use nuxie_render_api::{
    GpuCanvasShader, GpuCanvasShaderArtifact, GpuCanvasShaderBinding, GpuCanvasShaderEntry,
    GpuCanvasShaderProfile, GpuCanvasShaderProvenance, GpuCanvasShaderResourceKind,
    GpuCanvasShaderStage, GpuCanvasShaderTextureSampleType, GpuCanvasShaderTextureSamplerPair,
    GpuCanvasShaderTextureViewDimension, GpuCanvasWebGl2Shader,
};
#[cfg(any(
    feature = "apple-authored-msl",
    feature = "android-authored-wgsl",
    test
))]
use sha2::{Digest, Sha256};

use crate::envelope::SignedContent;
use crate::vm::{Error, Result};

const RSTB_MAGIC: u32 = 0x5253_5442;
const RSTB_VERSION: u16 = 4;
const WGSL_SOURCE_TARGET: u8 = 0;
const WGSL_BINDING_MAP_TARGET: u8 = 16;
const WEBGL2_SOURCE_TARGET: u8 = 1;
const WEBGL2_BINDING_MAP_TARGET: u8 = 11;
const WEBGL2_VERTEX_FIXUP_TARGET: u8 = 14;
const WEBGL2_FRAGMENT_FIXUP_TARGET: u8 = 15;
#[cfg(any(feature = "android-authored-wgsl", test))]
const VULKAN_SPIRV_SOURCE_TARGET: u8 = 5;
#[cfg(any(feature = "android-authored-wgsl", test))]
const VULKAN_SPIRV_BINDING_MAP_TARGET: u8 = 13;
const TEXTURE_SAMPLER_PAIR_SECTION: u8 = 1;
#[cfg(any(feature = "apple-authored-msl", test))]
const APPLE_METAL_SOURCE_TARGET: u8 = 2;
#[cfg(any(feature = "apple-authored-msl", test))]
const APPLE_METAL_BINDING_MAP_TARGET: u8 = 10;
#[cfg(any(feature = "apple-authored-msl", test))]
const SUPPLEMENTAL_REFLECTION_SECTION: u8 = 2;
#[cfg(any(feature = "apple-authored-msl", test))]
const SUPPLEMENTAL_REFLECTION_VERSION: u8 = 1;
const MAX_RSTB_BYTES: usize = 4 * 1024 * 1024;
const MAX_SHADER_MODULE_BYTES: usize = 1024 * 1024;
const BINDING_MAP_BLOB_VERSION: u8 = 2;
const BINDING_MAP_ALLOCATOR_VERSION: u8 = 1;
const BINDING_MAP_ENTRY_WIRE_SIZE: usize = 14;
const BINDING_MAP_ABSENT: u16 = u16::MAX;

/// Backend target selection over the actual translated ShaderAsset owner.
pub(crate) struct ShaderAsset {
    asset: nuxie_runtime::mechanical_port::source::core::CoreHandle,
    _standalone_arena: Option<nuxie_runtime::mechanical_port::source::core::CoreArena>,
    #[cfg(any(feature = "apple-authored-msl", test))]
    supplemental_reflection: Option<Vec<u8>>,
    #[cfg(any(
        feature = "apple-authored-msl",
        feature = "android-authored-wgsl",
        test
    ))]
    artifact_size: u64,
    #[cfg(any(
        feature = "apple-authored-msl",
        feature = "android-authored-wgsl",
        test
    ))]
    artifact_sha256: [u8; 32],
}
impl std::fmt::Debug for ShaderAsset {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ShaderAsset")
            .field("native", &self.asset)
            .finish_non_exhaustive()
    }
}
type NativeShaderAsset = nuxie_runtime::mechanical_port::source::assets::shader_asset::ShaderAsset;
impl ShaderAsset {
    pub(crate) fn decode(name: &str, payload: &[u8]) -> Result<Self> {
        use nuxie_runtime::mechanical_port::source::{
            core::CoreArena, factory::RuntimeFactoryHandle,
        };
        let mut factory =
            nuxie_render_api::PersistentFactory::new(nuxie_render_api::NullFactory::new());
        let factory = RuntimeFactoryHandle::from_factory(&mut factory)
            .expect("persistent shader decode factory");
        let mut asset = NativeShaderAsset::default();
        if !asset.decode(payload, &factory) {
            return Err(Error::runtime(format!(
                "ShaderAsset '{name}' neutral decode failed"
            )));
        }
        let arena = CoreArena::default();
        let handle = arena.insert(asset);
        let mut result = Self::from_native(handle)?;
        result._standalone_arena = Some(arena);
        Ok(result)
    }
    pub(crate) fn from_native(
        asset: nuxie_runtime::mechanical_port::source::core::CoreHandle,
    ) -> Result<Self> {
        let payload = asset
            .with_downcast::<NativeShaderAsset, _>(|asset| asset.encoded_payload().to_vec())
            .ok_or_else(|| Error::runtime("missing native ShaderAsset"))?;
        if asset
            .with_downcast::<NativeShaderAsset, _>(|asset| asset.content_bytes().len())
            .unwrap_or(0)
            > MAX_RSTB_BYTES
        {
            return Err(Error::runtime("ShaderAsset exceeds the host byte limit"));
        }
        #[cfg(any(feature = "apple-authored-msl", test))]
        let supplemental_reflection = {
            // Supplemental reflection is a host-only authenticated MSL extension.
            // Do not rebuild the native owner's target index or pair table.
            let content = SignedContent::parse(&payload)
                .map_err(|e| Error::runtime(e.to_string()))?
                .content;
            let mut cursor = Cursor::new(content);
            cursor.read_bytes(6, "header")?;
            let variants = cursor.read_u8("variant count")? as usize;
            let sections = cursor.read_u8("section count")?;
            cursor.read_bytes(variants * 9, "variant descriptors")?;
            let mut reflection = None;
            for _ in 0..sections {
                let tag = cursor.read_u8("section tag")?;
                let length = cursor.read_u16("section length")? as usize;
                let data = cursor.read_bytes(length, "section payload")?;
                if tag == SUPPLEMENTAL_REFLECTION_SECTION {
                    if reflection.is_some() {
                        return Err(Error::runtime("duplicate supplemental reflection sections"));
                    }
                    reflection = Some(data.to_vec());
                }
            }
            reflection
        };
        Ok(Self {
            asset,
            _standalone_arena: None,
            #[cfg(any(feature = "apple-authored-msl", test))]
            supplemental_reflection,
            #[cfg(any(
                feature = "apple-authored-msl",
                feature = "android-authored-wgsl",
                test
            ))]
            artifact_size: u64::try_from(payload.len())
                .map_err(|_| Error::runtime("ShaderAsset length does not fit in u64"))?,
            #[cfg(any(
                feature = "apple-authored-msl",
                feature = "android-authored-wgsl",
                test
            ))]
            artifact_sha256: Sha256::digest(payload).into(),
        })
    }
    pub(crate) fn decode_webgpu(&self, name: &str) -> Result<GpuCanvasShader> {
        let wgsl = self.variant(WGSL_SOURCE_TARGET).ok_or_else(|| {
            Error::runtime(format!(
                "ShaderAsset '{name}' has no WebGPU RSTB target-0 WGSL source"
            ))
        })?;
        let binding_map = self.variant(WGSL_BINDING_MAP_TARGET).ok_or_else(|| {
            Error::runtime(format!(
                "ShaderAsset '{name}' has no mandatory WebGPU RSTB target-16 binding map"
            ))
        })?;
        decode_whole_module_wgsl(
            name,
            &wgsl,
            &binding_map,
            self.asset
                .with_downcast::<NativeShaderAsset, _>(|asset| asset.base.asset_id())
                .ok_or_else(|| Error::runtime("missing native ShaderAsset"))?,
            self.texture_sampler_pairs(),
        )
    }

    pub(crate) fn decode_webgl2(&self, name: &str) -> Result<GpuCanvasWebGl2Shader> {
        let source = self.variant(WEBGL2_SOURCE_TARGET).ok_or_else(|| {
            Error::runtime(format!(
                "ShaderAsset '{name}' has no WebGL2 RSTB target-1 GLSL source"
            ))
        })?;
        let binding_map = self.variant(WEBGL2_BINDING_MAP_TARGET).ok_or_else(|| {
            Error::runtime(format!(
                "ShaderAsset '{name}' has no mandatory WebGL2 RSTB target-11 binding map"
            ))
        })?;
        let vertex_fixup = self.variant(WEBGL2_VERTEX_FIXUP_TARGET).ok_or_else(|| {
            Error::runtime(format!(
                "ShaderAsset '{name}' has no mandatory WebGL2 target-14 vertex fixup"
            ))
        })?;
        let fragment_fixup = self.variant(WEBGL2_FRAGMENT_FIXUP_TARGET).ok_or_else(|| {
            Error::runtime(format!(
                "ShaderAsset '{name}' has no mandatory WebGL2 target-15 fragment fixup"
            ))
        })?;
        let (entries, sources) = decode_per_entry_glsl(name, &source)?;
        validate_gl_fixup(name, "vertex", &vertex_fixup)?;
        validate_gl_fixup(name, "fragment", &fragment_fixup)?;
        Ok(GpuCanvasWebGl2Shader {
            entries,
            sources,
            bindings: decode_binding_map(name, &binding_map)?,
            binding_map_bytes: std::sync::Arc::from(binding_map),
            vertex_gl_fixup_bytes: std::sync::Arc::from(vertex_fixup),
            fragment_gl_fixup_bytes: std::sync::Arc::from(fragment_fixup),
            shader_asset_id: self
                .asset
                .with_downcast::<NativeShaderAsset, _>(|asset| asset.base.asset_id())
                .ok_or_else(|| Error::runtime("missing native ShaderAsset"))?,
            texture_sampler_pairs: self.texture_sampler_pairs(),
        })
    }

    pub(crate) fn decode_for_profile(
        &self,
        name: &str,
        profile: GpuCanvasShaderProfile,
        provenance: Option<GpuCanvasShaderProvenance>,
    ) -> Result<GpuCanvasShaderArtifact> {
        match profile {
            GpuCanvasShaderProfile::WebGpu => {
                #[cfg(feature = "android-authored-wgsl")]
                {
                    self.decode_android_wgsl(name, provenance)
                        .map(GpuCanvasShaderArtifact::WebGpu)
                }
                #[cfg(not(feature = "android-authored-wgsl"))]
                {
                    let _ = provenance;
                    self.decode_webgpu(name)
                        .map(GpuCanvasShaderArtifact::WebGpu)
                }
            }
            GpuCanvasShaderProfile::WebGl2 => self
                .decode_webgl2(name)
                .map(GpuCanvasShaderArtifact::WebGl2),
            #[cfg(any(feature = "apple-authored-msl", test))]
            GpuCanvasShaderProfile::TrustedAppleMetal => self
                .decode_apple_metal(name, provenance)
                .map(GpuCanvasShaderArtifact::TrustedAppleMetal),
            #[cfg(not(any(feature = "apple-authored-msl", test)))]
            GpuCanvasShaderProfile::TrustedAppleMetal => {
                let _ = provenance;
                Err(Error::runtime(format!(
                    "ShaderAsset '{name}' Apple Metal support is not compiled"
                )))
            }
            #[cfg(any(feature = "android-authored-wgsl", test))]
            GpuCanvasShaderProfile::TrustedVulkanSpirV => self
                .decode_vulkan_spirv(name, provenance)
                .map(GpuCanvasShaderArtifact::TrustedVulkanSpirV),
            #[cfg(not(any(feature = "android-authored-wgsl", test)))]
            GpuCanvasShaderProfile::TrustedVulkanSpirV => {
                let _ = provenance;
                Err(Error::runtime(format!(
                    "ShaderAsset '{name}' Vulkan SPIR-V support is not compiled"
                )))
            }
        }
    }

    #[cfg(feature = "android-authored-wgsl")]
    fn decode_android_wgsl(
        &self,
        name: &str,
        provenance: Option<GpuCanvasShaderProvenance>,
    ) -> Result<GpuCanvasShader> {
        let provenance = provenance.ok_or_else(|| {
            Error::runtime(format!(
                "ShaderAsset '{name}' has no verified provenance for Android WGSL"
            ))
        })?;
        if !provenance.authorizes_digest(self.artifact_size, &self.artifact_sha256) {
            return Err(Error::runtime(format!(
                "ShaderAsset '{name}' Android WGSL provenance does not authorize this artifact"
            )));
        }
        self.decode_webgpu(name)
    }

    #[cfg(any(feature = "android-authored-wgsl", test))]
    fn decode_vulkan_spirv(
        &self,
        name: &str,
        provenance: Option<GpuCanvasShaderProvenance>,
    ) -> Result<GpuCanvasVulkanSpirVShader> {
        let provenance = provenance.ok_or_else(|| {
            Error::runtime(format!(
                "ShaderAsset '{name}' has no verified provenance for Vulkan SPIR-V"
            ))
        })?;
        if !provenance.authorizes_digest(self.artifact_size, &self.artifact_sha256) {
            return Err(Error::runtime(format!(
                "ShaderAsset '{name}' Vulkan SPIR-V provenance does not authorize this artifact"
            )));
        }
        let source_container = self.variant(VULKAN_SPIRV_SOURCE_TARGET).ok_or_else(|| {
            Error::runtime(format!(
                "ShaderAsset '{name}' has no Vulkan RSTB target-5 SPIR-V source"
            ))
        })?;
        let binding_map = self
            .variant(VULKAN_SPIRV_BINDING_MAP_TARGET)
            .ok_or_else(|| {
                Error::runtime(format!(
                    "ShaderAsset '{name}' has no mandatory Vulkan RSTB target-13 binding map"
                ))
            })?;
        let (code, entries) = decode_whole_module_bytes(name, "SPIR-V", &source_container)?;
        let bindings = decode_binding_map(name, &binding_map)?;
        let shader_asset_id = self
            .asset
            .with_downcast::<NativeShaderAsset, _>(|asset| asset.base.asset_id())
            .ok_or_else(|| Error::runtime("missing native ShaderAsset"))?;
        // SAFETY: the selected target-5 module, ordered entry records, target-13
        // sidecar, asset id, and texture/sampler pairs all come directly from
        // this exact authenticated ShaderAsset. No source translation occurs.
        unsafe {
            GpuCanvasVulkanSpirVShader::from_verified_parts(
                provenance,
                self.artifact_size,
                self.artifact_sha256,
                std::sync::Arc::from(code),
                entries,
                bindings,
                std::sync::Arc::from(binding_map),
                shader_asset_id,
                self.texture_sampler_pairs(),
            )
        }
        .ok_or_else(|| {
            Error::runtime(format!(
                "ShaderAsset '{name}' Vulkan SPIR-V provenance was rejected"
            ))
        })
    }

    #[cfg(any(feature = "apple-authored-msl", test))]
    fn decode_apple_metal(
        &self,
        name: &str,
        provenance: Option<GpuCanvasShaderProvenance>,
    ) -> Result<GpuCanvasAppleMetalShader> {
        let provenance = provenance.ok_or_else(|| {
            Error::runtime(format!(
                "ShaderAsset '{name}' has no verified provenance for Apple Metal"
            ))
        })?;
        if !provenance.authorizes_digest(self.artifact_size, &self.artifact_sha256) {
            return Err(Error::runtime(format!(
                "ShaderAsset '{name}' Apple Metal provenance does not authorize this artifact"
            )));
        }
        let source_container = self.variant(APPLE_METAL_SOURCE_TARGET).ok_or_else(|| {
            Error::runtime(format!(
                "ShaderAsset '{name}' has no Apple Metal RSTB target-2 MSL source"
            ))
        })?;
        let binding_map = self
            .variant(APPLE_METAL_BINDING_MAP_TARGET)
            .ok_or_else(|| {
                Error::runtime(format!(
                    "ShaderAsset '{name}' has no mandatory Apple Metal RSTB target-10 binding map"
                ))
            })?;
        let reflection = self.supplemental_reflection.as_deref().ok_or_else(|| {
            Error::runtime(format!(
                "ShaderAsset '{name}' has no supplemental reflection section tag 2"
            ))
        })?;
        let (source, entries) = decode_whole_module_source(name, "MSL", &source_container)?;
        let bindings = decode_binding_map(name, &binding_map)?;
        let (entry_reflection, binding_reflection) = decode_supplemental_reflection(
            name,
            reflection,
            &source_container,
            &binding_map,
            &entries,
            &bindings,
        )?;
        // SAFETY: all native code, entry/map data, and supplemental reflection
        // above were decoded from this exact authenticated ShaderAsset. The
        // reflection carries and passed digests for the selected target-2 and
        // target-10 byte ranges, and its tables were cross-checked against
        // both decoded variants before construction.
        unsafe {
            GpuCanvasAppleMetalShader::from_verified_parts(
                provenance,
                self.artifact_size,
                self.artifact_sha256,
                source,
                entries,
                bindings,
                std::sync::Arc::from(binding_map),
                entry_reflection,
                binding_reflection,
                self.asset
                    .with_downcast::<NativeShaderAsset, _>(|asset| asset.base.asset_id())
                    .unwrap_or(0),
                self.texture_sampler_pairs(),
            )
        }
        .ok_or_else(|| {
            Error::runtime(format!(
                "ShaderAsset '{name}' Apple Metal provenance was rejected"
            ))
        })
    }

    fn variant(&self, target: u8) -> Option<Vec<u8>> {
        self.asset
            .with_downcast::<NativeShaderAsset, _>(|asset| {
                let bytes = asset.find_shader(target);
                (!bytes.is_empty()).then(|| bytes.to_vec())
            })
            .flatten()
    }
    fn texture_sampler_pairs(&self) -> Vec<GpuCanvasShaderTextureSamplerPair> {
        self.asset
            .with_downcast::<NativeShaderAsset, _>(|asset| {
                asset
                    .texture_sampler_pairs()
                    .iter()
                    .map(|pair| GpuCanvasShaderTextureSamplerPair {
                        texture_group: pair.tex_group,
                        texture_binding: pair.tex_binding,
                        sampler_group: pair.samp_group,
                        sampler_binding: pair.samp_binding,
                    })
                    .collect()
            })
            .unwrap_or_default()
    }
}

/// Decode one trusted, precompiled browser `ShaderAsset` for the renderer
/// profile selected by the caller. Source translation is intentionally absent:
/// the payload must already contain the exact RSTB target and sidecars required
/// by WebGPU or WebGL2.
pub fn decode_browser_shader_asset(
    name: &str,
    payload: &[u8],
    profile: GpuCanvasShaderProfile,
) -> Result<GpuCanvasShaderArtifact> {
    if matches!(
        profile,
        GpuCanvasShaderProfile::TrustedAppleMetal | GpuCanvasShaderProfile::TrustedVulkanSpirV
    ) {
        return Err(Error::runtime(
            "browser ShaderAsset decoding does not authorize native shader code",
        ));
    }
    ShaderAsset::decode(name, payload)?.decode_for_profile(name, profile, None)
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

    #[cfg(any(feature = "apple-authored-msl", test))]
    fn read_u64(&mut self, label: &str) -> Result<u64> {
        let bytes: [u8; 8] = self
            .read_bytes(8, label)?
            .try_into()
            .map_err(|_| Error::runtime(format!("RSTB is truncated in {label}")))?;
        Ok(u64::from_le_bytes(bytes))
    }

    fn read_string(&mut self, label: &str) -> Result<String> {
        let length = usize::from(self.read_u16(label)?);
        let bytes = self.read_bytes(length, label)?;
        std::str::from_utf8(bytes)
            .map(str::to_owned)
            .map_err(|_| Error::runtime(format!("RSTB {label} is not UTF-8")))
    }
}

#[cfg(test)]
pub(crate) fn decode_shader_asset(name: &str, payload: &[u8]) -> Result<GpuCanvasShader> {
    ShaderAsset::decode(name, payload)?.decode_webgpu(name)
}

fn decode_whole_module_wgsl(
    name: &str,
    source_container: &[u8],
    binding_map: &[u8],
    shader_asset_id: u32,
    texture_sampler_pairs: Vec<GpuCanvasShaderTextureSamplerPair>,
) -> Result<GpuCanvasShader> {
    let (source, entries) = decode_whole_module_source(name, "WGSL", source_container)?;
    Ok(GpuCanvasShader {
        source,
        entries,
        bindings: decode_binding_map(name, &binding_map)?,
        binding_map_bytes: std::sync::Arc::from(binding_map),
        shader_asset_id,
        texture_sampler_pairs,
    })
}

fn decode_per_entry_glsl(
    name: &str,
    source_container: &[u8],
) -> Result<(Vec<GpuCanvasShaderEntry>, Vec<String>)> {
    let mut cursor = Cursor::new(source_container);
    let entry_count = usize::from(cursor.read_u8("GLSL entry count")?);
    if entry_count == 0 {
        return Err(Error::runtime(format!(
            "ShaderAsset '{name}' GLSL entry table is empty"
        )));
    }
    let mut entries = Vec::with_capacity(entry_count);
    let mut sources = Vec::with_capacity(entry_count);
    for _ in 0..entry_count {
        let stage = match cursor.read_u8("GLSL shader stage")? {
            0 => GpuCanvasShaderStage::Vertex,
            1 => GpuCanvasShaderStage::Fragment,
            other => {
                return Err(Error::runtime(format!(
                    "ShaderAsset '{name}' WebGL2 stage {other} is unsupported"
                )));
            }
        };
        let logical_entry_point = cursor.read_string("GLSL logical entry point")?;
        let physical_entry_point = cursor.read_string("GLSL physical entry point")?;
        if logical_entry_point.is_empty() || physical_entry_point != "main" {
            return Err(Error::runtime(format!(
                "ShaderAsset '{name}' WebGL2 entries require a logical name and physical entry 'main'"
            )));
        }
        let source_length = usize::try_from(cursor.read_u32("GLSL source length")?)
            .map_err(|_| Error::runtime("GLSL source length is not addressable"))?;
        if source_length == 0 || source_length > MAX_SHADER_MODULE_BYTES {
            return Err(Error::runtime(format!(
                "ShaderAsset '{name}' GLSL module size must be between 1 and {MAX_SHADER_MODULE_BYTES} bytes"
            )));
        }
        let source = std::str::from_utf8(cursor.read_bytes(source_length, "GLSL source")?)
            .map_err(|_| Error::runtime(format!("ShaderAsset '{name}' GLSL source is not UTF-8")))?
            .to_owned();
        if !source.starts_with("#version 300 es") {
            return Err(Error::runtime(format!(
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
        return Err(Error::runtime(format!(
            "ShaderAsset '{name}' GLSL entry container has trailing bytes"
        )));
    }
    Ok((entries, sources))
}

fn validate_gl_fixup(name: &str, stage: &str, bytes: &[u8]) -> Result<()> {
    let mut cursor = Cursor::new(bytes);
    if cursor.read_u8("GL fixup version")? != 1 {
        return Err(Error::runtime(format!(
            "ShaderAsset '{name}' {stage} GL fixup has unsupported version"
        )));
    }
    let count = usize::from(cursor.read_u16("GL fixup count")?);
    for _ in 0..count {
        let kind = cursor.read_u8("GL fixup kind")?;
        if kind > 1 {
            return Err(Error::runtime(format!(
                "ShaderAsset '{name}' {stage} GL fixup has unknown kind {kind}"
            )));
        }
        let _slot = cursor.read_u8("GL fixup slot")?;
        let entry_name = cursor.read_string("GL fixup name")?;
        if entry_name.is_empty() {
            return Err(Error::runtime(format!(
                "ShaderAsset '{name}' {stage} GL fixup has an empty name"
            )));
        }
    }
    if cursor.remaining() != 0 {
        return Err(Error::runtime(format!(
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
        return Err(Error::runtime(format!(
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
                return Err(Error::runtime(format!(
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
        .map_err(|_| Error::runtime("shader source length is not addressable"))?;
    if source_length == 0 || source_length > MAX_SHADER_MODULE_BYTES {
        return Err(Error::runtime(format!(
            "ShaderAsset '{name}' {language} module size must be between 1 and {MAX_SHADER_MODULE_BYTES} bytes"
        )));
    }
    let source = std::str::from_utf8(cursor.read_bytes(source_length, "shader source")?)
        .map_err(|_| {
            Error::runtime(format!(
                "ShaderAsset '{name}' {language} source is not UTF-8"
            ))
        })?
        .to_owned();
    Ok((source, entries))
}

#[cfg(any(feature = "android-authored-wgsl", test))]
fn decode_whole_module_bytes(
    name: &str,
    language: &str,
    source_container: &[u8],
) -> Result<(Vec<u8>, Vec<GpuCanvasShaderEntry>)> {
    let mut cursor = Cursor::new(source_container);
    let entry_count = usize::from(cursor.read_u8("shader entry count")?);
    if entry_count == 0 {
        return Err(Error::runtime(format!(
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
                return Err(Error::runtime(format!(
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
        .map_err(|_| Error::runtime("shader source length is not addressable"))?;
    let source = cursor.read_bytes(source_length, "shader source")?.to_vec();
    Ok((source, entries))
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

#[cfg(any(feature = "apple-authored-msl", test))]
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
        return Err(Error::runtime(format!(
            "ShaderAsset '{name}' has unsupported supplemental reflection version"
        )));
    }
    let expected_source: [u8; 32] = cursor
        .read_bytes(32, "reflection source digest")?
        .try_into()
        .map_err(|_| Error::runtime("reflection source digest is truncated"))?;
    let expected_map: [u8; 32] = cursor
        .read_bytes(32, "reflection binding-map digest")?
        .try_into()
        .map_err(|_| Error::runtime("reflection binding-map digest is truncated"))?;
    if expected_source != <[u8; 32]>::from(Sha256::digest(source_container))
        || expected_map != <[u8; 32]>::from(Sha256::digest(binding_map))
    {
        return Err(Error::runtime(format!(
            "ShaderAsset '{name}' supplemental reflection does not match target 2/10 bytes"
        )));
    }

    let entry_count = usize::from(cursor.read_u8("reflection entry count")?);
    if entry_count != entries.len() {
        return Err(Error::runtime(format!(
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
            return Err(Error::runtime(format!(
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
            return Err(Error::runtime(format!(
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
        return Err(Error::runtime(format!(
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
            return Err(Error::runtime(format!(
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
            return Err(Error::runtime(format!(
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
        return Err(Error::runtime(format!(
            "ShaderAsset '{name}' supplemental reflection has trailing bytes"
        )));
    }
    Ok((entry_reflection, binding_reflection))
}

#[cfg(any(feature = "apple-authored-msl", test))]
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
                return Err(Error::runtime(format!(
                    "ShaderAsset '{name}' has invalid reflected interface binding"
                )));
            }
        };
        if result
            .iter()
            .any(|variable: &GpuCanvasShaderInterfaceVariable| variable.binding == binding)
        {
            return Err(Error::runtime(format!(
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

#[cfg(any(feature = "apple-authored-msl", test))]
fn decode_stage(name: &str, value: u8, label: &str) -> Result<GpuCanvasShaderStage> {
    match value {
        0 => Ok(GpuCanvasShaderStage::Vertex),
        1 => Ok(GpuCanvasShaderStage::Fragment),
        2 => Ok(GpuCanvasShaderStage::Compute),
        other => Err(Error::runtime(format!(
            "ShaderAsset '{name}' {label} stage {other} is unsupported"
        ))),
    }
}

#[cfg(any(feature = "apple-authored-msl", test))]
fn decode_interface_type(name: &str, value: u8) -> Result<GpuCanvasShaderInterfaceType> {
    use GpuCanvasShaderInterfaceType::*;
    [
        Float, Float2, Float3, Float4, Sint, Sint2, Sint3, Sint4, Uint, Uint2, Uint3, Uint4, Bool,
    ]
    .get(usize::from(value))
    .copied()
    .ok_or_else(|| {
        Error::runtime(format!(
            "ShaderAsset '{name}' has invalid reflected interface type"
        ))
    })
}

#[cfg(any(feature = "apple-authored-msl", test))]
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
            Error::runtime(format!(
                "ShaderAsset '{name}' has invalid reflected interpolation"
            ))
        })
}

#[cfg(any(feature = "apple-authored-msl", test))]
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
            Error::runtime(format!(
                "ShaderAsset '{name}' has invalid reflected sampling"
            ))
        })
}

#[cfg(any(feature = "apple-authored-msl", test))]
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
        Error::runtime(format!(
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

    const IMPORTED_GPU_CANVAS_UBO_WGSL: &str =
        include_str!("../tests/fixtures/imported-gpu-canvas-ubo-triangle.wgsl");
    const IMPORTED_GPU_CANVAS_BINDING_MAP: &[u8] = &[
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
        byte_source_container(entries, wgsl.as_bytes())
    }

    fn byte_source_container(entries: &[(u8, &str, &str)], source_bytes: &[u8]) -> Vec<u8> {
        let mut source = vec![entries.len() as u8];
        for (stage, logical, physical) in entries {
            source.push(*stage);
            put_string(&mut source, logical);
            put_string(&mut source, physical);
        }
        put_u32(&mut source, source_bytes.len() as u32);
        source.extend_from_slice(source_bytes);
        source
    }

    fn per_entry_source_container(entries: &[(u8, &str, &str, &str)]) -> Vec<u8> {
        let mut source = vec![entries.len() as u8];
        for (stage, logical, physical, glsl) in entries {
            source.push(*stage);
            put_string(&mut source, logical);
            put_string(&mut source, physical);
            put_u32(&mut source, glsl.len() as u32);
            source.extend_from_slice(glsl.as_bytes());
        }
        source
    }

    fn empty_gl_fixup() -> Vec<u8> {
        vec![1, 0, 0]
    }

    fn imported_gpu_canvas_source_container() -> Vec<u8> {
        source_container(
            &[(0, "vs_main", "vs_main"), (1, "fs_main", "fs_main")],
            IMPORTED_GPU_CANVAS_UBO_WGSL,
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
        rstb_payload_with_sections(variants, &[])
    }

    fn rstb_payload_with_sections(
        variants: &[(u8, Vec<u8>)],
        sections: &[(u8, Vec<u8>)],
    ) -> Vec<u8> {
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
        let mut payload = vec![0];
        put_u32(&mut payload, RSTB_MAGIC);
        put_u16(&mut payload, RSTB_VERSION);
        payload.extend_from_slice(&[descriptors.len() as u8, sections.len() as u8]);
        for (target, offset, size) in descriptors {
            payload.push(target);
            put_u32(&mut payload, offset as u32);
            put_u32(&mut payload, size as u32);
        }
        for (tag, section) in sections {
            payload.push(*tag);
            put_u16(&mut payload, section.len() as u16);
            payload.extend_from_slice(section);
        }
        payload.extend_from_slice(&blob_data);
        payload
    }

    fn native_binding_map() -> Vec<u8> {
        let mut map = vec![2, 1];
        put_u16(&mut map, 14);
        put_u32(&mut map, 3);
        // group, binding, kind, stages, space, vertex/fragment/compute slots,
        // texture dimension/sample type/multisampled.
        map.extend_from_slice(&[0, 0, 0, 3, 0, 0, 0, 0, 0, 0xff, 0xff, 0, 0, 0]);
        map.extend_from_slice(&[1, 2, 3, 2, 1, 0xff, 0xff, 3, 0, 0xff, 0xff, 2, 1, 0]);
        map.extend_from_slice(&[2, 1, 5, 4, 2, 0xff, 0xff, 0xff, 0xff, 1, 0, 0, 0, 0]);
        map
    }

    fn put_interface(
        bytes: &mut Vec<u8>,
        kind: u8,
        value: u16,
        interface_type: u8,
        interpolation: u8,
        sampling: u8,
    ) {
        bytes.push(kind);
        put_u16(bytes, value);
        bytes.extend_from_slice(&[interface_type, interpolation, sampling]);
    }

    fn native_reflection(source: &[u8], map: &[u8]) -> Vec<u8> {
        let mut bytes = vec![SUPPLEMENTAL_REFLECTION_VERSION];
        bytes.extend_from_slice(&Sha256::digest(source));
        bytes.extend_from_slice(&Sha256::digest(map));
        bytes.push(3);
        // vertex: vertex_index -> position + location(0)
        bytes.push(0);
        put_string(&mut bytes, "vertex");
        put_string(&mut bytes, "vs_native");
        for dimension in [1u32; 3] {
            put_u32(&mut bytes, dimension);
        }
        bytes.extend_from_slice(&[1, 2]);
        put_interface(&mut bytes, 1, 0, 8, 0xff, 0xff);
        put_interface(&mut bytes, 1, 2, 3, 0xff, 0xff);
        put_interface(&mut bytes, 0, 0, 1, 0, 0);
        // fragment: interpolated location(0) -> location(0)
        bytes.push(1);
        put_string(&mut bytes, "fragment");
        put_string(&mut bytes, "fs_native");
        for dimension in [1u32; 3] {
            put_u32(&mut bytes, dimension);
        }
        bytes.extend_from_slice(&[1, 1]);
        put_interface(&mut bytes, 0, 0, 1, 0, 0);
        put_interface(&mut bytes, 0, 0, 3, 0, 0);
        // compute entry with a nontrivial workgroup.
        bytes.push(2);
        put_string(&mut bytes, "compute");
        put_string(&mut bytes, "cs_native");
        for dimension in [8u32, 4, 2] {
            put_u32(&mut bytes, dimension);
        }
        bytes.extend_from_slice(&[0, 0]);
        put_u16(&mut bytes, 3);
        for (group, binding, array_count, min_size) in
            [(0u8, 0u8, 1u16, 64u64), (1, 2, 4, 0), (2, 1, 2, 0)]
        {
            bytes.extend_from_slice(&[group, binding]);
            put_u16(&mut bytes, array_count);
            bytes.extend_from_slice(&min_size.to_le_bytes());
        }
        bytes
    }

    fn native_payload() -> Vec<u8> {
        let source = source_container(
            &[
                (0, "vertex", "vs_native"),
                (1, "fragment", "fs_native"),
                (2, "compute", "cs_native"),
            ],
            "#include <metal_stdlib>\nusing namespace metal;",
        );
        let map = native_binding_map();
        let reflection = native_reflection(&source, &map);
        rstb_payload_with_sections(
            &[
                (WGSL_SOURCE_TARGET, imported_gpu_canvas_source_container()),
                (
                    WGSL_BINDING_MAP_TARGET,
                    IMPORTED_GPU_CANVAS_BINDING_MAP.to_vec(),
                ),
                (APPLE_METAL_SOURCE_TARGET, source),
                (APPLE_METAL_BINDING_MAP_TARGET, map),
            ],
            &[(SUPPLEMENTAL_REFLECTION_SECTION, reflection)],
        )
    }

    fn provenance(payload: &[u8]) -> GpuCanvasShaderProvenance {
        // SAFETY: tests deliberately establish the same exact-byte trust
        // boundary that production's cryptographic verifier owns.
        unsafe {
            GpuCanvasShaderProvenance::for_verified_artifact_digest_unchecked(
                payload.len() as u64,
                Sha256::digest(payload).into(),
            )
        }
    }

    fn imported_gpu_canvas_webgpu_payload() -> Vec<u8> {
        rstb_payload(&[
            (WGSL_SOURCE_TARGET, imported_gpu_canvas_source_container()),
            (
                WGSL_BINDING_MAP_TARGET,
                IMPORTED_GPU_CANVAS_BINDING_MAP.to_vec(),
            ),
        ])
    }

    #[test]
    fn vulkan_spirv_preserves_exact_target_5_13_asset_and_pair_data() {
        let code = vec![0x03, 0x02, 0x23, 0x07, 0xff, 0x00, 0x80, 0x11];
        let source = byte_source_container(
            &[
                (2, "compute_second", "compute_second"),
                (0, "vertex_first", "vertex_first"),
                (1, "fragment_last", "fragment_last"),
            ],
            &code,
        );
        let mut binding_map = native_binding_map();
        binding_map.extend_from_slice(&[0xa5, 0x5a]);
        let payload = rstb_payload_with_sections(
            &[
                (WGSL_SOURCE_TARGET, imported_gpu_canvas_source_container()),
                (
                    WGSL_BINDING_MAP_TARGET,
                    IMPORTED_GPU_CANVAS_BINDING_MAP.to_vec(),
                ),
                (VULKAN_SPIRV_SOURCE_TARGET, source),
                (VULKAN_SPIRV_BINDING_MAP_TARGET, binding_map.clone()),
            ],
            &[(
                TEXTURE_SAMPLER_PAIR_SECTION,
                vec![2, 1, 4, 1, 7, 2, 3, 5, 6],
            )],
        );
        let asset = ShaderAsset::decode("vulkan", &payload).unwrap();
        asset
            .asset
            .with_downcast_mut::<NativeShaderAsset, _>(|asset| asset.base.set_asset_id(73))
            .expect("native ShaderAsset");
        let artifact = asset
            .decode_for_profile(
                "vulkan",
                GpuCanvasShaderProfile::TrustedVulkanSpirV,
                Some(provenance(&payload)),
            )
            .expect("Vulkan selects the authenticated target-5/13 pair");
        let GpuCanvasShaderArtifact::TrustedVulkanSpirV(shader) = artifact else {
            panic!("Vulkan profile must return SPIR-V");
        };
        assert_eq!(shader.code(), code);
        assert_eq!(
            shader
                .entries()
                .iter()
                .map(|entry| (
                    entry.stage,
                    entry.logical_entry_point.as_str(),
                    entry.physical_entry_point.as_str(),
                ))
                .collect::<Vec<_>>(),
            vec![
                (
                    GpuCanvasShaderStage::Compute,
                    "compute_second",
                    "compute_second",
                ),
                (GpuCanvasShaderStage::Vertex, "vertex_first", "vertex_first",),
                (
                    GpuCanvasShaderStage::Fragment,
                    "fragment_last",
                    "fragment_last",
                ),
            ],
        );
        assert_eq!(shader.bindings().len(), 3);
        assert_eq!(shader.binding_map_bytes(), binding_map);
        assert_eq!(shader.shader_asset_id(), 73);
        assert_eq!(
            shader.texture_sampler_pairs(),
            [
                GpuCanvasShaderTextureSamplerPair {
                    texture_group: 1,
                    texture_binding: 4,
                    sampler_group: 1,
                    sampler_binding: 7,
                },
                GpuCanvasShaderTextureSamplerPair {
                    texture_group: 2,
                    texture_binding: 3,
                    sampler_group: 5,
                    sampler_binding: 6,
                },
            ],
        );
    }

    #[test]
    fn vulkan_spirv_rejects_a_zero_entry_target_5_container() {
        let payload = rstb_payload(&[
            (
                VULKAN_SPIRV_SOURCE_TARGET,
                byte_source_container(&[], &[0x03, 0x02, 0x23, 0x07]),
            ),
            (
                VULKAN_SPIRV_BINDING_MAP_TARGET,
                IMPORTED_GPU_CANVAS_BINDING_MAP.to_vec(),
            ),
        ]);
        let asset = ShaderAsset::decode("zero-entry-vulkan", &payload).unwrap();

        let error = asset
            .decode_for_profile(
                "zero-entry-vulkan",
                GpuCanvasShaderProfile::TrustedVulkanSpirV,
                Some(provenance(&payload)),
            )
            .expect_err("upstream buildShaderEntries fails without an entry record");

        assert!(error.to_string().contains("entry table is empty"));
    }

    #[test]
    fn vulkan_spirv_requires_exact_provenance_and_never_falls_back_to_wgsl() {
        let webgpu_only = imported_gpu_canvas_webgpu_payload();
        let asset = ShaderAsset::decode("webgpu-only", &webgpu_only).unwrap();
        let error = asset
            .decode_for_profile(
                "webgpu-only",
                GpuCanvasShaderProfile::TrustedVulkanSpirV,
                Some(provenance(&webgpu_only)),
            )
            .expect_err("Vulkan never falls back to target 0");
        assert!(error.to_string().contains("target-5"));

        let missing_vulkan_map = rstb_payload(&[
            (
                VULKAN_SPIRV_SOURCE_TARGET,
                byte_source_container(&[(0, "vertex", "vertex")], &[0x03, 0x02, 0x23, 0x07]),
            ),
            (
                WGSL_BINDING_MAP_TARGET,
                IMPORTED_GPU_CANVAS_BINDING_MAP.to_vec(),
            ),
        ]);
        let asset = ShaderAsset::decode("missing-vulkan-map", &missing_vulkan_map).unwrap();
        let error = asset
            .decode_for_profile(
                "missing-vulkan-map",
                GpuCanvasShaderProfile::TrustedVulkanSpirV,
                Some(provenance(&missing_vulkan_map)),
            )
            .expect_err("Vulkan never falls back to the target-16 WGSL map");
        assert!(error.to_string().contains("target-13"));

        let payload = rstb_payload(&[
            (
                VULKAN_SPIRV_SOURCE_TARGET,
                byte_source_container(&[(0, "vertex", "vertex")], &[0x03, 0x02, 0x23, 0x07]),
            ),
            (
                VULKAN_SPIRV_BINDING_MAP_TARGET,
                IMPORTED_GPU_CANVAS_BINDING_MAP.to_vec(),
            ),
        ]);
        let asset = ShaderAsset::decode("vulkan", &payload).unwrap();
        assert!(
            asset
                .decode_for_profile("vulkan", GpuCanvasShaderProfile::TrustedVulkanSpirV, None,)
                .expect_err("native Vulkan bytes require outer provenance")
                .to_string()
                .contains("no verified provenance")
        );
        assert!(
            asset
                .decode_for_profile(
                    "vulkan",
                    GpuCanvasShaderProfile::TrustedVulkanSpirV,
                    Some(provenance(b"different artifact")),
                )
                .expect_err("provenance is bound to the exact artifact")
                .to_string()
                .contains("does not authorize")
        );
    }

    #[cfg(feature = "android-authored-wgsl")]
    #[test]
    fn android_wgsl_requires_exact_unforgeable_provenance() {
        let payload = imported_gpu_canvas_webgpu_payload();
        let asset = ShaderAsset::decode("android", &payload).unwrap();
        assert!(
            asset
                .decode_for_profile("android", GpuCanvasShaderProfile::WebGpu, None)
                .is_err(),
            "ordinary Android imports must not admit exporter-authored WGSL"
        );

        let other = [0u8; 8];
        let error = asset
            .decode_for_profile(
                "android",
                GpuCanvasShaderProfile::WebGpu,
                Some(provenance(&other)),
            )
            .unwrap_err();
        assert!(error.to_string().contains("does not authorize"));

        let artifact = asset
            .decode_for_profile(
                "android",
                GpuCanvasShaderProfile::WebGpu,
                Some(provenance(&payload)),
            )
            .expect("the trusted product proof admits exact WGSL exporter bytes");
        assert!(matches!(artifact, GpuCanvasShaderArtifact::WebGpu(_)));
    }

    #[test]
    fn decodes_pinned_cpp_webgpu_whole_module_and_binding_map() {
        let payload = imported_gpu_canvas_webgpu_payload();
        assert_eq!(
            format!("{:x}", Sha256::digest(&payload[1..])),
            "546517d0dc9fbdaf9585f3daa6e440628e62292d7cb8aa7253fd3019aa35713d",
            "fixture must remain byte-identical to pinned C++ f4bb3025e263",
        );
        let shader = decode_shader_asset("scene", &payload)
            .expect("WebGPU selects target-0 WGSL and mandatory target-16 binding map");
        assert_eq!(shader.source, IMPORTED_GPU_CANVAS_UBO_WGSL);
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
    fn webgpu_preserves_shader_asset_id_and_all_texture_sampler_pairs() {
        let payload = rstb_payload_with_sections(
            &[
                (WGSL_SOURCE_TARGET, imported_gpu_canvas_source_container()),
                (
                    WGSL_BINDING_MAP_TARGET,
                    IMPORTED_GPU_CANVAS_BINDING_MAP.to_vec(),
                ),
            ],
            &[
                (TEXTURE_SAMPLER_PAIR_SECTION, vec![1, 0, 1, 0, 2]),
                (
                    TEXTURE_SAMPLER_PAIR_SECTION,
                    vec![2, 1, 3, 1, 4, 2, 5, 2, 6],
                ),
            ],
        );
        let asset = ShaderAsset::decode("webgpu-metadata", &payload).unwrap();
        asset
            .asset
            .with_downcast_mut::<NativeShaderAsset, _>(|asset| asset.base.set_asset_id(41))
            .expect("native ShaderAsset");

        let shader = asset.decode_webgpu("webgpu-metadata").unwrap();

        assert_eq!(shader.shader_asset_id, 41);
        assert_eq!(
            shader.texture_sampler_pairs,
            [
                GpuCanvasShaderTextureSamplerPair {
                    texture_group: 0,
                    texture_binding: 1,
                    sampler_group: 0,
                    sampler_binding: 2,
                },
                GpuCanvasShaderTextureSamplerPair {
                    texture_group: 1,
                    texture_binding: 3,
                    sampler_group: 1,
                    sampler_binding: 4,
                },
                GpuCanvasShaderTextureSamplerPair {
                    texture_group: 2,
                    texture_binding: 5,
                    sampler_group: 2,
                    sampler_binding: 6,
                },
            ]
        );
    }

    #[test]
    fn selects_webgl2_per_entry_sources_sidecars_and_texture_pairs() {
        let glsl = per_entry_source_container(&[
            (
                0,
                "authored_vertex",
                "main",
                "#version 300 es\nvoid main() { gl_Position = vec4(0.0); }\n",
            ),
            (
                1,
                "authored_fragment",
                "main",
                "#version 300 es\nprecision highp float; out vec4 color; void main() { color = vec4(1.0); }\n",
            ),
        ]);
        let payload = rstb_payload_with_sections(
            &[
                (WEBGL2_SOURCE_TARGET, glsl),
                (
                    WEBGL2_BINDING_MAP_TARGET,
                    IMPORTED_GPU_CANVAS_BINDING_MAP.to_vec(),
                ),
                (WEBGL2_VERTEX_FIXUP_TARGET, empty_gl_fixup()),
                (WEBGL2_FRAGMENT_FIXUP_TARGET, empty_gl_fixup()),
            ],
            &[
                (TEXTURE_SAMPLER_PAIR_SECTION, vec![1, 2, 4, 2, 7]),
                (TEXTURE_SAMPLER_PAIR_SECTION, vec![1, 3, 5, 3, 8]),
            ],
        );
        let asset = ShaderAsset::decode("portable", &payload).unwrap();
        asset
            .asset
            .with_downcast_mut::<NativeShaderAsset, _>(|asset| asset.base.set_asset_id(42))
            .expect("native ShaderAsset");
        let artifact = asset
            .decode_for_profile("portable", GpuCanvasShaderProfile::WebGl2, None)
            .expect("WebGL2 profile selects its exact variants");
        let GpuCanvasShaderArtifact::WebGl2(shader) = artifact else {
            panic!("expected WebGL2 artifact");
        };
        assert_eq!(shader.shader_asset_id, 42);
        assert_eq!(shader.entries.len(), 2);
        assert_eq!(shader.entries[0].logical_entry_point, "authored_vertex");
        assert_eq!(shader.entries[0].physical_entry_point, "main");
        assert!(
            shader
                .source_for_entry(1)
                .expect("fragment source")
                .starts_with("#version 300 es")
        );
        assert_eq!(
            shader.binding_map_bytes.as_ref(),
            IMPORTED_GPU_CANVAS_BINDING_MAP
        );
        assert_eq!(shader.vertex_gl_fixup_bytes.as_ref(), [1, 0, 0]);
        assert_eq!(
            shader.texture_sampler_pairs,
            vec![
                GpuCanvasShaderTextureSamplerPair {
                    texture_group: 2,
                    texture_binding: 4,
                    sampler_group: 2,
                    sampler_binding: 7,
                },
                GpuCanvasShaderTextureSamplerPair {
                    texture_group: 3,
                    texture_binding: 5,
                    sampler_group: 3,
                    sampler_binding: 8,
                },
            ]
        );
    }

    #[test]
    fn webgl2_rejects_non_main_entries_and_missing_sidecars_without_fallback() {
        let bad_source = per_entry_source_container(&[(
            0,
            "vertex",
            "vertex",
            "#version 300 es\nvoid main() {}\n",
        )]);
        let payload = rstb_payload(&[
            (WEBGL2_SOURCE_TARGET, bad_source),
            (
                WEBGL2_BINDING_MAP_TARGET,
                IMPORTED_GPU_CANVAS_BINDING_MAP.to_vec(),
            ),
            (WEBGL2_VERTEX_FIXUP_TARGET, empty_gl_fixup()),
            (WEBGL2_FRAGMENT_FIXUP_TARGET, empty_gl_fixup()),
        ]);
        let asset = ShaderAsset::decode("bad-gl", &payload).expect("outer RSTB decodes");
        assert!(
            asset
                .decode_for_profile("bad-gl", GpuCanvasShaderProfile::WebGl2, None)
                .expect_err("GLSL physical entry must be main")
                .to_string()
                .contains("physical entry 'main'")
        );

        let webgpu_only = rstb_payload(&[
            (WGSL_SOURCE_TARGET, imported_gpu_canvas_source_container()),
            (
                WGSL_BINDING_MAP_TARGET,
                IMPORTED_GPU_CANVAS_BINDING_MAP.to_vec(),
            ),
        ]);
        let asset = ShaderAsset::decode("webgpu-only", &webgpu_only).expect("RSTB decodes");
        assert!(
            asset
                .decode_for_profile("webgpu-only", GpuCanvasShaderProfile::WebGl2, None)
                .expect_err("WebGL2 never falls back to WGSL")
                .to_string()
                .contains("no WebGL2 RSTB target-1")
        );
    }

    #[test]
    fn rejects_out_of_bounds_variants_and_missing_entries() {
        let mut payload = imported_gpu_canvas_webgpu_payload();
        payload[10..14].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(decode_shader_asset("scene", &payload).is_err());

        let mut payload = imported_gpu_canvas_webgpu_payload();
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
                (WGSL_SOURCE_TARGET, imported_gpu_canvas_source_container()),
                (15, b"retired fragment fixup".to_vec()),
                (
                    WGSL_BINDING_MAP_TARGET,
                    IMPORTED_GPU_CANVAS_BINDING_MAP.to_vec(),
                ),
            ]),
        )
        .expect("the final target-0 descriptor wins exactly as in ShaderAsset::decode");
        assert_eq!(shader.source, IMPORTED_GPU_CANVAS_UBO_WGSL);
        assert_eq!(shader.bindings.len(), 1);
    }

    #[test]
    fn validates_only_final_last_wins_descriptors_like_cpp() {
        let source = imported_gpu_canvas_source_container();
        let mut blob_data = source.clone();
        let binding_map_offset = blob_data.len();
        blob_data.extend_from_slice(IMPORTED_GPU_CANVAS_BINDING_MAP);
        let payload = rstb_payload_with_blob(
            &[
                (WGSL_SOURCE_TARGET, u32::MAX as usize, 1),
                (WGSL_BINDING_MAP_TARGET, u32::MAX as usize, 1),
                (WGSL_SOURCE_TARGET, 0, source.len()),
                (
                    WGSL_BINDING_MAP_TARGET,
                    binding_map_offset,
                    IMPORTED_GPU_CANVAS_BINDING_MAP.len(),
                ),
            ],
            &blob_data,
        );

        let shader = decode_shader_asset("scene", &payload)
            .expect("C++ overwrites duplicate targets before validating final ranges");
        assert_eq!(shader.source, IMPORTED_GPU_CANVAS_UBO_WGSL);
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
            IMPORTED_GPU_CANVAS_UBO_WGSL,
        );
        let payload = rstb_payload(&[
            (WGSL_SOURCE_TARGET, source),
            (
                WGSL_BINDING_MAP_TARGET,
                IMPORTED_GPU_CANVAS_BINDING_MAP.to_vec(),
            ),
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
        let mut source = imported_gpu_canvas_source_container();
        source.extend_from_slice(b"source-extension");
        let source_offset = 3;
        let map_offset = source_offset + source.len() + 5;
        let mut blob_data = b"gap".to_vec();
        blob_data.extend_from_slice(&source);
        blob_data.extend_from_slice(b"gap!!");
        blob_data.extend_from_slice(IMPORTED_GPU_CANVAS_BINDING_MAP);
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
                    IMPORTED_GPU_CANVAS_BINDING_MAP.len(),
                ),
            ],
            &blob_data,
        );

        let shader = decode_shader_asset("scene", &payload)
            .expect("pinned C++ accepts aliases, gaps, and unreferenced trailing bytes");
        assert_eq!(shader.source, IMPORTED_GPU_CANVAS_UBO_WGSL);
        assert_eq!(shader.bindings.len(), 1);
    }

    #[test]
    fn requires_wgsl_source_and_binding_map_targets() {
        let source = imported_gpu_canvas_source_container();
        let missing_source = rstb_payload(&[(
            WGSL_BINDING_MAP_TARGET,
            IMPORTED_GPU_CANVAS_BINDING_MAP.to_vec(),
        )]);
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
        let source = imported_gpu_canvas_source_container();
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
        let mut extended_map = IMPORTED_GPU_CANVAS_BINDING_MAP.to_vec();
        extended_map[2..4].copy_from_slice(&15u16.to_le_bytes());
        extended_map.extend_from_slice(&[0xa5, 0x5a]);
        let payload = rstb_payload(&[
            (WGSL_SOURCE_TARGET, imported_gpu_canvas_source_container()),
            (WGSL_BINDING_MAP_TARGET, extended_map),
        ]);

        let shader = decode_shader_asset("scene", &payload)
            .expect("BindingMap v2 is append-only like the pinned C++ decoder");
        assert_eq!(shader.bindings.len(), 1);
        assert_eq!(shader.bindings[0].backend_slots, [None, Some(0), None]);
    }

    #[test]
    fn trusted_apple_metal_decodes_target_2_10_and_digest_bound_reflection() {
        let payload = native_payload();
        let asset = ShaderAsset::decode("native", &payload).unwrap();
        let artifact = asset
            .decode_for_profile(
                "native",
                GpuCanvasShaderProfile::TrustedAppleMetal,
                Some(provenance(&payload)),
            )
            .unwrap();
        let GpuCanvasShaderArtifact::TrustedAppleMetal(shader) = artifact else {
            panic!("Metal profile must not return WGSL");
        };
        assert!(shader.source().contains("metal_stdlib"));
        assert_eq!(shader.entries().len(), 3);
        assert_eq!(shader.bindings().len(), 3);
        assert_eq!(shader.binding_map_bytes(), native_binding_map());
        assert_eq!(shader.bindings()[0].backend_slots, [Some(0), Some(0), None]);
        assert_eq!(shader.bindings()[1].backend_space, 1);
        assert_eq!(shader.binding_reflection()[0].min_buffer_size, 64);
        assert_eq!(shader.binding_reflection()[1].array_count, 4);
        assert_eq!(shader.entry_reflection()[2].workgroup_size, [8, 4, 2]);
        assert_eq!(
            shader.entry_reflection()[0].inputs[0].binding,
            GpuCanvasShaderInterfaceBinding::Builtin(GpuCanvasShaderBuiltin::VertexIndex),
        );
        assert_eq!(
            shader.entry_reflection()[1].inputs[0].binding,
            GpuCanvasShaderInterfaceBinding::Location {
                location: 0,
                interpolation: Some(GpuCanvasShaderInterpolation::Perspective),
                sampling: Some(GpuCanvasShaderSampling::Center),
            },
        );
    }

    #[test]
    fn trusted_apple_metal_retains_exact_append_only_target_10_bytes() {
        let source = source_container(
            &[
                (0, "vertex", "vs_native"),
                (1, "fragment", "fs_native"),
                (2, "compute", "cs_native"),
            ],
            "#include <metal_stdlib>\nusing namespace metal;",
        );
        let original = native_binding_map();
        let mut extended = original[..8].to_vec();
        extended[2..4].copy_from_slice(&15u16.to_le_bytes());
        for (index, row) in original[8..].chunks_exact(14).enumerate() {
            extended.extend_from_slice(row);
            extended.push(0xa0 + index as u8);
        }
        extended.extend_from_slice(&[0xa5, 0x5a]);
        let reflection = native_reflection(&source, &extended);
        let payload = rstb_payload_with_sections(
            &[
                (APPLE_METAL_SOURCE_TARGET, source),
                (APPLE_METAL_BINDING_MAP_TARGET, extended.clone()),
            ],
            &[(SUPPLEMENTAL_REFLECTION_SECTION, reflection)],
        );
        let asset = ShaderAsset::decode("extended-native", &payload).unwrap();
        let artifact = asset
            .decode_for_profile(
                "extended-native",
                GpuCanvasShaderProfile::TrustedAppleMetal,
                Some(provenance(&payload)),
            )
            .unwrap();
        let GpuCanvasShaderArtifact::TrustedAppleMetal(shader) = artifact else {
            panic!("Metal profile must return the native artifact");
        };
        assert_eq!(shader.binding_map_bytes(), extended);
        assert_eq!(shader.bindings().len(), 3);
    }

    #[test]
    fn apple_metal_requires_exact_unforgeable_provenance() {
        let payload = native_payload();
        let asset = ShaderAsset::decode("native", &payload).unwrap();
        assert!(
            asset
                .decode_for_profile("native", GpuCanvasShaderProfile::TrustedAppleMetal, None)
                .is_err()
        );

        let other = [0u8; 8];
        let wrong = provenance(&other);
        let error = asset
            .decode_for_profile(
                "native",
                GpuCanvasShaderProfile::TrustedAppleMetal,
                Some(wrong),
            )
            .unwrap_err();
        assert!(error.to_string().contains("does not authorize"));

        // A syntactically signed envelope remains untrusted without a proof
        // minted by the external verifier.
        let mut signed = vec![0x80];
        signed.extend_from_slice(&[0xa5; 64]);
        signed.extend_from_slice(&payload[1..]);
        let signed_asset = ShaderAsset::decode("signed", &signed).unwrap();
        assert!(
            signed_asset
                .decode_for_profile("signed", GpuCanvasShaderProfile::TrustedAppleMetal, None)
                .is_err()
        );
    }

    #[test]
    fn apple_metal_never_falls_back_and_webgpu_uses_its_profile_policy() {
        let web = imported_gpu_canvas_webgpu_payload();
        let asset = ShaderAsset::decode("web", &web).unwrap();
        #[cfg(feature = "android-authored-wgsl")]
        let web_provenance = Some(provenance(&web));
        #[cfg(not(feature = "android-authored-wgsl"))]
        let web_provenance = None;
        assert!(matches!(
            asset
                .decode_for_profile("web", GpuCanvasShaderProfile::WebGpu, web_provenance)
                .unwrap(),
            GpuCanvasShaderArtifact::WebGpu(_)
        ));
        let error = asset
            .decode_for_profile(
                "web",
                GpuCanvasShaderProfile::TrustedAppleMetal,
                Some(provenance(&web)),
            )
            .unwrap_err();
        assert!(error.to_string().contains("target-2"));

        let dual_profile = native_payload();
        let asset = ShaderAsset::decode("dual-profile", &dual_profile).unwrap();
        #[cfg(feature = "android-authored-wgsl")]
        let web_provenance = Some(provenance(&dual_profile));
        #[cfg(not(feature = "android-authored-wgsl"))]
        let web_provenance = None;
        assert!(matches!(
            asset
                .decode_for_profile(
                    "dual-profile",
                    GpuCanvasShaderProfile::WebGpu,
                    web_provenance,
                )
                .unwrap(),
            GpuCanvasShaderArtifact::WebGpu(_)
        ));
    }

    #[test]
    fn apple_metal_fails_closed_at_every_reflection_boundary() {
        let valid = native_payload();
        let decoded = ShaderAsset::decode("native", &valid).unwrap();
        let reflection = decoded.supplemental_reflection.clone().unwrap();
        let source = decoded.variant(APPLE_METAL_SOURCE_TARGET).unwrap().to_vec();
        let map = decoded
            .variant(APPLE_METAL_BINDING_MAP_TARGET)
            .unwrap()
            .to_vec();

        let cases = [
            ("missing reflection", Vec::new()),
            ("bad version", {
                let mut value = reflection.clone();
                value[0] = 9;
                value
            }),
            ("wrong source digest", {
                let mut value = reflection.clone();
                value[1] ^= 1;
                value
            }),
            ("wrong map digest", {
                let mut value = reflection.clone();
                value[33] ^= 1;
                value
            }),
            ("wrong entry", {
                let mut value = reflection.clone();
                let at = value
                    .windows(b"vertex".len())
                    .position(|window| window == b"vertex")
                    .expect("logical entry marker");
                value[at..at + b"vertex".len()].copy_from_slice(b"badbad");
                value
            }),
            ("entry count mismatch", {
                let mut value = reflection.clone();
                value[65] = 2;
                value
            }),
            ("wrong entry stage", {
                let mut value = reflection.clone();
                value[66] = 1;
                value
            }),
            ("zero graphics workgroup", {
                let mut value = reflection.clone();
                value[86..90].fill(0);
                value
            }),
            ("invalid interface type", {
                let mut value = reflection.clone();
                value[103] = 0xfe;
                value
            }),
            ("duplicate interface binding", {
                let mut value = reflection.clone();
                value[112] = 1;
                value[113..115].copy_from_slice(&2u16.to_le_bytes());
                value[116..118].fill(u8::MAX);
                value
            }),
            ("truncated", reflection[..reflection.len() - 1].to_vec()),
            ("trailing", {
                let mut value = reflection.clone();
                value.push(0);
                value
            }),
        ];
        for (label, bad_reflection) in cases {
            let sections = if bad_reflection.is_empty() {
                vec![]
            } else {
                vec![(SUPPLEMENTAL_REFLECTION_SECTION, bad_reflection)]
            };
            let payload = rstb_payload_with_sections(
                &[
                    (WGSL_SOURCE_TARGET, imported_gpu_canvas_source_container()),
                    (
                        WGSL_BINDING_MAP_TARGET,
                        IMPORTED_GPU_CANVAS_BINDING_MAP.to_vec(),
                    ),
                    (APPLE_METAL_SOURCE_TARGET, source.clone()),
                    (APPLE_METAL_BINDING_MAP_TARGET, map.clone()),
                ],
                &sections,
            );
            let asset = ShaderAsset::decode(label, &payload).unwrap();
            assert!(
                asset
                    .decode_for_profile(
                        label,
                        GpuCanvasShaderProfile::TrustedAppleMetal,
                        Some(provenance(&payload))
                    )
                    .is_err(),
                "{label} must fail closed",
            );
        }
    }

    #[test]
    fn supplemental_reflection_rejects_zero_arrays_and_invalid_buffer_sizes() {
        let valid = native_payload();
        let decoded = ShaderAsset::decode("native", &valid).unwrap();
        let source = decoded.variant(APPLE_METAL_SOURCE_TARGET).unwrap().to_vec();
        let map = decoded
            .variant(APPLE_METAL_BINDING_MAP_TARGET)
            .unwrap()
            .to_vec();
        let reflection = decoded.supplemental_reflection.clone().unwrap();
        // Locate the binding table structurally rather than pinning an offset.
        let marker = [3, 0, 0, 0, 1, 0, 64, 0, 0, 0, 0, 0, 0, 0];
        let at = reflection
            .windows(marker.len())
            .position(|window| window == marker)
            .unwrap();
        for (label, mutate) in [
            ("zero-array", (at + 4, 0u8)),
            ("zero-buffer-size", (at + 6, 0u8)),
        ] {
            let mut invalid = reflection.clone();
            invalid[mutate.0] = mutate.1;
            if label == "zero-array" {
                invalid[at + 5] = 0;
            } else {
                invalid[at + 6..at + 14].fill(0);
            }
            let payload = rstb_payload_with_sections(
                &[
                    (WGSL_SOURCE_TARGET, imported_gpu_canvas_source_container()),
                    (
                        WGSL_BINDING_MAP_TARGET,
                        IMPORTED_GPU_CANVAS_BINDING_MAP.to_vec(),
                    ),
                    (APPLE_METAL_SOURCE_TARGET, source.clone()),
                    (APPLE_METAL_BINDING_MAP_TARGET, map.clone()),
                ],
                &[(SUPPLEMENTAL_REFLECTION_SECTION, invalid)],
            );
            let asset = ShaderAsset::decode(label, &payload).unwrap();
            assert!(
                asset
                    .decode_for_profile(
                        label,
                        GpuCanvasShaderProfile::TrustedAppleMetal,
                        Some(provenance(&payload))
                    )
                    .is_err(),
                "{label} must reject native data even though valid target 0/16 exists"
            );
        }
    }

    #[test]
    fn supplemental_reflection_rejects_wrong_binding_keys_and_nonbuffer_sizes() {
        let valid = native_payload();
        let decoded = ShaderAsset::decode("native", &valid).unwrap();
        let source = decoded.variant(APPLE_METAL_SOURCE_TARGET).unwrap().to_vec();
        let map = decoded
            .variant(APPLE_METAL_BINDING_MAP_TARGET)
            .unwrap()
            .to_vec();
        let reflection = decoded.supplemental_reflection.clone().unwrap();
        let marker = [3, 0, 0, 0, 1, 0, 64, 0, 0, 0, 0, 0, 0, 0];
        let at = reflection
            .windows(marker.len())
            .position(|window| window == marker)
            .unwrap();

        for (label, invalid) in [
            ("wrong-binding-key", {
                let mut value = reflection.clone();
                value[at + 3] = 9;
                value
            }),
            ("texture-buffer-size", {
                let mut value = reflection.clone();
                value[at + 18] = 1;
                value
            }),
        ] {
            let payload = rstb_payload_with_sections(
                &[
                    (WGSL_SOURCE_TARGET, imported_gpu_canvas_source_container()),
                    (
                        WGSL_BINDING_MAP_TARGET,
                        IMPORTED_GPU_CANVAS_BINDING_MAP.to_vec(),
                    ),
                    (APPLE_METAL_SOURCE_TARGET, source.clone()),
                    (APPLE_METAL_BINDING_MAP_TARGET, map.clone()),
                ],
                &[(SUPPLEMENTAL_REFLECTION_SECTION, invalid)],
            );
            let asset = ShaderAsset::decode(label, &payload).unwrap();
            assert!(
                asset
                    .decode_for_profile(
                        label,
                        GpuCanvasShaderProfile::TrustedAppleMetal,
                        Some(provenance(&payload)),
                    )
                    .is_err(),
                "{label} must not fall back to valid target 0/16"
            );
        }
    }

    #[test]
    fn duplicate_supplemental_reflection_sections_are_malformed() {
        let valid = native_payload();
        let decoded = ShaderAsset::decode("native", &valid).unwrap();
        let reflection = decoded.supplemental_reflection.clone().unwrap();
        let source = decoded.variant(APPLE_METAL_SOURCE_TARGET).unwrap().to_vec();
        let map = decoded
            .variant(APPLE_METAL_BINDING_MAP_TARGET)
            .unwrap()
            .to_vec();
        let payload = rstb_payload_with_sections(
            &[
                (APPLE_METAL_SOURCE_TARGET, source),
                (APPLE_METAL_BINDING_MAP_TARGET, map),
            ],
            &[
                (SUPPLEMENTAL_REFLECTION_SECTION, reflection.clone()),
                (SUPPLEMENTAL_REFLECTION_SECTION, reflection),
            ],
        );
        assert!(ShaderAsset::decode("duplicate-reflection", &payload).is_err());
    }

    fn make_upstream_rstb(
        version: u16,
        variants: &[(u8, Vec<u8>)],
        sections: &[(u8, Vec<u8>)],
        magic: u32,
    ) -> Vec<u8> {
        let mut out = Vec::new();
        put_u32(&mut out, magic);
        put_u16(&mut out, version);
        out.push(variants.len() as u8);
        out.push(sections.len() as u8);

        let mut blob_section = Vec::new();
        for (target, blob) in variants {
            out.push(*target);
            put_u32(&mut out, blob_section.len() as u32);
            put_u32(&mut out, blob.len() as u32);
            blob_section.extend_from_slice(blob);
        }
        for (tag, data) in sections {
            out.push(*tag);
            put_u16(&mut out, data.len() as u16);
            out.extend_from_slice(data);
        }
        out.extend_from_slice(&blob_section);
        out
    }

    fn upstream_envelope(rstb: &[u8]) -> Vec<u8> {
        let mut out = vec![0x00];
        out.extend_from_slice(rstb);
        out
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct UpstreamTextureSamplerPair {
        tex_group: u8,
        tex_binding: u8,
        samp_group: u8,
        samp_binding: u8,
    }

    fn upstream_texture_sampler_pairs(asset: &ShaderAsset) -> Vec<UpstreamTextureSamplerPair> {
        asset
            .asset
            .with_downcast::<NativeShaderAsset, _>(|asset| asset.texture_sampler_pairs().to_vec())
            .expect("decoded native ShaderAsset")
            .into_iter()
            .map(|pair| UpstreamTextureSamplerPair {
                tex_group: pair.tex_group,
                tex_binding: pair.tex_binding,
                samp_group: pair.samp_group,
                samp_binding: pair.samp_binding,
            })
            .collect()
    }

    fn make_upstream_tex_sampler_pairs_tag(pairs: &[[u8; 4]]) -> Vec<u8> {
        let mut out = vec![pairs.len() as u8];
        for pair in pairs {
            out.extend_from_slice(pair);
        }
        out
    }

    #[test]
    fn upstream_shader_asset_decode_valid() {
        let blob = vec![0xde, 0xad, 0xbe, 0xef, 0x42];
        let data = upstream_envelope(&make_upstream_rstb(4, &[(2, blob)], &[], 0x5253_5442));
        let asset = ShaderAsset::decode("asset", &data).unwrap();

        let result = asset.variant(2).unwrap();
        assert_eq!(result.len(), 5);
        assert_eq!(result[0], 0xde);
        assert_eq!(result[1], 0xad);
        assert_eq!(result[2], 0xbe);
        assert_eq!(result[3], 0xef);
        assert_eq!(result[4], 0x42);
    }

    #[test]
    fn upstream_shader_asset_decode_bad_magic() {
        let data = upstream_envelope(&make_upstream_rstb(4, &[], &[], 0xbadb_ad00));
        assert_eq!(ShaderAsset::decode("asset", &data).is_ok(), false);
    }

    #[test]
    fn upstream_shader_asset_decode_bad_version() {
        let data = upstream_envelope(&make_upstream_rstb(3, &[], &[], 0x5253_5442));
        assert_eq!(ShaderAsset::decode("asset", &data).is_ok(), false);
    }

    #[test]
    fn upstream_shader_asset_decode_truncated() {
        let data = [0x00, 0x52, 0x53, 0x54, 0x42, 0x01, 0x00];
        assert_eq!(ShaderAsset::decode("asset", &data).is_ok(), false);
    }

    #[test]
    fn upstream_shader_asset_find_shader_miss() {
        let data = upstream_envelope(&make_upstream_rstb(4, &[(2, vec![0xaa])], &[], 0x5253_5442));
        let asset = ShaderAsset::decode("asset", &data).unwrap();

        assert!(asset.variant(0).is_none());
        assert!(asset.variant(99).is_none());
        assert_eq!(asset.variant(2).unwrap().len(), 1);
    }

    #[test]
    fn upstream_shader_asset_multiple_targets() {
        let blob0 = vec![0x11, 0x22];
        let blob1 = vec![0x33, 0x44, 0x55];
        let blob2 = vec![0x66];
        let blob3 = vec![0x77, 0x88, 0x99, 0xaa];
        let data = upstream_envelope(&make_upstream_rstb(
            4,
            &[(0, blob0), (1, blob1), (2, blob2), (3, blob3)],
            &[],
            0x5253_5442,
        ));
        let asset = ShaderAsset::decode("asset", &data).unwrap();

        let r0 = asset.variant(0).unwrap();
        assert_eq!(r0.len(), 2);
        assert_eq!(r0[0], 0x11);
        let r1 = asset.variant(1).unwrap();
        assert_eq!(r1.len(), 3);
        assert_eq!(r1[0], 0x33);
        let r2 = asset.variant(2).unwrap();
        assert_eq!(r2.len(), 1);
        assert_eq!(r2[0], 0x66);
        let r3 = asset.variant(3).unwrap();
        assert_eq!(r3.len(), 4);
        assert_eq!(r3[0], 0x77);
    }

    #[test]
    fn upstream_shader_asset_decode_texture_sampler_pairs() {
        let pairs =
            make_upstream_tex_sampler_pairs_tag(&[[0, 1, 0, 2], [1, 3, 1, 4], [2, 5, 2, 6]]);
        let data = upstream_envelope(&make_upstream_rstb(
            4,
            &[(2, vec![0xaa])],
            &[(1, pairs)],
            0x5253_5442,
        ));
        let asset = ShaderAsset::decode("asset", &data).unwrap();

        let out = upstream_texture_sampler_pairs(&asset);
        assert_eq!(out.len(), 3);
        assert_eq!(out[0].tex_group, 0);
        assert_eq!(out[0].tex_binding, 1);
        assert_eq!(out[0].samp_group, 0);
        assert_eq!(out[0].samp_binding, 2);
        assert_eq!(out[1].tex_binding, 3);
        assert_eq!(out[2].samp_binding, 6);

        let blob = asset.variant(2).unwrap();
        assert_eq!(blob.len(), 1);
        assert_eq!(blob[0], 0xaa);
    }

    #[test]
    fn upstream_shader_asset_decode_texture_sampler_pairs_ignores_trailing_bytes() {
        let mut pairs = make_upstream_tex_sampler_pairs_tag(&[[0, 1, 0, 2]]);
        pairs.extend_from_slice(&[0xfe, 0xff]);
        let data = upstream_envelope(&make_upstream_rstb(
            4,
            &[(2, vec![0xaa])],
            &[(1, pairs)],
            0x5253_5442,
        ));
        let asset = ShaderAsset::decode("asset", &data).unwrap();

        assert_eq!(
            upstream_texture_sampler_pairs(&asset),
            vec![UpstreamTextureSamplerPair {
                tex_group: 0,
                tex_binding: 1,
                samp_group: 0,
                samp_binding: 2,
            }]
        );
        assert_eq!(asset.variant(2).unwrap(), [0xaa]);
    }

    #[test]
    fn upstream_shader_asset_decode_appends_texture_sampler_pair_sections() {
        let first = make_upstream_tex_sampler_pairs_tag(&[[0, 1, 0, 2]]);
        let second = make_upstream_tex_sampler_pairs_tag(&[[1, 3, 1, 4]]);
        let data = upstream_envelope(&make_upstream_rstb(
            4,
            &[(2, vec![0xaa])],
            &[(1, first), (1, second)],
            0x5253_5442,
        ));
        let asset = ShaderAsset::decode("asset", &data).unwrap();

        assert_eq!(
            upstream_texture_sampler_pairs(&asset),
            vec![
                UpstreamTextureSamplerPair {
                    tex_group: 0,
                    tex_binding: 1,
                    samp_group: 0,
                    samp_binding: 2,
                },
                UpstreamTextureSamplerPair {
                    tex_group: 1,
                    tex_binding: 3,
                    samp_group: 1,
                    samp_binding: 4,
                },
            ]
        );
        assert_eq!(asset.variant(2).unwrap(), [0xaa]);
    }

    #[test]
    fn upstream_shader_asset_decode_no_sections_empty_pairs() {
        let data = upstream_envelope(&make_upstream_rstb(4, &[(2, vec![0x42])], &[], 0x5253_5442));
        let asset = ShaderAsset::decode("asset", &data).unwrap();
        assert_eq!(upstream_texture_sampler_pairs(&asset).len(), 0);
    }

    #[test]
    fn upstream_shader_asset_decode_unknown_tag_skipped() {
        let bogus_section = vec![0xde, 0xad, 0xbe, 0xef];
        let data = upstream_envelope(&make_upstream_rstb(
            4,
            &[(2, vec![0x42])],
            &[(99, bogus_section)],
            0x5253_5442,
        ));
        let asset = ShaderAsset::decode("asset", &data).unwrap();
        assert_eq!(upstream_texture_sampler_pairs(&asset).len(), 0);
        let blob = asset.variant(2).unwrap();
        assert_eq!(blob.len(), 1);
        assert_eq!(blob[0], 0x42);
    }

    #[test]
    fn upstream_shader_asset_decode_truncated_section_header() {
        let mut rstb = make_upstream_rstb(4, &[(2, vec![0x42])], &[(1, vec![0])], 0x5253_5442);
        rstb.resize(17 + 1, 0);
        let data = upstream_envelope(&rstb);
        assert_eq!(ShaderAsset::decode("asset", &data).is_ok(), false);
    }

    #[test]
    fn upstream_shader_asset_decode_truncated_section_data() {
        let mut rstb = make_upstream_rstb(4, &[(2, vec![0x42])], &[], 0x5253_5442);
        rstb[7] = 1;
        rstb.push(1);
        rstb.push(10);
        rstb.push(0);
        rstb.push(0xaa);
        rstb.push(0xbb);
        let data = upstream_envelope(&rstb);
        assert_eq!(ShaderAsset::decode("asset", &data).is_ok(), false);
    }

    #[test]
    fn upstream_shader_asset_decode_rejects_out_of_range_variant() {
        let mut rstb = make_upstream_rstb(4, &[(2, vec![0xaa])], &[], 0x5253_5442);
        rstb[13] = 0xff;
        rstb[14] = 0xff;
        rstb[15] = 0xff;
        rstb[16] = 0x7f;
        let data = upstream_envelope(&rstb);
        let result = ShaderAsset::decode("asset", &data);
        assert_eq!(result.is_ok(), false);
        assert_eq!(
            result
                .as_ref()
                .ok()
                .and_then(|asset| asset.variant(2))
                .is_some(),
            false
        );
    }

    #[test]
    fn upstream_shader_asset_decode_rejects_overflowing_variant() {
        let mut rstb = make_upstream_rstb(4, &[(2, vec![0xaa])], &[], 0x5253_5442);
        rstb[9] = 0xf0;
        rstb[10] = 0xff;
        rstb[11] = 0xff;
        rstb[12] = 0xff;
        rstb[13] = 0x20;
        rstb[14] = 0x00;
        rstb[15] = 0x00;
        rstb[16] = 0x00;
        let data = upstream_envelope(&rstb);
        assert_eq!(ShaderAsset::decode("asset", &data).is_ok(), false);
    }

    #[test]
    fn upstream_shader_asset_decode_pair_count_mismatch_ignored() {
        let bad_pairs = vec![5, 0, 1, 0, 2, 1, 3, 1, 4];
        let data = upstream_envelope(&make_upstream_rstb(
            4,
            &[(2, vec![0xaa])],
            &[(1, bad_pairs)],
            0x5253_5442,
        ));
        let asset = ShaderAsset::decode("asset", &data).unwrap();
        assert_eq!(upstream_texture_sampler_pairs(&asset).len(), 0);
        let blob = asset.variant(2).unwrap();
        assert_eq!(blob.len(), 1);
        assert_eq!(blob[0], 0xaa);
    }
}
