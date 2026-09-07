/*
 * Copyright 2026 Rive
 */

// #include "rive/renderer/ore/ore_bind_group_layout.hpp"
// #include "rive/renderer/ore/ore_binding_map.hpp"

// #include <sstream>

// Mechanical translation of the complete pinned source implementation
// renderer/src/ore/ore_bind_group_layout.cpp.
// Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
use super::*;
use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_resource_hpp::AnyResourceHandle;
use crate::{context::ContextApi, shader_module::ShaderModule};

// namespace rive::ore

#[cfg(all(test, feature = "with-rive-tools"))]
mod shader_layout_tests {
    use super::*;

    #[test]
    fn shared_population_preserves_fields_and_reflects_binding_metadata() {
        let mut shader = ShaderModule::new();
        shader.m_bindingMap.push(&BindingMapEntry {
            group: 1,
            binding: 7,
            kind: ResourceKind::UniformBuffer,
            stageMask: 7,
            backendSlot: [4, BindingMap::kAbsent, 9],
            ..Default::default()
        });
        shader.m_bindingMap.push(&BindingMapEntry {
            group: 0,
            binding: 9,
            ..Default::default()
        });
        shader.m_bindingMap.push(&BindingMapEntry {
            group: 1,
            binding: 8,
            kind: ResourceKind::SampledTexture,
            textureViewDim: TextureViewDim::CubeArray,
            textureSampleType: TextureSampleType::Uint,
            textureMultisampled: true,
            ..Default::default()
        });
        let mut entries = [BindGroupLayoutEntry {
            minBindingSize: 123,
            nativeSlotCS: 456,
            ..Default::default()
        }; 2];
        assert_eq!(
            populateBindGroupLayoutEntriesFromShader(&mut entries, Some(&shader), 1, &[7, 8]),
            2
        );
        assert_eq!(entries[0].binding, 7);
        assert_eq!(entries[0].visibility.mask, 7);
        assert!(entries[0].hasDynamicOffset);
        assert_eq!(entries[0].nativeSlotVS, 4);
        assert_eq!(
            entries[0].nativeSlotFS,
            BindGroupLayoutEntry::kNativeSlotAbsent
        );
        assert_eq!(entries[0].minBindingSize, 123);
        assert_eq!(entries[0].nativeSlotCS, 456);
        assert_eq!(entries[1].binding, 8);
        assert!(!entries[1].hasDynamicOffset);
        assert!(entries[1].textureViewDim == TextureViewDimension::cubeArray);
        assert!(entries[1].textureSampleType == SampleType::uint);
        assert!(entries[1].textureMultisampled);
        assert_eq!(
            populateBindGroupLayoutEntriesFromShader(&mut entries[..1], Some(&shader), 1, &[]),
            2
        );
        assert!(!entries[0].hasDynamicOffset);
        assert_eq!(
            populateBindGroupLayoutEntriesFromShader(&mut entries, None, 1, &[]),
            0
        );
        assert_eq!(entries[0].binding, 7);
        assert_eq!(
            populateBindGroupLayoutEntriesFromShader(&mut [], Some(&shader), 1, &[]),
            2
        );
    }
}

impl BindGroupLayout {
    // const BindGroupLayoutEntry* BindGroupLayout::findEntry(uint32_t binding) const
    pub fn findEntry(&self, binding: u32) -> Option<&BindGroupLayoutEntry> {
        for e in &self.m_entries {
            if e.binding == binding {
                return Some(e);
            }
        }
        None
    }

    // bool BindGroupLayout::hasDynamicOffset(uint32_t binding) const
    pub fn hasDynamicOffset(&self, binding: u32) -> bool {
        let e = self.findEntry(binding);
        if let Some(e) = e {
            e.kind == BindingKind::uniformBuffer && e.hasDynamicOffset
        } else {
            false
        }
    }
}

