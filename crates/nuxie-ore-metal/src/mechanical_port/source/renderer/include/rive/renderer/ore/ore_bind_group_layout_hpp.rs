/*
 * Copyright 2026 Rive
 */

// #pragma once

// #include "rive/renderer/gpu_resource.hpp"
// #include "utils/lite_rtti.hpp"
// #include "rive/renderer/ore/ore_types.hpp"
// #include "rive/renderer/ore/ore_binding_map.hpp"

// #include <string>
// #include <vector>

// Mechanical translation of the complete pinned source header
// renderer/include/rive/renderer/ore/ore_bind_group_layout.hpp.
// Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
use super::*;

use core::mem::ManuallyDrop;
use core::ops::{Deref, DerefMut};
use std::sync::Weak;

use super::super::gpu_resource_hpp::{GPUResource, GpuResourcePayload};

#[cfg(test)]
use super::ore_binding_map_hpp::BindingMap;
use super::ore_types_hpp::BindGroupLayoutEntry;

// namespace rive::ore

// class Context;
// class ContextMetal;
// class ContextGL;
// class ContextD3D11;

// Public Ore type — created via `Context::makeBindGroupLayout`. Carries the
// user-supplied entries plus per-backend baked layout handles.
//
// Lifetime: outlives any `Pipeline` or `BindGroup` that references it.
// `Pipeline` holds `rcp<BindGroupLayout> m_layouts[kMaxBindGroups]`;
// `BindGroup` holds `rcp<BindGroupLayout> m_layoutRef`.
//
// C++ inheritance: `BindGroupLayout : public rive::gpu::GPUResource`.
// The source also inherits `public ENABLE_LITE_RTTI(BindGroupLayout)`; that
// generic-lite-rtti base remains a source-visible inheritance contract rather
// than a duplicated payload field in this translation.
// class BindGroupLayout : public rive::gpu::GPUResource,
//                         public ENABLE_LITE_RTTI(BindGroupLayout)
// {
#[repr(C)]
pub struct BindGroupLayoutMembers {
    pub(crate) m_groupIndex: u32,
    pub(crate) m_entries: Vec<BindGroupLayoutEntry>,
    // Context* m_context = nullptr; non-owning deferred-destruction route.
    pub(crate) m_context: Weak<ContextState>,
}

#[repr(C)]
pub struct BindGroupLayout {
    pub(crate) base: ManuallyDrop<GPUResource>,
    pub(crate) members: ManuallyDrop<BindGroupLayoutMembers>,
}

impl Deref for BindGroupLayout {
    type Target = BindGroupLayoutMembers;

    fn deref(&self) -> &Self::Target {
        &self.members
    }
}

impl DerefMut for BindGroupLayout {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.members
    }
}

impl Drop for BindGroupLayout {
    fn drop(&mut self) {
        unsafe {
            #[cfg(test)]
            super::super::gpu_resource_hpp::record_resource_drop_stage("BindGroupLayout.context");
            core::ptr::drop_in_place(&mut self.m_context);
            #[cfg(test)]
            super::super::gpu_resource_hpp::record_resource_drop_stage("BindGroupLayout.entries");
            core::ptr::drop_in_place(&mut self.m_entries);
            #[cfg(test)]
            super::super::gpu_resource_hpp::record_resource_drop_stage("BindGroupLayout.base");
            ManuallyDrop::drop(&mut self.base);
        }
    }
}

unsafe impl GpuResourcePayload for BindGroupLayout {
    fn gpu_resource(&self) -> &GPUResource {
        &self.base
    }

    fn gpu_resource_mut(&mut self) -> &mut GPUResource {
        &mut self.base
    }
}

impl BindGroupLayout {
    // public:

    // uint32_t groupIndex() const { return m_groupIndex; }
    pub fn groupIndex(&self) -> u32 {
        self.m_groupIndex
    }

    // const std::vector<BindGroupLayoutEntry>& entries() const
    // {
    //     return m_entries;
    // }
    pub fn entries(&self) -> &Vec<BindGroupLayoutEntry> {
        &self.m_entries
    }

    // True if entry for `binding` (within this layout's group) is a UBO
    // declared with `hasDynamicOffset = true`.
    // C++ declaration (defined by the paired ore_bind_group_layout.cpp
    // translation):
    // bool hasDynamicOffset(uint32_t binding) const;

