//! Complete mechanical declaration translation of
//! `renderer/include/rive/renderer/vulkan/vkutil.hpp`.

#![allow(non_camel_case_types, non_snake_case, non_upper_case_globals)]

use super::vulkan_context_decl::VulkanContext;
use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::{
    CullFace, DrawType, IAABB, StencilCompareOp, StencilOp, DEPTH_MAX, DEPTH_MIN,
};
use crate::mechanical_port::source::renderer::include::rive::renderer::texture_hpp::Texture;
use crate::mechanical_port::source::include::rive::refcnt_hpp::RefCntTarget;
use ash::vk;
use nuxie_ore_metal::gpu_resource::{
    GPUResource, GPUResourcePool, GpuResourcePayload, ResourceHandle,
};
use nuxie_render_api::{ColorInt, ImageSampler};
use std::cell::UnsafeCell;
use std::mem::ManuallyDrop;
use std::ops::Deref;
use std::sync::Arc;

pub(crate) const AMD: u32 = 0x1002;
pub(crate) const Imagination: u32 = 0x1010;
pub(crate) const NVIDIA: u32 = 0x10DE;
pub(crate) const ARM: u32 = 0x13B5;
pub(crate) const Qualcomm: u32 = 0x5143;
pub(crate) const Intel: u32 = 0x8086;
pub(crate) const Samsung: u32 = 0x144d;

pub(crate) const kColorWriteMaskNone: vk::ColorComponentFlags = vk::ColorComponentFlags::empty();
pub(crate) const kColorWriteMaskRGBA: vk::ColorComponentFlags =
    vk::ColorComponentFlags::from_raw(
        vk::ColorComponentFlags::R.as_raw()
            | vk::ColorComponentFlags::G.as_raw()
            | vk::ColorComponentFlags::B.as_raw()
            | vk::ColorComponentFlags::A.as_raw(),
    );

pub(crate) fn vkStencilOp(op: StencilOp) -> vk::StencilOp {
    match op {
        StencilOp::keep => vk::StencilOp::KEEP,
        StencilOp::replace => vk::StencilOp::REPLACE,
        StencilOp::zero => vk::StencilOp::ZERO,
        StencilOp::decrClamp => vk::StencilOp::DECREMENT_AND_CLAMP,
        StencilOp::incrWrap => vk::StencilOp::INCREMENT_AND_WRAP,
        StencilOp::decrWrap => vk::StencilOp::DECREMENT_AND_WRAP,
    }
}

pub(crate) fn vkCompareOp(op: StencilCompareOp) -> vk::CompareOp {
    match op {
        StencilCompareOp::less => vk::CompareOp::LESS,
        StencilCompareOp::equal => vk::CompareOp::EQUAL,
        StencilCompareOp::lessOrEqual => vk::CompareOp::LESS_OR_EQUAL,
        StencilCompareOp::notEqual => vk::CompareOp::NOT_EQUAL,
        StencilCompareOp::always => vk::CompareOp::ALWAYS,
    }
}

pub(crate) fn vkCullMode(face: CullFace) -> vk::CullModeFlags {
    match face {
        CullFace::none => vk::CullModeFlags::NONE,
        CullFace::clockwise => vk::CullModeFlags::FRONT,
        CullFace::counterclockwise => vk::CullModeFlags::BACK,
    }
}

pub(crate) fn hasPipelineDynamicState(draw_type: DrawType) -> bool {
    draw_type == DrawType::msaaDynamicMidpointFans
}

#[repr(i32)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Mappability { none, writeOnly, readWrite }

#[repr(C)]
pub(crate) struct Resource {
    base: ManuallyDrop<GPUResource>,
    pub(crate) m_vk: ManuallyDrop<Arc<VulkanContext>>,
}

impl Resource {
    pub(crate) fn new(vk: Arc<VulkanContext>) -> Self {
        Self {
            base: ManuallyDrop::new(nuxie_ore_metal::new_gpu_resource_backend_base()),
            m_vk: ManuallyDrop::new(vk),
        }
    }
    pub(crate) fn vk(&self) -> &VulkanContext { &self.m_vk }
}