// Map binding-map types to layout-entry types.
fn bindingKindFromResource(kind: ResourceKind) -> BindingKind {
    match kind {
        ResourceKind::UniformBuffer => BindingKind::uniformBuffer,
        ResourceKind::StorageBufferRO => BindingKind::storageBufferRO,
        ResourceKind::StorageBufferRW => BindingKind::storageBufferRW,
        ResourceKind::SampledTexture => BindingKind::sampledTexture,
        ResourceKind::StorageTexture => BindingKind::storageTexture,
        ResourceKind::Sampler => BindingKind::sampler,
        ResourceKind::ComparisonSampler => BindingKind::comparisonSampler,
        _ => BindingKind::uniformBuffer,
    }
}

fn viewDimFromBindingMap(dimension: TextureViewDim) -> TextureViewDimension {
    match dimension {
        TextureViewDim::Cube => TextureViewDimension::cube,
        TextureViewDim::CubeArray => TextureViewDimension::cubeArray,
        TextureViewDim::D3 => TextureViewDimension::texture3D,
        TextureViewDim::D2Array => TextureViewDimension::array2D,
        TextureViewDim::D1 | TextureViewDim::D2 | TextureViewDim::Undefined => {
            TextureViewDimension::texture2D
        }
        _ => TextureViewDimension::texture2D,
    }
}

fn sampleTypeFromBindingMap(sample: TextureSampleType) -> SampleType {
    match sample {
        TextureSampleType::UnfilterableFloat => SampleType::floatUnfilterable,
        TextureSampleType::Depth => SampleType::depth,
        TextureSampleType::Sint => SampleType::sint,
        TextureSampleType::Uint => SampleType::uint,
        TextureSampleType::Float | TextureSampleType::Undefined => SampleType::floatFilterable,
        _ => SampleType::floatFilterable,
    }
}

/// Fill entries from the shader's binding map, preserving caller-owned fields
/// not written upstream. The slice length represents `maxEntries`.
/// Returns the required count, which may exceed the provided slice length.
/// `dynamicUBOBindings` contains WGSL binding values within `groupIndex`.
pub fn populateBindGroupLayoutEntriesFromShader(
    entries: &mut [BindGroupLayoutEntry],
    shader: Option<&ShaderModule>,
    groupIndex: u32,
    dynamicUBOBindings: &[u32],
) -> u32 {
    let Some(shader) = shader else {
        return 0;
    };
    let mut n = 0;
    for i in 0..shader.m_bindingMap.size() {
        let e = shader.m_bindingMap.at(i);
        if u32::from(e.group) != groupIndex {
            continue;
        }
        let index = n;
        n += 1;
        let Some(out) = entries.get_mut(index) else {
            continue;
        };
        out.binding = u32::from(e.binding);
        out.kind = bindingKindFromResource(e.kind);
        let mut visibility = 0;
        for (source, destination) in [
            (BindingMap::kStageVertex, StageVisibility::kVertex),
            (BindingMap::kStageFragment, StageVisibility::kFragment),
            (BindingMap::kStageCompute, StageVisibility::kCompute),
        ] {
            if u32::from(e.stageMask) & source != 0 {
                visibility |= destination;
            }
        }
        out.visibility.mask = visibility;
        out.hasDynamicOffset =
            out.kind == BindingKind::uniformBuffer && dynamicUBOBindings.contains(&out.binding);
        out.textureViewDim = viewDimFromBindingMap(e.textureViewDim);
        out.textureSampleType = sampleTypeFromBindingMap(e.textureSampleType);
        out.textureMultisampled = e.textureMultisampled;
        let native_slot = |slot| {
            if slot == BindingMap::kAbsent {
                BindGroupLayoutEntry::kNativeSlotAbsent
            } else {
                u32::from(slot)
            }
        };
        out.nativeSlotVS = native_slot(e.backendSlot[0]);
        out.nativeSlotFS = native_slot(e.backendSlot[1]);
    }
    n as u32
}

