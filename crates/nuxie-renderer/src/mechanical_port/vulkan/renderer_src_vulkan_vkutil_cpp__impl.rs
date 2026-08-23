//! Complete mechanical implementation translation of
//! `renderer/src/vulkan/vkutil.cpp`.

#![allow(non_snake_case)]

use super::vkutil_decl::*;
use super::vulkan_context_decl::VulkanContext;
use crate::mechanical_port::source::renderer::include::rive::renderer::texture_hpp::Texture;
use ash::vk;
use ash::vk::Handle;
use nuxie_ore_metal::gpu_resource::{AnyResourceHandle, GPUResourcePool, ResourceHandle};
use std::ffi::CStr;
use std::mem::transmute;
use std::sync::Arc;
use vk_mem::{Alloc, AllocationCreateFlags, AllocationCreateInfo, MemoryUsage};

pub(crate) fn string_from_vk_result(result: vk::Result) -> &'static str {
    match result {
        vk::Result::SUCCESS => "SUCCESS", vk::Result::NOT_READY => "NOT_READY",
        vk::Result::TIMEOUT => "TIMEOUT", vk::Result::EVENT_SET => "EVENT_SET",
        vk::Result::EVENT_RESET => "EVENT_RESET", vk::Result::INCOMPLETE => "INCOMPLETE",
        vk::Result::ERROR_OUT_OF_HOST_MEMORY => "ERROR_OUT_OF_HOST_MEMORY",
        vk::Result::ERROR_OUT_OF_DEVICE_MEMORY => "ERROR_OUT_OF_DEVICE_MEMORY",
        vk::Result::ERROR_INITIALIZATION_FAILED => "ERROR_INITIALIZATION_FAILED",
        vk::Result::ERROR_DEVICE_LOST => "ERROR_DEVICE_LOST",
        vk::Result::ERROR_MEMORY_MAP_FAILED => "ERROR_MEMORY_MAP_FAILED",
        vk::Result::ERROR_LAYER_NOT_PRESENT => "ERROR_LAYER_NOT_PRESENT",
        vk::Result::ERROR_EXTENSION_NOT_PRESENT => "ERROR_EXTENSION_NOT_PRESENT",
        vk::Result::ERROR_FEATURE_NOT_PRESENT => "ERROR_FEATURE_NOT_PRESENT",
        vk::Result::ERROR_INCOMPATIBLE_DRIVER => "ERROR_INCOMPATIBLE_DRIVER",
        vk::Result::ERROR_TOO_MANY_OBJECTS => "ERROR_TOO_MANY_OBJECTS",
        vk::Result::ERROR_FORMAT_NOT_SUPPORTED => "ERROR_FORMAT_NOT_SUPPORTED",
        vk::Result::ERROR_SURFACE_LOST_KHR => "ERROR_SURFACE_LOST_KHR",
        vk::Result::SUBOPTIMAL_KHR => "SUBOPTIMAL_KHR",
        vk::Result::ERROR_OUT_OF_DATE_KHR => "ERROR_OUT_OF_DATE_KHR",
        vk::Result::ERROR_INCOMPATIBLE_DISPLAY_KHR => "ERROR_INCOMPATIBLE_DISPLAY_KHR",
        vk::Result::ERROR_NATIVE_WINDOW_IN_USE_KHR => "ERROR_NATIVE_WINDOW_IN_USE_KHR",
        vk::Result::ERROR_VALIDATION_FAILED_EXT => "ERROR_VALIDATION_FAILED_EXT",
        vk::Result::ERROR_OUT_OF_POOL_MEMORY => "ERROR_OUT_OF_POOL_MEMORY",
        _ => "<unknown>",
    }
}

pub(crate) fn vk_check(result: vk::Result, file: &str, line: u32) {
    if result == vk::Result::SUCCESS {
        return;
    }
    vk_abort(result, file, line)
}

