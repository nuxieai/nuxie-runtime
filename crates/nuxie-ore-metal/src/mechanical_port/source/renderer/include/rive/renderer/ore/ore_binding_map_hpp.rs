/*
 * Copyright 2026 Rive
 */

// #pragma once

// Portable C++ binding map matching the on-disk RSTB schema. Runtime
// parses one of these out of an RSTB sidecar via `fromBlob`; the
// editor-side toolchain produces the blob via `toBlob`.
//
// Usage pattern:
//   ore::BindingMap bm;
//   ore::BindingMap::fromBlob(bytes, size, &bm);
//   uint32_t metalSlot = bm.lookup(group, binding,
//                                  ore::ResourceKind::UniformBuffer,
//                                  ore::BindingMap::Stage::VS);
//
// BindingMap is owned by `ore::Pipeline` — populated at pipeline creation
// from the RSTB sidecar, consumed by each flatten backend's
// `makeBindGroup`. Vulkan/WebGPU construct it for reflection parity but
// never read it at bind time.

// #include <algorithm>
// #include <cassert>
// #include <cstdint>
// #include <cstring>
// #include <vector>

// namespace rive::ore
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#[cfg(all(test, feature = "with-rive-tools"))]
mod tests {
    use super::*;

    fn push_entry(map: &mut BindingMap, entry: BindingMapEntry) {
        map.push(&entry);
    }

    fn make_entry(
        group: u8,
        binding: u8,
        kind: ResourceKind,
        slot_vs: u16,
        slot_fs: u16,
        slot_cs: u16,
        stageMask: u8,
        backendSpace: u8,
    ) -> BindingMapEntry {
        BindingMapEntry {
            group,
            binding,
            kind,
            stageMask,
            backendSpace,
            backendSlot: [slot_vs, slot_fs, slot_cs],
            ..BindingMapEntry::default()
        }
    }

    #[test]
    fn binding_map_lookup_after_finalize() {
        let mut map = BindingMap::default();
        push_entry(
            &mut map,
            make_entry(
                1,
                2,
                ResourceKind::SampledTexture,
                5,
                5,
                BindingMap::kAbsent,
                (BindingMap::kStageVertex | BindingMap::kStageFragment) as u8,
                0,
            ),
        );
        push_entry(
            &mut map,
            make_entry(
                0,
                1,
                ResourceKind::UniformBuffer,
                1,
                1,
                BindingMap::kAbsent,
                (BindingMap::kStageVertex | BindingMap::kStageFragment) as u8,
                0,
            ),
        );
        push_entry(
            &mut map,
            make_entry(
                0,
                0,
                ResourceKind::UniformBuffer,
                0,
                0,
                BindingMap::kAbsent,
                (BindingMap::kStageVertex | BindingMap::kStageFragment) as u8,
                0,
            ),
        );
        map.finalize();

        assert_eq!(map.size(), 3);
        assert_eq!(map.at(0).group, 0);
        assert_eq!(map.at(0).binding, 0);
        assert_eq!(map.at(1).group, 0);
        assert_eq!(map.at(1).binding, 1);
        assert_eq!(map.at(2).group, 1);
        assert_eq!(map.at(2).binding, 2);
        assert_eq!(map.lookup(0, 0, ResourceKind::UniformBuffer, Stage::VS), 0);
        assert_eq!(map.lookup(0, 1, ResourceKind::UniformBuffer, Stage::VS), 1);
        assert_eq!(map.lookup(1, 2, ResourceKind::SampledTexture, Stage::VS), 5);
        assert_eq!(
            map.lookup(9, 9, ResourceKind::UniformBuffer, Stage::VS),
            BindingMap::kAbsent
        );
        assert_eq!(
            map.lookup(0, 0, ResourceKind::Sampler, Stage::VS),
            BindingMap::kAbsent
        );
    }

    #[test]
    fn binding_map_stage_visibility() {
        let mut map = BindingMap::default();
        push_entry(
            &mut map,
            make_entry(
                0,
                0,
                ResourceKind::UniformBuffer,
                3,
                BindingMap::kAbsent,
                BindingMap::kAbsent,
                BindingMap::kStageVertex as u8,
                0,
            ),
        );
        map.finalize();
        assert_eq!(map.lookup(0, 0, ResourceKind::UniformBuffer, Stage::VS), 3);
        assert_eq!(
            map.lookup(0, 0, ResourceKind::UniformBuffer, Stage::FS),
            BindingMap::kAbsent
        );
        assert_eq!(
            map.lookup(0, 0, ResourceKind::UniformBuffer, Stage::CS),
            BindingMap::kAbsent
        );
    }

