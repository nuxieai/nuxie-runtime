//! Complete mechanical implementation translation of
//! `renderer/src/vulkan/render_context_vulkan_impl.cpp`.

#![allow(non_snake_case, non_upper_case_globals)]

use super::common_layouts_decl as layout;
use super::draw_pipeline_layout_vulkan_decl::DrawPipelineLayoutVulkan;
use super::draw_pipeline_vulkan_decl::{DrawPipelineOptions, PipelineProps};
use super::pipeline_manager_vulkan_decl::PLSBackingType;
use super::pipeline_manager_vulkan_decl::{PipelineManagerVulkan, ShaderCompilationMode};
use super::render_context_vulkan_decl::*;
use super::render_pass_vulkan_decl::RenderPassOptionsVulkan;
use super::render_target_vulkan_decl::{
    RenderTargetVulkan, RenderTargetVulkanApi, RenderTargetVulkanImpl,
};
use super::vkutil_decl::{
    self as vkutil, ImageAccess, ImageAccessAction, Mappability, Texture2D, ViewportFromRect2D,
};
use super::vulkan_context_decl::{VulkanContext, VulkanFeatures};
use super::vulkan_shaders_decl as spirv;
#[cfg(feature = "native-ore-vulkan-experimental")]
use crate::mechanical_port::source::include::rive::refcnt_hpp::static_rcp_cast;
use crate::mechanical_port::source::include::rive::refcnt_hpp::{make_rcp, rcp, RefCntTarget};
use crate::mechanical_port::source::include::rive::renderer_hpp::{
    RenderBuffer, RenderBufferContract, RenderBufferFlags, RenderBufferType,
};
use crate::mechanical_port::source::include::utils::lite_rtti_hpp::{
    LiteRttiCastFrom, LiteRttiTypeId, CONST_ID,
};
use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::*;
#[cfg(feature = "native-ore-vulkan-experimental")]
use crate::mechanical_port::source::renderer::include::rive::renderer::render_canvas_hpp::RenderCanvas;
use crate::mechanical_port::source::renderer::include::rive::renderer::render_context_hpp::{
    FlushResources, RenderContext, RenderContextContract,
};
use crate::mechanical_port::source::renderer::include::rive::renderer::render_context_impl_hpp::{
    RenderContextImpl, RenderContextImplContract,
};
use crate::mechanical_port::source::renderer::include::rive::renderer::render_target_hpp::{
    RenderTarget, IAABB as TargetIAABB,
};
#[cfg(feature = "native-ore-vulkan-experimental")]
use crate::mechanical_port::source::renderer::include::rive::renderer::rive_render_image_hpp::RiveRenderImage;
#[cfg(feature = "native-ore-vulkan-experimental")]
use crate::mechanical_port::source::renderer::include::rive::renderer::texture_hpp::Texture;
use ash::vk;
use ash::vk::Handle;
use nuxie_ore_metal::gpu_resource::{GpuResourcePayload, ResourceHandle};
use nuxie_render_api::{BlendMode, ColorInt};
use std::ffi::{c_void, CStr};
use std::mem::{size_of, ManuallyDrop};
use std::ptr::NonNull;
use std::sync::Arc;
use std::time::Instant;

const PLS_TRANSIENT_COVERAGE_IDX: u32 = 0;
const PLS_TRANSIENT_CLIP_IDX: u32 = 1;
const PER_FLUSH_BINDINGS_SET: u32 = 0;
const PER_DRAW_BINDINGS_SET: u32 = 1;
const PLS_TEXTURE_BINDINGS_SET: u32 = 2;
const VULKAN_BINDINGS_SET_COUNT: u32 = 3;
const FLUSH_UNIFORM_BUFFER_IDX: u32 = 0;
const PATH_BUFFER_IDX: u32 = 2;
const PAINT_BUFFER_IDX: u32 = 3;
const PAINT_AUX_BUFFER_IDX: u32 = 4;
const CONTOUR_BUFFER_IDX: u32 = 5;
const COVERAGE_BUFFER_IDX: u32 = 6;
const TESS_VERTEX_TEXTURE_IDX: u32 = 7;
const GRAD_TEXTURE_IDX: u32 = 8;
const GAUSSIAN_INTEGRAL_TEXTURE_IDX: u32 = 9;
const FEATHER_ATLAS_TEXTURE_IDX: u32 = 10;
const IMAGE_TEXTURE_IDX: u32 = 11;
const COLOR_PLANE_IDX: usize = 0;
const CLIP_PLANE_IDX: usize = 1;
const SCRATCH_COLOR_PLANE_IDX: usize = 2;
const COVERAGE_PLANE_IDX: usize = 3;
const PLS_PLANE_COUNT: usize = 4;
const COALESCED_ATOMIC_RESOLVE_IDX: usize = SCRATCH_COLOR_PLANE_IDX;
const MSAA_DEPTH_STENCIL_IDX: usize = 1;
const MSAA_RESOLVE_IDX: usize = 2;
const MSAA_COLOR_SEED_IDX: usize = 3;

const K_MAX_IMAGE_TEXTURE_UPDATES: u32 = 256;

trait IAABBSourceOps {
    fn width(&self) -> i32;
    fn height(&self) -> i32;
    fn contains(&self, other: &IAABB) -> bool;
    fn offset(self, x: i32, y: i32) -> IAABB;
    fn intersect(self, other: &IAABB) -> IAABB;
    fn intersectOrEmpty(self, other: &IAABB) -> IAABB;
}

impl IAABBSourceOps for IAABB {
    fn width(&self) -> i32 {
        self.right - self.left
    }
    fn height(&self) -> i32 {
        self.bottom - self.top
    }
    fn contains(&self, other: &IAABB) -> bool {
        self.left <= other.left
            && self.top <= other.top
            && self.right >= other.right
            && self.bottom >= other.bottom
    }
    fn offset(self, x: i32, y: i32) -> IAABB {
        IAABB {
            left: self.left + x,
            top: self.top + y,
            right: self.right + x,
            bottom: self.bottom + y,
        }
    }
    fn intersect(self, other: &IAABB) -> IAABB {
        IAABB {
            left: self.left.max(other.left),
            top: self.top.max(other.top),
            right: self.right.min(other.right),
            bottom: self.bottom.min(other.bottom),
        }
    }
    fn intersectOrEmpty(self, other: &IAABB) -> IAABB {
        let result = self.intersect(other);
        if result.empty() {
            IAABB::default()
        } else {
            result
        }
    }
}

fn make_wh(width: i32, height: i32) -> IAABB {
    IAABB {
        left: 0,
        top: 0,
        right: width,
        bottom: height,
    }
}

fn to_gpu_bounds(value: TargetIAABB) -> IAABB {
    IAABB {
        left: value.left,
        top: value.top,
        right: value.right,
        bottom: value.bottom,
    }
}

fn to_target_bounds(value: &IAABB) -> TargetIAABB {
    TargetIAABB {
        left: value.left,
        top: value.top,
        right: value.right,
        bottom: value.bottom,
    }
}

fn product_sampler(
    value: crate::mechanical_port::source::include::rive::shapes::paint::image_sampler_hpp::ImageSampler,
) -> nuxie_render_api::ImageSampler {
    use nuxie_render_api::{ImageFilter, ImageWrap};
    nuxie_render_api::ImageSampler {
        wrap_x: match value.wrapX.0 {
            0 => ImageWrap::Clamp,
            1 => ImageWrap::Repeat,
            2 => ImageWrap::Mirror,
            _ => unreachable!(),
        },
        wrap_y: match value.wrapY.0 {
            0 => ImageWrap::Clamp,
            1 => ImageWrap::Repeat,
            2 => ImageWrap::Mirror,
            _ => unreachable!(),
        },
        filter: match value.filter.0 {
            0 => ImageFilter::Bilinear,
            1 => ImageFilter::Nearest,
            _ => unreachable!(),
        },
    }
}

fn instance_chunks(count: u32, first: u32, max: u32) -> impl Iterator<Item = (u32, u32)> {
    let mut remaining = count;
    let mut cursor = first;
    std::iter::from_fn(move || {
        if remaining == 0 {
            return None;
        }
        let chunk = remaining.min(max);
        let result = (chunk, cursor);
        remaining -= chunk;
        cursor += chunk;
        Some(result)
    })
}

fn cstr(bytes: &'static [u8]) -> &'static CStr {
    CStr::from_bytes_with_nul(bytes).expect("static Vulkan label")
}

fn as_bytes<T>(values: &[T]) -> &[u8] {
    unsafe { core::slice::from_raw_parts(values.as_ptr().cast(), core::mem::size_of_val(values)) }
}

#[cold]
#[track_caller]
fn vk_check<T>(result: Result<T, vk::Result>) -> T {
    match result {
        Ok(value) => value,
        Err(error) => super::vkutil_impl::vk_abort(error, file!(), line!()),
    }
}

unsafe fn rcp_ref<T: RefCntTarget>(value: &rcp<T>) -> &T {
    debug_assert!(!value.get().is_null());
    unsafe { &*value.get() }
}

unsafe fn rcp_mut<T: RefCntTarget>(value: &mut rcp<T>) -> &mut T {
    debug_assert!(!value.get().is_null());
    unsafe { &mut *value.get() }
}

fn render_buffer_usage_flags(buffer_type: RenderBufferType) -> vk::BufferUsageFlags {
    match buffer_type {
        RenderBufferType::index => vk::BufferUsageFlags::INDEX_BUFFER,
        RenderBufferType::vertex => vk::BufferUsageFlags::VERTEX_BUFFER,
    }
}

impl RenderBufferVulkanImpl {
    fn new(
        vk_context: Arc<VulkanContext>,
        buffer_type: RenderBufferType,
        flags: RenderBufferFlags,
        size_in_bytes: usize,
    ) -> Self {
        let pool = super::vkutil_decl::BufferPool::new(
            Arc::clone(&vk_context),
            render_buffer_usage_flags(buffer_type),
            size_in_bytes as vk::DeviceSize,
        );
        Self {
            base: ManuallyDrop::new(unsafe {
                crate::mechanical_port::source::renderer::include::rive::renderer::rive_render_buffer_hpp::RiveRenderBuffer::new_for_owner::<Self>(
                    buffer_type,
                    flags,
                    size_in_bytes,
                )
            }),
            m_bufferPool: ManuallyDrop::new(ResourceHandle::new_with_installed_manager(pool)),
            m_currentBuffer: ManuallyDrop::new(None),
        }
    }

    fn currentBuffer(&self) -> Option<&super::vkutil_decl::Buffer> {
        self.m_currentBuffer.as_ref().map(|buffer| &**buffer)
    }
}

impl LiteRttiTypeId for RenderBufferVulkanImpl {
    const LITE_RTTI_TYPE_ID: u32 = CONST_ID("RenderBufferVulkanImpl");
}

impl LiteRttiCastFrom<RenderBuffer> for RenderBufferVulkanImpl {
    unsafe fn from_base(base: *mut RenderBuffer) -> *mut Self {
        base.cast()
    }
}

impl RenderBufferContract for RenderBufferVulkanImpl {
    fn onMap(&mut self) -> *mut c_void {
        if let Some(buffer) = self.m_currentBuffer.take() {
            self.m_bufferPool.recycle(buffer);
        }
        let buffer = self.m_bufferPool.acquire();
        let contents = buffer.contents().cast::<c_void>();
        *self.m_currentBuffer = Some(buffer);
        contents
    }

    fn onUnmap(&mut self) {
        self.m_currentBuffer
            .as_ref()
            .expect("mapped Vulkan render buffer")
            .flushAllContents();
    }
}

unsafe impl RefCntTarget for RenderBufferVulkanImpl {
    fn r#ref(&self) {
        self.base.base.r#ref();
    }
    unsafe fn unref(&self) {
        unsafe { self.base.base.unref() }
    }
    unsafe fn onRefCntReachedZero(ptr: *const Self) {
        unsafe { drop(Box::from_raw(ptr.cast_mut())) }
    }
}

impl Drop for RenderBufferVulkanImpl {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.m_currentBuffer);
            ManuallyDrop::drop(&mut self.m_bufferPool);
            ManuallyDrop::drop(&mut self.base);
        }
    }
}

pub(crate) fn makeRenderBuffer(
    implementation: &RenderContextVulkanImpl,
    buffer_type: RenderBufferType,
    flags: RenderBufferFlags,
    size_in_bytes: usize,
) -> rcp<RenderBuffer> {
    let derived = make_rcp(|| {
        RenderBufferVulkanImpl::new(
            Arc::clone(&implementation.m_vk),
            buffer_type,
            flags,
            size_in_bytes,
        )
    });
    unsafe { crate::mechanical_port::source::include::rive::refcnt_hpp::static_rcp_cast(derived) }
}

pub(crate) fn makeImageTexture(
    implementation: &RenderContextVulkanImpl,
    width: u32,
    height: u32,
    mip_level_count: u32,
    format: crate::mechanical_port::source::include::rive::gpu_texture_format_hpp::GPUTextureFormat,
    image_data: &[u8],
    block_width: u8,
    block_height: u8,
    _srgb: bool,
    generate_remaining_mips: bool,
) -> rcp<Texture2D> {
    use crate::mechanical_port::source::include::rive::gpu_texture_format_hpp::GPUTextureFormat;
    let (vk_format, bytes_per_block, compressed) = match format {
        GPUTextureFormat::rgba32 => {
            debug_assert_eq!((block_width, block_height), (1, 1));
            (vk::Format::R8G8B8A8_UNORM, 4usize, false)
        }
        GPUTextureFormat::bc7 => (vk::Format::BC7_UNORM_BLOCK, 16, true),
        GPUTextureFormat::etc2 => (vk::Format::ETC2_R8G8B8A8_UNORM_BLOCK, 16, true),
        GPUTextureFormat::astc => {
            let index = crate::mechanical_port::source::decoders::include::rive::decoders::astc_footprints_hpp::astcFootprintIndex(block_width, block_height);
            if index < 0 {
                debug_assert!(false, "unsupported ASTC block footprint");
                return rcp::new();
            }
            (
                vk::Format::from_raw(vk::Format::ASTC_4X4_UNORM_BLOCK.as_raw() + 2 * index),
                16,
                true,
            )
        }
        _ => {
            debug_assert!(false, "unsupported format");
            return rcp::new();
        }
    };
    debug_assert!(!(generate_remaining_mips && compressed));
    let info = vk::ImageCreateInfo::default()
        .format(vk_format)
        .extent(vk::Extent3D {
            width,
            height,
            depth: 1,
        })
        .mip_levels(mip_level_count);
    let texture = implementation
        .m_vk
        .makeTexture2D(info, Some(cstr(b"RenderContext imageTexture\0")));
    if image_data.is_empty() {
        return texture;
    }
    if generate_remaining_mips {
        let mip0_bytes = width as usize * height as usize * bytes_per_block;
        unsafe { rcp_ref(&texture) }.scheduleUploadBytes(&image_data[..mip0_bytes]);
        return texture;
    }
    let mut regions = Vec::with_capacity(mip_level_count as usize);
    let mut source_offset = 0usize;
    for level in 0..mip_level_count {
        let level_width = (width >> level).max(1);
        let level_height = (height >> level).max(1);
        let blocks_x = (level_width + block_width as u32 - 1) / block_width as u32;
        let blocks_y = (level_height + block_height as u32 - 1) / block_height as u32;
        regions.push(vk::BufferImageCopy {
            buffer_offset: source_offset as u64,
            image_subresource: vk::ImageSubresourceLayers {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                mip_level: level,
                layer_count: 1,
                ..Default::default()
            },
            image_extent: vk::Extent3D {
                width: level_width,
                height: level_height,
                depth: 1,
            },
            ..Default::default()
        });
        source_offset += blocks_x as usize * blocks_y as usize * bytes_per_block;
    }
    let staging = implementation.m_vk.makeBuffer(
        vk::BufferCreateInfo::default()
            .size(source_offset as u64)
            .usage(vk::BufferUsageFlags::TRANSFER_SRC),
        Mappability::writeOnly,
    );
    unsafe {
        core::ptr::copy_nonoverlapping(image_data.as_ptr(), staging.contents(), source_offset)
    };
    staging.flushAllContents();
    unsafe { rcp_ref(&texture) }.scheduleUploadRegions(staging, regions);
    texture
}

pub(crate) unsafe fn adoptImageTexture(
    implementation: &RenderContextVulkanImpl,
    image: vk::Image,
    width: u32,
    height: u32,
    format: vk::Format,
) -> rcp<Texture2D> {
    if image == vk::Image::null() || width == 0 || height == 0 {
        return rcp::new();
    }
    let info = vk::ImageCreateInfo::default()
        .image_type(vk::ImageType::TYPE_2D)
        .format(format)
        .extent(vk::Extent3D {
            width,
            height,
            depth: 1,
        })
        .mip_levels(1)
        .array_layers(1)
        .samples(vk::SampleCountFlags::TYPE_1)
        .usage(vk::ImageUsageFlags::SAMPLED);
    let external =
        implementation
            .m_vk
            .makeExternalImage(image, info, Some(cstr(b"adopted external image\0")));
    let texture = implementation
        .m_vk
        .makeTexture2DFromImage(external, Some(cstr(b"adopted external image\0")));
    unsafe { rcp_ref(&texture) }.overrideLastAccess(ImageAccess {
        pipelineStages: vk::PipelineStageFlags::FRAGMENT_SHADER,
        accessMask: vk::AccessFlags::SHADER_READ,
        layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
    });
    texture
}

#[repr(C)]
struct RenderTargetVulkanTexture {
    base: ManuallyDrop<RenderTargetVulkan>,
    m_texture: ManuallyDrop<rcp<Texture2D>>,
}

unsafe fn destroy_render_target_vulkan_texture(ptr: *mut RenderTarget) {
    unsafe { drop(Box::from_raw(ptr.cast::<RenderTargetVulkanTexture>())) };
}

unsafe impl RefCntTarget for RenderTargetVulkanTexture {
    fn r#ref(&self) {
        self.base.base.r#ref()
    }
    unsafe fn unref(&self) {
        unsafe { self.base.base.unref() }
    }
    unsafe fn onRefCntReachedZero(ptr: *const Self) {
        unsafe { drop(Box::from_raw(ptr.cast_mut())) };
    }
}

impl RenderTargetVulkanApi for RenderTargetVulkanTexture {
    fn base(&self) -> &RenderTargetVulkan {
        &self.base
    }
    fn baseMut(&mut self) -> &mut RenderTargetVulkan {
        &mut self.base
    }
    fn targetImage(&self) -> vk::Image {
        unsafe { rcp_ref(&self.m_texture) }.vkImage()
    }
    fn targetImageView(&self) -> vk::ImageView {
        unsafe { rcp_ref(&self.m_texture) }.vkImageView()
    }
    fn updateLastAccess(&mut self, access: ImageAccess) {
        unsafe { *rcp_ref(&self.m_texture).lastAccessMut() = access };
    }
    fn accessTargetImage(
        &mut self,
        command_buffer: vk::CommandBuffer,
        dst_access: ImageAccess,
        action: ImageAccessAction,
    ) -> vk::Image {
        let texture = unsafe { rcp_ref(&self.m_texture) };
        texture.barrier(
            command_buffer,
            dst_access,
            action,
            vk::DependencyFlags::empty(),
        );
        texture.vkImage()
    }
    fn accessTargetImageView(
        &mut self,
        command_buffer: vk::CommandBuffer,
        dst_access: ImageAccess,
        action: ImageAccessAction,
    ) -> vk::ImageView {
        self.accessTargetImage(command_buffer, dst_access, action);
        unsafe { rcp_ref(&self.m_texture) }.vkImageView()
    }
}

impl Drop for RenderTargetVulkanTexture {
    fn drop(&mut self) {
        unsafe {
            ManuallyDrop::drop(&mut self.m_texture);
            ManuallyDrop::drop(&mut self.base);
        }
    }
}

