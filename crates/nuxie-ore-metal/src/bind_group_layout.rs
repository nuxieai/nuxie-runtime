// Mechanical translation of:
//   renderer/include/rive/renderer/ore/ore_bind_group_layout.hpp
//   renderer/src/ore/ore_bind_group_layout.cpp
// Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5
//
// Copyright 2026 Rive

//! Portable ORE bind-group layout and shader-reflection validation.
//!
//! Upstream stores a weak raw `Context*`, but the pinned shared and Metal
//! sources only write that field and never read it. Rust deliberately omits
//! the dead back-pointer: [`ResourceHandle`] already retains the resource
//! manager that owns deferred destruction, without coupling this leaf to a
//! context type that has not been ported yet.

#![allow(non_snake_case)]

use crate::binding_map::{BindingMap, BindingMapEntry, ResourceKind, TextureViewDim};
use crate::gpu_resource::{GpuResourceManager, ResourceHandle};
use crate::metal::MetalBackend;
use crate::types::{BackendId, BindGroupLayoutEntry, BindingKind, TextureViewDimension};
use std::any::Any;

/// Public ORE bind-group layout.
///
/// C++ derives this object from `GPUResource`. Rust stores the data payload
/// here and uses [`BindGroupLayout::into_resource`] for the translated
/// deferred-destruction owner.
#[derive(Debug, PartialEq, Eq)]
pub struct BindGroupLayout {
    group_index: u32,
    entries: Vec<BindGroupLayoutEntry>,
}

impl BindGroupLayout {
    /// Copy the entries accepted by the context factory into the payload.
    ///
    /// The later `ContextMetal` translation owns descriptor range validation,
    /// error publication, and resource construction. This leaf only owns the
    /// protected C++ payload copy.
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "the pending ore-context-metal unit will own this constructor"
        )
    )]
    pub(crate) fn from_context_entries(group_index: u32, entries: &[BindGroupLayoutEntry]) -> Self {
        Self {
            group_index,
            entries: entries.to_vec(),
        }
    }

    pub fn group_index(&self) -> u32 {
        self.group_index
    }

    /// C++ spelling retained for source-corresponding callers.
    pub fn groupIndex(&self) -> u32 {
        self.group_index()
    }

    pub fn entries(&self) -> &[BindGroupLayoutEntry] {
        &self.entries
    }

    /// C++ spelling retained for source-corresponding callers.
    pub fn entries_ref(&self) -> &[BindGroupLayoutEntry] {
        self.entries()
    }

    /// Return whether a binding is a dynamic-offset uniform buffer.
    pub fn has_dynamic_offset(&self, binding: u32) -> bool {
        let Some(entry) = self.find_entry(binding) else {
            return false;
        };
        entry.kind == BindingKind::uniformBuffer && entry.hasDynamicOffset
    }

    /// C++ spelling retained for source-corresponding callers.
    pub fn hasDynamicOffset(&self, binding: u32) -> bool {
        self.has_dynamic_offset(binding)
    }

    /// Find the first entry for a binding, preserving authored vector order.
    pub fn find_entry(&self, binding: u32) -> Option<&BindGroupLayoutEntry> {
        self.entries.iter().find(|entry| entry.binding == binding)
    }

    /// C++ spelling retained for source-corresponding callers.
    pub fn findEntry(&self, binding: u32) -> Option<&BindGroupLayoutEntry> {
        self.find_entry(binding)
    }

    /// Adopt this payload into the translated `GPUResource` lifetime owner.
    pub fn into_resource(self, manager: Option<GpuResourceManager>) -> ResourceHandle<Self> {
        ResourceHandle::new(manager, self)
    }
}

impl crate::types::BindGroupLayout for BindGroupLayout {
    fn backend_id(&self) -> BackendId {
        BackendId::of::<MetalBackend>()
    }

    fn as_any(&self) -> &(dyn Any + Send + Sync) {
        self
    }
}

