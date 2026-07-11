//! FFI-renderer bridge: Rust `Factory`/`Renderer` implementations that forward
//! every nuxie-render-api trait call to a caller-provided C vtable.
//!
//! The C caller owns every render object it creates: each `make_*` callback
//! returns an opaque `uint64_t` handle chosen by the caller, later mutation
//! and draw callbacks receive that handle back, and the matching `release_*`
//! callback fires exactly once when the Rust side drops the object. Every
//! callback is optional (may be NULL); missing callbacks degrade to no-ops so
//! a zeroed vtable behaves like a null renderer.

use nuxie::{
    BlendMode, ColorInt, Factory, FillRule, ImageSampler, Mat2D, RawPath, RenderBuffer,
    RenderBufferFlags, RenderBufferType, RenderImage, RenderPaint, RenderPaintStyle, RenderPath,
    RenderShader, Renderer, StrokeCap, StrokeJoin,
};
use std::any::Any;
use std::ffi::c_void;

/// Borrowed view of a [`RawPath`]: `verbs` holds `NuxPathVerb` values and
/// `points` holds `point_count` interleaved x,y pairs. Only valid for the
/// duration of the callback it is passed to.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NuxRawPathView {
    pub verbs: *const u8,
    pub verb_count: usize,
    pub points: *const f32,
    pub point_count: usize,
}

/// C mirror of [`ImageSampler`] using the `NuxImageWrap`/`NuxImageFilter`
/// enum values.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NuxImageSampler {
    pub wrap_x: u8,
    pub wrap_y: u8,
    pub filter: u8,
}

impl From<ImageSampler> for NuxImageSampler {
    fn from(sampler: ImageSampler) -> Self {
        Self {
            wrap_x: sampler.wrap_x as u8,
            wrap_y: sampler.wrap_y as u8,
            filter: sampler.filter as u8,
        }
    }
}

