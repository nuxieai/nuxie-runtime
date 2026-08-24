//! Headless native product root for the exact Vulkan translation.

use std::ffi::{CStr, CString, c_void};
#[cfg(target_os = "macos")]
use std::path::PathBuf;
use std::pin::Pin;
use std::ptr::NonNull;

use ash::vk;
use ash::vk::Handle;

use super::render_context_vulkan_decl::{ContextOptions, RenderContextVulkanImpl};
use super::render_target_vulkan_decl::{RenderTargetVulkanApi, RenderTargetVulkanImpl};
use super::vkutil_decl::{ImageAccess, ImageAccessAction};
use super::vulkan_context_decl::VulkanFeatures;
use crate::exact_source_adapter::ExactSourceBackend;
use crate::mechanical_port::source::include::rive::refcnt_hpp::rcp;
use crate::mechanical_port::source::renderer::include::rive::renderer::render_context_hpp::{
    FlushResources, FrameDescriptor, RenderContext, RenderContextContract,
};
use crate::{RenderMode, RendererError};

const TARGET_FORMAT: vk::Format = vk::Format::B8G8R8A8_UNORM;
const TARGET_USAGE: vk::ImageUsageFlags = vk::ImageUsageFlags::from_raw(
    vk::ImageUsageFlags::COLOR_ATTACHMENT.as_raw()
        | vk::ImageUsageFlags::INPUT_ATTACHMENT.as_raw()
        | vk::ImageUsageFlags::TRANSFER_SRC.as_raw()
        | vk::ImageUsageFlags::TRANSFER_DST.as_raw(),
);

struct TargetResources {
    image: vk::Image,
    image_memory: vk::DeviceMemory,
    image_view: vk::ImageView,
    readback: vk::Buffer,
    readback_memory: vk::DeviceMemory,
    command_pool: vk::CommandPool,
    command_buffer: vk::CommandBuffer,
    fence: vk::Fence,
}

pub(crate) struct VulkanProductBackend {
    entry: ash::Entry,
    instance: ash::Instance,
    physical_device: vk::PhysicalDevice,
    device: ash::Device,
    queue: vk::Queue,
    queue_family_index: u32,
    context: Option<Pin<Box<RenderContext>>>,
    target: rcp<RenderTargetVulkanImpl>,
    resources: TargetResources,
    width: u32,
    height: u32,
    frame_number: u64,
    active_frame: bool,
    adapter_name: String,
}

impl VulkanProductBackend {
    pub(crate) fn new(width: u32, height: u32) -> Result<Self, RendererError> {
        if width == 0 || height == 0 {
            return Err(RendererError::InvalidTextureExtent {
                label: "Vulkan target",
                width,
                height,
                max_dimension: u32::MAX,
            });
        }
        let entry = load_vulkan_entry()?;
        let (instance, get_instance_proc_addr) = create_instance(&entry)?;
        let (physical_device, queue_family_index) = select_physical_device(&instance)?;
        let properties = unsafe { instance.get_physical_device_properties(physical_device) };
        let adapter_name = unsafe { CStr::from_ptr(properties.device_name.as_ptr()) }
            .to_string_lossy()
            .into_owned();
        let (device, features) = create_device(
            &instance,
            physical_device,
            queue_family_index,
            properties.api_version,
        )?;
        let queue = unsafe { device.get_device_queue(queue_family_index, 0) };
        let mut context = unsafe {
            super::render_context_vulkan_decl::MakeContext(
                instance.handle(),
                physical_device,
                device.handle(),
                features,
                get_instance_proc_addr,
                ContextOptions::default(),
            )
        }
        .ok_or_else(|| RendererError::Device("exact Vulkan context admission failed".into()))?;
        #[cfg(feature = "rive-decoders")]
        crate::exact_source_adapter::install_bitmap_decoder(context.as_mut());
        let context_ref = unsafe { Pin::get_unchecked_mut(context.as_mut()) };
        let implementation = unsafe {
            &mut *context_ref
                .static_impl_cast::<RenderContextVulkanImpl>()
        };
        unsafe { implementation.setCanvasQueue(queue, queue_family_index) };
        let target = implementation.makeRenderTarget(width, height, TARGET_FORMAT, TARGET_USAGE);
        if !target.operator_bool() {
            return Err(RendererError::Device(
                "exact Vulkan render-target creation failed".into(),
            ));
        }
        let memory_properties =
            unsafe { instance.get_physical_device_memory_properties(physical_device) };
        let resources = create_target_resources(
            &device,
            &memory_properties,
            queue_family_index,
            width,
            height,
        )?;
        Ok(Self {
            entry,
            instance,
            physical_device,
            device,
            queue,
            queue_family_index,
            context: Some(context),
            target,
            resources,
            width,
            height,
            frame_number: 0,
            active_frame: false,
            adapter_name,
        })
    }