#[cfg(feature = "native-ore-vulkan-experimental")]
pub(crate) fn makeRenderCanvas(
    implementation: &mut RenderContextVulkanImpl,
    width: u32,
    height: u32,
) -> rcp<RenderCanvas> {
    let format = vk::Format::R8G8B8A8_UNORM;
    let usage = vk::ImageUsageFlags::COLOR_ATTACHMENT
        | vk::ImageUsageFlags::SAMPLED
        | vk::ImageUsageFlags::TRANSFER_SRC
        | vk::ImageUsageFlags::TRANSFER_DST
        | vk::ImageUsageFlags::INPUT_ATTACHMENT
        | vk::ImageUsageFlags::STORAGE;
    let texture = implementation.m_vk.makeTexture2D(
        vk::ImageCreateInfo::default()
            .format(format)
            .extent(vk::Extent3D {
                width,
                height,
                depth: 1,
            })
            .usage(usage),
        Some(cstr(b"RenderCanvas\0")),
    );
    let image_texture: rcp<Texture> = unsafe { static_rcp_cast(rcp::copy_ctor(&texture)) };
    let render_image = make_rcp(|| unsafe { RiveRenderImage::new(image_texture) });
    let target = make_rcp(|| {
        let mut render_target = RenderTarget::new(width, height);
        render_target.destroy_complete = destroy_render_target_vulkan_texture;
        RenderTargetVulkanTexture {
            base: ManuallyDrop::new(RenderTargetVulkan {
                base: ManuallyDrop::new(render_target),
                m_vk: ManuallyDrop::new(Arc::clone(&implementation.m_vk)),
                m_framebufferFormat: format,
                m_targetUsageFlags: usage,
                m_offscreenColorTexture: ManuallyDrop::new(rcp::new()),
                m_msaaColorTexture: ManuallyDrop::new(rcp::new()),
                m_msaaDepthStencilTexture: ManuallyDrop::new(rcp::new()),
            }),
            m_texture: ManuallyDrop::new(texture),
        }
    });
    let render_target: rcp<RenderTarget> = unsafe { static_rcp_cast(target) };
    make_rcp(|| unsafe { RenderCanvas::new(render_image, render_target) })
}

impl ResourceTexturePipeline {
    fn new(
        vk_context: Arc<VulkanContext>,
        format: vk::Format,
        load_op: vk::AttachmentLoadOp,
        consumption_stage: vk::PipelineStageFlags,
        label: &'static [u8],
        workarounds: DriverWorkarounds,
    ) -> Self {
        let attachment = vk::AttachmentDescription::default()
            .format(format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(load_op)
            .store_op(vk::AttachmentStoreOp::STORE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);
        let color_ref = vk::AttachmentReference {
            attachment: 0,
            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
        };
        let subpass = vk::SubpassDescription::default()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(core::slice::from_ref(&color_ref));
        let dependencies = [
            vk::SubpassDependency::default()
                .src_subpass(vk::SUBPASS_EXTERNAL)
                .dst_subpass(0)
                .src_stage_mask(consumption_stage)
                .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
                .src_access_mask(vk::AccessFlags::SHADER_READ)
                .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE),
            vk::SubpassDependency::default()
                .src_subpass(0)
                .dst_subpass(vk::SUBPASS_EXTERNAL)
                .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
                .dst_stage_mask(consumption_stage)
                .src_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
                .dst_access_mask(vk::AccessFlags::SHADER_READ),
        ];
        let create = vk::RenderPassCreateInfo::default()
            .attachments(core::slice::from_ref(&attachment))
            .subpasses(core::slice::from_ref(&subpass))
            .dependencies(&dependencies);
        let render_pass =
            vk_check(unsafe { vk_context.ashDevice().create_render_pass(&create, None) });
        vk_context.setDebugNameIfEnabled(
            render_pass,
            vk::ObjectType::RENDER_PASS,
            Some(cstr(label)),
        );
        let mut resuming = vk::RenderPass::null();
        if workarounds.needsInterruptibleRenderPasses() {
            let resume_attachment = attachment
                .load_op(vk::AttachmentLoadOp::LOAD)
                .initial_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);
            let resume_create = vk::RenderPassCreateInfo::default()
                .attachments(core::slice::from_ref(&resume_attachment))
                .subpasses(core::slice::from_ref(&subpass))
                .dependencies(&dependencies);
            resuming = vk_check(unsafe {
                vk_context
                    .ashDevice()
                    .create_render_pass(&resume_create, None)
            });
            let authored_label = cstr(label).to_string_lossy();
            let label_stem = authored_label
                .strip_suffix(" RenderPass")
                .unwrap_or(&authored_label);
            let resume_label = std::ffi::CString::new(format!("{label_stem} RESUME RenderPass"))
                .expect("source render-pass label has no interior nul");
            vk_context.setDebugNameIfEnabled(
                resuming,
                vk::ObjectType::RENDER_PASS,
                Some(&resume_label),
            );
        }
        Self {
            m_vk: vk_context,
            m_renderPass: render_pass,
            m_resumingRenderPass: resuming,
            m_instanceCountInCurrentRenderPass: 0,
        }
    }

    fn beginRenderPass(
        &mut self,
        command_buffer: vk::CommandBuffer,
        render_area: vk::Rect2D,
        framebuffer: vk::Framebuffer,
        resume: bool,
    ) {
        let pass = if resume {
            self.m_resumingRenderPass
        } else {
            self.m_renderPass
        };
        debug_assert_ne!(pass, vk::RenderPass::null());
        let clear = vk::ClearValue::default();
        let info = vk::RenderPassBeginInfo::default()
            .render_pass(pass)
            .framebuffer(framebuffer)
            .render_area(render_area)
            .clear_values(core::slice::from_ref(&clear));
        unsafe {
            self.m_vk.ashDevice().cmd_begin_render_pass(
                command_buffer,
                &info,
                vk::SubpassContents::INLINE,
            )
        };
        self.m_instanceCountInCurrentRenderPass = 0;
    }

    fn interruptRenderPassIfNeeded(
        &mut self,
        command_buffer: vk::CommandBuffer,
        render_area: vk::Rect2D,
        framebuffer: vk::Framebuffer,
        next_instance_count: u32,
        workarounds: DriverWorkarounds,
    ) {
        debug_assert!(
            self.m_instanceCountInCurrentRenderPass <= workarounds.maxInstancesPerRenderPass
        );
        debug_assert!(next_instance_count <= workarounds.maxInstancesPerRenderPass);
        if self.m_instanceCountInCurrentRenderPass + next_instance_count
            > workarounds.maxInstancesPerRenderPass
        {
            unsafe { self.m_vk.ashDevice().cmd_end_render_pass(command_buffer) };
            self.beginRenderPass(command_buffer, render_area, framebuffer, true);
        }
        self.m_instanceCountInCurrentRenderPass += next_instance_count;
    }
}

impl Drop for ResourceTexturePipeline {
    fn drop(&mut self) {
        unsafe {
            self.m_vk
                .ashDevice()
                .destroy_render_pass(self.m_renderPass, None);
            if self.m_resumingRenderPass != vk::RenderPass::null() {
                self.m_vk
                    .ashDevice()
                    .destroy_render_pass(self.m_resumingRenderPass, None);
            }
        }
    }
}

fn shader_module(vk_context: &VulkanContext, words: &[u32]) -> vk::ShaderModule {
    let info = vk::ShaderModuleCreateInfo::default().code(words);
    vk_check(unsafe { vk_context.ashDevice().create_shader_module(&info, None) })
}

fn shader_stage(
    stage: vk::ShaderStageFlags,
    module: vk::ShaderModule,
) -> vk::PipelineShaderStageCreateInfo<'static> {
    vk::PipelineShaderStageCreateInfo::default()
        .stage(stage)
        .module(module)
        .name(cstr(b"main\0"))
}

impl ColorRampPipeline {
    fn new(manager: &PipelineManagerVulkan, workarounds: DriverWorkarounds) -> Box<Self> {
        let mut base = ResourceTexturePipeline::new(
            Arc::clone(&manager.m_vk),
            vk::Format::R8G8B8A8_UNORM,
            vk::AttachmentLoadOp::DONT_CARE,
            vk::PipelineStageFlags::FRAGMENT_SHADER,
            b"ColorRamp RenderPass\0",
            workarounds,
        );
        let layouts = [manager.perFlushDescriptorSetLayout()];
        let pipeline_layout = vk_check(unsafe {
            manager.m_vk.ashDevice().create_pipeline_layout(
                &vk::PipelineLayoutCreateInfo::default().set_layouts(&layouts),
                None,
            )
        });
        let vertex_words = spirv::color_ramp_vert
            .read()
            .expect("embedded color ramp vertex shader");
        let fragment_words = spirv::color_ramp_frag
            .read()
            .expect("embedded color ramp fragment shader");
        let vertex = shader_module(&manager.m_vk, &vertex_words);
        let fragment = shader_module(&manager.m_vk, &fragment_words);
        let stages = [
            shader_stage(vk::ShaderStageFlags::VERTEX, vertex),
            shader_stage(vk::ShaderStageFlags::FRAGMENT, fragment),
        ];
        let binding = [vk::VertexInputBindingDescription {
            binding: 0,
            stride: size_of::<GradientSpan>() as u32,
            input_rate: vk::VertexInputRate::INSTANCE,
        }];
        let attributes = [vk::VertexInputAttributeDescription {
            location: 0,
            binding: 0,
            format: vk::Format::R32G32B32A32_UINT,
            offset: 0,
        }];
        let vertex_input = vk::PipelineVertexInputStateCreateInfo::default()
            .vertex_binding_descriptions(&binding)
            .vertex_attribute_descriptions(&attributes);
        let assembly = vk::PipelineInputAssemblyStateCreateInfo::default()
            .topology(vk::PrimitiveTopology::TRIANGLE_STRIP);
        let viewport = vk::PipelineViewportStateCreateInfo::default()
            .viewport_count(1)
            .scissor_count(1);
        let raster = vk::PipelineRasterizationStateCreateInfo::default()
            .line_width(1.0)
            .cull_mode(vk::CullModeFlags::BACK)
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE);
        let multisample = vk::PipelineMultisampleStateCreateInfo::default()
            .rasterization_samples(vk::SampleCountFlags::TYPE_1);
        let blend_attachment = [vk::PipelineColorBlendAttachmentState::default()
            .color_write_mask(vkutil::kColorWriteMaskRGBA)];
        let blend = vk::PipelineColorBlendStateCreateInfo::default().attachments(&blend_attachment);
        let dynamics = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic = vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamics);
        let create = vk::GraphicsPipelineCreateInfo::default()
            .stages(&stages)
            .vertex_input_state(&vertex_input)
            .input_assembly_state(&assembly)
            .viewport_state(&viewport)
            .rasterization_state(&raster)
            .multisample_state(&multisample)
            .color_blend_state(&blend)
            .dynamic_state(&dynamic)
            .layout(pipeline_layout)
            .render_pass(base.m_renderPass);
        let render_pipeline = match unsafe {
            manager.m_vk.ashDevice().create_graphics_pipelines(
                vk::PipelineCache::null(),
                &[create],
                None,
            )
        } {
            Ok(mut values) => values.remove(0),
            Err((_, error)) => super::vkutil_impl::vk_abort(error, file!(), line!()),
        };
        unsafe {
            manager.m_vk.ashDevice().destroy_shader_module(vertex, None);
            manager
                .m_vk
                .ashDevice()
                .destroy_shader_module(fragment, None);
        }
        Box::new(Self {
            base: ManuallyDrop::new(base),
            m_pipelineLayout: pipeline_layout,
            m_renderPipeline: render_pipeline,
        })
    }
}

impl Drop for ColorRampPipeline {
    fn drop(&mut self) {
        unsafe {
            self.base
                .m_vk
                .ashDevice()
                .destroy_pipeline_layout(self.m_pipelineLayout, None);
            self.base
                .m_vk
                .ashDevice()
                .destroy_pipeline(self.m_renderPipeline, None);
            ManuallyDrop::drop(&mut self.base);
        }
    }
}

impl TessellatePipeline {
    fn new(manager: &PipelineManagerVulkan, workarounds: DriverWorkarounds) -> Box<Self> {
        let base = ResourceTexturePipeline::new(
            Arc::clone(&manager.m_vk),
            vk::Format::R32G32B32A32_UINT,
            vk::AttachmentLoadOp::DONT_CARE,
            vk::PipelineStageFlags::VERTEX_SHADER,
            b"Tessellate RenderPass\0",
            workarounds,
        );
        let layouts = [
            manager.perFlushDescriptorSetLayout(),
            manager.emptyDescriptorSetLayout(),
        ];
        let pipeline_layout = vk_check(unsafe {
            manager.m_vk.ashDevice().create_pipeline_layout(
                &vk::PipelineLayoutCreateInfo::default().set_layouts(&layouts),
                None,
            )
        });
        let vertex_words = spirv::tessellate_vert
            .read()
            .expect("embedded tessellate vertex shader");
        let fragment_words = spirv::tessellate_frag
            .read()
            .expect("embedded tessellate fragment shader");
        let vertex = shader_module(&manager.m_vk, &vertex_words);
        let fragment = shader_module(&manager.m_vk, &fragment_words);
        let stages = [
            shader_stage(vk::ShaderStageFlags::VERTEX, vertex),
            shader_stage(vk::ShaderStageFlags::FRAGMENT, fragment),
        ];
        let binding = [vk::VertexInputBindingDescription {
            binding: 0,
            stride: size_of::<TessVertexSpan>() as u32,
            input_rate: vk::VertexInputRate::INSTANCE,
        }];
        let attributes = [
            vk::VertexInputAttributeDescription {
                location: 0,
                binding: 0,
                format: vk::Format::R32G32B32A32_SFLOAT,
                offset: 0,
            },
            vk::VertexInputAttributeDescription {
                location: 1,
                binding: 0,
                format: vk::Format::R32G32B32A32_SFLOAT,
                offset: 4 * size_of::<f32>() as u32,
            },
            vk::VertexInputAttributeDescription {
                location: 2,
                binding: 0,
                format: vk::Format::R32G32B32A32_SFLOAT,
                offset: 8 * size_of::<f32>() as u32,
            },
            vk::VertexInputAttributeDescription {
                location: 3,
                binding: 0,
                format: vk::Format::R32G32B32A32_UINT,
                offset: 12 * size_of::<f32>() as u32,
            },
        ];
        let vertex_input = vk::PipelineVertexInputStateCreateInfo::default()
            .vertex_binding_descriptions(&binding)
            .vertex_attribute_descriptions(&attributes);
        let assembly = vk::PipelineInputAssemblyStateCreateInfo::default()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST);
        let viewport = vk::PipelineViewportStateCreateInfo::default()
            .viewport_count(1)
            .scissor_count(1);
        let raster = vk::PipelineRasterizationStateCreateInfo::default()
            .line_width(1.0)
            .cull_mode(vk::CullModeFlags::BACK)
            .front_face(vk::FrontFace::COUNTER_CLOCKWISE);
        let multisample = vk::PipelineMultisampleStateCreateInfo::default()
            .rasterization_samples(vk::SampleCountFlags::TYPE_1);
        let blend_attachment = [vk::PipelineColorBlendAttachmentState::default()
            .color_write_mask(vkutil::kColorWriteMaskRGBA)];
        let blend = vk::PipelineColorBlendStateCreateInfo::default().attachments(&blend_attachment);
        let dynamics = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic = vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamics);
        let create = vk::GraphicsPipelineCreateInfo::default()
            .stages(&stages)
            .vertex_input_state(&vertex_input)
            .input_assembly_state(&assembly)
            .viewport_state(&viewport)
            .rasterization_state(&raster)
            .multisample_state(&multisample)
            .color_blend_state(&blend)
            .dynamic_state(&dynamic)
            .layout(pipeline_layout)
            .render_pass(base.m_renderPass);
        let render_pipeline = match unsafe {
            manager.m_vk.ashDevice().create_graphics_pipelines(
                vk::PipelineCache::null(),
                &[create],
                None,
            )
        } {
            Ok(mut values) => values.remove(0),
            Err((_, error)) => super::vkutil_impl::vk_abort(error, file!(), line!()),
        };
        unsafe {
            manager.m_vk.ashDevice().destroy_shader_module(vertex, None);
            manager
                .m_vk
                .ashDevice()
                .destroy_shader_module(fragment, None);
        }
        Box::new(Self {
            base: ManuallyDrop::new(base),
            m_pipelineLayout: pipeline_layout,
            m_renderPipeline: render_pipeline,
        })
    }
}

impl Drop for TessellatePipeline {
    fn drop(&mut self) {
        unsafe {
            self.base
                .m_vk
                .ashDevice()
                .destroy_pipeline_layout(self.m_pipelineLayout, None);
            self.base
                .m_vk
                .ashDevice()
                .destroy_pipeline(self.m_renderPipeline, None);
            ManuallyDrop::drop(&mut self.base);
        }
    }
}

impl FeatherAtlasPipeline {
    fn new(manager: &PipelineManagerVulkan, workarounds: DriverWorkarounds) -> Box<Self> {
        let base = ResourceTexturePipeline::new(
            Arc::clone(&manager.m_vk),
            manager.featherAtlasFormat(),
            vk::AttachmentLoadOp::CLEAR,
            vk::PipelineStageFlags::FRAGMENT_SHADER,
            b"Feather Atlas RenderPass\0",
            workarounds,
        );
        let layouts = [
            manager.perFlushDescriptorSetLayout(),
            manager.emptyDescriptorSetLayout(),
        ];
        let pipeline_layout = vk_check(unsafe {
            manager.m_vk.ashDevice().create_pipeline_layout(
                &vk::PipelineLayoutCreateInfo::default().set_layouts(&layouts),
                None,
            )
        });
        let vertex_words = spirv::render_atlas_vert
            .read()
            .expect("embedded atlas vertex shader");
        let fill_words = spirv::render_atlas_fill_frag
            .read()
            .expect("embedded atlas fill shader");
        let stroke_words = spirv::render_atlas_stroke_frag
            .read()
            .expect("embedded atlas stroke shader");
        let vertex = shader_module(&manager.m_vk, &vertex_words);
        let fill_fragment = shader_module(&manager.m_vk, &fill_words);
        let stroke_fragment = shader_module(&manager.m_vk, &stroke_words);
        let assembly = vk::PipelineInputAssemblyStateCreateInfo::default()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST);
        let viewport = vk::PipelineViewportStateCreateInfo::default()
            .viewport_count(1)
            .scissor_count(1);
        let raster = vk::PipelineRasterizationStateCreateInfo::default()
            .line_width(1.0)
            .cull_mode(vk::CullModeFlags::NONE)
            .front_face(vk::FrontFace::CLOCKWISE);
        let multisample = vk::PipelineMultisampleStateCreateInfo::default()
            .rasterization_samples(vk::SampleCountFlags::TYPE_1);
        let dynamics = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic = vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamics);
        let make_pipeline = |fragment: vk::ShaderModule, operation: vk::BlendOp| {
            let stages = [
                shader_stage(vk::ShaderStageFlags::VERTEX, vertex),
                shader_stage(vk::ShaderStageFlags::FRAGMENT, fragment),
            ];
            let attachment = [vk::PipelineColorBlendAttachmentState::default()
                .blend_enable(true)
                .src_color_blend_factor(vk::BlendFactor::ONE)
                .dst_color_blend_factor(vk::BlendFactor::ONE)
                .color_blend_op(operation)
                .color_write_mask(vk::ColorComponentFlags::R)];
            let blend = vk::PipelineColorBlendStateCreateInfo::default().attachments(&attachment);
            let create = vk::GraphicsPipelineCreateInfo::default()
                .stages(&stages)
                .vertex_input_state(&layout::PATH_VERTEX_INPUT_STATE)
                .input_assembly_state(&assembly)
                .viewport_state(&viewport)
                .rasterization_state(&raster)
                .multisample_state(&multisample)
                .color_blend_state(&blend)
                .dynamic_state(&dynamic)
                .layout(pipeline_layout)
                .render_pass(base.m_renderPass);
            match unsafe {
                manager.m_vk.ashDevice().create_graphics_pipelines(
                    vk::PipelineCache::null(),
                    &[create],
                    None,
                )
            } {
                Ok(mut values) => values.remove(0),
                Err((_, error)) => super::vkutil_impl::vk_abort(error, file!(), line!()),
            }
        };
        let fill_pipeline = make_pipeline(fill_fragment, vk::BlendOp::ADD);
        let stroke_pipeline = make_pipeline(stroke_fragment, vk::BlendOp::MAX);
        unsafe {
            manager.m_vk.ashDevice().destroy_shader_module(vertex, None);
            manager
                .m_vk
                .ashDevice()
                .destroy_shader_module(fill_fragment, None);
            manager
                .m_vk
                .ashDevice()
                .destroy_shader_module(stroke_fragment, None);
        }
        Box::new(Self {
            base: ManuallyDrop::new(base),
            m_pipelineLayout: pipeline_layout,
            m_fillPipeline: fill_pipeline,
            m_strokePipeline: stroke_pipeline,
        })
    }
}

impl Drop for FeatherAtlasPipeline {
    fn drop(&mut self) {
        unsafe {
            self.base
                .m_vk
                .ashDevice()
                .destroy_pipeline_layout(self.m_pipelineLayout, None);
            self.base
                .m_vk
                .ashDevice()
                .destroy_pipeline(self.m_fillPipeline, None);
            self.base
                .m_vk
                .ashDevice()
                .destroy_pipeline(self.m_strokePipeline, None);
            ManuallyDrop::drop(&mut self.base);
        }
    }
}