/// Caller-provided render vtable mirroring the nuxie-render-api traits.
///
/// Handles are opaque `uint64_t` values chosen by the caller; `0` is passed
/// for "no object" (for example a cleared shader). Transform arguments point
/// at six floats in `[xx, yx, xy, yy, tx, ty]` order. All callbacks may be
/// NULL.
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct NuxRenderCallbacks {
    pub user_data: *mut c_void,

    // Factory calls.
    pub make_render_path:
        Option<unsafe extern "C" fn(*mut c_void, *const NuxRawPathView, u8) -> u64>,
    pub make_empty_render_path: Option<unsafe extern "C" fn(*mut c_void) -> u64>,
    pub make_render_paint: Option<unsafe extern "C" fn(*mut c_void) -> u64>,
    pub make_linear_gradient: Option<
        unsafe extern "C" fn(*mut c_void, f32, f32, f32, f32, *const u32, *const f32, usize) -> u64,
    >,
    pub make_radial_gradient: Option<
        unsafe extern "C" fn(*mut c_void, f32, f32, f32, *const u32, *const f32, usize) -> u64,
    >,
    pub make_render_buffer: Option<unsafe extern "C" fn(*mut c_void, u8, u8, usize) -> u64>,
    pub decode_image:
        Option<unsafe extern "C" fn(*mut c_void, *const u8, usize, *mut u32, *mut u32) -> u64>,

    // Object releases (paired with the factory calls above).
    pub release_render_path: Option<unsafe extern "C" fn(*mut c_void, u64)>,
    pub release_render_paint: Option<unsafe extern "C" fn(*mut c_void, u64)>,
    pub release_render_shader: Option<unsafe extern "C" fn(*mut c_void, u64)>,
    pub release_render_buffer: Option<unsafe extern "C" fn(*mut c_void, u64)>,
    pub release_render_image: Option<unsafe extern "C" fn(*mut c_void, u64)>,

    // RenderPath mutation.
    pub render_path_rewind: Option<unsafe extern "C" fn(*mut c_void, u64)>,
    pub render_path_fill_rule: Option<unsafe extern "C" fn(*mut c_void, u64, u8)>,
    pub render_path_move_to: Option<unsafe extern "C" fn(*mut c_void, u64, f32, f32)>,
    pub render_path_line_to: Option<unsafe extern "C" fn(*mut c_void, u64, f32, f32)>,
    pub render_path_cubic_to:
        Option<unsafe extern "C" fn(*mut c_void, u64, f32, f32, f32, f32, f32, f32)>,
    pub render_path_close: Option<unsafe extern "C" fn(*mut c_void, u64)>,
    pub render_path_add_raw_path:
        Option<unsafe extern "C" fn(*mut c_void, u64, *const NuxRawPathView)>,
    pub render_path_add_render_path:
        Option<unsafe extern "C" fn(*mut c_void, u64, u64, *const f32)>,
    pub render_path_add_render_path_backwards:
        Option<unsafe extern "C" fn(*mut c_void, u64, u64, *const f32)>,

    // RenderPaint mutation.
    pub render_paint_style: Option<unsafe extern "C" fn(*mut c_void, u64, u8)>,
    pub render_paint_color: Option<unsafe extern "C" fn(*mut c_void, u64, u32)>,
    pub render_paint_thickness: Option<unsafe extern "C" fn(*mut c_void, u64, f32)>,
    pub render_paint_join: Option<unsafe extern "C" fn(*mut c_void, u64, u32)>,
    pub render_paint_cap: Option<unsafe extern "C" fn(*mut c_void, u64, u32)>,
    pub render_paint_feather: Option<unsafe extern "C" fn(*mut c_void, u64, f32)>,
    pub render_paint_blend_mode: Option<unsafe extern "C" fn(*mut c_void, u64, u8)>,
    pub render_paint_shader: Option<unsafe extern "C" fn(*mut c_void, u64, u64)>,
    pub render_paint_invalidate_stroke: Option<unsafe extern "C" fn(*mut c_void, u64)>,

    // RenderBuffer unmap: receives the staged bytes for the buffer handle.
    pub render_buffer_unmap: Option<unsafe extern "C" fn(*mut c_void, u64, *const u8, usize)>,

    // Renderer calls.
    pub save: Option<unsafe extern "C" fn(*mut c_void)>,
    pub restore: Option<unsafe extern "C" fn(*mut c_void)>,
    pub transform: Option<unsafe extern "C" fn(*mut c_void, *const f32)>,
    pub draw_path: Option<unsafe extern "C" fn(*mut c_void, u64, u64)>,
    pub clip_path: Option<unsafe extern "C" fn(*mut c_void, u64)>,
    pub draw_image: Option<unsafe extern "C" fn(*mut c_void, u64, NuxImageSampler, u8, f32)>,
    #[allow(clippy::type_complexity)]
    pub draw_image_mesh: Option<
        unsafe extern "C" fn(*mut c_void, u64, NuxImageSampler, u64, u64, u64, u32, u32, u8, f32),
    >,
    pub modulate_opacity: Option<unsafe extern "C" fn(*mut c_void, f32)>,
}

impl Default for NuxRenderCallbacks {
    /// Empty vtable: every callback NULL, which draws like a null renderer.
    fn default() -> Self {
        // SAFETY: every field is nullable (a raw pointer or `Option` of a
        // function pointer), so the all-zero bit pattern is valid.
        unsafe { std::mem::zeroed() }
    }
}

/// Invoke an optional callback that returns nothing.
macro_rules! call {
    ($callbacks:expr, $field:ident $(, $arg:expr)*) => {
        if let Some(callback) = $callbacks.$field {
            // SAFETY: the caller of `nux_artboard_instance_draw` guarantees
            // every non-NULL callback is valid for the duration of the draw.
            unsafe { callback($callbacks.user_data $(, $arg)*) }
        }
    };
}

/// Invoke an optional callback that returns a handle, defaulting to 0.
macro_rules! call_handle {
    ($callbacks:expr, $field:ident $(, $arg:expr)*) => {
        match $callbacks.$field {
            // SAFETY: see `call!`.
            Some(callback) => unsafe { callback($callbacks.user_data $(, $arg)*) },
            None => 0,
        }
    };
}

