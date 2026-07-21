use super::{RendererError, WebGl2Factory, WebGl2Frame, WgpuAdapterInfo, WgpuFactory, WgpuFrame};
use nuxie_render_api::{
    BlendMode, ColorInt, Factory, FillRule, GpuCanvasError, GpuCanvasPlan, GpuCanvasShader,
    ImageDecodeError, ImageSampler, Mat2D, RawPath, RenderBuffer, RenderBufferFlags,
    RenderBufferType, RenderImage, RenderPaint, RenderPath, RenderShader, Renderer,
};
use std::cell::Cell;
use std::error::Error;
use std::fmt;
use std::rc::Rc;
use wasm_bindgen::{Clamped, JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
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

/// Failure to retarget a browser renderer's canvas.
///
/// The lifecycle case is backend-independent: a resize never mutates the
/// active frame or queues a hidden resize behind it. Callers may retry after
/// that frame finishes while continuing to use document-side APIs.
#[derive(Debug)]
pub enum BrowserResizeError {
    /// A frame created by this factory has not finished or been dropped yet.
    FrameInFlight,
    /// The selected renderer rejected the requested target extent.
    Renderer(RendererError),
}

impl fmt::Display for BrowserResizeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FrameInFlight => {
                formatter.write_str("cannot resize while a browser frame is in flight")
            }
            Self::Renderer(error) => write!(formatter, "browser resize failed: {error}"),
        }
    }
}

impl Error for BrowserResizeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::FrameInFlight => None,
            Self::Renderer(error) => Some(error),
        }
    }
}

