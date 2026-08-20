//! Concrete Metal context and command-buffer ownership.
//!
//! This is a direct ownership adaptation of the pinned upstream context:
//! - `renderer/include/rive/renderer/metal/render_context_metal_impl.h:89-133`
//! - `renderer/src/metal/render_context_metal_impl.mm:454-616`
//! - `renderer/src/metal/render_context_metal_impl.mm:1227-1249`
//!
//! Pinned upstream source: `rive-runtime` at
//! `4ac7b32798da0482e441ef09304dc3b480ed3ee5`.

use super::capabilities::MetalCapabilitySelection;
use super::draw_shader::DrawShaderLibrary;
use super::make_solid_pipeline;
use super::samplers::NativeMetalSamplers;
use crate::RendererError;
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_metal::{
    MTLCommandBuffer, MTLCommandBufferStatus, MTLCommandQueue, MTLDevice, MTLDrawable,
    MTLRenderPipelineState,
};

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
    // The offline draw library and sampler table are retained now even though
    // the diagnostic draw encoder does not consume them yet. Upstream owns
    // both for the complete context lifetime.
    _draw_shader_library: DrawShaderLibrary,
    _samplers: NativeMetalSamplers,
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
            _samplers: samplers,
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