    #[test]
    fn binding_map_fs_only_entry_hides_vs_cs() {
        let mut map = BindingMap::default();
        push_entry(
            &mut map,
            make_entry(
                2,
                0,
                ResourceKind::SampledTexture,
                BindingMap::kAbsent,
                4,
                BindingMap::kAbsent,
                BindingMap::kStageFragment as u8,
                0,
            ),
        );
        map.finalize();
        assert_eq!(map.lookup(2, 0, ResourceKind::SampledTexture, Stage::FS), 4);
        assert_eq!(
            map.lookup(2, 0, ResourceKind::SampledTexture, Stage::VS),
            BindingMap::kAbsent
        );
        assert_eq!(
            map.lookup(2, 0, ResourceKind::SampledTexture, Stage::CS),
            BindingMap::kAbsent
        );
    }

    #[test]
    fn binding_map_empty_stage_mask_hides_every_stage() {
        let mut map = BindingMap::default();
        push_entry(
            &mut map,
            make_entry(0, 0, ResourceKind::UniformBuffer, 9, 9, 9, 0, 0),
        );
        map.finalize();
        for stage in [Stage::VS, Stage::FS, Stage::CS] {
            assert_eq!(
                map.lookup(0, 0, ResourceKind::UniformBuffer, stage),
                BindingMap::kAbsent
            );
        }
    }

    #[test]
    fn binding_map_sampler_comparison_sampler_kind_collapse() {
        let mut map = BindingMap::default();
        push_entry(
            &mut map,
            make_entry(
                0,
                0,
                ResourceKind::ComparisonSampler,
                7,
                7,
                BindingMap::kAbsent,
                (BindingMap::kStageVertex | BindingMap::kStageFragment) as u8,
                0,
            ),
        );
        map.finalize();
        assert_eq!(
            map.lookup(0, 0, ResourceKind::ComparisonSampler, Stage::FS),
            7
        );
        assert_eq!(map.lookup(0, 0, ResourceKind::Sampler, Stage::FS), 7);
        assert_eq!(
            map.lookup(0, 0, ResourceKind::SampledTexture, Stage::FS),
            BindingMap::kAbsent
        );
        assert_eq!(
            map.lookup(0, 0, ResourceKind::UniformBuffer, Stage::FS),
            BindingMap::kAbsent
        );

        let mut reverse = BindingMap::default();
        push_entry(
            &mut reverse,
            make_entry(
                0,
                0,
                ResourceKind::Sampler,
                2,
                2,
                BindingMap::kAbsent,
                (BindingMap::kStageVertex | BindingMap::kStageFragment) as u8,
                0,
            ),
        );
        reverse.finalize();
        assert_eq!(reverse.lookup(0, 0, ResourceKind::Sampler, Stage::FS), 2);
        assert_eq!(
            reverse.lookup(0, 0, ResourceKind::ComparisonSampler, Stage::FS),
            2
        );
    }

    #[test]
    fn binding_map_per_stage_slots_can_disagree() {
        let mut map = BindingMap::default();
        push_entry(
            &mut map,
            make_entry(
                0,
                0,
                ResourceKind::SampledTexture,
                1,
                4,
                9,
                (BindingMap::kStageVertex | BindingMap::kStageFragment | BindingMap::kStageCompute)
                    as u8,
                0,
            ),
        );
        map.finalize();
        assert_eq!(map.lookup(0, 0, ResourceKind::SampledTexture, Stage::VS), 1);
        assert_eq!(map.lookup(0, 0, ResourceKind::SampledTexture, Stage::FS), 4);
        assert_eq!(map.lookup(0, 0, ResourceKind::SampledTexture, Stage::CS), 9);
    }