impl From<RendererError> for BrowserResizeError {
    fn from(error: RendererError) -> Self {
        Self::Renderer(error)
    }
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
    width: u32,
    height: u32,
    active_frames: Rc<Cell<u32>>,
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
            BrowserBackendPreference::WebGpu => {
                probe_webgpu_adapter()
                    .await
                    .map_err(RendererError::Adapter)?;
                (
                    BrowserFactoryInner::WebGpu(WgpuFactory::new_async(width, height).await?),
                    None,
                )
            }
            BrowserBackendPreference::WebGl2 => (
                BrowserFactoryInner::WebGl2(WebGl2Factory::new(&canvas, width, height)?),
                None,
            ),
            BrowserBackendPreference::Auto => match probe_webgpu_adapter().await {
                Ok(()) => match WgpuFactory::new_async(width, height).await {
                    Ok(factory) => (BrowserFactoryInner::WebGpu(factory), None),
                    Err(webgpu_error) => {
                        let reason = webgpu_error.to_string();
                        let webgl2 = WebGl2Factory::new(&canvas, width, height).map_err(
                            |webgl2_error| {
                                RendererError::WebGl2(format!(
                                    "automatic fallback failed; WebGPU: {reason}; WebGL2: {webgl2_error}"
                                ))
                            },
                        )?;
                        (BrowserFactoryInner::WebGl2(webgl2), Some(reason))
                    }
                },
                Err(reason) => {
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
            width,
            height,
            active_frames: Rc::new(Cell::new(0)),
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

    /// Returns the current physical render-target size.
    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Retargets the selected renderer and canvas for future frames.
    ///
    /// Resizing is synchronous and never recreates the factory's device or
    /// leaks backend selection into the caller. If a frame is in flight, this
    /// returns [`BrowserResizeError::FrameInFlight`] without changing state;
    /// callers may retry after frame completion.
    pub fn resize(&mut self, width: u32, height: u32) -> Result<(), BrowserResizeError> {
        if self.active_frames.get() != 0 {
            return Err(BrowserResizeError::FrameInFlight);
        }
        if (width, height) == (self.width, self.height) {
            return Ok(());
        }
        match &mut self.inner {
            BrowserFactoryInner::WebGpu(factory) => {
                factory.resize(width, height)?;
                self.canvas.set_width(width);
                self.canvas.set_height(height);
            }
            BrowserFactoryInner::WebGl2(factory) => factory.resize(width, height)?,
        }
        self.width = width;
        self.height = height;
        Ok(())
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
        self.active_frames
            .set(self.active_frames.get().saturating_add(1));
        Ok(BrowserFrame {
            inner,
            canvas: self.canvas.clone(),
            lease: BrowserFrameLease {
                active_frames: Rc::clone(&self.active_frames),
            },
        })
    }
}

/// Probes the browser API before entering wgpu's adapter future. Some WebGPU
/// implementations resolve `GPU.requestAdapter()` with `null` when no adapter
/// is available, while the corresponding wgpu wasm future may remain pending.
/// Reading that result directly keeps automatic WebGL2 fallback and explicit
/// WebGPU failure finite.
async fn probe_webgpu_adapter() -> Result<(), String> {
    let global = js_sys::global();
    let navigator =
        js_sys::Reflect::get(&global, &JsValue::from_str("navigator")).map_err(|error| {
            format!(
                "browser navigator lookup failed: {}",
                js_value_message(error)
            )
        })?;
    if navigator.is_null() || navigator.is_undefined() {
        return Err("browser navigator is unavailable".into());
    }
    let gpu = js_sys::Reflect::get(&navigator, &JsValue::from_str("gpu"))
        .map_err(|error| format!("WebGPU API lookup failed: {}", js_value_message(error)))?;
    if gpu.is_null() || gpu.is_undefined() {
        return Err("WebGPU API is unavailable".into());
    }
    let request_adapter = js_sys::Reflect::get(&gpu, &JsValue::from_str("requestAdapter"))
        .map_err(|error| {
            format!(
                "WebGPU adapter probe lookup failed: {}",
                js_value_message(error)
            )
        })?
        .dyn_into::<js_sys::Function>()
        .map_err(|_| "WebGPU requestAdapter is unavailable".to_string())?;
    let request = request_adapter
        .call0(&gpu)
        .map_err(|error| format!("WebGPU adapter probe failed: {}", js_value_message(error)))?
        .dyn_into::<js_sys::Promise>()
        .map_err(|_| "WebGPU requestAdapter returned a non-Promise value".to_string())?;
    let adapter = JsFuture::from(request)
        .await
        .map_err(|error| format!("WebGPU adapter probe failed: {}", js_value_message(error)))?;
    if adapter.is_null() || adapter.is_undefined() {
        return Err("WebGPU adapter is unavailable".into());
    }
    Ok(())
}

fn js_value_message(error: JsValue) -> String {
    error.as_string().unwrap_or_else(|| format!("{error:?}"))
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

    fn make_gpu_canvas_image(
        &mut self,
        shader: &GpuCanvasShader,
        plan: &GpuCanvasPlan,
    ) -> Result<Box<dyn RenderImage>, GpuCanvasError> {
        match &mut self.inner {
            BrowserFactoryInner::WebGpu(factory) => factory.make_gpu_canvas_image(shader, plan),
            BrowserFactoryInner::WebGl2(factory) => factory.make_gpu_canvas_image(shader, plan),
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
    lease: BrowserFrameLease,
}

struct BrowserFrameLease {
    active_frames: Rc<Cell<u32>>,
}

impl Drop for BrowserFrameLease {
    fn drop(&mut self) {
        self.active_frames
            .set(self.active_frames.get().saturating_sub(1));
    }
}

impl BrowserFrame {
    /// Submits the frame, presents it to the canvas, and returns RGBA pixels.
    ///
    /// WebGPU submission and readback are asynchronous. WebGL2 flushes and
    /// reads the canvas before resolving this future.
    pub async fn finish(self) -> Result<Vec<u8>, RendererError> {
        let Self {
            inner,
            canvas,
            lease,
        } = self;
        let result = match inner {
            BrowserFrameInner::WebGpu(frame) => {
                let pixels = frame.finish_async().await?;
                present_pixels(&canvas, &pixels)?;
                Ok(pixels)
            }
            BrowserFrameInner::WebGl2(frame) => frame.finish(),
        };
        drop(lease);
        result
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
