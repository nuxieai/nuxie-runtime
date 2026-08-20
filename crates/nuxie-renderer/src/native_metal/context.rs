//! Concrete Metal context and command-buffer ownership.
//!
//! This is a direct ownership adaptation of the pinned upstream context:
//! - `renderer/include/rive/renderer/metal/render_context_metal_impl.h:89-280`
//! - `renderer/src/metal/render_context_metal_impl.mm:100-240,414-656`
//! - `renderer/src/metal/render_context_metal_impl.mm:1023-1079,1227-1509`
//! - `renderer/src/metal/render_context_metal_impl.mm:1898-1925,2016-2030`
//!
//! Pinned upstream source: `rive-runtime` at
//! `4ac7b32798da0482e441ef09304dc3b480ed3ee5`.

use super::capabilities::MetalCapabilitySelection;
use super::draw_pipeline::{DrawPipeline, MetalInterlockMode};
use super::draw_shader::DrawShaderLibrary;
use super::gradient_resource::{GradientResource, GRADIENT_TEXTURE_WIDTH};
use super::resource_ring::ResourceRing;
use super::samplers::NativeMetalSamplers;
use super::tessellation_resource::{
    TessellationResource, K_TESS_SPAN_INDICES, TESSELLATION_TEXTURE_WIDTH,
};
use super::{make_solid_pipeline, new_library_from_metallib_bytes};
use crate::gpu::{self, DrawType};
use crate::RendererError;
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_foundation::NSString;
use objc2_metal::{
    MTLBuffer, MTLCommandBuffer, MTLCommandBufferStatus, MTLCommandQueue, MTLDevice, MTLDrawable,
    MTLLibrary, MTLOrigin, MTLPixelFormat, MTLRegion, MTLRenderPipelineDescriptor,
    MTLRenderPipelineState, MTLResourceOptions, MTLSize, MTLTexture, MTLTextureDescriptor,
    MTLTextureType, MTLTextureUsage,
};
use std::ffi::c_void;
use std::ptr::NonNull;
use std::sync::Mutex;

const RESOURCE_METALLIB: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/native_metal_resources.metallib"));

const COLOR_RAMP_VERTEX_MAIN: &str = "EF";
const COLOR_RAMP_FRAGMENT_MAIN: &str = "FF";
const TESSELLATE_VERTEX_MAIN: &str = "WF";
const TESSELLATE_FRAGMENT_MAIN: &str = "XF";
const UBER_PATH_VERTEX_MAIN: &str = "p1111000000::GC";
const UBER_PATH_FRAGMENT_MAIN: &str = "p1111111100::JB";

struct ResourceState {
    ring: ResourceRing,
    gradient: GradientResource,
    tessellation: TessellationResource,
}

pub(crate) struct PreparedResourceLease {
    pub(crate) slot: usize,
    pub(crate) gradient: Retained<ProtocolObject<dyn MTLTexture>>,
    pub(crate) tessellation: Retained<ProtocolObject<dyn MTLTexture>>,
}

/// Long-lived resources shared by every frame from one native Metal factory.
///
/// The context is intentionally concrete. It is the Metal owner translated
/// from `RenderContextMetalImpl`, not a speculative cross-backend HAL.
pub(crate) struct NativeMetalContext {
    device: Retained<ProtocolObject<dyn MTLDevice>>,
    queue: Retained<ProtocolObject<dyn MTLCommandQueue>>,
    capabilities: MetalCapabilitySelection,
    solid_rgba_pipeline: Retained<ProtocolObject<dyn MTLRenderPipelineState>>,
    solid_bgra_pipeline: Retained<ProtocolObject<dyn MTLRenderPipelineState>>,
    _draw_shader_library: DrawShaderLibrary,
    _resource_shader_library: Retained<ProtocolObject<dyn MTLLibrary>>,
    color_ramp_pipeline: Retained<ProtocolObject<dyn MTLRenderPipelineState>>,
    tessellate_pipeline: Retained<ProtocolObject<dyn MTLRenderPipelineState>>,
    midpoint_draw_pipeline: DrawPipeline,
    tess_span_index_buffer: Retained<ProtocolObject<dyn MTLBuffer>>,
    path_patch_vertex_buffer: Retained<ProtocolObject<dyn MTLBuffer>>,
    path_patch_index_buffer: Retained<ProtocolObject<dyn MTLBuffer>>,
    gaussian_integral_texture: Retained<ProtocolObject<dyn MTLTexture>>,
    resources: Mutex<ResourceState>,
    samplers: NativeMetalSamplers,
}

