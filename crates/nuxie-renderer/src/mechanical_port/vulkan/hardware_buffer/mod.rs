//! Android video frames arrive from the decoder in AHardwareBuffers, usually
//! in a vendor YUV layout that only the driver understands. A frame is
//! imported as a Vulkan image over the buffer's own memory, with no copy, and
//! one small draw through a sampler Y'CbCr conversion turns it into RGBA in a
//! texture the renderer draws like any decoded image.
//!
//! The conversion applies the video's YUV matrix and range and no transfer
//! curve, as the GL video sampler behind the previous readback path did, so
//! colors stay the same up to chroma filtering. Hosts pass the matrix and
//! range the decoder tags its output with: the driver also suggests one, but
//! not reliably (the Android emulator reports full range for limited-range
//! video), so its suggestion is only the fallback.
//!
//! Conversions run on the renderer's queue and complete before `convert`
//! returns: a barrier orders them after frames already submitted that may
//! still sample the target, and the caller may release the buffer as soon as
//! the call returns.

// Host builds compile the region math for its tests only.
#![cfg_attr(not(target_os = "android"), allow(dead_code))]

mod shaders;

use crate::video_frame_geometry::FrameRegion;
use crate::RendererError;
use ash::vk;
use std::ffi::{c_void, CStr};
use std::ptr::NonNull;
use std::time::Duration;

/// Longest a conversion may take before it is reported as a device failure.
const CONVERSION_TIMEOUT: Duration = Duration::from_secs(2);

/// The Y'CbCr matrix a video's pixels were encoded with.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VideoMatrix {
    Bt601,
    Bt709,
    Bt2020,
}

/// How a decoded frame's Y'CbCr values map to RGB.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct VideoColor {
    pub matrix: VideoMatrix,
    /// Full range (0 to 255) rather than limited range (16 to 235 luma).
    pub full_range: bool,
}

/// `AHardwareBuffer_Desc` from the NDK.
#[repr(C)]
#[derive(Default)]
struct HardwareBufferDesc {
    width: u32,
    height: u32,
    layers: u32,
    format: u32,
    usage: u64,
    stride: u32,
    rfu0: u32,
    rfu1: u64,
}

/// `AHARDWAREBUFFER_USAGE_GPU_SAMPLED_IMAGE`.
const USAGE_GPU_SAMPLED_IMAGE: u64 = 1 << 8;

/// The NDK's AHardwareBuffer functions. They are API 26 while the runtime
/// targets API 23, so they are looked up in the NDK's public
/// `libnativewindow.so` rather than linked; a default-scope `dlsym` does not
/// see them from inside an app's linker namespace.
#[derive(Clone, Copy)]
struct HardwareBufferApi {
    describe: unsafe extern "C" fn(*const c_void, *mut HardwareBufferDesc),
    acquire: unsafe extern "C" fn(*mut c_void),
    release: unsafe extern "C" fn(*mut c_void),
}

fn hardware_buffer_api() -> Option<HardwareBufferApi> {
    extern "C" {
        fn dlopen(file: *const std::ffi::c_char, mode: std::ffi::c_int) -> *mut c_void;
        fn dlsym(handle: *mut c_void, symbol: *const std::ffi::c_char) -> *mut c_void;
    }
    const RTLD_NOW: std::ffi::c_int = 2;
    let library = unsafe { dlopen(c"libnativewindow.so".as_ptr(), RTLD_NOW) };
    if library.is_null() {
        return None;
    }
    let symbol = |name: &CStr| {
        let address = unsafe { dlsym(library, name.as_ptr()) };
        (!address.is_null()).then_some(address)
    };
    unsafe {
        Some(HardwareBufferApi {
            describe: std::mem::transmute(symbol(c"AHardwareBuffer_describe")?),
            acquire: std::mem::transmute(symbol(c"AHardwareBuffer_acquire")?),
            release: std::mem::transmute(symbol(c"AHardwareBuffer_release")?),
        })
    }
}

/// Width and height of an AHardwareBuffer, or `None` before Android 8.
///
/// # Safety
/// `buffer` is a live AHardwareBuffer.
pub unsafe fn hardware_buffer_size(buffer: NonNull<c_void>) -> Option<(u32, u32)> {
    let api = hardware_buffer_api()?;
    let mut desc = HardwareBufferDesc::default();
    unsafe { (api.describe)(buffer.as_ptr(), &mut desc) };
    Some((desc.width, desc.height))
}