    pub(crate) fn adapter_name(&self) -> &str {
        &self.adapter_name
    }

    pub(crate) fn resize(&mut self, width: u32, height: u32) -> Result<(), RendererError> {
        if width == 0 || height == 0 {
            return Err(RendererError::InvalidTextureExtent {
                label: "Vulkan target",
                width,
                height,
                max_dimension: u32::MAX,
            });
        }
        if self.active_frame {
            return Err(RendererError::Device(
                "cannot resize an active exact Vulkan frame".into(),
            ));
        }
        if width == self.width && height == self.height {
            return Ok(());
        }
        unsafe {
            self.device
                .device_wait_idle()
                .map_err(|error| RendererError::Device(format!("wait to resize Vulkan target: {error:?}")))?;
        }
        let target = {
            let context = unsafe { Pin::get_unchecked_mut(self.context_pin()) };
            let implementation = unsafe {
                &mut *context.static_impl_cast::<RenderContextVulkanImpl>()
            };
            implementation.makeRenderTarget(width, height, TARGET_FORMAT, TARGET_USAGE)
        };
        if !target.operator_bool() {
            return Err(RendererError::Device(
                "exact Vulkan render-target resize failed".into(),
            ));
        }
        let memory_properties = unsafe {
            self.instance
                .get_physical_device_memory_properties(self.physical_device)
        };
        let resources = create_target_resources(
            &self.device,
            &memory_properties,
            self.queue_family_index,
            width,
            height,
        )?;

        let mut old_target = std::mem::replace(&mut self.target, target);
        let old_resources = std::mem::replace(&mut self.resources, resources);
        self.width = width;
        self.height = height;
        old_target.operator_assign_null();
        unsafe { destroy_target_resources(&self.device, &old_resources) };
        Ok(())
    }

    fn context_pin(&mut self) -> Pin<&mut RenderContext> {
        self.context.as_mut().expect("live Vulkan context").as_mut()
    }

    fn target_mut(&mut self) -> &mut RenderTargetVulkanImpl {
        unsafe { &mut *self.target.get() }
    }

    fn read_pixels(&self) -> Result<Vec<u8>, RendererError> {
        let byte_len = usize::try_from(u64::from(self.width) * u64::from(self.height) * 4)
            .map_err(|_| RendererError::Map("Vulkan readback size overflow".into()))?;
        let mapped = unsafe {
            self.device
                .map_memory(
                    self.resources.readback_memory,
                    0,
                    byte_len as u64,
                    vk::MemoryMapFlags::empty(),
                )
                .map_err(|error| RendererError::Map(format!("map Vulkan readback: {error:?}")))?
        };
        let source = unsafe { std::slice::from_raw_parts(mapped.cast::<u8>(), byte_len) };
        let mut pixels = vec![0; byte_len];
        let stride = self.width as usize * 4;
        for y in 0..self.height as usize {
            let source_row = &source[y * stride..][..stride];
            let target_row = &mut pixels[y * stride..][..stride];
            target_row.copy_from_slice(source_row);
            for pixel in target_row.chunks_exact_mut(4) {
                pixel.swap(0, 2);
            }
        }
        unsafe { self.device.unmap_memory(self.resources.readback_memory) };
        Ok(pixels)
    }
}