impl RenderContextVulkanImpl {
    fn new(vk_context: Arc<VulkanContext>, options: ContextOptions) -> Self {
        let properties = vk_context.physicalDeviceProperties();
        let vendor = properties.vendor_id;
        let workarounds = DriverWorkarounds {
            maxInstancesPerRenderPass: if properties.api_version < vk::API_VERSION_1_3
                && (vendor == vkutil::ARM || vendor == vkutil::Imagination)
            {
                (1 << 13) - 1
            } else {
                u32::MAX
            },
            avoidManualMSAAResolves: vendor == vkutil::Samsung,
            needsManualMSAAResolveAfterDstRead: vendor == vkutil::Qualcomm,
        };
        let mut base = RenderContextImpl::default();
        base.m_platformFeatures.supportsRasterOrderingMode =
            !options.forceAtomicMode && vk_context.features.rasterizationOrderColorAttachmentAccess;
        base.m_platformFeatures.supportsAtomicMode = vk_context.features.fragmentStoresAndAtomics;
        #[cfg(target_os = "android")]
        {
            base.m_platformFeatures.supportsAtomicMode &= options.forceAtomicMode;
        }
        #[cfg(not(target_os = "android"))]
        {
            base.m_platformFeatures.supportsClockwiseMode =
                vk_context.features.fragmentShaderPixelInterlock
                    && !options.forceAtomicMode
                    && !base.m_platformFeatures.supportsRasterOrderingMode;
            base.m_platformFeatures.supportsClockwiseFixedFunctionMode =
                base.m_platformFeatures.supportsClockwiseMode
                    && !options.disableClockwiseFixedFunctionMode;
        }
        base.m_platformFeatures.supportsClockwiseAtomicMode =
            base.m_platformFeatures.supportsAtomicMode;
        base.m_platformFeatures.supportsClipPlanes =
            vk_context.features.shaderClipDistance && properties.limits.max_clip_distances >= 4;
        base.m_platformFeatures.supportsPipelineDynamicState = vk_context.features.apiVersion
            >= vk::API_VERSION_1_3
            && vk_context.features.colorWriteEnable
            && !workarounds.needsInterruptibleRenderPasses()
            && vendor != vkutil::Imagination;
        base.m_platformFeatures.clipSpaceBottomUp = false;
        base.m_platformFeatures.framebufferBottomUp = false;
        base.m_platformFeatures.msaaColorPreserveNeedsDraw = true;
        base.m_platformFeatures.maxTextureSize = properties.limits.max_image_dimension2_d;
        base.m_platformFeatures.supportsClipScissor = true;
        base.m_platformFeatures.supportsTextureCompressionBC =
            vk_context.features.textureCompressionBC;
        base.m_platformFeatures.supportsTextureCompressionASTC =
            vk_context.features.textureCompressionASTC_LDR;
        base.m_platformFeatures.supportsTextureCompressionETC2 =
            vk_context.features.textureCompressionETC2;
        base.m_platformFeatures.maxCoverageBufferLength =
            properties.limits.max_storage_buffer_range.min(1 << 28) as usize / size_of::<u32>();
        match vendor {
            vkutil::Qualcomm => {
                base.m_platformFeatures.supportsRasterOrderingMode = false;
                base.m_platformFeatures
                    .clockwiseAtomicBorrowedCoverageBarrierNeedsRenderPassInit = true;
                base.m_platformFeatures.pathIDGranularity = 2;
            }
            vkutil::ARM => {
                base.m_platformFeatures.supportsRasterOrderingMode = !options.forceAtomicMode
            }
            vkutil::Imagination => {
                base.m_platformFeatures.supportsRasterOrderingMode = !options.forceAtomicMode
                    && vk_context.features.apiVersion >= vk::API_VERSION_1_3
            }
            _ => {}
        }
        let manager = Arc::new(DescriptorSetPoolPool {
            base: nuxie_ore_metal::new_gpu_resource_pool_backend_base(
                vk_context.manager(),
                DescriptorSetPoolPool::MAX_POOL_SIZE,
            ),
            m_vk: Arc::clone(&vk_context),
        });
        Self {
            base: ManuallyDrop::new(base),
            m_vk: ManuallyDrop::new(Arc::clone(&vk_context)),
            m_canvasQueue: vk::Queue::null(),
            m_canvasQueueFamilyIndex: 0,
            m_canvasCommandPool: vk::CommandPool::null(),
            m_workarounds: workarounds,
            m_flushUniformBufferPool: ManuallyDrop::new(super::vkutil_decl::BufferPool::new(
                Arc::clone(&vk_context),
                vk::BufferUsageFlags::UNIFORM_BUFFER,
                0,
            )),
            m_pathBufferPool: ManuallyDrop::new(super::vkutil_decl::BufferPool::new(
                Arc::clone(&vk_context),
                vk::BufferUsageFlags::STORAGE_BUFFER,
                0,
            )),
            m_paintBufferPool: ManuallyDrop::new(super::vkutil_decl::BufferPool::new(
                Arc::clone(&vk_context),
                vk::BufferUsageFlags::STORAGE_BUFFER,
                0,
            )),
            m_paintAuxBufferPool: ManuallyDrop::new(super::vkutil_decl::BufferPool::new(
                Arc::clone(&vk_context),
                vk::BufferUsageFlags::STORAGE_BUFFER,
                0,
            )),
            m_contourBufferPool: ManuallyDrop::new(super::vkutil_decl::BufferPool::new(
                Arc::clone(&vk_context),
                vk::BufferUsageFlags::STORAGE_BUFFER,
                0,
            )),
            m_gradSpanBufferPool: ManuallyDrop::new(super::vkutil_decl::BufferPool::new(
                Arc::clone(&vk_context),
                vk::BufferUsageFlags::VERTEX_BUFFER,
                0,
            )),
            m_tessSpanBufferPool: ManuallyDrop::new(super::vkutil_decl::BufferPool::new(
                Arc::clone(&vk_context),
                vk::BufferUsageFlags::VERTEX_BUFFER,
                0,
            )),
            m_triangleBufferPool: ManuallyDrop::new(super::vkutil_decl::BufferPool::new(
                Arc::clone(&vk_context),
                vk::BufferUsageFlags::VERTEX_BUFFER,
                0,
            )),
            m_imageDrawInstanceBufferPool: ManuallyDrop::new(super::vkutil_decl::BufferPool::new(
                Arc::clone(&vk_context),
                vk::BufferUsageFlags::VERTEX_BUFFER,
                0,
            )),
            m_flushUniformBuffer: ManuallyDrop::new(None),
            m_pathBuffer: ManuallyDrop::new(None),
            m_paintBuffer: ManuallyDrop::new(None),
            m_paintAuxBuffer: ManuallyDrop::new(None),
            m_contourBuffer: ManuallyDrop::new(None),
            m_gradSpanBuffer: ManuallyDrop::new(None),
            m_tessSpanBuffer: ManuallyDrop::new(None),
            m_triangleBuffer: ManuallyDrop::new(None),
            m_imageDrawInstanceBuffer: ManuallyDrop::new(None),
            m_localEpoch: Instant::now(),
            m_nullImageTexture: ManuallyDrop::new(rcp::new()),
            m_colorRampPipeline: ManuallyDrop::new(None),
            m_gradTexture: ManuallyDrop::new(rcp::new()),
            m_gradTextureFramebuffer: ManuallyDrop::new(None),
            m_tessellatePipeline: ManuallyDrop::new(None),
            m_tessSpanIndexBuffer: ManuallyDrop::new(None),
            m_tessTexture: ManuallyDrop::new(rcp::new()),
            m_tesselationSyncIssueWorkaroundTexture: ManuallyDrop::new(rcp::new()),
            m_tessTextureFramebuffer: ManuallyDrop::new(None),
            m_featherAtlasPipeline: ManuallyDrop::new(None),
            m_featherAtlasTexture: ManuallyDrop::new(rcp::new()),
            m_featherAtlasFramebuffer: ManuallyDrop::new(None),
            m_plsTransientUsageFlags: vk::ImageUsageFlags::empty(),
            m_plsExtent: vk::Extent3D {
                width: 0,
                height: 0,
                depth: 1,
            },
            m_plsTransientPlaneCount: 0,
            m_plsTransientImageArray: ManuallyDrop::new(None),
            m_plsTransientCoverageView: ManuallyDrop::new(None),
            m_plsTransientClipView: ManuallyDrop::new(None),
            m_plsTransientScratchColorTexture: ManuallyDrop::new(rcp::new()),
            m_plsBlendStorageTexture_RGB10_A2: ManuallyDrop::new(rcp::new()),
            m_plsTransientClipTexture_R16F: ManuallyDrop::new(rcp::new()),
            m_plsOffscreenColorTexture: ManuallyDrop::new(rcp::new()),
            m_plsAtomicCoverageTexture: ManuallyDrop::new(rcp::new()),
            m_coverageBuffer: ManuallyDrop::new(None),
            m_gaussianIntegralTexture: ManuallyDrop::new(rcp::new()),
            m_pathPatchVertexBuffer: ManuallyDrop::new(None),
            m_pathPatchIndexBuffer: ManuallyDrop::new(None),
            m_imageRectVertexBuffer: ManuallyDrop::new(None),
            m_imageRectIndexBuffer: ManuallyDrop::new(None),
            m_descriptorSetPoolPool: ManuallyDrop::new(manager),
            m_pipelineManager: ManuallyDrop::new(None),
        }
    }

    fn initGPUObjects(&mut self, mode: ShaderCompilationMode) {
        let black = [0u8, 0, 0, 1];
        *self.m_nullImageTexture = self.m_vk.makeTexture2D(
            vk::ImageCreateInfo::default()
                .format(vk::Format::R8G8B8A8_UNORM)
                .extent(vk::Extent3D {
                    width: 1,
                    height: 1,
                    depth: 1,
                }),
            Some(cstr(b"null image texture\0")),
        );
        unsafe { rcp_ref(&self.m_nullImageTexture) }.scheduleUploadBytes(&black);
        let device_name =
            unsafe { CStr::from_ptr(self.m_vk.physicalDeviceProperties().device_name.as_ptr()) };
        if device_name
            .to_bytes()
            .windows(b"Adreno (TM) 8".len())
            .any(|window| window == b"Adreno (TM) 8")
        {
            *self.m_tesselationSyncIssueWorkaroundTexture = self.m_vk.makeTexture2D(
                vk::ImageCreateInfo::default()
                    .format(vk::Format::R8G8B8A8_UINT)
                    .extent(vk::Extent3D {
                        width: 1,
                        height: 1,
                        depth: 1,
                    })
                    .usage(vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_DST),
                Some(cstr(b"tesselation sync bug workaround texture\0")),
            );
        }
        *self.m_pipelineManager = Some(PipelineManagerVulkan::new(
            Arc::clone(&self.m_vk),
            mode,
            unsafe { rcp_ref(&self.m_nullImageTexture) }.vkImageView(),
        ));
        let manager = self.m_pipelineManager.as_ref().unwrap();
        *self.m_colorRampPipeline = Some(ColorRampPipeline::new(manager, self.m_workarounds));
        *self.m_tessellatePipeline = Some(TessellatePipeline::new(manager, self.m_workarounds));
        *self.m_featherAtlasPipeline = Some(FeatherAtlasPipeline::new(manager, self.m_workarounds));
        self.m_plsTransientUsageFlags =
            vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::INPUT_ATTACHMENT;
        if self.base.m_platformFeatures.supportsClockwiseMode {
            self.m_plsTransientUsageFlags |=
                vk::ImageUsageFlags::STORAGE | vk::ImageUsageFlags::TRANSFER_DST;
        } else if !self.m_workarounds.needsInterruptibleRenderPasses() {
            self.m_plsTransientUsageFlags |= vk::ImageUsageFlags::TRANSIENT_ATTACHMENT;
        }
        let mut gaussian = [0u16; GAUSSIAN_TABLE_SIZE as usize * 2];
        gaussian[..GAUSSIAN_TABLE_SIZE as usize].copy_from_slice(
            &crate::mechanical_port::source::renderer::src::gpu_cpp::g_gaussianIntegralTableF16,
        );
        gaussian[GAUSSIAN_TABLE_SIZE as usize..].copy_from_slice(
            &crate::mechanical_port::source::renderer::src::gpu_cpp::g_inverseGaussianIntegralTableF16,
        );
        *self.m_gaussianIntegralTexture = self.m_vk.makeTexture2D(
            vk::ImageCreateInfo::default()
                .format(vk::Format::R16_SFLOAT)
                .extent(vk::Extent3D {
                    width: GAUSSIAN_TABLE_SIZE,
                    height: 2,
                    depth: 1,
                }),
            Some(cstr(b"gaussian integral texture\0")),
        );
        unsafe { rcp_ref(&self.m_gaussianIntegralTexture) }
            .scheduleUploadBytes(as_bytes(&gaussian));
        let tess_indices = as_bytes(&kTessSpanIndices);
        let tess_index = self.m_vk.makeBuffer(
            vk::BufferCreateInfo::default()
                .size(tess_indices.len() as u64)
                .usage(vk::BufferUsageFlags::INDEX_BUFFER),
            Mappability::writeOnly,
        );
        unsafe {
            core::ptr::copy_nonoverlapping(
                tess_indices.as_ptr(),
                tess_index.contents(),
                tess_indices.len(),
            )
        };
        tess_index.flushAllContents();
        *self.m_tessSpanIndexBuffer = Some(tess_index);
        let vertex = self.m_vk.makeBuffer(
            vk::BufferCreateInfo::default()
                .size(kPatchVertexBufferCount as u64 * size_of::<PatchVertex>() as u64)
                .usage(vk::BufferUsageFlags::VERTEX_BUFFER),
            Mappability::writeOnly,
        );
        let index = self.m_vk.makeBuffer(
            vk::BufferCreateInfo::default()
                .size(kPatchIndexBufferCount as u64 * size_of::<u16>() as u64)
                .usage(vk::BufferUsageFlags::INDEX_BUFFER),
            Mappability::writeOnly,
        );
        unsafe { GeneratePatchBufferData(vertex.contents().cast(), index.contents().cast()) };
        vertex.flushAllContents();
        index.flushAllContents();
        *self.m_pathPatchVertexBuffer = Some(vertex);
        *self.m_pathPatchIndexBuffer = Some(index);
        let rect_vertices = as_bytes(&kImageRectVertices);
        let rect_vertex = self.m_vk.makeBuffer(
            vk::BufferCreateInfo::default()
                .size(rect_vertices.len() as u64)
                .usage(vk::BufferUsageFlags::VERTEX_BUFFER),
            Mappability::writeOnly,
        );
        unsafe {
            core::ptr::copy_nonoverlapping(
                rect_vertices.as_ptr(),
                rect_vertex.contents(),
                rect_vertices.len(),
            )
        };
        rect_vertex.flushAllContents();
        *self.m_imageRectVertexBuffer = Some(rect_vertex);
        let rect_indices = as_bytes(&kImageRectIndices);
        let rect_index = self.m_vk.makeBuffer(
            vk::BufferCreateInfo::default()
                .size(rect_indices.len() as u64)
                .usage(vk::BufferUsageFlags::INDEX_BUFFER),
            Mappability::writeOnly,
        );
        unsafe {
            core::ptr::copy_nonoverlapping(
                rect_indices.as_ptr(),
                rect_index.contents(),
                rect_indices.len(),
            )
        };
        rect_index.flushAllContents();
        *self.m_imageRectIndexBuffer = Some(rect_index);
    }
}

pub(crate) fn resizeGradientTexture(
    implementation: &mut RenderContextVulkanImpl,
    width: u32,
    height: u32,
) {
    let width = width.max(1);
    let height = height.max(1);
    *implementation.m_gradTexture = implementation.m_vk.makeTexture2D(
        vk::ImageCreateInfo::default()
            .format(vk::Format::R8G8B8A8_UNORM)
            .extent(vk::Extent3D {
                width,
                height,
                depth: 1,
            })
            .usage(vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::SAMPLED),
        Some(cstr(b"gradient texture\0")),
    );
    let attachment = unsafe { rcp_ref(&implementation.m_gradTexture) }.vkImageView();
    let pass = implementation
        .m_colorRampPipeline
        .as_ref()
        .unwrap()
        .base
        .m_renderPass;
    *implementation.m_gradTextureFramebuffer = Some(
        implementation.m_vk.makeFramebuffer(
            vk::FramebufferCreateInfo::default()
                .render_pass(pass)
                .attachments(core::slice::from_ref(&attachment))
                .width(width)
                .height(height)
                .layers(1),
        ),
    );
}

pub(crate) fn resizeTessellationTexture(
    implementation: &mut RenderContextVulkanImpl,
    width: u32,
    height: u32,
) {
    let width = width.max(1);
    let height = height.max(1);
    let mut usage = vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::SAMPLED;
    if implementation
        .m_tesselationSyncIssueWorkaroundTexture
        .operator_bool()
    {
        usage |= vk::ImageUsageFlags::TRANSFER_SRC;
    }
    *implementation.m_tessTexture = implementation.m_vk.makeTexture2D(
        vk::ImageCreateInfo::default()
            .format(vk::Format::R32G32B32A32_UINT)
            .extent(vk::Extent3D {
                width,
                height,
                depth: 1,
            })
            .usage(usage),
        Some(cstr(b"tesselation texture\0")),
    );
    let attachment = unsafe { rcp_ref(&implementation.m_tessTexture) }.vkImageView();
    let pass = implementation
        .m_tessellatePipeline
        .as_ref()
        .unwrap()
        .base
        .m_renderPass;
    *implementation.m_tessTextureFramebuffer = Some(
        implementation.m_vk.makeFramebuffer(
            vk::FramebufferCreateInfo::default()
                .render_pass(pass)
                .attachments(core::slice::from_ref(&attachment))
                .width(width)
                .height(height)
                .layers(1),
        ),
    );
}

pub(crate) fn resizeFeatherAtlasTexture(
    implementation: &mut RenderContextVulkanImpl,
    width: u32,
    height: u32,
) {
    let width = width.max(1);
    let height = height.max(1);
    let format = implementation
        .m_pipelineManager
        .as_ref()
        .unwrap()
        .featherAtlasFormat();
    *implementation.m_featherAtlasTexture = implementation.m_vk.makeTexture2D(
        vk::ImageCreateInfo::default()
            .format(format)
            .extent(vk::Extent3D {
                width,
                height,
                depth: 1,
            })
            .usage(vk::ImageUsageFlags::COLOR_ATTACHMENT | vk::ImageUsageFlags::SAMPLED),
        Some(cstr(b"feather atlas texture\0")),
    );
    let attachment = unsafe { rcp_ref(&implementation.m_featherAtlasTexture) }.vkImageView();
    let pass = implementation
        .m_featherAtlasPipeline
        .as_ref()
        .unwrap()
        .base
        .m_renderPass;
    *implementation.m_featherAtlasFramebuffer = Some(
        implementation.m_vk.makeFramebuffer(
            vk::FramebufferCreateInfo::default()
                .render_pass(pass)
                .attachments(core::slice::from_ref(&attachment))
                .width(width)
                .height(height)
                .layers(1),
        ),
    );
}

pub(crate) fn resizeTransientPLSBacking(
    implementation: &mut RenderContextVulkanImpl,
    width: u32,
    height: u32,
    plane_count: u32,
) {
    implementation.m_plsExtent = vk::Extent3D {
        width,
        height,
        depth: 1,
    };
    implementation.m_plsTransientPlaneCount = plane_count;
    *implementation.m_plsTransientImageArray = None;
    *implementation.m_plsTransientCoverageView = None;
    *implementation.m_plsTransientClipView = None;
    *implementation.m_plsTransientScratchColorTexture = rcp::new();
    *implementation.m_plsBlendStorageTexture_RGB10_A2 = rcp::new();
    *implementation.m_plsTransientClipTexture_R16F = rcp::new();
    *implementation.m_plsOffscreenColorTexture = rcp::new();
}

pub(crate) fn resizeAtomicCoverageBacking(
    implementation: &mut RenderContextVulkanImpl,
    width: u32,
    height: u32,
) {
    *implementation.m_plsAtomicCoverageTexture = rcp::new();
    if width != 0 && height != 0 {
        *implementation.m_plsAtomicCoverageTexture = implementation.m_vk.makeTexture2D(
            vk::ImageCreateInfo::default()
                .format(vk::Format::R32_UINT)
                .extent(vk::Extent3D {
                    width,
                    height,
                    depth: 1,
                })
                .usage(vk::ImageUsageFlags::STORAGE | vk::ImageUsageFlags::TRANSFER_DST),
            Some(cstr(b"atomic coverage backing\0")),
        );
    }
}

pub(crate) fn resizeCoverageBuffer(
    implementation: &mut RenderContextVulkanImpl,
    size_in_bytes: usize,
) {
    *implementation.m_coverageBuffer = if size_in_bytes == 0 {
        None
    } else {
        Some(
            implementation.m_vk.makeBuffer(
                vk::BufferCreateInfo::default()
                    .size(size_in_bytes as u64)
                    .usage(
                        vk::BufferUsageFlags::STORAGE_BUFFER | vk::BufferUsageFlags::TRANSFER_DST,
                    ),
                Mappability::none,
            ),
        )
    };
}

