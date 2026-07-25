use super::{RendererError, WgpuAdapterInfo, WgpuFactory, WgpuFrame};
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

/// Failure to retarget a browser renderer's canvas.
#[derive(Debug)]
pub enum BrowserResizeError {
    /// A frame created by this factory has not finished or been dropped yet.
    FrameInFlight,
    /// The renderer rejected the requested target extent.
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

/// Canvas-bound WebGPU renderer.
pub struct BrowserFactory {
    inner: WgpuFactory,
    canvas: HtmlCanvasElement,
    width: u32,
    height: u32,
    active_frames: Rc<Cell<u32>>,
}

impl BrowserFactory {
    /// Initializes WebGPU for `canvas` without blocking the browser event loop.
    ///
    /// WebGPU Core admission is attempted first, followed by Compatibility.
    /// Browsers without the API or a usable adapter return
    /// [`RendererError::Adapter`].
    pub async fn new(
        canvas: HtmlCanvasElement,
        width: u32,
        height: u32,
    ) -> Result<Self, RendererError> {
        probe_webgpu_adapter()
            .await
            .map_err(RendererError::Adapter)?;
        let inner = WgpuFactory::new_async(width, height).await?;
        canvas.set_width(width);
        canvas.set_height(height);
        Ok(Self {
            inner,
            canvas,
            width,
            height,
            active_frames: Rc::new(Cell::new(0)),
        })
    }

    /// Returns information about the selected WebGPU adapter.
    pub fn webgpu_adapter_info(&self) -> &WgpuAdapterInfo {
        self.inner.adapter_info()
    }

    /// Returns the current physical render-target size.
    pub fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    /// Retargets the renderer and canvas for future frames.
    ///
    /// If a frame is in flight, this returns
    /// [`BrowserResizeError::FrameInFlight`] without changing state; callers
    /// may retry after frame completion.
    pub fn resize(&mut self, width: u32, height: u32) -> Result<(), BrowserResizeError> {
        if self.active_frames.get() != 0 {
            return Err(BrowserResizeError::FrameInFlight);
        }
        if (width, height) == (self.width, self.height) {
            return Ok(());
        }
        self.inner.resize(width, height)?;
        self.canvas.set_width(width);
        self.canvas.set_height(height);
        self.width = width;
        self.height = height;
        Ok(())
    }

