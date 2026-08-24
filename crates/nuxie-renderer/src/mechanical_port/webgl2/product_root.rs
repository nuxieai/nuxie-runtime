//! Browser product root for the exact pinned WebGL2 translation.

use std::pin::Pin;

use web_sys::HtmlCanvasElement;

use super::browser_provider::BrowserWebGl2Provider;
use super::gles3_decl::{GLExecutionDomain, GL_NONE, GL_READ_FRAMEBUFFER};
use super::render_context_gl_decl::{ContextOptions, RenderContextGLImpl};
use super::render_target_gl_decl::FramebufferRenderTargetGL;
use crate::exact_source_adapter::ExactSourceBackend;
use crate::mechanical_port::source::include::rive::refcnt_hpp::{make_rcp, rcp};
use crate::mechanical_port::source::renderer::include::rive::renderer::render_context_hpp::{
    FlushResources, FrameDescriptor, RenderContext, RenderContextContract,
};
use crate::{RenderMode, RendererError};

pub(crate) struct WebGl2ProductBackend {
    context: Option<Pin<Box<RenderContext>>>,
    target: rcp<FramebufferRenderTargetGL>,
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
            BrowserWebGl2Provider::new(canvas, width, height)
                .map_err(|error| RendererError::Adapter(js_error(error)))?;
        let mut context = super::render_context_gl_impl::MakeContext(
            ContextOptions::default(),
            Box::new(provider),
        )
        .ok_or_else(|| RendererError::Device("exact WebGL2 context admission failed".into()))?;
        #[cfg(feature = "rive-decoders")]
        crate::exact_source_adapter::install_bitmap_decoder(context.as_mut());
        let implementation = unsafe {
            &mut *Pin::get_unchecked_mut(context.as_mut())
                .static_impl_cast::<RenderContextGLImpl>()
        };
        let execution = (&*implementation.rust_execution).clone();
        let target = make_rcp(|| {
            FramebufferRenderTargetGL::new(width, height, 0, sample_count, execution)
        });
        implementation.invalidateGLState();
        Ok(Self {
            context: Some(context),
            target,
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
}

impl ExactSourceBackend for WebGl2ProductBackend {
    fn context_mut(&mut self) -> Pin<&mut RenderContext> {
        self.context_pin()
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
        unsafe { &mut *self.target.get() }.bindDestinationFramebuffer(GL_READ_FRAMEBUFFER);
        let execution = self.execution_domain();
        let error = execution.withCurrent(|| execution.finishAndGetError());
        self.active_frame = false;
        if error != GL_NONE {
            return Err(RendererError::Device(format!(
                "exact WebGL2 post-flush error: {error:#x}"
            )));
        }
        Ok(execution.withCurrent(|| {
            execution.readPixelsRGBA8(0, 0, self.width, self.height)
        }))
    }

    fn abort_frame(&mut self) {
        if self.active_frame {
            unsafe { Pin::get_unchecked_mut(self.context_pin()) }.abortFrameExecutable();
            self.active_frame = false;
        }
    }
}

impl Drop for WebGl2ProductBackend {
    fn drop(&mut self) {
        self.abort_frame();
        self.target.operator_assign_null();
        self.context.take();
    }
}

fn js_error(error: wasm_bindgen::JsValue) -> String {
    error
        .as_string()
        .unwrap_or_else(|| format!("{error:?}"))
}
