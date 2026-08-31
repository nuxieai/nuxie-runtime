//! Headless native product root for the exact Vulkan translation.

#[cfg(feature = "native-ore-vulkan-experimental")]
use std::any::Any;
use std::ffi::{CStr, CString, c_void};
#[cfg(target_os = "macos")]
use std::path::PathBuf;
use std::pin::Pin;
use std::ptr::NonNull;
#[cfg(feature = "native-ore-vulkan-experimental")]
use std::rc::Rc;
#[cfg(feature = "native-ore-vulkan-experimental")]
use std::sync::Arc;

use ash::vk;
use ash::vk::Handle;

use super::render_context_vulkan_decl::{ContextOptions, RenderContextVulkanImpl};
use super::render_target_vulkan_decl::{RenderTargetVulkanApi, RenderTargetVulkanImpl};
use super::vkutil_decl::{ImageAccess, ImageAccessAction};
use super::vulkan_context_decl::VulkanFeatures;
#[cfg(feature = "native-ore-vulkan-experimental")]
use crate::exact_gpu_canvas::ExactGpuCanvas;
use crate::exact_source_adapter::ExactSourceBackend;
use crate::mechanical_port::source::include::rive::refcnt_hpp::rcp;
use crate::mechanical_port::source::renderer::include::rive::renderer::render_context_hpp::{
    FlushResources, FrameDescriptor, OreContext, RenderContext, RenderContextContract,
};
#[cfg(feature = "native-ore-vulkan-experimental")]
use crate::mechanical_port::source::renderer::include::rive::renderer::rive_render_image_hpp::RiveRenderImageHandle;
use crate::{RenderMode, RendererError};
#[cfg(feature = "native-ore-vulkan-experimental")]
use nuxie_render_api::{
    GpuCanvasError, GpuCanvasPipelineShaders, GpuCanvasPlan, GpuCanvasShaderArtifact,
    GpuCanvasShaderProfile, RenderGpuCanvasShader,
};

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

impl TargetResources {
    fn empty() -> Self {
        Self {
            image: vk::Image::null(),
            image_memory: vk::DeviceMemory::null(),
            image_view: vk::ImageView::null(),
            readback: vk::Buffer::null(),
            readback_memory: vk::DeviceMemory::null(),
            command_pool: vk::CommandPool::null(),
            command_buffer: vk::CommandBuffer::null(),
            fence: vk::Fence::null(),
        }
    }
}

struct TargetResourcesGuard<'a> {
    device: &'a ash::Device,
    resources: TargetResources,
    armed: bool,
}

impl<'a> TargetResourcesGuard<'a> {
    fn new(device: &'a ash::Device) -> Self {
        Self {
            device,
            resources: TargetResources::empty(),
            armed: true,
        }
    }

    fn finish(mut self) -> TargetResources {
        let resources = std::mem::replace(&mut self.resources, TargetResources::empty());
        self.armed = false;
        resources
    }
}

impl Drop for TargetResourcesGuard<'_> {
    fn drop(&mut self) {
        if self.armed {
            unsafe { destroy_target_resources(self.device, &self.resources) };
        }
    }
}

