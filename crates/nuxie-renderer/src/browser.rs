use super::{RendererError, WebGl2Factory, WebGl2Frame, WgpuAdapterInfo, WgpuFactory, WgpuFrame};
use nuxie_render_api::{
    BlendMode, ColorInt, Factory, FillRule, ImageDecodeError, ImageSampler, Mat2D, RawPath,
    RenderBuffer, RenderBufferFlags, RenderBufferType, RenderImage, RenderPaint, RenderPath,
    RenderShader, Renderer,
};
use wasm_bindgen::{Clamped, JsCast};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, ImageData};

/// Browser renderer selection policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowserBackendPreference {
    /// Prefer WebGPU and fall back to WebGL2 when WebGPU initialization fails.
    Auto,
    /// Require WebGPU and return its initialization error on failure.
    WebGpu,
    /// Require WebGL2 and do not attempt WebGPU initialization.
    WebGl2,
}

/// Backend selected for a browser renderer instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowserBackend {
    /// The asynchronous wgpu/WebGPU renderer.
    WebGpu,
    /// The WebGL2 compatibility renderer.
    WebGl2,
}

enum BrowserFactoryInner {
    WebGpu(WgpuFactory),
    WebGl2(WebGl2Factory),
}

/// Canvas-bound browser renderer that selects WebGPU or WebGL2.
pub struct BrowserFactory {
    inner: BrowserFactoryInner,
    canvas: HtmlCanvasElement,
    fallback_reason: Option<String>,
}

impl BrowserFactory {
    /// Initializes the requested backend for `canvas` without blocking the
    /// browser event loop.
    pub async fn new(
        canvas: HtmlCanvasElement,
        width: u32,
        height: u32,
        preference: BrowserBackendPreference,
    ) -> Result<Self, RendererError> {
        canvas.set_width(width);
        canvas.set_height(height);
        let (inner, fallback_reason) = match preference {
            BrowserBackendPreference::WebGpu => (
                BrowserFactoryInner::WebGpu(WgpuFactory::new_async(width, height).await?),
                None,
            ),
            BrowserBackendPreference::WebGl2 => (
                BrowserFactoryInner::WebGl2(WebGl2Factory::new(&canvas, width, height)?),
                None,
            ),
            BrowserBackendPreference::Auto => match WgpuFactory::new_async(width, height).await {
                Ok(factory) => (BrowserFactoryInner::WebGpu(factory), None),
                Err(webgpu_error) => {
                    let reason = webgpu_error.to_string();
                    let webgl2 =
                        WebGl2Factory::new(&canvas, width, height).map_err(|webgl2_error| {
                            RendererError::WebGl2(format!(
                            "automatic fallback failed; WebGPU: {reason}; WebGL2: {webgl2_error}"
                        ))
                        })?;
                    (BrowserFactoryInner::WebGl2(webgl2), Some(reason))
                }
            },
        };
        Ok(Self {
            inner,
            canvas,
            fallback_reason,
        })
    }

    /// Returns the backend selected during initialization.
    pub fn backend(&self) -> BrowserBackend {
        match &self.inner {
            BrowserFactoryInner::WebGpu(_) => BrowserBackend::WebGpu,
            BrowserFactoryInner::WebGl2(_) => BrowserBackend::WebGl2,
        }
    }

    /// Returns the WebGPU initialization error when `Auto` selected WebGL2.
    pub fn fallback_reason(&self) -> Option<&str> {
        self.fallback_reason.as_deref()
    }

    /// Returns adapter information when the selected backend is WebGPU.
    pub fn webgpu_adapter_info(&self) -> Option<&WgpuAdapterInfo> {
        match &self.inner {
            BrowserFactoryInner::WebGpu(factory) => Some(factory.adapter_info()),
            BrowserFactoryInner::WebGl2(_) => None,
        }
    }

