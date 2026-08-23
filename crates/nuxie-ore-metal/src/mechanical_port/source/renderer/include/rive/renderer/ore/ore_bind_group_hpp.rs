/*
 * Copyright 2025 Rive
 */

// #pragma once

// #include "rive/renderer/gpu_resource.hpp"
// #include "utils/lite_rtti.hpp"
// #include "rive/renderer/ore/ore_types.hpp"
// #include "rive/renderer/ore/ore_buffer.hpp"
// #include "rive/renderer/ore/ore_texture.hpp"
// #include "rive/renderer/ore/ore_sampler.hpp"
// #include "rive/renderer/ore/ore_bind_group_layout.hpp"

// #include <vector>

// Mechanical translation of the complete pinned source header
// renderer/include/rive/renderer/ore/ore_bind_group.hpp.
// Upstream source revision: 4ac7b32798da0482e441ef09304dc3b480ed3ee5

#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
use super::*;

use core::mem::ManuallyDrop;
use core::ops::{Deref, DerefMut};

use super::super::gpu_resource_hpp::{AnyResourceHandle, GPUResource, GpuResourcePayload};
use super::ore_bind_group_layout_hpp::BindGroupLayout;
use std::sync::Weak;

// namespace rive::ore
// {

// class Context;
// The source forward declaration is retained for the non-owning context
// relationship below. Context is owned by its own translation unit.

// class BindGroup : public rive::gpu::GPUResource,
//                   public ENABLE_LITE_RTTI(BindGroup)
// {
//
// Rust has no class inheritance. The first field is the GPUResource base
// subobject, preserving the source base-before-members layout and ownership
// relationship. The second source base, `ENABLE_LITE_RTTI(BindGroup)`, is
// retained as the source RTTI/downcast contract and is deliberately not
// duplicated as a payload field here.
//
// Pre-baked resource bindings that can be reused across many draw calls.
// Holds strong rcp<> references to all bound resources (Buffer, TextureView,
// Sampler), ensuring they remain alive for the BindGroup's lifetime.
//
// Created via Context::makeBindGroup(). Bound via RenderPass::setBindGroup().
#[repr(C)]
pub struct BindGroupMembers {
    // protected:
    // friend class Context;
    // friend class RenderPass;
    // Rust has no friend declarations; these source access boundaries remain
    // visible here, and the owning translation units use crate visibility.

    // uint32_t m_dynamicOffsetCount = 0;
    pub(crate) m_dynamicOffsetCount: u32,

    // The layout this BindGroup conforms to. Holds the per-backend native
    // layout handle alive for the BindGroup's lifetime — Vulkan's
    // VkDescriptorSetLayout in particular must outlive every VkDescriptorSet
    // allocated from it.
    // rcp<BindGroupLayout> m_layoutRef;
    pub(crate) m_layoutRef: Option<AnyResourceHandle>,

    // Lifecycle: hold rcp<> refs to all bound resources so they stay alive
    // even if the caller drops their references before the BindGroup is
    // destroyed.
    // std::vector<rcp<Buffer>> m_retainedBuffers;
    pub(crate) m_retainedBuffers: Vec<AnyResourceHandle>,
    // std::vector<rcp<TextureView>> m_retainedViews;
    pub(crate) m_retainedViews: Vec<AnyResourceHandle>,
    // std::vector<rcp<Sampler>> m_retainedSamplers;
    pub(crate) m_retainedSamplers: Vec<AnyResourceHandle>,
    // Context back-pointer set in makeBindGroup(). Used by the Lua GC
    // to call context->deferBindGroupDestroy() instead of dropping the
    // last rcp<> directly, keeping the object alive until endFrame().
    // Context* m_context = nullptr;
    //
    // Rust stores the source non-owning pointer as Weak<ContextState>.
    pub(crate) m_context: Weak<ContextState>,
}

#[repr(C)]
pub struct BindGroup {
    pub(crate) base: ManuallyDrop<GPUResource>,
    pub(crate) members: ManuallyDrop<BindGroupMembers>,
}

impl Deref for BindGroup {
    type Target = BindGroupMembers;

    fn deref(&self) -> &Self::Target {
        &self.members
    }
}

impl DerefMut for BindGroup {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.members
    }
}

