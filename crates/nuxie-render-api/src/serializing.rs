use super::{
    encoded_image_dimensions, BlendMode, ColorInt, Factory, FillRule, ImageDecodeError,
    ImageSampler, LayerMaskMode, Mat2D, PathVerb, RawPath, RawPathRef, RenderBuffer,
    RenderBufferFlags, RenderBufferType, RenderImage, RenderPaint, RenderPaintStyle, RenderPath,
    RenderShader, Renderer, StrokeCap, StrokeJoin, StrokePosition,
};
use crate::{
    DeferredCanvasHost, DeferredCanvasHostHandle, ImageMeshInstanceData, ImageMeshInstances,
    ImageMeshInstancesHandle, ImageMeshInstancesStorage, PersistentFactoryContext,
    RenderCanvasHandle,
};
use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::serialize_ops::*;

#[derive(Debug)]
pub(crate) struct Writer {
    bytes: Vec<u8>,
}

impl Writer {
    fn new() -> Self {
        Self {
            bytes: b"SRIV\x01".to_vec(),
        }
    }

    pub(crate) fn varuint(&mut self, mut value: u64) {
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

    pub(crate) fn float(&mut self, value: f32) {
        self.bytes.extend_from_slice(&value.to_le_bytes());
    }

    fn raw_path(&mut self, path: RawPathRef<'_>) {
        serialize_raw_path(self, path);
    }
}

pub struct SerializingFactory {
    writer: Rc<RefCell<Writer>>,
    canvases: Rc<RefCell<SerializingCanvases>>,
    next_paint_id: u64,
    next_path_id: u64,
    next_buffer_id: u64,
    next_shader_id: u64,
    next_image_mesh_instances_id: u64,
}

#[derive(Default)]
struct SerializingCanvases {
    next_image_id: u64,
    bitmap_cache_context: Option<PersistentFactoryContext>,
    image_ids: HashMap<usize, u64>,
    // Retain all registered canvases so recycled allocations never inherit IDs.
    retained: Vec<RenderCanvasHandle>,
}

#[derive(Clone)]
struct SerializingCanvasHost {
    writer: Rc<RefCell<Writer>>,
    canvases: Rc<RefCell<SerializingCanvases>>,
}

impl SerializingCanvasHost {
    fn canvas_id(&self, canvas: &RenderCanvasHandle) -> u64 {
        let (image, width, height) = {
            let canvas = canvas.borrow();
            (canvas.render_image(), canvas.width(), canvas.height())
        };
        let identity = image.image_identity();
        let mut canvases = self.canvases.borrow_mut();
        if let Some(id) = canvases.image_ids.get(&identity) {
            return *id;
        }
        let id = canvases.next_image_id;
        canvases.next_image_id += 1;
        canvases.image_ids.insert(identity, id);
        canvases.retained.push(canvas.clone());
        let mut writer = self.writer.borrow_mut();
        writer.varuint(MAKE_RENDER_CANVAS);
        writer.varuint(id);
        writer.varuint(u64::from(width));
        writer.varuint(u64::from(height));
        id
    }
}

impl DeferredCanvasHost for SerializingCanvasHost {
    fn supports_layer_mask(&self) -> bool {
        true
    }
    fn make_content_canvas(&mut self, width: u32, height: u32) -> Option<RenderCanvasHandle> {
        let mut context = self.canvases.borrow().bitmap_cache_context.clone()?;
        let canvas = context.make_deferred_render_canvas(width, height).ok()?;
        Some(Rc::new(RefCell::new(canvas)))
    }
    fn content_canvas_image(&mut self, canvas: &RenderCanvasHandle) -> Option<Rc<dyn RenderImage>> {
        Some(canvas.borrow().render_image())
    }
    fn begin_canvas_content(
        &mut self,
        canvas: RenderCanvasHandle,
        clear_color: ColorInt,
    ) -> Option<Box<dyn Renderer>> {
        let id = self.canvas_id(&canvas);
        let mut writer = self.writer.borrow_mut();
        writer.varuint(CANVAS_CONTENT_BEGIN);
        writer.varuint(id);
        writer.varuint(u64::from(clear_color));
        // Each owned proxy targets the same stateless recorder. Ending a nested
        // bracket cannot invalidate the outer proxy or release its writer.
        Some(Box::new(SerializingRenderer {
            writer: self.writer.clone(),
            canvases: self.canvases.clone(),
        }))
    }
    fn end_canvas_content(&mut self, canvas: &RenderCanvasHandle) {
        // Match source ordering, including malformed use of an undeclared end.
        self.writer.borrow_mut().varuint(CANVAS_CONTENT_END);
        let id = self.canvas_id(canvas);
        self.writer.borrow_mut().varuint(id);
    }
}

impl SerializingFactory {
    pub fn new() -> Self {
        Self {
            writer: Rc::new(RefCell::new(Writer::new())),
            canvases: Rc::new(RefCell::new(SerializingCanvases::default())),
            next_paint_id: 0,
            next_path_id: 0,
            next_buffer_id: 0,
            next_shader_id: 0,
            next_image_mesh_instances_id: 0,
        }
    }