pub(crate) struct VulkanProductBackend {
    entry: ash::Entry,
    instance: ash::Instance,
    physical_device: vk::PhysicalDevice,
    device: ash::Device,
    queue: vk::Queue,
    queue_family_index: u32,
    context: Option<Pin<Box<RenderContext>>>,
    #[cfg(feature = "native-ore-vulkan-experimental")]
    gpu_canvas: Option<ExactGpuCanvas<super::ContextVulkan>>,
    target: rcp<RenderTargetVulkanImpl>,
    resources: TargetResources,
    width: u32,
    height: u32,
    frame_number: u64,
    active_frame: bool,
    frame_recovery_error: Option<String>,
    #[cfg(test)]
    fail_next_finish: bool,
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
            #[cfg(feature = "native-ore-vulkan-experimental")]
            gpu_canvas: None,
            target,
            resources,
            width,
            height,
            frame_number: 0,
            active_frame: false,
            frame_recovery_error: None,
            #[cfg(test)]
            fail_next_finish: false,
            adapter_name,
        })
    }

    pub(crate) fn adapter_name(&self) -> &str {
        &self.adapter_name
    }

    #[cfg(feature = "native-ore-vulkan-experimental")]
    fn gpu_canvas_mut(
        &mut self,
    ) -> Result<&mut ExactGpuCanvas<super::ContextVulkan>, GpuCanvasError> {
        if self.gpu_canvas.is_none() {
            let ore_context = unsafe { Pin::get_unchecked_mut(self.context_pin()) }.oreExecutable();
            let mut ore_context = NonNull::new(ore_context)
                .ok_or_else(|| GpuCanvasError::new("exact Vulkan ORE context is unavailable"))?;
            let context = match unsafe { ore_context.as_mut() } {
                OreContext::Vulkan(context) => NonNull::from(context.as_mut()),
                #[allow(unreachable_patterns)]
                _ => {
                    return Err(GpuCanvasError::new(
                        "exact Vulkan RenderContext returned a foreign ORE context",
                    ));
                }
            };
            self.gpu_canvas = Some(unsafe {
                ExactGpuCanvas::new_borrowed(
                    context,
                    GpuCanvasShaderProfile::TrustedVulkanSpirV,
                )?
            });
        }
        Ok(self
            .gpu_canvas
            .as_mut()
            .expect("initialized source-owned Vulkan ORE context"))
    }

    #[cfg(test)]
    pub(crate) fn fail_next_finish_for_test(&mut self) {
        self.fail_next_finish = true;
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

    fn finish_submission(&mut self, command: vk::CommandBuffer) -> Result<(), RendererError> {
        unsafe {
            self.device.end_command_buffer(command).map_err(|error| {
                RendererError::Device(format!("end Vulkan frame: {error:?}"))
            })?;
            self.device
                .reset_fences(&[self.resources.fence])
                .map_err(|error| {
                    RendererError::Device(format!("reset Vulkan fence: {error:?}"))
                })?;
            #[cfg(test)]
            if std::mem::take(&mut self.fail_next_finish) {
                return Err(RendererError::Device(
                    "injected exact Vulkan submit failure".into(),
                ));
            }
            self.device
                .queue_submit(
                    self.queue,
                    &[vk::SubmitInfo::default().command_buffers(&[command])],
                    self.resources.fence,
                )
                .map_err(|error| {
                    RendererError::Device(format!("submit Vulkan frame: {error:?}"))
                })?;
            self.device
                .wait_for_fences(&[self.resources.fence], true, u64::MAX)
                .map_err(|error| {
                    RendererError::Device(format!("complete Vulkan frame: {error:?}"))
                })?;
        }
        Ok(())
    }

    fn recover_failed_submission(&mut self) -> Result<(), RendererError> {
        unsafe {
            self.device.device_wait_idle().map_err(|error| {
                RendererError::Device(format!("wait to recover failed Vulkan frame: {error:?}"))
            })?;
            let old_fence = std::mem::replace(&mut self.resources.fence, vk::Fence::null());
            self.device.destroy_fence(old_fence, None);
            self.resources.fence = self
                .device
                .create_fence(
                    &vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED),
                    None,
                )
                .map_err(|error| {
                    RendererError::Device(format!(
                        "recreate fence after failed Vulkan frame: {error:?}"
                    ))
                })?;
        }
        Ok(())
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
        if let Some(error) = &self.frame_recovery_error {
            return Err(RendererError::Device(format!(
                "exact Vulkan frame synchronization is unavailable: {error}"
            )));
        }
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
        }
        if let Err(finish_error) = self.finish_submission(command) {
            if let Err(recovery_error) = self.recover_failed_submission() {
                let recovery_error = recovery_error.to_string();
                self.frame_recovery_error = Some(recovery_error.clone());
                return Err(RendererError::Device(format!(
                    "{finish_error}; Vulkan frame recovery failed: {recovery_error}"
                )));
            }
            return Err(finish_error);
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

    #[cfg(feature = "native-ore-vulkan-experimental")]
    fn gpu_canvas_shader_profile(&self) -> GpuCanvasShaderProfile {
        GpuCanvasShaderProfile::TrustedVulkanSpirV
    }

    #[cfg(feature = "native-ore-vulkan-experimental")]
    fn make_gpu_canvas_shader_artifact(
        &mut self,
        artifact: &GpuCanvasShaderArtifact,
        execution_anchor: Rc<dyn Any>,
    ) -> Result<Arc<dyn RenderGpuCanvasShader>, GpuCanvasError> {
        self.gpu_canvas_mut()?
            .make_shader_artifact(artifact, execution_anchor)
    }

    #[cfg(feature = "native-ore-vulkan-experimental")]
    fn make_gpu_canvas_shader_occurrence(
        &mut self,
        prepared: &Arc<dyn RenderGpuCanvasShader>,
        execution_anchor: Rc<dyn Any>,
    ) -> Result<Arc<dyn RenderGpuCanvasShader>, GpuCanvasError> {
        self.gpu_canvas_mut()?
            .make_shader_occurrence(prepared, execution_anchor)
    }

    #[cfg(feature = "native-ore-vulkan-experimental")]
    fn make_gpu_canvas_image_with_pipelines(
        &mut self,
        pipelines: &[GpuCanvasPipelineShaders],
        plan: &GpuCanvasPlan,
        execution_anchor: Rc<dyn Any>,
    ) -> Result<RiveRenderImageHandle, GpuCanvasError> {
        let canvas = unsafe { Pin::get_unchecked_mut(self.context_pin()) }
            .makeRenderCanvasExecutable(plan.width, plan.height);
        if !canvas.operator_bool() {
            return Err(GpuCanvasError::new(
                "exact Vulkan failed to create a GPU-canvas render target",
            ));
        }

        let command = {
            let context = unsafe { Pin::get_unchecked_mut(self.context_pin()) };
            let implementation = unsafe {
                &mut *context.static_impl_cast::<RenderContextVulkanImpl>()
            };
            super::render_context_vulkan_impl::makeCommandBuffer(implementation)
        };
        let command = NonNull::new(command).ok_or_else(|| {
            GpuCanvasError::new("exact Vulkan failed to create a GPU-canvas command buffer")
        })?;

        // ORE and the main renderer share the exact same VulkanContext and
        // GPUResourceManager upstream. Feed both from this product root's one
        // host frame-number stream. Work issued during an active main frame
        // belongs to that frame; standalone synchronous canvas work advances
        // the stream before recording.
        let (safe_frame_number, current_frame_number) =
            gpu_canvas_frame_numbers(&mut self.frame_number, self.active_frame);

        let execution = (|| {
            let gpu_canvas = self.gpu_canvas_mut()?;
            unsafe {
                gpu_canvas.begin_frame_external(
                    safe_frame_number,
                    current_frame_number,
                    command,
                )
            };
            let result = gpu_canvas.execute_current_frame(
                &canvas,
                pipelines,
                plan,
                &execution_anchor,
            );
            gpu_canvas.end_frame();
            result
        })();

        // Pinned Vulkan owns this submission/free path. It runs even when the
        // authored pass is rejected after beginFrame, exactly as the source
        // canvas pre-pass must release its command-buffer allocation.
        {
            let context = unsafe { Pin::get_unchecked_mut(self.context_pin()) };
            let implementation = unsafe {
                &mut *context.static_impl_cast::<RenderContextVulkanImpl>()
            };
            unsafe {
                super::render_context_vulkan_impl::commitCommandBuffer(
                    implementation,
                    command.as_ptr(),
                )
            };
        }
        execution
    }
}