impl NativeMetalContext {
    pub(crate) fn new(
        device: Retained<ProtocolObject<dyn MTLDevice>>,
        capabilities: MetalCapabilitySelection,
    ) -> Result<Self, RendererError> {
        let queue = device.newCommandQueue().ok_or_else(|| {
            RendererError::NativeMetal("MTLDevice returned no command queue".to_owned())
        })?;
        let draw_shader_library = DrawShaderLibrary::load(&device)
            .map_err(|error| RendererError::NativeMetal(error.to_string()))?;
        let resource_shader_library = new_library_from_metallib_bytes(&device, RESOURCE_METALLIB)?;
        let color_ramp_pipeline = make_resource_pipeline(
            &device,
            &resource_shader_library,
            COLOR_RAMP_VERTEX_MAIN,
            COLOR_RAMP_FRAGMENT_MAIN,
            MTLPixelFormat::RGBA8Unorm,
        )?;
        let tessellate_pipeline = make_resource_pipeline(
            &device,
            &resource_shader_library,
            TESSELLATE_VERTEX_MAIN,
            TESSELLATE_FRAGMENT_MAIN,
            MTLPixelFormat::RGBA32Uint,
        )?;
        let midpoint_draw_pipeline = DrawPipeline::new(
            &device,
            Some(draw_shader_library.library()),
            &NSString::from_str(UBER_PATH_VERTEX_MAIN),
            &NSString::from_str(UBER_PATH_FRAGMENT_MAIN),
            DrawType::MidpointFanPatches,
            MetalInterlockMode::RasterOrdering,
            0,
        )
        .map_err(|error| RendererError::NativeMetal(error.to_string()))?;
        let tess_span_index_buffer = make_buffer(&device, &K_TESS_SPAN_INDICES)?;
        let (patch_vertices, patch_indices) = gpu::generate_patch_buffer_data();
        let path_patch_vertex_buffer = make_buffer(&device, &patch_vertices)?;
        let path_patch_index_buffer = make_buffer(&device, &patch_indices)?;
        let gaussian_integral_texture = make_gaussian_integral_texture(&device)?;
        let gradient = GradientResource::new(&device, GRADIENT_TEXTURE_WIDTH, 1)?
            .expect("the canonical gradient texture extent is nonzero");
        let tessellation = TessellationResource::new(&device, TESSELLATION_TEXTURE_WIDTH, 1)?
            .expect("the canonical tessellation texture extent is nonzero");
        let resources = Mutex::new(ResourceState {
            ring: ResourceRing::new(),
            gradient,
            tessellation,
        });
        let samplers = NativeMetalSamplers::new(&device)?;
        let solid_rgba_pipeline =
            make_solid_pipeline(&device, objc2_metal::MTLPixelFormat::RGBA8Unorm)?;
        let solid_bgra_pipeline =
            make_solid_pipeline(&device, objc2_metal::MTLPixelFormat::BGRA8Unorm)?;
        Ok(Self {
            device,
            queue,
            capabilities,
            solid_rgba_pipeline,
            solid_bgra_pipeline,
            _draw_shader_library: draw_shader_library,
            _resource_shader_library: resource_shader_library,
            color_ramp_pipeline,
            tessellate_pipeline,
            midpoint_draw_pipeline,
            tess_span_index_buffer,
            path_patch_vertex_buffer,
            path_patch_index_buffer,
            gaussian_integral_texture,
            resources,
            samplers,
        })
    }

    pub(crate) fn device(&self) -> &ProtocolObject<dyn MTLDevice> {
        &self.device
    }

    pub(crate) fn retained_device(&self) -> Retained<ProtocolObject<dyn MTLDevice>> {
        self.device.clone()
    }

    pub(crate) fn capabilities(&self) -> MetalCapabilitySelection {
        self.capabilities
    }

