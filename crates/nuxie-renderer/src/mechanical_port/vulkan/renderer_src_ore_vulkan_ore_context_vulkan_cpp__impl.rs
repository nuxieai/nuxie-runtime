//! Complete mechanical implementation translation of the cyclic component
//! owner `renderer/src/ore/vulkan/ore_context_vulkan.cpp`.

#![allow(non_snake_case)]

use super::ore_bind_group_layout_vulkan_decl::BindGroupLayoutVulkan;
use super::ore_bind_group_vulkan_decl::{BindGroupVulkan, ImageWrite, UBOWrite};
use super::ore_buffer_vulkan_decl::BufferVulkan;
use super::ore_context_vulkan_decl::{
    ContextVulkan, DescriptorPoolGeneration, DescriptorSetAllocation, VKRenderPassKey,
    VkPendingImageTransition, VkPendingTextureUpload, MAX_DESCRIPTOR_SETS_PER_GENERATION,
};
use super::ore_render_pass_vulkan_decl::RenderPassVulkan;
use super::ore_sampler_vulkan_decl::SamplerVulkan;
use super::ore_shader_module_vulkan_decl::ShaderModuleVulkan;
use super::ore_texture_vulkan_decl::TextureViewVulkan;
use super::ore_texture_vulkan_decl::TextureVulkan;
use super::ore_vulkan_dsl::{createDSLFromLayoutDesc, kVkMaxBindingsPerGroup};
use super::render_target_vulkan_decl::{RenderTargetVulkanApi, RenderTargetVulkanImpl};
use super::vkutil_decl::Texture2D;
use crate::mechanical_port::source::renderer::include::rive::renderer::render_canvas_hpp::RenderCanvas;
use ash::vk;
use ash::vk::Handle;
use nuxie_ore_metal::buffer::BufferApi;
use nuxie_ore_metal::context::{ActiveRenderPass, ContextApi, FrameDescriptor, ShaderTarget};
use nuxie_ore_metal::gpu_resource::{AnyResourceHandle, ResourceHandle};
use nuxie_ore_metal::render_pass::RenderPassApi;
use nuxie_ore_metal::texture::TextureApi;
use nuxie_ore_metal::types::{
    kMaxBindGroups, BindGroupDesc, BindGroupLayoutDesc, BindGroupLayoutEntry, BindingKind,
    BufferDesc, BufferUsage, CompareFunction, Features, Filter, LoadOp, RenderPassDesc,
    SamplerDesc, ShaderModuleDesc, StoreOp, TextureAspect, TextureDesc, TextureFormat, TextureType,
    TextureViewDesc, TextureViewDimension, WrapMode,
};
use std::mem::ManuallyDrop;
use std::ptr::NonNull;
use std::rc::Weak as RcWeak;
use std::sync::{Arc, Mutex};
use vk_mem::{Alloc, AllocationCreateFlags, AllocationCreateInfo, MemoryUsage};

impl DescriptorPoolGeneration {
    pub(crate) fn new(vk_context: Arc<super::vulkan_context_decl::VulkanContext>) -> Self {
        let per_type = MAX_DESCRIPTOR_SETS_PER_GENERATION * kVkMaxBindingsPerGroup;
        let pool_sizes = [
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::UNIFORM_BUFFER,
                descriptor_count: per_type,
            },
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC,
                descriptor_count: per_type,
            },
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::SAMPLED_IMAGE,
                descriptor_count: per_type,
            },
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::SAMPLER,
                descriptor_count: per_type,
            },
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::STORAGE_BUFFER,
                descriptor_count: per_type,
            },
            vk::DescriptorPoolSize {
                ty: vk::DescriptorType::STORAGE_IMAGE,
                descriptor_count: per_type,
            },
        ];
        let create_info = vk::DescriptorPoolCreateInfo::default()
            .max_sets(MAX_DESCRIPTOR_SETS_PER_GENERATION)
            .pool_sizes(&pool_sizes);
        let pool = unsafe {
            vk_context
                .m_ashDevice
                .create_descriptor_pool(&create_info, None)
        }
        .unwrap_or(vk::DescriptorPool::null());
        Self {
            m_vk: vk_context,
            m_vkPool: pool,
            m_setsAllocated: Mutex::new(0),
        }
    }

    pub(crate) fn tryAllocate(&self, dsl: vk::DescriptorSetLayout) -> vk::DescriptorSet {
        let mut allocated = self
            .m_setsAllocated
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if self.m_vkPool == vk::DescriptorPool::null()
            || *allocated >= MAX_DESCRIPTOR_SETS_PER_GENERATION
        {
            return vk::DescriptorSet::null();
        }
        let layouts = [dsl];
        let allocate_info = vk::DescriptorSetAllocateInfo::default()
            .descriptor_pool(self.m_vkPool)
            .set_layouts(&layouts);
        let set = unsafe {
            self.m_vk
                .m_ashDevice
                .allocate_descriptor_sets(&allocate_info)
        }
        .ok()
        .and_then(|sets| sets.into_iter().next())
        .unwrap_or(vk::DescriptorSet::null());
        if set != vk::DescriptorSet::null() {
            *allocated += 1;
        }
        set
    }
}

impl Drop for DescriptorPoolGeneration {
    fn drop(&mut self) {
        if self.m_vkPool != vk::DescriptorPool::null() {
            unsafe {
                self.m_vk
                    .m_ashDevice
                    .destroy_descriptor_pool(self.m_vkPool, None)
            };
        }
    }
}

pub(crate) fn vkAllocateDescriptorSet(
    context: &mut ContextVulkan,
    dsl: vk::DescriptorSetLayout,
) -> DescriptorSetAllocation {
    if context.m_currentDescriptorPool.is_none() {
        context.m_currentDescriptorPool = Some(Arc::new(DescriptorPoolGeneration::new(
            Arc::clone(&context.m_vk),
        )));
    }
    let mut set = context
        .m_currentDescriptorPool
        .as_ref()
        .unwrap()
        .tryAllocate(dsl);
    if set == vk::DescriptorSet::null() {
        context.m_currentDescriptorPool = Some(Arc::new(DescriptorPoolGeneration::new(
            Arc::clone(&context.m_vk),
        )));
        set = context
            .m_currentDescriptorPool
            .as_ref()
            .unwrap()
            .tryAllocate(dsl);
    }
    DescriptorSetAllocation {
        set,
        pool: context.m_currentDescriptorPool.clone(),
    }
}

pub(crate) fn vkQueuePendingTextureUpload(
    context: &mut ContextVulkan,
    pending: VkPendingTextureUpload,
) {
    context.m_vkPendingTextureUploads.push(pending);
}