fn plsTransientImageArray(
    implementation: &mut RenderContextVulkanImpl,
) -> &ResourceHandle<super::vkutil_decl::Image> {
    debug_assert!(implementation.m_plsExtent.width != 0 && implementation.m_plsExtent.height != 0);
    debug_assert_eq!(implementation.m_plsExtent.depth, 1);
    debug_assert_ne!(implementation.m_plsTransientPlaneCount, 0);
    if implementation.m_plsTransientImageArray.is_none() {
        *implementation.m_plsTransientImageArray = Some(
            implementation.m_vk.makeImage(
                vk::ImageCreateInfo::default()
                    .image_type(vk::ImageType::TYPE_2D)
                    .format(vk::Format::R32_UINT)
                    .extent(implementation.m_plsExtent)
                    .array_layers(implementation.m_plsTransientPlaneCount.min(2))
                    .usage(implementation.m_plsTransientUsageFlags),
                Some(cstr(b"plsTransientImageArray\0")),
            ),
        );
    }
    implementation.m_plsTransientImageArray.as_ref().unwrap()
}

fn makePLSTransientImageView(
    implementation: &mut RenderContextVulkanImpl,
    format: vk::Format,
    index: u32,
    name: &'static [u8],
) -> ResourceHandle<super::vkutil_decl::ImageView> {
    let plane_count = implementation.m_plsTransientPlaneCount;
    let image = plsTransientImageArray(implementation).clone();
    implementation.m_vk.makeImageViewWithInfo(
        image,
        vk::ImageViewCreateInfo::default()
            .view_type(vk::ImageViewType::TYPE_2D)
            .format(format)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: vk::ImageAspectFlags::COLOR,
                level_count: 1,
                base_array_layer: index.min(plane_count - 1),
                layer_count: 1,
                ..Default::default()
            }),
        Some(cstr(name)),
    )
}

fn plsTransientCoverageView(implementation: &mut RenderContextVulkanImpl) -> vk::ImageView {
    if implementation.m_plsTransientCoverageView.is_none() {
        *implementation.m_plsTransientCoverageView = Some(makePLSTransientImageView(
            implementation,
            vk::Format::R32_UINT,
            PLS_TRANSIENT_COVERAGE_IDX,
            b"plsTransientCoverageView\0",
        ));
    }
    implementation
        .m_plsTransientCoverageView
        .as_ref()
        .unwrap()
        .vkImageView()
}

fn plsTransientClipView(implementation: &mut RenderContextVulkanImpl) -> vk::ImageView {
    if implementation.m_plsTransientClipView.is_none() {
        *implementation.m_plsTransientClipView = Some(makePLSTransientImageView(
            implementation,
            vk::Format::R32_UINT,
            PLS_TRANSIENT_CLIP_IDX,
            b"plsTransientClipView\0",
        ));
    }
    implementation
        .m_plsTransientClipView
        .as_ref()
        .unwrap()
        .vkImageView()
}

fn lazyTexture(
    slot: &mut rcp<Texture2D>,
    vk_context: &Arc<VulkanContext>,
    info: vk::ImageCreateInfo<'_>,
    name: &'static [u8],
) -> *mut Texture2D {
    if !slot.operator_bool() {
        *slot = vk_context.makeTexture2D(info, Some(cstr(name)));
    }
    slot.get()
}

fn plsTransientScratchColorTexture(implementation: &mut RenderContextVulkanImpl) -> *mut Texture2D {
    debug_assert!(
        implementation.m_plsExtent.width != 0 && implementation.m_plsTransientPlaneCount != 0
    );
    lazyTexture(
        &mut implementation.m_plsTransientScratchColorTexture,
        &implementation.m_vk,
        vk::ImageCreateInfo::default()
            .flags(vk::ImageCreateFlags::MUTABLE_FORMAT)
            .format(vk::Format::R8G8B8A8_UNORM)
            .extent(implementation.m_plsExtent)
            .usage(implementation.m_plsTransientUsageFlags),
        b"plsTransientScratchColorTexture\0",
    )
}

fn plsBlendStorageTexture_RGB10_A2(implementation: &mut RenderContextVulkanImpl) -> *mut Texture2D {
    lazyTexture(
        &mut implementation.m_plsBlendStorageTexture_RGB10_A2,
        &implementation.m_vk,
        vk::ImageCreateInfo::default()
            .format(vk::Format::A2B10G10R10_UNORM_PACK32)
            .extent(implementation.m_plsExtent)
            .usage(vk::ImageUsageFlags::STORAGE | vk::ImageUsageFlags::TRANSFER_DST),
        b"plsBlendStorageTexture_RGB10_A2\0",
    )
}

fn plsTransientClipTexture_R16F(implementation: &mut RenderContextVulkanImpl) -> *mut Texture2D {
    lazyTexture(
        &mut implementation.m_plsTransientClipTexture_R16F,
        &implementation.m_vk,
        vk::ImageCreateInfo::default()
            .format(vk::Format::R16_SFLOAT)
            .extent(implementation.m_plsExtent)
            .usage(implementation.m_plsTransientUsageFlags),
        b"plsTransientClipTexture_R16F\0",
    )
}

fn accessPLSOffscreenColorTexture(
    implementation: &mut RenderContextVulkanImpl,
    command_buffer: vk::CommandBuffer,
    dst_access: ImageAccess,
    action: ImageAccessAction,
) -> *mut Texture2D {
    let usage = (implementation.m_plsTransientUsageFlags | vk::ImageUsageFlags::TRANSFER_SRC)
        & !vk::ImageUsageFlags::TRANSIENT_ATTACHMENT;
    let texture = lazyTexture(
        &mut implementation.m_plsOffscreenColorTexture,
        &implementation.m_vk,
        vk::ImageCreateInfo::default()
            .format(vk::Format::R8G8B8A8_UNORM)
            .extent(implementation.m_plsExtent)
            .usage(usage),
        b"PLSOffscreenColorTexture\0",
    );
    unsafe {
        (&*texture).barrier(
            command_buffer,
            dst_access,
            action,
            vk::DependencyFlags::empty(),
        )
    };
    texture
}

fn clearPLSOffscreenColorTexture(
    implementation: &mut RenderContextVulkanImpl,
    command_buffer: vk::CommandBuffer,
    clear_color: ColorInt,
    dst_access: ImageAccess,
) -> *mut Texture2D {
    let texture = accessPLSOffscreenColorTexture(
        implementation,
        command_buffer,
        ImageAccess {
            pipelineStages: vk::PipelineStageFlags::TRANSFER,
            accessMask: vk::AccessFlags::TRANSFER_WRITE,
            layout: vk::ImageLayout::GENERAL,
        },
        ImageAccessAction::invalidateContents,
    );
    implementation.m_vk.clearColorImage(
        command_buffer,
        clear_color,
        unsafe { (&*texture).vkImage() },
        vk::ImageLayout::GENERAL,
    );
    accessPLSOffscreenColorTexture(
        implementation,
        command_buffer,
        dst_access,
        ImageAccessAction::preserveContents,
    )
}

enum RenderTargetVulkanDispatch<'a> {
    External(&'a mut RenderTargetVulkanImpl),
    Texture(&'a mut RenderTargetVulkanTexture),
}

impl RenderTargetVulkanApi for RenderTargetVulkanDispatch<'_> {
    fn base(&self) -> &RenderTargetVulkan {
        match self {
            Self::External(target) => target.base(),
            Self::Texture(target) => target.base(),
        }
    }
    fn baseMut(&mut self) -> &mut RenderTargetVulkan {
        match self {
            Self::External(target) => target.baseMut(),
            Self::Texture(target) => target.baseMut(),
        }
    }
    fn targetImage(&self) -> vk::Image {
        match self {
            Self::External(target) => target.targetImage(),
            Self::Texture(target) => target.targetImage(),
        }
    }
    fn targetImageView(&self) -> vk::ImageView {
        match self {
            Self::External(target) => target.targetImageView(),
            Self::Texture(target) => target.targetImageView(),
        }
    }
    fn updateLastAccess(&mut self, access: ImageAccess) {
        match self {
            Self::External(target) => target.updateLastAccess(access),
            Self::Texture(target) => target.updateLastAccess(access),
        }
    }
    fn accessTargetImage(
        &mut self,
        command_buffer: vk::CommandBuffer,
        dst_access: ImageAccess,
        action: ImageAccessAction,
    ) -> vk::Image {
        match self {
            Self::External(target) => target.accessTargetImage(command_buffer, dst_access, action),
            Self::Texture(target) => target.accessTargetImage(command_buffer, dst_access, action),
        }
    }
    fn accessTargetImageView(
        &mut self,
        command_buffer: vk::CommandBuffer,
        dst_access: ImageAccess,
        action: ImageAccessAction,
    ) -> vk::ImageView {
        match self {
            Self::External(target) => {
                target.accessTargetImageView(command_buffer, dst_access, action)
            }
            Self::Texture(target) => {
                target.accessTargetImageView(command_buffer, dst_access, action)
            }
        }
    }
}

fn target_impl(desc: &FlushDescriptor) -> RenderTargetVulkanDispatch<'_> {
    let target = desc.renderTarget.expect("Vulkan flush render target");
    let base = unsafe { target.as_ref() };
    if std::ptr::fn_addr_eq(
        base.destroy_complete,
        destroy_render_target_vulkan_texture as unsafe fn(*mut RenderTarget),
    ) {
        return RenderTargetVulkanDispatch::Texture(unsafe {
            &mut *target.as_ptr().cast::<RenderTargetVulkanTexture>()
        });
    }
    debug_assert!(std::ptr::fn_addr_eq(
        base.destroy_complete,
        super::render_target_vulkan_impl::destroy_render_target_vulkan_impl
            as unsafe fn(*mut RenderTarget),
    ));
    RenderTargetVulkanDispatch::External(unsafe {
        &mut *target.as_ptr().cast::<RenderTargetVulkanImpl>()
    })
}

fn copyRenderTargetToPLSOffscreenColorTexture(
    implementation: &mut RenderContextVulkanImpl,
    command_buffer: vk::CommandBuffer,
    target: &mut dyn RenderTargetVulkanApi,
    copy_bounds: &IAABB,
    dst_access: ImageAccess,
) -> *mut Texture2D {
    let source = target.accessTargetImage(
        command_buffer,
        ImageAccess {
            pipelineStages: vk::PipelineStageFlags::TRANSFER,
            accessMask: vk::AccessFlags::TRANSFER_READ,
            layout: vk::ImageLayout::GENERAL,
        },
        ImageAccessAction::preserveContents,
    );
    let destination = accessPLSOffscreenColorTexture(
        implementation,
        command_buffer,
        ImageAccess {
            pipelineStages: vk::PipelineStageFlags::TRANSFER,
            accessMask: vk::AccessFlags::TRANSFER_WRITE,
            layout: vk::ImageLayout::GENERAL,
        },
        ImageAccessAction::invalidateContents,
    );
    implementation.m_vk.blitSubRect(
        command_buffer,
        source,
        vk::ImageLayout::GENERAL,
        unsafe { (&*destination).vkImage() },
        vk::ImageLayout::GENERAL,
        copy_bounds,
    );
    accessPLSOffscreenColorTexture(
        implementation,
        command_buffer,
        dst_access,
        ImageAccessAction::preserveContents,
    )
}

pub(crate) unsafe fn wantsManualRenderPassResolve(
    implementation: &RenderContextVulkanImpl,
    interlock_mode: InterlockMode,
    render_target: *const RenderTarget,
    update_bounds: &IAABB,
    virtual_tile_width: u32,
    virtual_tile_height: u32,
    draw_contents: DrawContents,
) -> bool {
    if interlock_mode == InterlockMode::rasterOrdering
        && virtual_tile_width == 0
        && virtual_tile_height == 0
        && !implementation
            .m_workarounds
            .needsInterruptibleRenderPasses()
    {
        #[cfg(not(target_vendor = "apple"))]
        {
            let target = unsafe { &*render_target.cast::<RenderTargetVulkan>() };
            return !target
                .m_targetUsageFlags
                .contains(vk::ImageUsageFlags::INPUT_ATTACHMENT);
        }
    }
    if interlock_mode == InterlockMode::msaa
        && !implementation.m_workarounds.avoidManualMSAAResolves
    {
        let target = unsafe { &*render_target };
        if !update_bounds.contains(&to_gpu_bounds(target.bounds())) {
            return true;
        }
        if implementation
            .m_workarounds
            .needsManualMSAAResolveAfterDstRead
            && draw_contents.0 & DrawContents::advancedBlend.0 != 0
        {
            return true;
        }
    }
    false
}

pub(crate) unsafe fn setCanvasQueue(
    implementation: &mut RenderContextVulkanImpl,
    queue: vk::Queue,
    family: u32,
) {
    implementation.m_canvasQueue = queue;
    implementation.m_canvasQueueFamilyIndex = family;
    if queue != vk::Queue::null()
        && (unsafe { rcp_ref(&implementation.m_gaussianIntegralTexture) }
            .lastAccess()
            .layout
            == vk::ImageLayout::UNDEFINED
            || unsafe { rcp_ref(&implementation.m_nullImageTexture) }
                .lastAccess()
                .layout
                == vk::ImageLayout::UNDEFINED)
    {
        let command = makeCommandBuffer(implementation);
        if !command.is_null() {
            let command_buffer = vk::CommandBuffer::from_raw(command as u64);
            unsafe { rcp_ref(&implementation.m_gaussianIntegralTexture) }
                .prepareForVertexOrFragmentShaderRead(command_buffer);
            unsafe { rcp_ref(&implementation.m_nullImageTexture) }
                .prepareForFragmentShaderRead(command_buffer);
            unsafe { commitCommandBuffer(implementation, command) };
        }
    }
}

pub(crate) fn makeCommandBuffer(implementation: &mut RenderContextVulkanImpl) -> *mut c_void {
    if implementation.m_canvasQueue == vk::Queue::null() {
        return core::ptr::null_mut();
    }
    if implementation.m_canvasCommandPool == vk::CommandPool::null() {
        let info = vk::CommandPoolCreateInfo::default()
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
            .queue_family_index(implementation.m_canvasQueueFamilyIndex);
        implementation.m_canvasCommandPool = vk_check(unsafe {
            implementation
                .m_vk
                .ashDevice()
                .create_command_pool(&info, None)
        });
    }
    let info = vk::CommandBufferAllocateInfo::default()
        .command_pool(implementation.m_canvasCommandPool)
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_buffer_count(1);
    let command = match unsafe {
        implementation
            .m_vk
            .ashDevice()
            .allocate_command_buffers(&info)
    } {
        Ok(values) => values[0],
        Err(_) => return core::ptr::null_mut(),
    };
    let begin =
        vk::CommandBufferBeginInfo::default().flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);
    let _ = unsafe {
        implementation
            .m_vk
            .ashDevice()
            .begin_command_buffer(command, &begin)
    };
    command.as_raw() as usize as *mut c_void
}

pub(crate) unsafe fn commitCommandBuffer(
    implementation: &mut RenderContextVulkanImpl,
    command: *mut c_void,
) {
    if command.is_null() {
        return;
    }
    let command_buffer = vk::CommandBuffer::from_raw(command as u64);
    let _ = unsafe {
        implementation
            .m_vk
            .ashDevice()
            .end_command_buffer(command_buffer)
    };
    let submit = vk::SubmitInfo::default().command_buffers(core::slice::from_ref(&command_buffer));
    let _ = unsafe {
        implementation.m_vk.ashDevice().queue_submit(
            implementation.m_canvasQueue,
            core::slice::from_ref(&submit),
            vk::Fence::null(),
        )
    };
    let _ = unsafe {
        implementation
            .m_vk
            .ashDevice()
            .queue_wait_idle(implementation.m_canvasQueue)
    };
    unsafe {
        implementation
            .m_vk
            .ashDevice()
            .free_command_buffers(implementation.m_canvasCommandPool, &[command_buffer])
    };
}

pub(crate) fn prepareToFlush(
    implementation: &mut RenderContextVulkanImpl,
    next_frame: u64,
    safe_frame: u64,
) {
    debug_assert!(implementation.m_flushUniformBuffer.is_none());
    debug_assert!(implementation.m_pathBuffer.is_none());
    debug_assert!(implementation.m_paintBuffer.is_none());
    debug_assert!(implementation.m_paintAuxBuffer.is_none());
    debug_assert!(implementation.m_contourBuffer.is_none());
    debug_assert!(implementation.m_gradSpanBuffer.is_none());
    debug_assert!(implementation.m_tessSpanBuffer.is_none());
    debug_assert!(implementation.m_triangleBuffer.is_none());
    debug_assert!(implementation.m_imageDrawInstanceBuffer.is_none());
    if next_frame != 0 {
        implementation
            .m_vk
            .advanceFrameNumber(next_frame, safe_frame);
    }
    *implementation.m_flushUniformBuffer = Some(implementation.m_flushUniformBufferPool.acquire());
    *implementation.m_pathBuffer = Some(implementation.m_pathBufferPool.acquire());
    *implementation.m_paintBuffer = Some(implementation.m_paintBufferPool.acquire());
    *implementation.m_paintAuxBuffer = Some(implementation.m_paintAuxBufferPool.acquire());
    *implementation.m_contourBuffer = Some(implementation.m_contourBufferPool.acquire());
    *implementation.m_gradSpanBuffer = Some(implementation.m_gradSpanBufferPool.acquire());
    *implementation.m_tessSpanBuffer = Some(implementation.m_tessSpanBufferPool.acquire());
    *implementation.m_triangleBuffer = Some(implementation.m_triangleBufferPool.acquire());
    *implementation.m_imageDrawInstanceBuffer =
        Some(implementation.m_imageDrawInstanceBufferPool.acquire());
}

impl DescriptorSetPool {
    fn new(vk_context: Arc<VulkanContext>) -> Self {
        let sizes = [
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count: 3,
            },
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC,
                descriptor_count: 1,
            },
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                descriptor_count: 259,
            },
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::SAMPLED_IMAGE,
                descriptor_count: 1,
            },
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::STORAGE_IMAGE,
                descriptor_count: 4,
            },
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::STORAGE_BUFFER,
                descriptor_count: 8,
            },
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::INPUT_ATTACHMENT,
                descriptor_count: 4,
            },
        ];
        let info = vk::DescriptorPoolCreateInfo::default()
            .flags(vk::DescriptorPoolCreateFlags::FREE_DESCRIPTOR_SET)
            .max_sets(259)
            .pool_sizes(&sizes);
        let pool = vk_check(unsafe { vk_context.ashDevice().create_descriptor_pool(&info, None) });
        Self {
            base: ManuallyDrop::new(nuxie_ore_metal::new_gpu_resource_backend_base()),
            m_vk: ManuallyDrop::new(vk_context),
            m_vkDescriptorPool: pool,
        }
    }

    fn allocateDescriptorSet(&self, layout: vk::DescriptorSetLayout) -> vk::DescriptorSet {
        let info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(self.m_vkDescriptorPool)
            .set_layouts(core::slice::from_ref(&layout));
        vk_check(unsafe { self.m_vk.ashDevice().allocate_descriptor_sets(&info) })[0]
    }

    fn reset(&self) {
        vk_check(unsafe {
            self.m_vk.ashDevice().reset_descriptor_pool(
                self.m_vkDescriptorPool,
                vk::DescriptorPoolResetFlags::empty(),
            )
        });
    }
}

impl Drop for DescriptorSetPool {
    fn drop(&mut self) {
        unsafe {
            self.m_vk
                .ashDevice()
                .destroy_descriptor_pool(self.m_vkDescriptorPool, None);
            ManuallyDrop::drop(&mut self.m_vk);
            ManuallyDrop::drop(&mut self.base);
        }
    }
}

impl DescriptorSetPoolPool {
    fn acquire(&self) -> ResourceHandle<DescriptorSetPool> {
        if let Some(resource) = self.base.acquire() {
            let pool = resource
                .downcast::<DescriptorSetPool>()
                .ok()
                .expect("descriptor pool type drift");
            pool.reset();
            pool
        } else {
            ResourceHandle::new(
                Some(self.m_vk.manager()),
                DescriptorSetPool::new(Arc::clone(&self.m_vk)),
            )
        }
    }
}

impl DescriptorSetAllocator {
    fn new(implementation: &RenderContextVulkanImpl) -> Self {
        let pool_pool = Arc::clone(&implementation.m_descriptorSetPoolPool);
        let pool = pool_pool.acquire();
        let manager = implementation.m_pipelineManager.as_ref().unwrap();
        let per_flush = pool.allocateDescriptorSet(manager.perFlushDescriptorSetLayout());
        Self {
            m_descriptorSetPoolPool: pool_pool,
            m_descriptorSetPool: Some(pool),
            m_perFlushDescriptorSet: per_flush,
            m_perDrawDescriptorSetLayout: manager.perDrawDescriptorSetLayout(),
            m_imageTextureUpdateCount: 0,
        }
    }

