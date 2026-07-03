use rive_render_api::{
    BlendMode, ColorInt, Factory, FillRule, ImageSampler, Mat2D, PathVerb, RawPath, RenderBuffer,
    RenderBufferFlags, RenderBufferType, RenderImage, RenderPaint, RenderPaintStyle, RenderPath,
    RenderShader, Renderer, StrokeCap, StrokeJoin, Vec2D,
};
use std::any::Any;
use std::error::Error;
use std::fmt;
use std::ptr::NonNull;
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NativeRendererError {
    CreateContext,
    BeginFrame,
    MissingRenderer,
}

impl fmt::Display for NativeRendererError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::CreateContext => write!(f, "failed to create native renderer context"),
            Self::BeginFrame => write!(f, "failed to begin native renderer frame"),
            Self::MissingRenderer => write!(f, "native renderer frame is not open"),
        }
    }
}

impl Error for NativeRendererError {}

pub struct FfiFactory {
    context: Rc<ContextHandle>,
    width: u32,
    height: u32,
}

impl FfiFactory {
    pub fn new_null(width: u32, height: u32) -> Result<Self, NativeRendererError> {
        let context = unsafe { ffi::rive_ffi_context_make_null(width, height) };
        let context = NonNull::new(context).ok_or(NativeRendererError::CreateContext)?;
        Ok(Self {
            context: Rc::new(ContextHandle(context)),
            width,
            height,
        })
    }

    pub fn begin_frame(&mut self, clear_color: ColorInt) -> Result<FfiFrame, NativeRendererError> {
        let ok = unsafe {
            ffi::rive_ffi_context_begin_frame(
                self.context.as_ptr(),
                self.width,
                self.height,
                clear_color,
            )
        };
        if ok == 0 {
            return Err(NativeRendererError::BeginFrame);
        }
        let renderer = unsafe { ffi::rive_ffi_context_renderer(self.context.as_ptr()) };
        let renderer = NonNull::new(renderer).ok_or(NativeRendererError::MissingRenderer)?;
        Ok(FfiFrame {
            context: Rc::clone(&self.context),
            renderer,
            ended: false,
        })
    }
}

struct ContextHandle(NonNull<ffi::Context>);

impl ContextHandle {
    fn as_ptr(&self) -> *mut ffi::Context {
        self.0.as_ptr()
    }
}

impl Drop for ContextHandle {
    fn drop(&mut self) {
        unsafe { ffi::rive_ffi_context_delete(self.as_ptr()) };
    }
}

