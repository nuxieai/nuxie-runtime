//! Browser product root for the exact pinned WebGL2 translation.

use std::any::Any;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::Arc;

use web_sys::HtmlCanvasElement;

use super::browser_provider::BrowserWebGl2Provider;
use super::gles3_decl::{GLExecutionDomain, GL_NONE, GL_READ_FRAMEBUFFER};
use super::render_context_gl_decl::{ContextOptions, RenderContextGLImpl};
use super::render_target_gl_decl::FramebufferRenderTargetGL;
use crate::exact_source_adapter::ExactSourceBackend;
use crate::exact_gpu_canvas::ExactGpuCanvas;
use crate::mechanical_port::source::include::rive::refcnt_hpp::{make_rcp, rcp};
use crate::mechanical_port::source::renderer::include::rive::renderer::render_context_hpp::{
    FlushResources, FrameDescriptor, RenderContext, RenderContextContract,
};
use crate::mechanical_port::source::renderer::include::rive::renderer::render_context_helper_impl_hpp::RenderContextHelperBackendContract;
use crate::{RenderMode, RendererError};
use nuxie_render_api::{
    GpuCanvasError, GpuCanvasPipelineShaders, GpuCanvasPlan, GpuCanvasShaderArtifact,
    GpuCanvasShaderProfile, RenderGpuCanvasShader,
};
use crate::mechanical_port::source::renderer::include::rive::renderer::rive_render_image_hpp::RiveRenderImageHandle;

pub(crate) struct WebGl2ProductBackend {
    context: Option<Pin<Box<RenderContext>>>,
    gpu_canvas: Option<ExactGpuCanvas<super::ContextGL>>,
    target: rcp<FramebufferRenderTargetGL>,
    canvas: HtmlCanvasElement,
    sample_count: u32,
    width: u32,
    height: u32,
    frame_number: u64,
    active_frame: bool,
    adapter_name: String,
}