    pub(crate) fn solid_pipeline(
        &self,
        pixel_format: objc2_metal::MTLPixelFormat,
    ) -> Result<&ProtocolObject<dyn MTLRenderPipelineState>, RendererError> {
        if pixel_format == objc2_metal::MTLPixelFormat::RGBA8Unorm {
            Ok(&self.solid_rgba_pipeline)
        } else if pixel_format == objc2_metal::MTLPixelFormat::BGRA8Unorm {
            Ok(&self.solid_bgra_pipeline)
        } else {
            Err(RendererError::NativeMetal(format!(
                "native Metal tracer does not support target pixel format {pixel_format:?}"
            )))
        }
    }

    pub(crate) fn color_ramp_pipeline(&self) -> &ProtocolObject<dyn MTLRenderPipelineState> {
        &self.color_ramp_pipeline
    }

    pub(crate) fn tessellate_pipeline(&self) -> &ProtocolObject<dyn MTLRenderPipelineState> {
        &self.tessellate_pipeline
    }

    pub(crate) fn midpoint_draw_pipeline(
        &self,
        pixel_format: MTLPixelFormat,
    ) -> Result<&ProtocolObject<dyn MTLRenderPipelineState>, RendererError> {
        self.midpoint_draw_pipeline
            .pipeline_state(pixel_format)
            .map_err(|error| RendererError::NativeMetal(error.to_string()))
    }

    pub(crate) fn tess_span_index_buffer(&self) -> &ProtocolObject<dyn MTLBuffer> {
        &self.tess_span_index_buffer
    }

    pub(crate) fn path_patch_vertex_buffer(&self) -> &ProtocolObject<dyn MTLBuffer> {
        &self.path_patch_vertex_buffer
    }

    pub(crate) fn path_patch_index_buffer(&self) -> &ProtocolObject<dyn MTLBuffer> {
        &self.path_patch_index_buffer
    }

    pub(crate) fn gaussian_integral_texture(&self) -> &ProtocolObject<dyn MTLTexture> {
        &self.gaussian_integral_texture
    }

    pub(crate) fn image_sampler(
        &self,
        sampler: nuxie_render_api::ImageSampler,
    ) -> &ProtocolObject<dyn objc2_metal::MTLSamplerState> {
        self.samplers.sampler(sampler)
    }

    pub(crate) fn prepare_resources(
        &self,
        gradient_height: usize,
        tessellation_height: usize,
    ) -> Result<PreparedResourceLease, RendererError> {
        let mut state = self.resources.lock().map_err(|_| {
            RendererError::NativeMetal("native Metal resource ring is poisoned".to_owned())
        })?;
        let slot = state.ring.prepare_to_flush().map_err(|error| {
            RendererError::NativeMetal(format!("reserve native Metal resource slot: {error:?}"))
        })?;
        let prepared = (|| {
            state
                .gradient
                .resize(self.device(), GRADIENT_TEXTURE_WIDTH, gradient_height)?;
            state.tessellation.resize(
                self.device(),
                TESSELLATION_TEXTURE_WIDTH,
                tessellation_height,
            )?;
            Ok(PreparedResourceLease {
                slot,
                gradient: state.gradient.retained_texture().ok_or_else(|| {
                    RendererError::NativeMetal("gradient resource texture is absent".to_owned())
                })?,
                tessellation: state.tessellation.retained_texture().ok_or_else(|| {
                    RendererError::NativeMetal("tessellation resource texture is absent".to_owned())
                })?,
            })
        })();
        if prepared.is_err() {
            let _ = state.ring.abandon(slot);
        }
        prepared
    }

    pub(crate) fn release_resources(&self, slot: usize) -> Result<(), RendererError> {
        self.resources
            .lock()
            .map_err(|_| {
                RendererError::NativeMetal("native Metal resource ring is poisoned".to_owned())
            })?
            .ring
            .release(slot)
            .map_err(|error| {
                RendererError::NativeMetal(format!("release native Metal resource slot: {error:?}"))
            })
    }

    /// Acquires the one command buffer that a frame owns until finish or drop.
    pub(crate) fn make_command_buffer(
        &self,
    ) -> Result<Retained<ProtocolObject<dyn MTLCommandBuffer>>, RendererError> {
        require_command_buffer(self.queue.commandBuffer())
    }