pub(crate) fn vk_abort<T>(result: vk::Result, file: &str, line: u32) -> T {
    eprintln!("Vulkan error {} ({}) at line: {} in file: {}",
        string_from_vk_result(result), result.as_raw(), line, file);
    std::process::abort()
}

impl Buffer {
    pub(crate) fn new(vk: Arc<VulkanContext>, info: vk::BufferCreateInfo<'_>, map: Mappability) -> Self {
        let mut info: vk::BufferCreateInfo<'static> = unsafe { transmute(info) };
        info.s_type = vk::StructureType::BUFFER_CREATE_INFO;
        let buffer = Self { base: std::mem::ManuallyDrop::new(Resource::new(vk)), m_mappability: map,
            m_info: std::cell::UnsafeCell::new(info),
            m_vmaAllocation: std::cell::UnsafeCell::new(None),
            m_vkBuffer: std::cell::UnsafeCell::new(vk::Buffer::null()),
            m_contents: std::cell::UnsafeCell::new(core::ptr::null_mut()) };
        buffer.init(); buffer
    }
    pub(crate) fn info(&self) -> vk::BufferCreateInfo<'static> { unsafe { *self.m_info.get() } }
    pub(crate) fn vkBuffer(&self) -> vk::Buffer { unsafe { *self.m_vkBuffer.get() } }
    pub(crate) fn vkBufferAddressOf(&self) -> *const vk::Buffer { self.m_vkBuffer.get() }
    pub(crate) fn contents(&self) -> *mut u8 {
        let contents = unsafe { *self.m_contents.get() }; assert!(!contents.is_null()); contents
    }
    pub(crate) fn resizeImmediately(&self, size: vk::DeviceSize) {
        unsafe {
            if (*self.m_info.get()).size == size { return; }
            if let Some(mut allocation) = (*self.m_vmaAllocation.get()).take() {
                if self.m_mappability != Mappability::none {
                    self.vk().allocator().unmap_memory(&mut allocation);
                }
                self.vk().allocator().destroy_buffer(*self.m_vkBuffer.get(), &mut allocation);
            }
            (*self.m_info.get()).size = size; self.init();
        }
    }
    fn init(&self) {
        unsafe {
            if (*self.m_info.get()).size == 0 {
                *self.m_vkBuffer.get() = vk::Buffer::null();
                *self.m_vmaAllocation.get() = None;
                *self.m_contents.get() = core::ptr::null_mut(); return;
            }
            let allocation_info = AllocationCreateInfo { flags: vma_flags_for_mappability(self.m_mappability),
                usage: MemoryUsage::Auto, ..Default::default() };
            let (buffer, mut allocation) = match self.vk().allocator().create_buffer(&*self.m_info.get(), &allocation_info) {
                Ok(value) => value, Err(error) => vk_abort(error, file!(), line!()),
            };
            *self.m_vkBuffer.get() = buffer;
            *self.m_contents.get() = if self.m_mappability != Mappability::none {
                match self.vk().allocator().map_memory(&mut allocation) {
                    Ok(pointer) => pointer, Err(error) => vk_abort(error, file!(), line!()),
                }
            } else { core::ptr::null_mut() };
            *self.m_vmaAllocation.get() = Some(allocation);
        }
    }
    pub(crate) fn flushContents(&self, size: vk::DeviceSize) {
        let allocation = unsafe { (&*self.m_vmaAllocation.get()).as_ref() }
            .expect("flushContents requires an allocated buffer");
        let _ = self.vk().allocator().flush_allocation(allocation, 0, size);
    }
    pub(crate) fn flushAllContents(&self) { self.flushContents(vk::WHOLE_SIZE); }
    pub(crate) fn invalidateContents(&self, size: vk::DeviceSize) {
        let allocation = unsafe { (&*self.m_vmaAllocation.get()).as_ref() }
            .expect("invalidateContents requires an allocated buffer");
        let _ = self.vk().allocator().invalidate_allocation(allocation, 0, size);
    }
    pub(crate) fn invalidateAllContents(&self) { self.invalidateContents(vk::WHOLE_SIZE); }
}
impl Drop for Buffer {
    fn drop(&mut self) {
        self.resizeImmediately(0);
        unsafe { std::mem::ManuallyDrop::drop(&mut self.base) };
    }
}

