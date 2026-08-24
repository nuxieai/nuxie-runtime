//! Complete mechanical implementation translation of
//! `renderer/src/vulkan/vulkan_context.cpp`.

#![allow(non_snake_case)]

use super::vkutil_decl::{
    self, Buffer, Framebuffer, Image, ImageAccess, ImageAccessAction, ImageView, Mappability,
    Texture2D,
};
use super::vkutil_impl::vk_abort;
use super::vulkan_context_decl::{VulkanContext, VulkanFeatures};
use crate::mechanical_port::source::renderer::include::rive::renderer::gpu_hpp::IAABB;
use ash::vk;
use ash::vk::Handle;
use nuxie_ore_metal::gpu_resource::{GPUResourceManager, ResourceHandle};
use crate::mechanical_port::source::include::rive::refcnt_hpp::{make_rcp, rcp};
use nuxie_render_api::ColorInt;
use std::ffi::CStr;
use std::mem::{ManuallyDrop, transmute_copy};
use std::sync::Arc;
use vk_mem::{AllocatorCreateFlags, AllocatorCreateInfo};

unsafe fn loadInstanceCommand<T: Copy>(
    get_instance_proc_addr: vk::PFN_vkGetInstanceProcAddr,
    instance: vk::Instance,
    name: &CStr,
) -> Option<T> {
    let command = unsafe { get_instance_proc_addr(instance, name.as_ptr()) }?;
    assert_eq!(core::mem::size_of::<T>(), core::mem::size_of_val(&command));
    Some(unsafe { transmute_copy(&command) })
}

unsafe fn loadDeviceCommand<T: Copy>(
    get_device_proc_addr: vk::PFN_vkGetDeviceProcAddr,
    device: vk::Device,
    name: &CStr,
) -> Option<T> {
    let command = unsafe { get_device_proc_addr(device, name.as_ptr()) }?;
    assert_eq!(core::mem::size_of::<T>(), core::mem::size_of_val(&command));
    Some(unsafe { transmute_copy(&command) })
}