    #[test]
    fn binding_map_toBlob_fromBlob_round_trip() {
        let mut original = BindingMap::default();
        push_entry(
            &mut original,
            make_entry(
                0,
                0,
                ResourceKind::UniformBuffer,
                0,
                0,
                BindingMap::kAbsent,
                (BindingMap::kStageVertex | BindingMap::kStageFragment) as u8,
                0,
            ),
        );
        push_entry(
            &mut original,
            make_entry(
                0,
                7,
                ResourceKind::UniformBuffer,
                1,
                1,
                BindingMap::kAbsent,
                (BindingMap::kStageVertex | BindingMap::kStageFragment) as u8,
                0,
            ),
        );
        push_entry(
            &mut original,
            make_entry(
                2,
                0,
                ResourceKind::SampledTexture,
                5,
                5,
                BindingMap::kAbsent,
                BindingMap::kStageFragment as u8,
                2,
            ),
        );
        original.finalize();

        let blob = original.toBlob();
        assert_eq!(blob.len(), 50);
        assert_eq!(blob[0], BindingMap::kBlobVersion);
        assert_eq!(blob[1], BindingMap::kAllocatorVersion);
        assert_eq!(&blob[2..4], &[14, 0]);
        assert_eq!(&blob[4..8], &[3, 0, 0, 0]);

        let mut restored = BindingMap::default();
        assert!(BindingMap::fromBlob(
            Some(&blob),
            (&blob).len(),
            Some(&mut restored)
        ));
        assert_eq!(restored.size(), 3);
        for index in 0..original.size() {
            assert_eq!(original.at(index), restored.at(index));
        }
    }

    #[test]
    fn binding_map_rejects_bad_blob_version() {
        let mut source = BindingMap::default();
        push_entry(
            &mut source,
            make_entry(
                0,
                0,
                ResourceKind::UniformBuffer,
                0,
                0,
                BindingMap::kAbsent,
                (BindingMap::kStageVertex | BindingMap::kStageFragment) as u8,
                0,
            ),
        );
        source.finalize();
        let blob = source.toBlob();
        let mut out = BindingMap::default();
        assert!(!BindingMap::fromBlob(None, 0, Some(&mut out)));
        assert!(!BindingMap::fromBlob(
            Some(&blob[..4]),
            (&blob[..4]).len(),
            Some(&mut out)
        ));

        let mut bad_blob_version = blob.clone();
        bad_blob_version[0] = BindingMap::kBlobVersion + 1;
        assert!(!BindingMap::fromBlob(
            Some(&bad_blob_version),
            (&bad_blob_version).len(),
            Some(&mut out)
        ));

        let mut bad_allocator_version = blob;
        bad_allocator_version[1] = BindingMap::kAllocatorVersion + 1;
        assert!(!BindingMap::fromBlob(
            Some(&bad_allocator_version),
            (&bad_allocator_version).len(),
            Some(&mut out)
        ));
    }

    #[test]
    fn binding_map_rejects_truncated_blob() {
        let mut source = BindingMap::default();
        push_entry(
            &mut source,
            make_entry(
                0,
                0,
                ResourceKind::UniformBuffer,
                0,
                0,
                BindingMap::kAbsent,
                (BindingMap::kStageVertex | BindingMap::kStageFragment) as u8,
                0,
            ),
        );
        push_entry(
            &mut source,
            make_entry(
                1,
                0,
                ResourceKind::SampledTexture,
                0,
                0,
                BindingMap::kAbsent,
                (BindingMap::kStageVertex | BindingMap::kStageFragment) as u8,
                0,
            ),
        );
        source.finalize();
        let blob = source.toBlob();
        let mut out = BindingMap::default();
        assert!(BindingMap::fromBlob(
            Some(&blob),
            (&blob).len(),
            Some(&mut out)
        ));
        assert!(!BindingMap::fromBlob(
            Some(&blob[..blob.len() - 1]),
            (&blob[..blob.len() - 1]).len(),
            Some(&mut out)
        ));
        assert!(out.empty());
    }

    #[test]
    fn binding_map_forward_compat_larger_entry_size_parses() {
        let mut source = BindingMap::default();
        push_entry(
            &mut source,
            make_entry(
                0,
                0,
                ResourceKind::UniformBuffer,
                0,
                0,
                BindingMap::kAbsent,
                (BindingMap::kStageVertex | BindingMap::kStageFragment) as u8,
                0,
            ),
        );
        source.finalize();
        let blob = source.toBlob();
        let current_entry_size = u16::from_le_bytes([blob[2], blob[3]]);
        const EXTRA_TRAILING: u16 = 4;
        let future_entry_size = current_entry_size + EXTRA_TRAILING;
        let mut future_blob = vec![0u8; 8 + usize::from(future_entry_size)];
        future_blob[0] = BindingMap::kBlobVersion;
        future_blob[1] = BindingMap::kAllocatorVersion;
        future_blob[2..4].copy_from_slice(&future_entry_size.to_le_bytes());
        future_blob[4] = 1;
        future_blob[8..8 + usize::from(current_entry_size)]
            .copy_from_slice(&blob[8..8 + usize::from(current_entry_size)]);
        future_blob[8 + usize::from(current_entry_size)..].fill(0xff);

        let mut out = BindingMap::default();
        assert!(BindingMap::fromBlob(
            Some(&future_blob),
            (&future_blob).len(),
            Some(&mut out)
        ));
        assert_eq!(out.size(), 1);
        assert_eq!(out.at(0).group, 0);
        assert_eq!(out.at(0).binding, 0);
        assert_eq!(out.at(0).kind, ResourceKind::UniformBuffer);
    }