pub(crate) fn vma_flags_for_mappability(map: Mappability) -> AllocationCreateFlags {
    match map { Mappability::none => AllocationCreateFlags::empty(),
        Mappability::writeOnly => AllocationCreateFlags::HOST_ACCESS_SEQUENTIAL_WRITE,
        Mappability::readWrite => AllocationCreateFlags::HOST_ACCESS_RANDOM }
}

impl BufferPool {
    pub(crate) fn new(vk: Arc<VulkanContext>, usage: vk::BufferUsageFlags, size: vk::DeviceSize) -> Self {
        let base = nuxie_ore_metal::new_gpu_resource_pool_backend_base(
            vk.manager(), Self::MAX_POOL_SIZE);
        Self { base: std::mem::ManuallyDrop::new(base),
            m_vk: std::mem::ManuallyDrop::new(vk), m_usageFlags: usage,
            m_targetSize: std::cell::UnsafeCell::new(size) }
    }
    pub(crate) fn size(&self) -> vk::DeviceSize { unsafe { *self.m_targetSize.get() } }
    pub(crate) fn setTargetSize(&self, mut size: vk::DeviceSize) {
        size = size.max(1);
        if self.m_usageFlags.contains(vk::BufferUsageFlags::UNIFORM_BUFFER) {
            size = size.max(256); assert_eq!(size % 256, 0);
        }
        unsafe { *self.m_targetSize.get() = size };
    }
    pub(crate) fn acquire(&self) -> ResourceHandle<Buffer> {
        if let Some(resource) = self.base.acquire() {
            let buffer = resource.downcast::<Buffer>().ok().expect("buffer pool type drift");
            if buffer.info().size != self.size() { buffer.resizeImmediately(self.size()); }
            buffer
        } else {
            self.m_vk.makeBuffer(vk::BufferCreateInfo::default().size(self.size())
                .usage(self.m_usageFlags), Mappability::writeOnly)
        }
    }
    pub(crate) fn recycle(&self, buffer: ResourceHandle<Buffer>) {
        self.base.recycle(Some(buffer.erase()));
    }
}

impl Drop for BufferPool {
    fn drop(&mut self) {
        unsafe {
            std::mem::ManuallyDrop::drop(&mut self.base);
            std::mem::ManuallyDrop::drop(&mut self.m_vk);
        }
    }
}

