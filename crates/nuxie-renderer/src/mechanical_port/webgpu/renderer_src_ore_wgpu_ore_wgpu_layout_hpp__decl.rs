//! Complete mechanical declaration/inline implementation translation of
//! `renderer/src/ore/wgpu/ore_wgpu_layout.hpp`.

#![allow(non_snake_case)]

use super::webgpu_cpp_decl::{
    BindGroupLayout as WagyuBindGroupLayout, BindGroupLayoutEntry as WGPUBindGroupLayoutEntry,
    BufferBindingType, Device as WagyuDevice, SamplerBindingType, ShaderStage as WGPUShaderStage,
    StorageTextureAccess, TextureFormat as WGPUTextureFormat,
    TextureSampleType as WGPUTextureSampleType,
    TextureViewDimension as WGPUTextureViewDimension,
};
use super::webgpu_decl::{
    WGPUBindGroupLayoutDescriptor, WGPUStringView, WGPUTextureFormat_RGBA8Unorm, WGPU_FALSE,
    WGPU_STRLEN, WGPU_TRUE,
};
use nuxie_ore_metal::binding_map::{
    ResourceKind, TextureSampleType as OreTextureSampleType, TextureViewDim,
};
use nuxie_ore_metal::types::{
    BindGroupLayoutDesc, BindGroupLayoutEntry as OreBindGroupLayoutEntry, BindingKind, SampleType,
    StageVisibility, TextureViewDimension as OreTextureViewDimension, kMaxBindGroups,
};

pub(crate) const PINNED_SOURCE: &str =
    include_str!("source/renderer_src_ore_wgpu_ore_wgpu_layout.hpp");

/// WebGPU-side alias of ORE's single authoritative group-count constant.
pub(crate) const kWGPUMaxGroups: u32 = kMaxBindGroups;
pub(crate) const kMaxEntriesPerGroup: usize = 16;

pub(crate) fn toWGPUViewDim(d: TextureViewDim) -> WGPUTextureViewDimension {
    match d.0 {
        0 => WGPUTextureViewDimension::e2D,
        1 => WGPUTextureViewDimension::e1D,
        2 => WGPUTextureViewDimension::e2D,
        3 => WGPUTextureViewDimension::e2DArray,
        4 => WGPUTextureViewDimension::Cube,
        5 => WGPUTextureViewDimension::CubeArray,
        6 => WGPUTextureViewDimension::e3D,
        // Preserve the source's post-switch fallback for unknown values.
        _ => WGPUTextureViewDimension::e2D,
    }
}

pub(crate) fn toWGPUSampleType(s: OreTextureSampleType) -> WGPUTextureSampleType {
    match s.0 {
        0 | 1 => WGPUTextureSampleType::Float,
        2 => WGPUTextureSampleType::UnfilterableFloat,
        3 => WGPUTextureSampleType::Depth,
        4 => WGPUTextureSampleType::Sint,
        5 => WGPUTextureSampleType::Uint,
        // Preserve the source's post-switch fallback for unknown values.
        _ => WGPUTextureSampleType::Float,
    }
}

fn wgpuBool(value: bool) -> u32 {
    if value { WGPU_TRUE } else { WGPU_FALSE }
}