    pub fn make_renderer(&self) -> SerializingRenderer {
        SerializingRenderer {
            writer: Rc::clone(&self.writer),
            canvases: self.canvases.clone(),
        }
    }

    pub fn enable_bitmap_cache(&mut self, render_context: Option<PersistentFactoryContext>) {
        self.canvases.borrow_mut().bitmap_cache_context = render_context;
    }

    fn canvas_host(&self) -> SerializingCanvasHost {
        SerializingCanvasHost {
            writer: self.writer.clone(),
            canvases: self.canvases.clone(),
        }
    }

    pub fn begin_canvas_content(
        &mut self,
        canvas: Option<RenderCanvasHandle>,
        clear_color: ColorInt,
    ) -> Option<Box<dyn Renderer>> {
        self.canvas_host()
            .begin_canvas_content(canvas?, clear_color)
    }

    pub fn end_canvas_content(&mut self, canvas: Option<&RenderCanvasHandle>) {
        if let Some(canvas) = canvas {
            self.canvas_host().end_canvas_content(canvas);
        }
    }

    pub fn content_canvas_image(
        &self,
        canvas: Option<&RenderCanvasHandle>,
    ) -> Option<Rc<dyn RenderImage>> {
        Some(canvas?.borrow().render_image())
    }

    pub fn image_id(&self, image: &dyn RenderImage) -> u64 {
        image_id(&self.canvases.borrow(), image)
    }

    pub fn frame_size(&mut self, width: u32, height: u32) {
        let mut writer = self.writer.borrow_mut();
        writer.varuint(FRAME_SIZE);
        writer.varuint(u64::from(width));
        writer.varuint(u64::from(height));
    }