fn with_raw_path_view<R>(path: &RawPath, action: impl FnOnce(*const NuxRawPathView) -> R) -> R {
    // SAFETY: PathVerb is #[repr(u8)], so a verb slice can be viewed as bytes.
    let verbs = path.verbs().as_ptr().cast::<u8>();
    let points = path
        .points()
        .iter()
        .flat_map(|point| [point.x, point.y])
        .collect::<Vec<f32>>();
    let view = NuxRawPathView {
        verbs,
        verb_count: path.verbs().len(),
        points: points.as_ptr(),
        point_count: path.points().len(),
    };
    action(&view)
}

fn render_path_handle(path: &dyn RenderPath) -> u64 {
    path.as_any()
        .downcast_ref::<CallbackRenderPath>()
        .map_or(0, |path| path.handle)
}

fn render_paint_handle(paint: &dyn RenderPaint) -> u64 {
    paint
        .as_any()
        .downcast_ref::<CallbackRenderPaint>()
        .map_or(0, |paint| paint.handle)
}

fn render_shader_handle(shader: Option<&dyn RenderShader>) -> u64 {
    shader
        .and_then(|shader| shader.as_any().downcast_ref::<CallbackRenderShader>())
        .map_or(0, |shader| shader.handle)
}

fn render_image_handle(image: Option<&dyn RenderImage>) -> u64 {
    image
        .and_then(|image| image.as_any().downcast_ref::<CallbackRenderImage>())
        .map_or(0, |image| image.handle)
}

fn render_buffer_handle(buffer: Option<&dyn RenderBuffer>) -> u64 {
    buffer
        .and_then(|buffer| buffer.as_any().downcast_ref::<CallbackRenderBuffer>())
        .map_or(0, |buffer| buffer.handle)
}

pub(crate) struct CallbackRenderPath {
    callbacks: NuxRenderCallbacks,
    handle: u64,
}

impl Drop for CallbackRenderPath {
    fn drop(&mut self) {
        call!(self.callbacks, release_render_path, self.handle);
    }
}

impl RenderPath for CallbackRenderPath {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn rewind(&mut self) {
        call!(self.callbacks, render_path_rewind, self.handle);
    }

    fn fill_rule(&mut self, value: FillRule) {
        call!(
            self.callbacks,
            render_path_fill_rule,
            self.handle,
            value as u8
        );
    }

    fn add_render_path(&mut self, path: &dyn RenderPath, transform: Mat2D) {
        call!(
            self.callbacks,
            render_path_add_render_path,
            self.handle,
            render_path_handle(path),
            transform.0.as_ptr()
        );
    }

    fn add_render_path_backwards(&mut self, path: &dyn RenderPath, transform: Mat2D) {
        call!(
            self.callbacks,
            render_path_add_render_path_backwards,
            self.handle,
            render_path_handle(path),
            transform.0.as_ptr()
        );
    }

    fn add_raw_path(&mut self, path: &RawPath) {
        with_raw_path_view(path, |view| {
            call!(self.callbacks, render_path_add_raw_path, self.handle, view);
        });
    }

    fn move_to(&mut self, x: f32, y: f32) {
        call!(self.callbacks, render_path_move_to, self.handle, x, y);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        call!(self.callbacks, render_path_line_to, self.handle, x, y);
    }

    fn cubic_to(&mut self, ox: f32, oy: f32, ix: f32, iy: f32, x: f32, y: f32) {
        call!(
            self.callbacks,
            render_path_cubic_to,
            self.handle,
            ox,
            oy,
            ix,
            iy,
            x,
            y
        );
    }

    fn close(&mut self) {
        call!(self.callbacks, render_path_close, self.handle);
    }
}

pub(crate) struct CallbackRenderPaint {
    callbacks: NuxRenderCallbacks,
    handle: u64,
}

impl Drop for CallbackRenderPaint {
    fn drop(&mut self) {
        call!(self.callbacks, release_render_paint, self.handle);
    }
}