/// Mechanical form of the binding-map path. The enclosing C descriptor starts
/// fully zero-initialized; only the layout selected by `kind` is populated.
pub(crate) fn makeWGPUBGLEntry(
    binding: u32,
    kind: ResourceKind,
    hasDynamicOffset: bool,
    visibility: WGPUShaderStage,
    textureViewDim: TextureViewDim,
    textureSampleType: OreTextureSampleType,
    textureMultisampled: bool,
) -> WGPUBindGroupLayoutEntry {
    let mut e = WGPUBindGroupLayoutEntry::default();
    e.binding = binding;
    e.visibility = visibility.into();

    match kind.0 {
        0 => {
            e.buffer.r#type = BufferBindingType::Uniform.into();
            e.buffer.hasDynamicOffset = wgpuBool(hasDynamicOffset);
        }
        1 => e.buffer.r#type = BufferBindingType::ReadOnlyStorage.into(),
        2 => e.buffer.r#type = BufferBindingType::Storage.into(),
        3 => {
            e.texture.sampleType = toWGPUSampleType(textureSampleType).into();
            e.texture.viewDimension = toWGPUViewDim(textureViewDim).into();
            e.texture.multisampled = wgpuBool(textureMultisampled);
        }
        4 => {
            e.storageTexture.access = StorageTextureAccess::WriteOnly.into();
            e.storageTexture.format = WGPUTextureFormat_RGBA8Unorm;
            e.storageTexture.viewDimension = toWGPUViewDim(textureViewDim).into();
        }
        5 => e.sampler.r#type = SamplerBindingType::Filtering.into(),
        6 => e.sampler.r#type = SamplerBindingType::Comparison.into(),
        // C++ has no default arm: an unknown value leaves the zero-init entry.
        _ => {}
    }
    e
}

pub(crate) fn makeWGPUBGLEntryWithSourceDefaults(
    binding: u32,
    kind: ResourceKind,
    hasDynamicOffset: bool,
    visibility: WGPUShaderStage,
) -> WGPUBindGroupLayoutEntry {
    makeWGPUBGLEntry(
        binding,
        kind,
        hasDynamicOffset,
        visibility,
        TextureViewDim::Undefined,
        OreTextureSampleType::Undefined,
        false,
    )
}

fn toWGPUDescViewDim(d: OreTextureViewDimension) -> WGPUTextureViewDimension {
    match d {
        OreTextureViewDimension::texture2D => WGPUTextureViewDimension::e2D,
        OreTextureViewDimension::cube => WGPUTextureViewDimension::Cube,
        OreTextureViewDimension::texture3D => WGPUTextureViewDimension::e3D,
        OreTextureViewDimension::array2D => WGPUTextureViewDimension::e2DArray,
        OreTextureViewDimension::cubeArray => WGPUTextureViewDimension::CubeArray,
    }
}

fn toWGPUDescSampleType(s: SampleType) -> WGPUTextureSampleType {
    match s {
        SampleType::floatFilterable => WGPUTextureSampleType::Float,
        SampleType::floatUnfilterable => WGPUTextureSampleType::UnfilterableFloat,
        SampleType::depth => WGPUTextureSampleType::Depth,
        SampleType::sint => WGPUTextureSampleType::Sint,
        SampleType::uint => WGPUTextureSampleType::Uint,
    }
}

pub(crate) fn makeWGPUBGLEntryFromDesc(
    src: &OreBindGroupLayoutEntry,
) -> WGPUBindGroupLayoutEntry {
    let mut e = WGPUBindGroupLayoutEntry::default();
    e.binding = src.binding;

    let mut vis = WGPUShaderStage::None;
    if src.visibility.mask & StageVisibility::kVertex != 0 {
        vis |= WGPUShaderStage::Vertex;
    }
    if src.visibility.mask & StageVisibility::kFragment != 0 {
        vis |= WGPUShaderStage::Fragment;
    }
    if src.visibility.mask & StageVisibility::kCompute != 0 {
        vis |= WGPUShaderStage::Compute;
    }
    e.visibility = vis.into();

    match src.kind {
        BindingKind::uniformBuffer => {
            e.buffer.r#type = BufferBindingType::Uniform.into();
            e.buffer.hasDynamicOffset = wgpuBool(src.hasDynamicOffset);
            e.buffer.minBindingSize = u64::from(src.minBindingSize);
        }
        BindingKind::storageBufferRO => {
            e.buffer.r#type = BufferBindingType::ReadOnlyStorage.into();
            e.buffer.minBindingSize = u64::from(src.minBindingSize);
        }
        BindingKind::storageBufferRW => {
            e.buffer.r#type = BufferBindingType::Storage.into();
            e.buffer.minBindingSize = u64::from(src.minBindingSize);
        }
        BindingKind::sampledTexture => {
            e.texture.sampleType = toWGPUDescSampleType(src.textureSampleType).into();
            e.texture.viewDimension = toWGPUDescViewDim(src.textureViewDim).into();
            e.texture.multisampled = wgpuBool(src.textureMultisampled);
        }
        BindingKind::storageTexture => {
            e.storageTexture.access = StorageTextureAccess::WriteOnly.into();
            e.storageTexture.format = WGPUTextureFormat_RGBA8Unorm;
            e.storageTexture.viewDimension = toWGPUDescViewDim(src.textureViewDim).into();
        }
        BindingKind::sampler => e.sampler.r#type = SamplerBindingType::Filtering.into(),
        BindingKind::comparisonSampler => {
            e.sampler.r#type = SamplerBindingType::Comparison.into();
        }
    }
    e
}