unsafe impl GpuResourcePayload for Resource {
    fn gpu_resource(&self) -> &GPUResource { &self.base }
    fn gpu_resource_mut(&mut self) -> &mut GPUResource { &mut self.base }
}

impl Drop for Resource {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.base);
            ManuallyDrop::drop(&mut self.m_vk);
        }
    }
}

#[repr(C)]
pub(crate) struct Buffer {
    pub(crate) base: ManuallyDrop<Resource>,
    pub(crate) m_mappability: Mappability,
    pub(crate) m_info: UnsafeCell<vk::BufferCreateInfo<'static>>,
    pub(crate) m_vmaAllocation: UnsafeCell<Option<vk_mem::Allocation>>,
    pub(crate) m_vkBuffer: UnsafeCell<vk::Buffer>,
    pub(crate) m_contents: UnsafeCell<*mut u8>,
}

unsafe impl Send for Buffer {}
unsafe impl GpuResourcePayload for Buffer {
    fn gpu_resource(&self) -> &GPUResource { self.base.gpu_resource() }
    fn gpu_resource_mut(&mut self) -> &mut GPUResource { self.base.gpu_resource_mut() }
}
impl Deref for Buffer { type Target = Resource; fn deref(&self) -> &Resource { &self.base } }

#[repr(C)]
pub(crate) struct BufferPool {
    pub(crate) base: ManuallyDrop<GPUResourcePool>,
    pub(crate) m_vk: ManuallyDrop<Arc<VulkanContext>>,
    pub(crate) m_usageFlags: vk::BufferUsageFlags,
    pub(crate) m_targetSize: UnsafeCell<vk::DeviceSize>,
}
unsafe impl Send for BufferPool {}

unsafe impl GpuResourcePayload for BufferPool {
    fn gpu_resource(&self) -> &GPUResource { self.base.gpu_resource() }
    fn gpu_resource_mut(&mut self) -> &mut GPUResource { self.base.gpu_resource_mut() }
}

impl BufferPool {
    pub(crate) const MAX_POOL_SIZE: usize = 8;
}

#[repr(C)]
pub(crate) struct Image {
    pub(crate) base: ManuallyDrop<Resource>,
    pub(crate) m_info: vk::ImageCreateInfo<'static>,
    pub(crate) m_vmaAllocation: UnsafeCell<Option<vk_mem::Allocation>>,
    pub(crate) m_vkImage: vk::Image,
}
unsafe impl Send for Image {}
unsafe impl GpuResourcePayload for Image {
    fn gpu_resource(&self) -> &GPUResource { self.base.gpu_resource() }
    fn gpu_resource_mut(&mut self) -> &mut GPUResource { self.base.gpu_resource_mut() }
}
impl Deref for Image { type Target = Resource; fn deref(&self) -> &Resource { &self.base } }