#[cfg(feature = "native-ore-vulkan-experimental")]
fn gpu_canvas_frame_numbers(frame_number: &mut u64, active_frame: bool) -> (u64, u64) {
    if !active_frame {
        *frame_number = frame_number.wrapping_add(1);
    }
    let current_frame_number = *frame_number;
    (
        current_frame_number.saturating_sub(1),
        current_frame_number,
    )
}

impl Drop for VulkanProductBackend {
    fn drop(&mut self) {
        self.abort_frame();
        unsafe {
            let _ = self.device.device_wait_idle();
        }
        #[cfg(feature = "native-ore-vulkan-experimental")]
        self.gpu_canvas.take();
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
        if resources.fence != vk::Fence::null() {
            device.destroy_fence(resources.fence, None);
        }
        if resources.command_pool != vk::CommandPool::null() {
            device.destroy_command_pool(resources.command_pool, None);
        }
        if resources.readback != vk::Buffer::null() {
            device.destroy_buffer(resources.readback, None);
        }
        if resources.readback_memory != vk::DeviceMemory::null() {
            device.free_memory(resources.readback_memory, None);
        }
        if resources.image_view != vk::ImageView::null() {
            device.destroy_image_view(resources.image_view, None);
        }
        if resources.image != vk::Image::null() {
            device.destroy_image(resources.image, None);
        }
        if resources.image_memory != vk::DeviceMemory::null() {
            device.free_memory(resources.image_memory, None);
        }
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
    let mut pending = TargetResourcesGuard::new(device);
    pending.resources.image = unsafe {
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
    let image_requirements =
        unsafe { device.get_image_memory_requirements(pending.resources.image) };
    let image_memory_type = find_memory_type(
        memory_properties,
        image_requirements.memory_type_bits,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    )?;
    pending.resources.image_memory = unsafe {
        device.allocate_memory(
            &vk::MemoryAllocateInfo::default()
                .allocation_size(image_requirements.size)
                .memory_type_index(image_memory_type),
            None,
        )
    }
    .map_err(|error| RendererError::Device(format!("allocate Vulkan target: {error:?}")))?;
    unsafe {
        device.bind_image_memory(
            pending.resources.image,
            pending.resources.image_memory,
            0,
        )
    }
        .map_err(|error| RendererError::Device(format!("bind Vulkan target: {error:?}")))?;
    pending.resources.image_view = unsafe {
        device.create_image_view(
            &vk::ImageViewCreateInfo::default()
                .image(pending.resources.image)
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
    pending.resources.readback = unsafe {
        device.create_buffer(
            &vk::BufferCreateInfo::default()
                .size(readback_size)
                .usage(vk::BufferUsageFlags::TRANSFER_DST)
                .sharing_mode(vk::SharingMode::EXCLUSIVE),
            None,
        )
    }
    .map_err(|error| RendererError::Device(format!("create Vulkan readback: {error:?}")))?;
    let readback_requirements =
        unsafe { device.get_buffer_memory_requirements(pending.resources.readback) };
    let readback_memory_type = find_memory_type(
        memory_properties,
        readback_requirements.memory_type_bits,
        vk::MemoryPropertyFlags::HOST_VISIBLE | vk::MemoryPropertyFlags::HOST_COHERENT,
    )?;
    pending.resources.readback_memory = unsafe {
        device.allocate_memory(
            &vk::MemoryAllocateInfo::default()
                .allocation_size(readback_requirements.size)
                .memory_type_index(readback_memory_type),
            None,
        )
    }
    .map_err(|error| RendererError::Device(format!("allocate Vulkan readback: {error:?}")))?;
    unsafe {
        device.bind_buffer_memory(
            pending.resources.readback,
            pending.resources.readback_memory,
            0,
        )
    }
        .map_err(|error| RendererError::Device(format!("bind Vulkan readback: {error:?}")))?;
    pending.resources.command_pool = unsafe {
        device.create_command_pool(
            &vk::CommandPoolCreateInfo::default()
                .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
                .queue_family_index(queue_family_index),
            None,
        )
    }
    .map_err(|error| RendererError::Device(format!("create Vulkan command pool: {error:?}")))?;
    pending.resources.command_buffer = unsafe {
        device.allocate_command_buffers(
            &vk::CommandBufferAllocateInfo::default()
                .command_pool(pending.resources.command_pool)
                .level(vk::CommandBufferLevel::PRIMARY)
                .command_buffer_count(1),
        )
    }
    .map_err(|error| RendererError::Device(format!("allocate Vulkan command buffer: {error:?}")))?
    .into_iter()
    .next()
    .ok_or_else(|| RendererError::Device("allocate Vulkan command buffer returned none".into()))?;
    pending.resources.fence = unsafe {
        device.create_fence(
            &vk::FenceCreateInfo::default().flags(vk::FenceCreateFlags::SIGNALED),
            None,
        )
    }
    .map_err(|error| RendererError::Device(format!("create Vulkan fence: {error:?}")))?;
    Ok(pending.finish())
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

#[cfg(all(test, feature = "native-ore-vulkan-experimental"))]
mod gpu_canvas_frame_number_tests {
    use super::*;
    use super::super::ore_texture_vulkan_decl::{TextureViewVulkan, TextureVulkan};
    use super::super::vkutil_decl::Texture2D;

    #[test]
    fn gpu_canvas_uses_the_single_monotonic_host_frame_stream() {
        let mut frame_number = 2;

        assert_eq!(gpu_canvas_frame_numbers(&mut frame_number, false), (2, 3));
        assert_eq!(frame_number, 3);

        assert_eq!(gpu_canvas_frame_numbers(&mut frame_number, true), (2, 3));
        assert_eq!(frame_number, 3);

        assert_eq!(gpu_canvas_frame_numbers(&mut frame_number, false), (3, 4));
        assert_eq!(frame_number, 4);
    }

    #[test]
    #[ignore = "requires a configured Vulkan test host"]
    fn wrap_canvas_texture_uses_the_texture_backed_target_and_publishes_resources() {
        let mut backend =
            VulkanProductBackend::new(2, 2).expect("configured Vulkan test host");
        let canvas = unsafe { Pin::get_unchecked_mut(backend.context_pin()) }
            .makeRenderCanvasExecutable(2, 2);
        assert!(canvas.operator_bool());

        let canvas_ptr = canvas.get();
        let (expected_image, expected_view) = {
            let canvas = unsafe { &mut *canvas_ptr };
            let texture_ptr = unsafe { &mut *canvas.renderImage() }.getTexture();
            assert!(!texture_ptr.is_null());
            let texture = unsafe { &*texture_ptr.cast::<Texture2D>() };
            (texture.vkImage(), texture.vkImageView())
        };
        assert_ne!(expected_image, vk::Image::null());
        assert_ne!(expected_view, vk::ImageView::null());

        let ore_context = unsafe { Pin::get_unchecked_mut(backend.context_pin()) }.oreExecutable();
        let mut ore_context = NonNull::new(ore_context).expect("source-owned Vulkan ORE context");
        let context = match unsafe { ore_context.as_mut() } {
            OreContext::Vulkan(context) => context.as_mut(),
            #[allow(unreachable_patterns)]
            _ => panic!("exact Vulkan RenderContext returned a foreign ORE context"),
        };
        let expected_manager = nuxie_ore_metal::context_backend_manager(&*context)
            .expect("source-owned Vulkan resource manager");
        let expected_domain = nuxie_ore_metal::context_backend_domain(&*context);
        let wrapped = unsafe { context.wrapCanvasTexture(canvas_ptr.cast::<c_void>()) }
            .expect("production wrapCanvasTexture result");

        assert!(wrapped.belongsTo(&expected_domain));
        assert!(wrapped
            .manager()
            .is_some_and(|manager| manager.ptr_eq(&expected_manager)));
        {
            let view = wrapped
                .downcast_ref::<TextureViewVulkan>()
                .expect("Vulkan texture-view payload");
            assert_eq!(view.m_vkImageView, expected_view);
            assert!(view.m_vkDestroyImageView.is_none());
            assert!(view.m_vkRenderTarget.is_some());

            let retained_texture = view.texture();
            assert!(retained_texture.belongsTo(&expected_domain));
            assert!(retained_texture
                .manager()
                .is_some_and(|manager| manager.ptr_eq(&expected_manager)));
            assert_eq!(retained_texture.width(), Some(2));
            assert_eq!(retained_texture.height(), Some(2));
            assert_eq!(retained_texture.isRenderTarget(), Some(true));
            let texture = retained_texture
                .downcast_ref::<TextureVulkan>()
                .expect("Vulkan texture payload");
            assert_eq!(texture.m_vkImage, expected_image);
            assert!(texture.m_vmaAllocation.is_none());
            assert_eq!(texture.m_vkLayout.get(), vk::ImageLayout::UNDEFINED);
        }
        drop(wrapped);
        drop(canvas);
    }
}