impl RenderPaint for CallbackRenderPaint {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn style(&mut self, style: RenderPaintStyle) {
        let style = match style {
            RenderPaintStyle::Stroke => 0u8,
            RenderPaintStyle::Fill => 1u8,
        };
        call!(self.callbacks, render_paint_style, self.handle, style);
    }

    fn color(&mut self, value: ColorInt) {
        call!(self.callbacks, render_paint_color, self.handle, value);
    }

    fn thickness(&mut self, value: f32) {
        call!(self.callbacks, render_paint_thickness, self.handle, value);
    }

    fn join(&mut self, value: StrokeJoin) {
        call!(self.callbacks, render_paint_join, self.handle, value as u32);
    }

    fn cap(&mut self, value: StrokeCap) {
        call!(self.callbacks, render_paint_cap, self.handle, value as u32);
    }

    fn feather(&mut self, value: f32) {
        call!(self.callbacks, render_paint_feather, self.handle, value);
    }

    fn blend_mode(&mut self, value: BlendMode) {
        call!(
            self.callbacks,
            render_paint_blend_mode,
            self.handle,
            value as u8
        );
    }

    fn shader(&mut self, shader: Option<&dyn RenderShader>) {
        call!(
            self.callbacks,
            render_paint_shader,
            self.handle,
            render_shader_handle(shader)
        );
    }

    fn invalidate_stroke(&mut self) {
        call!(self.callbacks, render_paint_invalidate_stroke, self.handle);
    }
}

pub(crate) struct CallbackRenderShader {
    callbacks: NuxRenderCallbacks,
    handle: u64,
}

impl Drop for CallbackRenderShader {
    fn drop(&mut self) {
        call!(self.callbacks, release_render_shader, self.handle);
    }
}

impl RenderShader for CallbackRenderShader {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

pub(crate) struct CallbackRenderImage {
    callbacks: NuxRenderCallbacks,
    handle: u64,
    width: u32,
    height: u32,
}

impl Drop for CallbackRenderImage {
    fn drop(&mut self) {
        call!(self.callbacks, release_render_image, self.handle);
    }
}

impl RenderImage for CallbackRenderImage {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }
}

pub(crate) struct CallbackRenderBuffer {
    callbacks: NuxRenderCallbacks,
    handle: u64,
    buffer_type: RenderBufferType,
    flags: RenderBufferFlags,
    bytes: Vec<u8>,
}

impl Drop for CallbackRenderBuffer {
    fn drop(&mut self) {
        call!(self.callbacks, release_render_buffer, self.handle);
    }
}

impl RenderBuffer for CallbackRenderBuffer {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn buffer_type(&self) -> RenderBufferType {
        self.buffer_type
    }

    fn flags(&self) -> RenderBufferFlags {
        self.flags
    }

    fn size_in_bytes(&self) -> usize {
        self.bytes.len()
    }

    fn map_mut(&mut self) -> &mut [u8] {
        &mut self.bytes
    }

    fn unmap(&mut self) {
        call!(
            self.callbacks,
            render_buffer_unmap,
            self.handle,
            self.bytes.as_ptr(),
            self.bytes.len()
        );
    }
}

/// [`Factory`] implementation forwarding object creation to the C vtable.
pub(crate) struct CallbackFactory {
    callbacks: NuxRenderCallbacks,
}

impl CallbackFactory {
    pub(crate) fn new(callbacks: NuxRenderCallbacks) -> Self {
        Self { callbacks }
    }
}

impl Factory for CallbackFactory {
    fn make_render_buffer(
        &mut self,
        buffer_type: RenderBufferType,
        flags: RenderBufferFlags,
        size_in_bytes: usize,
    ) -> Box<dyn RenderBuffer> {
        let handle = call_handle!(
            self.callbacks,
            make_render_buffer,
            buffer_type as u8,
            flags as u8,
            size_in_bytes
        );
        Box::new(CallbackRenderBuffer {
            callbacks: self.callbacks,
            handle,
            buffer_type,
            flags,
            bytes: vec![0; size_in_bytes],
        })
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
        let count = colors.len().min(stops.len());
        let handle = call_handle!(
            self.callbacks,
            make_linear_gradient,
            sx,
            sy,
            ex,
            ey,
            colors.as_ptr(),
            stops.as_ptr(),
            count
        );
        Box::new(CallbackRenderShader {
            callbacks: self.callbacks,
            handle,
        })
    }