#[repr(C)]
pub(crate) struct ImageView {
    pub(crate) base: ManuallyDrop<Resource>,
    pub(crate) m_textureRefOrNull: ManuallyDrop<Option<ResourceHandle<Image>>>,
    pub(crate) m_info: vk::ImageViewCreateInfo<'static>,
    pub(crate) m_vkImageView: vk::ImageView,
}
unsafe impl Send for ImageView {}
unsafe impl GpuResourcePayload for ImageView {
    fn gpu_resource(&self) -> &GPUResource { self.base.gpu_resource() }
    fn gpu_resource_mut(&mut self) -> &mut GPUResource { self.base.gpu_resource_mut() }
}
impl Deref for ImageView { type Target = Resource; fn deref(&self) -> &Resource { &self.base } }

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct ImageAccess {
    pub(crate) pipelineStages: vk::PipelineStageFlags,
    pub(crate) accessMask: vk::AccessFlags,
    pub(crate) layout: vk::ImageLayout,
}
impl Default for ImageAccess {
    fn default() -> Self {
        Self { pipelineStages: vk::PipelineStageFlags::TOP_OF_PIPE,
            accessMask: vk::AccessFlags::NONE, layout: vk::ImageLayout::UNDEFINED }
    }
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ImageAccessAction { preserveContents, invalidateContents }

#[repr(C)]
pub(crate) struct Texture2D {
    pub(crate) base: ManuallyDrop<Texture>,
    pub(crate) m_image: ManuallyDrop<ResourceHandle<Image>>,
    pub(crate) m_imageView: ManuallyDrop<ResourceHandle<ImageView>>,
    pub(crate) m_lastAccess: UnsafeCell<ImageAccess>,
    pub(crate) m_imageUploadBuffer: ManuallyDrop<UnsafeCell<Option<ResourceHandle<Buffer>>>>,
    pub(crate) m_imageUploadRegions: ManuallyDrop<UnsafeCell<Vec<vk::BufferImageCopy>>>,
    pub(crate) m_cachedDescriptorSet: UnsafeCell<vk::DescriptorSet>,
    pub(crate) m_cachedDescriptorSetFrameNumber: UnsafeCell<u64>,
    pub(crate) m_cachedDescriptorSetSampler: UnsafeCell<ImageSampler>,
}
unsafe impl Send for Texture2D {}

unsafe impl RefCntTarget for Texture2D {
    fn r#ref(&self) { self.base.r#ref(); }
    unsafe fn unref(&self) { unsafe { self.base.unref() }; }
    unsafe fn onRefCntReachedZero(ptr: *const Self) {
        let base = ptr.cast::<Texture>().cast_mut();
        unsafe { ((*base).destroy_complete)(base) };
    }
}

#[repr(C)]
pub(crate) struct Framebuffer {
    pub(crate) base: ManuallyDrop<Resource>,
    pub(crate) m_info: vk::FramebufferCreateInfo<'static>,
    pub(crate) m_vkFramebuffer: vk::Framebuffer,
}
unsafe impl Send for Framebuffer {}
unsafe impl GpuResourcePayload for Framebuffer {
    fn gpu_resource(&self) -> &GPUResource { self.base.gpu_resource() }
    fn gpu_resource_mut(&mut self) -> &mut GPUResource { self.base.gpu_resource_mut() }
}
impl Deref for Framebuffer { type Target = Resource; fn deref(&self) -> &Resource { &self.base } }

pub(crate) struct ViewportFromRect2D { m_viewport: vk::Viewport }
impl ViewportFromRect2D {
    pub(crate) fn new(rect: vk::Rect2D) -> Self {
        Self { m_viewport: vk::Viewport { x: rect.offset.x as f32, y: rect.offset.y as f32,
            width: rect.extent.width as f32, height: rect.extent.height as f32,
            min_depth: DEPTH_MIN, max_depth: DEPTH_MAX } }
    }
    pub(crate) fn as_ptr(&self) -> *const vk::Viewport { &self.m_viewport }
}

pub(crate) fn set_shader_code<'a>(info: &mut vk::ShaderModuleCreateInfo<'a>, code: &'a [u32]) {
    unsafe { set_shader_code_raw(info, code.as_ptr(), core::mem::size_of_val(code)) };
}
/// # Safety
/// `code` must remain valid for `code_size` bytes through Vulkan consumption.
pub(crate) unsafe fn set_shader_code_raw<'a>(info: &mut vk::ShaderModuleCreateInfo<'a>, code: *const u32, code_size: usize) {
    info.code_size = code_size;
    info.p_code = code;
}
pub(crate) fn set_shader_code_if_then_else<'a>(info: &mut vk::ShaderModuleCreateInfo<'a>, choose_if: bool, code_if: &'a [u32], code_else: &'a [u32]) {
    set_shader_code(info, if choose_if { code_if } else { code_else });
}
/// # Safety
/// The selected pointer must remain valid for its selected byte count through
/// Vulkan consumption.
pub(crate) unsafe fn set_shader_code_if_then_else_raw<'a>(info: &mut vk::ShaderModuleCreateInfo<'a>, choose_if: bool,
    code_if: *const u32, code_size_if: usize, code_else: *const u32, code_size_else: usize) {
    if choose_if {
        unsafe { set_shader_code_raw(info, code_if, code_size_if) };
    } else {
        unsafe { set_shader_code_raw(info, code_else, code_size_else) };
    }
}
pub(crate) fn color_clear_rgba32f(color: ColorInt) -> vk::ClearColorValue {
    let [a, r, g, b] = color.to_be_bytes();
    let alpha = f32::from(a) / 255.0;
    vk::ClearColorValue { float32: [f32::from(r) / 255.0 * alpha,
        f32::from(g) / 255.0 * alpha, f32::from(b) / 255.0 * alpha, alpha] }
}
pub(crate) fn color_clear_r32ui(value: u32) -> vk::ClearColorValue {
    vk::ClearColorValue { uint32: [value, 0, 0, 0] }
}
pub(crate) fn get_preferred_depth_stencil_format(d24: bool) -> vk::Format {
    if d24 { vk::Format::D24_UNORM_S8_UINT } else { vk::Format::D32_SFLOAT_S8_UINT }
}
pub(crate) fn rect2d(bounds: &IAABB) -> vk::Rect2D {
    vk::Rect2D { offset: vk::Offset2D { x: bounds.left, y: bounds.top },
        extent: vk::Extent2D { width: (bounds.right - bounds.left) as u32,
            height: (bounds.bottom - bounds.top) as u32 } }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_pipeline_state_mappings_are_exact() {
        assert_eq!(vkStencilOp(StencilOp::keep), vk::StencilOp::KEEP);
        assert_eq!(vkStencilOp(StencilOp::replace), vk::StencilOp::REPLACE);
        assert_eq!(vkStencilOp(StencilOp::zero), vk::StencilOp::ZERO);
        assert_eq!(vkStencilOp(StencilOp::decrClamp), vk::StencilOp::DECREMENT_AND_CLAMP);
        assert_eq!(vkStencilOp(StencilOp::incrWrap), vk::StencilOp::INCREMENT_AND_WRAP);
        assert_eq!(vkStencilOp(StencilOp::decrWrap), vk::StencilOp::DECREMENT_AND_WRAP);

        assert_eq!(vkCompareOp(StencilCompareOp::less), vk::CompareOp::LESS);
        assert_eq!(vkCompareOp(StencilCompareOp::equal), vk::CompareOp::EQUAL);
        assert_eq!(vkCompareOp(StencilCompareOp::lessOrEqual), vk::CompareOp::LESS_OR_EQUAL);
        assert_eq!(vkCompareOp(StencilCompareOp::notEqual), vk::CompareOp::NOT_EQUAL);
        assert_eq!(vkCompareOp(StencilCompareOp::always), vk::CompareOp::ALWAYS);

        assert_eq!(vkCullMode(CullFace::none), vk::CullModeFlags::NONE);
        assert_eq!(vkCullMode(CullFace::clockwise), vk::CullModeFlags::FRONT);
        assert_eq!(vkCullMode(CullFace::counterclockwise), vk::CullModeFlags::BACK);
        assert!(hasPipelineDynamicState(DrawType::msaaDynamicMidpointFans));
    }

    #[test]
    fn source_clear_color_is_argb_to_premultiplied_rgba() {
        let clear = color_clear_rgba32f(0x80402010);
        let actual = unsafe { clear.float32 };
        let alpha = 128.0 / 255.0;
        assert_eq!(actual, [64.0 / 255.0 * alpha, 32.0 / 255.0 * alpha,
            16.0 / 255.0 * alpha, alpha]);
        assert_eq!(unsafe { color_clear_r32ui(17).uint32 }, [17, 0, 0, 0]);
    }

    #[test]
    fn every_intrusive_source_base_is_at_offset_zero() {
        assert_eq!(core::mem::offset_of!(Resource, base), 0);
        assert_eq!(core::mem::offset_of!(Buffer, base), 0);
        assert_eq!(core::mem::offset_of!(BufferPool, base), 0);
        assert_eq!(core::mem::offset_of!(Image, base), 0);
        assert_eq!(core::mem::offset_of!(ImageView, base), 0);
        assert_eq!(core::mem::offset_of!(Texture2D, base), 0);
        assert_eq!(core::mem::offset_of!(Framebuffer, base), 0);
    }
}