    fn allocatePerDrawDescriptorSet(&mut self) -> vk::DescriptorSet {
        if self.m_imageTextureUpdateCount >= K_MAX_IMAGE_TEXTURE_UPDATES {
            self.m_descriptorSetPoolPool
                .base
                .recycle(self.m_descriptorSetPool.take().map(ResourceHandle::erase));
            self.m_descriptorSetPool = Some(self.m_descriptorSetPoolPool.acquire());
            self.m_imageTextureUpdateCount = 0;
        }
        self.m_imageTextureUpdateCount += 1;
        self.m_descriptorSetPool
            .as_ref()
            .unwrap()
            .allocateDescriptorSet(self.m_perDrawDescriptorSetLayout)
    }

    fn allocateDescriptorSet(&self, layout: vk::DescriptorSetLayout) -> vk::DescriptorSet {
        self.m_descriptorSetPool
            .as_ref()
            .unwrap()
            .allocateDescriptorSet(layout)
    }
}

impl Drop for DescriptorSetAllocator {
    fn drop(&mut self) {
        self.m_descriptorSetPoolPool
            .base
            .recycle(self.m_descriptorSetPool.take().map(ResourceHandle::erase));
    }
}

impl DrawRenderPass {
    fn new(
        implementation: &mut RenderContextVulkanImpl,
        desc: &FlushDescriptor,
        override_load: LoadAction,
        draw_bounds: IAABB,
        color_view: vk::ImageView,
        msaa_seed_view: vk::ImageView,
        msaa_resolve_view: vk::ImageView,
        options: RenderPassOptionsVulkan,
        scissor: IAABB,
    ) -> Self {
        let mut result = Self {
            m_impl: implementation,
            m_desc: desc,
            m_drawBounds: draw_bounds,
            m_colorImageView: color_view,
            m_msaaColorSeedImageView: msaa_seed_view,
            m_msaaResolveImageView: msaa_resolve_view,
            m_pipelineLayout: core::ptr::null(),
            m_renderPassOptions: RenderPassOptionsVulkan::none,
            m_scissor: scissor,
            m_patchCountInCurrentDrawPass: 0,
        };
        result.m_pipelineLayout = result.begin(override_load, options, scissor);
        result
    }

    fn pipelineLayout(&self) -> &DrawPipelineLayoutVulkan {
        unsafe { &*self.m_pipelineLayout }
    }

    fn begin(
        &mut self,
        override_load: LoadAction,
        options: RenderPassOptionsVulkan,
        scissor: IAABB,
    ) -> *const DrawPipelineLayoutVulkan {
        let implementation = unsafe { &mut *self.m_impl };
        let desc = unsafe { &*self.m_desc };
        let mut target = target_impl(desc);
        let command = vk::CommandBuffer::from_raw(
            desc.externalCommandBuffer
                .expect("Vulkan command buffer")
                .as_ptr() as u64,
        );
        let (render_pass_handle, pipeline_layout, backing) = {
            let manager = implementation.m_pipelineManager.as_ref().unwrap();
            let render_pass = manager.getRenderPassSynchronous(
                desc.interlockMode,
                options,
                target.base().m_framebufferFormat,
                override_load,
            );
            (
                render_pass.m_renderPass,
                render_pass
                    .drawPipelineLayout()
                    .expect("render pass pipeline layout")
                    as *const DrawPipelineLayoutVulkan,
                manager.plsBackingType(desc.interlockMode),
            )
        };
        let mut views = Vec::with_capacity(layout::MAX_RENDER_PASS_ATTACHMENTS as usize);
        let mut clears = Vec::with_capacity(layout::MAX_RENDER_PASS_ATTACHMENTS as usize);
        if backing == PLSBackingType::inputAttachment
            || unsafe { &*pipeline_layout }
                .renderPassOptions()
                .has(RenderPassOptionsVulkan::fixedFunctionColorOutput)
        {
            debug_assert_eq!(views.len(), COLOR_PLANE_IDX);
            views.push(self.m_colorImageView);
            clears.push(vk::ClearValue {
                color: vkutil::color_clear_rgba32f(desc.colorClearValue),
            });
        }
        match desc.interlockMode {
            InterlockMode::rasterOrdering => {
                debug_assert_eq!(views.len(), CLIP_PLANE_IDX);
                views.push(plsTransientClipView(implementation));
                clears.push(vk::ClearValue::default());
                debug_assert_eq!(views.len(), SCRATCH_COLOR_PLANE_IDX);
                views.push(unsafe {
                    (&*plsTransientScratchColorTexture(implementation)).vkImageView()
                });
                clears.push(vk::ClearValue::default());
                debug_assert_eq!(views.len(), COVERAGE_PLANE_IDX);
                views.push(plsTransientCoverageView(implementation));
                clears.push(vk::ClearValue {
                    color: vkutil::color_clear_r32ui(desc.coverageClearValue),
                });
                if options.has(RenderPassOptionsVulkan::manuallyResolved) {
                    debug_assert_eq!(views.len(), PLS_PLANE_COUNT);
                    views.push(target.accessTargetImageView(
                        command,
                        ImageAccess {
                            pipelineStages: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                            accessMask: vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
                            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                        },
                        ImageAccessAction::invalidateContents,
                    ));
                    clears.push(vk::ClearValue::default());
                }
            }
            InterlockMode::atomics => {
                debug_assert_eq!(views.len(), CLIP_PLANE_IDX);
                views.push(unsafe {
                    (&*plsTransientScratchColorTexture(implementation)).vkImageView()
                });
                clears.push(vk::ClearValue::default());
                if unsafe { &*pipeline_layout }
                    .renderPassOptions()
                    .has(RenderPassOptionsVulkan::atomicCoalescedResolveAndTransfer)
                {
                    debug_assert_eq!(views.len(), COALESCED_ATOMIC_RESOLVE_IDX);
                    let full = self.m_drawBounds.contains(&IAABB {
                        left: 0,
                        top: 0,
                        right: target.base().width() as i32,
                        bottom: target.base().height() as i32,
                    });
                    views.push(target.accessTargetImageView(
                        command,
                        ImageAccess {
                            pipelineStages: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                            accessMask: vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
                            layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
                        },
                        if full {
                            ImageAccessAction::invalidateContents
                        } else {
                            ImageAccessAction::preserveContents
                        },
                    ));
                    clears.push(vk::ClearValue::default());
                }
            }
            InterlockMode::clockwise => {}
            InterlockMode::clockwiseAtomic => {
                debug_assert_eq!(views.len(), CLIP_PLANE_IDX);
                views.push(unsafe {
                    (&*plsTransientClipTexture_R16F(implementation)).vkImageView()
                });
                clears.push(vk::ClearValue::default());
            }
            InterlockMode::msaa => {
                debug_assert_eq!(views.len(), MSAA_DEPTH_STENCIL_IDX);
                let depth = super::render_target_vulkan_impl::msaaDepthStencilTexture(&mut target);
                views.push(unsafe { (&*depth).vkImageView() });
                clears.push(vk::ClearValue {
                    depth_stencil: vk::ClearDepthStencilValue {
                        depth: desc.depthClearValue,
                        stencil: desc.stencilClearValue as u32,
                    },
                });
                debug_assert_eq!(views.len(), MSAA_RESOLVE_IDX);
                views.push(self.m_msaaResolveImageView);
                clears.push(vk::ClearValue::default());
                if unsafe { &*pipeline_layout }
                    .renderPassOptions()
                    .has(RenderPassOptionsVulkan::msaaSeedFromOffscreenTexture)
                {
                    debug_assert_eq!(views.len(), MSAA_COLOR_SEED_IDX);
                    views.push(self.m_msaaColorSeedImageView);
                    clears.push(vk::ClearValue::default());
                }
            }
        }
        let framebuffer = implementation.m_vk.makeFramebuffer(
            vk::FramebufferCreateInfo::default()
                .render_pass(render_pass_handle)
                .attachments(&views)
                .width(target.base().width())
                .height(target.base().height())
                .layers(1),
        );
        let begin = vk::RenderPassBeginInfo::default()
            .render_pass(render_pass_handle)
            .framebuffer(framebuffer.m_vkFramebuffer)
            .render_area(vkutil::rect2d(&self.m_drawBounds))
            .clear_values(&clears);
        unsafe {
            implementation.m_vk.ashDevice().cmd_begin_render_pass(
                command,
                &begin,
                vk::SubpassContents::INLINE,
            );
            let viewport = ViewportFromRect2D::new(vk::Rect2D {
                offset: vk::Offset2D::default(),
                extent: vk::Extent2D {
                    width: target.base().width(),
                    height: target.base().height(),
                },
            });
            implementation.m_vk.ashDevice().cmd_set_viewport(
                command,
                0,
                core::slice::from_raw_parts(viewport.as_ptr(), 1),
            );
        }
        self.m_renderPassOptions = options;
        self.m_scissor = scissor;
        self.m_patchCountInCurrentDrawPass = 0;
        pipeline_layout
    }

    fn restart(&mut self, load: LoadAction, options: RenderPassOptionsVulkan, scissor: IAABB) {
        let implementation = unsafe { &mut *self.m_impl };
        let desc = unsafe { &*self.m_desc };
        let command =
            vk::CommandBuffer::from_raw(desc.externalCommandBuffer.unwrap().as_ptr() as u64);
        unsafe { implementation.m_vk.ashDevice().cmd_end_render_pass(command) };
        let restarted = self.begin(load, options, scissor);
        debug_assert_eq!(restarted, self.m_pipelineLayout);
    }

    fn interruptIfNeeded(&mut self, next_count: u32, pending_count: u32) {
        let implementation = unsafe { &mut *self.m_impl };
        let desc = unsafe { &*self.m_desc };
        let max = implementation.m_workarounds.maxInstancesPerRenderPass;
        debug_assert!(next_count <= max);
        if desc.interlockMode == InterlockMode::rasterOrdering
            && self.m_patchCountInCurrentDrawPass + next_count > max
        {
            let mut options = self.m_renderPassOptions;
            debug_assert!(options.has(RenderPassOptionsVulkan::rasterOrderingInterruptible));
            debug_assert!(!options.has(RenderPassOptionsVulkan::manuallyResolved));
            options |= RenderPassOptionsVulkan::rasterOrderingResume;
            if pending_count <= max {
                options = RenderPassOptionsVulkan(
                    options.0 & !RenderPassOptionsVulkan::rasterOrderingInterruptible.0,
                );
            }
            self.restart(LoadAction::preserveRenderTarget, options, self.m_scissor);
        }
        self.m_patchCountInCurrentDrawPass += next_count;
    }
}

fn draw_list(desc: &FlushDescriptor) -> &crate::mechanical_port::source::renderer::include::rive::renderer::render_context_hpp::BlockAllocatedLinkedList<DrawBatch>{
    unsafe { desc.drawList.expect("Vulkan flush draw list").as_ref() }
}

fn write_buffer(
    vk_context: &VulkanContext,
    set: vk::DescriptorSet,
    binding: u32,
    descriptor_type: vk::DescriptorType,
    buffer: &super::vkutil_decl::Buffer,
    offset: u64,
    range: u64,
) {
    vk_context.updateBufferDescriptorSets(
        set,
        vk::WriteDescriptorSet::default()
            .dst_binding(binding)
            .descriptor_type(descriptor_type),
        &[vk::DescriptorBufferInfo {
            buffer: buffer.vkBuffer(),
            offset,
            range,
        }],
    );
}

fn write_image(
    vk_context: &VulkanContext,
    set: vk::DescriptorSet,
    binding: u32,
    descriptor_type: vk::DescriptorType,
    sampler: vk::Sampler,
    view: vk::ImageView,
    layout: vk::ImageLayout,
) {
    vk_context.updateImageDescriptorSets(
        set,
        vk::WriteDescriptorSet::default()
            .dst_binding(binding)
            .descriptor_type(descriptor_type),
        &[vk::DescriptorImageInfo {
            sampler,
            image_view: view,
            image_layout: layout,
        }],
    );
}