impl Image {
    pub(crate) fn new(vk: Arc<VulkanContext>, info: vk::ImageCreateInfo<'_>, name: Option<&CStr>) -> Self {
        let mut info: vk::ImageCreateInfo<'static> = unsafe { transmute(info) };
        info.s_type = vk::StructureType::IMAGE_CREATE_INFO;
        if info.mip_levels == 0 { info.mip_levels = 1; }
        else if info.mip_levels > 1 { info.usage |= vk::ImageUsageFlags::TRANSFER_SRC; }
        if info.array_layers == 0 { info.array_layers = 1; }
        if info.samples.is_empty() { info.samples = vk::SampleCountFlags::TYPE_1; }
        let mut allocation_info = AllocationCreateInfo {
            usage: MemoryUsage::Auto,
            ..Default::default()
        };
        let lazy = info.usage.contains(vk::ImageUsageFlags::TRANSIENT_ATTACHMENT);
        if lazy { allocation_info.usage = MemoryUsage::GpuLazy; }
        let attempt = unsafe { vk.allocator().create_image(&info, &allocation_info) };
        let (image, allocation) = match attempt {
            Ok(value) => value,
            Err(_) if lazy => {
                allocation_info.usage = MemoryUsage::Auto;
                match unsafe { vk.allocator().create_image(&info, &allocation_info) } {
                    Ok(value) => value, Err(error) => vk_abort(error, file!(), line!()),
                }
            }
            Err(error) => vk_abort(error, file!(), line!()),
        };
        vk.setDebugNameIfEnabled(image, vk::ObjectType::IMAGE, name);
        Self { base: std::mem::ManuallyDrop::new(Resource::new(vk)), m_info: info,
            m_vmaAllocation: std::cell::UnsafeCell::new(Some(allocation)), m_vkImage: image }
    }
    pub(crate) fn new_external(vk: Arc<VulkanContext>, image: vk::Image,
        info: vk::ImageCreateInfo<'_>, name: Option<&CStr>) -> Self {
        let mut info: vk::ImageCreateInfo<'static> = unsafe { transmute(info) };
        info.s_type = vk::StructureType::IMAGE_CREATE_INFO;
        vk.setDebugNameIfEnabled(image, vk::ObjectType::IMAGE, name);
        Self { base: std::mem::ManuallyDrop::new(Resource::new(vk)), m_info: info,
            m_vmaAllocation: std::cell::UnsafeCell::new(None), m_vkImage: image }
    }
    pub(crate) fn info(&self) -> vk::ImageCreateInfo<'static> { self.m_info }
    pub(crate) fn vkImage(&self) -> vk::Image { self.m_vkImage }
    pub(crate) fn vkImageAddressOf(&self) -> *const vk::Image { &self.m_vkImage }
}
impl Drop for Image {
    fn drop(&mut self) {
        if let Some(mut allocation) = unsafe { (&mut *self.m_vmaAllocation.get()).take() } {
            unsafe { self.vk().allocator().destroy_image(self.m_vkImage, &mut allocation) };
        }
        unsafe { std::mem::ManuallyDrop::drop(&mut self.base) };
    }
}

impl ImageView {
    pub(crate) fn new(vk: Arc<VulkanContext>, image_ref: Option<ResourceHandle<Image>>,
        info: vk::ImageViewCreateInfo<'_>, name: Option<&CStr>) -> Self {
        let mut info: vk::ImageViewCreateInfo<'static> = unsafe { transmute(info) };
        if info.image == vk::Image::null() {
            info.image = image_ref.as_ref().expect("image view requires image").vkImage();
        } else if let Some(image) = image_ref.as_ref() { assert_eq!(info.image, image.vkImage()); }
        info.s_type = vk::StructureType::IMAGE_VIEW_CREATE_INFO;
        let image_view = match unsafe { vk.ashDevice().create_image_view(&info, None) } {
            Ok(value) => value, Err(error) => vk_abort(error, file!(), line!()),
        };
        vk.setDebugNameIfEnabled(image_view, vk::ObjectType::IMAGE_VIEW, name);
        Self { base: std::mem::ManuallyDrop::new(Resource::new(vk)),
            m_textureRefOrNull: std::mem::ManuallyDrop::new(image_ref),
            m_info: info, m_vkImageView: image_view }
    }
    pub(crate) fn info(&self) -> vk::ImageViewCreateInfo<'static> { self.m_info }
    pub(crate) fn vkImageView(&self) -> vk::ImageView { self.m_vkImageView }
    pub(crate) fn vkImageViewAddressOf(&self) -> *const vk::ImageView { &self.m_vkImageView }
}
impl Drop for ImageView {
    fn drop(&mut self) {
        unsafe {
            self.vk().ashDevice().destroy_image_view(self.m_vkImageView, None);
            std::mem::ManuallyDrop::drop(&mut self.m_textureRefOrNull);
            std::mem::ManuallyDrop::drop(&mut self.base);
        }
    }
}