    /// Begins a frame for the canvas.
    pub fn begin_frame(&self, clear_color: ColorInt) -> Result<BrowserFrame, RendererError> {
        self.active_frames
            .set(self.active_frames.get().saturating_add(1));
        Ok(BrowserFrame {
            inner: self.inner.begin_frame(clear_color),
            canvas: self.canvas.clone(),
            lease: BrowserFrameLease {
                active_frames: Rc::clone(&self.active_frames),
            },
        })
    }
}

/// Probes the browser API before entering wgpu's adapter future. Some browser
/// implementations resolve `GPU.requestAdapter()` with `null` when no adapter
/// is available, while the corresponding wgpu wasm future may remain pending.
/// Match [`WgpuFactory`] admission by trying Core first and Compatibility
/// second.
async fn probe_webgpu_adapter() -> Result<(), String> {
    let global = js_sys::global();
    let navigator =
        js_sys::Reflect::get(&global, &JsValue::from_str("navigator")).map_err(|error| {
            format!(
                "WebGPU initialization could not read browser navigator: {}",
                js_value_message(error)
            )
        })?;
    if navigator.is_null() || navigator.is_undefined() {
        return Err("WebGPU initialization failed: browser navigator is unavailable".into());
    }
    let gpu = js_sys::Reflect::get(&navigator, &JsValue::from_str("gpu")).map_err(|error| {
        format!(
            "WebGPU initialization could not inspect navigator.gpu: {}",
            js_value_message(error)
        )
    })?;
    if gpu.is_null() || gpu.is_undefined() {
        return Err(
            "WebGPU API is unavailable; this browser renderer requires navigator.gpu".into(),
        );
    }
    let request_adapter = js_sys::Reflect::get(&gpu, &JsValue::from_str("requestAdapter"))
        .map_err(|error| {
            format!(
                "WebGPU adapter probe could not read requestAdapter: {}",
                js_value_message(error)
            )
        })?
        .dyn_into::<js_sys::Function>()
        .map_err(|_| "WebGPU requestAdapter is unavailable".to_string())?;
    let core_error = match probe_webgpu_adapter_level(&gpu, &request_adapter, None).await {
        Ok(()) => return Ok(()),
        Err(error) => error,
    };
    probe_webgpu_adapter_level(&gpu, &request_adapter, Some("compatibility"))
        .await
        .map_err(|compatibility_error| {
            format!(
                "no usable WebGPU adapter: Core probe failed ({core_error}); Compatibility probe failed ({compatibility_error})"
            )
        })
}

async fn probe_webgpu_adapter_level(
    gpu: &JsValue,
    request_adapter: &js_sys::Function,
    feature_level: Option<&str>,
) -> Result<(), String> {
    let request = match feature_level {
        Some(feature_level) => {
            let options = js_sys::Object::new();
            js_sys::Reflect::set(
                &options,
                &JsValue::from_str("featureLevel"),
                &JsValue::from_str(feature_level),
            )
            .map_err(|error| {
                format!(
                    "WebGPU adapter probe options failed: {}",
                    js_value_message(error)
                )
            })?;
            request_adapter.call1(gpu, &options)
        }
        None => request_adapter.call0(gpu),
    }
    .map_err(|error| format!("WebGPU requestAdapter failed: {}", js_value_message(error)))?
    .dyn_into::<js_sys::Promise>()
    .map_err(|_| "WebGPU requestAdapter returned a non-Promise value".to_string())?;
    let adapter = JsFuture::from(request)
        .await
        .map_err(|error| format!("WebGPU requestAdapter failed: {}", js_value_message(error)))?;
    if adapter.is_null() || adapter.is_undefined() {
        return Err("adapter is unavailable".into());
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
        self.inner
            .make_render_buffer(buffer_type, flags, size_in_bytes)
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
        self.inner
            .make_linear_gradient(sx, sy, ex, ey, colors, stops)
    }

    fn make_radial_gradient(
        &mut self,
        cx: f32,
        cy: f32,
        radius: f32,
        colors: &[ColorInt],
        stops: &[f32],
    ) -> Box<dyn RenderShader> {
        self.inner
            .make_radial_gradient(cx, cy, radius, colors, stops)
    }

    fn make_render_path(&mut self, raw_path: RawPath, fill_rule: FillRule) -> Box<dyn RenderPath> {
        self.inner.make_render_path(raw_path, fill_rule)
    }

    fn make_empty_render_path(&mut self) -> Box<dyn RenderPath> {
        self.inner.make_empty_render_path()
    }

    fn make_render_paint(&mut self) -> Box<dyn RenderPaint> {
        self.inner.make_render_paint()
    }

    fn decode_image(&mut self, data: &[u8]) -> Result<Box<dyn RenderImage>, ImageDecodeError> {
        self.inner.decode_image(data)
    }

    fn make_gpu_canvas_image(
        &mut self,
        shader: &GpuCanvasShader,
        plan: &GpuCanvasPlan,
    ) -> Result<Box<dyn RenderImage>, GpuCanvasError> {
        self.inner.make_gpu_canvas_image(shader, plan)
    }
}

/// In-progress browser frame created by [`BrowserFactory::begin_frame`].
pub struct BrowserFrame {
    inner: WgpuFrame,
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
    pub async fn finish(self) -> Result<Vec<u8>, RendererError> {
        let Self {
            inner,
            canvas,
            lease,
        } = self;
        let pixels = inner.finish_async().await?;
        present_pixels(&canvas, &pixels)?;
        drop(lease);
        Ok(pixels)
    }
}

impl Renderer for BrowserFrame {
    fn save(&mut self) {
        self.inner.save();
    }

    fn restore(&mut self) {
        self.inner.restore();
    }

    fn transform(&mut self, transform: Mat2D) {
        self.inner.transform(transform);
    }

    fn draw_path(&mut self, path: &dyn RenderPath, paint: &dyn RenderPaint) {
        self.inner.draw_path(path, paint);
    }

    fn clip_path(&mut self, path: &dyn RenderPath) {
        self.inner.clip_path(path);
    }

    fn draw_image(
        &mut self,
        image: Option<&dyn RenderImage>,
        sampler: ImageSampler,
        blend_mode: BlendMode,
        opacity: f32,
    ) {
        self.inner.draw_image(image, sampler, blend_mode, opacity);
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
        self.inner.draw_image_mesh(
            image,
            sampler,
            vertices,
            uv_coords,
            indices,
            vertex_count,
            index_count,
            blend_mode,
            opacity,
        );
    }

    fn modulate_opacity(&mut self, opacity: f32) {
        self.inner.modulate_opacity(opacity);
    }
}

fn present_pixels(canvas: &HtmlCanvasElement, pixels: &[u8]) -> Result<(), RendererError> {
    let context = canvas
        .get_context("2d")
        .map_err(js_error)?
        .ok_or_else(|| {
            RendererError::Device("browser canvas has no 2D presentation context".into())
        })?
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

fn js_error(error: JsValue) -> RendererError {
    RendererError::Device(format!(
        "browser canvas presentation failed: {}",
        error
            .as_string()
            .unwrap_or_else(|| format!("browser JavaScript error: {error:?}"))
    ))
}