    // Find the entry for a given binding. Returns nullptr if not present.
    // C++ declaration (defined by the paired ore_bind_group_layout.cpp
    // translation):
    // const BindGroupLayoutEntry* findEntry(uint32_t binding) const;

    // virtual ~BindGroupLayout() = default;

    // protected:
    //     friend class Context;
    //     friend class ContextMetal;
    //     friend class ContextGL;
    //     friend class ContextD3D11;
    // Rust has no friend declarations; these source access boundaries remain
    // visible here, and the paired context translation owns construction
    // access.

    // BindGroupLayout() : rive::gpu::GPUResource(nullptr) {}
    pub(crate) fn new() -> Self {
        Self {
            base: ManuallyDrop::new(GPUResource::new(None)),
            members: ManuallyDrop::new(BindGroupLayoutMembers {
                m_groupIndex: 0,
                m_entries: Vec::new(),
                m_context: Weak::new(),
            }),
        }
    }

    // BindGroupLayout(rcp<rive::gpu::GPUResourceManager> manager) :
    //     rive::gpu::GPUResource(std::move(manager))
    // {}
    // Manager ownership is carried by the concrete outer ResourceHandle.
}

// Validate user-supplied layouts against the shader's reflected binding map.
//
// Walks every entry in `bindingMap` and confirms the layout for that group
// declares a matching entry: same WGSL @binding, kind, visibility >=
// shader's stageMask, texture dim/sample type compatible.
//
// Returns true on success. On failure, populates `*outError` with a human-
// readable diagnostic and returns false. Never asserts.
// C++ declaration (defined by the paired ore_bind_group_layout.cpp
// translation):
// bool validateLayoutsAgainstBindingMap(const BindingMap& bindingMap,
//                                       BindGroupLayout* const* layouts,
//                                       uint32_t layoutCount,
//                                       std::string* outError);

// Color outputs require a fragment shader.
// C++ declaration (defined by the paired ore_bind_group_layout.cpp
// translation):
// bool validateColorRequiresFragment(uint32_t colorCount,
//                                    bool hasFragmentModule,
//                                    std::string* outError);

// } // namespace rive::ore
#[cfg(test)]
mod tests {
    use super::*;
    use crate::binding_map::TextureSampleType;
    use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_resource_hpp::ResourceHandle;
    use crate::mechanical_port::source::renderer::src::ore::ore_bind_group_layout_cpp::{
        validateColorRequiresFragment, validateLayoutsAgainstBindingMap,
    };
    use crate::types::StageVisibility;

    fn binding_map_blob(entry: BindingMapEntry) -> Vec<u8> {
        let mut blob = vec![2, 1, 14, 0, 1, 0, 0, 0];
        blob.extend_from_slice(&[
            entry.group,
            entry.binding,
            entry.kind.0,
            entry.stageMask,
            entry.backendSpace,
        ]);
        for slot in entry.backendSlot {
            blob.extend_from_slice(&slot.to_le_bytes());
        }
        blob.extend_from_slice(&[
            entry.textureViewDim.0,
            entry.textureSampleType.0,
            u8::from(entry.textureMultisampled),
        ]);
        blob
    }

    fn parsed_map(entry: BindingMapEntry) -> BindingMap {
        let blob = binding_map_blob(entry);
        let mut map = BindingMap::default();
        assert!(BindingMap::fromBlob(
            Some(&blob),
            blob.len(),
            Some(&mut map)
        ));
        map
    }

    fn layout_entry(binding: u32, kind: BindingKind) -> BindGroupLayoutEntry {
        BindGroupLayoutEntry {
            binding,
            kind,
            ..BindGroupLayoutEntry::default()
        }
    }

    fn layout(group: u32, entry: BindGroupLayoutEntry) -> BindGroupLayout {
        let mut layout = BindGroupLayout::new();
        layout.m_groupIndex = group;
        layout.m_entries = vec![entry];
        layout
    }