/// Everything a conversion depends on in the buffer's format.
#[derive(Clone, Copy, PartialEq, Eq)]
struct ConversionKey {
    format: vk::Format,
    external_format: u64,
    model: vk::SamplerYcbcrModelConversion,
    range: vk::SamplerYcbcrRange,
    components: [vk::ComponentSwizzle; 4],
    x_chroma_offset: vk::ChromaLocation,
    y_chroma_offset: vk::ChromaLocation,
    linear_chroma: bool,
}

/// The sampler, layouts and pipeline for one buffer format. The sampler is
/// immutable in its set layout, as Y'CbCr conversion requires.
struct Conversion {
    key: ConversionKey,
    ycbcr: vk::SamplerYcbcrConversion,
    sampler: vk::Sampler,
    set_layout: vk::DescriptorSetLayout,
    pipeline_layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,
    descriptor_pool: vk::DescriptorPool,
    descriptor_set: vk::DescriptorSet,
}

/// A decoder buffer imported as a Vulkan image. A decoder cycles through a
/// few buffers, so imports are kept and reused rather than made per frame.
/// Each holds a reference to its buffer, so the buffer's address stays unique
/// while it is cached.
struct ImportedBuffer {
    buffer: NonNull<c_void>,
    conversion: usize,
    image: vk::Image,
    memory: vk::DeviceMemory,
    view: vk::ImageView,
    last_use: u64,
}

/// Imports kept at once. A decoder cycles through its own output buffers
/// plus the reader's images, commonly ten or more between them.
const IMPORT_CAPACITY: usize = 16;

/// Conversions after which an unused import is released, so buffers of a
/// closed decoder or an old frame size do not stay alive.
const IMPORT_IDLE_CONVERSIONS: u64 = 120;

pub(super) struct HardwareBufferConverter {
    device: ash::Device,
    hardware_buffers: ash::android::external_memory_android_hardware_buffer::Device,
    api: HardwareBufferApi,
    queue: vk::Queue,
    queue_family_index: u32,
    command_pool: vk::CommandPool,
    command_buffer: vk::CommandBuffer,
    fence: vk::Fence,
    render_pass: vk::RenderPass,
    vertex: vk::ShaderModule,
    fragment: vk::ShaderModule,
    conversions: Vec<Conversion>,
    imports: Vec<ImportedBuffer>,
    uses: u64,
}

impl HardwareBufferConverter {
    /// The target textures' format, the renderer's image format.
    pub(super) const TARGET_FORMAT: vk::Format = vk::Format::R8G8B8A8_UNORM;

    /// Device extensions a converter needs besides Vulkan 1.1, which provides
    /// sampler Y'CbCr conversion, external memory and dedicated allocation.
    pub(super) const EXTENSIONS: [&'static CStr; 2] = [
        ash::android::external_memory_android_hardware_buffer::NAME,
        ash::ext::queue_family_foreign::NAME,
    ];

    /// # Safety
    /// `device` was created with [`Self::EXTENSIONS`] and the
    /// `samplerYcbcrConversion` feature, and outlives the converter, which
    /// must be dropped before it. `queue` is the renderer's queue.
    pub(super) unsafe fn new(
        instance: &ash::Instance,
        device: &ash::Device,
        queue: vk::Queue,
        queue_family_index: u32,
    ) -> Result<Self, RendererError> {
        let api = hardware_buffer_api().ok_or(RendererError::Unsupported(
            "AHardwareBuffer functions (Android 8)",
        ))?;
        let mut converter = Self {
            device: device.clone(),
            hardware_buffers: ash::android::external_memory_android_hardware_buffer::Device::new(
                instance, device,
            ),
            api,
            queue,
            queue_family_index,
            command_pool: vk::CommandPool::null(),
            command_buffer: vk::CommandBuffer::null(),
            fence: vk::Fence::null(),
            render_pass: vk::RenderPass::null(),
            vertex: vk::ShaderModule::null(),
            fragment: vk::ShaderModule::null(),
            conversions: Vec::new(),
            imports: Vec::new(),
            uses: 0,
        };
        // Drop releases whatever was created if a later step fails.
        unsafe { converter.create_shared_objects() }?;
        Ok(converter)
    }

