use super::{
    BlendMode, ColorInt, Factory, FillRule, ImageDecodeError, ImageSampler, Mat2D, PathVerb,
    RawPath, RenderBuffer, RenderBufferFlags, RenderBufferType, RenderImage, RenderPaint,
    RenderPaintStyle, RenderPath, RenderShader, Renderer, StrokeCap, StrokeJoin,
    encoded_image_dimensions,
};
use std::any::Any;
use std::cell::RefCell;
use std::rc::Rc;

const MAKE_RENDER_BUFFER: u64 = 0;
const MAKE_LINEAR_GRADIENT: u64 = 1;
const MAKE_RADIAL_GRADIENT: u64 = 2;
const MAKE_RENDER_PATH: u64 = 3;
const MAKE_RENDER_PAINT: u64 = 5;
const DECODE_IMAGE: u64 = 6;
const SAVE: u64 = 7;
const RESTORE: u64 = 8;
const TRANSFORM: u64 = 9;
const DRAW_PATH: u64 = 10;
const CLIP_PATH: u64 = 11;
const DRAW_IMAGE: u64 = 12;
const DRAW_IMAGE_MESH: u64 = 13;
const SET_VERTEX_BUFFER_DATA: u64 = 14;
const SET_INDEX_BUFFER_DATA: u64 = 15;
const ADD_RAW_PATH: u64 = 16;
const REWIND: u64 = 17;
const FILL_RULE: u64 = 18;
const STYLE: u64 = 20;
const COLOR: u64 = 21;
const THICKNESS: u64 = 22;
const JOIN: u64 = 23;
const CAP: u64 = 24;
const FEATHER: u64 = 25;
const BLEND_MODE: u64 = 26;
const SHADER: u64 = 27;
const FRAME: u64 = 28;
const FRAME_SIZE: u64 = 29;
const MODULATE_OPACITY: u64 = 30;

#[derive(Debug)]
struct Writer {
    bytes: Vec<u8>,
}

impl Writer {
    fn new() -> Self {
        Self {
            bytes: b"SRIV\x01".to_vec(),
        }
    }

    fn varuint(&mut self, mut value: u64) {
        loop {
            let mut byte = (value & 0x7f) as u8;
            value >>= 7;
            if value != 0 {
                byte |= 0x80;
            }
            self.bytes.push(byte);
            if value == 0 {
                break;
            }
        }
    }

    fn float(&mut self, value: f32) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    fn raw_path(&mut self, path: &RawPath) {
        self.varuint(path.verbs().len() as u64);
        for verb in path.verbs() {
            self.varuint(*verb as u64);
        }
        self.varuint(path.points().len() as u64);
        for point in path.points() {
            self.float(point.x);
            self.float(point.y);
        }
    }
}

pub struct SerializingFactory {
    writer: Rc<RefCell<Writer>>,
    next_image_id: u64,
    next_paint_id: u64,
    next_path_id: u64,
    next_buffer_id: u64,
    next_shader_id: u64,
}

impl SerializingFactory {
    pub fn new() -> Self {
        Self {
            writer: Rc::new(RefCell::new(Writer::new())),
            next_image_id: 0,
            next_paint_id: 0,
            next_path_id: 0,
            next_buffer_id: 0,
            next_shader_id: 0,
        }
    }

    pub fn make_renderer(&self) -> SerializingRenderer {
        SerializingRenderer {
            writer: Rc::clone(&self.writer),
        }
    }

    pub fn frame_size(&mut self, width: u32, height: u32) {
        let mut writer = self.writer.borrow_mut();
        writer.varuint(FRAME_SIZE);
        writer.varuint(u64::from(width));
        writer.varuint(u64::from(height));
    }

    pub fn add_frame(&mut self) {
        self.writer.borrow_mut().varuint(FRAME);
    }

    pub fn bytes(&self) -> std::cell::Ref<'_, [u8]> {
        std::cell::Ref::map(self.writer.borrow(), |writer| writer.bytes.as_slice())
    }
}

impl Default for SerializingFactory {
    fn default() -> Self {
        Self::new()
    }
}