fn fail(out_error: &mut Option<&mut String>, message: String) -> bool {
    if let Some(error) = out_error.as_mut() {
        **error = message;
    }
    false
}

// Map ore::BindingKind (public layout API) ↔ ore::ResourceKind (binding-map
// internal). Kept private to this translation unit.
fn kinds_match(layout_kind: BindingKind, shader_kind: ResourceKind) -> bool {
    match layout_kind {
        BindingKind::uniformBuffer => shader_kind == ResourceKind::UniformBuffer,
        BindingKind::storageBufferRO => shader_kind == ResourceKind::StorageBufferRO,
        BindingKind::storageBufferRW => shader_kind == ResourceKind::StorageBufferRW,
        BindingKind::sampledTexture => shader_kind == ResourceKind::SampledTexture,
        BindingKind::storageTexture => shader_kind == ResourceKind::StorageTexture,
        BindingKind::sampler | BindingKind::comparisonSampler => {
            // Sampler / ComparisonSampler are interchangeable on the bind-API
            // side, matching BindingMap::lookup's collapse.
            shader_kind == ResourceKind::Sampler || shader_kind == ResourceKind::ComparisonSampler
        }
    }
}

fn kind_name(kind: BindingKind) -> &'static str {
    match kind {
        BindingKind::uniformBuffer => "uniformBuffer",
        BindingKind::storageBufferRO => "storageBufferRO",
        BindingKind::storageBufferRW => "storageBufferRW",
        BindingKind::sampledTexture => "sampledTexture",
        BindingKind::storageTexture => "storageTexture",
        BindingKind::sampler => "sampler",
        BindingKind::comparisonSampler => "comparisonSampler",
    }
}

fn shader_kind_name(kind: ResourceKind) -> &'static str {
    match kind {
        ResourceKind::UniformBuffer => "uniformBuffer",
        ResourceKind::StorageBufferRO => "storageBufferRO",
        ResourceKind::StorageBufferRW => "storageBufferRW",
        ResourceKind::SampledTexture => "sampledTexture",
        ResourceKind::StorageTexture => "storageTexture",
        ResourceKind::Sampler => "sampler",
        ResourceKind::ComparisonSampler => "comparisonSampler",
        ResourceKind(_) => "?",
    }
}

/// Validate user-supplied layouts against the shader's reflected binding map.
///
/// `layouts` and each element are optional to preserve the C++ null-pointer
/// branches. `layout_count` remains a separate argument because the upstream
/// API validates the positional count independently of the array itself.
/// `out_error == None` preserves the C++ null output-string path.
pub fn validate_layouts_against_binding_map(
    binding_map: &BindingMap,
    layouts: Option<&[Option<&BindGroupLayout>]>,
    layout_count: u32,
    mut out_error: Option<&mut String>,
) -> bool {
    // Every binding the shader references must have a corresponding layout
    // entry. Unused layout entries are allowed.
    for index in 0..binding_map.size() {
        let shader_entry: &BindingMapEntry = binding_map.at(index);
        let group = u32::from(shader_entry.group);
        let binding = u32::from(shader_entry.binding);

        let layout = if group >= layout_count {
            None
        } else {
            layouts
                .and_then(|all_layouts| all_layouts.get(group as usize))
                .and_then(|layout| *layout)
        };
        let Some(layout) = layout else {
            return fail(
                &mut out_error,
                format!(
                    "@group({group}) @binding({binding}): shader declares {} but PipelineDesc::bindGroupLayouts has no entry for group {group}",
                    shader_kind_name(shader_entry.kind)
                ),
            );
        };

        if layout.group_index() != group {
            return fail(
                &mut out_error,
                format!(
                    "PipelineDesc::bindGroupLayouts[{group}]->groupIndex == {}, expected {group} (positional index must match layout's groupIndex)",
                    layout.group_index()
                ),
            );
        }

        let Some(layout_entry) = layout.find_entry(binding) else {
            return fail(
                &mut out_error,
                format!(
                    "@group({group}) @binding({binding}): layout has no entry for this binding (shader expects {})",
                    shader_kind_name(shader_entry.kind)
                ),
            );
        };

        if !kinds_match(layout_entry.kind, shader_entry.kind) {
            return fail(
                &mut out_error,
                format!(
                    "@group({group}) @binding({binding}): layout declares {} but shader declares {}",
                    kind_name(layout_entry.kind),
                    shader_kind_name(shader_entry.kind)
                ),
            );
        }

        // Visibility narrower than the shader's stageMask is rejected.
        // Layout visibility broader than the shader is allowed.
        let shader_stage_mask = shader_entry.stageMask;
        let layout_visibility = layout_entry.visibility.mask;
        if shader_stage_mask & !layout_visibility != 0 {
            return fail(
                &mut out_error,
                format!(
                    "@group({group}) @binding({binding}): layout visibility 0x{layout_visibility:x} missing stages required by shader (stageMask=0x{shader_stage_mask:x})"
                ),
            );
        }

        // Texture dimension compatibility (texture kinds only). The upstream
        // implementation deliberately does not compare sample type here.
        if matches!(
            layout_entry.kind,
            BindingKind::sampledTexture | BindingKind::storageTexture
        ) {
            if shader_entry.textureViewDim != TextureViewDim::Undefined
                && !dims_match(layout_entry.textureViewDim, shader_entry.textureViewDim)
            {
                return fail(
                    &mut out_error,
                    format!("@group({group}) @binding({binding}): texture view dimension mismatch"),
                );
            }
        }
    }
    true
}