pub(crate) fn Make(
    vk_context: Arc<super::vulkan_context_decl::VulkanContext>,
) -> Option<Box<ContextVulkan>> {
    let ash_instance = &vk_context.m_ashInstance;
    let ash_device = &vk_context.m_ashDevice;
    let format_properties = unsafe {
        ash_instance.get_physical_device_format_properties(
            vk_context.physicalDevice,
            vk::Format::D24_UNORM_S8_UINT,
        )
    };
    let required =
        vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT | vk::FormatFeatureFlags::SAMPLED_IMAGE;
    let depth24_format = if format_properties.optimal_tiling_features.contains(required) {
        vk::Format::D24_UNORM_S8_UINT
    } else {
        vk::Format::D32_SFLOAT_S8_UINT
    };

    let pool_info = vk::CommandPoolCreateInfo::default()
        .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
        .queue_family_index(0);
    let command_pool = unsafe { ash_device.create_command_pool(&pool_info, None) }
        .unwrap_or(vk::CommandPool::null());
    let command_info = vk::CommandBufferAllocateInfo::default()
        .command_pool(command_pool)
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_buffer_count(1);
    let command_buffer = unsafe { ash_device.allocate_command_buffers(&command_info) }
        .unwrap_or_default()
        .into_iter()
        .next()
        .unwrap_or(vk::CommandBuffer::null());

    let pool_sizes = [
        vk::DescriptorPoolSize {
            ty: vk::DescriptorType::UNIFORM_BUFFER,
            descriptor_count: 2048,
        },
        vk::DescriptorPoolSize {
            ty: vk::DescriptorType::SAMPLED_IMAGE,
            descriptor_count: 2048,
        },
        vk::DescriptorPoolSize {
            ty: vk::DescriptorType::SAMPLER,
            descriptor_count: 2048,
        },
    ];
    let descriptor_info = vk::DescriptorPoolCreateInfo::default()
        .max_sets(768)
        .pool_sizes(&pool_sizes);
    let descriptor_pool = unsafe { ash_device.create_descriptor_pool(&descriptor_info, None) }
        .unwrap_or(vk::DescriptorPool::null());
    let fence_info = vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED);
    let frame_fence =
        unsafe { ash_device.create_fence(&fence_info, None) }.unwrap_or(vk::Fence::null());

    let properties =
        unsafe { ash_instance.get_physical_device_properties(vk_context.physicalDevice) };
    let device_features =
        unsafe { ash_instance.get_physical_device_features(vk_context.physicalDevice) };
    let mut features = Features::default();
    features.colorBufferFloat = true;
    features.perTargetBlend = device_features.independent_blend == vk::TRUE;
    features.perTargetWriteMask = device_features.independent_blend == vk::TRUE;
    features.textureViewSampling = true;
    features.drawBaseInstance = true;
    features.depthBiasClamp = device_features.depth_bias_clamp == vk::TRUE;
    features.anisotropicFiltering = device_features.sampler_anisotropy == vk::TRUE;
    features.texture3D = true;
    features.textureArrays = true;
    features.computeShaders = true;
    features.storageBuffers = true;
    features.bc = device_features.texture_compression_bc == vk::TRUE;
    features.etc2 = device_features.texture_compression_etc2 == vk::TRUE;
    features.astc = device_features.texture_compression_astc_ldr == vk::TRUE;
    features.maxColorAttachments = properties.limits.max_color_attachments;
    features.maxTextureSize2D = properties.limits.max_image_dimension2_d;
    features.maxTextureSizeCube = properties.limits.max_image_dimension_cube;
    features.maxTextureSize3D = properties.limits.max_image_dimension3_d;
    features.maxUniformBufferSize = properties.limits.max_uniform_buffer_range;
    features.maxVertexAttributes = properties.limits.max_vertex_input_attributes;
    features.maxSamplers = properties.limits.max_per_stage_descriptor_samplers;

    let manager = vk_context.manager();
    Some(Box::new(ContextVulkan {
        base: ManuallyDrop::new(nuxie_ore_metal::new_context_backend_base(
            features,
            Some(manager),
        )),
        m_vk: ManuallyDrop::new(vk_context),
        m_vkQueue: vk::Queue::null(),
        m_vkQueueFamily: 0,
        m_vkDepth24Stencil8Format: depth24_format,
        m_vkCommandPool: command_pool,
        m_vkCommandBuffer: command_buffer,
        m_vkDescriptorPool: descriptor_pool,
        m_vkFrameFence: frame_fence,
        m_vkCmdBufRecording: false,
        m_currentDescriptorPool: None,
        m_vkEmptyDSL: vk::DescriptorSetLayout::null(),
        m_vkRenderPassCache: Vec::new(),
        m_vkPendingInitialTransitions: Vec::new(),
        m_vkPendingTextureUploads: Vec::new(),
    }))
}

impl Drop for ContextVulkan {
    fn drop(&mut self) {
        if self.m_vk.device != vk::Device::null() {
            unsafe {
                if self.m_vkQueue != vk::Queue::null() {
                    let _ = self.m_vk.m_ashDevice.queue_wait_idle(self.m_vkQueue);
                }
                if self.m_vkFrameFence != vk::Fence::null() {
                    self.m_vk
                        .m_ashDevice
                        .destroy_fence(self.m_vkFrameFence, None);
                }
                for (_, render_pass) in self.m_vkRenderPassCache.drain(..) {
                    self.m_vk.m_ashDevice.destroy_render_pass(render_pass, None);
                }
                if self.m_vkDescriptorPool != vk::DescriptorPool::null() {
                    self.m_vk
                        .m_ashDevice
                        .destroy_descriptor_pool(self.m_vkDescriptorPool, None);
                }
                self.m_currentDescriptorPool = None;
                if self.m_vkEmptyDSL != vk::DescriptorSetLayout::null() {
                    self.m_vk
                        .m_ashDevice
                        .destroy_descriptor_set_layout(self.m_vkEmptyDSL, None);
                }
                if self.m_vkCommandPool != vk::CommandPool::null() {
                    self.m_vk
                        .m_ashDevice
                        .destroy_command_pool(self.m_vkCommandPool, None);
                }
            }
        }
        self.m_vkPendingTextureUploads.clear();
        self.m_vkPendingInitialTransitions.clear();
        unsafe {
            ManuallyDrop::drop(&mut self.m_vk);
            ManuallyDrop::drop(&mut self.base);
        }
    }
}

fn pipelineStageAndAccessForLayout(
    layout: vk::ImageLayout,
) -> (vk::PipelineStageFlags, vk::AccessFlags) {
    match layout {
        vk::ImageLayout::UNDEFINED => (
            vk::PipelineStageFlags::TOP_OF_PIPE,
            vk::AccessFlags::empty(),
        ),
        vk::ImageLayout::TRANSFER_DST_OPTIMAL => (
            vk::PipelineStageFlags::TRANSFER,
            vk::AccessFlags::TRANSFER_WRITE,
        ),
        vk::ImageLayout::TRANSFER_SRC_OPTIMAL => (
            vk::PipelineStageFlags::TRANSFER,
            vk::AccessFlags::TRANSFER_READ,
        ),
        vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL => (
            vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT,
            vk::AccessFlags::COLOR_ATTACHMENT_WRITE | vk::AccessFlags::COLOR_ATTACHMENT_READ,
        ),
        vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL
        | vk::ImageLayout::DEPTH_STENCIL_READ_ONLY_OPTIMAL => (
            vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS
                | vk::PipelineStageFlags::LATE_FRAGMENT_TESTS,
            vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE
                | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_READ,
        ),
        vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL => (
            vk::PipelineStageFlags::FRAGMENT_SHADER | vk::PipelineStageFlags::VERTEX_SHADER,
            vk::AccessFlags::SHADER_READ,
        ),
        _ => (
            vk::PipelineStageFlags::ALL_COMMANDS,
            vk::AccessFlags::MEMORY_READ | vk::AccessFlags::MEMORY_WRITE,
        ),
    }
}

pub(crate) fn vkQueueTransitionToLayout(
    context: &mut ContextVulkan,
    texture: &AnyResourceHandle,
    aspectMask: vk::ImageAspectFlags,
    newLayout: vk::ImageLayout,
) {
    let Some(vk_texture) = texture.downcast_ref::<TextureVulkan>() else {
        return;
    };
    if vk_texture.m_vkImage == vk::Image::null() || vk_texture.m_vkLayout.get() == newLayout {
        return;
    }
    if let Some(existing) = context
        .m_vkPendingInitialTransitions
        .iter_mut()
        .find(|existing| existing.texture.ptr_eq(texture))
    {
        existing.aspectMask |= aspectMask;
        existing.newLayout = newLayout;
        return;
    }
    context
        .m_vkPendingInitialTransitions
        .push(VkPendingImageTransition {
            texture: texture.clone(),
            aspectMask,
            oldLayout: vk_texture.m_vkLayout.get(),
            newLayout,
        });
}

pub(crate) fn vkFlushPendingInitialTransitions(context: &mut ContextVulkan) {
    if context.m_vkPendingInitialTransitions.is_empty() {
        return;
    }
    let mut barriers = Vec::with_capacity(context.m_vkPendingInitialTransitions.len());
    let mut src_stages = vk::PipelineStageFlags::empty();
    let mut dst_stages = vk::PipelineStageFlags::empty();
    for pending in &context.m_vkPendingInitialTransitions {
        let Some(texture) = pending.texture.downcast_ref::<TextureVulkan>() else {
            continue;
        };
        let old_layout = texture.m_vkLayout.get();
        if old_layout == pending.newLayout {
            continue;
        }
        let (src_stage, src_access) = pipelineStageAndAccessForLayout(old_layout);
        let (dst_stage, dst_access) = pipelineStageAndAccessForLayout(pending.newLayout);
        src_stages |= src_stage;
        dst_stages |= dst_stage;
        barriers.push(
            vk::ImageMemoryBarrier::default()
                .src_access_mask(src_access)
                .dst_access_mask(dst_access)
                .old_layout(old_layout)
                .new_layout(pending.newLayout)
                .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .image(texture.m_vkImage)
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: pending.aspectMask,
                    base_mip_level: 0,
                    level_count: vk::REMAINING_MIP_LEVELS,
                    base_array_layer: 0,
                    layer_count: vk::REMAINING_ARRAY_LAYERS,
                }),
        );
        texture.m_vkLayout.set(pending.newLayout);
    }
    if !barriers.is_empty() {
        unsafe {
            context.m_vk.m_ashDevice.cmd_pipeline_barrier(
                context.m_vkCommandBuffer,
                src_stages,
                dst_stages,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &barriers,
            );
        }
    }
    context.m_vkPendingInitialTransitions.clear();
}