impl Factory for SerializingFactory {
    fn make_render_buffer(
        &mut self,
        buffer_type: RenderBufferType,
        flags: RenderBufferFlags,
        size_in_bytes: usize,
    ) -> Box<dyn RenderBuffer> {
        let id = self.next_buffer_id;
        self.next_buffer_id += 1;
        {
            let mut writer = self.writer.borrow_mut();
            writer.varuint(MAKE_RENDER_BUFFER);
            writer.varuint(id);
            writer.varuint(size_in_bytes as u64);
            writer.varuint(buffer_type as u64);
            writer.varuint(flags as u64);
        }
        Box::new(SerializingRenderBuffer {
            writer: Rc::clone(&self.writer),
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
        let mut writer = self.writer.borrow_mut();
        writer.varuint(MAKE_LINEAR_GRADIENT);
        writer.varuint(id);
        write_stops(&mut writer, colors, stops);
        for value in [sx, sy, ex, ey] {
            writer.float(value);
        }
        Box::new(SerializingRenderShader { id })
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
        let mut writer = self.writer.borrow_mut();
        writer.varuint(MAKE_RADIAL_GRADIENT);
        writer.varuint(id);
        write_stops(&mut writer, colors, stops);
        for value in [cx, cy, radius] {
            writer.float(value);
        }
        Box::new(SerializingRenderShader { id })
    }

    fn make_render_path(&mut self, raw_path: RawPath, fill_rule: FillRule) -> Box<dyn RenderPath> {
        let id = self.next_path_id;
        self.next_path_id += 1;
        {
            let mut writer = self.writer.borrow_mut();
            writer.varuint(MAKE_RENDER_PATH);
            writer.varuint(id);
            writer.varuint(ADD_RAW_PATH);
            writer.varuint(id);
            writer.raw_path(&raw_path);
        }
        Box::new(SerializingRenderPath {
            writer: Rc::clone(&self.writer),
            id,
            fill_rule,
            raw_path,
        })
    }

    fn make_empty_render_path(&mut self) -> Box<dyn RenderPath> {
        self.make_path()
    }

    fn make_render_paint(&mut self) -> Box<dyn RenderPaint> {
        let id = self.next_paint_id;
        self.next_paint_id += 1;
        {
            let mut writer = self.writer.borrow_mut();
            writer.varuint(MAKE_RENDER_PAINT);
            writer.varuint(id);
        }
        Box::new(SerializingRenderPaint {
            writer: Rc::clone(&self.writer),
            id,
            style: RenderPaintStyle::Fill,
            color: 0xff000000,
            thickness: 1.0,
            join: StrokeJoin::Miter,
            cap: StrokeCap::Butt,
            feather: 0.0,
            blend_mode: BlendMode::SrcOver,
            shader_id: None,
        })
    }

    fn decode_image(&mut self, data: &[u8]) -> Result<Box<dyn RenderImage>, ImageDecodeError> {
        let id = self.next_image_id;
        self.next_image_id += 1;
        {
            let mut writer = self.writer.borrow_mut();
            writer.varuint(DECODE_IMAGE);
            writer.varuint(id);
            writer.varuint(data.len() as u64);
            writer.bytes.extend_from_slice(data);
        }
        let (width, height) = encoded_image_dimensions(data);
        Ok(Box::new(SerializingRenderImage { id, width, height }))
    }
}

impl SerializingFactory {
    fn make_path(&mut self) -> Box<dyn RenderPath> {
        let id = self.next_path_id;
        self.next_path_id += 1;
        {
            let mut writer = self.writer.borrow_mut();
            writer.varuint(MAKE_RENDER_PATH);
            writer.varuint(id);
        }
        Box::new(SerializingRenderPath {
            writer: Rc::clone(&self.writer),
            id,
            fill_rule: FillRule::NonZero,
            raw_path: RawPath::new(),
        })
    }
}

fn write_stops(writer: &mut Writer, colors: &[ColorInt], stops: &[f32]) {
    writer.varuint(colors.len() as u64);
    for (&color, &stop) in colors.iter().zip(stops) {
        writer.varuint(u64::from(color));
        writer.float(stop);
    }
}

struct SerializingRenderShader {
    id: u64,
}

impl RenderShader for SerializingRenderShader {
    fn as_any(&self) -> &dyn Any {
        self
    }
}

struct SerializingRenderImage {
    id: u64,
    width: u32,
    height: u32,
}

impl RenderImage for SerializingRenderImage {
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

struct SerializingRenderPaint {
    writer: Rc<RefCell<Writer>>,
    id: u64,
    style: RenderPaintStyle,
    color: ColorInt,
    thickness: f32,
    join: StrokeJoin,
    cap: StrokeCap,
    feather: f32,
    blend_mode: BlendMode,
    shader_id: Option<u64>,
}

impl SerializingRenderPaint {
    fn write_uint(&self, operation: u64, value: u64) {
        let mut writer = self.writer.borrow_mut();
        writer.varuint(operation);
        writer.varuint(self.id);
        writer.varuint(value);
    }

    fn write_float(&self, operation: u64, value: f32) {
        let mut writer = self.writer.borrow_mut();
        writer.varuint(operation);
        writer.varuint(self.id);
        writer.float(value);
    }
}

impl RenderPaint for SerializingRenderPaint {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn style(&mut self, style: RenderPaintStyle) {
        if self.style != style {
            self.style = style;
            self.write_uint(
                STYLE,
                match style {
                    RenderPaintStyle::Stroke => 0,
                    RenderPaintStyle::Fill => 1,
                },
            );
        }
    }

    fn color(&mut self, value: ColorInt) {
        if self.color != value {
            self.color = value;
            self.write_uint(COLOR, u64::from(value));
        }
    }

    fn thickness(&mut self, value: f32) {
        if self.thickness != value {
            self.thickness = value;
            self.write_float(THICKNESS, value);
        }
    }

    fn join(&mut self, value: StrokeJoin) {
        if self.join != value {
            self.join = value;
            self.write_uint(JOIN, value as u64);
        }
    }

    fn cap(&mut self, value: StrokeCap) {
        if self.cap != value {
            self.cap = value;
            self.write_uint(CAP, value as u64);
        }
    }

    fn feather(&mut self, value: f32) {
        if self.feather != value {
            self.feather = value;
            self.write_float(FEATHER, value);
        }
    }

    fn blend_mode(&mut self, value: BlendMode) {
        if self.blend_mode != value {
            self.blend_mode = value;
            self.write_uint(BLEND_MODE, value as u64);
        }
    }

    fn shader(&mut self, shader: Option<&dyn RenderShader>) {
        let id = shader.map(|shader| {
            shader
                .as_any()
                .downcast_ref::<SerializingRenderShader>()
                .expect("SerializingFactory requires SerializingRenderShader")
                .id
        });
        if self.shader_id != id {
            self.shader_id = id;
            self.write_uint(SHADER, id.unwrap_or(0));
        }
    }

    fn invalidate_stroke(&mut self) {}
}

struct SerializingRenderPath {
    writer: Rc<RefCell<Writer>>,
    id: u64,
    fill_rule: FillRule,
    raw_path: RawPath,
}

impl SerializingRenderPath {
    fn emit_raw_path(&self, path: &RawPath) {
        let mut writer = self.writer.borrow_mut();
        writer.varuint(ADD_RAW_PATH);
        writer.varuint(self.id);
        writer.raw_path(path);
    }
}

impl RenderPath for SerializingRenderPath {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn rewind(&mut self) {
        self.raw_path.rewind();
        let mut writer = self.writer.borrow_mut();
        writer.varuint(REWIND);
        writer.varuint(self.id);
    }

    fn reserve(&mut self, verbs: usize, points: usize) {
        self.raw_path.reserve(verbs, points);
    }

    fn fill_rule(&mut self, value: FillRule) {
        if self.fill_rule != value {
            self.fill_rule = value;
            let mut writer = self.writer.borrow_mut();
            writer.varuint(FILL_RULE);
            writer.varuint(self.id);
            writer.varuint(value as u64);
        }
    }

    fn add_render_path(&mut self, path: &dyn RenderPath, transform: Mat2D) {
        let path = serializing_path(path);
        let mut appended = RawPath::new();
        appended.add_path(&path.raw_path, transform);
        self.add_raw_path(&appended);
    }

    fn add_render_path_backwards(&mut self, path: &dyn RenderPath, transform: Mat2D) {
        let path = serializing_path(path);
        let mut appended = RawPath::new();
        appended.add_path_backwards(&path.raw_path, transform);
        self.add_raw_path(&appended);
    }

    fn add_raw_path(&mut self, path: &RawPath) {
        self.raw_path.add_path(path, Mat2D::IDENTITY);
        self.emit_raw_path(path);
    }

    fn move_to(&mut self, x: f32, y: f32) {
        let mut path = RawPath::new();
        path.move_to(x, y);
        self.add_raw_path(&path);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        let mut path = RawPath::new();
        path.line_to(x, y);
        self.add_raw_path(&path);
    }

    fn cubic_to(&mut self, ox: f32, oy: f32, ix: f32, iy: f32, x: f32, y: f32) {
        let mut path = RawPath::new();
        path.cubic_to(ox, oy, ix, iy, x, y);
        self.add_raw_path(&path);
    }

    fn close(&mut self) {
        let mut path = RawPath::new();
        path.close();
        self.add_raw_path(&path);
    }
}

struct SerializingRenderBuffer {
    writer: Rc<RefCell<Writer>>,
    id: u64,
    buffer_type: RenderBufferType,
    flags: RenderBufferFlags,
    bytes: Vec<u8>,
}

impl RenderBuffer for SerializingRenderBuffer {
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
        let mut writer = self.writer.borrow_mut();
        writer.varuint(match self.buffer_type {
            RenderBufferType::Vertex => SET_VERTEX_BUFFER_DATA,
            RenderBufferType::Index => SET_INDEX_BUFFER_DATA,
        });
        writer.varuint(self.id);
        match self.buffer_type {
            RenderBufferType::Vertex => {
                for bytes in self.bytes.chunks_exact(4) {
                    writer.float(f32::from_le_bytes(
                        bytes.try_into().expect("four-byte chunk"),
                    ));
                }
            }
            RenderBufferType::Index => {
                for bytes in self.bytes.chunks_exact(2) {
                    writer.varuint(u64::from(u16::from_le_bytes(
                        bytes.try_into().expect("two-byte chunk"),
                    )));
                }
            }
        }
    }
}

pub struct SerializingRenderer {
    writer: Rc<RefCell<Writer>>,
}

impl Renderer for SerializingRenderer {
    fn save(&mut self) {
        self.writer.borrow_mut().varuint(SAVE);
    }

    fn restore(&mut self) {
        self.writer.borrow_mut().varuint(RESTORE);
    }

    fn transform(&mut self, transform: Mat2D) {
        let mut writer = self.writer.borrow_mut();
        writer.varuint(TRANSFORM);
        for value in transform.0 {
            writer.float(value);
        }
    }

    fn draw_path(&mut self, path: &dyn RenderPath, paint: &dyn RenderPaint) {
        let path = serializing_path(path);
        let paint = paint
            .as_any()
            .downcast_ref::<SerializingRenderPaint>()
            .expect("SerializingFactory requires SerializingRenderPaint");
        let mut writer = self.writer.borrow_mut();
        writer.varuint(DRAW_PATH);
        writer.varuint(path.id);
        writer.varuint(paint.id);
    }

    fn clip_path(&mut self, path: &dyn RenderPath) {
        let path = serializing_path(path);
        let mut writer = self.writer.borrow_mut();
        writer.varuint(CLIP_PATH);
        writer.varuint(path.id);
    }

    fn draw_image(
        &mut self,
        image: Option<&dyn RenderImage>,
        _sampler: ImageSampler,
        blend_mode: BlendMode,
        opacity: f32,
    ) {
        let image = serializing_image(image);
        let mut writer = self.writer.borrow_mut();
        writer.varuint(DRAW_IMAGE);
        writer.varuint(image.id);
        writer.varuint(blend_mode as u64);
        writer.float(opacity);
    }

    fn draw_image_mesh(
        &mut self,
        image: Option<&dyn RenderImage>,
        _sampler: ImageSampler,
        vertices: Option<&dyn RenderBuffer>,
        uv_coords: Option<&dyn RenderBuffer>,
        indices: Option<&dyn RenderBuffer>,
        _vertex_count: u32,
        _index_count: u32,
        blend_mode: BlendMode,
        opacity: f32,
    ) {
        let image = serializing_image(image);
        let mut writer = self.writer.borrow_mut();
        writer.varuint(DRAW_IMAGE_MESH);
        writer.varuint(image.id);
        writer.varuint(blend_mode as u64);
        writer.float(opacity);
        for buffer in [vertices, uv_coords, indices] {
            writer.varuint(serializing_buffer(buffer).id);
        }
    }

    fn modulate_opacity(&mut self, opacity: f32) {
        let mut writer = self.writer.borrow_mut();
        writer.varuint(MODULATE_OPACITY);
        writer.float(opacity);
    }
}

fn serializing_path(path: &dyn RenderPath) -> &SerializingRenderPath {
    path.as_any()
        .downcast_ref::<SerializingRenderPath>()
        .expect("SerializingFactory requires SerializingRenderPath")
}

fn serializing_image(image: Option<&dyn RenderImage>) -> &SerializingRenderImage {
    image
        .and_then(|image| image.as_any().downcast_ref::<SerializingRenderImage>())
        .expect("SerializingFactory requires a non-null SerializingRenderImage")
}

fn serializing_buffer(buffer: Option<&dyn RenderBuffer>) -> &SerializingRenderBuffer {
    buffer
        .and_then(|buffer| buffer.as_any().downcast_ref::<SerializingRenderBuffer>())
        .expect("SerializingFactory requires a non-null SerializingRenderBuffer")
}

#[allow(dead_code)]
fn _path_verb_wire_values_are_stable(verb: PathVerb) -> u64 {
    verb as u64
}