/// C++ spelling retained for source-corresponding callers.
pub fn validateLayoutsAgainstBindingMap(
    binding_map: &BindingMap,
    layouts: Option<&[Option<&BindGroupLayout>]>,
    layout_count: u32,
    out_error: Option<&mut String>,
) -> bool {
    validate_layouts_against_binding_map(binding_map, layouts, layout_count, out_error)
}

fn dims_match(layout_dimension: TextureViewDimension, shader_dimension: TextureViewDim) -> bool {
    match layout_dimension {
        TextureViewDimension::texture2D => shader_dimension == TextureViewDim::D2,
        TextureViewDimension::cube => shader_dimension == TextureViewDim::Cube,
        TextureViewDimension::texture3D => shader_dimension == TextureViewDim::D3,
        TextureViewDimension::array2D => shader_dimension == TextureViewDim::D2Array,
        TextureViewDimension::cubeArray => shader_dimension == TextureViewDim::CubeArray,
    }
}

/// Color outputs require a fragment shader.
pub fn validate_color_requires_fragment(
    color_count: u32,
    has_fragment_module: bool,
    mut out_error: Option<&mut String>,
) -> bool {
    if color_count > 0 && !has_fragment_module {
        return fail(
            &mut out_error,
            "pipeline declares color outputs but has no fragment shader; supply `fragment`, or omit `colorTargets` for a depth-only pipeline".to_owned(),
        );
    }
    true
}