    unsafe fn create_shared_objects(&mut self) -> Result<(), RendererError> {
        let device = &self.device;
        self.command_pool = unsafe {
            device.create_command_pool(
                &vk::CommandPoolCreateInfo::default()
                    .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER)
                    .queue_family_index(self.queue_family_index),
                None,
            )
        }
        .map_err(|error| device_error("create command pool", error))?;
        self.command_buffer = unsafe {
            device.allocate_command_buffers(
                &vk::CommandBufferAllocateInfo::default()
                    .command_pool(self.command_pool)
                    .level(vk::CommandBufferLevel::PRIMARY)
                    .command_buffer_count(1),
            )
        }
        .map_err(|error| device_error("allocate command buffer", error))?[0];
        self.fence = unsafe { device.create_fence(&vk::FenceCreateInfo::default(), None) }
            .map_err(|error| device_error("create fence", error))?;
        let attachment = [vk::AttachmentDescription::default()
            .format(Self::TARGET_FORMAT)
            .samples(vk::SampleCountFlags::TYPE_1)
            // Every pixel is written, so the previous contents are not needed.
            .load_op(vk::AttachmentLoadOp::DONT_CARE)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)];
        let color = [vk::AttachmentReference::default()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL)];
        let subpass = [vk::SubpassDescription::default()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(&color)];
        let dependencies = [
            // Earlier work on the queue may still sample the target.
            vk::SubpassDependency::default()
                .src_subpass(vk::SUBPASS_EXTERNAL)
                .dst_subpass(0)
                .src_stage_mask(vk::PipelineStageFlags::ALL_COMMANDS)
                .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
                .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE),
            // Later frames sample the converted image.
            vk::SubpassDependency::default()
                .src_subpass(0)
                .dst_subpass(vk::SUBPASS_EXTERNAL)
                .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
                .src_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE)
                .dst_stage_mask(vk::PipelineStageFlags::FRAGMENT_SHADER)
                .dst_access_mask(vk::AccessFlags::SHADER_READ),
        ];
        self.render_pass = unsafe {
            device.create_render_pass(
                &vk::RenderPassCreateInfo::default()
                    .attachments(&attachment)
                    .subpasses(&subpass)
                    .dependencies(&dependencies),
                None,
            )
        }
        .map_err(|error| device_error("create render pass", error))?;
        let module = |code: &[u32]| unsafe {
            device.create_shader_module(&vk::ShaderModuleCreateInfo::default().code(code), None)
        };
        self.vertex =
            module(shaders::VERTEX).map_err(|error| device_error("create vertex shader", error))?;
        self.fragment = module(shaders::FRAGMENT)
            .map_err(|error| device_error("create fragment shader", error))?;
        Ok(())
    }

    /// Converts the picture in `buffer` into the image behind `target_view`,
    /// a [`Self::TARGET_FORMAT`] color attachment of the region's display
    /// extent. The target ends in `SHADER_READ_ONLY_OPTIMAL`.
    ///
    /// # Safety
    /// `buffer` is a live AHardwareBuffer whose producer has finished writing
    /// it. `target_view` belongs to this converter's device, and no one reads
    /// its image in work submitted after this call starts.
    pub(super) unsafe fn convert(
        &mut self,
        buffer: NonNull<c_void>,
        region: FrameRegion,
        color: Option<VideoColor>,
        target_view: vk::ImageView,
    ) -> Result<(), RendererError> {
        let mut desc = HardwareBufferDesc::default();
        unsafe { (self.api.describe)(buffer.as_ptr(), &mut desc) };
        if desc.usage & USAGE_GPU_SAMPLED_IMAGE == 0 {
            return Err(RendererError::InvalidImageUpload(
                "hardware buffer lacks GPU sampled-image usage".into(),
            ));
        }
        let [_, _, right, bottom] = region.crop;
        if right > desc.width || bottom > desc.height {
            return Err(RendererError::InvalidImageUpload(format!(
                "crop {:?} does not fit a {}x{} buffer",
                region.crop, desc.width, desc.height
            )));
        }
        let mut format_properties = vk::AndroidHardwareBufferFormatPropertiesANDROID::default();
        let mut properties =
            vk::AndroidHardwareBufferPropertiesANDROID::default().push_next(&mut format_properties);
        unsafe {
            self.hardware_buffers
                .get_android_hardware_buffer_properties(buffer.as_ptr().cast(), &mut properties)
        }
        .map_err(|error| device_error("query hardware buffer", error))?;
        let (allocation_size, memory_type_bits) =
            (properties.allocation_size, properties.memory_type_bits);
        let conversion = unsafe { self.conversion(&format_properties, color) }?;
        let import = unsafe {
            self.imported(
                buffer,
                conversion,
                &desc,
                &format_properties,
                allocation_size,
                memory_type_bits,
            )
        }?;
        let (image, view) = (self.imports[import].image, self.imports[import].view);
        let (target_width, target_height) = region.display_extent();
        let attachments = [target_view];
        let framebuffer = unsafe {
            self.device.create_framebuffer(
                &vk::FramebufferCreateInfo::default()
                    .render_pass(self.render_pass)
                    .attachments(&attachments)
                    .width(target_width)
                    .height(target_height)
                    .layers(1),
                None,
            )
        }
        .map_err(|error| device_error("create conversion framebuffer", error))?;
        let result = unsafe {
            self.record_and_submit(
                image,
                view,
                framebuffer,
                region.source_transform(desc.width, desc.height),
                (target_width, target_height),
                &self.conversions[conversion],
            )
        };
        // The conversion either completed or never reached the queue.
        unsafe { self.device.destroy_framebuffer(framebuffer, None) };
        result
    }

    /// The index of the import of `buffer` for `conversion`, imported on first
    /// use. When the cache is full the least recently used import goes.
    unsafe fn imported(
        &mut self,
        buffer: NonNull<c_void>,
        conversion: usize,
        desc: &HardwareBufferDesc,
        format: &vk::AndroidHardwareBufferFormatPropertiesANDROID<'_>,
        allocation_size: vk::DeviceSize,
        memory_type_bits: u32,
    ) -> Result<usize, RendererError> {
        self.uses += 1;
        if let Some(index) = self
            .imports
            .iter()
            .position(|import| import.buffer == buffer && import.conversion == conversion)
        {
            self.imports[index].last_use = self.uses;
            return Ok(index);
        }
        let uses = self.uses;
        let (idle, kept): (Vec<_>, Vec<_>) = std::mem::take(&mut self.imports)
            .into_iter()
            .partition(|import| uses - import.last_use > IMPORT_IDLE_CONVERSIONS);
        self.imports = kept;
        for import in &idle {
            unsafe { self.forget(import) };
        }
        if self.imports.len() >= IMPORT_CAPACITY {
            let oldest = (0..self.imports.len())
                .min_by_key(|&index| self.imports[index].last_use)
                .expect("the cache is full");
            let import = self.imports.swap_remove(oldest);
            unsafe { self.forget(&import) };
        }
        let mut import = ImportedBuffer {
            buffer,
            conversion,
            image: vk::Image::null(),
            memory: vk::DeviceMemory::null(),
            view: vk::ImageView::null(),
            last_use: self.uses,
        };
        let imported = unsafe {
            self.import(
                &mut import,
                desc,
                format,
                allocation_size,
                memory_type_bits,
                self.conversions[conversion].ycbcr,
            )
        };
        if let Err(error) = imported {
            unsafe { self.destroy_objects(&import) };
            return Err(error);
        }
        unsafe { (self.api.acquire)(buffer.as_ptr()) };
        self.imports.push(import);
        Ok(self.imports.len() - 1)
    }

    /// The index of the conversion for this buffer format, created on first
    /// use. A decoder keeps one format, so there is usually one.
    unsafe fn conversion(
        &mut self,
        format: &vk::AndroidHardwareBufferFormatPropertiesANDROID<'_>,
        color: Option<VideoColor>,
    ) -> Result<usize, RendererError> {
        let components = format.sampler_ycbcr_conversion_components;
        let key = ConversionKey {
            format: format.format,
            external_format: if format.format == vk::Format::UNDEFINED {
                format.external_format
            } else {
                0
            },
            model: color.map_or(format.suggested_ycbcr_model, |color| match color.matrix {
                VideoMatrix::Bt601 => vk::SamplerYcbcrModelConversion::YCBCR_601,
                VideoMatrix::Bt709 => vk::SamplerYcbcrModelConversion::YCBCR_709,
                VideoMatrix::Bt2020 => vk::SamplerYcbcrModelConversion::YCBCR_2020,
            }),
            range: color.map_or(format.suggested_ycbcr_range, |color| {
                if color.full_range {
                    vk::SamplerYcbcrRange::ITU_FULL
                } else {
                    vk::SamplerYcbcrRange::ITU_NARROW
                }
            }),
            components: [components.r, components.g, components.b, components.a],
            x_chroma_offset: format.suggested_x_chroma_offset,
            y_chroma_offset: format.suggested_y_chroma_offset,
            linear_chroma: format
                .format_features
                .contains(vk::FormatFeatureFlags::SAMPLED_IMAGE_YCBCR_CONVERSION_LINEAR_FILTER),
        };
        if let Some(index) = self.conversions.iter().position(|c| c.key == key) {
            return Ok(index);
        }
        let mut conversion = Conversion {
            key,
            ycbcr: vk::SamplerYcbcrConversion::null(),
            sampler: vk::Sampler::null(),
            set_layout: vk::DescriptorSetLayout::null(),
            pipeline_layout: vk::PipelineLayout::null(),
            pipeline: vk::Pipeline::null(),
            descriptor_pool: vk::DescriptorPool::null(),
            descriptor_set: vk::DescriptorSet::null(),
        };
        let created = unsafe { self.create_conversion(&mut conversion, components) };
        if let Err(error) = created {
            unsafe { destroy_conversion(&self.device, &conversion) };
            return Err(error);
        }
        self.conversions.push(conversion);
        Ok(self.conversions.len() - 1)
    }

    unsafe fn create_conversion(
        &self,
        conversion: &mut Conversion,
        components: vk::ComponentMapping,
    ) -> Result<(), RendererError> {
        let device = &self.device;
        let key = conversion.key;
        let filter = if key.linear_chroma {
            vk::Filter::LINEAR
        } else {
            vk::Filter::NEAREST
        };
        let mut external =
            vk::ExternalFormatANDROID::default().external_format(key.external_format);
        conversion.ycbcr = unsafe {
            device.create_sampler_ycbcr_conversion(
                &vk::SamplerYcbcrConversionCreateInfo::default()
                    .format(key.format)
                    .ycbcr_model(key.model)
                    .ycbcr_range(key.range)
                    .components(components)
                    .x_chroma_offset(key.x_chroma_offset)
                    .y_chroma_offset(key.y_chroma_offset)
                    .chroma_filter(filter)
                    .push_next(&mut external),
                None,
            )
        }
        .map_err(|error| device_error("create Y'CbCr conversion", error))?;
        let mut sampler_conversion =
            vk::SamplerYcbcrConversionInfo::default().conversion(conversion.ycbcr);
        conversion.sampler = unsafe {
            device.create_sampler(
                &vk::SamplerCreateInfo::default()
                    // Y'CbCr samplers filter with the conversion's chroma filter.
                    .mag_filter(filter)
                    .min_filter(filter)
                    .mipmap_mode(vk::SamplerMipmapMode::NEAREST)
                    .address_mode_u(vk::SamplerAddressMode::CLAMP_TO_EDGE)
                    .address_mode_v(vk::SamplerAddressMode::CLAMP_TO_EDGE)
                    .address_mode_w(vk::SamplerAddressMode::CLAMP_TO_EDGE)
                    .push_next(&mut sampler_conversion),
                None,
            )
        }
        .map_err(|error| device_error("create Y'CbCr sampler", error))?;
        let immutable = [conversion.sampler];
        let binding = [vk::DescriptorSetLayoutBinding::default()
            .binding(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(1)
            .stage_flags(vk::ShaderStageFlags::FRAGMENT)
            .immutable_samplers(&immutable)];
        conversion.set_layout = unsafe {
            device.create_descriptor_set_layout(
                &vk::DescriptorSetLayoutCreateInfo::default().bindings(&binding),
                None,
            )
        }
        .map_err(|error| device_error("create descriptor set layout", error))?;
        let set_layouts = [conversion.set_layout];
        let push_constants = [vk::PushConstantRange::default()
            .stage_flags(vk::ShaderStageFlags::VERTEX)
            .size(std::mem::size_of::<[[f32; 4]; 2]>() as u32)];
        conversion.pipeline_layout = unsafe {
            device.create_pipeline_layout(
                &vk::PipelineLayoutCreateInfo::default()
                    .set_layouts(&set_layouts)
                    .push_constant_ranges(&push_constants),
                None,
            )
        }
        .map_err(|error| device_error("create pipeline layout", error))?;
        // A Y'CbCr descriptor may take several descriptors of the pool; the
        // count is implementation-defined, and three covers three planes.
        let pool_sizes = [vk::DescriptorPoolSize::default()
            .ty(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .descriptor_count(3)];
        conversion.descriptor_pool = unsafe {
            device.create_descriptor_pool(
                &vk::DescriptorPoolCreateInfo::default()
                    .max_sets(1)
                    .pool_sizes(&pool_sizes),
                None,
            )
        }
        .map_err(|error| device_error("create descriptor pool", error))?;
        conversion.descriptor_set = unsafe {
            device.allocate_descriptor_sets(
                &vk::DescriptorSetAllocateInfo::default()
                    .descriptor_pool(conversion.descriptor_pool)
                    .set_layouts(&set_layouts),
            )
        }
        .map_err(|error| device_error("allocate descriptor set", error))?[0];
        let stages = [
            vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::VERTEX)
                .module(self.vertex)
                .name(c"main"),
            vk::PipelineShaderStageCreateInfo::default()
                .stage(vk::ShaderStageFlags::FRAGMENT)
                .module(self.fragment)
                .name(c"main"),
        ];
        let vertex_input = vk::PipelineVertexInputStateCreateInfo::default();
        let input_assembly = vk::PipelineInputAssemblyStateCreateInfo::default()
            .topology(vk::PrimitiveTopology::TRIANGLE_LIST);
        let viewport = vk::PipelineViewportStateCreateInfo::default()
            .viewport_count(1)
            .scissor_count(1);
        let rasterization = vk::PipelineRasterizationStateCreateInfo::default()
            .polygon_mode(vk::PolygonMode::FILL)
            .cull_mode(vk::CullModeFlags::NONE)
            .line_width(1.0);
        let multisample = vk::PipelineMultisampleStateCreateInfo::default()
            .rasterization_samples(vk::SampleCountFlags::TYPE_1);
        let blend_attachment = [vk::PipelineColorBlendAttachmentState::default()
            .color_write_mask(vk::ColorComponentFlags::RGBA)];
        let blend = vk::PipelineColorBlendStateCreateInfo::default().attachments(&blend_attachment);
        let dynamic_states = [vk::DynamicState::VIEWPORT, vk::DynamicState::SCISSOR];
        let dynamic = vk::PipelineDynamicStateCreateInfo::default().dynamic_states(&dynamic_states);
        let pipeline = [vk::GraphicsPipelineCreateInfo::default()
            .stages(&stages)
            .vertex_input_state(&vertex_input)
            .input_assembly_state(&input_assembly)
            .viewport_state(&viewport)
            .rasterization_state(&rasterization)
            .multisample_state(&multisample)
            .color_blend_state(&blend)
            .dynamic_state(&dynamic)
            .layout(conversion.pipeline_layout)
            .render_pass(self.render_pass)
            .subpass(0)];
        conversion.pipeline =
            unsafe { device.create_graphics_pipelines(vk::PipelineCache::null(), &pipeline, None) }
                .map_err(|(_, error)| device_error("create conversion pipeline", error))?[0];
        Ok(())
    }

    /// Creates the image, memory and view of `import` over its buffer. On
    /// failure the caller destroys whatever was created.
    unsafe fn import(
        &self,
        import: &mut ImportedBuffer,
        desc: &HardwareBufferDesc,
        format_properties: &vk::AndroidHardwareBufferFormatPropertiesANDROID<'_>,
        allocation_size: vk::DeviceSize,
        memory_type_bits: u32,
        ycbcr: vk::SamplerYcbcrConversion,
    ) -> Result<(), RendererError> {
        let device = &self.device;
        let format = format_properties.format;
        let external_format = if format == vk::Format::UNDEFINED {
            format_properties.external_format
        } else {
            0
        };
        let mut external_memory = vk::ExternalMemoryImageCreateInfo::default()
            .handle_types(vk::ExternalMemoryHandleTypeFlags::ANDROID_HARDWARE_BUFFER_ANDROID);
        let mut external = vk::ExternalFormatANDROID::default().external_format(external_format);
        import.image = unsafe {
            device.create_image(
                &vk::ImageCreateInfo::default()
                    .image_type(vk::ImageType::TYPE_2D)
                    .format(format)
                    .extent(vk::Extent3D {
                        width: desc.width,
                        height: desc.height,
                        depth: 1,
                    })
                    .mip_levels(1)
                    .array_layers(1)
                    .samples(vk::SampleCountFlags::TYPE_1)
                    .tiling(vk::ImageTiling::OPTIMAL)
                    .usage(vk::ImageUsageFlags::SAMPLED)
                    .sharing_mode(vk::SharingMode::EXCLUSIVE)
                    .initial_layout(vk::ImageLayout::UNDEFINED)
                    .push_next(&mut external_memory)
                    .push_next(&mut external),
                None,
            )
        }
        .map_err(|error| device_error("create imported image", error))?;
        let memory_type_index = memory_type_bits.trailing_zeros();
        if memory_type_index >= 32 {
            return Err(RendererError::Device(
                "hardware buffer reports no Vulkan memory type".into(),
            ));
        }
        let mut import_info = vk::ImportAndroidHardwareBufferInfoANDROID::default()
            .buffer(import.buffer.as_ptr().cast());
        let mut dedicated = vk::MemoryDedicatedAllocateInfo::default().image(import.image);
        import.memory = unsafe {
            device.allocate_memory(
                &vk::MemoryAllocateInfo::default()
                    .allocation_size(allocation_size)
                    .memory_type_index(memory_type_index)
                    .push_next(&mut import_info)
                    .push_next(&mut dedicated),
                None,
            )
        }
        .map_err(|error| device_error("import hardware buffer memory", error))?;
        unsafe { device.bind_image_memory(import.image, import.memory, 0) }
            .map_err(|error| device_error("bind imported memory", error))?;
        let mut view_conversion = vk::SamplerYcbcrConversionInfo::default().conversion(ycbcr);
        import.view = unsafe {
            device.create_image_view(
                &vk::ImageViewCreateInfo::default()
                    .image(import.image)
                    .view_type(vk::ImageViewType::TYPE_2D)
                    .format(format)
                    .subresource_range(color_range())
                    .push_next(&mut view_conversion),
                None,
            )
        }
        .map_err(|error| device_error("create imported view", error))?;
        Ok(())
    }

    unsafe fn record_and_submit(
        &self,
        image: vk::Image,
        view: vk::ImageView,
        framebuffer: vk::Framebuffer,
        source_transform: [[f32; 4]; 2],
        (width, height): (u32, u32),
        conversion: &Conversion,
    ) -> Result<(), RendererError> {
        let device = &self.device;
        let (descriptor_set, pipeline, pipeline_layout) = (
            conversion.descriptor_set,
            conversion.pipeline,
            conversion.pipeline_layout,
        );
        let image_info = [vk::DescriptorImageInfo::default()
            .image_view(view)
            .image_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)];
        let write = [vk::WriteDescriptorSet::default()
            .dst_set(descriptor_set)
            .dst_binding(0)
            .descriptor_type(vk::DescriptorType::COMBINED_IMAGE_SAMPLER)
            .image_info(&image_info)];
        // The previous conversion completed, so its set is free to rewrite.
        unsafe { device.update_descriptor_sets(&write, &[]) };
        let commands = self.command_buffer;
        unsafe {
            device
                .reset_command_buffer(commands, vk::CommandBufferResetFlags::empty())
                .map_err(|error| device_error("reset conversion commands", error))?;
            device
                .begin_command_buffer(
                    commands,
                    &vk::CommandBufferBeginInfo::default()
                        .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT),
                )
                .map_err(|error| device_error("begin conversion commands", error))?;
            // Take the buffer over from the decoder. Its contents come from
            // outside Vulkan, so the acquire names the foreign queue family.
            let acquire = [vk::ImageMemoryBarrier::default()
                .src_access_mask(vk::AccessFlags::empty())
                .dst_access_mask(vk::AccessFlags::SHADER_READ)
                .old_layout(vk::ImageLayout::UNDEFINED)
                .new_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .src_queue_family_index(vk::QUEUE_FAMILY_FOREIGN_EXT)
                .dst_queue_family_index(self.queue_family_index)
                .image(image)
                .subresource_range(color_range())];
            device.cmd_pipeline_barrier(
                commands,
                vk::PipelineStageFlags::TOP_OF_PIPE,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &acquire,
            );
            device.cmd_begin_render_pass(
                commands,
                &vk::RenderPassBeginInfo::default()
                    .render_pass(self.render_pass)
                    .framebuffer(framebuffer)
                    .render_area(vk::Rect2D {
                        offset: vk::Offset2D::default(),
                        extent: vk::Extent2D { width, height },
                    }),
                vk::SubpassContents::INLINE,
            );
            device.cmd_bind_pipeline(commands, vk::PipelineBindPoint::GRAPHICS, pipeline);
            device.cmd_set_viewport(
                commands,
                0,
                &[vk::Viewport {
                    x: 0.0,
                    y: 0.0,
                    width: width as f32,
                    height: height as f32,
                    min_depth: 0.0,
                    max_depth: 1.0,
                }],
            );
            device.cmd_set_scissor(
                commands,
                0,
                &[vk::Rect2D {
                    offset: vk::Offset2D::default(),
                    extent: vk::Extent2D { width, height },
                }],
            );
            device.cmd_bind_descriptor_sets(
                commands,
                vk::PipelineBindPoint::GRAPHICS,
                pipeline_layout,
                0,
                &[descriptor_set],
                &[],
            );
            let push: [u8; 32] = std::mem::transmute(source_transform);
            device.cmd_push_constants(
                commands,
                pipeline_layout,
                vk::ShaderStageFlags::VERTEX,
                0,
                &push,
            );
            device.cmd_draw(commands, 3, 1, 0, 0);
            device.cmd_end_render_pass(commands);
            // Hand the buffer back so the decoder can reuse it.
            let release = [vk::ImageMemoryBarrier::default()
                .src_access_mask(vk::AccessFlags::SHADER_READ)
                .dst_access_mask(vk::AccessFlags::empty())
                .old_layout(vk::ImageLayout::SHADER_READ_ONLY_OPTIMAL)
                .new_layout(vk::ImageLayout::GENERAL)
                .src_queue_family_index(self.queue_family_index)
                .dst_queue_family_index(vk::QUEUE_FAMILY_FOREIGN_EXT)
                .image(image)
                .subresource_range(color_range())];
            device.cmd_pipeline_barrier(
                commands,
                vk::PipelineStageFlags::FRAGMENT_SHADER,
                vk::PipelineStageFlags::BOTTOM_OF_PIPE,
                vk::DependencyFlags::empty(),
                &[],
                &[],
                &release,
            );
            device
                .end_command_buffer(commands)
                .map_err(|error| device_error("end conversion commands", error))?;
            let command_buffers = [commands];
            device
                .queue_submit(
                    self.queue,
                    &[vk::SubmitInfo::default().command_buffers(&command_buffers)],
                    self.fence,
                )
                .map_err(|error| device_error("submit conversion", error))?;
            let waited =
                device.wait_for_fences(&[self.fence], true, CONVERSION_TIMEOUT.as_nanos() as u64);
            // A lost or hung conversion leaves the fence unusable; the device
            // is reported as failed either way.
            waited.map_err(|error| device_error("wait for conversion", error))?;
            device
                .reset_fences(&[self.fence])
                .map_err(|error| device_error("reset conversion fence", error))?;
        }
        Ok(())
    }

    /// Destroys a cached import and releases its buffer reference.
    unsafe fn forget(&self, import: &ImportedBuffer) {
        unsafe {
            self.destroy_objects(import);
            (self.api.release)(import.buffer.as_ptr());
        }
    }

    unsafe fn destroy_objects(&self, import: &ImportedBuffer) {
        let device = &self.device;
        unsafe {
            if import.view != vk::ImageView::null() {
                device.destroy_image_view(import.view, None);
            }
            if import.image != vk::Image::null() {
                device.destroy_image(import.image, None);
            }
            if import.memory != vk::DeviceMemory::null() {
                device.free_memory(import.memory, None);
            }
        }
    }
}