impl WebGl2ProductBackend {
    pub(crate) fn new(
        canvas: HtmlCanvasElement,
        width: u32,
        height: u32,
    ) -> Result<Self, RendererError> {
        if width == 0 || height == 0 {
            return Err(RendererError::InvalidTextureExtent {
                label: "WebGL2 target",
                width,
                height,
                max_dimension: u32::MAX,
            });
        }
        let (provider, adapter_name, sample_count) =
            BrowserWebGl2Provider::new(canvas.clone(), width, height)
                .map_err(|error| RendererError::Adapter(js_error(error)))?;
        let mut context = super::render_context_gl_impl::MakeContext(
            ContextOptions::default(),
            Box::new(provider),
        )
        .ok_or_else(|| RendererError::Device("exact WebGL2 context admission failed".into()))?;
        #[cfg(feature = "rive-decoders")]
        crate::exact_source_adapter::install_bitmap_decoder(context.as_mut());
        let implementation = unsafe {
            &mut *Pin::get_unchecked_mut(context.as_mut()).static_impl_cast::<RenderContextGLImpl>()
        };
        let execution = (&*implementation.rust_execution).clone();
        let target =
            make_rcp(|| FramebufferRenderTargetGL::new(width, height, 0, sample_count, execution));
        implementation.invalidateGLState();
        Ok(Self {
            context: Some(context),
            gpu_canvas: None,
            target,
            canvas,
            sample_count,
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

    fn context_pin(&mut self) -> Pin<&mut RenderContext> {
        self.context.as_mut().expect("live WebGL2 context").as_mut()
    }

    fn implementation_mut(&mut self) -> &mut RenderContextGLImpl {
        unsafe {
            &mut *Pin::get_unchecked_mut(self.context_pin())
                .static_impl_cast::<RenderContextGLImpl>()
        }
    }

    fn execution_domain(&mut self) -> GLExecutionDomain {
        self.implementation_mut().rust_execution.domain().clone()
    }

    fn gpu_canvas_mut(&mut self) -> Result<&mut ExactGpuCanvas<super::ContextGL>, GpuCanvasError> {
        if self.gpu_canvas.is_none() {
            let context = unsafe { Pin::get_unchecked_mut(self.context_pin()) }.oreExecutable();
            let context = match unsafe { context.as_ref() } {
                Some(
                    crate::mechanical_port::source::include::rive::factory_hpp::OreContext::GL(
                        context,
                    ),
                ) => context.clone(),
                _ => {
                    return Err(GpuCanvasError::new(
                        "exact WebGL2 ORE context is unavailable",
                    ));
                }
            };
            self.gpu_canvas = Some(ExactGpuCanvas::new_shared(
                context,
                GpuCanvasShaderProfile::WebGl2,
            )?);
        }
        Ok(self
            .gpu_canvas
            .as_mut()
            .expect("initialized WebGL2 ORE context"))
    }

    fn resize_target(&mut self, width: u32, height: u32) -> Result<(), RendererError> {
        if width == 0 || height == 0 {
            return Err(RendererError::InvalidTextureExtent {
                label: "WebGL2 target",
                width,
                height,
                max_dimension: u32::MAX,
            });
        }
        if self.active_frame {
            return Err(RendererError::Device(
                "cannot resize exact WebGL2 target while a frame is active".into(),
            ));
        }
        if (width, height) == (self.width, self.height) {
            return Ok(());
        }
        self.canvas.set_width(width);
        self.canvas.set_height(height);
        let execution = (&*self.implementation_mut().rust_execution).clone();
        let replacement = make_rcp(|| {
            FramebufferRenderTargetGL::new(width, height, 0, self.sample_count, execution)
        });
        self.target.operator_assign_null();
        self.target = replacement;
        self.implementation_mut().invalidateGLState();
        self.width = width;
        self.height = height;
        Ok(())
    }

    fn finish_frame_inner(
        &mut self,
        frame_number: u64,
        readback: bool,
    ) -> Result<Option<Vec<u8>>, RendererError> {
        if !self.active_frame || frame_number != self.frame_number {
            return Err(RendererError::Device(
                "exact WebGL2 frame ownership mismatch".into(),
            ));
        }
        let resources = FlushResources {
            renderTarget: self.target.get().cast(),
            externalCommandBuffer: std::ptr::null_mut(),
            currentFrameNumber: frame_number,
            safeFrameNumber: frame_number.saturating_sub(1),
        };
        unsafe {
            Pin::get_unchecked_mut(self.context_pin()).flushExecutable(&resources);
        }
        self.implementation_mut().unbindGLInternalResources();
        if readback {
            unsafe { &mut *self.target.get() }.bindDestinationFramebuffer(GL_READ_FRAMEBUFFER);
        }
        let execution = self.execution_domain();
        let error = execution.withCurrent(|| execution.finishAndGetError());
        self.active_frame = false;
        if error != GL_NONE {
            return Err(RendererError::Device(format!(
                "exact WebGL2 post-flush error: {error:#x}"
            )));
        }
        Ok(readback.then(|| {
            execution.withCurrent(|| execution.readPixelsRGBA8(0, 0, self.width, self.height))
        }))
    }
}

impl ExactSourceBackend for WebGl2ProductBackend {
    fn context_mut(&mut self) -> Pin<&mut RenderContext> {
        self.context_pin()
    }

    fn resize(&mut self, width: u32, height: u32) -> Result<(), RendererError> {
        self.resize_target(width, height)
    }

    fn begin_frame(&mut self, clear_color: u32, mode: RenderMode) -> Result<u64, RendererError> {
        if self.active_frame {
            return Err(RendererError::Device(
                "exact WebGL2 context already has an active frame".into(),
            ));
        }
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
        unsafe { Pin::get_unchecked_mut(self.context_pin()) }.beginFrameExecutable(&descriptor);
        self.frame_number = self.frame_number.wrapping_add(1);
        self.active_frame = true;
        Ok(self.frame_number)
    }

    fn finish_frame(&mut self, frame_number: u64) -> Result<Vec<u8>, RendererError> {
        self.finish_frame_inner(frame_number, true)?
            .ok_or_else(|| RendererError::Map("WebGL2 readback did not produce pixels".into()))
    }

    fn finish_frame_without_readback(&mut self, frame_number: u64) -> Result<(), RendererError> {
        self.finish_frame_inner(frame_number, false).map(drop)
    }

    fn abort_frame(&mut self) {
        if self.active_frame {
            unsafe { Pin::get_unchecked_mut(self.context_pin()) }.abortFrameExecutable();
            self.active_frame = false;
        }
    }

    fn after_deferred_ore_frame(&mut self) {
        self.implementation_mut().invalidateGLState();
    }

    fn gpu_canvas_shader_profile(&self) -> GpuCanvasShaderProfile {
        GpuCanvasShaderProfile::WebGl2
    }

    fn make_gpu_canvas_shader_artifact(
        &mut self,
        artifact: &GpuCanvasShaderArtifact,
        execution_anchor: Rc<dyn Any>,
    ) -> Result<Arc<dyn RenderGpuCanvasShader>, GpuCanvasError> {
        self.gpu_canvas_mut()?
            .make_shader_artifact(artifact, execution_anchor)
    }

    fn make_gpu_canvas_shader_occurrence(
        &mut self,
        prepared: &Arc<dyn RenderGpuCanvasShader>,
        execution_anchor: Rc<dyn Any>,
    ) -> Result<Arc<dyn RenderGpuCanvasShader>, GpuCanvasError> {
        self.gpu_canvas_mut()?
            .make_shader_occurrence(prepared, execution_anchor)
    }

    fn make_gpu_canvas_image_with_pipelines(
        &mut self,
        pipelines: &[GpuCanvasPipelineShaders],
        plan: &GpuCanvasPlan,
        execution_anchor: Rc<dyn Any>,
    ) -> Result<RiveRenderImageHandle, GpuCanvasError> {
        // Scripted drawables materialize their offscreen image during the
        // outer presentation traversal. Both ORE contexts share the exact GL
        // execution domain, but the GPU canvas owns its own framebuffer and
        // frame lifecycle. Invalidate the outer context afterwards so its
        // deferred presentation flush restores every GL binding it needs.
        let canvas = self
            .implementation_mut()
            .makeRenderCanvas(plan.width, plan.height);
        if !canvas.operator_bool() {
            return Err(GpuCanvasError::new(
                "exact WebGL2 failed to create a GPU-canvas render target",
            ));
        }
        let result = {
            let gpu_canvas = self.gpu_canvas_mut()?;
            let frame_number = gpu_canvas.next_frame_number();
            gpu_canvas.begin_frame(frame_number);
            let result =
                gpu_canvas.execute_current_frame(&canvas, pipelines, plan, &execution_anchor);
            gpu_canvas.end_frame();
            result
        };
        self.implementation_mut().invalidateGLState();
        result
    }
}

impl Drop for WebGl2ProductBackend {
    fn drop(&mut self) {
        self.abort_frame();
        self.gpu_canvas.take();
        self.target.operator_assign_null();
        self.context.take();
    }
}

fn js_error(error: wasm_bindgen::JsValue) -> String {
    error.as_string().unwrap_or_else(|| format!("{error:?}"))
}