/// Intern by baked identity, with heap storage for groups wider than sixteen.
pub fn makeBindGroupLayoutFromShader(
    ctx: &mut dyn ContextApi,
    shader: Option<&ShaderModule>,
    groupIndex: u32,
    dynamicUBOBindings: &[u32],
) -> Option<AnyResourceHandle> {
    let layoutId = if dynamicUBOBindings.is_empty() {
        shader.map_or(BindingMap::kNoLayoutId, |shader| {
            shader.m_bindingMap.layoutIdForGroup(groupIndex)
        })
    } else {
        BindingMap::kNoLayoutId
    };
    if layoutId != BindingMap::kNoLayoutId {
        if let Some(hit) = ctx.findInternedBindGroupLayout(layoutId) {
            return Some(hit);
        }
    }
    let mut entries = [BindGroupLayoutEntry::default(); 16];
    let n = populateBindGroupLayoutEntriesFromShader(
        &mut entries,
        shader,
        groupIndex,
        dynamicUBOBindings,
    );
    let mut spilled = Vec::new();
    let entries = if n > entries.len() as u32 {
        spilled.resize(n as usize, BindGroupLayoutEntry::default());
        populateBindGroupLayoutEntriesFromShader(
            &mut spilled, shader, groupIndex, dynamicUBOBindings,
        );
        spilled.as_slice()
    } else {
        entries.as_slice()
    };
    let layout = ctx.makeBindGroupLayout(&BindGroupLayoutDesc {
        groupIndex,
        entries: Some(entries),
        entryCount: n,
        ..BindGroupLayoutDesc::default()
    });
    if layoutId != BindingMap::kNoLayoutId {
        if let Some(layout) = &layout {
            ctx.internBindGroupLayout(layoutId, layout.clone());
        }
    }
    layout
}

// Map ore::BindingKind (public layout API) ↔ ore::ResourceKind (binding-map
// internal). Kept private to this TU.
#[allow(unreachable_patterns)] // Preserve the source defensive default for future enum values.
fn kindsMatch(layoutKind: BindingKind, shaderKind: ResourceKind) -> bool {
    match layoutKind {
        BindingKind::uniformBuffer => shaderKind == ResourceKind::UniformBuffer,
        BindingKind::storageBufferRO => shaderKind == ResourceKind::StorageBufferRO,
        BindingKind::storageBufferRW => shaderKind == ResourceKind::StorageBufferRW,
        BindingKind::sampledTexture => shaderKind == ResourceKind::SampledTexture,
        BindingKind::storageTexture => shaderKind == ResourceKind::StorageTexture,
        BindingKind::sampler => {
            // Sampler / ComparisonSampler are interchangeable on the
            // bind-API side — matches the BindingMap::lookup collapse
            // (ore_binding_map.hpp:201-208). Layout is allowed to declare
            // either; runtime treats them as one bind-time category.
            shaderKind == ResourceKind::Sampler || shaderKind == ResourceKind::ComparisonSampler
        }
        BindingKind::comparisonSampler => {
            shaderKind == ResourceKind::Sampler || shaderKind == ResourceKind::ComparisonSampler
        }
        _ => false,
    }
}

#[allow(unreachable_patterns)] // Preserve the source diagnostic fallback for future enum values.
fn kindName(k: BindingKind) -> &'static str {
    match k {
        BindingKind::uniformBuffer => "uniformBuffer",
        BindingKind::storageBufferRO => "storageBufferRO",
        BindingKind::storageBufferRW => "storageBufferRW",
        BindingKind::sampledTexture => "sampledTexture",
        BindingKind::storageTexture => "storageTexture",
        BindingKind::sampler => "sampler",
        BindingKind::comparisonSampler => "comparisonSampler",
        _ => "?",
    }
}