    #[test]
    fn binding_map_forward_compat_smaller_entry_size_rejected() {
        let mut blob = vec![0u8; 8];
        blob[0] = BindingMap::kBlobVersion;
        blob[1] = BindingMap::kAllocatorVersion;
        blob[2..4].copy_from_slice(&10u16.to_le_bytes());
        let mut out = BindingMap::default();
        assert!(!BindingMap::fromBlob(
            Some(&blob),
            (&blob).len(),
            Some(&mut out)
        ));
    }

    #[test]
    fn binding_map_accepts_unknown_enum_bytes_and_nonzero_bool() {
        let mut blob = vec![0u8; 8 + 14usize];
        blob[0] = BindingMap::kBlobVersion;
        blob[1] = BindingMap::kAllocatorVersion;
        blob[2..4].copy_from_slice(&14u16.to_le_bytes());
        blob[4] = 1;
        blob[8 + 2] = 0xfa;
        blob[8 + 11] = 0xfb;
        blob[8 + 12] = 0xfc;
        blob[8 + 13] = 0x7f;

        let mut out = BindingMap::default();
        assert!(BindingMap::fromBlob(
            Some(&blob),
            (&blob).len(),
            Some(&mut out)
        ));
        assert_eq!(out.at(0).kind.0, 0xfa);
        assert_eq!(out.at(0).textureViewDim.0, 0xfb);
        assert_eq!(out.at(0).textureSampleType.0, 0xfc);
        assert!(out.at(0).textureMultisampled);
        let serialized = out.toBlob();
        assert_eq!(serialized[8 + 2], 0xfa);
        assert_eq!(serialized[8 + 11], 0xfb);
        assert_eq!(serialized[8 + 12], 0xfc);
        assert_eq!(serialized[8 + 13], 1);
    }

    #[test]
    fn oversized_size_adapters_reset_output_before_rejecting() {
        let mut populated = BindingMap::default();
        push_entry(
            &mut populated,
            make_entry(
                0,
                0,
                ResourceKind::UniformBuffer,
                0,
                0,
                BindingMap::kAbsent,
                BindingMap::kStageVertex as u8,
                0,
            ),
        );
        populated.finalize();
        let blob = populated.toBlob();

        let mut out = populated.clone();
        assert!(!BindingMap::fromBlob(
            Some(&blob),
            blob.len() + 1,
            Some(&mut out),
        ));
        assert!(out.empty());
        assert!(!out.isFinalized());

        out = populated;
        assert!(!BindingMap::fromBlob(
            Some(&blob),
            blob.len() + 1,
            Some(&mut out),
        ));
        assert!(out.empty());
        assert!(!out.isFinalized());
    }

    #[test]
    fn resource_kind_numeric_values_are_frozen() {
        assert_eq!(ResourceKind::UniformBuffer.0, 0);
        assert_eq!(ResourceKind::StorageBufferRO.0, 1);
        assert_eq!(ResourceKind::StorageBufferRW.0, 2);
        assert_eq!(ResourceKind::SampledTexture.0, 3);
        assert_eq!(ResourceKind::StorageTexture.0, 4);
        assert_eq!(ResourceKind::Sampler.0, 5);
        assert_eq!(ResourceKind::ComparisonSampler.0, 6);
    }

    #[test]
    fn lookupBackendSlot_helper() {
        let mut map = BindingMap::default();
        push_entry(
            &mut map,
            make_entry(
                0,
                0,
                ResourceKind::UniformBuffer,
                7,
                7,
                BindingMap::kAbsent,
                (BindingMap::kStageVertex | BindingMap::kStageFragment) as u8,
                0,
            ),
        );
        map.finalize();
        assert_eq!(
            lookupBackendSlot(&map, 0, 0, ResourceKind::UniformBuffer, Stage::VS),
            7
        );
        assert_eq!(
            lookupBackendSlot(&map, 0, 0, ResourceKind::UniformBuffer, Stage::FS),
            7
        );
    }
}