impl VulkanContext {
    /// # Safety
    /// The raw handles and loader must be a compatible live Vulkan tuple and
    /// must remain valid through `shutdown` and final context release.
    pub(crate) unsafe fn new(
        instance: vk::Instance,
        physicalDevice: vk::PhysicalDevice,
        device: vk::Device,
        features: VulkanFeatures,
        get_instance_proc_addr: vk::PFN_vkGetInstanceProcAddr,
    ) -> Arc<Self> {
        // The source GPUResourceManager base is constructed before every
        // Vulkan member and allocator initializer.
        let manager_owner = nuxie_ore_metal::gpu_resource::GPUResourceManagerOwner::new();
        let static_fn = ash::StaticFn { get_instance_proc_addr };
        let ash_instance = unsafe { ash::Instance::load(&static_fn, instance) };
        let ash_device = unsafe { ash::Device::load(ash_instance.fp_v1_0(), device) };
        let GetDeviceProcAddr: Option<vk::PFN_vkGetDeviceProcAddr> =
            unsafe { loadInstanceCommand(get_instance_proc_addr, instance, c"vkGetDeviceProcAddr") };
        let get_device_proc_addr =
            GetDeviceProcAddr.expect("Vulkan instance must publish vkGetDeviceProcAddr");
        let GetPhysicalDeviceFormatProperties = unsafe {
            loadInstanceCommand(
                get_instance_proc_addr,
                instance,
                c"vkGetPhysicalDeviceFormatProperties",
            )
        };
        let GetPhysicalDeviceProperties = unsafe {
            loadInstanceCommand(
                get_instance_proc_addr,
                instance,
                c"vkGetPhysicalDeviceProperties",
            )
        };
        let GetPhysicalDeviceFeatures = unsafe {
            loadInstanceCommand(
                get_instance_proc_addr,
                instance,
                c"vkGetPhysicalDeviceFeatures",
            )
        };
        let SetDebugUtilsObjectNameEXT = unsafe {
            loadInstanceCommand(
                get_instance_proc_addr,
                instance,
                c"vkSetDebugUtilsObjectNameEXT",
            )
        };
        let mut allocator_info = AllocatorCreateInfo::new(&ash_instance, &ash_device, physicalDevice);
        allocator_info.flags = AllocatorCreateFlags::EXTERNALLY_SYNCHRONIZED;
        allocator_info.vulkan_api_version = features.apiVersion;
        let allocator = match unsafe { vk_mem::Allocator::new(allocator_info) } {
            Ok(allocator) => allocator,
            Err(result) => vk_abort(result, file!(), line!()),
        };
        let properties = unsafe { ash_instance.get_physical_device_properties(physicalDevice) };
        assert!(properties.api_version >= features.apiVersion,
            "Supplied API version should not be newer than the physical device");
        let d24 = unsafe { ash_instance.get_physical_device_format_properties(
            physicalDevice, vk::Format::D24_UNORM_S8_UINT) }
            .optimal_tiling_features.contains(vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT);
        let d32 = unsafe { ash_instance.get_physical_device_format_properties(
            physicalDevice, vk::Format::D32_SFLOAT_S8_UINT) }
            .optimal_tiling_features.contains(vk::FormatFeatureFlags::DEPTH_STENCIL_ATTACHMENT);
        assert!(d24 || d32, "No suitable depth format supported!");

        macro_rules! load_device_command {
            ($name:ident) => {{
                let name = CStr::from_bytes_with_nul(
                    concat!("vk", stringify!($name), "\0").as_bytes(),
                )
                .expect("static Vulkan command name");
                unsafe { loadDeviceCommand(get_device_proc_addr, device, name) }
            }};
        }

        Arc::new(Self {
            m_managerOwner: manager_owner,
            instance,
            physicalDevice,
            device,
            features,
            GetDeviceProcAddr,
            GetPhysicalDeviceFormatProperties,
            GetPhysicalDeviceProperties,
            GetPhysicalDeviceFeatures,
            SetDebugUtilsObjectNameEXT,
            AllocateCommandBuffers: load_device_command!(AllocateCommandBuffers),
            AllocateDescriptorSets: load_device_command!(AllocateDescriptorSets),
            BeginCommandBuffer: load_device_command!(BeginCommandBuffer),
            CmdBeginRenderPass: load_device_command!(CmdBeginRenderPass),
            CmdBindDescriptorSets: load_device_command!(CmdBindDescriptorSets),
            CmdBindIndexBuffer: load_device_command!(CmdBindIndexBuffer),
            CmdBindPipeline: load_device_command!(CmdBindPipeline),
            CmdBindVertexBuffers: load_device_command!(CmdBindVertexBuffers),
            CmdBlitImage: load_device_command!(CmdBlitImage),
            CmdClearColorImage: load_device_command!(CmdClearColorImage),
            CmdCopyBufferToImage: load_device_command!(CmdCopyBufferToImage),
            CmdDraw: load_device_command!(CmdDraw),
            CmdDrawIndexed: load_device_command!(CmdDrawIndexed),
            CmdEndRenderPass: load_device_command!(CmdEndRenderPass),
            CmdFillBuffer: load_device_command!(CmdFillBuffer),
            CmdNextSubpass: load_device_command!(CmdNextSubpass),
            CmdPipelineBarrier: load_device_command!(CmdPipelineBarrier),
            CmdSetBlendConstants: load_device_command!(CmdSetBlendConstants),
            CmdSetColorWriteEnableEXT: load_device_command!(CmdSetColorWriteEnableEXT),
            CmdSetCullMode: load_device_command!(CmdSetCullMode),
            CmdSetDepthWriteEnable: load_device_command!(CmdSetDepthWriteEnable),
            CmdSetScissor: load_device_command!(CmdSetScissor),
            CmdSetStencilCompareMask: load_device_command!(CmdSetStencilCompareMask),
            CmdSetStencilOp: load_device_command!(CmdSetStencilOp),
            CmdSetStencilReference: load_device_command!(CmdSetStencilReference),
            CmdSetStencilWriteMask: load_device_command!(CmdSetStencilWriteMask),
            CmdSetViewport: load_device_command!(CmdSetViewport),
            CreateCommandPool: load_device_command!(CreateCommandPool),
            CreateDescriptorPool: load_device_command!(CreateDescriptorPool),
            CreateDescriptorSetLayout: load_device_command!(CreateDescriptorSetLayout),
            CreateFramebuffer: load_device_command!(CreateFramebuffer),
            CreateFence: load_device_command!(CreateFence),
            CreateGraphicsPipelines: load_device_command!(CreateGraphicsPipelines),
            CreateImageView: load_device_command!(CreateImageView),
            CreatePipelineLayout: load_device_command!(CreatePipelineLayout),
            CreateRenderPass: load_device_command!(CreateRenderPass),
            CreateSampler: load_device_command!(CreateSampler),
            CreateShaderModule: load_device_command!(CreateShaderModule),
            DestroyCommandPool: load_device_command!(DestroyCommandPool),
            DestroyDescriptorPool: load_device_command!(DestroyDescriptorPool),
            DestroyDescriptorSetLayout: load_device_command!(DestroyDescriptorSetLayout),
            DestroyFence: load_device_command!(DestroyFence),
            DestroyFramebuffer: load_device_command!(DestroyFramebuffer),
            DestroyImageView: load_device_command!(DestroyImageView),
            DestroyPipeline: load_device_command!(DestroyPipeline),
            DestroyPipelineLayout: load_device_command!(DestroyPipelineLayout),
            DestroyRenderPass: load_device_command!(DestroyRenderPass),
            DestroySampler: load_device_command!(DestroySampler),
            DestroyShaderModule: load_device_command!(DestroyShaderModule),
            EndCommandBuffer: load_device_command!(EndCommandBuffer),
            FreeCommandBuffers: load_device_command!(FreeCommandBuffers),
            FreeDescriptorSets: load_device_command!(FreeDescriptorSets),
            QueueSubmit: load_device_command!(QueueSubmit),
            QueueWaitIdle: load_device_command!(QueueWaitIdle),
            ResetCommandBuffer: load_device_command!(ResetCommandBuffer),
            ResetDescriptorPool: load_device_command!(ResetDescriptorPool),
            ResetFences: load_device_command!(ResetFences),
            UpdateDescriptorSets: load_device_command!(UpdateDescriptorSets),
            WaitForFences: load_device_command!(WaitForFences),
            m_ashInstance: ash_instance,
            m_ashDevice: ash_device,
            m_vmaAllocator: ManuallyDrop::new(allocator),
            m_physicalDeviceProperties: properties,
            m_supportsD24S8: d24,
        })
    }