fn load_vulkan_entry() -> Result<ash::Entry, RendererError> {
    match unsafe { ash::Entry::load() } {
        Ok(entry) => Ok(entry),
        Err(system_error) => {
            #[cfg(target_os = "macos")]
            {
                let mut candidates = Vec::new();
                if let Some(path) = std::env::var_os("NUXIE_MOLTENVK_LIBRARY") {
                    candidates.push(PathBuf::from(path));
                }
                if let Some(paths) = std::env::var_os("DYLD_LIBRARY_PATH") {
                    candidates.extend(
                        std::env::split_paths(&paths)
                            .map(|directory| directory.join("libMoltenVK.dylib")),
                    );
                }
                let mut failures = Vec::new();
                for candidate in candidates {
                    if !candidate.is_file() {
                        continue;
                    }
                    match unsafe { ash::Entry::load_from(&candidate) } {
                        Ok(entry) => return Ok(entry),
                        Err(error) => failures.push(format!("{}: {error}", candidate.display())),
                    }
                }
                return Err(RendererError::Adapter(format!(
                    "load Vulkan ({system_error}); pinned MoltenVK candidates failed: {}",
                    if failures.is_empty() {
                        "none found".to_owned()
                    } else {
                        failures.join("; ")
                    }
                )));
            }
            #[cfg(not(target_os = "macos"))]
            {
                Err(RendererError::Adapter(format!(
                    "load Vulkan: {system_error}"
                )))
            }
        }
    }
}

impl ExactSourceBackend for VulkanProductBackend {
    fn context_mut(&mut self) -> Pin<&mut RenderContext> {
        self.context_pin()
    }

    fn resize(&mut self, width: u32, height: u32) -> Result<(), RendererError> {
        VulkanProductBackend::resize(self, width, height)
    }

