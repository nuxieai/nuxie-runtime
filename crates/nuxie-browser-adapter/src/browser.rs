use crate::browser_surface_lifecycle::{
    SurfaceAcquisitionFailure, SurfaceRecoveryAction, SurfaceRecoveryError, acquire_surface_texture,
};
use nuxie_render_api::{
    BlendMode, ColorInt, Factory, FillRule, GpuCanvasError, GpuCanvasPlan, GpuCanvasShader,
    ImageDecodeError, ImageSampler, Mat2D, RawPath, RenderBuffer, RenderBufferFlags,
    RenderBufferType, RenderGpuCanvasShader, RenderImage, RenderPaint, RenderPath, RenderShader,
    Renderer,
};
use nuxie_renderer::{
    RendererError, WgpuAdapterInfo, WgpuFactory, WgpuFrame, WgpuPresentationAcquireError,
    WgpuPresentationAlpha, WgpuPresentationFrame, WgpuPresentationSurface,
};
use raw_window_handle::{
    DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, WebCanvasWindowHandle,
    WindowHandle,
};
use std::cell::Cell;
use std::cell::RefCell;
use std::error::Error;
use std::fmt;
use std::rc::Rc;
use std::sync::Arc;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::HtmlCanvasElement;

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
    presentation: Rc<BrowserPresentation>,
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
        let presentation = Rc::new(BrowserPresentation::new(
            &inner,
            canvas.clone(),
            width,
            height,
        )?);
        Ok(Self {
            inner,
            canvas,
            presentation,
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
        self.presentation.configure(width, height);
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
            presentation: Rc::clone(&self.presentation),
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

    fn make_gpu_canvas_shader(
        &mut self,
        shader: &GpuCanvasShader,
    ) -> Result<Arc<dyn RenderGpuCanvasShader>, GpuCanvasError> {
        self.inner.make_gpu_canvas_shader(shader)
    }

    fn make_gpu_canvas_image(
        &mut self,
        vertex_shader: &Arc<dyn RenderGpuCanvasShader>,
        fragment_shader: &Arc<dyn RenderGpuCanvasShader>,
        plan: &GpuCanvasPlan,
    ) -> Result<Box<dyn RenderImage>, GpuCanvasError> {
        self.inner
            .make_gpu_canvas_image(vertex_shader, fragment_shader, plan)
    }
}

/// In-progress browser frame created by [`BrowserFactory::begin_frame`].
pub struct BrowserFrame {
    inner: WgpuFrame,
    presentation: Rc<BrowserPresentation>,
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
    /// Submits the frame directly to the WebGPU canvas surface.
    pub async fn present(self) -> Result<(), RendererError> {
        let Self {
            inner,
            presentation,
            lease,
        } = self;
        presentation.current_frame()?.present(inner).await?;
        drop(lease);
        Ok(())
    }

    /// Finishes the frame and returns exactly `width * height * 4` RGBA bytes.
    ///
    /// This explicit capture path performs a GPU-to-CPU readback. Use [`Self::present`]
    /// for ordinary browser frames.
    pub async fn finish_with_readback(self) -> Result<Vec<u8>, RendererError> {
        let Self {
            inner,
            presentation: _,
            lease,
        } = self;
        let pixels = inner.finish_async().await?;
        drop(lease);
        Ok(pixels)
    }
}

struct BrowserPresentation {
    canvas: HtmlCanvasElement,
    surface: RefCell<WgpuPresentationSurface>,
    width: Cell<u32>,
    height: Cell<u32>,
}

impl BrowserPresentation {
    fn new(
        factory: &WgpuFactory,
        canvas: HtmlCanvasElement,
        width: u32,
        height: u32,
    ) -> Result<Self, RendererError> {
        let surface = factory.create_presentation_surface(
            CanvasSurfaceTarget(canvas.clone()),
            width,
            height,
            WgpuPresentationAlpha::Premultiplied,
        )?;
        Ok(Self {
            canvas,
            surface: RefCell::new(surface),
            width: Cell::new(width),
            height: Cell::new(height),
        })
    }

    fn configure(&self, width: u32, height: u32) {
        self.surface.borrow_mut().configure(width, height);
        self.width.set(width);
        self.height.set(height);
    }

    fn current_frame(&self) -> Result<WgpuPresentationFrame, RendererError> {
        acquire_surface_texture(
            || self.acquire_current_frame(),
            |action| match action {
                SurfaceRecoveryAction::ReconfigureAndRetry => self.reconfigure_surface(),
                SurfaceRecoveryAction::RecreateAndRetry => self.recreate_surface(),
            },
        )
        .map_err(|error| match error {
            SurfaceRecoveryError::Acquisition { failure, recovery } => {
                surface_acquisition_error(failure, recovery)
            }
            SurfaceRecoveryError::Recovery(error) => error,
        })
    }

    fn acquire_current_frame(&self) -> Result<WgpuPresentationFrame, SurfaceAcquisitionFailure> {
        self.surface.borrow().acquire().map_err(surface_failure)
    }

    fn reconfigure_surface(&self) -> Result<(), RendererError> {
        self.surface
            .borrow_mut()
            .configure(self.width.get(), self.height.get());
        Ok(())
    }

    fn recreate_surface(&self) -> Result<(), RendererError> {
        self.surface
            .borrow_mut()
            .recreate(CanvasSurfaceTarget(self.canvas.clone()))
    }
}

#[derive(Clone)]
struct CanvasSurfaceTarget(HtmlCanvasElement);

impl HasWindowHandle for CanvasSurfaceTarget {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        let value: &JsValue = &self.0;
        let raw = WebCanvasWindowHandle::from_wasm_bindgen_0_2(value).into();
        // SAFETY: `raw` points at the JsValue retained by this owned target.
        // wgpu boxes the target before asking for the handle and retains that
        // box for the complete surface lifetime.
        Ok(unsafe { WindowHandle::borrow_raw(raw) })
    }
}

impl HasDisplayHandle for CanvasSurfaceTarget {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        Ok(DisplayHandle::web())
    }
}

fn surface_acquisition_error(
    failure: SurfaceAcquisitionFailure,
    recovery: Option<SurfaceRecoveryAction>,
) -> RendererError {
    let status = match failure {
        SurfaceAcquisitionFailure::Timeout => "acquisition timed out",
        SurfaceAcquisitionFailure::Occluded => "is occluded",
        SurfaceAcquisitionFailure::Outdated => "remained outdated",
        SurfaceAcquisitionFailure::Lost => "remained lost",
        SurfaceAcquisitionFailure::Validation => "acquisition failed validation",
    };
    let recovery = recovery
        .map(|recovery| match recovery {
            SurfaceRecoveryAction::ReconfigureAndRetry => {
                " after surface reconfiguration".to_owned()
            }
            SurfaceRecoveryAction::RecreateAndRetry => " after surface recreation".to_owned(),
        })
        .unwrap_or_default();
    RendererError::Device(format!("browser WebGPU canvas surface {status}{recovery}"))
}

fn surface_failure(error: WgpuPresentationAcquireError) -> SurfaceAcquisitionFailure {
    match error {
        WgpuPresentationAcquireError::Timeout => SurfaceAcquisitionFailure::Timeout,
        WgpuPresentationAcquireError::Occluded => SurfaceAcquisitionFailure::Occluded,
        WgpuPresentationAcquireError::Outdated => SurfaceAcquisitionFailure::Outdated,
        WgpuPresentationAcquireError::Lost => SurfaceAcquisitionFailure::Lost,
        WgpuPresentationAcquireError::Validation => SurfaceAcquisitionFailure::Validation,
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