pub(crate) unsafe fn flush(implementation: &mut RenderContextVulkanImpl, desc: &FlushDescriptor) {
    let mut target = target_impl(desc);
    let mut draw_bounds = desc.renderTargetUpdateBounds;
    if draw_bounds.empty() {
        return;
    }
    let mut options = RenderPassOptionsVulkan::none;
    if desc.fixedFunctionColorOutput {
        options |= RenderPassOptionsVulkan::fixedFunctionColorOutput;
    }
    if desc.manuallyResolved {
        options |= RenderPassOptionsVulkan::manuallyResolved;
    } else if desc.interlockMode == InterlockMode::msaa {
        draw_bounds = to_gpu_bounds(target.base().bounds());
    }
    debug_assert!(
        desc.interlockMode != InterlockMode::msaa
            || desc.manuallyResolved
            || draw_bounds == to_gpu_bounds(target.base().bounds())
    );
    let command = vk::CommandBuffer::from_raw(
        desc.externalCommandBuffer
            .expect("Vulkan command buffer")
            .as_ptr() as u64,
    );
    unsafe { rcp_ref(&implementation.m_gaussianIntegralTexture) }
        .prepareForVertexOrFragmentShaderRead(command);
    unsafe { rcp_ref(&implementation.m_nullImageTexture) }.prepareForFragmentShaderRead(command);
    let mut pending_tess_patches = 0u32;
    for batch in draw_list(desc).iter() {
        if let Some(texture) = batch.imageTexture {
            unsafe {
                (&*texture.as_ptr().cast::<Texture2D>()).prepareForFragmentShaderRead(command)
            };
        }
        match batch.drawType {
            DrawType::midpointFanPatches
            | DrawType::midpointFanCenterAAPatches
            | DrawType::outerCurvePatches
            | DrawType::msaaOuterCubics
            | DrawType::msaaStrokes
            | DrawType::msaaMidpointFanBorrowedCoverage
            | DrawType::msaaDynamicMidpointFans
            | DrawType::msaaMidpointFans
            | DrawType::msaaMidpointFanStencilReset
            | DrawType::msaaMidpointFanPathsStencil
            | DrawType::msaaMidpointFanPathsCover => pending_tess_patches += batch.elementCount,
            _ => {}
        }
    }
    if desc.interlockMode == InterlockMode::rasterOrdering
        && pending_tess_patches > implementation.m_workarounds.maxInstancesPerRenderPass
    {
        debug_assert!(!implementation
            .m_plsTransientUsageFlags
            .contains(vk::ImageUsageFlags::TRANSIENT_ATTACHMENT));
        debug_assert!(!desc.manuallyResolved);
        options |= RenderPassOptionsVulkan::rasterOrderingInterruptible;
    }
    let mut allocator = DescriptorSetAllocator::new(implementation);
    let per_flush = allocator.m_perFlushDescriptorSet;
    write_buffer(
        &implementation.m_vk,
        per_flush,
        FLUSH_UNIFORM_BUFFER_IDX,
        vk::DescriptorType::UNIFORM_BUFFER,
        implementation.m_flushUniformBuffer.as_ref().unwrap(),
        desc.flushUniformDataOffsetInBytes as u64,
        size_of::<FlushUniforms>() as u64,
    );
    write_buffer(
        &implementation.m_vk,
        per_flush,
        PATH_BUFFER_IDX,
        vk::DescriptorType::STORAGE_BUFFER,
        implementation.m_pathBuffer.as_ref().unwrap(),
        desc.firstPath as u64 * size_of::<PathData>() as u64,
        vk::WHOLE_SIZE,
    );
    write_buffer(
        &implementation.m_vk,
        per_flush,
        PAINT_BUFFER_IDX,
        vk::DescriptorType::STORAGE_BUFFER,
        implementation.m_paintBuffer.as_ref().unwrap(),
        desc.firstPaint as u64 * size_of::<PaintData>() as u64,
        vk::WHOLE_SIZE,
    );
    write_buffer(
        &implementation.m_vk,
        per_flush,
        PAINT_AUX_BUFFER_IDX,
        vk::DescriptorType::STORAGE_BUFFER,
        implementation.m_paintAuxBuffer.as_ref().unwrap(),
        desc.firstPaintAux as u64 * size_of::<PaintAuxData>() as u64,
        vk::WHOLE_SIZE,
    );
    write_buffer(
        &implementation.m_vk,
        per_flush,
        CONTOUR_BUFFER_IDX,
        vk::DescriptorType::STORAGE_BUFFER,
        implementation.m_contourBuffer.as_ref().unwrap(),
        desc.firstContour as u64 * size_of::<ContourData>() as u64,
        vk::WHOLE_SIZE,
    );
    if desc.interlockMode == InterlockMode::clockwiseAtomic {
        if let Some(buffer) = implementation.m_coverageBuffer.as_ref() {
            write_buffer(
                &implementation.m_vk,
                per_flush,
                COVERAGE_BUFFER_IDX,
                vk::DescriptorType::STORAGE_BUFFER,
                buffer,
                0,
                vk::WHOLE_SIZE,
            );
        }
    }
    write_image(
        &implementation.m_vk,
        per_flush,
        TESS_VERTEX_TEXTURE_IDX,
        vk::DescriptorType::SAMPLED_IMAGE,
        vk::Sampler::null(),
        unsafe { rcp_ref(&implementation.m_tessTexture) }.vkImageView(),
        vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
    );
    let null_view = unsafe { rcp_ref(&implementation.m_nullImageTexture) }.vkImageView();
    let grad_view = if desc.gradSpanCount != 0 {
        unsafe { rcp_ref(&implementation.m_gradTexture) }.vkImageView()
    } else {
        null_view
    };
    write_image(
        &implementation.m_vk,
        per_flush,
        GRAD_TEXTURE_IDX,
        vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
        vk::Sampler::null(),
        grad_view,
        vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
    );
    write_image(
        &implementation.m_vk,
        per_flush,
        GAUSSIAN_INTEGRAL_TEXTURE_IDX,
        vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
        vk::Sampler::null(),
        unsafe { rcp_ref(&implementation.m_gaussianIntegralTexture) }.vkImageView(),
        vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
    );
    let atlas_view = if desc.featherAtlasFillBatchCount | desc.featherAtlasStrokeBatchCount != 0 {
        unsafe { rcp_ref(&implementation.m_featherAtlasTexture) }.vkImageView()
    } else {
        null_view
    };
    write_image(
        &implementation.m_vk,
        per_flush,
        FEATHER_ATLAS_TEXTURE_IDX,
        vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
        vk::Sampler::null(),
        atlas_view,
        vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
    );

    if desc.gradSpanCount > 0 {
        let area = vk::Rect2D {
            offset: vk::Offset2D::default(),
            extent: vk::Extent2D {
                width: kGradTextureWidth,
                height: desc.gradDataHeight,
            },
        };
        let pipeline = implementation.m_colorRampPipeline.as_mut().unwrap();
        pipeline.base.beginRenderPass(
            command,
            area,
            implementation
                .m_gradTextureFramebuffer
                .as_ref()
                .unwrap()
                .m_vkFramebuffer,
            false,
        );
        let viewport = ViewportFromRect2D::new(area);
        unsafe {
            implementation.m_vk.ashDevice().cmd_set_viewport(
                command,
                0,
                core::slice::from_raw_parts(viewport.as_ptr(), 1),
            );
            implementation
                .m_vk
                .ashDevice()
                .cmd_set_scissor(command, 0, &[area]);
            implementation.m_vk.ashDevice().cmd_bind_vertex_buffers(
                command,
                0,
                &[implementation.m_gradSpanBuffer.as_ref().unwrap().vkBuffer()],
                &[desc.firstGradSpan as u64 * size_of::<GradientSpan>() as u64],
            );
            implementation.m_vk.ashDevice().cmd_bind_descriptor_sets(
                command,
                vk::PipelineBindPoint::GRAPHICS,
                pipeline.m_pipelineLayout,
                PER_FLUSH_BINDINGS_SET,
                &[per_flush],
                &[],
            );
            implementation.m_vk.ashDevice().cmd_bind_pipeline(
                command,
                vk::PipelineBindPoint::GRAPHICS,
                pipeline.m_renderPipeline,
            );
        }
        for (count, first) in instance_chunks(
            desc.gradSpanCount,
            0,
            implementation.m_workarounds.maxInstancesPerRenderPass,
        ) {
            pipeline.base.interruptRenderPassIfNeeded(
                command,
                area,
                implementation
                    .m_gradTextureFramebuffer
                    .as_ref()
                    .unwrap()
                    .m_vkFramebuffer,
                count,
                implementation.m_workarounds,
            );
            unsafe {
                implementation.m_vk.ashDevice().cmd_draw(
                    command,
                    GRAD_SPAN_TRI_STRIP_VERTEX_COUNT,
                    count,
                    0,
                    first,
                )
            };
        }
        unsafe {
            implementation.m_vk.ashDevice().cmd_end_render_pass(command);
            rcp_ref(&implementation.m_gradTexture)
                .lastAccessMut()
                .layout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
        }
    }

    if desc.tessVertexSpanCount > 0 {
        unsafe { rcp_ref(&implementation.m_tessTexture) }.barrier(
            command,
            ImageAccess {
                pipelineStages: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                accessMask: vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
                layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            },
            ImageAccessAction::invalidateContents,
            vk::DependencyFlags::empty(),
        );
        let area = vk::Rect2D {
            offset: vk::Offset2D::default(),
            extent: vk::Extent2D {
                width: kTessTextureWidth as u32,
                height: desc.tessDataHeight,
            },
        };
        let pipeline = implementation.m_tessellatePipeline.as_mut().unwrap();
        pipeline.base.beginRenderPass(
            command,
            area,
            implementation
                .m_tessTextureFramebuffer
                .as_ref()
                .unwrap()
                .m_vkFramebuffer,
            false,
        );
        let viewport = ViewportFromRect2D::new(area);
        unsafe {
            implementation.m_vk.ashDevice().cmd_set_viewport(
                command,
                0,
                core::slice::from_raw_parts(viewport.as_ptr(), 1),
            );
            implementation
                .m_vk
                .ashDevice()
                .cmd_set_scissor(command, 0, &[area]);
            implementation.m_vk.ashDevice().cmd_bind_vertex_buffers(
                command,
                0,
                &[implementation.m_tessSpanBuffer.as_ref().unwrap().vkBuffer()],
                &[desc.firstTessVertexSpan as u64 * size_of::<TessVertexSpan>() as u64],
            );
            implementation.m_vk.ashDevice().cmd_bind_index_buffer(
                command,
                implementation
                    .m_tessSpanIndexBuffer
                    .as_ref()
                    .unwrap()
                    .vkBuffer(),
                0,
                vk::IndexType::UINT16,
            );
            implementation.m_vk.ashDevice().cmd_bind_descriptor_sets(
                command,
                vk::PipelineBindPoint::GRAPHICS,
                pipeline.m_pipelineLayout,
                PER_FLUSH_BINDINGS_SET,
                &[per_flush],
                &[],
            );
            implementation.m_vk.ashDevice().cmd_bind_pipeline(
                command,
                vk::PipelineBindPoint::GRAPHICS,
                pipeline.m_renderPipeline,
            );
        }
        for (count, first) in instance_chunks(
            desc.tessVertexSpanCount,
            0,
            implementation.m_workarounds.maxInstancesPerRenderPass,
        ) {
            pipeline.base.interruptRenderPassIfNeeded(
                command,
                area,
                implementation
                    .m_tessTextureFramebuffer
                    .as_ref()
                    .unwrap()
                    .m_vkFramebuffer,
                count,
                implementation.m_workarounds,
            );
            unsafe {
                implementation.m_vk.ashDevice().cmd_draw_indexed(
                    command,
                    kTessSpanIndices.len() as u32,
                    count,
                    0,
                    0,
                    first,
                )
            };
        }
        unsafe {
            implementation.m_vk.ashDevice().cmd_end_render_pass(command);
            rcp_ref(&implementation.m_tessTexture)
                .lastAccessMut()
                .layout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
        }
        if implementation
            .m_tesselationSyncIssueWorkaroundTexture
            .operator_bool()
        {
            let tess = unsafe { rcp_ref(&implementation.m_tessTexture) };
            tess.barrier(
                command,
                ImageAccess {
                    pipelineStages: vk::PipelineStageFlags::TRANSFER,
                    accessMask: vk::AccessFlags::TRANSFER_READ,
                    layout: vk::ImageLayout::GENERAL,
                },
                ImageAccessAction::preserveContents,
                vk::DependencyFlags::empty(),
            );
            let workaround =
                unsafe { rcp_ref(&implementation.m_tesselationSyncIssueWorkaroundTexture) };
            if workaround.lastAccess().layout != vk::ImageLayout::TRANSFER_DST_OPTIMAL {
                workaround.barrier(
                    command,
                    ImageAccess {
                        pipelineStages: vk::PipelineStageFlags::TRANSFER,
                        accessMask: vk::AccessFlags::TRANSFER_WRITE,
                        layout: vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                    },
                    ImageAccessAction::invalidateContents,
                    vk::DependencyFlags::empty(),
                );
            }
            implementation.m_vk.blitSubRect(
                command,
                tess.vkImage(),
                vk::ImageLayout::GENERAL,
                workaround.vkImage(),
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                &make_wh(workaround.width() as i32, workaround.height() as i32),
            );
        }
    }
    unsafe { rcp_ref(&implementation.m_tessTexture) }.barrier(
        command,
        ImageAccess {
            pipelineStages: vk::PipelineStageFlags::VERTEX_SHADER,
            accessMask: vk::AccessFlags::SHADER_READ,
            layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        },
        ImageAccessAction::preserveContents,
        vk::DependencyFlags::empty(),
    );

    if desc.featherAtlasFillBatchCount | desc.featherAtlasStrokeBatchCount != 0 {
        let area = vk::Rect2D {
            offset: vk::Offset2D::default(),
            extent: vk::Extent2D {
                width: desc.featherAtlasContentWidth as u32,
                height: desc.featherAtlasContentHeight as u32,
            },
        };
        let pipeline = implementation.m_featherAtlasPipeline.as_mut().unwrap();
        pipeline.base.beginRenderPass(
            command,
            area,
            implementation
                .m_featherAtlasFramebuffer
                .as_ref()
                .unwrap()
                .m_vkFramebuffer,
            false,
        );
        let viewport = ViewportFromRect2D::new(area);
        unsafe {
            implementation.m_vk.ashDevice().cmd_set_viewport(
                command,
                0,
                core::slice::from_raw_parts(viewport.as_ptr(), 1),
            );
            implementation.m_vk.ashDevice().cmd_bind_vertex_buffers(
                command,
                0,
                &[implementation
                    .m_pathPatchVertexBuffer
                    .as_ref()
                    .unwrap()
                    .vkBuffer()],
                &[0],
            );
            implementation.m_vk.ashDevice().cmd_bind_index_buffer(
                command,
                implementation
                    .m_pathPatchIndexBuffer
                    .as_ref()
                    .unwrap()
                    .vkBuffer(),
                0,
                vk::IndexType::UINT16,
            );
            implementation.m_vk.ashDevice().cmd_bind_descriptor_sets(
                command,
                vk::PipelineBindPoint::GRAPHICS,
                pipeline.m_pipelineLayout,
                PER_FLUSH_BINDINGS_SET,
                &[per_flush],
                &[],
            );
        }
        let fills = if desc.featherAtlasFillBatchCount == 0 {
            &[][..]
        } else {
            unsafe {
                core::slice::from_raw_parts(
                    desc.featherAtlasFillBatches.unwrap().as_ptr(),
                    desc.featherAtlasFillBatchCount,
                )
            }
        };
        if !fills.is_empty() {
            unsafe {
                implementation.m_vk.ashDevice().cmd_bind_pipeline(
                    command,
                    vk::PipelineBindPoint::GRAPHICS,
                    pipeline.m_fillPipeline,
                )
            };
            for batch in fills {
                let scissor = vk::Rect2D {
                    offset: vk::Offset2D {
                        x: batch.scissor.left as i32,
                        y: batch.scissor.top as i32,
                    },
                    extent: vk::Extent2D {
                        width: (batch.scissor.right - batch.scissor.left) as u32,
                        height: (batch.scissor.bottom - batch.scissor.top) as u32,
                    },
                };
                unsafe {
                    implementation
                        .m_vk
                        .ashDevice()
                        .cmd_set_scissor(command, 0, &[scissor])
                };
                for (count, first) in instance_chunks(
                    batch.patchCount,
                    batch.basePatch,
                    implementation.m_workarounds.maxInstancesPerRenderPass,
                ) {
                    pipeline.base.interruptRenderPassIfNeeded(
                        command,
                        area,
                        implementation
                            .m_featherAtlasFramebuffer
                            .as_ref()
                            .unwrap()
                            .m_vkFramebuffer,
                        count,
                        implementation.m_workarounds,
                    );
                    unsafe {
                        implementation.m_vk.ashDevice().cmd_draw_indexed(
                            command,
                            kMidpointFanCenterAAPatchIndexCount,
                            count,
                            kMidpointFanCenterAAPatchBaseIndex,
                            0,
                            first,
                        )
                    };
                }
            }
        }
        let strokes = if desc.featherAtlasStrokeBatchCount == 0 {
            &[][..]
        } else {
            unsafe {
                core::slice::from_raw_parts(
                    desc.featherAtlasStrokeBatches.unwrap().as_ptr(),
                    desc.featherAtlasStrokeBatchCount,
                )
            }
        };
        if !strokes.is_empty() {
            unsafe {
                implementation.m_vk.ashDevice().cmd_bind_pipeline(
                    command,
                    vk::PipelineBindPoint::GRAPHICS,
                    pipeline.m_strokePipeline,
                )
            };
            for batch in strokes {
                let scissor = vk::Rect2D {
                    offset: vk::Offset2D {
                        x: batch.scissor.left as i32,
                        y: batch.scissor.top as i32,
                    },
                    extent: vk::Extent2D {
                        width: (batch.scissor.right - batch.scissor.left) as u32,
                        height: (batch.scissor.bottom - batch.scissor.top) as u32,
                    },
                };
                unsafe {
                    implementation
                        .m_vk
                        .ashDevice()
                        .cmd_set_scissor(command, 0, &[scissor])
                };
                for (count, first) in instance_chunks(
                    batch.patchCount,
                    batch.basePatch,
                    implementation.m_workarounds.maxInstancesPerRenderPass,
                ) {
                    pipeline.base.interruptRenderPassIfNeeded(
                        command,
                        area,
                        implementation
                            .m_featherAtlasFramebuffer
                            .as_ref()
                            .unwrap()
                            .m_vkFramebuffer,
                        count,
                        implementation.m_workarounds,
                    );
                    unsafe {
                        implementation.m_vk.ashDevice().cmd_draw_indexed(
                            command,
                            kMidpointFanPatchBorderIndexCount,
                            count,
                            kMidpointFanPatchBaseIndex,
                            0,
                            first,
                        )
                    };
                }
            }
        }
        unsafe {
            implementation.m_vk.ashDevice().cmd_end_render_pass(command);
            rcp_ref(&implementation.m_featherAtlasTexture)
                .lastAccessMut()
                .layout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
        }
    }

    let color_load_access = ImageAccess {
        pipelineStages: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
        accessMask: vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
        layout: if options.has(RenderPassOptionsVulkan::fixedFunctionColorOutput) {
            vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL
        } else {
            vk::ImageLayout::GENERAL
        },
    };
    let full_target = draw_bounds.contains(&IAABB {
        left: 0,
        top: 0,
        right: target.base().width() as i32,
        bottom: target.base().height() as i32,
    });
    let target_action = if full_target && desc.colorLoadAction != LoadAction::preserveRenderTarget {
        ImageAccessAction::invalidateContents
    } else {
        ImageAccessAction::preserveContents
    };
    let backing = implementation
        .m_pipelineManager
        .as_ref()
        .unwrap()
        .plsBackingType(desc.interlockMode);
    let mut color_view = vk::ImageView::null();
    let mut color_offscreen = false;
    let mut msaa_resolve_view = vk::ImageView::null();
    let mut msaa_seed_view = vk::ImageView::null();
    if desc.interlockMode == InterlockMode::msaa {
        color_view = unsafe {
            (&*super::render_target_vulkan_impl::msaaColorTexture(&mut target)).vkImageView()
        };
        if desc.colorLoadAction == LoadAction::preserveRenderTarget {
            let copied = super::render_target_vulkan_impl::copyTargetImageToOffscreenColorTexture(
                &mut target,
                command,
                ImageAccess {
                    pipelineStages: vk::PipelineStageFlags::FRAGMENT_SHADER,
                    accessMask: vk::AccessFlags::INPUT_ATTACHMENT_READ,
                    layout: vk::ImageLayout::GENERAL,
                },
                &to_target_bounds(&draw_bounds),
            );
            msaa_seed_view = unsafe { (&*copied).vkImageView() };
            options |= RenderPassOptionsVulkan::msaaSeedFromOffscreenTexture;
        }
        msaa_resolve_view = target.accessTargetImageView(
            command,
            ImageAccess {
                pipelineStages: vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                accessMask: vk::AccessFlags::COLOR_ATTACHMENT_WRITE,
                layout: vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            },
            if full_target {
                ImageAccessAction::invalidateContents
            } else {
                ImageAccessAction::preserveContents
            },
        );
    } else if options.has(RenderPassOptionsVulkan::fixedFunctionColorOutput)
        || ((desc.interlockMode == InterlockMode::rasterOrdering
            || desc.interlockMode == InterlockMode::atomics)
            && target
                .base()
                .m_targetUsageFlags
                .contains(vk::ImageUsageFlags::INPUT_ATTACHMENT))
    {
        color_view = target.accessTargetImageView(command, color_load_access, target_action);
    } else if backing == PLSBackingType::storageTexture {
        let storage_access = ImageAccess {
            pipelineStages: vk::PipelineStageFlags::FRAGMENT_SHADER,
            accessMask: vk::AccessFlags::SHADER_READ | vk::AccessFlags::SHADER_WRITE,
            layout: vk::ImageLayout::GENERAL,
        };
        if target
            .base()
            .m_targetUsageFlags
            .contains(vk::ImageUsageFlags::STORAGE)
            && target.base().m_framebufferFormat == vk::Format::R8G8B8A8_UNORM
        {
            color_view = if desc.colorLoadAction == LoadAction::clear {
                super::render_target_vulkan_impl::clearTargetImageView(
                    &mut target,
                    command,
                    desc.colorClearValue,
                    storage_access,
                )
            } else {
                target.accessTargetImageView(command, storage_access, target_action)
            };
        } else {
            color_view = match desc.colorLoadAction {
                LoadAction::clear => unsafe {
                    (&*clearPLSOffscreenColorTexture(
                        implementation,
                        command,
                        desc.colorClearValue,
                        storage_access,
                    ))
                        .vkImageView()
                },
                LoadAction::preserveRenderTarget => unsafe {
                    (&*copyRenderTargetToPLSOffscreenColorTexture(
                        implementation,
                        command,
                        &mut target,
                        &draw_bounds,
                        storage_access,
                    ))
                        .vkImageView()
                },
                LoadAction::dontCare => unsafe {
                    (&*accessPLSOffscreenColorTexture(
                        implementation,
                        command,
                        storage_access,
                        ImageAccessAction::invalidateContents,
                    ))
                        .vkImageView()
                },
            };
            color_offscreen = true;
        }
    } else {
        color_view = if desc.colorLoadAction == LoadAction::preserveRenderTarget {
            unsafe {
                (&*super::render_target_vulkan_impl::copyTargetImageToOffscreenColorTexture(
                    &mut target,
                    command,
                    color_load_access,
                    &to_target_bounds(&draw_bounds),
                ))
                    .vkImageView()
            }
        } else {
            unsafe {
                (&*super::render_target_vulkan_impl::accessOffscreenColorTexture(
                    &mut target,
                    command,
                    color_load_access,
                    ImageAccessAction::invalidateContents,
                ))
                    .vkImageView()
            }
        };
        if desc.interlockMode == InterlockMode::atomics {
            options |= RenderPassOptionsVulkan::atomicCoalescedResolveAndTransfer;
        }
        color_offscreen = true;
    }

    if desc.interlockMode == InterlockMode::clockwise
        || desc.interlockMode == InterlockMode::atomics
    {
        let storage_image = if desc.interlockMode == InterlockMode::atomics {
            unsafe { rcp_ref(&implementation.m_plsAtomicCoverageTexture) }.vkImage()
        } else {
            plsTransientImageArray(implementation).vkImage()
        };
        implementation.m_vk.imageMemoryBarrier(
            command,
            vk::PipelineStageFlags::FRAGMENT_SHADER,
            vk::PipelineStageFlags::TRANSFER,
            vk::DependencyFlags::empty(),
            vk::ImageMemoryBarrier::default()
                .src_access_mask(vk::AccessFlags::SHADER_READ | vk::AccessFlags::SHADER_WRITE)
                .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                .old_layout(vk::ImageLayout::UNDEFINED)
                .new_layout(vk::ImageLayout::GENERAL)
                .image(storage_image),
        );
        let clear_range = vk::ImageSubresourceRange {
            aspect_mask: vk::ImageAspectFlags::COLOR,
            level_count: 1,
            layer_count: if desc.interlockMode == InterlockMode::atomics {
                1
            } else if desc.combinedShaderFeatures.0 & ShaderFeatures::ENABLE_CLIPPING.0 != 0 {
                2
            } else {
                1
            },
            ..Default::default()
        };
        let clear = if desc.interlockMode == InterlockMode::atomics {
            vkutil::color_clear_r32ui(desc.coverageClearValue)
        } else {
            debug_assert_eq!(desc.coverageClearValue, 0);
            vk::ClearColorValue::default()
        };
        unsafe {
            implementation.m_vk.ashDevice().cmd_clear_color_image(
                command,
                storage_image,
                vk::ImageLayout::GENERAL,
                &clear,
                &[clear_range],
            )
        };
        implementation.m_vk.imageMemoryBarrier(
            command,
            vk::PipelineStageFlags::TRANSFER,
            vk::PipelineStageFlags::FRAGMENT_SHADER,
            vk::DependencyFlags::empty(),
            vk::ImageMemoryBarrier::default()
                .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                .dst_access_mask(vk::AccessFlags::SHADER_READ | vk::AccessFlags::SHADER_WRITE)
                .old_layout(vk::ImageLayout::GENERAL)
                .new_layout(vk::ImageLayout::GENERAL)
                .image(storage_image),
        );
    } else {
        debug_assert!(
            desc.interlockMode != InterlockMode::rasterOrdering
                || !implementation.base.m_platformFeatures.supportsClockwiseMode
        );
    }
    if (desc.interlockMode == InterlockMode::clockwise
        || desc.interlockMode == InterlockMode::clockwiseAtomic)
        && !options.has(RenderPassOptionsVulkan::fixedFunctionColorOutput)
    {
        unsafe {
            (&*plsBlendStorageTexture_RGB10_A2(implementation)).barrier(
                command,
                ImageAccess {
                    pipelineStages: vk::PipelineStageFlags::FRAGMENT_SHADER,
                    accessMask: vk::AccessFlags::SHADER_READ | vk::AccessFlags::SHADER_READ,
                    layout: vk::ImageLayout::GENERAL,
                },
                ImageAccessAction::invalidateContents,
                vk::DependencyFlags::empty(),
            )
        };
    }
    if desc.interlockMode == InterlockMode::clockwiseAtomic {
        let mut last_stage = vk::PipelineStageFlags::FRAGMENT_SHADER;
        let mut last_access = vk::AccessFlags::SHADER_WRITE;
        if desc.needsCoverageBufferClear {
            let coverage = implementation
                .m_coverageBuffer
                .as_ref()
                .expect("coverage buffer clear");
            implementation.m_vk.bufferMemoryBarrier(
                command,
                last_stage,
                vk::PipelineStageFlags::TRANSFER,
                vk::DependencyFlags::empty(),
                vk::BufferMemoryBarrier::default()
                    .src_access_mask(last_access)
                    .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                    .buffer(coverage.vkBuffer()),
            );
            unsafe {
                implementation.m_vk.ashDevice().cmd_fill_buffer(
                    command,
                    coverage.vkBuffer(),
                    0,
                    coverage.info().size,
                    0,
                )
            };
            last_stage = vk::PipelineStageFlags::TRANSFER;
            last_access = vk::AccessFlags::TRANSFER_WRITE;
        }
        if let Some(coverage) = implementation.m_coverageBuffer.as_ref() {
            implementation.m_vk.bufferMemoryBarrier(
                command,
                last_stage,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
                vk::DependencyFlags::empty(),
                vk::BufferMemoryBarrier::default()
                    .src_access_mask(last_access)
                    .dst_access_mask(vk::AccessFlags::SHADER_READ)
                    .buffer(coverage.vkBuffer()),
            );
        }
    }
    let mut tile_width = draw_bounds.width();
    let mut tile_height = draw_bounds.height();
    if desc.virtualTileWidth != 0
        && desc.virtualTileHeight != 0
        && desc.interlockMode != InterlockMode::msaa
    {
        tile_width = desc.virtualTileWidth as i32;
        tile_height = desc.virtualTileHeight as i32;
    }
    let initial_scissor = make_wh(tile_width, tile_height)
        .offset(draw_bounds.left, draw_bounds.top)
        .intersect(&draw_bounds);
    let mut draw_pass = DrawRenderPass::new(
        implementation,
        desc,
        desc.colorLoadAction,
        draw_bounds,
        color_view,
        msaa_seed_view,
        msaa_resolve_view,
        options,
        initial_scissor,
    );
    let mut pls_set = vk::DescriptorSet::null();
    if draw_pass.pipelineLayout().plsLayout() != vk::DescriptorSetLayout::null() {
        let descriptor_type = if backing == PLSBackingType::storageTexture {
            vk::DescriptorType::STORAGE_IMAGE
        } else {
            vk::DescriptorType::INPUT_ATTACHMENT
        };
        pls_set = allocator.allocateDescriptorSet(draw_pass.pipelineLayout().plsLayout());
        if !options.has(RenderPassOptionsVulkan::fixedFunctionColorOutput) {
            write_image(
                &implementation.m_vk,
                pls_set,
                COLOR_PLANE_IDX as u32,
                descriptor_type,
                vk::Sampler::null(),
                color_view,
                vk::ImageLayout::GENERAL,
            );
        }
        if desc.interlockMode != InterlockMode::msaa {
            let view = if desc.interlockMode == InterlockMode::atomics {
                unsafe { (&*plsTransientScratchColorTexture(implementation)).vkImageView() }
            } else if desc.interlockMode == InterlockMode::clockwiseAtomic {
                unsafe { (&*plsTransientClipTexture_R16F(implementation)).vkImageView() }
            } else {
                plsTransientClipView(implementation)
            };
            write_image(
                &implementation.m_vk,
                pls_set,
                CLIP_PLANE_IDX as u32,
                descriptor_type,
                vk::Sampler::null(),
                view,
                vk::ImageLayout::GENERAL,
            );
        }
        if desc.interlockMode == InterlockMode::rasterOrdering
            || ((desc.interlockMode == InterlockMode::clockwise
                || desc.interlockMode == InterlockMode::clockwiseAtomic)
                && !options.has(RenderPassOptionsVulkan::fixedFunctionColorOutput))
        {
            let view = if desc.interlockMode == InterlockMode::clockwise
                || desc.interlockMode == InterlockMode::clockwiseAtomic
            {
                unsafe { (&*plsBlendStorageTexture_RGB10_A2(implementation)).vkImageView() }
            } else {
                unsafe { (&*plsTransientScratchColorTexture(implementation)).vkImageView() }
            };
            let ty = if desc.interlockMode == InterlockMode::clockwiseAtomic {
                vk::DescriptorType::STORAGE_IMAGE
            } else {
                descriptor_type
            };
            write_image(
                &implementation.m_vk,
                pls_set,
                SCRATCH_COLOR_PLANE_IDX as u32,
                ty,
                vk::Sampler::null(),
                view,
                vk::ImageLayout::GENERAL,
            );
        }
        if desc.interlockMode == InterlockMode::rasterOrdering
            || desc.interlockMode == InterlockMode::atomics
            || desc.interlockMode == InterlockMode::clockwise
        {
            let view = if desc.interlockMode == InterlockMode::atomics {
                unsafe { rcp_ref(&implementation.m_plsAtomicCoverageTexture) }.vkImageView()
            } else {
                plsTransientCoverageView(implementation)
            };
            let ty = if desc.interlockMode == InterlockMode::atomics {
                vk::DescriptorType::STORAGE_IMAGE
            } else {
                descriptor_type
            };
            write_image(
                &implementation.m_vk,
                pls_set,
                COVERAGE_PLANE_IDX as u32,
                ty,
                vk::Sampler::null(),
                view,
                vk::ImageLayout::GENERAL,
            );
        }
        if msaa_seed_view != vk::ImageView::null() {
            write_image(
                &implementation.m_vk,
                pls_set,
                MSAA_COLOR_SEED_IDX as u32,
                vk::DescriptorType::INPUT_ATTACHMENT,
                vk::Sampler::null(),
                msaa_seed_view,
                vk::ImageLayout::GENERAL,
            );
        }
    }
    let sets = [
        per_flush,
        implementation
            .m_pipelineManager
            .as_ref()
            .unwrap()
            .nullImageDescriptorSet(),
        pls_set,
    ];
    let set_count = if draw_pass.pipelineLayout().plsLayout() != vk::DescriptorSetLayout::null() {
        VULKAN_BINDINGS_SET_COUNT
    } else {
        VULKAN_BINDINGS_SET_COUNT - 1
    };
    unsafe {
        implementation.m_vk.ashDevice().cmd_bind_descriptor_sets(
            command,
            vk::PipelineBindPoint::GRAPHICS,
            draw_pass.pipelineLayout().vkPipelineLayout(),
            PER_FLUSH_BINDINGS_SET,
            &sets[..set_count as usize],
            &[],
        )
    };
    let mut y = draw_bounds.top;
    while y < draw_bounds.bottom {
        let mut x = draw_bounds.left;
        while x < draw_bounds.right {
            if x > draw_bounds.left || y > draw_bounds.top {
                let scissor = make_wh(tile_width, tile_height)
                    .offset(x, y)
                    .intersect(&draw_bounds);
                draw_pass.restart(
                    if desc.colorLoadAction == LoadAction::dontCare {
                        LoadAction::dontCare
                    } else {
                        LoadAction::preserveRenderTarget
                    },
                    options,
                    scissor,
                );
            }
            submitDrawList(
                implementation,
                desc,
                &mut allocator,
                &mut draw_pass,
                pending_tess_patches,
            );
            x += tile_width;
        }
        y += tile_height;
    }
    unsafe { implementation.m_vk.ashDevice().cmd_end_render_pass(command) };
    if color_offscreen
        && !(options.has(RenderPassOptionsVulkan::manuallyResolved)
            || options.has(RenderPassOptionsVulkan::atomicCoalescedResolveAndTransfer))
    {
        debug_assert!(
            desc.interlockMode != InterlockMode::atomics
                && desc.interlockMode != InterlockMode::msaa
        );
        let copy_access = ImageAccess {
            pipelineStages: vk::PipelineStageFlags::TRANSFER,
            accessMask: vk::AccessFlags::TRANSFER_READ,
            layout: vk::ImageLayout::GENERAL,
        };
        let source = if backing == PLSBackingType::storageTexture {
            unsafe {
                (&*accessPLSOffscreenColorTexture(
                    implementation,
                    command,
                    copy_access,
                    ImageAccessAction::preserveContents,
                ))
                    .vkImage()
            }
        } else {
            unsafe {
                (&*super::render_target_vulkan_impl::accessOffscreenColorTexture(
                    &mut target,
                    command,
                    copy_access,
                    ImageAccessAction::preserveContents,
                ))
                    .vkImage()
            }
        };
        let destination = target.accessTargetImage(
            command,
            ImageAccess {
                pipelineStages: vk::PipelineStageFlags::TRANSFER,
                accessMask: vk::AccessFlags::TRANSFER_WRITE,
                layout: vk::ImageLayout::GENERAL,
            },
            ImageAccessAction::invalidateContents,
        );
        implementation.m_vk.blitSubRect(
            command,
            source,
            vk::ImageLayout::GENERAL,
            destination,
            vk::ImageLayout::GENERAL,
            &draw_bounds,
        );
    }
}

