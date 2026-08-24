//! Complete mechanical declaration translation of
//! `renderer/src/ore/vulkan/ore_texture_vulkan.hpp`.

#![allow(non_snake_case)]

use super::ore_context_vulkan_decl::{ContextVulkan, ContextVulkanLifetime};
use super::render_target_vulkan_decl::RetainedRenderTargetVulkan;
use super::vulkan_context_decl::VulkanContext;
use ash::vk;
use nuxie_ore_metal::gpu_resource::{
    AnyResourceHandle, GPUResource, GPUResourceManager, GpuResourcePayload,
};
use nuxie_ore_metal::texture::{Texture, TextureApi, TextureUploadError, TextureView};
use nuxie_ore_metal::types::{
    TextureDataDesc, TextureDesc, TextureFormat, TextureType, TextureViewDesc,
};
use std::cell::Cell;
use std::mem::ManuallyDrop;
use std::ops::{Deref, DerefMut};
use std::rc::{Rc, Weak};
use std::sync::Arc;

#[repr(C)]
pub(crate) struct TextureVulkan {
    pub(crate) base: ManuallyDrop<Texture>,
    pub(crate) m_vkImage: vk::Image,
    pub(crate) m_vmaAllocation: Option<Box<vk_mem::Allocation>>,
    pub(crate) m_vkLayout: Cell<vk::ImageLayout>,
    pub(crate) m_vkDevice: vk::Device,
    pub(crate) m_vk: ManuallyDrop<Option<Arc<VulkanContext>>>,
    pub(crate) m_vkOreContext: Cell<*mut ContextVulkan>,
    pub(super) m_contextLifetime: Weak<ContextVulkanLifetime>,
}

impl TextureVulkan {
    pub(crate) fn new(
        manager: GPUResourceManager,
        desc: &TextureDesc<'_>,
        context: &mut ContextVulkan,
    ) -> Self {
        Self {
            base: ManuallyDrop::new(nuxie_ore_metal::new_texture_backend_base(manager, desc)),
            m_vkImage: vk::Image::null(),
            m_vmaAllocation: None,
            m_vkLayout: Cell::new(vk::ImageLayout::UNDEFINED),
            m_vkDevice: context.m_vk.device,
            m_vk: ManuallyDrop::new(Some(Arc::clone(&context.m_vk))),
            m_vkOreContext: Cell::new(context),
            m_contextLifetime: Rc::downgrade(&context.m_lifetime),
        }
    }

    pub(super) fn oreContextMut(&self) -> &mut ContextVulkan {
        assert!(
            self.m_contextLifetime
                .upgrade()
                .is_some_and(|lifetime| lifetime.isLive()),
            "TextureVulkan cannot access a retired ContextVulkan"
        );
        let context = self.m_vkOreContext.get();
        assert!(
            !context.is_null(),
            "TextureVulkan requires its source ContextVulkan back-pointer"
        );
        unsafe { &mut *context }
    }
}

impl Deref for TextureVulkan {
    type Target = Texture;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl DerefMut for TextureVulkan {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
unsafe impl GpuResourcePayload for TextureVulkan {
    fn gpu_resource(&self) -> &GPUResource {
        self.base.gpu_resource()
    }
    fn gpu_resource_mut(&mut self) -> &mut GPUResource {
        self.base.gpu_resource_mut()
    }
}
impl TextureApi for TextureVulkan {
    fn width(&self) -> u32 {
        self.base.width()
    }
    fn height(&self) -> u32 {
        self.base.height()
    }
    fn depthOrArrayLayers(&self) -> u32 {
        self.base.depthOrArrayLayers()
    }
    fn format(&self) -> TextureFormat {
        self.base.format()
    }
    fn r#type(&self) -> TextureType {
        self.base.r#type()
    }
    fn numMipmaps(&self) -> u32 {
        self.base.numMipmaps()
    }
    fn sampleCount(&self) -> u32 {
        self.base.sampleCount()
    }
    fn isRenderTarget(&self) -> bool {
        self.base.isRenderTarget()
    }
    fn upload(&self, data: &TextureDataDesc<'_>) -> Result<(), TextureUploadError> {
        super::ore_texture_vulkan_impl::upload(self, data, None)
    }
    fn uploadWithOwner(
        &self,
        data: &TextureDataDesc<'_>,
        owner: AnyResourceHandle,
    ) -> Result<(), TextureUploadError> {
        super::ore_texture_vulkan_impl::upload(self, data, Some(owner))
    }
}

#[repr(C)]
pub(crate) struct TextureViewVulkan {
    pub(crate) base: ManuallyDrop<TextureView>,
    pub(crate) m_vkDevice: vk::Device,
    pub(crate) m_vkImageView: vk::ImageView,
    pub(crate) m_vkDestroyImageView: Option<vk::PFN_vkDestroyImageView>,
    pub(crate) m_vkRenderTarget: Option<RetainedRenderTargetVulkan>,
}

impl TextureViewVulkan {
    pub(crate) fn new(
        manager: GPUResourceManager,
        texture: AnyResourceHandle,
        desc: &TextureViewDesc<'_>,
    ) -> Self {
        Self {
            base: ManuallyDrop::new(nuxie_ore_metal::new_texture_view_backend_base(
                manager, texture, desc,
            )),
            m_vkDevice: vk::Device::null(),
            m_vkImageView: vk::ImageView::null(),
            m_vkDestroyImageView: None,
            m_vkRenderTarget: None,
        }
    }
}

impl Deref for TextureViewVulkan {
    type Target = TextureView;
    fn deref(&self) -> &Self::Target {
        &self.base
    }
}
impl DerefMut for TextureViewVulkan {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.base
    }
}
unsafe impl GpuResourcePayload for TextureViewVulkan {
    fn gpu_resource(&self) -> &GPUResource {
        self.base.gpu_resource()
    }
    fn gpu_resource_mut(&mut self) -> &mut GPUResource {
        self.base.gpu_resource_mut()
    }
}
