//! renderer/cmd/deferred_render_resource.hpp at e949498e.
use super::{
    command_stream::WirePod,
    render_command_buffer::{RenderCommandBuffer, SharedIdAllocator},
    render_commands::*,
    render_handle::INVALID_RENDER_HANDLE,
};
use nuxie_render_api::*;
use std::{
    any::Any,
    cell::{Cell, RefCell},
    rc::Rc,
    sync::{Arc, Mutex, Weak},
};

pub type SharedRenderCommandBuffer = Arc<Mutex<RenderCommandBuffer>>;
pub struct DeferredResourceBase {
    pub id: u32,
    pub buffer: Weak<Mutex<RenderCommandBuffer>>,
    generation: u32,
    allocator: SharedIdAllocator,
    kind: ResourceKind,
}
impl DeferredResourceBase {
    pub fn generation(&self) -> u32 {
        self.generation
    }
    pub fn new(
        kind: ResourceKind,
        id: u32,
        generation: u32,
        buffer: &SharedRenderCommandBuffer,
        allocator: SharedIdAllocator,
    ) -> Self {
        Self {
            id,
            generation,
            buffer: Arc::downgrade(buffer),
            allocator,
            kind,
        }
    }
    pub fn commands(&self) -> SharedRenderCommandBuffer {
        self.buffer
            .upgrade()
            .expect("live deferred recording session")
    }
    pub fn append<P: WirePod>(&self, command: RenderCmd, pod: &P) {
        self.commands().lock().unwrap().append(command, pod);
    }
}
impl Drop for DeferredResourceBase {
    fn drop(&mut self) {
        // A weak upgrade keeps the recorder alive through queue insertion, the
        // same lifetime exclusion as upstream's live-recorder registry lock.
        if let Some(buffer) = self.buffer.upgrade() {
            let buffer = buffer.lock().unwrap();
            if buffer.recorder_live() {
                buffer.queue_destroy(
                    self.kind as u8,
                    self.id,
                    self.generation,
                    Some(self.allocator.clone()),
                );
            }
        }
    }
}
pub struct VersionedDeferredResource {
    pub base: DeferredResourceBase,
    version: Cell<u32>,
    drawn_frame: Cell<u32>,
}
impl VersionedDeferredResource {
    pub fn new(base: DeferredResourceBase) -> Self {
        Self {
            base,
            version: Cell::new(0),
            drawn_frame: Cell::new(u32::MAX),
        }
    }
    pub fn version(&self) -> u32 {
        self.version.get()
    }
    pub fn mark_drawn(&self) {
        self.drawn_frame
            .set(self.base.commands().lock().unwrap().frame_id());
    }
    pub fn bump(&self) {
        let commands = self.base.commands();
        let mut commands = commands.lock().unwrap();
        if self.drawn_frame.get() != commands.frame_id() {
            return;
        }
        self.version.set(self.version.get().wrapping_add(1));
        self.drawn_frame.set(u32::MAX);
        commands.append(
            RenderCmd::ResourceNewVersion,
            &ResourceVersionPod {
                kind: self.base.kind as u8,
                id: self.base.id,
                version: self.version.get(),
            },
        );
    }
}