struct PipelineBinder<'a> {
    vk: &'a VulkanContext,
    pipeline: vk::Pipeline,
    scissor: IAABB,
    have_scissor: bool,
}

impl<'a> PipelineBinder<'a> {
    fn new(vk: &'a VulkanContext) -> Self {
        Self {
            vk,
            pipeline: vk::Pipeline::null(),
            scissor: IAABB::default(),
            have_scissor: false,
        }
    }
    fn bind(&mut self, command: vk::CommandBuffer, pipeline: vk::Pipeline, scissor: IAABB) {
        unsafe {
            if pipeline != self.pipeline {
                self.vk.ashDevice().cmd_bind_pipeline(
                    command,
                    vk::PipelineBindPoint::GRAPHICS,
                    pipeline,
                );
                self.pipeline = pipeline;
            }
            if !self.have_scissor || scissor != self.scissor {
                self.vk
                    .ashDevice()
                    .cmd_set_scissor(command, 0, &[vkutil::rect2d(&scissor)]);
                self.scissor = scissor;
                self.have_scissor = true;
            }
        }
    }
    fn setDynamicState(&self, command: vk::CommandBuffer, state: &PipelineState) {
        unsafe {
            (self
                .vk
                .CmdSetDepthWriteEnable
                .expect("Vulkan 1.3 depth write command"))(
                command, state.depthWriteEnabled.into()
            );
            self.vk.ashDevice().cmd_set_stencil_compare_mask(
                command,
                vk::StencilFaceFlags::FRONT_AND_BACK,
                state.stencilCompareMask as u32,
            );
            self.vk.ashDevice().cmd_set_stencil_write_mask(
                command,
                vk::StencilFaceFlags::FRONT_AND_BACK,
                state.stencilWriteMask as u32,
            );
            let set_ops = |faces: vk::StencilFaceFlags, ops: &StencilFaceOps| {
                self.vk.ashDevice().cmd_set_stencil_op(
                    command,
                    faces,
                    vkutil::vkStencilOp(ops.stencilFailOp),
                    vkutil::vkStencilOp(ops.depthStencilPassOp),
                    vkutil::vkStencilOp(ops.depthFailOp),
                    vkutil::vkCompareOp(ops.compareOp),
                );
            };
            set_ops(
                if state.stencilDoubleSided {
                    vk::StencilFaceFlags::FRONT
                } else {
                    vk::StencilFaceFlags::FRONT_AND_BACK
                },
                &state.stencilFrontOps,
            );
            if state.stencilDoubleSided {
                set_ops(vk::StencilFaceFlags::BACK, &state.stencilBackOps);
            }
            (self.vk.CmdSetCullMode.expect("Vulkan 1.3 cull command"))(
                command,
                vkutil::vkCullMode(state.cullFace),
            );
            let color_write: vk::Bool32 = state.colorWriteEnabled.into();
            (self
                .vk
                .CmdSetColorWriteEnableEXT
                .expect("color write enable command"))(command, 1, &color_write);
        }
    }
}

fn batch_scissor(batch: &DrawBatch) -> Option<IAABB> {
    batch.scissorRect.map(|value| IAABB {
        left: value.left as i32,
        top: value.top as i32,
        right: value.right as i32,
        bottom: value.bottom as i32,
    })
}

fn submitDrawList(
    implementation: &mut RenderContextVulkanImpl,
    desc: &FlushDescriptor,
    allocator: &mut DescriptorSetAllocator,
    draw_pass: &mut DrawRenderPass,
    mut pending_tess_patches: u32,
) {
    let command = vk::CommandBuffer::from_raw(desc.externalCommandBuffer.unwrap().as_ptr() as u64);
    let target = target_impl(desc);
    let render_pass_scissor = draw_pass.m_scissor;
    let mut binder = PipelineBinder::new(&implementation.m_vk);
    for batch in draw_list(desc).iter() {
        debug_assert!(batch.elementCount > 0);
        if let Some(texture_ptr) = batch.imageTexture {
            let texture = unsafe { &*texture_ptr.as_ptr().cast::<Texture2D>() };
            let product_image_sampler = product_sampler(batch.imageSampler);
            let mut set = texture.getCachedDescriptorSet(
                implementation.m_vk.currentFrameNumber(),
                product_image_sampler,
            );
            if set == vk::DescriptorSet::null() {
                set = allocator.allocatePerDrawDescriptorSet();
                write_image(
                    &implementation.m_vk,
                    set,
                    IMAGE_TEXTURE_IDX,
                    vk::DescriptorType::COMBINED_IMAGE_SAMPLER,
                    implementation
                        .m_pipelineManager
                        .as_ref()
                        .unwrap()
                        .imageSampler(batch.imageSampler.asKey() as u32),
                    texture.vkImageView(),
                    vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
                );
                texture.updateCachedDescriptorSet(
                    set,
                    implementation.m_vk.currentFrameNumber(),
                    product_image_sampler,
                );
            }
            unsafe {
                implementation.m_vk.ashDevice().cmd_bind_descriptor_sets(
                    command,
                    vk::PipelineBindPoint::GRAPHICS,
                    draw_pass.pipelineLayout().vkPipelineLayout(),
                    PER_DRAW_BINDINGS_SET,
                    &[set],
                    &[],
                )
            };
        }
        let shader_features = if desc.interlockMode == InterlockMode::atomics {
            desc.combinedShaderFeatures
        } else {
            batch.shaderFeatures
        };
        let mut misc = batch.shaderMiscFlags;
        if draw_pass
            .m_renderPassOptions
            .has(RenderPassOptionsVulkan::atomicCoalescedResolveAndTransfer)
            && batch.drawType == DrawType::renderPassResolve
        {
            misc |= ShaderMiscFlags::coalescedResolveAndTransfer;
        }
        let draw_options = if desc.wireframe && implementation.m_vk.features.fillModeNonSolid {
            DrawPipelineOptions::wireframe
        } else {
            DrawPipelineOptions::none
        };
        let next_subpass_mask = BarrierFlags::plsAtomicPreResolve.0
            | BarrierFlags::msaaPostInit.0
            | BarrierFlags::preManualResolve.0
            | BarrierFlags::clockwiseBorrowedCoverage.0;
        if batch.barriers.0 & next_subpass_mask != 0 {
            unsafe {
                implementation
                    .m_vk
                    .ashDevice()
                    .cmd_next_subpass(command, vk::SubpassContents::INLINE)
            };
        }
        if batch.barriers.0 & (BarrierFlags::plsAtomic.0 | BarrierFlags::dstBlend.0) != 0 {
            implementation.m_vk.memoryBarrier(
                command,
                vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
                vk::DependencyFlags::BY_REGION,
                vk::MemoryBarrier::default()
                    .src_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
                    .dst_access_mask(vk::AccessFlags::INPUT_ATTACHMENT_READ),
            );
        }
        let pipeline = implementation
            .m_pipelineManager
            .as_ref()
            .unwrap()
            .tryGetPipeline(
                &PipelineProps {
                    drawType: batch.drawType,
                    shaderFeatures: shader_features,
                    interlockMode: desc.interlockMode,
                    shaderMiscFlags: misc,
                    drawContents: batch.drawContents,
                    blendMode: batch.firstBlendMode,
                    drawPipelineOptions: draw_options,
                    renderPassOptions: draw_pass.m_renderPassOptions,
                    renderTargetFormat: target.base().m_framebufferFormat,
                    colorLoadAction: desc.colorLoadAction,
                    #[cfg(feature = "with-rive-tools")]
                    synthesizedFailureType: desc.synthesizedFailureType,
                },
                &implementation.base.m_platformFeatures,
            );
        if let Some(pipeline) = pipeline {
            let desired = if (batch.drawType == DrawType::renderPassInitialize
                || batch.drawType == DrawType::renderPassResolve)
                && desc.interlockMode == InterlockMode::clockwiseAtomic
            {
                IAABB {
                    left: render_pass_scissor.left,
                    top: render_pass_scissor.top,
                    right: render_pass_scissor.left + 1,
                    bottom: render_pass_scissor.top + 1,
                }
            } else if let Some(scissor) = batch_scissor(batch) {
                render_pass_scissor.intersectOrEmpty(&scissor)
            } else {
                render_pass_scissor
            };
            binder.bind(command, pipeline.m_vkPipeline, desired);
        }
        match batch.drawType {
            DrawType::midpointFanPatches
            | DrawType::midpointFanCenterAAPatches
            | DrawType::outerCurvePatches
            | DrawType::msaaOuterCubics
            | DrawType::msaaStrokes
            | DrawType::msaaMidpointFanBorrowedCoverage
            | DrawType::msaaMidpointFans
            | DrawType::msaaMidpointFanStencilReset
            | DrawType::msaaMidpointFanPathsStencil
            | DrawType::msaaMidpointFanPathsCover => {
                unsafe {
                    implementation.m_vk.ashDevice().cmd_bind_vertex_buffers(
                        command,
                        0,
                        &[implementation
                            .m_pathPatchVertexBuffer
                            .as_ref()
                            .unwrap()
                            .vkBuffer()],
                        &[0],
                    );
                    implementation.m_vk.ashDevice().cmd_bind_index_buffer(
                        command,
                        implementation
                            .m_pathPatchIndexBuffer
                            .as_ref()
                            .unwrap()
                            .vkBuffer(),
                        0,
                        vk::IndexType::UINT16,
                    );
                }
                for (count, first) in instance_chunks(
                    batch.elementCount,
                    batch.baseElement,
                    implementation.m_workarounds.maxInstancesPerRenderPass,
                ) {
                    draw_pass.interruptIfNeeded(count, pending_tess_patches);
                    pending_tess_patches -= count;
                    if let Some(pipeline) = pipeline {
                        unsafe {
                            implementation.m_vk.ashDevice().cmd_draw_indexed(
                                command,
                                batch.indexCountPerInstance,
                                count,
                                batch.baseIndex,
                                0,
                                first,
                            )
                        };
                    }
                }
            }
            DrawType::msaaDynamicMidpointFans => {
                pending_tess_patches -= batch.elementCount;
                if pipeline.is_none() {
                    continue;
                }
                debug_assert!(!implementation
                    .m_workarounds
                    .needsInterruptibleRenderPasses());
                unsafe {
                    implementation.m_vk.ashDevice().cmd_bind_vertex_buffers(
                        command,
                        0,
                        &[implementation
                            .m_pathPatchVertexBuffer
                            .as_ref()
                            .unwrap()
                            .vkBuffer()],
                        &[0],
                    );
                    implementation.m_vk.ashDevice().cmd_bind_index_buffer(
                        command,
                        implementation
                            .m_pathPatchIndexBuffer
                            .as_ref()
                            .unwrap()
                            .vkBuffer(),
                        0,
                        vk::IndexType::UINT16,
                    );
                }
                for pass in [
                    DrawType::msaaMidpointFanBorrowedCoverage,
                    DrawType::msaaMidpointFans,
                    DrawType::msaaMidpointFanStencilReset,
                ] {
                    let state =
                        crate::mechanical_port::source::renderer::src::gpu_cpp::get_pipeline_state(
                            pass,
                            desc.interlockMode,
                            batch.shaderMiscFlags,
                            batch.drawContents,
                            desc.fixedFunctionColorOutput,
                            batch.firstBlendMode,
                            &implementation.base.m_platformFeatures,
                        );
                    binder.setDynamicState(command, &state);
                    unsafe {
                        implementation.m_vk.ashDevice().cmd_draw_indexed(
                            command,
                            batch.indexCountPerInstance,
                            batch.elementCount,
                            batch.baseIndex,
                            0,
                            batch.baseElement,
                        )
                    };
                }
            }
            DrawType::clipReset | DrawType::interiorTriangulation | DrawType::featherAtlasBlit => {
                unsafe {
                    implementation.m_vk.ashDevice().cmd_bind_vertex_buffers(
                        command,
                        0,
                        &[implementation.m_triangleBuffer.as_ref().unwrap().vkBuffer()],
                        &[0],
                    )
                };
                if pipeline.is_some() {
                    unsafe {
                        implementation.m_vk.ashDevice().cmd_draw(
                            command,
                            batch.elementCount,
                            1,
                            batch.baseElement,
                            0,
                        )
                    };
                }
            }
            DrawType::imageRect => {
                debug_assert_eq!(desc.interlockMode, InterlockMode::atomics);
                unsafe {
                    implementation.m_vk.ashDevice().cmd_bind_vertex_buffers(
                        command,
                        layout::ImageRectGeometryBufferBinding,
                        &[implementation
                            .m_imageRectVertexBuffer
                            .as_ref()
                            .unwrap()
                            .vkBuffer()],
                        &[0],
                    );
                    implementation.m_vk.ashDevice().cmd_bind_vertex_buffers(
                        command,
                        layout::ImageRectImageAttribBufferBinding,
                        &[implementation
                            .m_imageDrawInstanceBuffer
                            .as_ref()
                            .unwrap()
                            .vkBuffer()],
                        &[0],
                    );
                    implementation.m_vk.ashDevice().cmd_bind_index_buffer(
                        command,
                        implementation
                            .m_imageRectIndexBuffer
                            .as_ref()
                            .unwrap()
                            .vkBuffer(),
                        0,
                        vk::IndexType::UINT16,
                    );
                    if pipeline.is_some() {
                        implementation.m_vk.ashDevice().cmd_draw_indexed(
                            command,
                            batch.indexCountPerInstance,
                            batch.elementCount,
                            batch.baseIndex,
                            0,
                            batch.baseElement,
                        );
                    }
                }
            }
            DrawType::imageMesh => {
                let Some(vertex) = batch.vertexBuffer else {
                    continue;
                };
                let Some(uv) = batch.uvBuffer else { continue };
                let Some(index) = batch.indexBuffer else {
                    continue;
                };
                if unsafe { vertex.as_ref().liteTypeID() }
                    != RenderBufferVulkanImpl::LITE_RTTI_TYPE_ID
                    || unsafe { uv.as_ref().liteTypeID() }
                        != RenderBufferVulkanImpl::LITE_RTTI_TYPE_ID
                    || unsafe { index.as_ref().liteTypeID() }
                        != RenderBufferVulkanImpl::LITE_RTTI_TYPE_ID
                {
                    continue;
                }
                let vertex =
                    unsafe { &*vertex.as_ptr().cast::<RenderBufferVulkanImpl>() }.currentBuffer();
                let uv = unsafe { &*uv.as_ptr().cast::<RenderBufferVulkanImpl>() }.currentBuffer();
                let index =
                    unsafe { &*index.as_ptr().cast::<RenderBufferVulkanImpl>() }.currentBuffer();
                let (Some(vertex), Some(uv), Some(index)) = (vertex, uv, index) else {
                    continue;
                };
                unsafe {
                    implementation.m_vk.ashDevice().cmd_bind_vertex_buffers(
                        command,
                        layout::ImageMeshVertexBufferBinding,
                        &[vertex.vkBuffer()],
                        &[0],
                    );
                    implementation.m_vk.ashDevice().cmd_bind_vertex_buffers(
                        command,
                        layout::ImageMeshUVBufferBinding,
                        &[uv.vkBuffer()],
                        &[0],
                    );
                    implementation.m_vk.ashDevice().cmd_bind_vertex_buffers(
                        command,
                        layout::ImageMeshImageAttribBufferBinding,
                        &[implementation
                            .m_imageDrawInstanceBuffer
                            .as_ref()
                            .unwrap()
                            .vkBuffer()],
                        &[0],
                    );
                    implementation.m_vk.ashDevice().cmd_bind_index_buffer(
                        command,
                        index.vkBuffer(),
                        0,
                        vk::IndexType::UINT16,
                    );
                    if pipeline.is_some() {
                        implementation.m_vk.ashDevice().cmd_draw_indexed(
                            command,
                            batch.indexCountPerInstance,
                            batch.elementCount,
                            batch.baseIndex,
                            0,
                            batch.baseElement,
                        );
                    }
                }
            }
            DrawType::renderPassInitialize | DrawType::renderPassResolve => {
                if pipeline.is_some() {
                    unsafe {
                        implementation
                            .m_vk
                            .ashDevice()
                            .cmd_draw(command, 4, 1, 0, 0)
                    };
                }
            }
        }
    }
    if desc.unresolvedBarriers.0 & BarrierFlags::clockwiseBorrowedCoverage.0 != 0 {
        unsafe {
            implementation
                .m_vk
                .ashDevice()
                .cmd_next_subpass(command, vk::SubpassContents::INLINE)
        };
    }
    debug_assert_eq!(pending_tess_patches, 0);
}