pub(crate) fn vkFlushPendingTextureUploads(context: &mut ContextVulkan) {
    if context.m_vkPendingTextureUploads.is_empty() {
        return;
    }
    assert!(context.m_vkCmdBufRecording);
    assert_ne!(context.m_vkCommandBuffer, vk::CommandBuffer::null());

    struct DistinctTexture {
        texture: *const TextureVulkan,
        oldLayout: vk::ImageLayout,
        aspectMask: vk::ImageAspectFlags,
    }
    struct ValidUpload {
        srcBuffer: vk::Buffer,
        dstImage: vk::Image,
        region: vk::BufferImageCopy,
    }
    let mut distinct: Vec<DistinctTexture> = Vec::new();
    let mut uploads = Vec::with_capacity(context.m_vkPendingTextureUploads.len());
    for pending in &context.m_vkPendingTextureUploads {
        let Some(texture) = pending.texture.downcast_ref::<TextureVulkan>() else {
            continue;
        };
        let Some(buffer) = pending.stagingBuffer.downcast_ref::<BufferVulkan>() else {
            continue;
        };
        if texture.m_vkImage == vk::Image::null() {
            continue;
        }
        uploads.push(ValidUpload {
            srcBuffer: buffer.current(),
            dstImage: texture.m_vkImage,
            region: pending.region,
        });
        let pointer = texture as *const TextureVulkan;
        if let Some(existing) = distinct.iter_mut().find(|entry| entry.texture == pointer) {
            existing.aspectMask |= pending.aspectMask;
        } else {
            distinct.push(DistinctTexture {
                texture: pointer,
                oldLayout: texture.m_vkLayout.get(),
                aspectMask: pending.aspectMask,
            });
        }
    }
    if uploads.is_empty() {
        context.m_vkPendingTextureUploads.clear();
        return;
    }

    let mut to_transfer = Vec::with_capacity(distinct.len());
    let mut src_stages = vk::PipelineStageFlags::empty();
    for entry in &distinct {
        let texture = unsafe { &*entry.texture };
        let (src_stage, src_access) = pipelineStageAndAccessForLayout(entry.oldLayout);
        src_stages |= src_stage;
        to_transfer.push(
            vk::ImageMemoryBarrier::default()
                .src_access_mask(src_access)
                .dst_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                .old_layout(entry.oldLayout)
                .new_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                .src_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .dst_queue_family_index(vk::QUEUE_FAMILY_IGNORED)
                .image(texture.m_vkImage)
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: entry.aspectMask,
                    base_mip_level: 0,
                    level_count: vk::REMAINING_MIP_LEVELS,
                    base_array_layer: 0,
                    layer_count: vk::REMAINING_ARRAY_LAYERS,
                }),
        );
    }
    unsafe {
        context.m_vk.m_ashDevice.cmd_pipeline_barrier(
            context.m_vkCommandBuffer,
            src_stages,
            vk::PipelineStageFlags::TRANSFER,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &to_transfer,
        );
        for upload in &uploads {
            context.m_vk.m_ashDevice.cmd_copy_buffer_to_image(
                context.m_vkCommandBuffer,
                upload.srcBuffer,
                upload.dstImage,
                vk::ImageLayout::TRANSFER_DST_OPTIMAL,
                std::slice::from_ref(&upload.region),
            );
        }
    }

    let (dst_stage, dst_access) =
        pipelineStageAndAccessForLayout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);
    let to_shader = to_transfer
        .iter()
        .map(|barrier| {
            let mut barrier = *barrier;
            barrier.src_access_mask = vk::AccessFlags::TRANSFER_WRITE;
            barrier.dst_access_mask = dst_access;
            barrier.old_layout = vk::ImageLayout::TRANSFER_DST_OPTIMAL;
            barrier.new_layout = vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL;
            barrier
        })
        .collect::<Vec<_>>();
    unsafe {
        context.m_vk.m_ashDevice.cmd_pipeline_barrier(
            context.m_vkCommandBuffer,
            vk::PipelineStageFlags::TRANSFER,
            dst_stage,
            vk::DependencyFlags::empty(),
            &[],
            &[],
            &to_shader,
        );
    }
    for entry in &distinct {
        unsafe { &*entry.texture }
            .m_vkLayout
            .set(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);
    }
    context.m_vkPendingTextureUploads.clear();
}

pub(crate) fn beginFrame(context: &mut ContextVulkan, desc: &FrameDescriptor) {
    let external = desc
        .externalCommandBuffer
        .expect("ContextVulkan::beginFrame requires an external command buffer");
    nuxie_ore_metal::context_backend_manager(&context.base)
        .expect("ContextVulkan requires its source manager")
        .advanceFrameNumber(desc.currentFrameNumber, desc.safeFrameNumber);
    unsafe {
        let _ = context.m_vk.m_ashDevice.reset_descriptor_pool(
            context.m_vkDescriptorPool,
            vk::DescriptorPoolResetFlags::empty(),
        );
    }
    context.m_vkCommandBuffer = vk::CommandBuffer::from_raw(external.as_ptr() as u64);
    context.m_vkCmdBufRecording = true;
    vkFlushPendingTextureUploads(context);
    vkFlushPendingInitialTransitions(context);
}

pub(crate) fn waitForGPU(_context: &mut ContextVulkan) {}

pub(crate) fn endFrame(_context: &mut ContextVulkan) {}

fn contextResourceParts(
    context: &ContextVulkan,
) -> (
    nuxie_ore_metal::gpu_resource::GPUResourceManager,
    nuxie_ore_metal::gpu_resource::ResourceDomain,
) {
    (
        nuxie_ore_metal::context_backend_manager(&context.base)
            .expect("ContextVulkan requires its source manager"),
        nuxie_ore_metal::context_backend_domain(&context.base),
    )
}

pub(crate) fn makeBuffer(
    context: &mut ContextVulkan,
    desc: &BufferDesc<'_>,
) -> Option<AnyResourceHandle> {
    let (manager, domain) = contextResourceParts(context);
    let usage = match desc.usage {
        BufferUsage::vertex => vk::BufferUsageFlags::VERTEX_BUFFER,
        BufferUsage::index => vk::BufferUsageFlags::INDEX_BUFFER,
        BufferUsage::uniform => vk::BufferUsageFlags::UNIFORM_BUFFER,
        BufferUsage::upload => vk::BufferUsageFlags::TRANSFER_SRC,
    };
    let mut buffer = BufferVulkan::new(manager.clone(), desc.size, desc.usage);
    unsafe {
        // The context, device, and every allocation below are one retained
        // VulkanContext/VMA provenance domain.
        buffer.setVulkanContext(Arc::clone(&context.m_vk));
        buffer.setDeviceAndUsage(context.m_vk.device, usage);
    }
    let create_info = vk::BufferCreateInfo::default()
        .size(u64::from(desc.size))
        .usage(usage);
    let allocation_info = AllocationCreateInfo {
        flags: AllocationCreateFlags::MAPPED,
        #[allow(deprecated)]
        usage: MemoryUsage::CpuToGpu,
        ..Default::default()
    };
    let (vk_buffer, allocation) = unsafe {
        context
            .m_vk
            .allocator()
            .create_buffer(&create_info, &allocation_info)
    }
    .ok()?;
    let mapped = context
        .m_vk
        .allocator()
        .get_allocation_info(&allocation)
        .mapped_data
        .cast::<u8>();
    unsafe {
        // `vk_buffer`, `allocation`, and `mapped` were returned together by
        // the allocator retained in the context installed above.
        buffer.installPooledBacking(vk_buffer, allocation, mapped);
    }
    if let Some(data) = desc.data_prefix().ok()? {
        unsafe { std::ptr::copy_nonoverlapping(data.as_ptr(), mapped, desc.size as usize) };
    }
    Some(ResourceHandle::new_buffer_in_domain(Some(manager), domain, buffer).erase())
}

fn isDepthStencilFormatLocal(format: TextureFormat) -> bool {
    matches!(
        format,
        TextureFormat::depth16unorm
            | TextureFormat::depth24plusStencil8
            | TextureFormat::depth32float
            | TextureFormat::depth32floatStencil8
    )
}

fn hasStencilLocal(format: TextureFormat) -> bool {
    matches!(
        format,
        TextureFormat::depth24plusStencil8 | TextureFormat::depth32floatStencil8
    )
}