    /// Begins a frame for the canvas.
    ///
    /// Only one WebGL2 frame may be active for a factory at a time.
    pub fn begin_frame(&self, clear_color: ColorInt) -> Result<BrowserFrame, RendererError> {
        let inner = match &self.inner {
            BrowserFactoryInner::WebGpu(factory) => {
                BrowserFrameInner::WebGpu(factory.begin_frame(clear_color))
            }
            BrowserFactoryInner::WebGl2(factory) => {
                BrowserFrameInner::WebGl2(factory.begin_frame(clear_color)?)
            }
        };
        Ok(BrowserFrame {
            inner,
            canvas: self.canvas.clone(),
        })
    }
}

impl Factory for BrowserFactory {
    fn make_render_buffer(
        &mut self,
        buffer_type: RenderBufferType,
        flags: RenderBufferFlags,
        size_in_bytes: usize,
    ) -> Box<dyn RenderBuffer> {
        match &mut self.inner {
            BrowserFactoryInner::WebGpu(factory) => {
                factory.make_render_buffer(buffer_type, flags, size_in_bytes)
            }
            BrowserFactoryInner::WebGl2(factory) => {
                factory.make_render_buffer(buffer_type, flags, size_in_bytes)
            }
        }
    }

    fn make_linear_gradient(
        &mut self,
        sx: f32,
        sy: f32,
        ex: f32,
        ey: f32,
        colors: &[ColorInt],
        stops: &[f32],
    ) -> Box<dyn RenderShader> {
        match &mut self.inner {
            BrowserFactoryInner::WebGpu(factory) => {
                factory.make_linear_gradient(sx, sy, ex, ey, colors, stops)
            }
            BrowserFactoryInner::WebGl2(factory) => {
                factory.make_linear_gradient(sx, sy, ex, ey, colors, stops)
            }
        }
    }

    fn make_radial_gradient(
        &mut self,
        cx: f32,
        cy: f32,
        radius: f32,
        colors: &[ColorInt],
        stops: &[f32],
    ) -> Box<dyn RenderShader> {
        match &mut self.inner {
            BrowserFactoryInner::WebGpu(factory) => {
                factory.make_radial_gradient(cx, cy, radius, colors, stops)
            }
            BrowserFactoryInner::WebGl2(factory) => {
                factory.make_radial_gradient(cx, cy, radius, colors, stops)
            }
        }
    }

    fn make_render_path(&mut self, raw_path: RawPath, fill_rule: FillRule) -> Box<dyn RenderPath> {
        match &mut self.inner {
            BrowserFactoryInner::WebGpu(factory) => factory.make_render_path(raw_path, fill_rule),
            BrowserFactoryInner::WebGl2(factory) => factory.make_render_path(raw_path, fill_rule),
        }
    }

    fn make_empty_render_path(&mut self) -> Box<dyn RenderPath> {
        match &mut self.inner {
            BrowserFactoryInner::WebGpu(factory) => factory.make_empty_render_path(),
            BrowserFactoryInner::WebGl2(factory) => factory.make_empty_render_path(),
        }
    }

    fn make_render_paint(&mut self) -> Box<dyn RenderPaint> {
        match &mut self.inner {
            BrowserFactoryInner::WebGpu(factory) => factory.make_render_paint(),
            BrowserFactoryInner::WebGl2(factory) => factory.make_render_paint(),
        }
    }

    fn decode_image(&mut self, data: &[u8]) -> Result<Box<dyn RenderImage>, ImageDecodeError> {
        match &mut self.inner {
            BrowserFactoryInner::WebGpu(factory) => factory.decode_image(data),
            BrowserFactoryInner::WebGl2(factory) => factory.decode_image(data),
        }
    }
}

enum BrowserFrameInner {
    WebGpu(WgpuFrame),
    WebGl2(WebGl2Frame),
}

/// In-progress browser frame created by [`BrowserFactory::begin_frame`].
pub struct BrowserFrame {
    inner: BrowserFrameInner,
    canvas: HtmlCanvasElement,
}

impl BrowserFrame {
    /// Submits the frame, presents it to the canvas, and returns RGBA pixels.
    ///
    /// WebGPU submission and readback are asynchronous. WebGL2 flushes and
    /// reads the canvas before resolving this future.
    pub async fn finish(self) -> Result<Vec<u8>, RendererError> {
        match self.inner {
            BrowserFrameInner::WebGpu(frame) => {
                let pixels = frame.finish_async().await?;
                present_pixels(&self.canvas, &pixels)?;
                Ok(pixels)
            }
            BrowserFrameInner::WebGl2(frame) => frame.finish(),
        }
    }
}