    fn begin_frame(&mut self, clear_color: u32, mode: RenderMode) -> Result<u64, RendererError> {
        if self.active_frame {
            return Err(RendererError::Device(
                "exact Vulkan context already has an active frame".into(),
            ));
        }
        unsafe {
            self.device
                .wait_for_fences(&[self.resources.fence], true, u64::MAX)
                .map_err(|error| RendererError::Device(format!("wait Vulkan frame: {error:?}")))?;
            self.device
                .reset_command_pool(
                    self.resources.command_pool,
                    vk::CommandPoolResetFlags::empty(),
                )
                .map_err(|error| RendererError::Device(format!("reset Vulkan frame: {error:?}")))?;
            self.device
                .begin_command_buffer(
                    self.resources.command_buffer,
                    &vk::CommandBufferBeginInfo::default()
                        .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT),
                )
                .map_err(|error| RendererError::Device(format!("begin Vulkan frame: {error:?}")))?;
        }
        let image = self.resources.image;
        let image_view = self.resources.image_view;
        let last_access = self.target_mut().targetLastAccess().to_owned();
        unsafe {
            self.target_mut()
                .setTargetImageView(image_view, image, last_access)
        };
        let mut descriptor = FrameDescriptor {
            renderTargetWidth: self.width,
            renderTargetHeight: self.height,
            clearColor: clear_color,
            ..FrameDescriptor::default()
        };
        match mode {
            RenderMode::RasterOrdering => {}
            RenderMode::Msaa => descriptor.msaaSampleCount = 4,
            RenderMode::ClockwiseAtomic => {
                descriptor.disableRasterOrdering = true;
                descriptor.clockwiseFillOverride = true;
            }
        }
        let context = unsafe { Pin::get_unchecked_mut(self.context_pin()) };
        context.beginFrameExecutable(&descriptor);
        self.frame_number = self.frame_number.wrapping_add(1);
        self.active_frame = true;
        Ok(self.frame_number)
    }

    fn finish_frame(&mut self, frame_number: u64) -> Result<Vec<u8>, RendererError> {
        if !self.active_frame || frame_number != self.frame_number {
            return Err(RendererError::Device(
                "exact Vulkan frame ownership mismatch".into(),
            ));
        }
        let target = self.target.get().cast();
        let command = self.resources.command_buffer;
        let external_command = NonNull::new(command.as_raw() as usize as *mut c_void)
            .ok_or_else(|| RendererError::Device("null Vulkan command buffer".into()))?;
        let resources = FlushResources {
            renderTarget: target,
            externalCommandBuffer: external_command.as_ptr(),
            currentFrameNumber: frame_number,
            safeFrameNumber: frame_number.saturating_sub(1),
        };
        let context = unsafe { Pin::get_unchecked_mut(self.context_pin()) };
        unsafe { context.flushExecutable(&resources) };

        let transfer = ImageAccess {
            pipelineStages: vk::PipelineStageFlags::TRANSFER,
            accessMask: vk::AccessFlags::TRANSFER_READ,
            layout: vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
        };
        let image = self.target_mut().accessTargetImage(
            command,
            transfer,
            ImageAccessAction::preserveContents,
        );
        let copy = vk::BufferImageCopy::default()
            .image_subresource(
                vk::ImageSubresourceLayers::default()
                    .aspect_mask(vk::ImageAspectFlags::COLOR)
                    .layer_count(1),
            )
            .image_extent(vk::Extent3D {
                width: self.width,
                height: self.height,
                depth: 1,
            });
        unsafe {
            self.device.cmd_copy_image_to_buffer(
                command,
                image,
                vk::ImageLayout::TRANSFER_SRC_OPTIMAL,
                self.resources.readback,
                &[copy],
            );
            self.device.cmd_pipeline_barrier(
                command,
                vk::PipelineStageFlags::TRANSFER,
                vk::PipelineStageFlags::HOST,
                vk::DependencyFlags::empty(),
                &[],
                &[vk::BufferMemoryBarrier::default()
                    .src_access_mask(vk::AccessFlags::TRANSFER_WRITE)
                    .dst_access_mask(vk::AccessFlags::HOST_READ)
                    .buffer(self.resources.readback)
                    .size(vk::WHOLE_SIZE)],
                &[],
            );
            self.device
                .end_command_buffer(command)
                .map_err(|error| RendererError::Device(format!("end Vulkan frame: {error:?}")))?;
            self.device
                .reset_fences(&[self.resources.fence])
                .map_err(|error| RendererError::Device(format!("reset Vulkan fence: {error:?}")))?;
            self.device
                .queue_submit(
                    self.queue,
                    &[vk::SubmitInfo::default().command_buffers(&[command])],
                    self.resources.fence,
                )
                .map_err(|error| RendererError::Device(format!("submit Vulkan frame: {error:?}")))?;
            self.device
                .wait_for_fences(&[self.resources.fence], true, u64::MAX)
                .map_err(|error| RendererError::Device(format!("complete Vulkan frame: {error:?}")))?;
        }
        self.active_frame = false;
        self.read_pixels()
    }

    fn abort_frame(&mut self) {
        if self.active_frame {
            unsafe { Pin::get_unchecked_mut(self.context_pin()) }.abortFrameExecutable();
            self.active_frame = false;
        }
    }
}

impl Drop for VulkanProductBackend {
    fn drop(&mut self) {
        self.abort_frame();
        unsafe {
            let _ = self.device.device_wait_idle();
        }
        self.target.operator_assign_null();
        self.context.take();
        unsafe {
            destroy_target_resources(&self.device, &self.resources);
            self.device.destroy_device(None);
            self.instance.destroy_instance(None);
        }
        let _ = (&self.entry, self.physical_device, self.queue_family_index);
    }
}

unsafe fn destroy_target_resources(device: &ash::Device, resources: &TargetResources) {
    unsafe {
        device.destroy_fence(resources.fence, None);
        device.destroy_command_pool(resources.command_pool, None);
        device.destroy_buffer(resources.readback, None);
        device.free_memory(resources.readback_memory, None);
        device.destroy_image_view(resources.image_view, None);
        device.destroy_image(resources.image, None);
        device.free_memory(resources.image_memory, None);
    }
}

