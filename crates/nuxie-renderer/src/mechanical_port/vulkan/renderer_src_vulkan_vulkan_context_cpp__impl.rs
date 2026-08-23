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
use std::mem::{ManuallyDrop, transmute};
use std::sync::Arc;
use vk_mem::{AllocatorCreateFlags, AllocatorCreateInfo};

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
        let debug_name = CStr::from_bytes_with_nul(b"vkSetDebugUtilsObjectNameEXT\0").unwrap();
        let raw_debug = unsafe { get_instance_proc_addr(instance, debug_name.as_ptr()) };
        let debug_fn = raw_debug.map(|function| unsafe { transmute(function) });
        let get_device_proc_addr = ash_instance.fp_v1_0().get_device_proc_addr;
        let color_write_name = CStr::from_bytes_with_nul(b"vkCmdSetColorWriteEnableEXT\0").unwrap();
        let cull_mode_name = CStr::from_bytes_with_nul(b"vkCmdSetCullMode\0").unwrap();
        let depth_write_name = CStr::from_bytes_with_nul(b"vkCmdSetDepthWriteEnable\0").unwrap();
        let cmd_set_color_write_enable_ext: Option<vk::PFN_vkCmdSetColorWriteEnableEXT> =
            unsafe { get_device_proc_addr(device, color_write_name.as_ptr()) }
                .map(|function| unsafe { transmute(function) });
        let cmd_set_cull_mode: Option<vk::PFN_vkCmdSetCullMode> =
            unsafe { get_device_proc_addr(device, cull_mode_name.as_ptr()) }
                .map(|function| unsafe { transmute(function) });
        let cmd_set_depth_write_enable: Option<vk::PFN_vkCmdSetDepthWriteEnable> =
            unsafe { get_device_proc_addr(device, depth_write_name.as_ptr()) }
                .map(|function| unsafe { transmute(function) });
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

        Arc::new(Self {
            m_managerOwner: manager_owner,
            instance,
            physicalDevice,
            device,
            features,
            m_ashInstance: ash_instance,
            m_ashDevice: ash_device,
            m_setDebugUtilsObjectNameEXT: debug_fn,
            CmdSetColorWriteEnableEXT: cmd_set_color_write_enable_ext,
            CmdSetCullMode: cmd_set_cull_mode,
            CmdSetDepthWriteEnable: cmd_set_depth_write_enable,
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
        let (Some(function), Some(name)) = (self.m_setDebugUtilsObjectNameEXT, name) else { return; };
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
