//! Complete mechanical declaration translation of
//! `renderer/src/ore/vulkan/ore_render_pass_vulkan.hpp`.

#![allow(non_snake_case)]

use super::ore_context_vulkan_decl::ContextVulkan;
use super::ore_pipeline_vulkan_decl::PipelineVulkan;
use super::render_target_vulkan_decl::RenderTargetVulkanApi;
use super::vkutil_decl::Framebuffer;
use ash::vk;
use nuxie_ore_metal::context::ActiveRenderPass;
use nuxie_ore_metal::gpu_resource::{AnyResourceHandle, ResourceHandle};
use nuxie_ore_metal::render_pass::RenderPass;
use std::cell::{RefCell, RefMut};
use std::mem::ManuallyDrop;
use std::ptr::NonNull;
use std::rc::{Rc, Weak as RcWeak};

pub(crate) struct ResolveTarget {
    pub(crate) image: vk::Image,
    pub(crate) baseMip: u32,
    pub(crate) baseLayer: u32,
    pub(crate) layerCount: u32,
    pub(crate) renderTarget: Option<NonNull<dyn RenderTargetVulkanApi>>,
    pub(crate) texture: Option<AnyResourceHandle>,
}

impl Default for ResolveTarget {
    fn default() -> Self {
        Self {
            image: vk::Image::null(),
            baseMip: 0,
            baseLayer: 0,
            layerCount: 1,
            renderTarget: None,
            texture: None,
        }
    }
}

#[repr(C)]
pub(crate) struct RenderPassVulkanState {
    pub(crate) base: ManuallyDrop<RenderPass>,
    pub(crate) m_vkContext: *mut ContextVulkan,
    pub(crate) m_currentPipeline: ManuallyDrop<Option<ResourceHandle<PipelineVulkan>>>,
    pub(crate) m_vkCmdBuf: vk::CommandBuffer,
    pub(crate) m_framebuffer: ManuallyDrop<Option<ResourceHandle<Framebuffer>>>,
    pub(crate) m_vkIndexBuffer: vk::Buffer,
    pub(crate) m_vkIndexType: vk::IndexType,
    pub(crate) m_vkIndexOffset: u32,
    pub(crate) m_vkColorImages: [vk::Image; 4],
    pub(crate) m_vkColorBaseLayer: [u32; 4],
    pub(crate) m_vkColorLayerCount: [u32; 4],
    pub(crate) m_vkColorCount: u32,
    pub(crate) m_vkColorRenderTargets: [Option<NonNull<dyn RenderTargetVulkanApi>>; 4],
    pub(crate) m_vkColorTextures: ManuallyDrop<[Option<AnyResourceHandle>; 4]>,
    pub(crate) m_vkResolveTargets: ManuallyDrop<[ResolveTarget; 4]>,
    pub(crate) m_vkDepthImage: vk::Image,
    pub(crate) m_vkDepthBaseLayer: u32,
    pub(crate) m_vkDepthLayerCount: u32,
    pub(crate) m_vkDepthTexture: ManuallyDrop<Option<AnyResourceHandle>>,
    pub(crate) m_vkStencilRef: u32,
}

impl RenderPassVulkanState {
    pub(crate) fn new(context: &mut ContextVulkan) -> Self {
        Self {
            base: ManuallyDrop::new(nuxie_ore_metal::new_render_pass_backend_base(&context.base)),
            m_vkContext: context,
            m_currentPipeline: ManuallyDrop::new(None),
            m_vkCmdBuf: vk::CommandBuffer::null(),
            m_framebuffer: ManuallyDrop::new(None),
            m_vkIndexBuffer: vk::Buffer::null(),
            m_vkIndexType: vk::IndexType::UINT16,
            m_vkIndexOffset: 0,
            m_vkColorImages: [vk::Image::null(); 4],
            m_vkColorBaseLayer: [0; 4],
            m_vkColorLayerCount: [0; 4],
            m_vkColorCount: 0,
            m_vkColorRenderTargets: [None; 4],
            m_vkColorTextures: ManuallyDrop::new(std::array::from_fn(|_| None)),
            m_vkResolveTargets: ManuallyDrop::new(std::array::from_fn(|_| {
                ResolveTarget::default()
            })),
            m_vkDepthImage: vk::Image::null(),
            m_vkDepthBaseLayer: 0,
            m_vkDepthLayerCount: 1,
            m_vkDepthTexture: ManuallyDrop::new(None),
            m_vkStencilRef: 0,
        }
    }

    pub(super) fn context(&self) -> &ContextVulkan {
        assert!(!self.m_vkContext.is_null());
        unsafe { &*self.m_vkContext }
    }

    pub(super) fn contextMut(&mut self) -> &mut ContextVulkan {
        assert!(!self.m_vkContext.is_null());
        unsafe { &mut *self.m_vkContext }
    }
}

impl std::ops::Deref for RenderPassVulkanState {
    type Target = RenderPass;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}

impl std::ops::DerefMut for RenderPassVulkanState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}

pub(crate) struct RenderPassVulkanInner {
    pub(crate) state: RefCell<RenderPassVulkanState>,
}

impl RenderPassVulkanInner {
    pub(crate) fn borrowState(&self) -> RefMut<'_, RenderPassVulkanState> {
        self.state.borrow_mut()
    }
}

impl ActiveRenderPass for RenderPassVulkanInner {
    fn isFinished(&self) -> bool {
        self.state.borrow().base.isFinished()
    }

    fn finish(&self) {
        super::ore_render_pass_vulkan_impl::finish(&mut self.borrowState());
    }
}

/// Unique public owner plus the `Rc` required to represent Context's weak
/// active-pass pointer safely. The inner state is the exact source object.
pub(crate) struct RenderPassVulkan {
    pub(crate) inner: Rc<RenderPassVulkanInner>,
}

impl RenderPassVulkan {
    pub(crate) fn new(context: &mut ContextVulkan) -> Self {
        Self {
            inner: Rc::new(RenderPassVulkanInner {
                state: RefCell::new(RenderPassVulkanState::new(context)),
            }),
        }
    }

    pub(crate) fn borrowState(&self) -> RefMut<'_, RenderPassVulkanState> {
        self.inner.borrowState()
    }

    pub(crate) fn activeToken(&self) -> RcWeak<dyn ActiveRenderPass> {
        let token: Rc<dyn ActiveRenderPass> = self.inner.clone();
        Rc::downgrade(&token)
    }
}