    /// Commits the frame-owned command buffer and propagates Metal completion.
    pub(crate) fn commit_and_wait(
        command_buffer: &ProtocolObject<dyn MTLCommandBuffer>,
    ) -> Result<(), RendererError> {
        command_buffer.commit();
        command_buffer.waitUntilCompleted();
        command_buffer_completion_result(
            command_buffer.status(),
            command_buffer.error().map(|error| format!("{error:?}")),
        )
    }

    /// Commits renderer work, then schedules the product drawable on the next
    /// command buffer from the same queue. This preserves the pinned product
    /// boundary in `fiddle_context_metal.mm:114-121,186-190`.
    pub(crate) fn commit_and_present(
        &self,
        render_command_buffer: &ProtocolObject<dyn MTLCommandBuffer>,
        drawable: &ProtocolObject<dyn MTLDrawable>,
    ) -> Result<(), RendererError> {
        render_command_buffer.commit();
        let presentation_command_buffer = match self.make_command_buffer() {
            Ok(command_buffer) => command_buffer,
            Err(presentation_error) => {
                render_command_buffer.waitUntilCompleted();
                let render_result = command_buffer_completion_result(
                    render_command_buffer.status(),
                    render_command_buffer
                        .error()
                        .map(|error| format!("{error:?}")),
                );
                return render_result.and(Err(presentation_error));
            }
        };
        presentation_command_buffer.presentDrawable(drawable);
        presentation_command_buffer.commit();

        render_command_buffer.waitUntilCompleted();
        let render_result = command_buffer_completion_result(
            render_command_buffer.status(),
            render_command_buffer
                .error()
                .map(|error| format!("{error:?}")),
        );
        presentation_command_buffer.waitUntilCompleted();
        let presentation_result = command_buffer_completion_result(
            presentation_command_buffer.status(),
            presentation_command_buffer
                .error()
                .map(|error| format!("{error:?}")),
        );
        render_result.and(presentation_result)
    }
}

fn make_resource_pipeline(
    device: &ProtocolObject<dyn MTLDevice>,
    library: &ProtocolObject<dyn MTLLibrary>,
    vertex_name: &str,
    fragment_name: &str,
    pixel_format: MTLPixelFormat,
) -> Result<Retained<ProtocolObject<dyn MTLRenderPipelineState>>, RendererError> {
    let vertex = library
        .newFunctionWithName(&NSString::from_str(vertex_name))
        .ok_or_else(|| {
            RendererError::NativeMetal(format!("missing resource vertex {vertex_name}"))
        })?;
    let fragment = library
        .newFunctionWithName(&NSString::from_str(fragment_name))
        .ok_or_else(|| {
            RendererError::NativeMetal(format!("missing resource fragment {fragment_name}"))
        })?;
    let descriptor = MTLRenderPipelineDescriptor::new();
    descriptor.setVertexFunction(Some(&vertex));
    descriptor.setFragmentFunction(Some(&fragment));
    // SAFETY: Metal render-pipeline descriptors always expose eight color
    // attachment descriptor slots; upstream configures slot zero for both
    // resource pipelines, and `descriptor` retains the returned attachment.
    let attachment = unsafe { descriptor.colorAttachments().objectAtIndexedSubscript(0) };
    attachment.setPixelFormat(pixel_format);
    device
        .newRenderPipelineStateWithDescriptor_error(&descriptor)
        .map_err(|error| {
            RendererError::NativeMetal(format!(
                "create {vertex_name}/{fragment_name} resource pipeline: {error:?}"
            ))
        })
}

fn make_buffer<T: bytemuck::Pod>(
    device: &ProtocolObject<dyn MTLDevice>,
    values: &[T],
) -> Result<Retained<ProtocolObject<dyn MTLBuffer>>, RendererError> {
    let bytes: &[u8] = bytemuck::cast_slice(values);
    let pointer = NonNull::new(bytes.as_ptr().cast_mut().cast::<c_void>()).ok_or_else(|| {
        RendererError::NativeMetal("native Metal static buffer pointer is null".to_owned())
    })?;
    // SAFETY: `T: Pod` makes the complete slice initialized plain data, and
    // `newBufferWithBytes` copies all `bytes.len()` bytes before it returns.
    // The source slice therefore remains valid for the entire Objective-C call.
    unsafe {
        device.newBufferWithBytes_length_options(
            pointer,
            bytes.len(),
            MTLResourceOptions::StorageModeShared,
        )
    }
    .ok_or_else(|| {
        RendererError::NativeMetal("failed to allocate native Metal static buffer".to_owned())
    })
}