pub(crate) fn makeTexture(
    context: &mut ContextVulkan,
    desc: &TextureDesc<'_>,
) -> Option<AnyResourceHandle> {
    let (manager, domain) = contextResourceParts(context);
    let mut texture = TextureVulkan::new(manager.clone(), desc);
    texture.m_vkDevice = context.m_vk.device;
    *texture.m_vk = Some(Arc::clone(&context.m_vk));
    texture.m_vkOreContext.set(context);

    let mut usage = vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_DST;
    if desc.renderTarget {
        usage |= if isDepthStencilFormatLocal(desc.format) {
            vk::ImageUsageFlags::DEPTH_STENCIL_ATTACHMENT
        } else {
            vk::ImageUsageFlags::COLOR_ATTACHMENT
        };
    }
    let (image_type, flags, array_layers) = match desc.r#type {
        TextureType::texture2D => (vk::ImageType::TYPE_2D, vk::ImageCreateFlags::empty(), 1),
        TextureType::cube => (
            vk::ImageType::TYPE_2D,
            vk::ImageCreateFlags::CUBE_COMPATIBLE,
            6,
        ),
        TextureType::texture3D => (vk::ImageType::TYPE_3D, vk::ImageCreateFlags::empty(), 1),
        TextureType::array2D => (
            vk::ImageType::TYPE_2D,
            vk::ImageCreateFlags::empty(),
            desc.depthOrArrayLayers,
        ),
    };
    let image_info = vk::ImageCreateInfo::default()
        .flags(flags)
        .image_type(image_type)
        .format(context.vkFormatFor(desc.format))
        .extent(vk::Extent3D {
            width: desc.width,
            height: desc.height,
            depth: 1,
        })
        .mip_levels(desc.numMipmaps.max(1))
        .array_layers(array_layers)
        .samples(vk::SampleCountFlags::from_raw(desc.sampleCount))
        .tiling(vk::ImageTiling::OPTIMAL)
        .usage(usage)
        .initial_layout(vk::ImageLayout::UNDEFINED);
    let allocation_info = AllocationCreateInfo {
        #[allow(deprecated)]
        usage: MemoryUsage::GpuOnly,
        ..Default::default()
    };
    if let Ok((image, allocation)) = unsafe {
        context
            .m_vk
            .allocator()
            .create_image(&image_info, &allocation_info)
    } {
        texture.m_vkImage = image;
        texture.m_vmaAllocation = Some(Box::new(allocation));
    }
    texture.m_vkLayout.set(vk::ImageLayout::UNDEFINED);
    Some(ResourceHandle::new_texture_in_domain(Some(manager), domain, texture).erase())
}

pub(crate) fn makeTextureView(
    context: &mut ContextVulkan,
    desc: &TextureViewDesc<'_>,
) -> Option<AnyResourceHandle> {
    let texture_handle = desc.texture?;
    let texture = texture_handle.downcast_ref::<TextureVulkan>()?;
    let (manager, domain) = contextResourceParts(context);
    let mut view = TextureViewVulkan::new(manager.clone(), texture_handle.clone(), desc);
    view.m_vkDevice = context.m_vk.device;
    let aspect_mask = match desc.aspect {
        TextureAspect::all if isDepthStencilFormatLocal(texture.format()) => {
            let mut mask = vk::ImageAspectFlags::DEPTH;
            if hasStencilLocal(texture.format()) {
                mask |= vk::ImageAspectFlags::STENCIL;
            }
            mask
        }
        TextureAspect::all => vk::ImageAspectFlags::COLOR,
        TextureAspect::depthOnly => vk::ImageAspectFlags::DEPTH,
        TextureAspect::stencilOnly => vk::ImageAspectFlags::STENCIL,
    };
    let view_type = match desc.dimension {
        TextureViewDimension::texture2D => vk::ImageViewType::TYPE_2D,
        TextureViewDimension::cube => vk::ImageViewType::CUBE,
        TextureViewDimension::texture3D => vk::ImageViewType::TYPE_3D,
        TextureViewDimension::array2D => vk::ImageViewType::TYPE_2D_ARRAY,
        TextureViewDimension::cubeArray => vk::ImageViewType::CUBE_ARRAY,
    };
    let view_info = vk::ImageViewCreateInfo::default()
        .image(texture.m_vkImage)
        .view_type(view_type)
        .format(context.vkFormatFor(texture.format()))
        .subresource_range(vk::ImageSubresourceRange {
            aspect_mask,
            base_mip_level: desc.baseMipLevel,
            level_count: if desc.mipCount > 0 {
                desc.mipCount
            } else {
                vk::REMAINING_MIP_LEVELS
            },
            base_array_layer: desc.baseLayer,
            layer_count: if desc.layerCount > 0 {
                desc.layerCount
            } else {
                vk::REMAINING_ARRAY_LAYERS
            },
        });
    view.m_vkImageView = unsafe { context.m_vk.m_ashDevice.create_image_view(&view_info, None) }
        .unwrap_or(vk::ImageView::null());
    view.m_vkDestroyImageView = Some(context.m_vk.m_ashDevice.fp_v1_0().destroy_image_view);
    Some(ResourceHandle::new_in_domain(Some(manager), domain, view).erase())
}

fn samplerCompareToVk(compare: CompareFunction) -> vk::CompareOp {
    match compare {
        CompareFunction::none | CompareFunction::always => vk::CompareOp::ALWAYS,
        CompareFunction::never => vk::CompareOp::NEVER,
        CompareFunction::less => vk::CompareOp::LESS,
        CompareFunction::equal => vk::CompareOp::EQUAL,
        CompareFunction::lessEqual => vk::CompareOp::LESS_OR_EQUAL,
        CompareFunction::greater => vk::CompareOp::GREATER,
        CompareFunction::notEqual => vk::CompareOp::NOT_EQUAL,
        CompareFunction::greaterEqual => vk::CompareOp::GREATER_OR_EQUAL,
    }
}

pub(crate) fn makeSampler(
    context: &mut ContextVulkan,
    desc: &SamplerDesc<'_>,
) -> Option<AnyResourceHandle> {
    let (manager, domain) = contextResourceParts(context);
    let mut sampler = SamplerVulkan::new();
    let filter = |filter| match filter {
        Filter::linear => vk::Filter::LINEAR,
        Filter::nearest => vk::Filter::NEAREST,
    };
    let mipmap = |filter| match filter {
        Filter::linear => vk::SamplerMipmapMode::LINEAR,
        Filter::nearest => vk::SamplerMipmapMode::NEAREST,
    };
    let wrap = |mode| match mode {
        WrapMode::repeat => vk::SamplerAddressMode::REPEAT,
        WrapMode::mirrorRepeat => vk::SamplerAddressMode::MIRRORED_REPEAT,
        WrapMode::clampToEdge => vk::SamplerAddressMode::CLAMP_TO_EDGE,
    };
    let sampler_info = vk::SamplerCreateInfo::default()
        .mag_filter(filter(desc.magFilter))
        .min_filter(filter(desc.minFilter))
        .mipmap_mode(mipmap(desc.mipmapFilter))
        .address_mode_u(wrap(desc.wrapU))
        .address_mode_v(wrap(desc.wrapV))
        .address_mode_w(wrap(desc.wrapW))
        .mip_lod_bias(0.0)
        .anisotropy_enable(desc.maxAnisotropy > 1 && context.features().anisotropicFiltering)
        .max_anisotropy(desc.maxAnisotropy as f32)
        .compare_enable(desc.compare != CompareFunction::none)
        .compare_op(samplerCompareToVk(desc.compare))
        .min_lod(desc.minLod)
        .max_lod(desc.maxLod)
        .border_color(vk::BorderColor::FLOAT_TRANSPARENT_BLACK);
    let native = unsafe { context.m_vk.m_ashDevice.create_sampler(&sampler_info, None) }
        .unwrap_or(vk::Sampler::null());
    unsafe {
        sampler.setNativeSampler(
            context.m_vk.device,
            native,
            context.m_vk.m_ashDevice.fp_v1_0().destroy_sampler,
        );
    }
    Some(ResourceHandle::new_in_domain(Some(manager), domain, sampler).erase())
}

