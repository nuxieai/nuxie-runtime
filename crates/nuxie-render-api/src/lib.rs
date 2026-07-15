// Coarsely translated from:
// /Users/levi/dev/oss/rive-runtime/include/rive/renderer.hpp
// /Users/levi/dev/oss/rive-runtime/include/rive/factory.hpp
// /Users/levi/dev/rive-rust/tools/golden-runner/recording_renderer.cpp
use std::any::Any;
use std::cell::RefCell;
use std::fmt::Write;
use std::os::raw::{c_char, c_double, c_int};
use std::rc::Rc;

pub type ColorInt = u32;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec2D {
    pub x: f32,
    pub y: f32,
}

impl Vec2D {
    pub const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

/// Axis-aligned bounds in the same coordinate space as the queried geometry.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Aabb {
    pub min_x: f32,
    pub min_y: f32,
    pub max_x: f32,
    pub max_y: f32,
}

impl Aabb {
    pub const fn new(min_x: f32, min_y: f32, max_x: f32, max_y: f32) -> Self {
        Self {
            min_x,
            min_y,
            max_x,
            max_y,
        }
    }

    pub fn width(self) -> f32 {
        self.max_x - self.min_x
    }

    pub fn height(self) -> f32 {
        self.max_y - self.min_y
    }