fn make_gaussian_integral_texture(
    device: &ProtocolObject<dyn MTLDevice>,
) -> Result<Retained<ProtocolObject<dyn MTLTexture>>, RendererError> {
    let descriptor = MTLTextureDescriptor::new();
    descriptor.setPixelFormat(MTLPixelFormat::R16Float);
    descriptor.setTextureType(MTLTextureType::Type1DArray);
    descriptor.setUsage(MTLTextureUsage::ShaderRead);
    // SAFETY: these are the exact non-zero upstream Gaussian table dimensions;
    // all values fit the typed Metal `NSUInteger`/Rust `usize` parameters.
    unsafe {
        descriptor.setWidth(crate::feather_lut::TABLE_SIZE);
        descriptor.setMipmapLevelCount(1);
        descriptor.setArrayLength(2);
    }
    let texture = device
        .newTextureWithDescriptor(&descriptor)
        .ok_or_else(|| {
            RendererError::NativeMetal(
                "failed to allocate native Metal Gaussian-integral texture".to_owned(),
            )
        })?;
    let rows = crate::feather_lut::table_rows();
    let bytes_per_row = crate::feather_lut::TABLE_SIZE * std::mem::size_of::<u16>();
    let region = MTLRegion {
        origin: MTLOrigin { x: 0, y: 0, z: 0 },
        size: MTLSize {
            width: crate::feather_lut::TABLE_SIZE,
            height: 1,
            depth: 1,
        },
    };
    for (slice, row) in rows.iter().enumerate() {
        let pointer = NonNull::new(row.as_ptr().cast_mut().cast::<c_void>()).ok_or_else(|| {
            RendererError::NativeMetal(
                "native Metal Gaussian-integral table pointer is null".to_owned(),
            )
        })?;
        // SAFETY: each source row contains exactly `TABLE_SIZE` initialized
        // `u16` texels, `region` selects one matching R16Float row, and Metal
        // consumes the borrowed bytes synchronously during `replaceRegion`.
        unsafe {
            texture.replaceRegion_mipmapLevel_slice_withBytes_bytesPerRow_bytesPerImage(
                region,
                0,
                slice,
                pointer,
                bytes_per_row,
                bytes_per_row,
            );
        }
    }
    Ok(texture)
}

fn require_command_buffer(
    command_buffer: Option<Retained<ProtocolObject<dyn MTLCommandBuffer>>>,
) -> Result<Retained<ProtocolObject<dyn MTLCommandBuffer>>, RendererError> {
    command_buffer.ok_or_else(|| {
        RendererError::NativeMetal("MTLCommandQueue returned no command buffer".to_owned())
    })
}

fn command_buffer_completion_result(
    status: MTLCommandBufferStatus,
    error: Option<String>,
) -> Result<(), RendererError> {
    if status == MTLCommandBufferStatus::Completed {
        return Ok(());
    }
    let detail = error.unwrap_or_else(|| format!("status {status:?}"));
    Err(RendererError::NativeMetal(format!(
        "command buffer failed: {detail}"
    )))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_command_buffer_acquisition_fails_closed() {
        assert!(matches!(
            require_command_buffer(None),
            Err(RendererError::NativeMetal(message))
                if message == "MTLCommandQueue returned no command buffer"
        ));
    }

    #[test]
    fn completion_status_propagates_success_and_failure_details() {
        assert!(command_buffer_completion_result(MTLCommandBufferStatus::Completed, None).is_ok());
        assert!(matches!(
            command_buffer_completion_result(
                MTLCommandBufferStatus::Error,
                Some("synthetic Metal failure".to_owned()),
            ),
            Err(RendererError::NativeMetal(message))
                if message == "command buffer failed: synthetic Metal failure"
        ));
        assert!(matches!(
            command_buffer_completion_result(MTLCommandBufferStatus::Committed, None),
            Err(RendererError::NativeMetal(message))
                if message.starts_with("command buffer failed: status ")
        ));
    }
}