pub(crate) fn makeShaderModule(
    context: &mut ContextVulkan,
    desc: &ShaderModuleDesc<'_>,
) -> Option<AnyResourceHandle> {
    let (manager, domain) = contextResourceParts(context);
    let bytes = desc.code?.get(..desc.codeSize as usize)?;
    if bytes.len() % 4 != 0 {
        return None;
    }
    let words = bytes
        .chunks_exact(4)
        .map(|chunk| u32::from_ne_bytes(chunk.try_into().unwrap()))
        .collect::<Vec<_>>();
    let info = vk::ShaderModuleCreateInfo::default().code(&words);
    let native = match unsafe { context.m_vk.m_ashDevice.create_shader_module(&info, None) } {
        Ok(module) if module != vk::ShaderModule::null() => module,
        Err(result) => {
            context.base.setLastError(format!(
                "Ore Vulkan: vkCreateShaderModule failed (VkResult={})",
                result.as_raw()
            ));
            return None;
        }
        _ => return None,
    };
    let mut module = ShaderModuleVulkan::new();
    unsafe {
        module.setNativeShaderModule(
            context.m_vk.device,
            native,
            context.m_vk.m_ashDevice.fp_v1_0().destroy_shader_module,
        );
    }
    module.applyBindingMapFromDesc(desc);
    Some(ResourceHandle::new_in_domain(Some(manager), domain, module).erase())
}

pub(crate) fn makeBindGroupLayout(
    context: &mut ContextVulkan,
    desc: &BindGroupLayoutDesc<'_>,
) -> Option<AnyResourceHandle> {
    if desc.groupIndex >= kMaxBindGroups {
        context.base.setLastError(format!(
            "makeBindGroupLayout: groupIndex {} out of range [0, {})",
            desc.groupIndex, kMaxBindGroups
        ));
        return None;
    }
    let entries = desc.entries.get(..desc.entryCount as usize)?.to_vec();
    let (manager, domain) = contextResourceParts(context);
    let mut layout = BindGroupLayoutVulkan::new();
    nuxie_ore_metal::install_bind_group_layout_backend_parts(
        &mut layout,
        &context.base,
        desc.groupIndex,
        entries,
    );
    let native = unsafe {
        createDSLFromLayoutDesc(
            context
                .m_vk
                .m_ashDevice
                .fp_v1_0()
                .create_descriptor_set_layout,
            context.m_vk.device,
            desc,
        )
    };
    if native == vk::DescriptorSetLayout::null() {
        context.base.setLastError(format!(
            "makeBindGroupLayout: vkCreateDescriptorSetLayout failed (group={})",
            desc.groupIndex
        ));
        return None;
    }
    unsafe {
        layout.setNativeDescriptorSetLayout(
            context.m_vk.device,
            native,
            context
                .m_vk
                .m_ashDevice
                .fp_v1_0()
                .destroy_descriptor_set_layout,
        );
    }
    Some(ResourceHandle::new_in_domain(Some(manager), domain, layout).erase())
}

fn resolveLayoutBinding(
    context: &ContextVulkan,
    layout: &BindGroupLayoutVulkan,
    groupIndex: u32,
    binding: u32,
    expected: BindingKind,
) -> Option<u32> {
    let Some(entry) = layout.findEntry(binding) else {
        context.base.setLastError(format!(
            "makeBindGroup: (group={groupIndex}, binding={binding}) not declared in BindGroupLayout"
        ));
        return None;
    };
    let sampler_compatible = matches!(
        entry.kind,
        BindingKind::sampler | BindingKind::comparisonSampler
    ) && matches!(
        expected,
        BindingKind::sampler | BindingKind::comparisonSampler
    );
    if entry.kind != expected && !sampler_compatible {
        context.base.setLastError(format!(
            "makeBindGroup: (group={groupIndex}, binding={binding}) layout kind mismatch"
        ));
        return None;
    }
    Some(
        if entry.nativeSlotVS != BindGroupLayoutEntry::kNativeSlotAbsent {
            entry.nativeSlotVS
        } else if entry.nativeSlotFS != BindGroupLayoutEntry::kNativeSlotAbsent {
            entry.nativeSlotFS
        } else {
            binding
        },
    )
}

pub(crate) fn makeBindGroup(
    context: &mut ContextVulkan,
    desc: &BindGroupDesc<'_>,
) -> Option<AnyResourceHandle> {
    let Some(layout_handle) = desc.layout else {
        context.setLastError("makeBindGroup: BindGroupDesc::layout is null");
        return None;
    };
    let layout = layout_handle
        .downcast_ref::<BindGroupLayoutVulkan>()
        .expect("ContextVulkan requires a BindGroupLayoutVulkan");
    let groupIndex = layout.groupIndex();
    if groupIndex >= kMaxBindGroups {
        context.base.setLastError(format!(
            "makeBindGroup: layout->groupIndex {groupIndex} out of range [0, {kMaxBindGroups})"
        ));
        return None;
    }
    let (manager, domain) = contextResourceParts(context);
    let mut bg = BindGroupVulkan::new(manager.clone());
    bg.setContext(context);
    bg.m_vkDSL = layout.m_vkDSL;
    let dynamicCount = layout
        .entries()
        .iter()
        .filter(|entry| entry.kind == BindingKind::uniformBuffer && entry.hasDynamicOffset)
        .count() as u32;
    let mut retainedBuffers = Vec::new();
    let mut retainedViews = Vec::new();
    let mut retainedSamplers = Vec::new();

    for ubo in desc.ubos.get(..desc.uboCount as usize)? {
        let Some(dstBinding) = resolveLayoutBinding(
            context,
            layout,
            groupIndex,
            ubo.slot,
            BindingKind::uniformBuffer,
        ) else {
            continue;
        };
        let buffer_handle = ubo.buffer.expect("UBOEntry::buffer is non-null");
        let buffer = buffer_handle
            .downcast_ref::<BufferVulkan>()
            .expect("ContextVulkan requires BufferVulkan UBOs");
        bg.m_uboWrites.push(UBOWrite {
            buffer,
            dstBinding,
            offset: ubo.offset,
            range: if ubo.size > 0 {
                ubo.size
            } else {
                buffer.size()
            },
            r#type: if layout.hasDynamicOffset(ubo.slot) {
                vk::DescriptorType::UNIFORM_BUFFER_DYNAMIC
            } else {
                vk::DescriptorType::UNIFORM_BUFFER
            },
        });
        retainedBuffers.push(buffer_handle.clone());
    }

    for texture_entry in desc.textures.get(..desc.textureCount as usize)? {
        let Some(dstBinding) = resolveLayoutBinding(
            context,
            layout,
            groupIndex,
            texture_entry.slot,
            BindingKind::sampledTexture,
        ) else {
            continue;
        };
        let view_handle = texture_entry.view.expect("TexEntry::view is non-null");
        let view = view_handle
            .downcast_ref::<TextureViewVulkan>()
            .expect("ContextVulkan requires TextureViewVulkan bindings");
        let base_texture_handle = view.texture();
        let base_texture = base_texture_handle
            .downcast_ref::<TextureVulkan>()
            .expect("TextureViewVulkan retains TextureVulkan");
        let mut aspect = if isDepthStencilFormatLocal(base_texture.format()) {
            vk::ImageAspectFlags::DEPTH
        } else {
            vk::ImageAspectFlags::COLOR
        };
        if hasStencilLocal(base_texture.format()) {
            aspect |= vk::ImageAspectFlags::STENCIL;
        }
        context.vkQueueTransitionToLayout(
            base_texture_handle,
            aspect,
            vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
        );
        bg.m_imageWrites.push(ImageWrite {
            dstBinding,
            r#type: vk::DescriptorType::SAMPLED_IMAGE,
            imageView: view.m_vkImageView,
            imageLayout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL,
            sampler: vk::Sampler::null(),
        });
        retainedViews.push(view_handle.clone());
    }

    for sampler_entry in desc.samplers.get(..desc.samplerCount as usize)? {
        let Some(dstBinding) = resolveLayoutBinding(
            context,
            layout,
            groupIndex,
            sampler_entry.slot,
            BindingKind::sampler,
        ) else {
            continue;
        };
        let sampler_handle = sampler_entry
            .sampler
            .expect("SampEntry::sampler is non-null");
        let sampler = sampler_handle
            .downcast_ref::<SamplerVulkan>()
            .expect("ContextVulkan requires SamplerVulkan bindings");
        bg.m_imageWrites.push(ImageWrite {
            dstBinding,
            r#type: vk::DescriptorType::SAMPLER,
            imageView: vk::ImageView::null(),
            imageLayout: vk::ImageLayout::UNDEFINED,
            sampler: sampler.m_vkSampler,
        });
        retainedSamplers.push(sampler_handle.clone());
    }
    nuxie_ore_metal::install_bind_group_backend_parts(
        &mut bg.base,
        dynamicCount,
        Some(layout_handle.clone()),
        retainedBuffers,
        retainedViews,
        retainedSamplers,
    );
    if bg.resolveDescriptorSet() == vk::DescriptorSet::null() {
        return None;
    }
    Some(ResourceHandle::new_in_domain(Some(manager), domain, bg).erase())
}