    pub fn add_frame(&mut self) {
        super::increment_artboard_draw_frame_id();
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

impl DeferredCanvasHost for SerializingFactory {
    fn supports_layer_mask(&self) -> bool {
        true
    }
    fn make_content_canvas(&mut self, width: u32, height: u32) -> Option<RenderCanvasHandle> {
        self.canvas_host().make_content_canvas(width, height)
    }
    fn content_canvas_image(&mut self, canvas: &RenderCanvasHandle) -> Option<Rc<dyn RenderImage>> {
        SerializingFactory::content_canvas_image(self, Some(canvas))
    }
    fn begin_canvas_content(
        &mut self,
        canvas: RenderCanvasHandle,
        clear_color: ColorInt,
    ) -> Option<Box<dyn Renderer>> {
        SerializingFactory::begin_canvas_content(self, Some(canvas), clear_color)
    }
    fn end_canvas_content(&mut self, canvas: &RenderCanvasHandle) {
        SerializingFactory::end_canvas_content(self, Some(canvas))
    }
}

impl Factory for SerializingFactory {
    fn supports_layer_mask(&self) -> bool {
        true
    }
    fn make_image_mesh_instances(&mut self, count: usize) -> ImageMeshInstancesHandle {
        let id = self.next_image_mesh_instances_id;
        self.next_image_mesh_instances_id += 1;
        {
            let mut writer = self.writer.borrow_mut();
            writer.varuint(MAKE_IMAGE_MESH_INSTANCES);
            writer.varuint(id);
            writer.varuint(count as u64);
        }
        Rc::new(RefCell::new(SerializingImageMeshInstances {
            storage: ImageMeshInstancesStorage::new(count),
            writer: self.writer.clone(),
            id,
        }))
    }
    fn render_context(&mut self) -> Option<PersistentFactoryContext> {
        self.canvases.borrow().bitmap_cache_context.clone()
    }
    fn deferred_canvas_host(&mut self) -> Option<DeferredCanvasHostHandle> {
        self.canvases.borrow().bitmap_cache_context.as_ref()?;
        Some(Rc::new(RefCell::new(self.canvas_host())))
    }
    fn canvas_content_host(&mut self) -> Option<DeferredCanvasHostHandle> {
        self.deferred_canvas_host()
    }
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
        Box::new(SerializingRenderShader {
            id,
            identity: Rc::new(()),
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
        let id = self.next_shader_id;
        self.next_shader_id += 1;
        let mut writer = self.writer.borrow_mut();
        writer.varuint(MAKE_RADIAL_GRADIENT);
        writer.varuint(id);
        write_stops(&mut writer, colors, stops);
        for value in [cx, cy, radius] {
            writer.float(value);
        }
        Box::new(SerializingRenderShader {
            id,
            identity: Rc::new(()),
        })
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
            writer.raw_path(raw_path.as_ref());
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
            stroke_position: StrokePosition::Center,
            feather: 0.0,
            additiveness: 0.0,
            blend_mode: BlendMode::SrcOver,
            shader_id: None,
            shader_transform: Mat2D::IDENTITY,
        })
    }

    fn decode_image(&mut self, data: &[u8]) -> Result<Box<dyn RenderImage>, ImageDecodeError> {
        let id = {
            let mut canvases = self.canvases.borrow_mut();
            let id = canvases.next_image_id;
            canvases.next_image_id += 1;
            id
        };
        {
            let mut writer = self.writer.borrow_mut();
            writer.varuint(DECODE_IMAGE);
            writer.varuint(id);
            writer.varuint(data.len() as u64);
            writer.bytes.extend_from_slice(data);
        }
        let (width, height) = encoded_image_dimensions(data);
        Ok(Box::new(SerializingRenderImage {
            id,
            width,
            height,
            identity: Rc::new(()),
        }))
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

#[derive(Clone)]
struct SerializingRenderShader {
    id: u64,
    identity: Rc<()>,
}

impl RenderShader for SerializingRenderShader {
    fn retain_shader(&self) -> Rc<dyn RenderShader> {
        Rc::new(self.clone())
    }
    fn shader_identity(&self) -> usize {
        Rc::as_ptr(&self.identity) as usize
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

#[derive(Clone)]
struct SerializingRenderImage {
    identity: Rc<()>,
    id: u64,
    width: u32,
    height: u32,
}

impl RenderImage for SerializingRenderImage {
    fn retain_image(&self) -> Rc<dyn RenderImage> {
        Rc::new(self.clone())
    }
    fn image_identity(&self) -> usize {
        Rc::as_ptr(&self.identity) as usize
    }
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
    stroke_position: StrokePosition,
    feather: f32,
    additiveness: f32,
    blend_mode: BlendMode,
    shader_id: Option<u64>,
    shader_transform: Mat2D,
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

    fn stroke_position(&mut self, value: StrokePosition) {
        if self.stroke_position == value {
            return;
        }
        self.stroke_position = value;
        self.write_uint(STROKE_POSITION, value as u64);
    }

    fn feather(&mut self, value: f32) {
        if self.feather != value {
            self.feather = value;
            self.write_float(FEATHER, value);
        }
    }

    fn additiveness(&mut self, value: f32) {
        if self.additiveness == value {
            return;
        }
        self.additiveness = value;
        self.write_float(ADDITIVENESS, value);
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

    fn shader_transform(&mut self, transform: Mat2D) {
        if self.shader_transform == transform {
            return;
        }
        self.shader_transform = transform;
        let mut writer = self.writer.borrow_mut();
        writer.varuint(SHADER_TRANSFORM);
        writer.varuint(self.id);
        for value in transform.0 {
            writer.float(value);
        }
    }

    fn modulated_image(
        &mut self,
        image: Option<&dyn RenderImage>,
        sampler: ImageSampler,
        matrix: Mat2D,
    ) {
        let mut writer = self.writer.borrow_mut();
        writer.varuint(PAINT_MODULATED_IMAGE);
        writer.varuint(self.id);
        writer.varuint(image.map_or(0, |image| {
            image
                .as_any()
                .downcast_ref::<SerializingRenderImage>()
                .expect("SerializingFactory requires SerializingRenderImage")
                .id
                + 1
        }));
        writer.varuint(sampler.filter as u64);
        writer.varuint(sampler.wrap_x as u64);
        writer.varuint(sampler.wrap_y as u64);
        for value in matrix.0 {
            writer.float(value);
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
    fn emit_raw_path(&self, path: RawPathRef<'_>) {
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

    fn as_any_mut(&mut self) -> &mut dyn Any {
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
        self.add_raw_path(appended.as_ref());
    }

    fn add_render_path_self(&mut self, transform: Mat2D) {
        let source = self.raw_path.clone();
        let mut appended = RawPath::new();
        appended.add_path(&source, transform);
        self.add_raw_path(appended.as_ref());
    }

    fn add_render_path_backwards(&mut self, path: &dyn RenderPath, transform: Mat2D) {
        let path = serializing_path(path);
        let mut appended = RawPath::new();
        appended.add_path_backwards_with_transform(&path.raw_path, transform);
        self.add_raw_path(appended.as_ref());
    }

    fn add_raw_path(&mut self, path: RawPathRef<'_>) {
        self.raw_path.add_path_view(path);
        self.emit_raw_path(path);
    }

    fn move_to(&mut self, x: f32, y: f32) {
        let mut path = RawPath::new();
        path.move_to(x, y);
        self.add_raw_path(path.as_ref());
    }

    fn line_to(&mut self, x: f32, y: f32) {
        let mut path = RawPath::new();
        path.line_to(x, y);
        self.add_raw_path(path.as_ref());
    }

    fn cubic_to(&mut self, ox: f32, oy: f32, ix: f32, iy: f32, x: f32, y: f32) {
        let mut path = RawPath::new();
        path.cubic_to(ox, oy, ix, iy, x, y);
        self.add_raw_path(path.as_ref());
    }

    fn close(&mut self) {
        let mut path = RawPath::new();
        path.close();
        self.add_raw_path(path.as_ref());
    }
}

struct SerializingImageMeshInstances {
    storage: ImageMeshInstancesStorage,
    writer: Rc<RefCell<Writer>>,
    id: u64,
}

impl ImageMeshInstances for SerializingImageMeshInstances {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn instance_data(&self) -> &[ImageMeshInstanceData] {
        self.storage.instance_data()
    }
    fn edit(&mut self, count: Option<usize>) -> &mut [ImageMeshInstanceData] {
        self.storage.edit(count)
    }
    fn end_edit(&mut self) {
        self.storage.end_edit();
        let data = self.storage.instance_data();
        let mut writer = self.writer.borrow_mut();
        writer.varuint(SET_IMAGE_MESH_INSTANCES_DATA);
        writer.varuint(self.id);
        writer.varuint(data.len() as u64);
        for instance in data {
            for value in instance.transform.0 {
                writer.float(value);
            }
            for value in instance.uv_translate {
                writer.float(value);
            }
            for value in instance.uv_scale {
                writer.float(value);
            }
            writer.float(instance.opacity);
            writer.float(instance.additiveness);
        }
    }
    fn edit_count(&self) -> usize {
        self.storage.edit_count()
    }
    fn is_editing(&self) -> bool {
        self.storage.is_editing()
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
    canvases: Rc<RefCell<SerializingCanvases>>,
}

impl Renderer for SerializingRenderer {
    fn apply_layer_mask(
        &mut self,
        mask: Option<&dyn RenderImage>,
        _sampler: ImageSampler,
        mode: LayerMaskMode,
    ) {
        let id = image_id(
            &self.canvases.borrow(),
            mask.expect("non-null serialized mask"),
        );
        let mut writer = self.writer.borrow_mut();
        writer.varuint(APPLY_LAYER_MASK);
        writer.varuint(id);
        writer.varuint(mode as u64);
    }
    fn draw_image_mesh_instanced(
        &mut self,
        image: Option<&dyn RenderImage>,
        _sampler: ImageSampler,
        vertices: Option<&dyn RenderBuffer>,
        uv_coords: Option<&dyn RenderBuffer>,
        indices: Option<&dyn RenderBuffer>,
        _vertex_count: u32,
        _index_count: u32,
        instances: Option<&ImageMeshInstancesHandle>,
    ) {
        let id = image_id(
            &self.canvases.borrow(),
            image.expect("non-null serialized image"),
        );
        let instances = instances.expect("non-null serialized instances").borrow();
        let instances = instances
            .as_any()
            .downcast_ref::<SerializingImageMeshInstances>()
            .expect("SerializingFactory requires SerializingImageMeshInstances");
        let mut writer = self.writer.borrow_mut();
        writer.varuint(DRAW_IMAGE_MESH_INSTANCED);
        writer.varuint(id);
        for buffer in [vertices, uv_coords, indices] {
            writer.varuint(serializing_buffer(buffer).id);
        }
        writer.varuint(instances.id);
    }
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
        self.draw_image_with_additiveness(image, _sampler, blend_mode, opacity, 0.0);
    }

    fn draw_image_with_additiveness(
        &mut self,
        image: Option<&dyn RenderImage>,
        _sampler: ImageSampler,
        blend_mode: BlendMode,
        opacity: f32,
        additiveness: f32,
    ) {
        let id = image_id(
            &self.canvases.borrow(),
            image.expect("non-null serialized image"),
        );
        let mut writer = self.writer.borrow_mut();
        let is_additive = additiveness != 0.0;
        writer.varuint(if is_additive {
            DRAW_IMAGE_ADDITIVE
        } else {
            DRAW_IMAGE
        });
        writer.varuint(id);
        writer.varuint(blend_mode as u64);
        writer.float(opacity);
        if is_additive {
            writer.float(additiveness);
        }
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
        self.draw_image_mesh_with_additiveness(
            image,
            _sampler,
            vertices,
            uv_coords,
            indices,
            _vertex_count,
            _index_count,
            blend_mode,
            opacity,
            0.0,
        );
    }

    fn draw_image_mesh_with_additiveness(
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
        additiveness: f32,
    ) {
        let id = image_id(
            &self.canvases.borrow(),
            image.expect("non-null serialized image"),
        );
        let mut writer = self.writer.borrow_mut();
        let is_additive = additiveness != 0.0;
        writer.varuint(if is_additive {
            DRAW_IMAGE_MESH_ADDITIVE
        } else {
            DRAW_IMAGE_MESH
        });
        writer.varuint(id);
        writer.varuint(blend_mode as u64);
        writer.float(opacity);
        for buffer in [vertices, uv_coords, indices] {
            writer.varuint(serializing_buffer(buffer).id);
        }
        if is_additive {
            writer.float(additiveness);
        }
    }

    fn modulate_opacity(&mut self, opacity: f32) {
        let mut writer = self.writer.borrow_mut();
        writer.varuint(MODULATE_OPACITY);
        writer.float(opacity);
    }

    fn modulate_color(&mut self, color: ColorInt, replace: bool) {
        let mut writer = self.writer.borrow_mut();
        writer.varuint(MODULATE_COLOR);
        writer.varuint(color as u64);
        writer.varuint(u64::from(replace));
    }
}

fn serializing_path(path: &dyn RenderPath) -> &SerializingRenderPath {
    path.as_any()
        .downcast_ref::<SerializingRenderPath>()
        .expect("SerializingFactory requires SerializingRenderPath")
}

fn image_id(canvases: &SerializingCanvases, image: &dyn RenderImage) -> u64 {
    canvases
        .image_ids
        .get(&image.image_identity())
        .copied()
        .unwrap_or_else(|| {
            image
                .as_any()
                .downcast_ref::<SerializingRenderImage>()
                .expect(
                    "SerializingFactory requires its decoded image or a registered canvas image",
                )
                .id
        })
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