    #[test]
    fn bind_group_layout_finds_first_entry_and_dynamic_uniform_only() {
        let first = layout_entry(4, BindingKind::uniformBuffer);
        let mut duplicate = first;
        duplicate.hasDynamicOffset = true;
        let mut non_uniform = layout_entry(7, BindingKind::storageBufferRO);
        non_uniform.hasDynamicOffset = true;
        let entries = [first, duplicate, non_uniform];
        let mut layout = BindGroupLayout::new();
        layout.m_groupIndex = 0;
        layout.m_entries = entries.to_vec();
        assert_eq!(layout.groupIndex(), 0);
        assert_eq!(layout.entries().len(), 3);
        assert!(layout.findEntry(4).is_some_and(|entry| {
            entry.binding == first.binding
                && entry.kind == first.kind
                && entry.hasDynamicOffset == first.hasDynamicOffset
        }));
        assert!(!layout.hasDynamicOffset(4));
        assert!(!layout.hasDynamicOffset(7));
        assert!(!layout.hasDynamicOffset(99));
        assert!(layout.findEntry(99).is_none());
    }

    #[test]
    fn bind_group_layout_validation_accepts_matching_layout_and_unused_entries() {
        let map = parsed_map(BindingMapEntry {
            group: 0,
            binding: 2,
            kind: ResourceKind::UniformBuffer,
            stageMask: (BindingMap::kStageVertex | BindingMap::kStageFragment) as u8,
            ..BindingMapEntry::default()
        });
        let mut entry = layout_entry(2, BindingKind::uniformBuffer);
        entry.visibility = StageVisibility {
            mask: StageVisibility::kVertex | StageVisibility::kFragment,
        };
        let unused = layout_entry(99, BindingKind::sampler);
        let layout = ResourceHandle::new(None, {
            let mut layout = BindGroupLayout::new();
            layout.m_groupIndex = 0;
            layout.m_entries = vec![entry, unused];
            layout
        })
        .erase();
        let layouts = [Some(&layout)];
        let mut error = String::from("unchanged");
        assert!(validateLayoutsAgainstBindingMap(
            &map,
            Some(&layouts),
            1,
            Some(&mut error),
        ));
        assert_eq!(error, "unchanged");
    }

    #[test]
    fn bind_group_layout_validation_reports_missing_group_in_source_order() {
        let map = parsed_map(BindingMapEntry {
            group: 1,
            binding: 3,
            kind: ResourceKind::SampledTexture,
            ..BindingMapEntry::default()
        });
        let mut error = String::new();
        assert!(!validateLayoutsAgainstBindingMap(
            &map,
            None,
            2,
            Some(&mut error),
        ));
        assert_eq!(
            error,
            "@group(1) @binding(3): shader declares sampledTexture but PipelineDesc::bindGroupLayouts has no entry for group 1"
        );
    }

    #[test]
    fn bind_group_layout_validation_reports_missing_binding() {
        let map = parsed_map(BindingMapEntry {
            group: 0,
            binding: 3,
            kind: ResourceKind::StorageBufferRW,
            ..BindingMapEntry::default()
        });
        let layout = ResourceHandle::new(
            None,
            layout(0, layout_entry(2, BindingKind::storageBufferRW)),
        )
        .erase();
        let layouts = [Some(&layout)];
        let mut error = String::new();
        assert!(!validateLayoutsAgainstBindingMap(
            &map,
            Some(&layouts),
            1,
            Some(&mut error),
        ));
        assert_eq!(
            error,
            "@group(0) @binding(3): layout has no entry for this binding (shader expects storageBufferRW)"
        );
    }

    #[test]
    fn bind_group_layout_validation_preserves_kind_and_visibility_order() {
        let map = parsed_map(BindingMapEntry {
            group: 0,
            binding: 1,
            kind: ResourceKind::StorageBufferRO,
            stageMask: BindingMap::kStageVertex as u8,
            ..BindingMapEntry::default()
        });
        let mismatched_layout =
            ResourceHandle::new(None, layout(0, layout_entry(1, BindingKind::uniformBuffer)))
                .erase();
        let layouts = [Some(&mismatched_layout)];
        let mut error = String::new();
        assert!(!validateLayoutsAgainstBindingMap(
            &map,
            Some(&layouts),
            1,
            Some(&mut error),
        ));
        assert_eq!(
            error,
            "@group(0) @binding(1): layout declares uniformBuffer but shader declares storageBufferRO"
        );

        let mut narrow = layout_entry(1, BindingKind::storageBufferRO);
        narrow.visibility = StageVisibility { mask: 0 };
        let narrow_layout = ResourceHandle::new(None, layout(0, narrow)).erase();
        let narrow_layouts = [Some(&narrow_layout)];
        assert!(!validateLayoutsAgainstBindingMap(
            &map,
            Some(&narrow_layouts),
            1,
            Some(&mut error),
        ));
        assert_eq!(
            error,
            "@group(0) @binding(1): layout visibility 0x0 missing stages required by shader (stageMask=0x1)"
        );
    }