fn create_instance(
    entry: &ash::Entry,
) -> Result<(ash::Instance, vk::PFN_vkGetInstanceProcAddr), RendererError> {
    let application_name = CString::new("Nuxie exact Vulkan renderer").unwrap();
    let supported = unsafe { entry.enumerate_instance_extension_properties(None) }
        .map_err(|error| RendererError::Adapter(format!("enumerate Vulkan extensions: {error:?}")))?;
    let portability = CString::new("VK_KHR_portability_enumeration").unwrap();
    let has_portability = supported.iter().any(|property| unsafe {
        CStr::from_ptr(property.extension_name.as_ptr()) == portability.as_c_str()
    });
    let mut extensions = Vec::new();
    let mut flags = vk::InstanceCreateFlags::empty();
    if has_portability {
        extensions.push(portability.as_ptr());
        flags |= vk::InstanceCreateFlags::ENUMERATE_PORTABILITY_KHR;
    }
    let supported_version = unsafe { entry.try_enumerate_instance_version() }
        .map_err(|error| RendererError::Adapter(format!("query Vulkan version: {error:?}")))?
        .unwrap_or(vk::API_VERSION_1_0);
    if supported_version < vk::API_VERSION_1_1 {
        return Err(RendererError::Unsupported("Vulkan 1.1 device"));
    }
    let api_version = supported_version.min(vk::API_VERSION_1_3);
    let application = vk::ApplicationInfo::default()
        .application_name(&application_name)
        .engine_name(&application_name)
        .api_version(api_version);
    let create = vk::InstanceCreateInfo::default()
        .flags(flags)
        .application_info(&application)
        .enabled_extension_names(&extensions);
    let instance = unsafe { entry.create_instance(&create, None) }
        .map_err(|error| RendererError::Adapter(format!("create Vulkan instance: {error:?}")))?;
    Ok((instance, entry.static_fn().get_instance_proc_addr))
}

fn select_physical_device(
    instance: &ash::Instance,
) -> Result<(vk::PhysicalDevice, u32), RendererError> {
    let devices = unsafe { instance.enumerate_physical_devices() }
        .map_err(|error| RendererError::Adapter(format!("enumerate Vulkan devices: {error:?}")))?;
    for preferred_discrete in [true, false] {
        for device in &devices {
            let properties = unsafe { instance.get_physical_device_properties(*device) };
            if properties.api_version < vk::API_VERSION_1_1
                || (preferred_discrete
                    && properties.device_type != vk::PhysicalDeviceType::DISCRETE_GPU)
            {
                continue;
            }
            let families = unsafe { instance.get_physical_device_queue_family_properties(*device) };
            if let Some((index, _)) = families
                .iter()
                .enumerate()
                .find(|(_, family)| family.queue_flags.contains(vk::QueueFlags::GRAPHICS))
            {
                return Ok((*device, index as u32));
            }
        }
    }
    Err(RendererError::Adapter(
        "no Vulkan 1.1 graphics device".into(),
    ))
}