fn labelStringView(label: Option<&str>) -> WGPUStringView {
    static EMPTY_LABEL: &[u8] = b"\0";
    match label {
        // The C++ ternary supplies `""`, whose StringView constructor uses
        // WGPU_STRLEN. Keep that exact sentinel and a non-null empty C string.
        None => WGPUStringView {
            data: EMPTY_LABEL.as_ptr().cast(),
            length: WGPU_STRLEN,
        },
        // Rust strings need not be NUL terminated, so carry their exact length.
        Some(label) => WGPUStringView {
            data: label.as_ptr().cast(),
            length: label.len(),
        },
    }
}

fn prepareWGPUBindGroupLayoutFromDesc(
    desc: &BindGroupLayoutDesc<'_>,
) -> ([WGPUBindGroupLayoutEntry; kMaxEntriesPerGroup], usize) {
    let mut entries = std::array::from_fn(|_| WGPUBindGroupLayoutEntry::default());
    let authoredCount = (desc.entryCount as usize).min(desc.entries.len());
    let n = authoredCount.min(kMaxEntriesPerGroup);
    for (dst, src) in entries.iter_mut().zip(desc.entries.iter()).take(n) {
        *dst = makeWGPUBGLEntryFromDesc(src);
    }
    (entries, n)
}

pub(crate) fn buildWGPUBindGroupLayoutFromDesc(
    device: &WagyuDevice,
    desc: &BindGroupLayoutDesc<'_>,
) -> WagyuBindGroupLayout {
    let (entries, n) = prepareWGPUBindGroupLayoutFromDesc(desc);
    let mut bglDesc = WGPUBindGroupLayoutDescriptor::default();
    bglDesc.label = labelStringView(desc.label);
    bglDesc.entryCount = n;
    bglDesc.entries = if n > 0 {
        entries.as_ptr()
    } else {
        std::ptr::null()
    };
    unsafe { device.CreateBindGroupLayout(&bglDesc) }
}