pub(crate) fn beginRenderPass(
    context: &mut ContextVulkan,
    desc: &RenderPassDesc<'_>,
    _outError: Option<&mut String>,
) -> Option<Box<dyn RenderPassApi>> {
    context.finishActiveRenderPass();
    assert!(desc.colorCount <= 4);

    let pass = RenderPassVulkan::new(context);
    let mut state = pass.borrowState();
    state.m_vkCmdBuf = context.m_vkCommandBuffer;
    state.m_vkColorCount = desc.colorCount;

    let mut key = VKRenderPassKey {
        colorCount: desc.colorCount,
        sampleCount: 1,
        ..Default::default()
    };
    let mut passWidth = 0;
    let mut passHeight = 0;
    let mut attachmentViews = [vk::ImageView::null(); 9];
    let mut attachmentCount = 0usize;
    let mut resolveViews = [vk::ImageView::null(); 4];
    let mut anyResolve = false;
    let mut colorFormats = [TextureFormat::r8unorm; 4];

    for index in 0..desc.colorCount as usize {
        let attachment = &desc.colorAttachments[index];
        let viewHandle = attachment
            .view
            .expect("RenderPass color attachment view is non-null");
        let view = viewHandle
            .downcast_ref::<TextureViewVulkan>()
            .expect("ContextVulkan requires TextureViewVulkan attachments");
        let textureHandle = view.texture();
        let texture = textureHandle
            .downcast_ref::<TextureVulkan>()
            .expect("TextureViewVulkan retains TextureVulkan");
        key.colorFormats[index] = texture.format();
        colorFormats[index] = texture.format();
        key.colorLoadOps[index] = attachment.loadOp;
        key.colorStoreOps[index] = attachment.storeOp;
        key.colorHasResolve[index] = attachment.resolveTarget.is_some();

        if let Some(resolveHandle) = attachment.resolveTarget {
            anyResolve = true;
            let resolveView = resolveHandle
                .downcast_ref::<TextureViewVulkan>()
                .expect("ContextVulkan requires Vulkan resolve views");
            resolveViews[index] = resolveView.m_vkImageView;
            let resolveTextureHandle = resolveView.texture();
            if let Some(resolveTexture) = resolveTextureHandle.downcast_ref::<TextureVulkan>() {
                state.m_vkResolveTargets[index].image = resolveTexture.m_vkImage;
                state.m_vkResolveTargets[index].texture = Some(resolveTextureHandle.clone());
            }
            state.m_vkResolveTargets[index].baseMip = resolveView.baseMipLevel();
            state.m_vkResolveTargets[index].baseLayer = resolveView.baseLayer();
            state.m_vkResolveTargets[index].layerCount = resolveView.layerCount();
            state.m_vkResolveTargets[index].renderTarget = resolveView.m_vkRenderTarget;
        }
        if key.sampleCount == 1 {
            key.sampleCount = texture.sampleCount();
        }
        passWidth = texture.width();
        passHeight = texture.height();
        attachmentViews[attachmentCount] = view.m_vkImageView;
        attachmentCount += 1;
        state.m_vkColorImages[index] = texture.m_vkImage;
        state.m_vkColorBaseLayer[index] = view.baseLayer();
        state.m_vkColorLayerCount[index] = view.layerCount();
        state.m_vkColorRenderTargets[index] = view.m_vkRenderTarget;
        state.m_vkColorTextures[index] = Some(textureHandle.clone());
        if attachment.loadOp == LoadOp::load {
            context.vkQueueTransitionToLayout(
                textureHandle,
                vk::ImageAspectFlags::COLOR,
                vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL,
            );
        }
    }

    if anyResolve {
        for index in 0..desc.colorCount as usize {
            if key.colorHasResolve[index] {
                attachmentViews[attachmentCount] = resolveViews[index];
                attachmentCount += 1;
            }
        }
    }

    let mut depthFormat = TextureFormat::r8unorm;
    if let Some(depthViewHandle) = desc.depthStencil.view {
        let depthView = depthViewHandle
            .downcast_ref::<TextureViewVulkan>()
            .expect("ContextVulkan requires a Vulkan depth view");
        let depthTextureHandle = depthView.texture();
        let depthTexture = depthTextureHandle
            .downcast_ref::<TextureVulkan>()
            .expect("TextureViewVulkan retains a Vulkan depth texture");
        key.hasDepth = true;
        key.depthFormat = depthTexture.format();
        depthFormat = depthTexture.format();
        key.depthLoadOp = desc.depthStencil.depthLoadOp;
        key.depthStoreOp = desc.depthStencil.depthStoreOp;
        attachmentViews[attachmentCount] = depthView.m_vkImageView;
        attachmentCount += 1;
        state.m_vkDepthImage = depthTexture.m_vkImage;
        state.m_vkDepthBaseLayer = depthView.baseLayer();
        state.m_vkDepthLayerCount = depthView.layerCount();
        *state.m_vkDepthTexture = Some(depthTextureHandle.clone());
        if desc.depthStencil.depthLoadOp == LoadOp::load {
            let mut aspect = vk::ImageAspectFlags::DEPTH;
            if hasStencilLocal(depthTexture.format()) {
                aspect |= vk::ImageAspectFlags::STENCIL;
            }
            context.vkQueueTransitionToLayout(
                depthTextureHandle,
                aspect,
                vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL,
            );
        }
        if passWidth == 0 {
            passWidth = depthTexture.width();
            passHeight = depthTexture.height();
        }
    }
    nuxie_ore_metal::render_pass_install_attachment_metadata(
        &mut state.base,
        colorFormats,
        desc.colorCount,
        depthFormat,
        key.hasDepth,
        key.sampleCount,
    );

    let renderPass = context.getOrCreateRenderPass(&key);
    let framebufferInfo = vk::FramebufferCreateInfo::default()
        .render_pass(renderPass)
        .attachments(&attachmentViews[..attachmentCount])
        .width(passWidth)
        .height(passHeight)
        .layers(1);
    *state.m_framebuffer = Some(context.m_vk.makeFramebuffer(framebufferInfo));

    let mut clearValues = [vk::ClearValue::default(); 9];
    for index in 0..desc.colorCount as usize {
        let color = desc.colorAttachments[index].clearColor;
        clearValues[index] = vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [color.r, color.g, color.b, color.a],
            },
        };
    }
    if key.hasDepth {
        let resolveCount = if anyResolve {
            key.colorHasResolve[..desc.colorCount as usize]
                .iter()
                .filter(|hasResolve| **hasResolve)
                .count()
        } else {
            0
        };
        clearValues[desc.colorCount as usize + resolveCount] = vk::ClearValue {
            depth_stencil: vk::ClearDepthStencilValue {
                depth: desc.depthStencil.depthClearValue,
                stencil: desc.depthStencil.stencilClearValue,
            },
        };
    }
    let framebuffer = state
        .m_framebuffer
        .as_ref()
        .expect("framebuffer was created")
        .vkFramebuffer();
    let renderArea = vk::Rect2D {
        offset: vk::Offset2D { x: 0, y: 0 },
        extent: vk::Extent2D {
            width: passWidth,
            height: passHeight,
        },
    };
    let beginInfo = vk::RenderPassBeginInfo::default()
        .render_pass(renderPass)
        .framebuffer(framebuffer)
        .render_area(renderArea)
        .clear_values(&clearValues[..attachmentCount]);

    context.vkFlushPendingTextureUploads();
    context.vkFlushPendingInitialTransitions();
    unsafe {
        context.m_vk.m_ashDevice.cmd_begin_render_pass(
            context.m_vkCommandBuffer,
            &beginInfo,
            vk::SubpassContents::INLINE,
        );
    }
    for texture in state
        .m_vkColorTextures
        .iter()
        .take(desc.colorCount as usize)
        .flatten()
    {
        if let Some(texture) = texture.downcast_ref::<TextureVulkan>() {
            texture
                .m_vkLayout
                .set(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);
        }
    }
    if let Some(texture) = state
        .m_vkDepthTexture
        .as_ref()
        .and_then(|texture| texture.downcast_ref::<TextureVulkan>())
    {
        texture
            .m_vkLayout
            .set(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);
    }
    let viewport = vk::Viewport {
        x: 0.0,
        y: 0.0,
        width: passWidth as f32,
        height: passHeight as f32,
        min_depth: 0.0,
        max_depth: 1.0,
    };
    let scissor = renderArea;
    unsafe {
        context.m_vk.m_ashDevice.cmd_set_viewport(
            context.m_vkCommandBuffer,
            0,
            std::slice::from_ref(&viewport),
        );
        context.m_vk.m_ashDevice.cmd_set_scissor(
            context.m_vkCommandBuffer,
            0,
            std::slice::from_ref(&scissor),
        );
    }
    drop(state);
    Some(Box::new(pass))
}