// ----------------------------------------------------------------------------
// Reserved native slots
//
// Ore reserves a small number of native slots in each backend's slot
// namespace for its own internal machinery (push-constant emulation,
// dynamic-sized storage-buffer length arrays). The binding-map
// allocator must never hand out a user binding at one of these slots, even
// though no current Ore feature consumes them — they are forward-compat
// scaffolding so that adding push constants or compute storage buffers
// later cannot collide with a slot a user shader has already allocated.
//
// Values must agree with the RSTB-emit path in scripting_workspace.
// ----------------------------------------------------------------------------

// Metal `[[buffer(30)]]` reservation. Metal's hardware buffer table is 31
// slots (`MTLBufferTableSize`); reserving the topmost matches Dawn's MSL
// backend convention (`kImmediateBlockBufferSlot`). Targets:
//   1. Push-constant emulation (Vulkan native, D3D12 root constants — Metal
//      has no equivalent, must be emulated as a buffer bind).
//   2. Sizes buffer for dynamic-sized storage-buffer `arrayLength()`
//      lookups (compute path, currently disabled — see
//      `Features::storageBuffers`).
pub const kMetalReservedBufferSlot: u32 = 30;

// Maximum buffer slot the binding-map allocator may assign to a user
// binding on Metal. Allocator stops at this; overflow is a hard error.
pub const kMetalMaxUserBufferSlot: u32 = kMetalReservedBufferSlot - 1;

// ----------------------------------------------------------------------------
// ResourceKind
//
// Numeric values are the frozen on-disk RSTB schema. Never renumber
// existing variants; new variants append at the next integer.
// ----------------------------------------------------------------------------

// This transparent byte preserves the C++ static_cast behavior in fromBlob:
// unknown future discriminants remain representable in an Entry.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ResourceKind(pub u8);

impl ResourceKind {
    pub const UniformBuffer: Self = Self(0);
    pub const StorageBufferRO: Self = Self(1);
    pub const StorageBufferRW: Self = Self(2);
    pub const SampledTexture: Self = Self(3);
    pub const StorageTexture: Self = Self(4);
    pub const Sampler: Self = Self(5);
    pub const ComparisonSampler: Self = Self(6);
}

// ----------------------------------------------------------------------------
// TextureViewDim / TextureSampleType
//
// Mirror Dawn's `wgpu::TextureViewDimension` and `wgpu::TextureSampleType`
// (matching numeric values) so the WebGPU backend can cast straight into
// the Dawn enums when building `BindGroupLayoutEntry.texture`. The values
// are what Dawn's frontend validator compares against the shader's reflected
// `BindingInfo` (see `ValidateCompatibilityOfSingleBindingWithLayout` in
// Dawn's `ShaderModule.cpp`).
//
// Carried per-entry in `BindingMap::Entry` for every backend — non-WebGPU
// backends ignore these fields since VK/D3D/Metal descriptor types are
// dimension-agnostic.
//
// Numeric values are frozen on-disk RSTB schema.
// ----------------------------------------------------------------------------

// Transparent bytes preserve the C++ static_cast behavior for forward schema
// values while retaining the named constants below.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextureViewDim(pub u8);

impl TextureViewDim {
    pub const Undefined: Self = Self(0);
    pub const D1: Self = Self(1);
    pub const D2: Self = Self(2);
    pub const D2Array: Self = Self(3);
    pub const Cube: Self = Self(4);
    pub const CubeArray: Self = Self(5);
    pub const D3: Self = Self(6);
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextureSampleType(pub u8);

impl TextureSampleType {
    pub const Undefined: Self = Self(0);
    pub const Float: Self = Self(1);
    pub const UnfilterableFloat: Self = Self(2);
    pub const Depth: Self = Self(3);
    pub const Sint: Self = Self(4);
    pub const Uint: Self = Self(5);
}

// ----------------------------------------------------------------------------
// BindingMap
// ----------------------------------------------------------------------------

// namespace-nested C++ declarations are source-shaped siblings because Rust
// does not permit nested enum/struct declarations in an impl/type body.

// Shader stage index for per-stage slot lookup. Order matches the
// fixed-width per-stage backend-slot fields in the on-disk RSTB row.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    VS = 0,
    FS = 1,
    CS = 2,
}