    /// Inclusive containment, including points on the maximum edges.
    pub fn contains(self, point: Vec2D) -> bool {
        point.x >= self.min_x
            && point.x <= self.max_x
            && point.y >= self.min_y
            && point.y <= self.max_y
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Mat2D(pub [f32; 6]);

impl Mat2D {
    pub const IDENTITY: Self = Self([1.0, 0.0, 0.0, 1.0, 0.0, 0.0]);

    pub fn transform_point(self, point: Vec2D) -> Vec2D {
        let [xx, yx, xy, yy, tx, ty] = self.0;
        Vec2D {
            x: xx * point.x + xy * point.y + tx,
            y: yx * point.x + yy * point.y + ty,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FillRule {
    NonZero = 0,
    EvenOdd = 1,
    Clockwise = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PathVerb {
    Move = 0,
    Line = 1,
    Quad = 2,
    Cubic = 4,
    Close = 5,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RawPath {
    verbs: Vec<PathVerb>,
    points: Vec<Vec2D>,
}

impl RawPath {
    pub fn new() -> Self {
        Self {
            verbs: Vec::new(),
            points: Vec::new(),
        }
    }

    pub fn verbs(&self) -> &[PathVerb] {
        &self.verbs
    }

    pub fn points(&self) -> &[Vec2D] {
        &self.points
    }

    pub fn rewind(&mut self) {
        self.verbs.clear();
        self.points.clear();
    }

    pub fn reserve(&mut self, verbs: usize, points: usize) {
        self.verbs.reserve(verbs);
        self.points.reserve(points);
    }

    pub fn move_to(&mut self, x: f32, y: f32) {
        self.verbs.push(PathVerb::Move);
        self.points.push(Vec2D::new(x, y));
    }

    pub fn line_to(&mut self, x: f32, y: f32) {
        self.inject_implicit_move_if_needed();
        self.verbs.push(PathVerb::Line);
        self.points.push(Vec2D::new(x, y));
    }

    pub fn quad_to(&mut self, ox: f32, oy: f32, x: f32, y: f32) {
        self.inject_implicit_move_if_needed();
        self.verbs.push(PathVerb::Quad);
        self.points.push(Vec2D::new(ox, oy));
        self.points.push(Vec2D::new(x, y));
    }

    pub fn cubic_to(&mut self, ox: f32, oy: f32, ix: f32, iy: f32, x: f32, y: f32) {
        self.inject_implicit_move_if_needed();
        self.verbs.push(PathVerb::Cubic);
        self.points.push(Vec2D::new(ox, oy));
        self.points.push(Vec2D::new(ix, iy));
        self.points.push(Vec2D::new(x, y));
    }

    pub fn close(&mut self) {
        if !self.verbs.is_empty() && self.verbs.last() != Some(&PathVerb::Close) {
            self.verbs.push(PathVerb::Close);
        }
    }

    pub fn add_path(&mut self, path: &RawPath, transform: Mat2D) {
        self.verbs.extend_from_slice(&path.verbs);
        self.points.extend(
            path.points
                .iter()
                .copied()
                .map(|point| transform.transform_point(point)),
        );
    }

    pub fn add_path_backwards(&mut self, path: &RawPath, transform: Mat2D) {
        if path.verbs.is_empty() {
            return;
        }

        let initial_verb_count = self.verbs.len();
        let initial_point_count = self.points.len();
        self.points.reserve(path.points.len());
        self.points.extend(
            path.points
                .iter()
                .rev()
                .copied()
                .map(|point| transform.transform_point(point)),
        );

        // Reverse the verbs while moving each close from the end of its
        // original contour to the end of the reversed contour.
        self.verbs.reserve(path.verbs.len());
        self.verbs.push(PathVerb::Move);
        let mut closed = false;
        for (index, verb) in path.verbs.iter().enumerate().rev() {
            if *verb == PathVerb::Close {
                debug_assert!(!closed, "a contour may contain only one close verb");
                closed = true;
                continue;
            }

            if *verb == PathVerb::Move && closed {
                self.verbs.push(PathVerb::Close);
                closed = false;
            }

            if index == 0 {
                debug_assert_eq!(*verb, PathVerb::Move);
                break;
            }

            self.verbs.push(*verb);
        }
        debug_assert!(!closed, "every close verb must have a preceding move verb");

        self.prune_empty_segments_from(initial_verb_count, initial_point_count);
    }

    fn prune_empty_segments_from(&mut self, verb_start: usize, point_start: usize) {
        let mut source_point = point_start;
        let mut destination_verb = verb_start;
        let mut destination_point = point_start;

        for source_verb in verb_start..self.verbs.len() {
            let verb = self.verbs[source_verb];
            let point_count = match verb {
                PathVerb::Move | PathVerb::Line => 1,
                PathVerb::Quad => 2,
                PathVerb::Cubic => 3,
                PathVerb::Close => 0,
            };
            let has_geometry = match verb {
                PathVerb::Move | PathVerb::Close => true,
                PathVerb::Line => self.points[source_point] != self.points[source_point - 1],
                PathVerb::Quad => {
                    self.points[source_point + 1] != self.points[source_point]
                        || self.points[source_point] != self.points[source_point - 1]
                }
                PathVerb::Cubic => {
                    self.points[source_point + 2] != self.points[source_point + 1]
                        || self.points[source_point + 1] != self.points[source_point]
                        || self.points[source_point] != self.points[source_point - 1]
                }
            };

            if has_geometry {
                if source_verb != destination_verb {
                    self.verbs[destination_verb] = verb;
                    for point in 0..point_count {
                        self.points[destination_point + point] = self.points[source_point + point];
                    }
                }
                destination_verb += 1;
                destination_point += point_count;
            }
            source_point += point_count;
        }

        self.verbs.truncate(destination_verb);
        self.points.truncate(destination_point);
    }

    fn inject_implicit_move_if_needed(&mut self) {
        if !self.verbs.is_empty() && self.verbs.last() != Some(&PathVerb::Close) {
            return;
        }

        let mut point_index = 0;
        let mut last_move = Vec2D::new(0.0, 0.0);
        for verb in &self.verbs {
            match verb {
                PathVerb::Move => {
                    last_move = self.points[point_index];
                    point_index += 1;
                }
                PathVerb::Line => point_index += 1,
                PathVerb::Quad => point_index += 2,
                PathVerb::Cubic => point_index += 3,
                PathVerb::Close => {}
            }
        }
        self.move_to(last_move.x, last_move.y);
    }
}

impl Default for RawPath {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum BlendMode {
    SrcOver = 3,
    Screen = 14,
    Overlay = 15,
    Darken = 16,
    Lighten = 17,
    ColorDodge = 18,
    ColorBurn = 19,
    HardLight = 20,
    SoftLight = 21,
    Difference = 22,
    Exclusion = 23,
    Multiply = 24,
    Hue = 25,
    Saturation = 26,
    Color = 27,
    Luminosity = 28,
}

impl Default for BlendMode {
    fn default() -> Self {
        Self::SrcOver
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum StrokeJoin {
    Miter = 0,
    Round = 1,
    Bevel = 2,
}

impl Default for StrokeJoin {
    fn default() -> Self {
        Self::Miter
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum StrokeCap {
    Butt = 0,
    Round = 1,
    Square = 2,
}

impl Default for StrokeCap {
    fn default() -> Self {
        Self::Butt
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderPaintStyle {
    Stroke,
    Fill,
}

impl Default for RenderPaintStyle {
    fn default() -> Self {
        Self::Fill
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ImageFilter {
    Bilinear = 0,
    Nearest = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ImageWrap {
    Clamp = 0,
    Repeat = 1,
    Mirror = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ImageSampler {
    pub wrap_x: ImageWrap,
    pub wrap_y: ImageWrap,
    pub filter: ImageFilter,
}

impl ImageSampler {
    pub const LINEAR_CLAMP: Self = Self {
        wrap_x: ImageWrap::Clamp,
        wrap_y: ImageWrap::Clamp,
        filter: ImageFilter::Bilinear,
    };

    pub fn as_key(self) -> u8 {
        self.wrap_x as u8 + (self.wrap_y as u8 * 3) + (self.filter as u8 * 3 * 3)
    }
}

impl Default for ImageSampler {
    fn default() -> Self {
        Self::LINEAR_CLAMP
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum RenderBufferType {
    Index = 0,
    Vertex = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum RenderBufferFlags {
    None = 0,
    MappedOnceAtInitialization = 1,
}

pub trait RenderBuffer: Any {
    fn as_any(&self) -> &dyn Any;
    fn buffer_type(&self) -> RenderBufferType;
    fn flags(&self) -> RenderBufferFlags;
    fn size_in_bytes(&self) -> usize;
    fn map_mut(&mut self) -> &mut [u8];
    fn unmap(&mut self);
}

pub trait RenderShader: Any {
    fn as_any(&self) -> &dyn Any;
}

pub trait RenderImage: Any {
    fn as_any(&self) -> &dyn Any;
    fn width(&self) -> u32;
    fn height(&self) -> u32;
    fn uv_transform(&self) -> Mat2D {
        Mat2D::IDENTITY
    }
}

pub trait RenderPaint: Any {
    fn as_any(&self) -> &dyn Any;
    fn style(&mut self, style: RenderPaintStyle);
    fn color(&mut self, value: ColorInt);
    fn thickness(&mut self, value: f32);
    fn join(&mut self, value: StrokeJoin);
    fn cap(&mut self, value: StrokeCap);
    fn feather(&mut self, value: f32);
    fn blend_mode(&mut self, value: BlendMode);
    fn shader(&mut self, shader: Option<&dyn RenderShader>);
    fn invalidate_stroke(&mut self);
}

pub trait RenderPath: Any {
    fn as_any(&self) -> &dyn Any;
    fn rewind(&mut self);
    fn reserve(&mut self, _verbs: usize, _points: usize) {}
    fn fill_rule(&mut self, value: FillRule);
    fn add_render_path(&mut self, path: &dyn RenderPath, transform: Mat2D);
    fn add_render_path_backwards(&mut self, path: &dyn RenderPath, transform: Mat2D);
    fn add_raw_path(&mut self, path: &RawPath);
    fn move_to(&mut self, x: f32, y: f32);
    fn line_to(&mut self, x: f32, y: f32);
    fn cubic_to(&mut self, ox: f32, oy: f32, ix: f32, iy: f32, x: f32, y: f32);
    fn close(&mut self);
}

pub trait Renderer {
    fn save(&mut self);
    fn restore(&mut self);
    fn transform(&mut self, transform: Mat2D);
    fn draw_path(&mut self, path: &dyn RenderPath, paint: &dyn RenderPaint);
    fn clip_path(&mut self, path: &dyn RenderPath);
    fn draw_image(
        &mut self,
        image: Option<&dyn RenderImage>,
        sampler: ImageSampler,
        blend_mode: BlendMode,
        opacity: f32,
    );
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
    );
    fn modulate_opacity(&mut self, opacity: f32);
}

pub trait Factory {
    fn make_render_buffer(
        &mut self,
        buffer_type: RenderBufferType,
        flags: RenderBufferFlags,
        size_in_bytes: usize,
    ) -> Box<dyn RenderBuffer>;
    fn make_linear_gradient(
        &mut self,
        sx: f32,
        sy: f32,
        ex: f32,
        ey: f32,
        colors: &[ColorInt],
        stops: &[f32],
    ) -> Box<dyn RenderShader>;
    fn make_radial_gradient(
        &mut self,
        cx: f32,
        cy: f32,
        radius: f32,
        colors: &[ColorInt],
        stops: &[f32],
    ) -> Box<dyn RenderShader>;
    fn make_render_path(&mut self, raw_path: RawPath, fill_rule: FillRule) -> Box<dyn RenderPath>;
    fn make_empty_render_path(&mut self) -> Box<dyn RenderPath>;
    fn make_render_paint(&mut self) -> Box<dyn RenderPaint>;
    fn decode_image(&mut self, data: &[u8]) -> Box<dyn RenderImage>;
}

#[derive(Debug, Default)]
struct RecordingStream {
    lines: String,
}

impl RecordingStream {
    fn line(&mut self, value: impl AsRef<str>) {
        self.lines.push_str(value.as_ref());
        self.lines.push('\n');
    }

    fn line_with(&mut self, write_line: impl FnOnce(&mut String)) {
        write_line(&mut self.lines);
        self.lines.push('\n');
    }

    fn clear(&mut self) {
        self.lines.clear();
    }
}

pub struct RecordingRenderer {
    stream: Rc<RefCell<RecordingStream>>,
}

impl RecordingRenderer {
    fn new(stream: Rc<RefCell<RecordingStream>>) -> Self {
        Self { stream }
    }
}

pub struct RecordingFactory {
    stream: Rc<RefCell<RecordingStream>>,
    next_image_id: u64,
    next_paint_id: u64,
    next_path_id: u64,
    next_buffer_id: u64,
    next_shader_id: u64,
}

pub struct NullRenderer;

impl NullRenderer {
    pub fn new() -> Self {
        Self
    }
}

impl Default for NullRenderer {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Default)]
pub struct NullFactory;

impl NullFactory {
    pub fn new() -> Self {
        Self
    }

    pub fn make_renderer(&self) -> NullRenderer {
        NullRenderer::new()
    }
}

impl RecordingFactory {
    pub fn new() -> Self {
        let stream = Rc::new(RefCell::new(RecordingStream::default()));
        stream.borrow_mut().line("rive-golden-stream-v1");
        Self {
            stream,
            next_image_id: 1,
            next_paint_id: 1,
            next_path_id: 1,
            next_buffer_id: 1,
            next_shader_id: 1,
        }
    }

    pub fn make_renderer(&self) -> RecordingRenderer {
        RecordingRenderer::new(Rc::clone(&self.stream))
    }

    pub fn source(&mut self, file: &str, artboard: &str, scene: &str) {
        self.stream.borrow_mut().line(format!(
            "source file={} artboard={} scene={}",
            quoted_string(file),
            quoted_string(artboard),
            quoted_string(scene)
        ));
    }

    pub fn add_sample(&mut self, seconds: f32) {
        self.stream
            .borrow_mut()
            .line(format!("sample seconds={}", float_to_string(seconds)));
    }

    pub fn add_input_event(&mut self, kind: &str, seconds: f32, x: f32, y: f32, pointer_id: i32) {
        self.stream.borrow_mut().line(format!(
            "input kind={kind} seconds={} position=({},{}) pointerId={pointer_id}",
            float_to_string(seconds),
            float_to_string(x),
            float_to_string(y)
        ));
    }

    pub fn add_frame(&mut self) {
        self.stream.borrow_mut().line("frame");
    }

    pub fn frame_size(&mut self, width: u32, height: u32) {
        self.stream
            .borrow_mut()
            .line(format!("frameSize width={width} height={height}"));
    }

    pub fn clear_color(&mut self, color: ColorInt) {
        self.stream
            .borrow_mut()
            .line(format!("clearColor value=0x{color:08x}"));
    }

    pub fn stream(&self) -> String {
        self.stream.borrow().lines.clone()
    }

    pub fn clear(&mut self) {
        let mut stream = self.stream.borrow_mut();
        stream.clear();
        stream.line("rive-golden-stream-v1");
    }
}

impl Default for RecordingFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl Factory for RecordingFactory {
    fn make_render_buffer(
        &mut self,
        buffer_type: RenderBufferType,
        flags: RenderBufferFlags,
        size_in_bytes: usize,
    ) -> Box<dyn RenderBuffer> {
        let id = self.next_buffer_id;
        self.next_buffer_id += 1;
        self.stream.borrow_mut().line(format!(
            "makeRenderBuffer id={id} type={} flags={} size={size_in_bytes}",
            buffer_type as u8, flags as u8
        ));
        Box::new(RecordingRenderBuffer {
            stream: Rc::clone(&self.stream),
            id,
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
        let id = self.next_shader_id;
        self.next_shader_id += 1;
        let mut line = format!(
            "makeLinearGradient id={id} start=({},{}) end=({},{}) stops=[",
            float_to_string(sx),
            float_to_string(sy),
            float_to_string(ex),
            float_to_string(ey)
        );
        write_stops(&mut line, colors, stops);
        line.push(']');
        self.stream.borrow_mut().line(line);
        Box::new(RecordingRenderShader { id })
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
        let id = self.next_shader_id;
        self.next_shader_id += 1;
        let mut line = format!(
            "makeRadialGradient id={id} center=({},{}) radius={} stops=[",
            float_to_string(cx),
            float_to_string(cy),
            float_to_string(radius)
        );
        write_stops(&mut line, colors, stops);
        line.push(']');
        self.stream.borrow_mut().line(line);
        Box::new(RecordingRenderShader { id })
    }

    fn make_render_path(&mut self, raw_path: RawPath, fill_rule: FillRule) -> Box<dyn RenderPath> {
        let id = self.next_path_id;
        self.next_path_id += 1;
        let path = RecordingRenderPath {
            id,
            raw_path,
            fill_rule,
        };
        self.stream.borrow_mut().line_with(|line| {
            line.push_str("makeRenderPath ");
            path.write_snapshot(line);
        });
        Box::new(path)
    }

    fn make_empty_render_path(&mut self) -> Box<dyn RenderPath> {
        let id = self.next_path_id;
        self.next_path_id += 1;
        let path = RecordingRenderPath {
            id,
            raw_path: RawPath::new(),
            fill_rule: FillRule::NonZero,
        };
        self.stream.borrow_mut().line_with(|line| {
            line.push_str("makeEmptyRenderPath ");
            path.write_snapshot(line);
        });
        Box::new(path)
    }

    fn make_render_paint(&mut self) -> Box<dyn RenderPaint> {
        let id = self.next_paint_id;
        self.next_paint_id += 1;
        let paint = RecordingRenderPaint {
            id,
            style: RenderPaintStyle::Fill,
            color: 0xff000000,
            thickness: 1.0,
            join: StrokeJoin::Miter,
            cap: StrokeCap::Butt,
            feather: 0.0,
            blend_mode: BlendMode::SrcOver,
            shader_id: 0,
        };
        self.stream.borrow_mut().line_with(|line| {
            line.push_str("makeRenderPaint ");
            paint.write_snapshot(line);
        });
        Box::new(paint)
    }

    fn decode_image(&mut self, data: &[u8]) -> Box<dyn RenderImage> {
        let id = self.next_image_id;
        self.next_image_id += 1;
        let (width, height) = encoded_image_dimensions(data);
        self.stream.borrow_mut().line(format!(
            "decodeImage id={id} width={width} height={height} data={}",
            hex_bytes(data)
        ));
        Box::new(RecordingRenderImage { id, width, height })
    }
}

impl Factory for NullFactory {
    fn make_render_buffer(
        &mut self,
        buffer_type: RenderBufferType,
        flags: RenderBufferFlags,
        size_in_bytes: usize,
    ) -> Box<dyn RenderBuffer> {
        Box::new(NullRenderBuffer {
            buffer_type,
            flags,
            bytes: vec![0; size_in_bytes],
        })
    }

    fn make_linear_gradient(
        &mut self,
        _sx: f32,
        _sy: f32,
        _ex: f32,
        _ey: f32,
        _colors: &[ColorInt],
        _stops: &[f32],
    ) -> Box<dyn RenderShader> {
        Box::new(NullRenderShader)
    }

    fn make_radial_gradient(
        &mut self,
        _cx: f32,
        _cy: f32,
        _radius: f32,
        _colors: &[ColorInt],
        _stops: &[f32],
    ) -> Box<dyn RenderShader> {
        Box::new(NullRenderShader)
    }

    fn make_render_path(&mut self, raw_path: RawPath, fill_rule: FillRule) -> Box<dyn RenderPath> {
        Box::new(NullRenderPath {
            raw_path,
            fill_rule,
        })
    }

    fn make_empty_render_path(&mut self) -> Box<dyn RenderPath> {
        Box::new(NullRenderPath {
            raw_path: RawPath::new(),
            fill_rule: FillRule::NonZero,
        })
    }

    fn make_render_paint(&mut self) -> Box<dyn RenderPaint> {
        Box::new(NullRenderPaint {
            style: RenderPaintStyle::Fill,
            color: 0xff000000,
            thickness: 1.0,
            join: StrokeJoin::Miter,
            cap: StrokeCap::Butt,
            feather: 0.0,
            blend_mode: BlendMode::SrcOver,
        })
    }

    fn decode_image(&mut self, data: &[u8]) -> Box<dyn RenderImage> {
        let (width, height) = encoded_image_dimensions(data);
        Box::new(NullRenderImage { width, height })
    }
}

struct NullRenderShader;

impl RenderShader for NullRenderShader {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

struct NullRenderImage {
    width: u32,
    height: u32,
}

impl RenderImage for NullRenderImage {
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

struct NullRenderPaint {
    style: RenderPaintStyle,
    color: ColorInt,
    thickness: f32,
    join: StrokeJoin,
    cap: StrokeCap,
    feather: f32,
    blend_mode: BlendMode,
}

impl RenderPaint for NullRenderPaint {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn style(&mut self, style: RenderPaintStyle) {
        self.style = style;
    }

    fn color(&mut self, value: ColorInt) {
        self.color = value;
    }

    fn thickness(&mut self, value: f32) {
        self.thickness = value;
    }

    fn join(&mut self, value: StrokeJoin) {
        self.join = value;
    }

    fn cap(&mut self, value: StrokeCap) {
        self.cap = value;
    }

    fn feather(&mut self, value: f32) {
        self.feather = value;
    }

    fn blend_mode(&mut self, value: BlendMode) {
        self.blend_mode = value;
    }

    fn shader(&mut self, _shader: Option<&dyn RenderShader>) {}

    fn invalidate_stroke(&mut self) {}
}

struct NullRenderPath {
    raw_path: RawPath,
    fill_rule: FillRule,
}

impl RenderPath for NullRenderPath {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn rewind(&mut self) {
        self.raw_path.rewind();
    }

    fn reserve(&mut self, verbs: usize, points: usize) {
        self.raw_path.reserve(verbs, points);
    }

    fn fill_rule(&mut self, value: FillRule) {
        self.fill_rule = value;
    }

    fn add_render_path(&mut self, path: &dyn RenderPath, transform: Mat2D) {
        let path = null_path(path);
        self.raw_path.add_path(&path.raw_path, transform);
    }

    fn add_render_path_backwards(&mut self, path: &dyn RenderPath, transform: Mat2D) {
        let path = null_path(path);
        self.raw_path.add_path_backwards(&path.raw_path, transform);
    }

    fn add_raw_path(&mut self, path: &RawPath) {
        self.raw_path.add_path(path, Mat2D::IDENTITY);
    }

    fn move_to(&mut self, x: f32, y: f32) {
        self.raw_path.move_to(x, y);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.raw_path.line_to(x, y);
    }

    fn cubic_to(&mut self, ox: f32, oy: f32, ix: f32, iy: f32, x: f32, y: f32) {
        self.raw_path.cubic_to(ox, oy, ix, iy, x, y);
    }

    fn close(&mut self) {
        self.raw_path.close();
    }
}

struct NullRenderBuffer {
    buffer_type: RenderBufferType,
    flags: RenderBufferFlags,
    bytes: Vec<u8>,
}

impl RenderBuffer for NullRenderBuffer {
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

    fn unmap(&mut self) {}
}

impl Renderer for NullRenderer {
    fn save(&mut self) {}

    fn restore(&mut self) {}

    fn transform(&mut self, _transform: Mat2D) {}

    fn draw_path(&mut self, _path: &dyn RenderPath, _paint: &dyn RenderPaint) {}

    fn clip_path(&mut self, _path: &dyn RenderPath) {}

    fn draw_image(
        &mut self,
        _image: Option<&dyn RenderImage>,
        _sampler: ImageSampler,
        _blend_mode: BlendMode,
        _opacity: f32,
    ) {
    }

    fn draw_image_mesh(
        &mut self,
        _image: Option<&dyn RenderImage>,
        _sampler: ImageSampler,
        _vertices: Option<&dyn RenderBuffer>,
        _uv_coords: Option<&dyn RenderBuffer>,
        _indices: Option<&dyn RenderBuffer>,
        _vertex_count: u32,
        _index_count: u32,
        _blend_mode: BlendMode,
        _opacity: f32,
    ) {
    }

    fn modulate_opacity(&mut self, _opacity: f32) {}
}

fn null_path(path: &dyn RenderPath) -> &NullRenderPath {
    path.as_any()
        .downcast_ref::<NullRenderPath>()
        .expect("NullFactory requires NullRenderPath")
}

struct RecordingRenderShader {
    id: u64,
}

impl RenderShader for RecordingRenderShader {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

struct RecordingRenderImage {
    id: u64,
    width: u32,
    height: u32,
}

impl RenderImage for RecordingRenderImage {
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

struct RecordingRenderPaint {
    id: u64,
    style: RenderPaintStyle,
    color: ColorInt,
    thickness: f32,
    join: StrokeJoin,
    cap: StrokeCap,
    feather: f32,
    blend_mode: BlendMode,
    shader_id: u64,
}

impl RecordingRenderPaint {
    fn write_snapshot(&self, out: &mut String) {
        write!(out, "{{id={},style=", self.id).expect("writing to a String cannot fail");
        out.push_str(match self.style {
            RenderPaintStyle::Stroke => "stroke",
            RenderPaintStyle::Fill => "fill",
        });
        out.push_str(",color=");
        write_color(out, self.color);
        out.push_str(",thickness=");
        write_float(out, self.thickness);
        write!(
            out,
            ",join={},cap={},feather=",
            self.join as u32, self.cap as u32
        )
        .expect("writing to a String cannot fail");
        write_float(out, self.feather);
        write!(
            out,
            ",blendMode={},shader={}}}",
            self.blend_mode as u8, self.shader_id
        )
        .expect("writing to a String cannot fail");
    }
}

impl RenderPaint for RecordingRenderPaint {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn style(&mut self, style: RenderPaintStyle) {
        self.style = style;
    }

    fn color(&mut self, value: ColorInt) {
        self.color = value;
    }

    fn thickness(&mut self, value: f32) {
        self.thickness = value;
    }

    fn join(&mut self, value: StrokeJoin) {
        self.join = value;
    }

    fn cap(&mut self, value: StrokeCap) {
        self.cap = value;
    }

    fn feather(&mut self, value: f32) {
        self.feather = value;
    }

    fn blend_mode(&mut self, value: BlendMode) {
        self.blend_mode = value;
    }

    fn shader(&mut self, shader: Option<&dyn RenderShader>) {
        self.shader_id = shader
            .and_then(|shader| shader.as_any().downcast_ref::<RecordingRenderShader>())
            .map(|shader| shader.id)
            .unwrap_or(0);
    }

    fn invalidate_stroke(&mut self) {}
}

struct RecordingRenderPath {
    id: u64,
    raw_path: RawPath,
    fill_rule: FillRule,
}

impl RecordingRenderPath {
    fn write_snapshot(&self, out: &mut String) {
        write!(
            out,
            "{{id={},fillRule={},path=",
            self.id, self.fill_rule as u8
        )
        .expect("writing to a String cannot fail");
        write_raw_path(out, &self.raw_path);
        out.push('}');
    }
}

impl RenderPath for RecordingRenderPath {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn rewind(&mut self) {
        self.raw_path.rewind();
    }

    fn reserve(&mut self, verbs: usize, points: usize) {
        self.raw_path.reserve(verbs, points);
    }

    fn fill_rule(&mut self, value: FillRule) {
        self.fill_rule = value;
    }

    fn add_render_path(&mut self, path: &dyn RenderPath, transform: Mat2D) {
        let path = recording_path(path);
        self.raw_path.add_path(&path.raw_path, transform);
    }

    fn add_render_path_backwards(&mut self, path: &dyn RenderPath, transform: Mat2D) {
        let path = recording_path(path);
        self.raw_path.add_path_backwards(&path.raw_path, transform);
    }

    fn add_raw_path(&mut self, path: &RawPath) {
        self.raw_path.add_path(path, Mat2D::IDENTITY);
    }

    fn move_to(&mut self, x: f32, y: f32) {
        self.raw_path.move_to(x, y);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.raw_path.line_to(x, y);
    }

    fn cubic_to(&mut self, ox: f32, oy: f32, ix: f32, iy: f32, x: f32, y: f32) {
        self.raw_path.cubic_to(ox, oy, ix, iy, x, y);
    }

    fn close(&mut self) {
        self.raw_path.close();
    }
}

struct RecordingRenderBuffer {
    stream: Rc<RefCell<RecordingStream>>,
    id: u64,
    buffer_type: RenderBufferType,
    flags: RenderBufferFlags,
    bytes: Vec<u8>,
}

impl RenderBuffer for RecordingRenderBuffer {
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
        self.stream.borrow_mut().line(format!(
            "bufferData id={} type={} size={} data={}",
            self.id,
            self.buffer_type as u8,
            self.bytes.len(),
            hex_bytes(&self.bytes)
        ));
    }
}

impl Renderer for RecordingRenderer {
    fn save(&mut self) {
        self.stream.borrow_mut().line("save");
    }

    fn restore(&mut self) {
        self.stream.borrow_mut().line("restore");
    }

    fn transform(&mut self, transform: Mat2D) {
        self.stream
            .borrow_mut()
            .line(format!("transform matrix={}", mat_to_string(transform)));
    }

    fn draw_path(&mut self, path: &dyn RenderPath, paint: &dyn RenderPaint) {
        let path = recording_path(path);
        let paint = recording_paint(paint);
        self.stream.borrow_mut().line_with(|line| {
            line.push_str("drawPath path=");
            path.write_snapshot(line);
            line.push_str(" paint=");
            paint.write_snapshot(line);
        });
    }

    fn clip_path(&mut self, path: &dyn RenderPath) {
        let path = recording_path(path);
        self.stream.borrow_mut().line_with(|line| {
            line.push_str("clipPath path=");
            path.write_snapshot(line);
        });
    }

    fn draw_image(
        &mut self,
        image: Option<&dyn RenderImage>,
        sampler: ImageSampler,
        blend_mode: BlendMode,
        opacity: f32,
    ) {
        self.stream.borrow_mut().line(format!(
            "drawImage image={} sampler={} blendMode={} opacity={}",
            image_id(image),
            sampler_to_string(sampler),
            blend_mode as u8,
            float_to_string(opacity)
        ));
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
        self.stream.borrow_mut().line(format!(
            "drawImageMesh image={} sampler={} vertices={} uvs={} indices={} vertexCount={} indexCount={} blendMode={} opacity={}",
            image_id(image),
            sampler_to_string(sampler),
            buffer_id(vertices),
            buffer_id(uv_coords),
            buffer_id(indices),
            vertex_count,
            index_count,
            blend_mode as u8,
            float_to_string(opacity)
        ));
    }

    fn modulate_opacity(&mut self, opacity: f32) {
        self.stream.borrow_mut().line(format!(
            "modulateOpacity opacity={}",
            float_to_string(opacity)
        ));
    }
}

fn recording_path(path: &dyn RenderPath) -> &RecordingRenderPath {
    path.as_any()
        .downcast_ref::<RecordingRenderPath>()
        .expect("RecordingRenderer requires RecordingRenderPath")
}

fn recording_paint(paint: &dyn RenderPaint) -> &RecordingRenderPaint {
    paint
        .as_any()
        .downcast_ref::<RecordingRenderPaint>()
        .expect("RecordingRenderer requires RecordingRenderPaint")
}

fn image_id(image: Option<&dyn RenderImage>) -> u64 {
    image
        .and_then(|image| image.as_any().downcast_ref::<RecordingRenderImage>())
        .map(|image| image.id)
        .unwrap_or(0)
}

fn buffer_id(buffer: Option<&dyn RenderBuffer>) -> u64 {
    buffer
        .and_then(|buffer| buffer.as_any().downcast_ref::<RecordingRenderBuffer>())
        .map(|buffer| buffer.id)
        .unwrap_or(0)
}

fn write_stops(out: &mut String, colors: &[ColorInt], stops: &[f32]) {
    for (index, (color, stop)) in colors.iter().zip(stops).enumerate() {
        if index != 0 {
            out.push(',');
        }
        out.push_str("{color=");
        write_color(out, *color);
        out.push_str(",stop=");
        write_float(out, *stop);
        out.push('}');
    }
}

fn write_raw_path(out: &mut String, path: &RawPath) {
    out.push_str("{verbs=[");
    for (index, verb) in path.verbs().iter().enumerate() {
        if index != 0 {
            out.push(',');
        }
        out.push_str(match verb {
            PathVerb::Move => "move",
            PathVerb::Line => "line",
            PathVerb::Quad => "quad",
            PathVerb::Cubic => "cubic",
            PathVerb::Close => "close",
        });
    }
    out.push_str("],points=[");
    for (index, point) in path.points().iter().enumerate() {
        if index != 0 {
            out.push(',');
        }
        out.push('(');
        write_float(out, point.x);
        out.push(',');
        write_float(out, point.y);
        out.push(')');
    }
    out.push_str("]}");
}

fn sampler_to_string(sampler: ImageSampler) -> String {
    format!(
        "{{wrapX={},wrapY={},filter={},key={}}}",
        sampler.wrap_x as u8,
        sampler.wrap_y as u8,
        sampler.filter as u8,
        sampler.as_key()
    )
}

fn mat_to_string(mat: Mat2D) -> String {
    let mut out = String::from("[");
    for (index, value) in mat.0.into_iter().enumerate() {
        if index != 0 {
            out.push(',');
        }
        write_float(&mut out, value);
    }
    out.push(']');
    out
}

fn write_color(out: &mut String, color: ColorInt) {
    write!(out, "0x{color:08x}").expect("writing to a String cannot fail");
}

fn quoted_string(value: &str) -> String {
    let mut out = String::from("\"");
    for ch in value.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(ch),
        }
    }
    out.push('"');
    out
}

fn hex_bytes(bytes: &[u8]) -> String {
    let mut out = String::new();
    for byte in bytes {
        write!(out, "{byte:02x}").expect("writing to a String cannot fail");
    }
    out
}

fn encoded_image_dimensions(bytes: &[u8]) -> (u32, u32) {
    png_dimensions(bytes)
        .or_else(|| jpeg_dimensions(bytes))
        .or_else(|| webp_dimensions(bytes))
        .unwrap_or((0, 0))
}

fn png_dimensions(bytes: &[u8]) -> Option<(u32, u32)> {
    const PNG_SIGNATURE: &[u8; 8] = b"\x89PNG\r\n\x1a\n";
    if bytes.len() < 24 || &bytes[..8] != PNG_SIGNATURE || &bytes[12..16] != b"IHDR" {
        return None;
    }
    Some((read_be_u32(bytes, 16)?, read_be_u32(bytes, 20)?))
}

fn jpeg_dimensions(bytes: &[u8]) -> Option<(u32, u32)> {
    if bytes.len() < 4 || bytes[0] != 0xff || bytes[1] != 0xd8 {
        return None;
    }

    let mut offset = 2usize;
    while offset + 4 <= bytes.len() {
        while offset < bytes.len() && bytes[offset] != 0xff {
            offset += 1;
        }
        while offset < bytes.len() && bytes[offset] == 0xff {
            offset += 1;
        }
        let marker = *bytes.get(offset)?;
        offset += 1;
        if marker == 0xd9 || marker == 0xda {
            break;
        }
        if (0xd0..=0xd7).contains(&marker) {
            continue;
        }

        let segment_length = usize::from(read_be_u16(bytes, offset)?);
        if segment_length < 2 || offset + segment_length > bytes.len() {
            break;
        }
        if jpeg_start_of_frame(marker) && segment_length >= 7 {
            let height = u32::from(read_be_u16(bytes, offset + 3)?);
            let width = u32::from(read_be_u16(bytes, offset + 5)?);
            return Some((width, height));
        }
        offset += segment_length;
    }

    None
}

fn jpeg_start_of_frame(marker: u8) -> bool {
    (0xc0..=0xc3).contains(&marker)
        || (0xc5..=0xc7).contains(&marker)
        || (0xc9..=0xcb).contains(&marker)
        || (0xcd..=0xcf).contains(&marker)
}

fn webp_dimensions(bytes: &[u8]) -> Option<(u32, u32)> {
    if bytes.len() < 20 || &bytes[..4] != b"RIFF" || &bytes[8..12] != b"WEBP" {
        return None;
    }

    let mut offset = 12usize;
    while offset + 8 <= bytes.len() {
        let chunk_data = offset + 8;
        let chunk_size = usize::try_from(read_le_u32(bytes, offset + 4)?).ok()?;
        if chunk_data + chunk_size > bytes.len() {
            break;
        }

        if &bytes[offset..offset + 4] == b"VP8X" && chunk_size >= 10 {
            return Some((
                read_le_u24(bytes, chunk_data + 4)? + 1,
                read_le_u24(bytes, chunk_data + 7)? + 1,
            ));
        }
        if &bytes[offset..offset + 4] == b"VP8L" && chunk_size >= 5 && bytes[chunk_data] == 0x2f {
            let width = 1
                + u32::from(bytes[chunk_data + 1])
                + (u32::from(bytes[chunk_data + 2] & 0x3f) << 8);
            let height = 1
                + u32::from(bytes[chunk_data + 2] >> 6)
                + (u32::from(bytes[chunk_data + 3]) << 2)
                + (u32::from(bytes[chunk_data + 4] & 0x0f) << 10);
            return Some((width, height));
        }
        if &bytes[offset..offset + 4] == b"VP8 "
            && chunk_size >= 10
            && &bytes[chunk_data + 3..chunk_data + 6] == b"\x9d\x01\x2a"
        {
            return Some((
                u32::from(read_le_u16(bytes, chunk_data + 6)? & 0x3fff),
                u32::from(read_le_u16(bytes, chunk_data + 8)? & 0x3fff),
            ));
        }

        offset = chunk_data + chunk_size + (chunk_size & 1);
    }

    None
}

fn read_be_u32(bytes: &[u8], offset: usize) -> Option<u32> {
    Some(u32::from_be_bytes(
        bytes.get(offset..offset + 4)?.try_into().ok()?,
    ))
}

fn read_be_u16(bytes: &[u8], offset: usize) -> Option<u16> {
    Some(u16::from_be_bytes(
        bytes.get(offset..offset + 2)?.try_into().ok()?,
    ))
}

fn read_le_u32(bytes: &[u8], offset: usize) -> Option<u32> {
    Some(u32::from_le_bytes(
        bytes.get(offset..offset + 4)?.try_into().ok()?,
    ))
}

fn read_le_u24(bytes: &[u8], offset: usize) -> Option<u32> {
    let bytes = bytes.get(offset..offset + 3)?;
    Some(u32::from(bytes[0]) | (u32::from(bytes[1]) << 8) | (u32::from(bytes[2]) << 16))
}

fn read_le_u16(bytes: &[u8], offset: usize) -> Option<u16> {
    Some(u16::from_le_bytes(
        bytes.get(offset..offset + 2)?.try_into().ok()?,
    ))
}

unsafe extern "C" {
    fn snprintf(buffer: *mut c_char, size: usize, format: *const c_char, ...) -> c_int;
}

fn float_to_string(value: f32) -> String {
    let mut out = String::new();
    write_float(&mut out, value);
    out
}

fn write_float(out: &mut String, value: f32) {
    let mut buffer = [0 as c_char; 64];
    let format = b"%.9g\0";
    // C++ RecordingRenderer uses iostream defaultfloat with float max_digits10.
    // C's %.9g produces the same significant-digit spelling for finite f32s.
    let written = unsafe {
        snprintf(
            buffer.as_mut_ptr(),
            buffer.len(),
            format.as_ptr().cast(),
            value as c_double,
        )
    };
    assert!(written >= 0 && (written as usize) < buffer.len());
    let bytes =
        unsafe { std::slice::from_raw_parts(buffer.as_ptr().cast::<u8>(), written as usize) };
    let formatted = std::str::from_utf8(bytes).expect("snprintf emitted UTF-8 float digits");
    out.push_str(formatted);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_backwards_round_trip(path: &RawPath) {
        let mut backwards = RawPath::new();
        backwards.add_path_backwards(path, Mat2D::IDENTITY);

        let mut restored = RawPath::new();
        restored.add_path_backwards(&backwards, Mat2D::IDENTITY);

        assert_eq!(&restored, path);
    }

    #[test]
    fn raw_path_mutators_normalize_contours_for_backwards_reversal() {
        let mut leading_line = RawPath::new();
        leading_line.line_to(1.0, 2.0);
        assert_eq!(leading_line.verbs(), &[PathVerb::Move, PathVerb::Line]);
        assert_eq!(
            leading_line.points(),
            &[Vec2D::new(0.0, 0.0), Vec2D::new(1.0, 2.0)]
        );
        assert_backwards_round_trip(&leading_line);

        let mut leading_quad = RawPath::new();
        leading_quad.quad_to(1.0, 2.0, 3.0, 4.0);
        assert_eq!(leading_quad.verbs(), &[PathVerb::Move, PathVerb::Quad]);
        assert_backwards_round_trip(&leading_quad);

        let mut path = RawPath::new();
        path.cubic_to(1.0, 2.0, 3.0, 4.0, 5.0, 6.0);
        path.close();
        path.close();
        path.cubic_to(7.0, 8.0, 9.0, 10.0, 11.0, 12.0);
        assert_eq!(
            path.verbs(),
            &[
                PathVerb::Move,
                PathVerb::Cubic,
                PathVerb::Close,
                PathVerb::Move,
                PathVerb::Cubic,
            ]
        );
        assert_eq!(
            path.points(),
            &[
                Vec2D::new(0.0, 0.0),
                Vec2D::new(1.0, 2.0),
                Vec2D::new(3.0, 4.0),
                Vec2D::new(5.0, 6.0),
                Vec2D::new(0.0, 0.0),
                Vec2D::new(7.0, 8.0),
                Vec2D::new(9.0, 10.0),
                Vec2D::new(11.0, 12.0),
            ]
        );
        assert_backwards_round_trip(&path);
    }

    #[test]
    fn add_path_backwards_handles_empty_close_only_move_only_and_empty_contours() {
        let empty = RawPath::new();
        assert_backwards_round_trip(&empty);

        let mut close_only = RawPath::new();
        close_only.close();
        assert_eq!(close_only, empty);
        assert_backwards_round_trip(&close_only);

        let mut move_only = RawPath::new();
        move_only.move_to(1.0, 2.0);
        assert_backwards_round_trip(&move_only);

        let mut empty_contours = RawPath::new();
        empty_contours.move_to(1.0, 2.0);
        empty_contours.move_to(3.0, 4.0);
        let mut reversed = RawPath::new();
        reversed.add_path_backwards(&empty_contours, Mat2D::IDENTITY);
        assert_eq!(reversed.verbs(), &[PathVerb::Move, PathVerb::Move]);
        assert_eq!(
            reversed.points(),
            &[Vec2D::new(3.0, 4.0), Vec2D::new(1.0, 2.0)]
        );
        assert_backwards_round_trip(&empty_contours);
    }

    #[test]
    fn add_path_backwards_reverses_open_line_quad_and_cubic_segments() {
        let mut source = RawPath::new();
        source.move_to(1.0, 2.0);
        source.line_to(3.0, 4.0);
        source.quad_to(5.0, 6.0, 7.0, 8.0);
        source.cubic_to(9.0, 10.0, 11.0, 12.0, 13.0, 14.0);

        let mut reversed = RawPath::new();
        reversed.add_path_backwards(&source, Mat2D::IDENTITY);

        assert_eq!(
            reversed.verbs(),
            &[
                PathVerb::Move,
                PathVerb::Cubic,
                PathVerb::Quad,
                PathVerb::Line
            ]
        );
        assert_eq!(
            reversed.points(),
            &[
                Vec2D::new(13.0, 14.0),
                Vec2D::new(11.0, 12.0),
                Vec2D::new(9.0, 10.0),
                Vec2D::new(7.0, 8.0),
                Vec2D::new(5.0, 6.0),
                Vec2D::new(3.0, 4.0),
                Vec2D::new(1.0, 2.0),
            ]
        );
    }

    #[test]
    fn add_path_backwards_reverses_contour_order_and_preserves_closes() {
        let mut source = RawPath::new();
        source.move_to(0.0, 0.0);
        source.line_to(1.0, 0.0);
        source.quad_to(2.0, 0.0, 3.0, 0.0);
        source.close();
        source.move_to(10.0, 0.0);
        source.cubic_to(11.0, 0.0, 12.0, 0.0, 13.0, 0.0);
        source.move_to(20.0, 0.0);
        source.line_to(21.0, 0.0);
        source.close();

        let mut reversed = RawPath::new();
        reversed.add_path_backwards(&source, Mat2D::IDENTITY);

        assert_eq!(
            reversed.verbs(),
            &[
                PathVerb::Move,
                PathVerb::Line,
                PathVerb::Close,
                PathVerb::Move,
                PathVerb::Cubic,
                PathVerb::Move,
                PathVerb::Quad,
                PathVerb::Line,
                PathVerb::Close,
            ]
        );
        assert_eq!(
            reversed.points(),
            &[
                Vec2D::new(21.0, 0.0),
                Vec2D::new(20.0, 0.0),
                Vec2D::new(13.0, 0.0),
                Vec2D::new(12.0, 0.0),
                Vec2D::new(11.0, 0.0),
                Vec2D::new(10.0, 0.0),
                Vec2D::new(3.0, 0.0),
                Vec2D::new(2.0, 0.0),
                Vec2D::new(1.0, 0.0),
                Vec2D::new(0.0, 0.0),
            ]
        );
    }

    #[test]
    fn add_path_backwards_transforms_only_the_appended_reversed_path() {
        let mut source = RawPath::new();
        source.move_to(1.0, 2.0);
        source.line_to(3.0, 4.0);

        let mut destination = RawPath::new();
        destination.move_to(-1.0, -2.0);
        destination.add_path_backwards(&source, Mat2D([2.0, 0.0, 0.0, 3.0, 5.0, 7.0]));

        assert_eq!(
            destination.verbs(),
            &[PathVerb::Move, PathVerb::Move, PathVerb::Line]
        );
        assert_eq!(
            destination.points(),
            &[
                Vec2D::new(-1.0, -2.0),
                Vec2D::new(11.0, 19.0),
                Vec2D::new(7.0, 13.0),
            ]
        );
    }

    #[test]
    fn add_path_backwards_prunes_segments_collapsed_by_transform() {
        let mut source = RawPath::new();
        source.move_to(1.0, 2.0);
        source.line_to(3.0, 4.0);

        let mut reversed = RawPath::new();
        reversed.add_path_backwards(&source, Mat2D([0.0, 0.0, 0.0, 0.0, 5.0, 7.0]));

        assert_eq!(reversed.verbs(), &[PathVerb::Move]);
        assert_eq!(reversed.points(), &[Vec2D::new(5.0, 7.0)]);
    }

    #[test]
    fn add_path_backwards_keeps_transformed_curves_with_distinct_controls() {
        let mut source = RawPath::new();
        source.move_to(0.0, 0.0);
        source.quad_to(1.0, 2.0, 0.0, 0.0);
        source.cubic_to(3.0, 4.0, 5.0, 6.0, 0.0, 0.0);

        let mut reversed = RawPath::new();
        reversed.add_path_backwards(&source, Mat2D([2.0, 0.0, 0.0, 3.0, 5.0, 7.0]));

        assert_eq!(
            reversed.verbs(),
            &[PathVerb::Move, PathVerb::Cubic, PathVerb::Quad]
        );
        assert_eq!(
            reversed.points(),
            &[
                Vec2D::new(5.0, 7.0),
                Vec2D::new(15.0, 25.0),
                Vec2D::new(11.0, 19.0),
                Vec2D::new(5.0, 7.0),
                Vec2D::new(7.0, 13.0),
                Vec2D::new(5.0, 7.0),
            ]
        );
    }

    #[test]
    fn add_path_backwards_prunes_fully_collapsed_transformed_curves() {
        let mut source = RawPath::new();
        source.move_to(1.0, 2.0);
        source.quad_to(3.0, 4.0, 5.0, 6.0);
        source.cubic_to(7.0, 8.0, 9.0, 10.0, 11.0, 12.0);

        let mut reversed = RawPath::new();
        reversed.add_path_backwards(&source, Mat2D([0.0, 0.0, 0.0, 0.0, 5.0, 7.0]));

        assert_eq!(reversed.verbs(), &[PathVerb::Move]);
        assert_eq!(reversed.points(), &[Vec2D::new(5.0, 7.0)]);
    }

    #[test]
    fn recording_serializer_matches_cpp_smoke_stream() {
        let mut factory = RecordingFactory::new();
        let mut renderer = factory.make_renderer();
        let mut path = factory.make_empty_render_path();
        let mut paint = factory.make_render_paint();

        path.move_to(0.0, 0.0);
        path.line_to(10.0, 0.0);
        path.line_to(10.0, 10.0);
        path.close();
        paint.color(0xff336699);

        factory.source("smoke", "", "manual");
        factory.frame_size(64, 64);
        factory.add_sample(0.0);
        renderer.save();
        renderer.draw_path(path.as_ref(), paint.as_ref());
        renderer.restore();
        factory.add_frame();

        assert_eq!(
            factory.stream(),
            concat!(
                "rive-golden-stream-v1\n",
                "makeEmptyRenderPath {id=1,fillRule=0,path={verbs=[],points=[]}}\n",
                "makeRenderPaint {id=1,style=fill,color=0xff000000,thickness=1,join=0,cap=0,feather=0,blendMode=3,shader=0}\n",
                "source file=\"smoke\" artboard=\"\" scene=\"manual\"\n",
                "frameSize width=64 height=64\n",
                "sample seconds=0\n",
                "save\n",
                "drawPath path={id=1,fillRule=0,path={verbs=[move,line,line,close],points=[(0,0),(10,0),(10,10)]}} paint={id=1,style=fill,color=0xff336699,thickness=1,join=0,cap=0,feather=0,blendMode=3,shader=0}\n",
                "restore\n",
                "frame\n",
            )
        );
    }

    #[test]
    fn records_buffers_gradients_images_and_meshes() {
        let mut factory = RecordingFactory::new();
        let mut renderer = factory.make_renderer();
        let shader = factory.make_linear_gradient(
            0.0,
            0.5,
            10.0,
            20.0,
            &[0xff000000, 0xffffffff],
            &[0.0, 1.0],
        );
        let mut paint = factory.make_render_paint();
        paint.shader(Some(shader.as_ref()));
        let image = factory.decode_image(&[1, 2, 3]);
        let mut vertices = factory.make_render_buffer(
            RenderBufferType::Vertex,
            RenderBufferFlags::MappedOnceAtInitialization,
            4,
        );
        vertices.map_mut().copy_from_slice(&[1, 2, 3, 4]);
        vertices.unmap();

        renderer.draw_image(
            Some(image.as_ref()),
            ImageSampler {
                wrap_x: ImageWrap::Repeat,
                wrap_y: ImageWrap::Mirror,
                filter: ImageFilter::Nearest,
            },
            BlendMode::Multiply,
            0.5,
        );
        renderer.draw_image_mesh(
            Some(image.as_ref()),
            ImageSampler::LINEAR_CLAMP,
            Some(vertices.as_ref()),
            None,
            None,
            2,
            3,
            BlendMode::SrcOver,
            1.0,
        );

        let stream = factory.stream();
        assert!(stream.contains(
            "makeLinearGradient id=1 start=(0,0.5) end=(10,20) stops=[{color=0xff000000,stop=0},{color=0xffffffff,stop=1}]\n"
        ));
        assert!(stream.contains("makeRenderPaint {id=1,style=fill,color=0xff000000,thickness=1,join=0,cap=0,feather=0,blendMode=3,shader=0}\n"));
        assert!(stream.contains("decodeImage id=1 width=0 height=0 data=010203\n"));
        assert!(stream.contains("makeRenderBuffer id=1 type=1 flags=1 size=4\n"));
        assert!(stream.contains("bufferData id=1 type=1 size=4 data=01020304\n"));
        assert!(stream.contains(
            "drawImage image=1 sampler={wrapX=1,wrapY=2,filter=1,key=16} blendMode=24 opacity=0.5\n"
        ));
        assert!(stream.contains(
            "drawImageMesh image=1 sampler={wrapX=0,wrapY=0,filter=0,key=0} vertices=1 uvs=0 indices=0 vertexCount=2 indexCount=3 blendMode=3 opacity=1\n"
        ));
    }

    #[test]
    fn c_style_float_formatter_matches_cpp_significant_digits() {
        assert_eq!(float_to_string(0.05000000074505806), "0.0500000007");
        assert_eq!(float_to_string(-0.0), "-0");
        assert_eq!(float_to_string(384.37109375), "384.371094");
    }
}