pub(crate) unsafe fn wrapCanvasTexture(
    context: &mut ContextVulkan,
    canvas: *mut core::ffi::c_void,
) -> Option<AnyResourceHandle> {
    let canvas = unsafe { canvas.cast::<RenderCanvas>().as_mut() }?;
    let target = unsafe {
        canvas
            .renderTarget()
            .cast::<RenderTargetVulkanImpl>()
            .as_mut()
    }?;
    let image = target.targetImage();
    let imageView = target.targetImageView();
    if image == vk::Image::null() || imageView == vk::ImageView::null() {
        return None;
    }
    let format = match target.base.framebufferFormat() {
        vk::Format::B8G8R8A8_UNORM => TextureFormat::bgra8unorm,
        vk::Format::R16G16B16A16_SFLOAT => TextureFormat::rgba16float,
        vk::Format::A2B10G10R10_UNORM_PACK32 => TextureFormat::rgb10a2unorm,
        _ => TextureFormat::rgba8unorm,
    };
    let desc = TextureDesc {
        width: canvas.width(),
        height: canvas.height(),
        format,
        r#type: TextureType::texture2D,
        renderTarget: true,
        numMipmaps: 1,
        sampleCount: 1,
        ..Default::default()
    };
    let (manager, domain) = contextResourceParts(context);
    let mut texture = TextureVulkan::new(manager.clone(), &desc);
    texture.m_vkImage = image;
    *texture.m_vk = Some(Arc::clone(&context.m_vk));
    texture.m_vkLayout.set(vk::ImageLayout::UNDEFINED);
    let texture =
        ResourceHandle::new_texture_in_domain(Some(manager.clone()), domain.clone(), texture)
            .erase();
    let viewDesc = TextureViewDesc {
        texture: Some(&texture),
        dimension: TextureViewDimension::texture2D,
        aspect: TextureAspect::all,
        baseMipLevel: 0,
        mipCount: 1,
        baseLayer: 0,
        layerCount: 1,
    };
    let mut view = TextureViewVulkan::new(manager.clone(), texture.clone(), &viewDesc);
    view.m_vkImageView = imageView;
    let targetApi: &mut dyn RenderTargetVulkanApi = target;
    view.m_vkRenderTarget = Some(NonNull::from(targetApi));
    Some(ResourceHandle::new_in_domain(Some(manager), domain, view).erase())
}

pub(crate) unsafe fn wrapRiveTexture(
    context: &mut ContextVulkan,
    texture: *mut core::ffi::c_void,
    width: u32,
    height: u32,
) -> Option<AnyResourceHandle> {
    let source = unsafe { texture.cast::<Texture2D>().as_ref() }?;
    let image = source.vkImage();
    let imageView = source.vkImageView();
    if image == vk::Image::null() || imageView == vk::ImageView::null() {
        return None;
    }
    assert_ne!(
        context.m_vkCommandBuffer,
        vk::CommandBuffer::null(),
        "wrapRiveTexture requires an open frame: call beginFrame() first"
    );
    source.prepareForFragmentShaderRead(context.m_vkCommandBuffer);
    let desc = TextureDesc {
        width,
        height,
        format: TextureFormat::rgba8unorm,
        r#type: TextureType::texture2D,
        renderTarget: false,
        numMipmaps: 1,
        sampleCount: 1,
        ..Default::default()
    };
    let (manager, domain) = contextResourceParts(context);
    let mut wrapped = TextureVulkan::new(manager.clone(), &desc);
    wrapped.m_vkImage = image;
    *wrapped.m_vk = Some(Arc::clone(&context.m_vk));
    wrapped
        .m_vkLayout
        .set(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL);
    let wrapped =
        ResourceHandle::new_texture_in_domain(Some(manager.clone()), domain.clone(), wrapped)
            .erase();
    let viewDesc = TextureViewDesc {
        texture: Some(&wrapped),
        dimension: TextureViewDimension::texture2D,
        aspect: TextureAspect::all,
        baseMipLevel: 0,
        mipCount: 1,
        baseLayer: 0,
        layerCount: 1,
    };
    let mut view = TextureViewVulkan::new(manager.clone(), wrapped.clone(), &viewDesc);
    view.m_vkImageView = imageView;
    Some(ResourceHandle::new_in_domain(Some(manager), domain, view).erase())
}

impl ContextApi for ContextVulkan {
    fn features(&self) -> Features {
        self.base.features()
    }
    fn lastError(&self) -> String {
        self.base.lastError()
    }
    fn activeRenderPass(&self) -> Option<RcWeak<dyn ActiveRenderPass>> {
        self.base.activeRenderPass()
    }
    fn setActiveRenderPass(&self, pass: Option<&dyn RenderPassApi>) {
        self.base.setActiveRenderPass(pass);
    }
    fn finishActiveRenderPass(&self) {
        self.base.finishActiveRenderPass();
    }
    fn clearLastError(&self) {
        self.base.clearLastError();
    }
    fn setLastError(&self, message: &str) {
        self.base.setLastError(message);
    }
    fn makeBuffer(&mut self, desc: &BufferDesc<'_>) -> Option<AnyResourceHandle> {
        ContextVulkan::makeBuffer(self, desc)
    }
    fn makeTexture(&mut self, desc: &TextureDesc<'_>) -> Option<AnyResourceHandle> {
        ContextVulkan::makeTexture(self, desc)
    }
    fn makeTextureView(&mut self, desc: &TextureViewDesc<'_>) -> Option<AnyResourceHandle> {
        ContextVulkan::makeTextureView(self, desc)
    }
    fn makeSampler(&mut self, desc: &SamplerDesc<'_>) -> Option<AnyResourceHandle> {
        ContextVulkan::makeSampler(self, desc)
    }
    fn makeShaderModule(&mut self, desc: &ShaderModuleDesc<'_>) -> Option<AnyResourceHandle> {
        ContextVulkan::makeShaderModule(self, desc)
    }
    fn makeBindGroupLayout(&mut self, desc: &BindGroupLayoutDesc<'_>) -> Option<AnyResourceHandle> {
        ContextVulkan::makeBindGroupLayout(self, desc)
    }
    fn makePipeline(
        &mut self,
        desc: &nuxie_ore_metal::types::PipelineDesc<'_>,
        outError: Option<&mut String>,
    ) -> Option<AnyResourceHandle> {
        ContextVulkan::makePipeline(self, desc, outError)
    }
    fn makeBindGroup(&mut self, desc: &BindGroupDesc<'_>) -> Option<AnyResourceHandle> {
        ContextVulkan::makeBindGroup(self, desc)
    }
    fn beginRenderPass(
        &mut self,
        desc: &RenderPassDesc<'_>,
        outError: Option<&mut String>,
    ) -> Option<Box<dyn RenderPassApi>> {
        ContextVulkan::beginRenderPass(self, desc, outError)
    }
    fn beginFrame(&mut self, descriptor: &FrameDescriptor) {
        ContextVulkan::beginFrame(self, descriptor);
    }
    fn endFrame(&mut self) {
        ContextVulkan::endFrame(self);
    }
    fn waitForGPU(&mut self) {
        ContextVulkan::waitForGPU(self);
    }
    unsafe fn wrapCanvasTexture(
        &mut self,
        canvas: *mut core::ffi::c_void,
    ) -> Option<AnyResourceHandle> {
        unsafe { ContextVulkan::wrapCanvasTexture(self, canvas) }
    }
    unsafe fn wrapRiveTexture(
        &mut self,
        texture: *mut core::ffi::c_void,
        width: u32,
        height: u32,
    ) -> Option<AnyResourceHandle> {
        unsafe { ContextVulkan::wrapRiveTexture(self, texture, width, height) }
    }
    fn shaderTarget(&self) -> ShaderTarget {
        ShaderTarget::spirv
    }
}