impl Texture2D {
    pub(crate) fn new(vk: Arc<VulkanContext>, info: vk::ImageCreateInfo<'_>, name: Option<&CStr>) -> Self {
        let mut info = info;
        let base = texture2d_base(info.extent.width, info.extent.height);
        if info.image_type.as_raw() == 0 { info.image_type = vk::ImageType::TYPE_2D; }
        assert_eq!(info.image_type, vk::ImageType::TYPE_2D);
        if info.extent.depth == 0 { info.extent.depth = 1; }
        if info.usage.is_empty() { info.usage = vk::ImageUsageFlags::SAMPLED | vk::ImageUsageFlags::TRANSFER_DST; }
        let image = vk.makeImage(info, name);
        Self::new_from_base_and_image(base, vk, image, name)
    }
    pub(crate) fn new_from_image(vk: Arc<VulkanContext>, image: ResourceHandle<Image>, name: Option<&CStr>) -> Self {
        let info = image.info();
        let base = texture2d_base(info.extent.width, info.extent.height);
        Self::new_from_base_and_image(base, vk, image, name)
    }
    fn new_from_base_and_image(base: Texture, vk: Arc<VulkanContext>,
        image: ResourceHandle<Image>, name: Option<&CStr>) -> Self {
        let view = vk.makeImageView(image.clone(), name);
        Self { base: std::mem::ManuallyDrop::new(base),
            m_image: std::mem::ManuallyDrop::new(image),
            m_imageView: std::mem::ManuallyDrop::new(view),
            m_lastAccess: std::cell::UnsafeCell::new(ImageAccess::default()),
            m_imageUploadBuffer: std::mem::ManuallyDrop::new(std::cell::UnsafeCell::new(None)),
            m_imageUploadRegions: std::mem::ManuallyDrop::new(std::cell::UnsafeCell::new(Vec::new())),
            m_cachedDescriptorSet: std::cell::UnsafeCell::new(vk::DescriptorSet::null()),
            m_cachedDescriptorSetFrameNumber: std::cell::UnsafeCell::new(0),
            m_cachedDescriptorSetSampler: std::cell::UnsafeCell::new(nuxie_render_api::ImageSampler::default()) }
    }
    pub(crate) fn width(&self) -> u32 { self.base.width() }
    pub(crate) fn height(&self) -> u32 { self.base.height() }
    pub(crate) fn vkImage(&self) -> vk::Image { self.m_image.vkImage() }
    pub(crate) fn vkImageView(&self) -> vk::ImageView { self.m_imageView.vkImageView() }
    pub(crate) fn vkImageViewAddressOf(&self) -> *const vk::ImageView {
        self.m_imageView.vkImageViewAddressOf()
    }
    pub(crate) fn nativeHandle(&self) -> *mut core::ffi::c_void {
        self.vkImage().as_raw() as usize as *mut core::ffi::c_void
    }
    pub(crate) fn lastAccess(&self) -> ImageAccess { unsafe { *self.m_lastAccess.get() } }
    /// # Safety
    /// Mirrors the source's mutable reference accessor. The caller must uphold
    /// the renderer's single-threaded, non-aliasing mutation discipline.
    pub(crate) unsafe fn lastAccessMut(&self) -> &mut ImageAccess {
        unsafe { &mut *self.m_lastAccess.get() }
    }
    pub(crate) fn scheduleUploadBytes(&self, bytes: &[u8]) {
        let buffer = self.m_image.m_vk.makeBuffer(vk::BufferCreateInfo::default()
            .size(bytes.len() as u64).usage(vk::BufferUsageFlags::TRANSFER_SRC), Mappability::writeOnly);
        unsafe { core::ptr::copy_nonoverlapping(bytes.as_ptr(), buffer.contents(), bytes.len()) };
        buffer.flushAllContents(); self.scheduleUpload(buffer);
    }
    pub(crate) fn scheduleUpload(&self, buffer: ResourceHandle<Buffer>) {
        unsafe { *self.m_imageUploadBuffer.get() = Some(buffer); (&mut *self.m_imageUploadRegions.get()).clear(); }
    }
    pub(crate) fn scheduleUploadRegions(&self, buffer: ResourceHandle<Buffer>, regions: Vec<vk::BufferImageCopy>) {
        unsafe { *self.m_imageUploadBuffer.get() = Some(buffer); *self.m_imageUploadRegions.get() = regions; }
    }
    pub(crate) fn prepareForVertexOrFragmentShaderRead(&self, command: vk::CommandBuffer) {
        if unsafe { (&*self.m_imageUploadBuffer.get()).is_some() } { self.applyImageUploadBuffer(command); }
        let access = ImageAccess { pipelineStages: vk::PipelineStageFlags::VERTEX_SHADER |
            vk::PipelineStageFlags::FRAGMENT_SHADER, accessMask: vk::AccessFlags::SHADER_READ,
            layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL };
        if self.lastAccess() != access { self.barrier(command, access, ImageAccessAction::preserveContents, vk::DependencyFlags::empty()); }
    }
    pub(crate) fn prepareForFragmentShaderRead(&self, command: vk::CommandBuffer) {
        if unsafe { (&*self.m_imageUploadBuffer.get()).is_some() } { self.applyImageUploadBuffer(command); }
        let access = ImageAccess { pipelineStages: vk::PipelineStageFlags::FRAGMENT_SHADER,
            accessMask: vk::AccessFlags::SHADER_READ, layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL };
        if self.lastAccess() != access { self.barrier(command, access, ImageAccessAction::preserveContents, vk::DependencyFlags::empty()); }
    }
    pub(crate) fn getCachedDescriptorSet(&self, frame: u64, sampler: nuxie_render_api::ImageSampler) -> vk::DescriptorSet {
        unsafe { if frame == *self.m_cachedDescriptorSetFrameNumber.get() && sampler == *self.m_cachedDescriptorSetSampler.get()
            { *self.m_cachedDescriptorSet.get() } else { vk::DescriptorSet::null() } }
    }
    pub(crate) fn updateCachedDescriptorSet(&self, set: vk::DescriptorSet, frame: u64, sampler: nuxie_render_api::ImageSampler) {
        unsafe { *self.m_cachedDescriptorSet.get() = set; *self.m_cachedDescriptorSetFrameNumber.get() = frame;
            *self.m_cachedDescriptorSetSampler.get() = sampler; }
    }
    pub(crate) fn overrideLastAccess(&self, access: ImageAccess) { unsafe { *self.m_lastAccess.get() = access }; }
    fn applyImageUploadBuffer(&self, command: vk::CommandBuffer) {
        let buffer = unsafe { (&*self.m_imageUploadBuffer.get()).as_ref().expect("upload buffer") };
        self.barrier(command, ImageAccess { pipelineStages: vk::PipelineStageFlags::TRANSFER,
            accessMask: vk::AccessFlags::TRANSFER_WRITE, layout: vk::ImageLayout::TRANSFER_DST_OPTIMAL },
            ImageAccessAction::invalidateContents, vk::DependencyFlags::empty());
        let regions = unsafe { &mut *self.m_imageUploadRegions.get() };
        if !regions.is_empty() {
            unsafe { self.m_image.vk().ashDevice().cmd_copy_buffer_to_image(command, buffer.vkBuffer(),
                self.vkImage(), vk::ImageLayout::TRANSFER_DST_OPTIMAL, regions) };
            self.barrier(command, ImageAccess { pipelineStages: vk::PipelineStageFlags::FRAGMENT_SHADER,
                accessMask: vk::AccessFlags::SHADER_READ, layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL },
                ImageAccessAction::preserveContents, vk::DependencyFlags::empty());
            regions.clear();
        } else {
            let region = vk::BufferImageCopy { image_subresource: vk::ImageSubresourceLayers {
                aspect_mask: vk::ImageAspectFlags::COLOR, layer_count: 1, ..Default::default() },
                image_extent: vk::Extent3D { width: self.width(), height: self.height(), depth: 1 },
                ..Default::default() };
            unsafe { self.m_image.vk().ashDevice().cmd_copy_buffer_to_image(command, buffer.vkBuffer(),
                self.vkImage(), vk::ImageLayout::TRANSFER_DST_OPTIMAL, &[region]) };
            self.generateMipmaps(command, ImageAccess { pipelineStages: vk::PipelineStageFlags::FRAGMENT_SHADER,
                accessMask: vk::AccessFlags::SHADER_READ, layout: vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL });
        }
        unsafe { *self.m_imageUploadBuffer.get() = None };
    }
    pub(crate) fn barrier(&self, command: vk::CommandBuffer, dst: ImageAccess,
        action: ImageAccessAction, dependency: vk::DependencyFlags) {
        let src = self.lastAccess(); let next = self.m_image.vk().simpleImageMemoryBarrier(
            command, src, dst, self.vkImage(), action, dependency);
        unsafe { *self.m_lastAccess.get() = next };
    }
    pub(crate) fn generateMipmaps(&self, command: vk::CommandBuffer, dst: ImageAccess) {
        let levels = self.m_image.info().mip_levels;
        if levels <= 1 { self.barrier(command, dst, ImageAccessAction::preserveContents, vk::DependencyFlags::empty()); return; }
        self.barrier(command, ImageAccess { pipelineStages: vk::PipelineStageFlags::TRANSFER,
            accessMask: vk::AccessFlags::TRANSFER_WRITE, layout: vk::ImageLayout::TRANSFER_DST_OPTIMAL },
            ImageAccessAction::preserveContents, vk::DependencyFlags::empty());
        let mut src_size = [self.width() as i32, self.height() as i32];
        for level in 1..levels {
            let dst_size = [(src_size[0] >> 1).max(1), (src_size[1] >> 1).max(1)];
            self.m_image.vk().imageMemoryBarrier(command, vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::TRANSFER, vk::DependencyFlags::empty(),
                vk::ImageMemoryBarrier::default().src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                    .dst_access_mask(vk::AccessFlags::TRANSFER_READ)
                    .old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                    .new_layout(vk::ImageLayout::TRANSFER_SRC_OPTIMAL).image(self.vkImage())
                    .subresource_range(vk::ImageSubresourceRange { base_mip_level: level - 1,
                        level_count: 1, ..Default::default() }));
            let blit = vk::ImageBlit { src_subresource: vk::ImageSubresourceLayers {
                    aspect_mask: vk::ImageAspectFlags::COLOR, mip_level: level - 1,
                    layer_count: 1, ..Default::default() },
                src_offsets: [vk::Offset3D::default(), vk::Offset3D { x: src_size[0], y: src_size[1], z: 1 }],
                dst_subresource: vk::ImageSubresourceLayers { aspect_mask: vk::ImageAspectFlags::COLOR,
                    mip_level: level, layer_count: 1, ..Default::default() },
                dst_offsets: [vk::Offset3D::default(), vk::Offset3D { x: dst_size[0], y: dst_size[1], z: 1 }] };
            unsafe { self.m_image.vk().ashDevice().cmd_blit_image(command, self.vkImage(),
                vk::ImageLayout::TRANSFER_SRC_OPTIMAL, self.vkImage(),
                vk::ImageLayout::TRANSFER_DST_OPTIMAL, &[blit], vk::Filter::LINEAR) };
            src_size = dst_size;
        }
        let mut barriers = [
            vk::ImageMemoryBarrier::default().src_access_mask(vk::AccessFlags::TRANSFER_READ)
                .dst_access_mask(dst.accessMask).old_layout(vk::ImageLayout::TRANSFER_SRC_OPTIMAL)
                .new_layout(dst.layout).image(self.vkImage()).subresource_range(vk::ImageSubresourceRange {
                    level_count: levels - 1, ..Default::default() }),
            vk::ImageMemoryBarrier::default().src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                .dst_access_mask(dst.accessMask).old_layout(vk::ImageLayout::TRANSFER_DST_OPTIMAL)
                .new_layout(dst.layout).image(self.vkImage()).subresource_range(vk::ImageSubresourceRange {
                    base_mip_level: levels - 1, level_count: 1, ..Default::default() }) ];
        self.m_image.vk().imageMemoryBarriers(command, vk::PipelineStageFlags::TRANSFER,
            dst.pipelineStages, vk::DependencyFlags::empty(), &mut barriers);
        unsafe { *self.m_lastAccess.get() = dst };
    }
}