impl Factory for FfiFactory {
    fn make_render_buffer(
        &mut self,
        buffer_type: RenderBufferType,
        flags: RenderBufferFlags,
        size_in_bytes: usize,
    ) -> Box<dyn RenderBuffer> {
        let handle = unsafe {
            ffi::rive_ffi_make_render_buffer(
                self.context.as_ptr(),
                buffer_type as u8,
                flags as u8,
                size_in_bytes,
            )
        };
        Box::new(FfiRenderBuffer {
            handle: non_null(handle, "rive_ffi_make_render_buffer"),
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
        assert_eq!(colors.len(), stops.len());
        let handle = unsafe {
            ffi::rive_ffi_make_linear_gradient(
                self.context.as_ptr(),
                sx,
                sy,
                ex,
                ey,
                colors.as_ptr(),
                stops.as_ptr(),
                colors.len(),
            )
        };
        Box::new(FfiRenderShader {
            handle: non_null(handle, "rive_ffi_make_linear_gradient"),
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
        assert_eq!(colors.len(), stops.len());
        let handle = unsafe {
            ffi::rive_ffi_make_radial_gradient(
                self.context.as_ptr(),
                cx,
                cy,
                radius,
                colors.as_ptr(),
                stops.as_ptr(),
                colors.len(),
            )
        };
        Box::new(FfiRenderShader {
            handle: non_null(handle, "rive_ffi_make_radial_gradient"),
        })
    }

    fn make_render_path(&mut self, raw_path: RawPath, fill_rule: FillRule) -> Box<dyn RenderPath> {
        let (verbs, points) = raw_path_parts(&raw_path);
        let handle = unsafe {
            ffi::rive_ffi_make_render_path(
                self.context.as_ptr(),
                verbs.as_ptr(),
                verbs.len(),
                points.as_ptr(),
                points.len(),
                fill_rule as u8,
            )
        };
        Box::new(FfiRenderPath {
            handle: non_null(handle, "rive_ffi_make_render_path"),
        })
    }

    fn make_empty_render_path(&mut self) -> Box<dyn RenderPath> {
        let handle = unsafe { ffi::rive_ffi_make_empty_render_path(self.context.as_ptr()) };
        Box::new(FfiRenderPath {
            handle: non_null(handle, "rive_ffi_make_empty_render_path"),
        })
    }

    fn make_render_paint(&mut self) -> Box<dyn RenderPaint> {
        let handle = unsafe { ffi::rive_ffi_make_render_paint(self.context.as_ptr()) };
        Box::new(FfiRenderPaint {
            handle: non_null(handle, "rive_ffi_make_render_paint"),
        })
    }

    fn decode_image(&mut self, data: &[u8]) -> Box<dyn RenderImage> {
        let handle =
            unsafe { ffi::rive_ffi_decode_image(self.context.as_ptr(), data.as_ptr(), data.len()) };
        Box::new(FfiRenderImage {
            handle: non_null(handle, "rive_ffi_decode_image"),
        })
    }
}

pub struct FfiFrame {
    context: Rc<ContextHandle>,
    renderer: NonNull<ffi::Renderer>,
    ended: bool,
}

impl FfiFrame {
    pub fn draw_count(&self) -> u64 {
        unsafe { ffi::rive_ffi_context_draw_count(self.context.as_ptr()) }
    }

    pub fn end(mut self) -> u64 {
        self.close()
    }

    fn close(&mut self) -> u64 {
        let count = self.draw_count();
        if !self.ended {
            self.ended = true;
            unsafe { ffi::rive_ffi_context_end_frame(self.context.as_ptr()) };
        }
        count
    }
}

impl Drop for FfiFrame {
    fn drop(&mut self) {
        self.close();
        unsafe { ffi::rive_ffi_renderer_delete(self.renderer.as_ptr()) };
    }
}

impl Renderer for FfiFrame {
    fn save(&mut self) {
        unsafe { ffi::rive_ffi_renderer_save(self.renderer.as_ptr()) };
    }

    fn restore(&mut self) {
        unsafe { ffi::rive_ffi_renderer_restore(self.renderer.as_ptr()) };
    }

    fn transform(&mut self, transform: Mat2D) {
        unsafe { ffi::rive_ffi_renderer_transform(self.renderer.as_ptr(), transform.into()) };
    }

    fn draw_path(&mut self, path: &dyn RenderPath, paint: &dyn RenderPaint) {
        unsafe {
            ffi::rive_ffi_renderer_draw_path(
                self.renderer.as_ptr(),
                ffi_path(path).handle.as_ptr(),
                ffi_paint(paint).handle.as_ptr(),
            )
        };
    }

    fn clip_path(&mut self, path: &dyn RenderPath) {
        unsafe {
            ffi::rive_ffi_renderer_clip_path(self.renderer.as_ptr(), ffi_path(path).handle.as_ptr())
        };
    }

    fn draw_image(
        &mut self,
        image: Option<&dyn RenderImage>,
        sampler: ImageSampler,
        blend_mode: BlendMode,
        opacity: f32,
    ) {
        unsafe {
            ffi::rive_ffi_renderer_draw_image(
                self.renderer.as_ptr(),
                image
                    .map(ffi_image)
                    .map(|image| image.handle.as_ptr())
                    .unwrap_or(std::ptr::null_mut()),
                sampler.as_key(),
                blend_mode as u8,
                opacity,
            )
        };
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
        unsafe {
            ffi::rive_ffi_renderer_draw_image_mesh(
                self.renderer.as_ptr(),
                image
                    .map(ffi_image)
                    .map(|image| image.handle.as_ptr())
                    .unwrap_or(std::ptr::null_mut()),
                sampler.as_key(),
                vertices
                    .map(ffi_buffer)
                    .map(|buffer| buffer.handle.as_ptr())
                    .unwrap_or(std::ptr::null_mut()),
                uv_coords
                    .map(ffi_buffer)
                    .map(|buffer| buffer.handle.as_ptr())
                    .unwrap_or(std::ptr::null_mut()),
                indices
                    .map(ffi_buffer)
                    .map(|buffer| buffer.handle.as_ptr())
                    .unwrap_or(std::ptr::null_mut()),
                vertex_count,
                index_count,
                blend_mode as u8,
                opacity,
            )
        };
    }

    fn modulate_opacity(&mut self, opacity: f32) {
        unsafe { ffi::rive_ffi_renderer_modulate_opacity(self.renderer.as_ptr(), opacity) };
    }
}

struct FfiRenderShader {
    handle: NonNull<ffi::RenderShader>,
}

impl Drop for FfiRenderShader {
    fn drop(&mut self) {
        unsafe { ffi::rive_ffi_render_shader_delete(self.handle.as_ptr()) };
    }
}

impl RenderShader for FfiRenderShader {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

struct FfiRenderImage {
    handle: NonNull<ffi::RenderImage>,
}

impl Drop for FfiRenderImage {
    fn drop(&mut self) {
        unsafe { ffi::rive_ffi_render_image_delete(self.handle.as_ptr()) };
    }
}

impl RenderImage for FfiRenderImage {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn width(&self) -> u32 {
        unsafe { ffi::rive_ffi_render_image_width(self.handle.as_ptr()) }
    }

    fn height(&self) -> u32 {
        unsafe { ffi::rive_ffi_render_image_height(self.handle.as_ptr()) }
    }
}

struct FfiRenderPaint {
    handle: NonNull<ffi::RenderPaint>,
}

impl Drop for FfiRenderPaint {
    fn drop(&mut self) {
        unsafe { ffi::rive_ffi_render_paint_delete(self.handle.as_ptr()) };
    }
}

impl RenderPaint for FfiRenderPaint {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn style(&mut self, style: RenderPaintStyle) {
        let style = match style {
            RenderPaintStyle::Stroke => 0,
            RenderPaintStyle::Fill => 1,
        };
        unsafe { ffi::rive_ffi_render_paint_style(self.handle.as_ptr(), style) };
    }

    fn color(&mut self, value: ColorInt) {
        unsafe { ffi::rive_ffi_render_paint_color(self.handle.as_ptr(), value) };
    }

    fn thickness(&mut self, value: f32) {
        unsafe { ffi::rive_ffi_render_paint_thickness(self.handle.as_ptr(), value) };
    }

    fn join(&mut self, value: StrokeJoin) {
        unsafe { ffi::rive_ffi_render_paint_join(self.handle.as_ptr(), value as u32) };
    }

    fn cap(&mut self, value: StrokeCap) {
        unsafe { ffi::rive_ffi_render_paint_cap(self.handle.as_ptr(), value as u32) };
    }

    fn feather(&mut self, value: f32) {
        unsafe { ffi::rive_ffi_render_paint_feather(self.handle.as_ptr(), value) };
    }

    fn blend_mode(&mut self, value: BlendMode) {
        unsafe { ffi::rive_ffi_render_paint_blend_mode(self.handle.as_ptr(), value as u8) };
    }

    fn shader(&mut self, shader: Option<&dyn RenderShader>) {
        let shader = shader
            .map(ffi_shader)
            .map(|shader| shader.handle.as_ptr())
            .unwrap_or(std::ptr::null_mut());
        unsafe { ffi::rive_ffi_render_paint_shader(self.handle.as_ptr(), shader) };
    }

    fn invalidate_stroke(&mut self) {
        unsafe { ffi::rive_ffi_render_paint_invalidate_stroke(self.handle.as_ptr()) };
    }
}

struct FfiRenderPath {
    handle: NonNull<ffi::RenderPath>,
}

impl Drop for FfiRenderPath {
    fn drop(&mut self) {
        unsafe { ffi::rive_ffi_render_path_delete(self.handle.as_ptr()) };
    }
}

impl RenderPath for FfiRenderPath {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn rewind(&mut self) {
        unsafe { ffi::rive_ffi_render_path_rewind(self.handle.as_ptr()) };
    }

    fn fill_rule(&mut self, value: FillRule) {
        unsafe { ffi::rive_ffi_render_path_fill_rule(self.handle.as_ptr(), value as u8) };
    }

    fn add_render_path(&mut self, path: &dyn RenderPath, transform: Mat2D) {
        unsafe {
            ffi::rive_ffi_render_path_add_render_path(
                self.handle.as_ptr(),
                ffi_path(path).handle.as_ptr(),
                transform.into(),
            )
        };
    }

    fn add_render_path_backwards(&mut self, path: &dyn RenderPath, transform: Mat2D) {
        unsafe {
            ffi::rive_ffi_render_path_add_render_path_backwards(
                self.handle.as_ptr(),
                ffi_path(path).handle.as_ptr(),
                transform.into(),
            )
        };
    }

    fn add_raw_path(&mut self, path: &RawPath) {
        let (verbs, points) = raw_path_parts(path);
        unsafe {
            ffi::rive_ffi_render_path_add_raw_path(
                self.handle.as_ptr(),
                verbs.as_ptr(),
                verbs.len(),
                points.as_ptr(),
                points.len(),
            )
        };
    }

    fn move_to(&mut self, x: f32, y: f32) {
        unsafe { ffi::rive_ffi_render_path_move_to(self.handle.as_ptr(), x, y) };
    }

    fn line_to(&mut self, x: f32, y: f32) {
        unsafe { ffi::rive_ffi_render_path_line_to(self.handle.as_ptr(), x, y) };
    }

    fn cubic_to(&mut self, ox: f32, oy: f32, ix: f32, iy: f32, x: f32, y: f32) {
        unsafe { ffi::rive_ffi_render_path_cubic_to(self.handle.as_ptr(), ox, oy, ix, iy, x, y) };
    }

    fn close(&mut self) {
        unsafe { ffi::rive_ffi_render_path_close(self.handle.as_ptr()) };
    }
}

struct FfiRenderBuffer {
    handle: NonNull<ffi::RenderBuffer>,
    buffer_type: RenderBufferType,
    flags: RenderBufferFlags,
    bytes: Vec<u8>,
}

impl Drop for FfiRenderBuffer {
    fn drop(&mut self) {
        unsafe { ffi::rive_ffi_render_buffer_delete(self.handle.as_ptr()) };
    }
}

impl RenderBuffer for FfiRenderBuffer {
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
        unsafe {
            ffi::rive_ffi_render_buffer_write(
                self.handle.as_ptr(),
                self.bytes.as_ptr(),
                self.bytes.len(),
            )
        };
    }
}

fn raw_path_parts(path: &RawPath) -> (Vec<u8>, Vec<FfiVec2D>) {
    (
        path.verbs().iter().map(|verb| *verb as u8).collect(),
        path.points().iter().copied().map(Into::into).collect(),
    )
}

fn non_null<T>(ptr: *mut T, function: &str) -> NonNull<T> {
    NonNull::new(ptr).unwrap_or_else(|| panic!("{function} returned null"))
}

fn ffi_shader(shader: &dyn RenderShader) -> &FfiRenderShader {
    shader
        .as_any()
        .downcast_ref::<FfiRenderShader>()
        .expect("rive-renderer-ffi requires FfiRenderShader")
}

fn ffi_image(image: &dyn RenderImage) -> &FfiRenderImage {
    image
        .as_any()
        .downcast_ref::<FfiRenderImage>()
        .expect("rive-renderer-ffi requires FfiRenderImage")
}

fn ffi_paint(paint: &dyn RenderPaint) -> &FfiRenderPaint {
    paint
        .as_any()
        .downcast_ref::<FfiRenderPaint>()
        .expect("rive-renderer-ffi requires FfiRenderPaint")
}

fn ffi_path(path: &dyn RenderPath) -> &FfiRenderPath {
    path.as_any()
        .downcast_ref::<FfiRenderPath>()
        .expect("rive-renderer-ffi requires FfiRenderPath")
}

fn ffi_buffer(buffer: &dyn RenderBuffer) -> &FfiRenderBuffer {
    buffer
        .as_any()
        .downcast_ref::<FfiRenderBuffer>()
        .expect("rive-renderer-ffi requires FfiRenderBuffer")
}

#[repr(C)]
#[derive(Clone, Copy)]
struct FfiVec2D {
    x: f32,
    y: f32,
}

impl From<Vec2D> for FfiVec2D {
    fn from(value: Vec2D) -> Self {
        Self {
            x: value.x,
            y: value.y,
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
struct FfiMat2D {
    values: [f32; 6],
}

impl From<Mat2D> for FfiMat2D {
    fn from(value: Mat2D) -> Self {
        Self { values: value.0 }
    }
}

mod ffi {
    use super::{FfiMat2D, FfiVec2D};
    use std::ffi::c_void;

    pub type Context = c_void;
    pub type Renderer = c_void;
    pub type RenderPath = c_void;
    pub type RenderPaint = c_void;
    pub type RenderShader = c_void;
    pub type RenderImage = c_void;
    pub type RenderBuffer = c_void;

    unsafe extern "C" {
        pub fn rive_ffi_context_make_null(width: u32, height: u32) -> *mut Context;
        pub fn rive_ffi_context_delete(context: *mut Context);
        pub fn rive_ffi_context_begin_frame(
            context: *mut Context,
            width: u32,
            height: u32,
            clear_color: u32,
        ) -> i32;
        pub fn rive_ffi_context_end_frame(context: *mut Context);
        pub fn rive_ffi_context_draw_count(context: *mut Context) -> u64;
        pub fn rive_ffi_context_renderer(context: *mut Context) -> *mut Renderer;
        pub fn rive_ffi_renderer_delete(renderer: *mut Renderer);

        pub fn rive_ffi_make_linear_gradient(
            context: *mut Context,
            sx: f32,
            sy: f32,
            ex: f32,
            ey: f32,
            colors: *const u32,
            stops: *const f32,
            count: usize,
        ) -> *mut RenderShader;
        pub fn rive_ffi_make_radial_gradient(
            context: *mut Context,
            cx: f32,
            cy: f32,
            radius: f32,
            colors: *const u32,
            stops: *const f32,
            count: usize,
        ) -> *mut RenderShader;
        pub fn rive_ffi_render_shader_delete(shader: *mut RenderShader);

        pub fn rive_ffi_make_render_path(
            context: *mut Context,
            verbs: *const u8,
            verb_count: usize,
            points: *const FfiVec2D,
            point_count: usize,
            fill_rule: u8,
        ) -> *mut RenderPath;
        pub fn rive_ffi_make_empty_render_path(context: *mut Context) -> *mut RenderPath;
        pub fn rive_ffi_render_path_delete(path: *mut RenderPath);
        pub fn rive_ffi_render_path_rewind(path: *mut RenderPath);
        pub fn rive_ffi_render_path_fill_rule(path: *mut RenderPath, fill_rule: u8);
        pub fn rive_ffi_render_path_add_render_path(
            path: *mut RenderPath,
            source: *mut RenderPath,
            transform: FfiMat2D,
        );
        pub fn rive_ffi_render_path_add_render_path_backwards(
            path: *mut RenderPath,
            source: *mut RenderPath,
            transform: FfiMat2D,
        );
        pub fn rive_ffi_render_path_add_raw_path(
            path: *mut RenderPath,
            verbs: *const u8,
            verb_count: usize,
            points: *const FfiVec2D,
            point_count: usize,
        );
        pub fn rive_ffi_render_path_move_to(path: *mut RenderPath, x: f32, y: f32);
        pub fn rive_ffi_render_path_line_to(path: *mut RenderPath, x: f32, y: f32);
        pub fn rive_ffi_render_path_cubic_to(
            path: *mut RenderPath,
            ox: f32,
            oy: f32,
            ix: f32,
            iy: f32,
            x: f32,
            y: f32,
        );
        pub fn rive_ffi_render_path_close(path: *mut RenderPath);

        pub fn rive_ffi_make_render_paint(context: *mut Context) -> *mut RenderPaint;
        pub fn rive_ffi_render_paint_delete(paint: *mut RenderPaint);
        pub fn rive_ffi_render_paint_style(paint: *mut RenderPaint, style: u8);
        pub fn rive_ffi_render_paint_color(paint: *mut RenderPaint, color: u32);
        pub fn rive_ffi_render_paint_thickness(paint: *mut RenderPaint, thickness: f32);
        pub fn rive_ffi_render_paint_join(paint: *mut RenderPaint, join: u32);
        pub fn rive_ffi_render_paint_cap(paint: *mut RenderPaint, cap: u32);
        pub fn rive_ffi_render_paint_feather(paint: *mut RenderPaint, feather: f32);
        pub fn rive_ffi_render_paint_blend_mode(paint: *mut RenderPaint, blend_mode: u8);
        pub fn rive_ffi_render_paint_shader(paint: *mut RenderPaint, shader: *mut RenderShader);
        pub fn rive_ffi_render_paint_invalidate_stroke(paint: *mut RenderPaint);

        pub fn rive_ffi_decode_image(
            context: *mut Context,
            bytes: *const u8,
            len: usize,
        ) -> *mut RenderImage;
        pub fn rive_ffi_render_image_delete(image: *mut RenderImage);
        pub fn rive_ffi_render_image_width(image: *mut RenderImage) -> u32;
        pub fn rive_ffi_render_image_height(image: *mut RenderImage) -> u32;

        pub fn rive_ffi_make_render_buffer(
            context: *mut Context,
            buffer_type: u8,
            flags: u8,
            size_in_bytes: usize,
        ) -> *mut RenderBuffer;
        pub fn rive_ffi_render_buffer_delete(buffer: *mut RenderBuffer);
        pub fn rive_ffi_render_buffer_write(
            buffer: *mut RenderBuffer,
            bytes: *const u8,
            len: usize,
        );

        pub fn rive_ffi_renderer_save(renderer: *mut Renderer);
        pub fn rive_ffi_renderer_restore(renderer: *mut Renderer);
        pub fn rive_ffi_renderer_transform(renderer: *mut Renderer, transform: FfiMat2D);
        pub fn rive_ffi_renderer_draw_path(
            renderer: *mut Renderer,
            path: *mut RenderPath,
            paint: *mut RenderPaint,
        );
        pub fn rive_ffi_renderer_clip_path(renderer: *mut Renderer, path: *mut RenderPath);
        pub fn rive_ffi_renderer_draw_image(
            renderer: *mut Renderer,
            image: *mut RenderImage,
            sampler: u8,
            blend_mode: u8,
            opacity: f32,
        );
        pub fn rive_ffi_renderer_draw_image_mesh(
            renderer: *mut Renderer,
            image: *mut RenderImage,
            sampler: u8,
            vertices: *mut RenderBuffer,
            uv_coords: *mut RenderBuffer,
            indices: *mut RenderBuffer,
            vertex_count: u32,
            index_count: u32,
            blend_mode: u8,
            opacity: f32,
        );
        pub fn rive_ffi_renderer_modulate_opacity(renderer: *mut Renderer, opacity: f32);
    }
}

#[allow(dead_code)]
fn _path_verb_values_are_cxx_compatible() {
    let _ = PathVerb::Move as u8;
}