// One row of the binding map. Layout matches the on-disk RSTB row
// but packed tighter (fewer bits where the semantics allow).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BindingMapEntry {
    pub group: u8,
    pub binding: u8,
    pub kind: ResourceKind,
    pub stageMask: u8,         // Bitwise-OR of kStage* bits.
    pub backendSpace: u8,      // D3D12 register space / Vulkan set = group.
    pub backendSlot: [u16; 3], // [VS, FS, CS]
    // Texture reflection — populated for SampledTexture /
    // StorageTexture kinds; `Undefined` / `false` elsewhere. Consumed
    // by the WebGPU backend's BGL builder to feed Dawn's frontend
    // shader-vs-layout compatibility check. Ignored by VK / D3D /
    // Metal which bind textures via dimension-agnostic descriptors.
    pub textureViewDim: TextureViewDim,
    pub textureSampleType: TextureSampleType,
    pub textureMultisampled: bool,
}

// C++ `BindingMap::Entry` is a nested type; this top-level alias preserves
// the source spelling for translated method signatures.
pub type Entry = BindingMapEntry;

impl Default for BindingMapEntry {
    fn default() -> Self {
        Self {
            group: 0,
            binding: 0,
            kind: ResourceKind::UniformBuffer,
            stageMask: 0,
            backendSpace: 0,
            backendSlot: [
                BindingMap::kAbsent,
                BindingMap::kAbsent,
                BindingMap::kAbsent,
            ],
            textureViewDim: TextureViewDim::Undefined,
            textureSampleType: TextureSampleType::Undefined,
            textureMultisampled: false,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BindingMap {
    pub(crate) m_entries: Vec<Entry>,
    #[cfg(feature = "with-rive-tools")]
    pub(crate) m_finalized: bool,
}

impl BindingMap {
    // BindingMap() = default;

    // Stage bitmask bits, frozen RSTB schema.
    pub const kStageVertex: u32 = 1u32 << 0;
    pub const kStageFragment: u32 = 1u32 << 1;
    pub const kStageCompute: u32 = 1u32 << 2;

    // Sentinel for a per-stage slot that the resource is not visible to.
    // The 16-bit width fits every real slot on every backend
    // (Metal: 31, D3D11: 128, GL: ~64).
    pub const kAbsent: u16 = u16::MAX;

    // RSTB blob version byte. Bumped when the on-disk schema changes in a
    // way that renders old blobs unreadable. A mismatch on load is a loud
    // error, never a silent misbind.
    pub const kBlobVersion: u8 = 2;

    // Allocator version currently supported. Pipelines load with this
    // value; any blob stamped with a different version fails `fromBlob`
    // with a clear error.
    //
    // WebGPU-aligned global-counter-per-kind allocation.
    pub const kAllocatorVersion: u8 = 1;

    // Parse a blob produced by `toBlob` (or by the RSTB-emit path in
    // scripting_workspace). Returns a populated `BindingMap` + `true` on
    // success; empty map + `false` on any size/version mismatch. The
    // caller is expected to surface a clear error on failure — never
    // fall back to a different layout silently.
    //
    // First two bytes of the blob are `kBlobVersion` and
    // `kAllocatorVersion`; mismatch either and parse fails loudly.
    // Serialization (`toBlob`) lives in the tooling-gated portion of the API.
    // C++ declaration (defined by the paired ore_binding_map.cpp translation):
    // static bool fromBlob(const uint8_t* data, size_t size, BindingMap* out);

    // Per-stage backend-slot lookup. Returns kAbsent when the resource
    // is not in the map, or not visible to the requested stage, or the
    // map entry's kind doesn't match the caller's expected kind.
    //
    // `Sampler` and `ComparisonSampler` are treated as interchangeable
    // — the runtime binding API (both Luau-facing `GPUSampler` and the
    // C++ `BindGroupDesc::SampEntry`) is a single "sampler" category,
    // so the caller can't distinguish them at bind time. The allocator
    // stores whichever kind was declared in WGSL; this helper accepts
    // either side's query.
    //
    // The single hot path the flatten-backend `makeBindGroup` calls at
    // pipeline-binding time.
    pub fn lookup(&self, group: u32, binding: u32, kind: ResourceKind, stage: Stage) -> u16 {
        let e = match self.findEntry(group, binding) {
            Some(e) => e,
            None => return Self::kAbsent,
        };
        let kindMatches = e.kind == kind
            || ((kind == ResourceKind::Sampler || kind == ResourceKind::ComparisonSampler)
                && (e.kind == ResourceKind::Sampler || e.kind == ResourceKind::ComparisonSampler));
        if !kindMatches {
            return Self::kAbsent;
        }
        let stageBit = 1u32 << (stage as u32);
        if ((e.stageMask as u32) & stageBit) == 0 {
            return Self::kAbsent;
        }
        e.backendSlot[stage as usize]
    }

    pub fn size(&self) -> usize {
        self.m_entries.len()
    }

    pub fn empty(&self) -> bool {
        self.m_entries.is_empty()
    }

    // Iteration accessor. Used by every flatten backend's layout
    // builder at pipeline creation time (Vulkan DSL, D3D12 root sig,
    // WebGPU BGL) to walk the map's entries and emit one layout
    // entry per binding. Runtime-available because those builders
    // ship in non-tooling runtimes (`rive_native` without
    // `WITH_RIVE_TOOLS`).
    pub fn at(&self, i: usize) -> &Entry {
        &self.m_entries[i]
    }

    // ----------------------------------------------------------------
    // Tooling-only API. Compiled only in builds that define
    // WITH_RIVE_TOOLS (editor, scripting_workspace, unit_tests). The
    // shipped runtime binary has none of this — no std::sort, no
    // serialization, no state flag.
    // ----------------------------------------------------------------
    // C++ declaration (defined by the paired ore_binding_map.cpp translation):
    // std::vector<uint8_t> toBlob() const;
    #[cfg(feature = "with-rive-tools")]
    pub fn push(&mut self, e: &Entry) {
        self.m_entries.push(*e);
        self.m_finalized = false;
    }

    // C++ declaration (defined by the paired ore_binding_map.cpp translation):
    // void finalize();
    #[cfg(feature = "with-rive-tools")]
    pub fn isFinalized(&self) -> bool {
        self.m_finalized
    }

    #[cfg(feature = "with-rive-tools")]
    // Iteration accessor returning the backing vector — tooling-only
    // because raw vector access is used by RSTB emit / inspectors
    // that may modify via const_cast patterns. Runtime code uses
    // `at(i)` + `size()` above.
    pub fn entries(&self) -> &Vec<Entry> {
        &self.m_entries
    }

    #[cfg(feature = "with-rive-tools")]
    // Slot-unaware lookup for tools that want the full entry.
    pub fn lookupEntry(&self, group: u32, binding: u32) -> Option<&Entry> {
        self.findEntry(group, binding)
    }

    // Internal binary search — not tooling-gated because lookup() uses it
    // on the hot runtime path.
    fn findEntry(&self, group: u32, binding: u32) -> Option<&Entry> {
        #[cfg(feature = "with-rive-tools")]
        debug_assert!(self.m_finalized, "BindingMap::lookup before finalize");

        // Translation of std::lower_bound over (group, binding). The loop
        // deliberately returns the first equivalent row, as lower_bound does
        // for malformed/tooling input containing duplicate keys.
        let mut first = 0usize;
        let mut last = self.m_entries.len();
        while first < last {
            let middle = first + (last - first) / 2;
            let key = (
                self.m_entries[middle].group as u32,
                self.m_entries[middle].binding as u32,
            );
            if key < (group, binding) {
                first = middle + 1;
            } else {
                last = middle;
            }
        }
        if first == self.m_entries.len()
            || self.m_entries[first].group as u32 != group
            || self.m_entries[first].binding as u32 != binding
        {
            return None;
        }
        Some(&self.m_entries[first])
    }
}

// Shared helper for flatten-backend `makeBindGroup` implementations.
// Asserts on missing binding in debug builds; returns kAbsent in release
// so the caller can recover / skip the invalid bind (matches how D3D12
// handles "slot not set" via its slot-mask bitmap).
pub fn lookupBackendSlot(
    map: &BindingMap,
    group: u32,
    binding: u32,
    kind: ResourceKind,
    stage: Stage,
) -> u16 {
    let slot = map.lookup(group, binding, kind, stage);
    debug_assert_ne!(
        slot,
        BindingMap::kAbsent,
        "BindingMap lookup failed for (group, binding, kind, stage)"
    );
    slot
}

// namespace rive::ore
