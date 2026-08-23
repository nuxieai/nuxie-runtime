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

// namespace rive::ore

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