fn shaderKindName(k: ResourceKind) -> &'static str {
    match k {
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

pub fn validateLayoutsAgainstBindingMap(
    bindingMap: &BindingMap,
    layouts: Option<&[Option<&AnyResourceHandle>]>,
    layoutCount: u32,
    mut outError: Option<&mut String>,
) -> bool {
    let mut fail = |msg: String| {
        if let Some(error) = outError.as_mut() {
            **error = msg;
        }
        false
    };

    // Every binding the shader references must have a corresponding layout
    // entry. Unused layout entries (declared but not referenced by the shader)
    // are allowed — Dawn permits this and it lets pipelines reuse a more
    // permissive layout than the shader strictly needs.
    for i in 0..bindingMap.size() {
        let shaderEntry: &Entry = bindingMap.at(i);
        let group: u32 = shaderEntry.group.into();
        let binding: u32 = shaderEntry.binding.into();

        if group >= layoutCount
            || layouts.is_none()
            || layouts
                .and_then(|allLayouts| allLayouts.get(group as usize))
                .and_then(|layout| *layout)
                .is_none()
        {
            let msg = format!(
                "@group({group}) @binding({binding}): shader declares {} but PipelineDesc::bindGroupLayouts has no entry for group {group}",
                shaderKindName(shaderEntry.kind)
            );
            return fail(msg);
        }

        let layoutHandle = layouts
            .and_then(|allLayouts| allLayouts.get(group as usize))
            .and_then(|layout| *layout)
            .expect("layout was checked above");
        let Some(layout) = layoutHandle.downcast_ref::<BindGroupLayout>() else {
            return fail(format!(
                "PipelineDesc::bindGroupLayouts[{group}] is not a BindGroupLayout"
            ));
        };
        if layout.groupIndex() != group {
            let msg = format!(
                "PipelineDesc::bindGroupLayouts[{group}]->groupIndex == {}, expected {group} (positional index must match layout's groupIndex)",
                layout.groupIndex()
            );
            return fail(msg);
        }

        let layoutEntry: &BindGroupLayoutEntry = match layout.findEntry(binding) {
            Some(entry) => entry,
            None => {
                let msg = format!(
                    "@group({group}) @binding({binding}): layout has no entry for this binding (shader expects {})",
                    shaderKindName(shaderEntry.kind)
                );
                return fail(msg);
            }
        };

        if !kindsMatch(layoutEntry.kind, shaderEntry.kind) {
            let msg = format!(
                "@group({group}) @binding({binding}): layout declares {} but shader declares {}",
                kindName(layoutEntry.kind),
                shaderKindName(shaderEntry.kind)
            );
            return fail(msg);
        }

        // Visibility narrower than the shader's stageMask is rejected.
        // Layout broader than shader is fine (allowed by WebGPU spec).
        let shaderStageMask: u8 = shaderEntry.stageMask;
        let layoutVisibility: u8 = layoutEntry.visibility.mask;
        if (shaderStageMask & !layoutVisibility) != 0 {
            let msg = format!(
                "@group({group}) @binding({binding}): layout visibility 0x{:x} missing stages required by shader (stageMask=0x{:x})",
                layoutVisibility, shaderStageMask
            );
            return fail(msg);
        }

        // Texture dimension/sampleType compatibility (texture kinds only).
        if layoutEntry.kind == BindingKind::sampledTexture
            || layoutEntry.kind == BindingKind::storageTexture
        {
            // Map TextureViewDim (binding-map) ↔ TextureViewDimension
            // (public layout API). The binding-map enum has D1/D2/D2Array/
            // Cube/CubeArray/D3; the public enum has texture2D/cube/
            // texture3D/array2D/cubeArray.
            #[allow(unreachable_patterns)] // Source retains a defensive fallback for enum growth.
            let dimsMatch = |a: TextureViewDimension, b: TextureViewDim| match a {
                TextureViewDimension::texture2D => b == TextureViewDim::D2,
                TextureViewDimension::cube => b == TextureViewDim::Cube,
                TextureViewDimension::texture3D => b == TextureViewDim::D3,
                TextureViewDimension::array2D => b == TextureViewDim::D2Array,
                TextureViewDimension::cubeArray => b == TextureViewDim::CubeArray,
                _ => false,
            };

            // Shader's textureViewDim is Undefined for non-texture kinds
            // and may also be Undefined for textures the shader compiler
            // didn't reflect a dim for. Skip the check when shader side
            // is Undefined.
            if shaderEntry.textureViewDim != TextureViewDim::Undefined
                && !dimsMatch(layoutEntry.textureViewDim, shaderEntry.textureViewDim)
            {
                let msg =
                    format!("@group({group}) @binding({binding}): texture view dimension mismatch");
                return fail(msg);
            }
        }
    }
    true
}

/// Rust derived-class integration spelling of
/// `validateLayoutsAgainstBindingMap`.
///
/// A C++ `BindGroupLayoutVulkan*` converts implicitly to its
/// `BindGroupLayout*` base. Rust resource handles retain the concrete payload,
/// so sibling backend crates perform the downcast and pass these exact base
/// references through this seam. The validation body and diagnostics remain
/// identical to the pinned source function above.
#[doc(hidden)]
pub fn validateLayoutBasesAgainstBindingMap(
    bindingMap: &BindingMap,
    layouts: Option<&[Option<&BindGroupLayout>]>,
    layoutCount: u32,
    mut outError: Option<&mut String>,
) -> bool {
    let mut fail = |msg: String| {
        if let Some(error) = outError.as_mut() {
            **error = msg;
        }
        false
    };

    for i in 0..bindingMap.size() {
        let shaderEntry: &Entry = bindingMap.at(i);
        let group: u32 = shaderEntry.group.into();
        let binding: u32 = shaderEntry.binding.into();

        if group >= layoutCount
            || layouts.is_none()
            || layouts
                .and_then(|allLayouts| allLayouts.get(group as usize))
                .and_then(|layout| *layout)
                .is_none()
        {
            return fail(format!(
                "@group({group}) @binding({binding}): shader declares {} but PipelineDesc::bindGroupLayouts has no entry for group {group}",
                shaderKindName(shaderEntry.kind)
            ));
        }

        let layout = layouts
            .and_then(|allLayouts| allLayouts.get(group as usize))
            .and_then(|layout| *layout)
            .expect("layout was checked above");
        if layout.groupIndex() != group {
            return fail(format!(
                "PipelineDesc::bindGroupLayouts[{group}]->groupIndex == {}, expected {group} (positional index must match layout's groupIndex)",
                layout.groupIndex()
            ));
        }

        let layoutEntry: &BindGroupLayoutEntry = match layout.findEntry(binding) {
            Some(entry) => entry,
            None => {
                return fail(format!(
                    "@group({group}) @binding({binding}): layout has no entry for this binding (shader expects {})",
                    shaderKindName(shaderEntry.kind)
                ));
            }
        };

        if !kindsMatch(layoutEntry.kind, shaderEntry.kind) {
            return fail(format!(
                "@group({group}) @binding({binding}): layout declares {} but shader declares {}",
                kindName(layoutEntry.kind),
                shaderKindName(shaderEntry.kind)
            ));
        }

        let shaderStageMask: u8 = shaderEntry.stageMask;
        let layoutVisibility: u8 = layoutEntry.visibility.mask;
        if (shaderStageMask & !layoutVisibility) != 0 {
            return fail(format!(
                "@group({group}) @binding({binding}): layout visibility 0x{:x} missing stages required by shader (stageMask=0x{:x})",
                layoutVisibility, shaderStageMask
            ));
        }

        if matches!(
            layoutEntry.kind,
            BindingKind::sampledTexture | BindingKind::storageTexture
        ) {
            let dimsMatch = |a: TextureViewDimension, b: TextureViewDim| match a {
                TextureViewDimension::texture2D => b == TextureViewDim::D2,
                TextureViewDimension::cube => b == TextureViewDim::Cube,
                TextureViewDimension::texture3D => b == TextureViewDim::D3,
                TextureViewDimension::array2D => b == TextureViewDim::D2Array,
                TextureViewDimension::cubeArray => b == TextureViewDim::CubeArray,
            };
            if shaderEntry.textureViewDim != TextureViewDim::Undefined
                && !dimsMatch(layoutEntry.textureViewDim, shaderEntry.textureViewDim)
            {
                return fail(format!(
                    "@group({group}) @binding({binding}): texture view dimension mismatch"
                ));
            }
        }
    }
    true
}

pub fn validateColorRequiresFragment(
    colorCount: u32,
    hasFragmentModule: bool,
    mut outError: Option<&mut String>,
) -> bool {
    if colorCount > 0 && !hasFragmentModule {
        if let Some(error) = outError.as_mut() {
            **error = "pipeline declares color outputs but has no fragment shader; supply `fragment`, or omit `colorTargets` for a depth-only pipeline".to_owned();
        }
        return false;
    }
    true
}

// } // namespace rive::ore
