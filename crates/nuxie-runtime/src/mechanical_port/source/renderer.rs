use crate::mechanical_port::source::{
    command_path::CommandPath,
    layout::{Alignment, Fit},
    math::{aabb::Aabb, mat2d::Mat2D, raw_path::RawPath},
    refcnt::{Rcp, RefCnt, RefCounted},
    shapes::paint::{
        blend_mode::BlendMode, color::ColorInt, image_sampler::ImageSampler, stroke_cap::StrokeCap,
        stroke_join::StrokeJoin,
    },
};

pub fn compute_alignment(
    fit: Fit,
    alignment: Alignment,
    frame: &Aabb,
    content: &Aabb,
    scale_factor: f32,
) -> Mat2D {
    let content_width = content.width();
    let content_height = content.height();
    let x = -content.left() - content_width * 0.5 - alignment.x() * content_width * 0.5;
    let y = -content.top() - content_height * 0.5 - alignment.y() * content_height * 0.5;

    let (scale_x, scale_y) = match fit {
        Fit::Fill => (
            frame.width() / content_width,
            frame.height() / content_height,
        ),
        Fit::Contain => {
            let scale = (frame.width() / content_width).min(frame.height() / content_height);
            (scale, scale)
        }
        Fit::Cover => {
            let scale = (frame.width() / content_width).max(frame.height() / content_height);
            (scale, scale)
        }
        Fit::FitHeight => {
            let scale = frame.height() / content_height;
            (scale, scale)
        }
        Fit::FitWidth => {
            let scale = frame.width() / content_width;
            (scale, scale)
        }
        Fit::Layout => (scale_factor, scale_factor),
        Fit::None => (1.0, 1.0),
        Fit::ScaleDown => {
            let scale = (frame.width() / content_width)
                .min(frame.height() / content_height)
                .min(1.0);
            (scale, scale)
        }
    };

    let translation = Mat2D::from_translate(
        frame.left() + frame.width() * 0.5 + alignment.x() * frame.width() * 0.5,
        frame.top() + frame.height() * 0.5 + alignment.y() * frame.height() * 0.5,
    );
    translation * Mat2D::from_scale(scale_x, scale_y) * Mat2D::from_translate(x, y)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RenderBufferType {
    Index,
    Vertex,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[repr(transparent)]
pub struct RenderBufferFlags(pub u8);

impl RenderBufferFlags {
    pub const NONE: Self = Self(0);
    pub const MAPPED_ONCE_AT_INITIALIZATION: Self = Self(1 << 0);

    pub fn contains(self, value: Self) -> bool {
        self.0 & value.0 == value.0
    }
}

pub struct RenderBuffer {
    ref_count: RefCnt,
    buffer_type: RenderBufferType,
    flags: RenderBufferFlags,
    size_in_bytes: usize,
    dirty: bool,
    map_count: usize,
    unmap_count: usize,
}

impl RenderBuffer {
    pub fn new(
        buffer_type: RenderBufferType,
        flags: RenderBufferFlags,
        size_in_bytes: usize,
    ) -> Self {
        Self {
            ref_count: RefCnt::new(),
            buffer_type,
            flags,
            size_in_bytes,
            dirty: false,
            map_count: 0,
            unmap_count: 0,
        }
    }

    pub fn buffer_type(&self) -> RenderBufferType {
        self.buffer_type
    }
    pub fn flags(&self) -> RenderBufferFlags {
        self.flags
    }
    pub fn size_in_bytes(&self) -> usize {
        self.size_in_bytes
    }

    pub fn map(&mut self) -> *mut u8 {
        assert!(
            self.map_count == 0
                || !self
                    .flags
                    .contains(RenderBufferFlags::MAPPED_ONCE_AT_INITIALIZATION)
        );
        assert_eq!(self.map_count, self.unmap_count);
        self.map_count += 1;
        self.dirty = true;
        self.on_map()
    }

    pub fn unmap(&mut self) {
        assert_eq!(self.unmap_count + 1, self.map_count);
        self.unmap_count += 1;
        self.on_unmap();
    }

    pub fn on_map(&mut self) -> *mut u8 {
        panic!("abstract RenderBuffer::on_map");
    }
    pub fn on_unmap(&mut self) {
        panic!("abstract RenderBuffer::on_unmap");
    }

    pub fn check_and_reset_dirty(&mut self) -> bool {
        assert_eq!(self.map_count, self.unmap_count);
        if self.dirty {
            self.dirty = false;
            return true;
        }
        false
    }
}

unsafe impl RefCounted for RenderBuffer {
    fn ref_count(&self) -> &RefCnt {
        &self.ref_count
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RenderPaintStyle {
    Stroke,
    Fill,
}

pub struct RenderShader {
    ref_count: RefCnt,
}

impl Default for RenderShader {
    fn default() -> Self {
        Self {
            ref_count: RefCnt::new(),
        }
    }
}

unsafe impl RefCounted for RenderShader {
    fn ref_count(&self) -> &RefCnt {
        &self.ref_count
    }
}

pub struct RenderPaint {
    ref_count: RefCnt,
}

impl Default for RenderPaint {
    fn default() -> Self {
        Self {
            ref_count: RefCnt::new(),
        }
    }
}

impl RenderPaint {
    pub fn style(&mut self, _style: RenderPaintStyle) {
        panic!("abstract RenderPaint::style");
    }
    pub fn color(&mut self, _value: ColorInt) {
        panic!("abstract RenderPaint::color");
    }
    pub fn thickness(&mut self, _value: f32) {
        panic!("abstract RenderPaint::thickness");
    }
    pub fn join(&mut self, _value: StrokeJoin) {
        panic!("abstract RenderPaint::join");
    }
    pub fn cap(&mut self, _value: StrokeCap) {
        panic!("abstract RenderPaint::cap");
    }
    pub fn feather(&mut self, _value: f32) {}
    pub fn blend_mode(&mut self, _value: BlendMode) {
        panic!("abstract RenderPaint::blend_mode");
    }
    pub fn shader(&mut self, _shader: Rcp<RenderShader>) {
        panic!("abstract RenderPaint::shader");
    }
    pub fn invalidate_stroke(&mut self) {
        panic!("abstract RenderPaint::invalidate_stroke");
    }
}

unsafe impl RefCounted for RenderPaint {
    fn ref_count(&self) -> &RefCnt {
        &self.ref_count
    }
}

pub struct RenderImage {
    ref_count: RefCnt,
    width: i32,
    height: i32,
    uv_transform: Mat2D,
    delegate: Option<*mut dyn RenderImageDelegate>,
}

impl Default for RenderImage {
    fn default() -> Self {
        Self::new(Mat2D::default())
    }
}

impl RenderImage {
    pub fn new(uv_transform: Mat2D) -> Self {
        Self {
            ref_count: RefCnt::new(),
            width: 0,
            height: 0,
            uv_transform,
            delegate: None,
        }
    }
    pub fn width(&self) -> i32 {
        self.width
    }
    pub fn height(&self) -> i32 {
        self.height
    }
    pub fn uv_transform(&self) -> &Mat2D {
        &self.uv_transform
    }
    pub fn set_delegate(&mut self, delegate: Option<*mut dyn RenderImageDelegate>) {
        self.delegate = delegate;
    }
    pub fn decoded_async(&self) {
        if let Some(delegate) = self.delegate {
            unsafe { &mut *delegate }.decoded_async();
        }
    }
}

unsafe impl RefCounted for RenderImage {
    fn ref_count(&self) -> &RefCnt {
        &self.ref_count
    }
}

pub trait RenderImageDelegate {
    fn decoded_async(&mut self);
}

pub struct RenderPath;

impl RenderPath {
    pub fn add_path_backwards(&mut self, path: &mut dyn CommandPath, transform: &Mat2D) {
        self.add_render_path(path.render_path(), transform);
    }

    pub fn add_render_path(&mut self, _path: &RenderPath, _transform: &Mat2D) {
        panic!("abstract RenderPath::add_render_path");
    }
    pub fn add_render_path_backwards(&mut self, _path: &RenderPath, _transform: &Mat2D) {}
    pub fn add_raw_path(&mut self, _path: &RawPath) {
        panic!("abstract RenderPath::add_raw_path");
    }
}

impl CommandPath for RenderPath {
    fn rewind(&mut self) {
        panic!("abstract RenderPath::rewind");
    }

    fn set_fill_rule(
        &mut self,
        _value: crate::mechanical_port::source::math::path_types::FillRule,
    ) {
        panic!("abstract RenderPath::set_fill_rule");
    }

    fn add_path(&mut self, path: &mut dyn CommandPath, transform: &Mat2D) {
        self.add_render_path(path.render_path(), transform);
    }

    fn move_to(&mut self, _x: f32, _y: f32) {
        panic!("abstract RenderPath::move_to");
    }

    fn line_to(&mut self, _x: f32, _y: f32) {
        panic!("abstract RenderPath::line_to");
    }

    fn cubic_to(&mut self, _ox: f32, _oy: f32, _ix: f32, _iy: f32, _x: f32, _y: f32) {
        panic!("abstract RenderPath::cubic_to");
    }

    fn close(&mut self) {
        panic!("abstract RenderPath::close");
    }

    fn render_path_mut(&mut self) -> &mut RenderPath {
        self
    }

    fn render_path(&self) -> &RenderPath {
        self
    }
}

pub trait Renderer {
    fn save(&mut self);
    fn restore(&mut self);
    fn transform(&mut self, transform: &Mat2D);
    fn draw_path(&mut self, path: &mut RenderPath, paint: &mut RenderPaint);
    fn clip_path(&mut self, path: &mut RenderPath);
    fn draw_image(
        &mut self,
        image: &RenderImage,
        sampler: ImageSampler,
        blend_mode: BlendMode,
        opacity: f32,
    );
    #[allow(clippy::too_many_arguments)]
    fn draw_image_mesh(
        &mut self,
        image: &RenderImage,
        sampler: ImageSampler,
        vertices_f32: Rcp<RenderBuffer>,
        uv_coords_f32: Rcp<RenderBuffer>,
        indices_u16: Rcp<RenderBuffer>,
        vertex_count: u32,
        index_count: u32,
        blend_mode: BlendMode,
        opacity: f32,
    );
    fn modulate_opacity(&mut self, opacity: f32);

    fn translate(&mut self, x: f32, y: f32) {
        self.transform(&Mat2D::from_translate(x, y));
    }
    fn scale(&mut self, sx: f32, sy: f32) {
        self.transform(&Mat2D::from_scale(sx, sy));
    }
    fn rotate(&mut self, radians: f32) {
        let (sin, cos) = radians.sin_cos();
        self.transform(&Mat2D::new(cos, sin, -sin, cos, 0.0, 0.0));
    }
    fn align(
        &mut self,
        fit: Fit,
        alignment: Alignment,
        frame: &Aabb,
        content: &Aabb,
        scale_factor: f32,
    ) {
        self.transform(&compute_alignment(
            fit,
            alignment,
            frame,
            content,
            scale_factor,
        ));
    }
}

pub fn is_white_space(character: u32) -> bool {
    character <= u32::from(b' ') || character == 0x2028 || character == 0x200b
}