fn texture2d_base(width: u32, height: u32) -> Texture {
    let mut base = Texture::new(width, height);
    base.destroy_complete = destroy_texture2d;
    base.setNativeHandleDispatch(texture2d_native_handle);
    base
}

unsafe fn destroy_texture2d(base: *mut Texture) {
    unsafe { drop(Box::from_raw(base.cast::<Texture2D>())) };
}

unsafe fn texture2d_native_handle(base: *const Texture) -> *mut core::ffi::c_void {
    unsafe { (&*base.cast::<Texture2D>()).nativeHandle() }
}

impl Drop for Texture2D {
    fn drop(&mut self) {
        unsafe {
            std::mem::ManuallyDrop::drop(&mut self.m_imageUploadRegions);
            std::mem::ManuallyDrop::drop(&mut self.m_imageUploadBuffer);
            std::mem::ManuallyDrop::drop(&mut self.m_imageView);
            std::mem::ManuallyDrop::drop(&mut self.m_image);
            std::mem::ManuallyDrop::drop(&mut self.base);
        }
    }
}

impl Framebuffer {
    pub(crate) fn new(vk: Arc<VulkanContext>, info: vk::FramebufferCreateInfo<'_>) -> Self {
        let mut info: vk::FramebufferCreateInfo<'static> = unsafe { transmute(info) };
        info.s_type = vk::StructureType::FRAMEBUFFER_CREATE_INFO;
        let framebuffer = match unsafe { vk.ashDevice().create_framebuffer(&info, None) } {
            Ok(value) => value, Err(error) => vk_abort(error, file!(), line!()),
        };
        Self { base: std::mem::ManuallyDrop::new(Resource::new(vk)),
            m_info: info, m_vkFramebuffer: framebuffer }
    }
    pub(crate) fn info(&self) -> vk::FramebufferCreateInfo<'static> { self.m_info }
    pub(crate) fn vkFramebuffer(&self) -> vk::Framebuffer { self.m_vkFramebuffer }
}
impl Drop for Framebuffer {
    fn drop(&mut self) {
        unsafe {
            self.vk().ashDevice().destroy_framebuffer(self.m_vkFramebuffer, None);
            std::mem::ManuallyDrop::drop(&mut self.base);
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_result_strings_and_success_check_are_exact() {
        assert_eq!(string_from_vk_result(vk::Result::SUCCESS), "SUCCESS");
        assert_eq!(string_from_vk_result(vk::Result::ERROR_DEVICE_LOST), "ERROR_DEVICE_LOST");
        assert_eq!(string_from_vk_result(vk::Result::from_raw(i32::MIN)), "<unknown>");
        vk_check(vk::Result::SUCCESS, file!(), line!());
    }

    #[test]
    fn source_mappability_maps_to_exact_vma_host_access_flags() {
        assert_eq!(vma_flags_for_mappability(Mappability::none).bits(),
            AllocationCreateFlags::empty().bits());
        assert_eq!(vma_flags_for_mappability(Mappability::writeOnly).bits(),
            AllocationCreateFlags::HOST_ACCESS_SEQUENTIAL_WRITE.bits());
        assert_eq!(vma_flags_for_mappability(Mappability::readWrite).bits(),
            AllocationCreateFlags::HOST_ACCESS_RANDOM.bits());
    }
}