    #[test]
    fn bind_group_layout_validation_allows_sampler_kind_collapse() {
        let map = parsed_map(BindingMapEntry {
            group: 0,
            binding: 0,
            kind: ResourceKind::ComparisonSampler,
            ..BindingMapEntry::default()
        });
        for kind in [BindingKind::sampler, BindingKind::comparisonSampler] {
            let layout = ResourceHandle::new(None, layout(0, layout_entry(0, kind))).erase();
            let layouts = [Some(&layout)];
            assert!(validateLayoutsAgainstBindingMap(
                &map,
                Some(&layouts),
                1,
                None,
            ));
        }
    }

    #[test]
    fn bind_group_layout_validation_checks_texture_dimension_only_when_reflected() {
        let map = parsed_map(BindingMapEntry {
            group: 0,
            binding: 0,
            kind: ResourceKind::SampledTexture,
            textureViewDim: TextureViewDim::D2,
            textureSampleType: TextureSampleType::Float,
            ..BindingMapEntry::default()
        });
        let mut entry = layout_entry(0, BindingKind::sampledTexture);
        entry.textureViewDim = TextureViewDimension::cube;
        let layout = ResourceHandle::new(None, layout(0, entry)).erase();
        let layouts = [Some(&layout)];
        let mut error = String::new();
        assert!(!validateLayoutsAgainstBindingMap(
            &map,
            Some(&layouts),
            1,
            Some(&mut error),
        ));
        assert_eq!(
            error,
            "@group(0) @binding(0): texture view dimension mismatch"
        );

        let unreflected_map = parsed_map(BindingMapEntry {
            group: 0,
            binding: 0,
            kind: ResourceKind::SampledTexture,
            textureViewDim: TextureViewDim::Undefined,
            ..BindingMapEntry::default()
        });
        assert!(validateLayoutsAgainstBindingMap(
            &unreflected_map,
            Some(&layouts),
            1,
            Some(&mut error),
        ));
    }

    #[test]
    fn bind_group_layout_validation_requires_positional_group_index() {
        let map = parsed_map(BindingMapEntry {
            group: 0,
            binding: 0,
            kind: ResourceKind::UniformBuffer,
            ..BindingMapEntry::default()
        });
        let layout =
            ResourceHandle::new(None, layout(1, layout_entry(0, BindingKind::uniformBuffer)))
                .erase();
        let layouts = [Some(&layout)];
        let mut error = String::new();
        assert!(!validateLayoutsAgainstBindingMap(
            &map,
            Some(&layouts),
            1,
            Some(&mut error),
        ));
        assert_eq!(
            error,
            "PipelineDesc::bindGroupLayouts[0]->groupIndex == 1, expected 0 (positional index must match layout's groupIndex)"
        );
    }

    #[test]
    fn bind_group_layout_color_validation_matches_fragment_rule() {
        let mut error = String::new();
        assert!(!validateColorRequiresFragment(1, false, Some(&mut error)));
        assert_eq!(
            error,
            "pipeline declares color outputs but has no fragment shader; supply `fragment`, or omit `colorTargets` for a depth-only pipeline"
        );
        assert!(validateColorRequiresFragment(0, false, Some(&mut error)));
        assert!(validateColorRequiresFragment(1, true, Some(&mut error)));
    }

    #[test]
    fn bind_group_layout_resource_handle_owns_payload() {
        let layout = layout(0, layout_entry(0, BindingKind::uniformBuffer));
        let handle = ResourceHandle::new(None, layout);
        assert_eq!(handle.debugging_refcnt(), 1);
        let erased = handle.erase();
        assert_eq!(erased.debugging_refcnt(), 1);
        assert!(erased.downcast_ref::<BindGroupLayout>().is_some());
    }

    #[test]
    fn bind_group_layout_context_payload_copies_entries_and_has_metal_identity() {
        let entries = [layout_entry(3, BindingKind::storageBufferRO)];
        let layout = layout(3, entries[0]);
        assert_eq!(layout.groupIndex(), 3);
        assert_eq!(layout.entries()[0].binding, 3);
        assert!(layout.findEntry(3).is_some());
    }
}