impl Drop for HardwareBufferConverter {
    fn drop(&mut self) {
        let device = &self.device;
        unsafe {
            // Conversions wait for completion, so nothing is in flight here.
            for import in &self.imports {
                self.forget(import);
            }
            for conversion in &self.conversions {
                destroy_conversion(device, conversion);
            }
            for module in [self.vertex, self.fragment] {
                if module != vk::ShaderModule::null() {
                    device.destroy_shader_module(module, None);
                }
            }
            if self.render_pass != vk::RenderPass::null() {
                device.destroy_render_pass(self.render_pass, None);
            }
            if self.fence != vk::Fence::null() {
                device.destroy_fence(self.fence, None);
            }
            if self.command_pool != vk::CommandPool::null() {
                device.destroy_command_pool(self.command_pool, None);
            }
        }
    }
}

unsafe fn destroy_conversion(device: &ash::Device, conversion: &Conversion) {
    unsafe {
        if conversion.pipeline != vk::Pipeline::null() {
            device.destroy_pipeline(conversion.pipeline, None);
        }
        if conversion.descriptor_pool != vk::DescriptorPool::null() {
            device.destroy_descriptor_pool(conversion.descriptor_pool, None);
        }
        if conversion.pipeline_layout != vk::PipelineLayout::null() {
            device.destroy_pipeline_layout(conversion.pipeline_layout, None);
        }
        if conversion.set_layout != vk::DescriptorSetLayout::null() {
            device.destroy_descriptor_set_layout(conversion.set_layout, None);
        }
        if conversion.sampler != vk::Sampler::null() {
            device.destroy_sampler(conversion.sampler, None);
        }
        if conversion.ycbcr != vk::SamplerYcbcrConversion::null() {
            device.destroy_sampler_ycbcr_conversion(conversion.ycbcr, None);
        }
    }
}

fn color_range() -> vk::ImageSubresourceRange {
    vk::ImageSubresourceRange::default()
        .aspect_mask(vk::ImageAspectFlags::COLOR)
        .level_count(1)
        .layer_count(1)
}

fn device_error(operation: &str, error: vk::Result) -> RendererError {
    RendererError::Device(format!(
        "hardware buffer conversion: {operation}: {error:?}"
    ))
}