pub(crate) const SOURCE_INLINE_FUNCTION_COUNT: usize = 5;
pub(crate) const SOURCE_SWITCH_COUNT: usize = 6;
pub(crate) const SOURCE_SWITCH_CASE_COUNT: usize = 38;
pub(crate) const SOURCE_RESOURCE_KIND_CASE_COUNT: usize = 7;
pub(crate) const SOURCE_BINDING_KIND_CASE_COUNT: usize = 7;
pub(crate) const SOURCE_DEFAULT_ARGUMENT_COUNT: usize = 3;
pub(crate) const SOURCE_CREATE_BIND_GROUP_LAYOUT_CALL_COUNT: usize = 1;
const _: [(); 11573] = [(); PINNED_SOURCE.len()];

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::webgpu_decl::{
        WGPUBufferBindingType_ReadOnlyStorage, WGPUBufferBindingType_Storage,
        WGPUBufferBindingType_Undefined, WGPUBufferBindingType_Uniform,
        WGPUSamplerBindingType_Comparison, WGPUSamplerBindingType_Filtering,
        WGPUSamplerBindingType_Undefined, WGPUShaderStage_Compute, WGPUShaderStage_Fragment,
        WGPUShaderStage_Vertex, WGPUStorageTextureAccess_WriteOnly,
        WGPUTextureSampleType_Depth, WGPUTextureViewDimension_2D,
        WGPUTextureViewDimension_CubeArray,
    };

    #[test]
    fn complete_source_denominator_is_locked() {
        assert_eq!(PINNED_SOURCE.lines().count(), 283);
        assert_eq!(kWGPUMaxGroups, kMaxBindGroups);
        assert_eq!(SOURCE_INLINE_FUNCTION_COUNT, 5);
        assert_eq!(SOURCE_SWITCH_COUNT, 6);
        assert_eq!(SOURCE_SWITCH_CASE_COUNT, 38);
        assert_eq!(SOURCE_RESOURCE_KIND_CASE_COUNT, 7);
        assert_eq!(SOURCE_BINDING_KIND_CASE_COUNT, 7);
        assert_eq!(SOURCE_DEFAULT_ARGUMENT_COUNT, 3);
        assert_eq!(SOURCE_CREATE_BIND_GROUP_LAYOUT_CALL_COUNT, 1);
    }

    #[test]
    fn reflection_dimensions_and_sample_types_preserve_defaults_and_depth() {
        assert_eq!(
            toWGPUViewDim(TextureViewDim::Undefined).0,
            WGPUTextureViewDimension_2D as u32
        );
        assert_eq!(
            toWGPUViewDim(TextureViewDim::CubeArray).0,
            WGPUTextureViewDimension_CubeArray as u32
        );
        assert_eq!(
            toWGPUViewDim(TextureViewDim(255)).0,
            WGPUTextureViewDimension_2D as u32
        );
        assert_eq!(
            toWGPUSampleType(OreTextureSampleType::Depth).0,
            WGPUTextureSampleType_Depth as u32
        );
        assert_eq!(
            toWGPUSampleType(OreTextureSampleType(255)),
            WGPUTextureSampleType::Float
        );
    }

    #[test]
    fn binding_map_resource_kinds_fill_only_the_source_selected_shape() {
        let visibility =
            (WGPUShaderStage::Vertex | WGPUShaderStage::Fragment).intoBitmask();
        let uniform = makeWGPUBGLEntryWithSourceDefaults(
            9,
            ResourceKind::UniformBuffer,
            true,
            visibility,
        );
        assert_eq!(uniform.binding, 9);
        assert_eq!(uniform.visibility, WGPUShaderStage_Vertex | WGPUShaderStage_Fragment);
        assert_eq!(uniform.buffer.r#type, WGPUBufferBindingType_Uniform);
        assert_eq!(uniform.buffer.hasDynamicOffset, WGPU_TRUE);
        assert_eq!(uniform.texture.sampleType, 0);
        assert_eq!(uniform.sampler.r#type, 0);

        let sampled = makeWGPUBGLEntry(
            3,
            ResourceKind::SampledTexture,
            false,
            WGPUShaderStage::Compute,
            TextureViewDim::CubeArray,
            OreTextureSampleType::Depth,
            true,
        );
        assert_eq!(sampled.visibility, WGPUShaderStage_Compute);
        assert_eq!(sampled.texture.sampleType, WGPUTextureSampleType_Depth);
        assert_eq!(sampled.texture.viewDimension, WGPUTextureViewDimension_CubeArray);
        assert_eq!(sampled.texture.multisampled, WGPU_TRUE);
        assert_eq!(sampled.buffer.r#type, 0);
        assert_eq!(sampled.sampler.r#type, 0);

        let unknown = makeWGPUBGLEntryWithSourceDefaults(
            1,
            ResourceKind(255),
            false,
            WGPUShaderStage::None,
        );
        assert_eq!(unknown.buffer.r#type, 0);
        assert_eq!(unknown.sampler.r#type, 0);
        assert_eq!(unknown.texture.sampleType, 0);
        assert_eq!(unknown.storageTexture.access, 0);
    }

    #[test]
    fn public_desc_maps_every_resource_shape_and_stage_mask() {
        let cases = [
            (BindingKind::uniformBuffer, WGPUBufferBindingType_Uniform),
            (BindingKind::storageBufferRO, WGPUBufferBindingType_ReadOnlyStorage),
            (BindingKind::storageBufferRW, WGPUBufferBindingType_Storage),
        ];
        for (kind, expected) in cases {
            let src = OreBindGroupLayoutEntry {
                kind,
                visibility: StageVisibility {
                    mask: StageVisibility::kVertex
                        | StageVisibility::kFragment
                        | StageVisibility::kCompute,
                },
                hasDynamicOffset: true,
                minBindingSize: 77,
                ..Default::default()
            };
            let entry = makeWGPUBGLEntryFromDesc(&src);
            assert_eq!(entry.buffer.r#type, expected);
            assert_eq!(entry.buffer.minBindingSize, 77);
            assert_eq!(
                entry.visibility,
                WGPUShaderStage_Vertex | WGPUShaderStage_Fragment | WGPUShaderStage_Compute
            );
        }

        let sampler = makeWGPUBGLEntryFromDesc(&OreBindGroupLayoutEntry {
            kind: BindingKind::sampler,
            ..Default::default()
        });
        let comparison = makeWGPUBGLEntryFromDesc(&OreBindGroupLayoutEntry {
            kind: BindingKind::comparisonSampler,
            ..Default::default()
        });
        assert_eq!(sampler.sampler.r#type, WGPUSamplerBindingType_Filtering);
        assert_eq!(comparison.sampler.r#type, WGPUSamplerBindingType_Comparison);

        let storage = makeWGPUBGLEntryFromDesc(&OreBindGroupLayoutEntry {
            kind: BindingKind::storageTexture,
            textureViewDim: OreTextureViewDimension::cubeArray,
            ..Default::default()
        });
        assert_eq!(storage.storageTexture.access, WGPUStorageTextureAccess_WriteOnly);
        assert_eq!(storage.storageTexture.format, WGPUTextureFormat_RGBA8Unorm);
        assert_eq!(
            storage.storageTexture.viewDimension,
            WGPUTextureViewDimension_CubeArray
        );
    }

    #[test]
    fn descriptor_build_caps_at_sixteen_and_preserves_zero_entry_null_rule() {
        let authored = vec![OreBindGroupLayoutEntry::default(); 18];
        let desc = BindGroupLayoutDesc {
            entries: &authored,
            entryCount: 18,
            ..Default::default()
        };
        let (entries, n) = prepareWGPUBindGroupLayoutFromDesc(&desc);
        assert_eq!(n, 16);
        assert_eq!(entries[15].binding, 0);

        let explicitly_short = BindGroupLayoutDesc {
            entries: &authored,
            entryCount: 3,
            ..Default::default()
        };
        let (_, short_n) = prepareWGPUBindGroupLayoutFromDesc(&explicitly_short);
        assert_eq!(short_n, 3);

        let empty = BindGroupLayoutDesc::default();
        let (_, empty_n) = prepareWGPUBindGroupLayoutFromDesc(&empty);
        assert_eq!(empty_n, 0);
        let empty_label = labelStringView(None);
        assert!(!empty_label.data.is_null());
        assert_eq!(empty_label.length, WGPU_STRLEN);
        let authored_label = labelStringView(Some("layout"));
        assert_eq!(authored_label.length, 6);
    }
}