    pub(crate) fn shutdown(&self) { self.m_managerOwner.shutdown(); }

    pub(crate) fn advanceFrameNumber(&self, current: u64, safe: u64) {
        self.manager().advanceFrameNumber(current, safe);
    }

    pub(crate) fn currentFrameNumber(&self) -> u64 { self.manager().currentFrameNumber() }

    pub(crate) fn safeFrameNumber(&self) -> u64 { self.manager().safeFrameNumber() }

    pub(crate) fn isFormatSupportedWithFeatureFlags(
        &self, format: vk::Format, flags: vk::FormatFeatureFlags,
    ) -> bool {
        unsafe { self.m_ashInstance.get_physical_device_format_properties(self.physicalDevice, format) }
            .optimal_tiling_features.contains(flags)
    }

    pub(crate) fn makeBuffer(
        self: &Arc<Self>, info: vk::BufferCreateInfo<'_>, mappability: Mappability,
    ) -> ResourceHandle<Buffer> {
        let payload = Buffer::new(Arc::clone(self), info, mappability);
        ResourceHandle::new(Some(self.manager()), payload)
    }

    pub(crate) fn makeImage(
        self: &Arc<Self>, info: vk::ImageCreateInfo<'_>, name: Option<&CStr>,
    ) -> ResourceHandle<Image> {
        let payload = Image::new(Arc::clone(self), info, name);
        ResourceHandle::new(Some(self.manager()), payload)
    }

    pub(crate) fn makeExternalImage(
        self: &Arc<Self>, image: vk::Image, info: vk::ImageCreateInfo<'_>, name: Option<&CStr>,
    ) -> ResourceHandle<Image> {
        let payload = Image::new_external(Arc::clone(self), image, info, name);
        ResourceHandle::new(Some(self.manager()), payload)
    }

    pub(crate) fn makeFramebuffer(
        self: &Arc<Self>, info: vk::FramebufferCreateInfo<'_>,
    ) -> ResourceHandle<Framebuffer> {
        let payload = Framebuffer::new(Arc::clone(self), info);
        ResourceHandle::new(Some(self.manager()), payload)
    }

    pub(crate) fn makeImageView(
        self: &Arc<Self>, image: ResourceHandle<Image>, name: Option<&CStr>,
    ) -> ResourceHandle<ImageView> {
        let image_info = image.info();
        let info = vk::ImageViewCreateInfo::default()
            .view_type(image_view_type_for_image_type(image_info.image_type))
            .format(image_info.format)
            .subresource_range(vk::ImageSubresourceRange {
                aspect_mask: image_aspect_flags_for_format(image_info.format),
                level_count: image_info.mip_levels,
                layer_count: 1,
                ..Default::default()
            });
        self.makeImageViewWithInfo(image, info, name)
    }