impl Renderer for BrowserFrame {
    fn save(&mut self) {
        match &mut self.inner {
            BrowserFrameInner::WebGpu(frame) => frame.save(),
            BrowserFrameInner::WebGl2(frame) => frame.save(),
        }
    }

    fn restore(&mut self) {
        match &mut self.inner {
            BrowserFrameInner::WebGpu(frame) => frame.restore(),
            BrowserFrameInner::WebGl2(frame) => frame.restore(),
        }
    }

    fn transform(&mut self, transform: Mat2D) {
        match &mut self.inner {
            BrowserFrameInner::WebGpu(frame) => frame.transform(transform),
            BrowserFrameInner::WebGl2(frame) => frame.transform(transform),
        }
    }

    fn draw_path(&mut self, path: &dyn RenderPath, paint: &dyn RenderPaint) {
        match &mut self.inner {
            BrowserFrameInner::WebGpu(frame) => frame.draw_path(path, paint),
            BrowserFrameInner::WebGl2(frame) => frame.draw_path(path, paint),
        }
    }

    fn clip_path(&mut self, path: &dyn RenderPath) {
        match &mut self.inner {
            BrowserFrameInner::WebGpu(frame) => frame.clip_path(path),
            BrowserFrameInner::WebGl2(frame) => frame.clip_path(path),
        }
    }

    fn draw_image(
        &mut self,
        image: Option<&dyn RenderImage>,
        sampler: ImageSampler,
        blend_mode: BlendMode,
        opacity: f32,
    ) {
        match &mut self.inner {
            BrowserFrameInner::WebGpu(frame) => {
                frame.draw_image(image, sampler, blend_mode, opacity)
            }
            BrowserFrameInner::WebGl2(frame) => {
                frame.draw_image(image, sampler, blend_mode, opacity)
            }
        }
    }

    fn draw_image_mesh(
        &mut self,
        image: Option<&dyn RenderImage>,
        sampler: ImageSampler,
        vertices: Option<&dyn RenderBuffer>,
        uv_coords: Option<&dyn RenderBuffer>,
        indices: Option<&dyn RenderBuffer>,
        vertex_count: u32,
        index_count: u32,
        blend_mode: BlendMode,
        opacity: f32,
    ) {
        match &mut self.inner {
            BrowserFrameInner::WebGpu(frame) => frame.draw_image_mesh(
                image,
                sampler,
                vertices,
                uv_coords,
                indices,
                vertex_count,
                index_count,
                blend_mode,
                opacity,
            ),
            BrowserFrameInner::WebGl2(frame) => frame.draw_image_mesh(
                image,
                sampler,
                vertices,
                uv_coords,
                indices,
                vertex_count,
                index_count,
                blend_mode,
                opacity,
            ),
        }
    }

    fn modulate_opacity(&mut self, opacity: f32) {
        match &mut self.inner {
            BrowserFrameInner::WebGpu(frame) => frame.modulate_opacity(opacity),
            BrowserFrameInner::WebGl2(frame) => frame.modulate_opacity(opacity),
        }
    }
}

fn present_pixels(canvas: &HtmlCanvasElement, pixels: &[u8]) -> Result<(), RendererError> {
    let context = canvas
        .get_context("2d")
        .map_err(js_error)?
        .ok_or_else(|| RendererError::WebGl2("canvas has no 2D presentation context".into()))?
        .dyn_into::<CanvasRenderingContext2d>()
        .map_err(|error| js_error(error.into()))?;
    let image = ImageData::new_with_u8_clamped_array_and_sh(
        Clamped(pixels),
        canvas.width(),
        canvas.height(),
    )
    .map_err(js_error)?;
    context.put_image_data(&image, 0.0, 0.0).map_err(js_error)
}

fn js_error(error: wasm_bindgen::JsValue) -> RendererError {
    RendererError::WebGl2(
        error
            .as_string()
            .unwrap_or_else(|| format!("browser JavaScript error: {error:?}")),
    )
}