    fn make_radial_gradient(
        &mut self,
        cx: f32,
        cy: f32,
        radius: f32,
        colors: &[ColorInt],
        stops: &[f32],
    ) -> Box<dyn RenderShader> {
        let count = colors.len().min(stops.len());
        let handle = call_handle!(
            self.callbacks,
            make_radial_gradient,
            cx,
            cy,
            radius,
            colors.as_ptr(),
            stops.as_ptr(),
            count
        );
        Box::new(CallbackRenderShader {
            callbacks: self.callbacks,
            handle,
        })
    }

    fn make_render_path(&mut self, raw_path: RawPath, fill_rule: FillRule) -> Box<dyn RenderPath> {
        let handle = with_raw_path_view(&raw_path, |view| {
            call_handle!(self.callbacks, make_render_path, view, fill_rule as u8)
        });
        Box::new(CallbackRenderPath {
            callbacks: self.callbacks,
            handle,
        })
    }

    fn make_empty_render_path(&mut self) -> Box<dyn RenderPath> {
        let handle = call_handle!(self.callbacks, make_empty_render_path);
        Box::new(CallbackRenderPath {
            callbacks: self.callbacks,
            handle,
        })
    }

    fn make_render_paint(&mut self) -> Box<dyn RenderPaint> {
        let handle = call_handle!(self.callbacks, make_render_paint);
        Box::new(CallbackRenderPaint {
            callbacks: self.callbacks,
            handle,
        })
    }

    fn decode_image(&mut self, data: &[u8]) -> Box<dyn RenderImage> {
        let mut width = 0u32;
        let mut height = 0u32;
        let handle = call_handle!(
            self.callbacks,
            decode_image,
            data.as_ptr(),
            data.len(),
            &mut width,
            &mut height
        );
        Box::new(CallbackRenderImage {
            callbacks: self.callbacks,
            handle,
            width,
            height,
        })
    }
}

/// [`Renderer`] implementation forwarding draw calls to the C vtable.
pub(crate) struct CallbackRenderer {
    callbacks: NuxRenderCallbacks,
}

impl CallbackRenderer {
    pub(crate) fn new(callbacks: NuxRenderCallbacks) -> Self {
        Self { callbacks }
    }
}

impl Renderer for CallbackRenderer {
    fn save(&mut self) {
        call!(self.callbacks, save);
    }

    fn restore(&mut self) {
        call!(self.callbacks, restore);
    }

    fn transform(&mut self, transform: Mat2D) {
        call!(self.callbacks, transform, transform.0.as_ptr());
    }

    fn draw_path(&mut self, path: &dyn RenderPath, paint: &dyn RenderPaint) {
        call!(
            self.callbacks,
            draw_path,
            render_path_handle(path),
            render_paint_handle(paint)
        );
    }

    fn clip_path(&mut self, path: &dyn RenderPath) {
        call!(self.callbacks, clip_path, render_path_handle(path));
    }

    fn draw_image(
        &mut self,
        image: Option<&dyn RenderImage>,
        sampler: ImageSampler,
        blend_mode: BlendMode,
        opacity: f32,
    ) {
        call!(
            self.callbacks,
            draw_image,
            render_image_handle(image),
            NuxImageSampler::from(sampler),
            blend_mode as u8,
            opacity
        );
    }

    #[allow(clippy::too_many_arguments)]
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
        call!(
            self.callbacks,
            draw_image_mesh,
            render_image_handle(image),
            NuxImageSampler::from(sampler),
            render_buffer_handle(vertices),
            render_buffer_handle(uv_coords),
            render_buffer_handle(indices),
            vertex_count,
            index_count,
            blend_mode as u8,
            opacity
        );
    }

    fn modulate_opacity(&mut self, opacity: f32) {
        call!(self.callbacks, modulate_opacity, opacity);
    }
}
