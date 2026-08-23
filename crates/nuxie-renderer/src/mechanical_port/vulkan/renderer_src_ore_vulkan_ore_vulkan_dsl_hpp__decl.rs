/*
 * Copyright 2026 Rive
 */

//! Complete mechanical translation of
//! `renderer/src/ore/vulkan/ore_vulkan_dsl.hpp` at the pinned upstream ref.

#![allow(non_snake_case, non_upper_case_globals)]

use ash::vk;
use nuxie_ore_metal::types::{
    BindGroupLayoutDesc, BindGroupLayoutEntry, BindingKind, StageVisibility, kMaxBindGroups,
};

pub(crate) const kVkMaxGroups: u32 = kMaxBindGroups;
pub(crate) const kVkMaxBindingsPerGroup: u32 = 16;

pub(crate) fn oreBindingKindToVk(kind: BindingKind, dynamic: bool) -> vk::DescriptorType {
    match kind {
        BindingKind::uniformBuffer if dynamic => vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC,
        BindingKind::uniformBuffer => vk::DescriptorType::UNIFORM_BUFFER,
        BindingKind::storageBufferRO | BindingKind::storageBufferRW => {
            vk::DescriptorType::STORAGE_BUFFER
        }
        BindingKind::sampledTexture => vk::DescriptorType::SAMPLED_IMAGE,
        BindingKind::storageTexture => vk::DescriptorType::STORAGE_IMAGE,
        BindingKind::sampler | BindingKind::comparisonSampler => vk::DescriptorType::SAMPLER,
    }
}

pub(crate) fn oreVisibilityToVk(visibility: StageVisibility) -> vk::ShaderStageFlags {
    let mut flags = vk::ShaderStageFlags::empty();
    if visibility.mask & StageVisibility::kVertex != 0 {
        flags |= vk::ShaderStageFlags::VERTEX;
    }
    if visibility.mask & StageVisibility::kFragment != 0 {
        flags |= vk::ShaderStageFlags::FRAGMENT;
    }
    if visibility.mask & StageVisibility::kCompute != 0 {
        flags |= vk::ShaderStageFlags::COMPUTE;
    }
    flags
}

/// Builds the descriptor-set layout exactly once from the public ORE layout.
///
/// # Safety
/// `pfnCreateDSL` and `device` must be the live function/device pair supplied
/// by the owning Vulkan context, matching the source function-pointer contract.
pub(crate) unsafe fn createDSLFromLayoutDesc(
    pfnCreateDSL: vk::PFN_vkCreateDescriptorSetLayout,
    device: vk::Device,
    desc: &BindGroupLayoutDesc<'_>,
) -> vk::DescriptorSetLayout {
    let mut bindings = [vk::DescriptorSetLayoutBinding::default();
        kVkMaxBindingsPerGroup as usize];
    let count = desc.entries.len().min(kVkMaxBindingsPerGroup as usize);

    for (binding, entry) in bindings.iter_mut().zip(desc.entries.iter()).take(count) {
        let dynamic = entry.kind == BindingKind::uniformBuffer && entry.hasDynamicOffset;
        let native_binding = if entry.nativeSlotVS != BindGroupLayoutEntry::kNativeSlotAbsent {
            entry.nativeSlotVS
        } else if entry.nativeSlotFS != BindGroupLayoutEntry::kNativeSlotAbsent {
            entry.nativeSlotFS
        } else {
            entry.binding
        };
        *binding = vk::DescriptorSetLayoutBinding::default()
            .binding(native_binding)
            .descriptor_type(oreBindingKindToVk(entry.kind, dynamic))
            .descriptor_count(1)
            .stage_flags(oreVisibilityToVk(entry.visibility));
    }

    let create_info = vk::DescriptorSetLayoutCreateInfo::default().bindings(&bindings[..count]);
    let mut descriptor_set_layout = vk::DescriptorSetLayout::null();
    let _ = unsafe {
        pfnCreateDSL(
            device,
            &create_info,
            core::ptr::null(),
            &mut descriptor_set_layout,
        )
    };
    descriptor_set_layout
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn binding_kinds_and_visibility_preserve_source_mapping() {
        assert_eq!(
            oreBindingKindToVk(BindingKind::uniformBuffer, true),
            vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC
        );
        assert_eq!(
            oreBindingKindToVk(BindingKind::comparisonSampler, false),
            vk::DescriptorType::SAMPLER
        );
        assert_eq!(
            oreVisibilityToVk(StageVisibility {
                mask: StageVisibility::kVertex
                    | StageVisibility::kFragment
                    | StageVisibility::kCompute,
            }),
            vk::ShaderStageFlags::VERTEX
                | vk::ShaderStageFlags::FRAGMENT
                | vk::ShaderStageFlags::COMPUTE
        );
    }
}