pub(crate) fn postFlush(implementation: &mut RenderContextVulkanImpl) {
    implementation
        .m_flushUniformBufferPool
        .recycle(implementation.m_flushUniformBuffer.take().unwrap());
    implementation
        .m_pathBufferPool
        .recycle(implementation.m_pathBuffer.take().unwrap());
    implementation
        .m_paintBufferPool
        .recycle(implementation.m_paintBuffer.take().unwrap());
    implementation
        .m_paintAuxBufferPool
        .recycle(implementation.m_paintAuxBuffer.take().unwrap());
    implementation
        .m_contourBufferPool
        .recycle(implementation.m_contourBuffer.take().unwrap());
    implementation
        .m_gradSpanBufferPool
        .recycle(implementation.m_gradSpanBuffer.take().unwrap());
    implementation
        .m_tessSpanBufferPool
        .recycle(implementation.m_tessSpanBuffer.take().unwrap());
    implementation
        .m_triangleBufferPool
        .recycle(implementation.m_triangleBuffer.take().unwrap());
    implementation
        .m_imageDrawInstanceBufferPool
        .recycle(implementation.m_imageDrawInstanceBuffer.take().unwrap());
}

pub(crate) fn hotloadShaders(implementation: &mut RenderContextVulkanImpl, data: &[u32]) {
    implementation
        .m_pipelineManager
        .as_mut()
        .unwrap()
        .as_mut()
        .clearCache();
    // The pinned API publishes spans into process-global shader slots; callers
    // are required to keep the hotload blob alive for every recreated pipeline.
    let source_lifetime: &'static [u32] = unsafe { core::mem::transmute(data) };
    spirv::hotload_shaders(source_lifetime);
    let manager = implementation.m_pipelineManager.as_ref().unwrap();
    *implementation.m_colorRampPipeline = Some(ColorRampPipeline::new(
        manager,
        implementation.m_workarounds,
    ));
    *implementation.m_tessellatePipeline = Some(TessellatePipeline::new(
        manager,
        implementation.m_workarounds,
    ));
    *implementation.m_featherAtlasPipeline = Some(FeatherAtlasPipeline::new(
        manager,
        implementation.m_workarounds,
    ));
}

pub(crate) fn startAsyncPipelineCreation(
    implementation: &RenderContextVulkanImpl,
    mode: InterlockMode,
    format: vk::Format,
    usage: vk::ImageUsageFlags,
    load: LoadAction,
) {
    implementation
        .m_pipelineManager
        .as_ref()
        .unwrap()
        .queueUbershaderPipelineCreation(
            mode,
            format,
            usage,
            load,
            implementation.platformFeatures(),
        );
}

pub(crate) fn startAsyncPipelineCreationForRenderTarget(
    implementation: &RenderContextVulkanImpl,
    mode: InterlockMode,
    target: &RenderTargetVulkan,
    load: LoadAction,
) {
    startAsyncPipelineCreation(
        implementation,
        mode,
        target.framebufferFormat(),
        target.targetUsageFlags(),
        load,
    );
}

pub(crate) fn waitForAsyncPipelineCreation(implementation: &RenderContextVulkanImpl) {
    implementation
        .m_pipelineManager
        .as_ref()
        .unwrap()
        .waitForAllBackgroundPipelineCreation();
}

macro_rules! resize_buffer {
    ($name:ident, $field:ident, $pool:ident) => {
        fn $name(&mut self, size: usize) {
            debug_assert!(self.$field.is_none());
            self.$pool.setTargetSize(size as u64);
        }
    };
}
macro_rules! map_buffer {
    ($name:ident, $field:ident) => {
        fn $name(&mut self, _size: usize) -> *mut c_void {
            self.$field
                .as_ref()
                .expect("prepared Vulkan flush buffer")
                .contents()
                .cast()
        }
    };
}
macro_rules! unmap_buffer {
    ($name:ident, $field:ident) => {
        fn $name(&mut self, size: usize) {
            self.$field
                .as_ref()
                .expect("prepared Vulkan flush buffer")
                .flushContents(size as u64)
        }
    };
}

impl RenderContextImplContract for RenderContextVulkanImpl {
    fn renderContextImpl(&self) -> &RenderContextImpl {
        &self.base
    }
    fn renderContextImplMut(&mut self) -> &mut RenderContextImpl {
        &mut self.base
    }
    fn makeRenderBuffer(
        &mut self,
        ty: RenderBufferType,
        flags: RenderBufferFlags,
        size: usize,
    ) -> rcp<RenderBuffer> {
        makeRenderBuffer(self, ty, flags, size)
    }
    fn makeImageTexture(
        &mut self,
        width: u32,
        height: u32,
        mip_levels: u32,
        format: crate::mechanical_port::source::include::rive::gpu_texture_format_hpp::GPUTextureFormat,
        data: &[u8],
        block_width: u8,
        block_height: u8,
        srgb: bool,
        generate_mips: bool,
    ) -> rcp<crate::mechanical_port::source::renderer::include::rive::renderer::texture_hpp::Texture>
    {
        let texture = makeImageTexture(
            self,
            width,
            height,
            mip_levels,
            format,
            data,
            block_width,
            block_height,
            srgb,
            generate_mips,
        );
        unsafe {
            crate::mechanical_port::source::include::rive::refcnt_hpp::static_rcp_cast(texture)
        }
    }
    #[cfg(feature = "native-ore-vulkan-experimental")]
    fn makeRenderCanvas(&mut self, width: u32, height: u32) -> rcp<RenderCanvas> {
        makeRenderCanvas(self, width, height)
    }
    #[cfg(feature = "native-ore-vulkan-experimental")]
    fn makeOreContext(
        &mut self,
    ) -> Option<Box<crate::mechanical_port::source::include::rive::factory_hpp::OreContext>> {
        let context = super::ore_context_vulkan_decl::ContextVulkan::Make(Arc::clone(&self.m_vk))?;
        Some(Box::new(
            crate::mechanical_port::source::include::rive::factory_hpp::OreContext::Vulkan(context),
        ))
    }
    resize_buffer!(
        resizeFlushUniformBuffer,
        m_flushUniformBuffer,
        m_flushUniformBufferPool
    );
    fn resizePathBuffer(&mut self, size: usize, _: StorageBufferStructure) {
        debug_assert!(self.m_pathBuffer.is_none());
        self.m_pathBufferPool.setTargetSize(size as u64);
    }
    fn resizePaintBuffer(&mut self, size: usize, _: StorageBufferStructure) {
        debug_assert!(self.m_paintBuffer.is_none());
        self.m_paintBufferPool.setTargetSize(size as u64);
    }
    fn resizePaintAuxBuffer(&mut self, size: usize, _: StorageBufferStructure) {
        debug_assert!(self.m_paintAuxBuffer.is_none());
        self.m_paintAuxBufferPool.setTargetSize(size as u64);
    }
    fn resizeContourBuffer(&mut self, size: usize, _: StorageBufferStructure) {
        debug_assert!(self.m_contourBuffer.is_none());
        self.m_contourBufferPool.setTargetSize(size as u64);
    }
    resize_buffer!(resizeGradSpanBuffer, m_gradSpanBuffer, m_gradSpanBufferPool);
    resize_buffer!(
        resizeTessVertexSpanBuffer,
        m_tessSpanBuffer,
        m_tessSpanBufferPool
    );
    resize_buffer!(
        resizeTriangleVertexBuffer,
        m_triangleBuffer,
        m_triangleBufferPool
    );
    resize_buffer!(
        resizeImageDrawInstanceBuffer,
        m_imageDrawInstanceBuffer,
        m_imageDrawInstanceBufferPool
    );
    unsafe fn wantsManualRenderPassResolve(
        &self,
        mode: InterlockMode,
        target: *const RenderTarget,
        bounds: &IAABB,
        tile_width: u32,
        tile_height: u32,
        contents: DrawContents,
    ) -> bool {
        unsafe {
            wantsManualRenderPassResolve(
                self,
                mode,
                target,
                bounds,
                tile_width,
                tile_height,
                contents,
            )
        }
    }
    fn prepareToFlush(&mut self, next: u64, safe: u64) {
        prepareToFlush(self, next, safe)
    }
    map_buffer!(mapFlushUniformBuffer, m_flushUniformBuffer);
    map_buffer!(mapPathBuffer, m_pathBuffer);
    map_buffer!(mapPaintBuffer, m_paintBuffer);
    map_buffer!(mapPaintAuxBuffer, m_paintAuxBuffer);
    map_buffer!(mapContourBuffer, m_contourBuffer);
    map_buffer!(mapGradSpanBuffer, m_gradSpanBuffer);
    map_buffer!(mapTessVertexSpanBuffer, m_tessSpanBuffer);
    map_buffer!(mapTriangleVertexBuffer, m_triangleBuffer);
    map_buffer!(mapImageDrawInstanceBuffer, m_imageDrawInstanceBuffer);
    unmap_buffer!(unmapFlushUniformBuffer, m_flushUniformBuffer);
    unmap_buffer!(unmapPathBuffer, m_pathBuffer);
    unmap_buffer!(unmapPaintBuffer, m_paintBuffer);
    unmap_buffer!(unmapPaintAuxBuffer, m_paintAuxBuffer);
    unmap_buffer!(unmapContourBuffer, m_contourBuffer);
    unmap_buffer!(unmapGradSpanBuffer, m_gradSpanBuffer);
    unmap_buffer!(unmapTessVertexSpanBuffer, m_tessSpanBuffer);
    unmap_buffer!(unmapTriangleVertexBuffer, m_triangleBuffer);
    unmap_buffer!(unmapImageDrawInstanceBuffer, m_imageDrawInstanceBuffer);
    fn resizeGradientTexture(&mut self, width: u32, height: u32) {
        resizeGradientTexture(self, width, height)
    }
    fn resizeTessellationTexture(&mut self, width: u32, height: u32) {
        resizeTessellationTexture(self, width, height)
    }
    fn resizeFeatherAtlasTexture(&mut self, width: u32, height: u32) {
        resizeFeatherAtlasTexture(self, width, height)
    }
    fn resizeTransientPLSBacking(&mut self, width: u32, height: u32, planes: u32) {
        resizeTransientPLSBacking(self, width, height, planes)
    }
    fn resizeAtomicCoverageBacking(&mut self, width: u32, height: u32) {
        resizeAtomicCoverageBacking(self, width, height)
    }
    fn resizeCoverageBuffer(&mut self, size: usize) {
        resizeCoverageBuffer(self, size)
    }
    unsafe fn flush(&mut self, descriptor: &FlushDescriptor) {
        unsafe { flush(self, descriptor) }
    }
    unsafe fn postFlush(&mut self, _: &FlushResources) {
        postFlush(self)
    }
    fn makeCommandBuffer(&mut self) -> *mut c_void {
        makeCommandBuffer(self)
    }
    unsafe fn commitCommandBuffer(&mut self, command: *mut c_void) {
        unsafe { commitCommandBuffer(self, command) }
    }
    fn secondsNow(&self) -> f64 {
        self.m_localEpoch.elapsed().as_secs_f64()
    }
}

pub(crate) unsafe fn MakeContext(
    instance: vk::Instance,
    physical_device: vk::PhysicalDevice,
    device: vk::Device,
    features: VulkanFeatures,
    get_instance_proc_addr: vk::PFN_vkGetInstanceProcAddr,
    options: ContextOptions,
) -> Option<std::pin::Pin<Box<RenderContext>>> {
    #[cfg(target_os = "android")]
    {
        unsafe extern "C" {
            fn __system_property_get(
                name: *const core::ffi::c_char,
                value: *mut core::ffi::c_char,
            ) -> core::ffi::c_int;
        }
        let mut value = [0 as core::ffi::c_char; 92];
        let length =
            unsafe { __system_property_get(c"ro.build.version.sdk".as_ptr(), value.as_mut_ptr()) };
        let api_level = if length > 0 {
            unsafe { CStr::from_ptr(value.as_ptr()) }
                .to_string_lossy()
                .parse::<i32>()
                .unwrap_or_default()
        } else {
            0
        };
        if api_level < 29 {
            eprintln!("ERROR: Rive Vulkan renderer requires Android 10 or newer.");
            return None;
        }
    }
    let vk_context = unsafe {
        VulkanContext::new(
            instance,
            physical_device,
            device,
            features,
            get_instance_proc_addr,
        )
    };
    let properties = vk_context.physicalDeviceProperties();
    if properties.api_version < vk::API_VERSION_1_1 {
        eprintln!(
            "ERROR: Rive Vulkan renderer requires a driver that supports at least Vulkan 1.1."
        );
        return None;
    }
    if properties.vendor_id == vkutil::Imagination && properties.api_version < vk::API_VERSION_1_3 {
        eprintln!("ERROR: Rive Vulkan renderer requires a driver that supports at least Vulkan 1.3 on PowerVR chipsets.");
        return None;
    }
    let mut implementation = Box::new(RenderContextVulkanImpl::new(vk_context, options));
    if options.forceAtomicMode && !implementation.platformFeatures().supportsAtomicMode {
        eprintln!("ERROR: Requested \"atomic\" mode but Vulkan does not support fragmentStoresAndAtomics on this platform.");
        return None;
    }
    implementation.initGPUObjects(options.shaderCompilationMode);
    Some(<RenderContext as RenderContextContract>::new(
        implementation,
    ))
}

impl Drop for RenderContextVulkanImpl {
    fn drop(&mut self) {
        debug_assert!(
            self.m_flushUniformBuffer.is_none()
                && self.m_pathBuffer.is_none()
                && self.m_paintBuffer.is_none()
                && self.m_paintAuxBuffer.is_none()
                && self.m_contourBuffer.is_none()
                && self.m_gradSpanBuffer.is_none()
                && self.m_tessSpanBuffer.is_none()
                && self.m_triangleBuffer.is_none()
                && self.m_imageDrawInstanceBuffer.is_none()
        );
        unsafe {
            if self.m_canvasCommandPool != vk::CommandPool::null() {
                self.m_vk
                    .ashDevice()
                    .destroy_command_pool(self.m_canvasCommandPool, None);
            }
            self.m_vk.shutdown();
            ManuallyDrop::drop(&mut self.m_pipelineManager);
            ManuallyDrop::drop(&mut self.m_descriptorSetPoolPool);
            ManuallyDrop::drop(&mut self.m_imageRectIndexBuffer);
            ManuallyDrop::drop(&mut self.m_imageRectVertexBuffer);
            ManuallyDrop::drop(&mut self.m_pathPatchIndexBuffer);
            ManuallyDrop::drop(&mut self.m_pathPatchVertexBuffer);
            ManuallyDrop::drop(&mut self.m_gaussianIntegralTexture);
            ManuallyDrop::drop(&mut self.m_coverageBuffer);
            ManuallyDrop::drop(&mut self.m_plsAtomicCoverageTexture);
            ManuallyDrop::drop(&mut self.m_plsOffscreenColorTexture);
            ManuallyDrop::drop(&mut self.m_plsTransientClipTexture_R16F);
            ManuallyDrop::drop(&mut self.m_plsBlendStorageTexture_RGB10_A2);
            ManuallyDrop::drop(&mut self.m_plsTransientScratchColorTexture);
            ManuallyDrop::drop(&mut self.m_plsTransientClipView);
            ManuallyDrop::drop(&mut self.m_plsTransientCoverageView);
            ManuallyDrop::drop(&mut self.m_plsTransientImageArray);
            ManuallyDrop::drop(&mut self.m_featherAtlasFramebuffer);
            ManuallyDrop::drop(&mut self.m_featherAtlasTexture);
            ManuallyDrop::drop(&mut self.m_featherAtlasPipeline);
            ManuallyDrop::drop(&mut self.m_tessTextureFramebuffer);
            ManuallyDrop::drop(&mut self.m_tesselationSyncIssueWorkaroundTexture);
            ManuallyDrop::drop(&mut self.m_tessTexture);
            ManuallyDrop::drop(&mut self.m_tessSpanIndexBuffer);
            ManuallyDrop::drop(&mut self.m_tessellatePipeline);
            ManuallyDrop::drop(&mut self.m_gradTextureFramebuffer);
            ManuallyDrop::drop(&mut self.m_gradTexture);
            ManuallyDrop::drop(&mut self.m_colorRampPipeline);
            ManuallyDrop::drop(&mut self.m_nullImageTexture);
            ManuallyDrop::drop(&mut self.m_imageDrawInstanceBuffer);
            ManuallyDrop::drop(&mut self.m_triangleBuffer);
            ManuallyDrop::drop(&mut self.m_tessSpanBuffer);
            ManuallyDrop::drop(&mut self.m_gradSpanBuffer);
            ManuallyDrop::drop(&mut self.m_contourBuffer);
            ManuallyDrop::drop(&mut self.m_paintAuxBuffer);
            ManuallyDrop::drop(&mut self.m_paintBuffer);
            ManuallyDrop::drop(&mut self.m_pathBuffer);
            ManuallyDrop::drop(&mut self.m_flushUniformBuffer);
            ManuallyDrop::drop(&mut self.m_imageDrawInstanceBufferPool);
            ManuallyDrop::drop(&mut self.m_triangleBufferPool);
            ManuallyDrop::drop(&mut self.m_tessSpanBufferPool);
            ManuallyDrop::drop(&mut self.m_gradSpanBufferPool);
            ManuallyDrop::drop(&mut self.m_contourBufferPool);
            ManuallyDrop::drop(&mut self.m_paintAuxBufferPool);
            ManuallyDrop::drop(&mut self.m_paintBufferPool);
            ManuallyDrop::drop(&mut self.m_pathBufferPool);
            ManuallyDrop::drop(&mut self.m_flushUniformBufferPool);
            ManuallyDrop::drop(&mut self.m_vk);
            ManuallyDrop::drop(&mut self.base);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_constants_and_options_are_frozen() {
        assert_eq!(
            ContextOptions::default().shaderCompilationMode,
            ShaderCompilationMode::standard
        );
        assert_eq!(PLS_TRANSIENT_COVERAGE_IDX, 0);
        assert_eq!(PLS_TRANSIENT_CLIP_IDX, 1);
        assert_eq!(K_MAX_IMAGE_TEXTURE_UPDATES, 256);
        assert_eq!(DescriptorSetPoolPool::MAX_POOL_SIZE, 64);
    }
}