    pub(crate) fn makeImageViewWithInfo(
        self: &Arc<Self>, image: ResourceHandle<Image>, info: vk::ImageViewCreateInfo<'_>,
        name: Option<&CStr>,
    ) -> ResourceHandle<ImageView> {
        let payload = ImageView::new(Arc::clone(self), Some(image), info, name);
        ResourceHandle::new(Some(self.manager()), payload)
    }

    pub(crate) fn makeExternalImageView(
        self: &Arc<Self>, info: vk::ImageViewCreateInfo<'_>, name: Option<&CStr>,
    ) -> ResourceHandle<ImageView> {
        let payload = ImageView::new(Arc::clone(self), None, info, name);
        ResourceHandle::new(Some(self.manager()), payload)
    }

    pub(crate) fn makeTexture2D(
        self: &Arc<Self>, info: vk::ImageCreateInfo<'_>, name: Option<&CStr>,
    ) -> rcp<Texture2D> {
        make_rcp(|| Texture2D::new(Arc::clone(self), info, name))
    }

    pub(crate) fn makeTexture2DFromImage(
        self: &Arc<Self>, image: ResourceHandle<Image>, name: Option<&CStr>,
    ) -> rcp<Texture2D> {
        make_rcp(|| Texture2D::new_from_image(Arc::clone(self), image, name))
    }

    pub(crate) fn updateImageDescriptorSets(
        &self, set: vk::DescriptorSet, mut write: vk::WriteDescriptorSet<'_>,
        infos: &[vk::DescriptorImageInfo],
    ) {
        write.s_type = vk::StructureType::WRITE_DESCRIPTOR_SET;
        write.dst_set = set;
        write.descriptor_count = u32::try_from(infos.len()).unwrap();
        write.p_image_info = infos.as_ptr();
        unsafe { self.m_ashDevice.update_descriptor_sets(&[write], &[]) };
    }

    pub(crate) fn updateBufferDescriptorSets(
        &self, set: vk::DescriptorSet, mut write: vk::WriteDescriptorSet<'_>,
        infos: &[vk::DescriptorBufferInfo],
    ) {
        write.s_type = vk::StructureType::WRITE_DESCRIPTOR_SET;
        write.dst_set = set;
        write.descriptor_count = u32::try_from(infos.len()).unwrap();
        write.p_buffer_info = infos.as_ptr();
        unsafe { self.m_ashDevice.update_descriptor_sets(&[write], &[]) };
    }

    pub(crate) fn memoryBarrier(&self, command: vk::CommandBuffer,
        src: vk::PipelineStageFlags, dst: vk::PipelineStageFlags,
        dependency: vk::DependencyFlags, mut barrier: vk::MemoryBarrier<'_>) {
        barrier.s_type = vk::StructureType::MEMORY_BARRIER;
        unsafe { self.m_ashDevice.cmd_pipeline_barrier(command, src, dst, dependency,
            &[barrier], &[], &[]) };
    }