/// C++ spelling retained for source-corresponding callers.
pub fn validateColorRequiresFragment(
    color_count: u32,
    has_fragment_module: bool,
    out_error: Option<&mut String>,
) -> bool {
    validate_color_requires_fragment(color_count, has_fragment_module, out_error)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::binding_map::TextureSampleType;
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
        assert!(BindingMap::from_blob(&blob, &mut map));
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
        BindGroupLayout::from_context_entries(group, &[entry])
    }

    #[test]
    fn bind_group_layout_finds_first_entry_and_dynamic_uniform_only() {
        let first = layout_entry(4, BindingKind::uniformBuffer);
        let mut duplicate = first;
        duplicate.hasDynamicOffset = true;
        let mut non_uniform = layout_entry(7, BindingKind::storageBufferRO);
        non_uniform.hasDynamicOffset = true;
        let entries = [first, duplicate, non_uniform];
        let layout = BindGroupLayout::from_context_entries(0, &entries);
        assert_eq!(layout.groupIndex(), 0);
        assert_eq!(layout.entries().len(), 3);
        assert_eq!(layout.findEntry(4), Some(&first));
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
            stageMask: (BindingMap::K_STAGE_VERTEX | BindingMap::K_STAGE_FRAGMENT) as u8,
            ..BindingMapEntry::default()
        });
        let mut entry = layout_entry(2, BindingKind::uniformBuffer);
        entry.visibility = StageVisibility {
            mask: StageVisibility::kVertex | StageVisibility::kFragment,
        };
        let unused = layout_entry(99, BindingKind::sampler);
        let layout = BindGroupLayout::from_context_entries(0, &[entry, unused]);
        let layouts = [Some(&layout)];
        let mut error = String::from("unchanged");
        assert!(validate_layouts_against_binding_map(
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
        assert!(!validate_layouts_against_binding_map(
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
        let layout = layout(0, layout_entry(2, BindingKind::storageBufferRW));
        let layouts = [Some(&layout)];
        let mut error = String::new();
        assert!(!validate_layouts_against_binding_map(
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
            stageMask: BindingMap::K_STAGE_VERTEX as u8,
            ..BindingMapEntry::default()
        });
        let mismatched_layout = layout(0, layout_entry(1, BindingKind::uniformBuffer));
        let layouts = [Some(&mismatched_layout)];
        let mut error = String::new();
        assert!(!validate_layouts_against_binding_map(
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
        let narrow_layout = layout(0, narrow);
        let narrow_layouts = [Some(&narrow_layout)];
        assert!(!validate_layouts_against_binding_map(
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
            let layout = layout(0, layout_entry(0, kind));
            let layouts = [Some(&layout)];
            assert!(validate_layouts_against_binding_map(
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
        let layout = layout(0, entry);
        let layouts = [Some(&layout)];
        let mut error = String::new();
        assert!(!validate_layouts_against_binding_map(
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
        assert!(validate_layouts_against_binding_map(
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
        let layout = layout(1, layout_entry(0, BindingKind::uniformBuffer));
        let layouts = [Some(&layout)];
        let mut error = String::new();
        assert!(!validate_layouts_against_binding_map(
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
        assert!(!validate_color_requires_fragment(
            1,
            false,
            Some(&mut error)
        ));
        assert_eq!(
            error,
            "pipeline declares color outputs but has no fragment shader; supply `fragment`, or omit `colorTargets` for a depth-only pipeline"
        );
        assert!(validate_color_requires_fragment(0, false, Some(&mut error)));
        assert!(validate_color_requires_fragment(1, true, Some(&mut error)));
    }

    #[test]
    fn bind_group_layout_resource_handle_owns_payload() {
        let layout = layout(0, layout_entry(0, BindingKind::uniformBuffer));
        let handle = layout.into_resource(None);
        assert_eq!(handle.debugging_ref_count(), 1);
        let erased = handle.erase();
        assert_eq!(erased.debugging_ref_count(), 1);
        assert!(erased.downcast_ref::<BindGroupLayout>().is_some());
    }

    #[test]
    fn bind_group_layout_context_payload_copies_entries_and_has_metal_identity() {
        let mut entries = [layout_entry(3, BindingKind::storageBufferRO)];
        let layout = BindGroupLayout::from_context_entries(3, &entries);
        entries[0].binding = 9;
        assert_eq!(entries[0].binding, 9);
        assert_eq!(layout.group_index(), 3);
        assert_eq!(layout.entries()[0].binding, 3);
        let resource: &dyn crate::types::BindGroupLayout = &layout;
        assert_eq!(resource.backend_id(), BackendId::of::<MetalBackend>());
        assert!(
            resource
                .downcast_ref::<BindGroupLayout>(BackendId::of::<MetalBackend>())
                .is_some()
        );

        enum OtherBackend {}
        assert!(
            resource
                .downcast_ref::<BindGroupLayout>(BackendId::of::<OtherBackend>())
                .is_none()
        );
    }
}