#[derive(Clone)]
pub struct DeferredRenderShader {
    pub base: Rc<DeferredResourceBase>,
}
impl RenderShader for DeferredRenderShader {
    fn shader_identity(&self) -> usize {
        Rc::as_ptr(&self.base) as usize
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn retain_shader(&self) -> Rc<dyn RenderShader> {
        Rc::new(self.clone())
    }
}

pub struct DeferredRenderPaint {
    pub resource: VersionedDeferredResource,
    style: u8,
    color: u32,
    thickness: f32,
    join: u8,
    cap: u8,
    feather: f32,
    blend_mode: u8,
    shader: Option<Rc<dyn RenderShader>>,
    shader_identity: Option<usize>,
    color_known: bool,
    stroke_invalidated: Cell<bool>,
}
impl Drop for DeferredRenderPaint {
    fn drop(&mut self) {
        // C++ destroys members before inherited resource state: a shader held
        // only by this paint must queue its destroy before the paint's own.
        drop(self.shader.take());
    }
}
impl DeferredRenderPaint {
    pub fn new(base: DeferredResourceBase) -> Self {
        Self {
            resource: VersionedDeferredResource::new(base),
            style: 1,
            color: 0xff000000,
            thickness: 1.0,
            join: 0,
            cap: 0,
            feather: 0.0,
            blend_mode: 3,
            shader: None,
            shader_identity: None,
            color_known: true,
            stroke_invalidated: Cell::new(false),
        }
    }
    pub fn mark_drawn(&self) {
        self.resource.mark_drawn();
        self.stroke_invalidated.set(false);
    }
    fn emit_u8(&self, command: RenderCmd, value: u8) {
        self.resource.bump();
        self.resource.base.append(
            command,
            &PaintU8Pod {
                paint: self.resource.base.id,
                value,
            },
        );
    }
    fn emit_float(&self, command: RenderCmd, value: f32) {
        self.resource.bump();
        self.resource.base.append(
            command,
            &PaintFloatPod {
                paint: self.resource.base.id,
                value,
            },
        );
    }
}
impl RenderPaint for DeferredRenderPaint {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn style(&mut self, value: RenderPaintStyle) {
        let value = value as u8;
        if self.style == value {
            return;
        }
        self.style = value;
        self.emit_u8(RenderCmd::PaintStyle, value);
    }
    fn color(&mut self, value: ColorInt) {
        let unchanged = self.color == value;
        self.color = value;
        if unchanged && self.color_known {
            return;
        }
        self.color_known = true;
        self.resource.bump();
        self.resource.base.append(
            RenderCmd::PaintColor,
            &PaintColorPod {
                paint: self.resource.base.id,
                color: value,
            },
        );
    }
    fn thickness(&mut self, value: f32) {
        if self.thickness == value {
            return;
        }
        self.thickness = value;
        self.emit_float(RenderCmd::PaintThickness, value);
    }
    fn join(&mut self, value: StrokeJoin) {
        let value = value as u8;
        if self.join == value {
            return;
        }
        self.join = value;
        self.emit_u8(RenderCmd::PaintJoin, value);
    }
    fn cap(&mut self, value: StrokeCap) {
        let value = value as u8;
        if self.cap == value {
            return;
        }
        self.cap = value;
        self.emit_u8(RenderCmd::PaintCap, value);
    }
    fn feather(&mut self, value: f32) {
        if self.feather == value {
            return;
        }
        self.feather = value;
        self.emit_float(RenderCmd::PaintFeather, value);
    }
    fn blend_mode(&mut self, value: BlendMode) {
        let value = value as u8;
        if self.blend_mode == value {
            return;
        }
        self.blend_mode = value;
        self.emit_u8(RenderCmd::PaintBlendMode, value);
    }
    fn shader(&mut self, value: Option<&dyn RenderShader>) {
        let identity = value.map(RenderShader::shader_identity);
        if self.shader_identity == identity {
            return;
        }
        self.shader_identity = identity;
        self.shader = value.map(RenderShader::retain_shader);
        let shader = self
            .shader
            .as_ref()
            .and_then(|s| s.as_any().downcast_ref::<DeferredRenderShader>())
            .map_or(INVALID_RENDER_HANDLE, |s| s.base.id);
        self.color_known = false;
        self.resource.bump();
        self.resource.base.append(
            RenderCmd::PaintShader,
            &PaintShaderPod {
                paint: self.resource.base.id,
                shader,
            },
        );
    }
    fn invalidate_stroke(&mut self) {
        if self.stroke_invalidated.replace(true) {
            return;
        }
        self.resource.base.append(
            RenderCmd::PaintInvalidateStroke,
            &ResIdPod {
                id: self.resource.base.id,
            },
        );
    }
}

pub struct DeferredRenderPath {
    pub resource: VersionedDeferredResource,
    scratch: RefCell<RawPath>,
    fill_rule: FillRule,
    have_fill_rule: bool,
}
impl DeferredRenderPath {
    pub fn new(base: DeferredResourceBase) -> Self {
        Self {
            resource: VersionedDeferredResource::new(base),
            scratch: RefCell::new(RawPath::new()),
            fill_rule: FillRule::NonZero,
            have_fill_rule: false,
        }
    }
    pub fn flush_scratch(&self) {
        let mut scratch = self.scratch.borrow_mut();
        if scratch.verbs().is_empty() {
            return;
        }
        self.record_add_raw_path(&scratch);
        scratch.rewind();
    }
    pub fn flush_scratch_of(path: &dyn RenderPath) {
        if let Some(path) = path.as_any().downcast_ref::<Self>() {
            path.flush_scratch();
        }
    }
    pub fn id_of_path(path: &dyn RenderPath) -> u32 {
        path.as_any()
            .downcast_ref::<Self>()
            .map_or(INVALID_RENDER_HANDLE, |p| p.resource.base.id)
    }
    fn record_add_raw_path(&self, path: &RawPath) {
        self.resource.bump();
        let commands = self.resource.base.commands();
        let mut commands = commands.lock().unwrap();
        let (verbs, points) = raw_path_bytes(path);
        let blob_offset = commands.append_blob(&verbs);
        let points_offset = commands.append_blob(&points);
        commands.append(
            RenderCmd::PathAddRawPath,
            &PathRawPod {
                blob_offset,
                points_offset,
                path: self.resource.base.id,
                verb_count: path.verbs().len() as u32,
                point_count: path.points().len() as u32,
                pad: 0,
            },
        );
    }
}
pub fn raw_path_bytes(path: &RawPath) -> (Vec<u8>, Vec<u8>) {
    let verbs = path.verbs().iter().map(|&v| v as u8).collect();
    let mut points = Vec::with_capacity(path.points().len() * 8);
    for point in path.points() {
        point.x.encode(&mut points);
        point.y.encode(&mut points);
    }
    (verbs, points)
}
impl RenderPath for DeferredRenderPath {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn rewind(&mut self) {
        self.resource.bump();
        self.scratch.get_mut().rewind();
        self.resource.base.append(
            RenderCmd::PathRewind,
            &ResIdPod {
                id: self.resource.base.id,
            },
        );
    }
    fn fill_rule(&mut self, value: FillRule) {
        if self.have_fill_rule && self.fill_rule == value {
            return;
        }
        self.have_fill_rule = true;
        self.fill_rule = value;
        self.resource.bump();
        self.resource.base.append(
            RenderCmd::PathFillRule,
            &PathFillRulePod {
                path: self.resource.base.id,
                fill_rule: value as u8,
            },
        );
    }
    fn add_render_path(&mut self, path: &dyn RenderPath, transform: Mat2D) {
        self.resource.bump();
        self.flush_scratch();
        Self::flush_scratch_of(path);
        let [xx, xy, yx, yy, tx, ty] = transform.0;
        self.resource.base.append(
            RenderCmd::PathAddRenderPath,
            &PathAddPathPod {
                path: self.resource.base.id,
                src: Self::id_of_path(path),
                xx,
                xy,
                yx,
                yy,
                tx,
                ty,
            },
        );
    }
    fn add_render_path_backwards(&mut self, _path: &dyn RenderPath, _transform: Mat2D) {
        // Inherits RenderPath's no-op on non-Rive renderers (renderer.hpp:191).
    }
    fn add_raw_path(&mut self, path: &RawPath) {
        self.flush_scratch();
        self.record_add_raw_path(path);
    }
    fn move_to(&mut self, x: f32, y: f32) {
        self.scratch.get_mut().move_to(x, y);
    }
    fn line_to(&mut self, x: f32, y: f32) {
        self.scratch.get_mut().line_to(x, y);
    }
    fn cubic_to(&mut self, ox: f32, oy: f32, ix: f32, iy: f32, x: f32, y: f32) {
        self.scratch.get_mut().cubic_to(ox, oy, ix, iy, x, y);
    }
    fn close(&mut self) {
        self.scratch.get_mut().close();
    }
}

#[derive(Clone)]
pub struct DeferredRenderImage {
    pub base: Rc<DeferredResourceBase>,
    pub width: u32,
    pub height: u32,
}
impl RenderImage for DeferredRenderImage {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn width(&self) -> u32 {
        self.width
    }
    fn height(&self) -> u32 {
        self.height
    }
    fn retain_image(&self) -> Rc<dyn RenderImage> {
        Rc::new(self.clone())
    }
    fn image_identity(&self) -> usize {
        Rc::as_ptr(&self.base) as usize
    }
    fn deferred_image_id(&self) -> Option<u32> {
        Some(self.base.id)
    }
}

pub struct DeferredRenderBuffer {
    pub resource: VersionedDeferredResource,
    pub buffer_type: RenderBufferType,
    pub flags: RenderBufferFlags,
    pub size: usize,
    scratch: Vec<u8>,
}
impl DeferredRenderBuffer {
    pub fn new(
        base: DeferredResourceBase,
        buffer_type: RenderBufferType,
        flags: RenderBufferFlags,
        size: usize,
    ) -> Self {
        Self {
            resource: VersionedDeferredResource::new(base),
            buffer_type,
            flags,
            size,
            scratch: Vec::new(),
        }
    }
}
impl RenderBuffer for DeferredRenderBuffer {
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
        self.size
    }
    fn map_mut(&mut self) -> &mut [u8] {
        self.scratch.resize(self.size, 0);
        &mut self.scratch
    }
    fn unmap(&mut self) {
        self.resource.bump();
        let commands = self.resource.base.commands();
        let mut commands = commands.lock().unwrap();
        let blob_offset = commands.append_blob(&self.scratch);
        commands.append(
            RenderCmd::BufferData,
            &BufferDataPod {
                blob_offset,
                buffer: self.resource.base.id,
                size: self.scratch.len() as u32,
            },
        );
    }
}