    pub(crate) fn imageMemoryBarriers(&self, command: vk::CommandBuffer,
        src: vk::PipelineStageFlags, dst: vk::PipelineStageFlags,
        dependency: vk::DependencyFlags, barriers: &mut [vk::ImageMemoryBarrier<'_>]) {
        for barrier in barriers.iter_mut() {
            barrier.s_type = vk::StructureType::IMAGE_MEMORY_BARRIER;
            if barrier.subresource_range.aspect_mask.is_empty() {
                barrier.subresource_range.aspect_mask = vk::ImageAspectFlags::COLOR;
            }
            if barrier.subresource_range.level_count == 0 {
                barrier.subresource_range.level_count = vk::REMAINING_MIP_LEVELS;
            }
            if barrier.subresource_range.layer_count == 0 {
                barrier.subresource_range.layer_count = vk::REMAINING_ARRAY_LAYERS;
            }
        }
        unsafe { self.m_ashDevice.cmd_pipeline_barrier(command, src, dst, dependency,
            &[], &[], barriers) };
    }

    pub(crate) fn imageMemoryBarrier(&self, command: vk::CommandBuffer,
        src: vk::PipelineStageFlags, dst: vk::PipelineStageFlags,
        dependency: vk::DependencyFlags, mut barrier: vk::ImageMemoryBarrier<'_>) {
        self.imageMemoryBarriers(command, src, dst, dependency, core::slice::from_mut(&mut barrier));
    }

    pub(crate) fn simpleImageMemoryBarrier(&self, command: vk::CommandBuffer,
        src: ImageAccess, dst: ImageAccess, image: vk::Image,
        action: ImageAccessAction, dependency: vk::DependencyFlags) -> ImageAccess {
        assert_ne!(image, vk::Image::null());
        if src != dst {
            self.imageMemoryBarrier(command, src.pipelineStages, dst.pipelineStages, dependency,
                vk::ImageMemoryBarrier::default()
                    .src_access_mask(src.accessMask).dst_access_mask(dst.accessMask)
                    .old_layout(if action == ImageAccessAction::preserveContents { src.layout }
                        else { vk::ImageLayout::UNDEFINED })
                    .new_layout(dst.layout).image(image));
        }
        dst
    }

    pub(crate) fn bufferMemoryBarrier(&self, command: vk::CommandBuffer,
        src: vk::PipelineStageFlags, dst: vk::PipelineStageFlags,
        dependency: vk::DependencyFlags, mut barrier: vk::BufferMemoryBarrier<'_>) {
        barrier.s_type = vk::StructureType::BUFFER_MEMORY_BARRIER;
        if barrier.size == 0 { barrier.size = vk::WHOLE_SIZE; }
        unsafe { self.m_ashDevice.cmd_pipeline_barrier(command, src, dst, dependency,
            &[], &[barrier], &[]) };
    }

    pub(crate) fn clearColorImage(&self, command: vk::CommandBuffer, color: ColorInt,
        image: vk::Image, layout: vk::ImageLayout) {
        let value = vkutil_decl::color_clear_rgba32f(color);
        let range = vk::ImageSubresourceRange { aspect_mask: vk::ImageAspectFlags::COLOR,
            level_count: 1, layer_count: 1, ..Default::default() };
        unsafe { self.m_ashDevice.cmd_clear_color_image(command, image, layout, &value, &[range]) };
    }

    pub(crate) fn blitSubRect(&self, command: vk::CommandBuffer, src: vk::Image,
        src_layout: vk::ImageLayout, dst: vk::Image, dst_layout: vk::ImageLayout,
        bounds: &IAABB) {
        if bounds.empty() { return; }
        let sub = vk::ImageSubresourceLayers { aspect_mask: vk::ImageAspectFlags::COLOR,
            layer_count: 1, ..Default::default() };
        let blit = vk::ImageBlit { src_subresource: sub,
            src_offsets: [vk::Offset3D { x: bounds.left, y: bounds.top, z: 0 },
                vk::Offset3D { x: bounds.right, y: bounds.bottom, z: 1 }],
            dst_subresource: sub,
            dst_offsets: [vk::Offset3D { x: bounds.left, y: bounds.top, z: 0 },
                vk::Offset3D { x: bounds.right, y: bounds.bottom, z: 1 }] };
        unsafe { self.m_ashDevice.cmd_blit_image(command, src, src_layout, dst, dst_layout,
            &[blit], vk::Filter::NEAREST) };
    }

    pub(crate) fn setDebugNameIfEnabled<T: Handle>(&self, handle: T,
        object_type: vk::ObjectType, name: Option<&CStr>) {
        let (Some(function), Some(name)) = (self.SetDebugUtilsObjectNameEXT, name) else { return; };
        let info = vk::DebugUtilsObjectNameInfoEXT {
            object_type,
            object_handle: handle.as_raw(),
            p_object_name: name.as_ptr(),
            ..Default::default()
        };
        let _ = unsafe { function(self.device, &info) };
    }
}

fn image_view_type_for_image_type(image_type: vk::ImageType) -> vk::ImageViewType {
    match image_type { vk::ImageType::TYPE_2D => vk::ImageViewType::TYPE_2D,
        _ => panic!("unsupported Vulkan image type {image_type:?}") }
}
fn image_aspect_flags_for_format(format: vk::Format) -> vk::ImageAspectFlags {
    match format {
        vk::Format::D24_UNORM_S8_UINT | vk::Format::D32_SFLOAT_S8_UINT =>
            vk::ImageAspectFlags::DEPTH | vk::ImageAspectFlags::STENCIL,
        _ => vk::ImageAspectFlags::COLOR,
    }
}

impl Drop for VulkanContext {
    fn drop(&mut self) {
        unsafe { ManuallyDrop::drop(&mut self.m_vmaAllocator) };
    }
}