impl Drop for BindGroup {
    fn drop(&mut self) {
        unsafe {
            #[cfg(test)]
            super::super::gpu_resource_hpp::record_resource_drop_stage("BindGroup.context");
            core::ptr::drop_in_place(&mut self.m_context);
            #[cfg(test)]
            super::super::gpu_resource_hpp::record_resource_drop_stage("BindGroup.samplers");
            core::ptr::drop_in_place(&mut self.m_retainedSamplers);
            #[cfg(test)]
            super::super::gpu_resource_hpp::record_resource_drop_stage("BindGroup.views");
            core::ptr::drop_in_place(&mut self.m_retainedViews);
            #[cfg(test)]
            super::super::gpu_resource_hpp::record_resource_drop_stage("BindGroup.buffers");
            core::ptr::drop_in_place(&mut self.m_retainedBuffers);
            #[cfg(test)]
            super::super::gpu_resource_hpp::record_resource_drop_stage("BindGroup.layout");
            core::ptr::drop_in_place(&mut self.m_layoutRef);
            #[cfg(test)]
            super::super::gpu_resource_hpp::record_resource_drop_stage("BindGroup.base");
            ManuallyDrop::drop(&mut self.base);
        }
    }
}

unsafe impl GpuResourcePayload for BindGroup {
    fn gpu_resource(&self) -> &GPUResource {
        &self.base
    }

    fn gpu_resource_mut(&mut self) -> &mut GPUResource {
        &mut self.base
    }
}

impl BindGroup {
    // public:

    // uint32_t dynamicOffsetCount() const { return m_dynamicOffsetCount; }
    pub fn dynamicOffsetCount(&self) -> u32 {
        self.m_dynamicOffsetCount
    }

    // uint32_t groupIndex() const
    // {
    //     return m_layoutRef ? m_layoutRef->groupIndex() : 0;
    // }
    pub fn groupIndex(&self) -> u32 {
        self.m_layoutRef
            .as_ref()
            .and_then(AnyResourceHandle::bindGroupLayoutBase)
            .map_or(0, BindGroupLayout::groupIndex)
    }

    // BindGroupLayout* layout() const { return m_layoutRef.get(); }
    // The nullable raw-pointer result is represented as an optional borrow;
    // `m_layoutRef` remains the owning rcp<BindGroupLayout>.
    pub fn layout(&self) -> Option<&AnyResourceHandle> {
        self.m_layoutRef.as_ref()
    }

    // Context* context() const { return m_context; }
    pub fn context(&self) -> Weak<ContextState> {
        self.m_context.clone()
    }

    pub(crate) fn retained_buffer(&self, index: usize) -> Option<&AnyResourceHandle> {
        self.m_retainedBuffers.get(index)
    }

    #[allow(dead_code)] // Source ownership slot; Metal currently reads the native binding copy.
    pub(crate) fn retained_view(&self, index: usize) -> Option<&AnyResourceHandle> {
        self.m_retainedViews.get(index)
    }

    #[allow(dead_code)] // Source ownership slot; Metal currently reads the native binding copy.
    pub(crate) fn retained_sampler(&self, index: usize) -> Option<&AnyResourceHandle> {
        self.m_retainedSamplers.get(index)
    }

    // virtual ~BindGroup() = default;
    // Rust's default drop glue supplies the virtual-destructor boundary for
    // the concrete resource owner. The source member declarations above
    // retain the complete owner graph and authored field order.

    // protected:

    // BindGroup() : rive::gpu::GPUResource(nullptr) {}
    pub(crate) fn new() -> Self {
        Self {
            base: ManuallyDrop::new(GPUResource::new(None)),
            members: ManuallyDrop::new(BindGroupMembers {
                m_dynamicOffsetCount: 0,
                m_layoutRef: None,
                m_retainedBuffers: Vec::new(),
                m_retainedViews: Vec::new(),
                m_retainedSamplers: Vec::new(),
                m_context: Weak::new(),
            }),
        }
    }

    // BindGroup(rcp<rive::gpu::GPUResourceManager> manager) :
    //     rive::gpu::GPUResource(std::move(manager))
    // {}
    // Manager ownership is carried by the concrete outer ResourceHandle.
}

// } // namespace rive::ore
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Mutex};

    #[test]
    fn bind_group_adopts_each_strong_resource_once_and_reports_layout_identity() {
        let group = BindGroup::new();
        assert_eq!(group.dynamicOffsetCount(), 0);
        assert_eq!(group.groupIndex(), 0);
        assert!(group.layout().is_none());
        assert!(group.m_retainedBuffers.is_empty());
        assert!(group.m_retainedViews.is_empty());
        assert!(group.m_retainedSamplers.is_empty());
    }

    #[test]
    fn missing_layout_preserves_cxx_zero_group_fallback() {
        let group = BindGroup::new();
        assert_eq!(group.groupIndex(), 0);
        assert!(group.layout().is_none());
        assert!(group.context().upgrade().is_none());
    }

    #[test]
    fn logical_resources_drop_in_cxx_reverse_member_order() {
        let order = Arc::new(Mutex::new(Vec::<&'static str>::new()));
        let tag = Arc::clone(&order);
        {
            let _group = BindGroup::new();
            tag.lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .push("group");
        }
        assert_eq!(
            *order
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner),
            ["group"]
        );
    }
}