fn oreFormatToVkLocal(format: TextureFormat) -> vk::Format {
    match format {
        TextureFormat::r8unorm => vk::Format::R8_UNORM,
        TextureFormat::rg8unorm => vk::Format::R8G8_UNORM,
        TextureFormat::rgba8unorm => vk::Format::R8G8B8A8_UNORM,
        TextureFormat::rgba8snorm => vk::Format::R8G8B8A8_SNORM,
        TextureFormat::bgra8unorm => vk::Format::B8G8R8A8_UNORM,
        TextureFormat::rgba16float => vk::Format::R16G16B16A16_SFLOAT,
        TextureFormat::rg16float => vk::Format::R16G16_SFLOAT,
        TextureFormat::r16float => vk::Format::R16_SFLOAT,
        TextureFormat::rgba32float => vk::Format::R32G32B32A32_SFLOAT,
        TextureFormat::rg32float => vk::Format::R32G32_SFLOAT,
        TextureFormat::r32float => vk::Format::R32_SFLOAT,
        TextureFormat::rgb10a2unorm => vk::Format::A2B10G10R10_UNORM_PACK32,
        TextureFormat::r11g11b10float => vk::Format::B10G11R11_UFLOAT_PACK32,
        TextureFormat::depth16unorm => vk::Format::D16_UNORM,
        TextureFormat::depth24plusStencil8 => vk::Format::UNDEFINED,
        TextureFormat::depth32float => vk::Format::D32_SFLOAT,
        TextureFormat::depth32floatStencil8 => vk::Format::D32_SFLOAT_S8_UINT,
        TextureFormat::bc1unorm => vk::Format::BC1_RGBA_UNORM_BLOCK,
        TextureFormat::bc3unorm => vk::Format::BC3_UNORM_BLOCK,
        TextureFormat::bc7unorm => vk::Format::BC7_UNORM_BLOCK,
        TextureFormat::etc2rgb8 => vk::Format::ETC2_R8G8B8_UNORM_BLOCK,
        TextureFormat::etc2rgba8 => vk::Format::ETC2_R8G8B8A8_UNORM_BLOCK,
        TextureFormat::astc4x4 => vk::Format::ASTC_4X4_UNORM_BLOCK,
        TextureFormat::astc6x6 => vk::Format::ASTC_6X6_UNORM_BLOCK,
        TextureFormat::astc8x8 => vk::Format::ASTC_8X8_UNORM_BLOCK,
    }
}

pub(crate) fn vkFormatFor(context: &ContextVulkan, format: TextureFormat) -> vk::Format {
    if format == TextureFormat::depth24plusStencil8 {
        context.m_vkDepth24Stencil8Format
    } else {
        oreFormatToVkLocal(format)
    }
}

fn oreLoadOpToVk(op: LoadOp) -> vk::AttachmentLoadOp {
    match op {
        LoadOp::clear => vk::AttachmentLoadOp::CLEAR,
        LoadOp::load => vk::AttachmentLoadOp::LOAD,
        LoadOp::dontCare => vk::AttachmentLoadOp::DONT_CARE,
    }
}

fn oreStoreOpToVk(op: StoreOp) -> vk::AttachmentStoreOp {
    match op {
        StoreOp::store => vk::AttachmentStoreOp::STORE,
        StoreOp::discard => vk::AttachmentStoreOp::DONT_CARE,
    }
}

pub(crate) fn vkGetOrCreateEmptyDSL(context: &mut ContextVulkan) -> vk::DescriptorSetLayout {
    if context.m_vkEmptyDSL != vk::DescriptorSetLayout::null() {
        return context.m_vkEmptyDSL;
    }
    let info = vk::DescriptorSetLayoutCreateInfo::default();
    context.m_vkEmptyDSL = unsafe {
        context
            .m_vk
            .m_ashDevice
            .create_descriptor_set_layout(&info, None)
    }
    .unwrap_or(vk::DescriptorSetLayout::null());
    context.m_vkEmptyDSL
}

pub(crate) fn getOrCreateRenderPass(
    context: &mut ContextVulkan,
    key: &VKRenderPassKey,
) -> vk::RenderPass {
    if let Some((_, renderPass)) = context
        .m_vkRenderPassCache
        .iter()
        .find(|(existing, _)| existing == key)
    {
        return *renderPass;
    }
    const MAX_ATTACHMENTS: usize = 9;
    let mut attachments = Vec::with_capacity(MAX_ATTACHMENTS);
    let mut colorRefs = [vk::AttachmentReference::default(); 4];
    let mut resolveRefs = [vk::AttachmentReference::default(); 4];
    let mut depthRef = vk::AttachmentReference::default();
    let mut anyResolve = false;
    for index in 0..key.colorCount as usize {
        let attachment = attachments.len();
        attachments.push(
            vk::AttachmentDescription::default()
                .format(context.vkFormatFor(key.colorFormats[index]))
                .samples(vk::SampleCountFlags::from_raw(key.sampleCount))
                .load_op(oreLoadOpToVk(key.colorLoadOps[index]))
                .store_op(oreStoreOpToVk(key.colorStoreOps[index]))
                .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
                .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
                .initial_layout(if key.colorLoadOps[index] == LoadOp::load {
                    vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL
                } else {
                    vk::ImageLayout::UNDEFINED
                })
                .final_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL),
        );
        colorRefs[index] = vk::AttachmentReference::default()
            .attachment(attachment as u32)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);
        anyResolve |= key.colorHasResolve[index];
    }
    if anyResolve {
        for index in 0..key.colorCount as usize {
            if !key.colorHasResolve[index] {
                resolveRefs[index] = vk::AttachmentReference::default()
                    .attachment(vk::ATTACHMENT_UNUSED)
                    .layout(vk::ImageLayout::UNDEFINED);
                continue;
            }
            let attachment = attachments.len();
            attachments.push(
                vk::AttachmentDescription::default()
                    .format(context.vkFormatFor(key.colorFormats[index]))
                    .samples(vk::SampleCountFlags::TYPE_1)
                    .load_op(vk::AttachmentLoadOp::DONT_CARE)
                    .store_op(vk::AttachmentStoreOp::STORE)
                    .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
                    .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
                    .initial_layout(vk::ImageLayout::UNDEFINED)
                    .final_layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL),
            );
            resolveRefs[index] = vk::AttachmentReference::default()
                .attachment(attachment as u32)
                .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);
        }
    }
    if key.hasDepth {
        let attachment = attachments.len();
        attachments.push(
            vk::AttachmentDescription::default()
                .format(context.vkFormatFor(key.depthFormat))
                .samples(vk::SampleCountFlags::from_raw(key.sampleCount))
                .load_op(oreLoadOpToVk(key.depthLoadOp))
                .store_op(oreStoreOpToVk(key.depthStoreOp))
                .stencil_load_op(oreLoadOpToVk(key.depthLoadOp))
                .stencil_store_op(oreStoreOpToVk(key.depthStoreOp))
                .initial_layout(if key.depthLoadOp == LoadOp::load {
                    vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL
                } else {
                    vk::ImageLayout::UNDEFINED
                })
                .final_layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL),
        );
        depthRef = vk::AttachmentReference::default()
            .attachment(attachment as u32)
            .layout(vk::ImageLayout::DEPTH_STENCIL_ATTACHMENT_OPTIMAL);
    }
    assert!(attachments.len() <= MAX_ATTACHMENTS);
    let subpass = vk::SubpassDescription {
        pipeline_bind_point: vk::PipelineBindPoint::GRAPHICS,
        color_attachment_count: key.colorCount,
        p_color_attachments: colorRefs.as_ptr(),
        p_resolve_attachments: if anyResolve {
            resolveRefs.as_ptr()
        } else {
            std::ptr::null()
        },
        p_depth_stencil_attachment: if key.hasDepth {
            std::ptr::from_ref(&depthRef)
        } else {
            std::ptr::null()
        },
        ..Default::default()
    };
    let stages = vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT
        | vk::PipelineStageFlags::EARLY_FRAGMENT_TESTS;
    let deps = [
        vk::SubpassDependency::default()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_stage_mask(stages)
            .dst_stage_mask(stages)
            .dst_access_mask(
                vk::AccessFlags::COLOR_ATTACHMENT_WRITE
                    | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
            ),
        vk::SubpassDependency::default()
            .src_subpass(0)
            .dst_subpass(vk::SUBPASS_EXTERNAL)
            .src_stage_mask(stages)
            .dst_stage_mask(
                vk::PipelineStageFlags::VERTEX_SHADER | vk::PipelineStageFlags::FRAGMENT_SHADER,
            )
            .src_access_mask(
                vk::AccessFlags::COLOR_ATTACHMENT_WRITE
                    | vk::AccessFlags::DEPTH_STENCIL_ATTACHMENT_WRITE,
            )
            .dst_access_mask(vk::AccessFlags::SHADER_READ),
    ];
    let info = vk::RenderPassCreateInfo::default()
        .attachments(&attachments)
        .subpasses(std::slice::from_ref(&subpass))
        .dependencies(&deps);
    let renderPass = unsafe { context.m_vk.m_ashDevice.create_render_pass(&info, None) }
        .unwrap_or(vk::RenderPass::null());
    context.m_vkRenderPassCache.push((*key, renderPass));
    renderPass
}