fn create_device(
    instance: &ash::Instance,
    physical_device: vk::PhysicalDevice,
    queue_family_index: u32,
    api_version: u32,
) -> Result<(ash::Device, VulkanFeatures), RendererError> {
    let supported_features = unsafe { instance.get_physical_device_features(physical_device) };
    let requested_features = vk::PhysicalDeviceFeatures::default()
        .independent_blend(supported_features.independent_blend != 0)
        .fill_mode_non_solid(supported_features.fill_mode_non_solid != 0)
        .fragment_stores_and_atomics(supported_features.fragment_stores_and_atomics != 0)
        .shader_clip_distance(supported_features.shader_clip_distance != 0)
        .texture_compression_bc(supported_features.texture_compression_bc != 0)
        .texture_compression_astc_ldr(supported_features.texture_compression_astc_ldr != 0)
        .texture_compression_etc2(supported_features.texture_compression_etc2 != 0);
    let supported_extensions = unsafe {
        instance.enumerate_device_extension_properties(physical_device)
    }
    .map_err(|error| RendererError::Adapter(format!("enumerate Vulkan device extensions: {error:?}")))?;
    let supports = |name: &CStr| {
        supported_extensions.iter().any(|property| unsafe {
            CStr::from_ptr(property.extension_name.as_ptr()) == name
        })
    };
    let portability = CString::new("VK_KHR_portability_subset").unwrap();
    let raster_order = CString::new("VK_EXT_rasterization_order_attachment_access").unwrap();
    let amd_raster_order = CString::new("VK_AMD_rasterization_order_attachment_access").unwrap();
    let fragment_interlock = CString::new("VK_EXT_fragment_shader_interlock").unwrap();
    let mut extensions = Vec::new();
    let has_portability = supports(&portability);
    if has_portability {
        extensions.push(portability.as_ptr());
    }
    let selected_raster_extension = if supports(&raster_order) {
        Some(raster_order.as_c_str())
    } else if supports(&amd_raster_order) {
        Some(amd_raster_order.as_c_str())
    } else {
        None
    };
    if let Some(name) = selected_raster_extension {
        extensions.push(name.as_ptr());
    }
    let mut queried_interlock = vk::PhysicalDeviceFragmentShaderInterlockFeaturesEXT::default();
    let mut queried_features =
        vk::PhysicalDeviceFeatures2::default().push_next(&mut queried_interlock);
    unsafe { instance.get_physical_device_features2(physical_device, &mut queried_features) };
    let has_fragment_interlock = supports(&fragment_interlock)
        && queried_interlock.fragment_shader_pixel_interlock != 0;
    if has_fragment_interlock {
        extensions.push(fragment_interlock.as_ptr());
    }
    let mut raster_features = vk::PhysicalDeviceRasterizationOrderAttachmentAccessFeaturesEXT::default()
        .rasterization_order_color_attachment_access(selected_raster_extension.is_some());
    let mut interlock_features = vk::PhysicalDeviceFragmentShaderInterlockFeaturesEXT::default()
        .fragment_shader_pixel_interlock(has_fragment_interlock);
    let priority = [1.0f32];
    let queues = [vk::DeviceQueueCreateInfo::default()
        .queue_family_index(queue_family_index)
        .queue_priorities(&priority)];
    let mut create = vk::DeviceCreateInfo::default()
        .queue_create_infos(&queues)
        .enabled_extension_names(&extensions)
        .enabled_features(&requested_features);
    if selected_raster_extension.is_some() {
        create = create.push_next(&mut raster_features);
    }
    if has_fragment_interlock {
        create = create.push_next(&mut interlock_features);
    }
    let device = unsafe { instance.create_device(physical_device, &create, None) }
        .map_err(|error| RendererError::Device(format!("create Vulkan device: {error:?}")))?;
    Ok((
        device,
        VulkanFeatures {
            apiVersion: api_version,
            independentBlend: requested_features.independent_blend != 0,
            fillModeNonSolid: requested_features.fill_mode_non_solid != 0,
            fragmentStoresAndAtomics: requested_features.fragment_stores_and_atomics != 0,
            shaderClipDistance: requested_features.shader_clip_distance != 0,
            rasterizationOrderColorAttachmentAccess: selected_raster_extension.is_some(),
            fragmentShaderPixelInterlock: has_fragment_interlock,
            colorWriteEnable: false,
            VK_KHR_portability_subset: has_portability,
            textureCompressionBC: requested_features.texture_compression_bc != 0,
            textureCompressionASTC_LDR: requested_features.texture_compression_astc_ldr != 0,
            textureCompressionETC2: requested_features.texture_compression_etc2 != 0,
        },
    ))
}

fn create_target_resources(
    device: &ash::Device,
    memory_properties: &vk::PhysicalDeviceMemoryProperties,
    queue_family_index: u32,
    width: u32,
    height: u32,
) -> Result<TargetResources, RendererError> {
    let image = unsafe {
        device.create_image(
            &vk::ImageCreateInfo::default()
                .image_type(vk::ImageType::TYPE_2D)
                .format(TARGET_FORMAT)
                .extent(vk::Extent3D { width, height, depth: 1 })
                .mip_levels(1)
                .array_layers(1)
                .samples(vk::SampleCountFlags::TYPE_1)
                .tiling(vk::ImageTiling::OPTIMAL)
                .usage(TARGET_USAGE)
                .sharing_mode(vk::SharingMode::EXCLUSIVE),
            None,
        )
    }
    .map_err(|error| RendererError::Device(format!("create Vulkan target image: {error:?}")))?;
    let image_requirements = unsafe { device.get_image_memory_requirements(image) };
    let image_memory_type = find_memory_type(
        memory_properties,
        image_requirements.memory_type_bits,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    )?;
    let image_memory = unsafe {
        device.allocate_memory(
            &vk::MemoryAllocateInfo::default()
                .allocation_size(image_requirements.size)
                .memory_type_index(image_memory_type),
            None,
        )
    }
    .map_err(|error| RendererError::Device(format!("allocate Vulkan target: {error:?}")))?;
    unsafe { device.bind_image_memory(image, image_memory, 0) }
        .map_err(|error| RendererError::Device(format!("bind Vulkan target: {error:?}")))?;
    let image_view = unsafe {
        device.create_image_view(
            &vk::ImageViewCreateInfo::default()
                .image(image)
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(TARGET_FORMAT)
                .subresource_range(
                    vk::ImageSubresourceRange::default()
                        .aspect_mask(vk::ImageAspectFlags::COLOR)
                        .level_count(1)
                        .layer_count(1),
                ),
            None,
        )
    }
    .map_err(|error| RendererError::Device(format!("create Vulkan target view: {error:?}")))?;
    let readback_size = u64::from(width) * u64::from(height) * 4;
    let readback = unsafe {
        device.create_buffer(
            &vk::BufferCreateInfo::default()
                .size(readback_size)
                .usage(vk::BufferUsageFlags::TRANSFER_DST)
                .sharing_mode(vk::SharingMode::EXCLUSIVE),
            None,
        )
    }
    .map_err(|error| RendererError::Device(format!("create Vulkan readback: {error:?}")))?;
    let readback_requirements = unsafe { device.get_buffer_memory_requirements(readback) };
    let readback_memory_type = find_memory_type(
        memory_properties,
        readback_requirements.memory_type_bits,
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
    )?;
    let readback_memory = unsafe {
        device.allocate_memory(
            &vk::MemoryAllocateInfo::default()
                .allocation_size(readback_requirements.size)
                .memory_type_index(readback_memory_type),
            None,
        )
    }
    .map_err(|error| RendererError::Device(format!("allocate Vulkan readback: {error:?}")))?;
    unsafe { device.bind_buffer_memory(readback, readback_memory, 0) }
        .map_err(|error| RendererError::Device(format!("bind Vulkan readback: {error:?}")))?;
    let command_pool = unsafe {
        device.create_command_pool(
            &vk::CommandPoolCreateInfo::default()
                .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
                .queue_family_index(queue_family_index),
            None,
        )
    }
    .map_err(|error| RendererError::Device(format!("create Vulkan command pool: {error:?}")))?;
    let command_buffer = unsafe {
        device.allocate_command_buffers(
            &vk::CommandBufferAllocateInfo::default()
                .command_pool(command_pool)
                .level(vk::CommandBufferLevel::PRIMARY)
                .command_buffer_count(1),
        )
    }
    .map_err(|error| RendererError::Device(format!("allocate Vulkan command buffer: {error:?}")))?[0];
    let fence = unsafe {
        device.create_fence(
            &vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED),
            None,
        )
    }
    .map_err(|error| RendererError::Device(format!("create Vulkan fence: {error:?}")))?;
    Ok(TargetResources {
        image,
        image_memory,
        image_view,
        readback,
        readback_memory,
        command_pool,
        command_buffer,
        fence,
    })
}

fn find_memory_type(
    properties: &vk::PhysicalDeviceMemoryProperties,
    allowed: u32,
    required: vk::MemoryPropertyFlags,
) -> Result<u32, RendererError> {
    (0..properties.memory_type_count)
        .find(|index| {
            allowed & (1 << index) != 0
                && properties.memory_types[*index as usize]
                    .property_flags
                    .contains(required)
        })
        .ok_or_else(|| RendererError::Device(format!("no Vulkan memory type for {required:?}")))
}
