//! renderer/cmd/deferred_session.hpp at e3c5dec2.
use super::render_replay::RendererOwner;
use super::{
    deferred_render_factory::*, deferred_render_resource::SharedRenderCommandBuffer,
    foreign_image_registry::ForeignImageRegistry, render_commands::*, render_handle::*,
};
use crate::authored_ore_shader::{profile_for_target, ExactGpuCanvasShaderOccurrence};
use crate::deferred::ore::ore_deferred_context::DeferredOreContext;
use nuxie_ore_metal::context::{ContextApi, ReplayCaps};
use nuxie_render_api::*;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SegmentTarget {
    Canvas,
    Screen,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DeferredSegment {
    pub target: SegmentTarget,
    pub target_id: u64,
    pub begin: u32,
    pub end: u32,
}

// The source session's route fields have a separate interior owner so calling
// its returned renderer can route without borrowing the factory or ORE owner.
pub struct SessionRouting {
    pub buffer: SharedRenderCommandBuffer,
    pub canvases: Rc<RefCell<ForeignImageRegistry>>,
    pub content_canvases: HashMap<u32, RenderCanvasHandle>,
    canvas_clear: HashMap<u64, u32>,
    pub segments: Vec<DeferredSegment>,
    active_target: u64,
    active_routed: bool,
    active_begin: u32,
    open_screen: u64,
    has_open_screen: bool,
    pub has_ore_marker: bool,
}
impl SessionRouting {
    pub fn new(
        buffer: SharedRenderCommandBuffer,
        canvases: Rc<RefCell<ForeignImageRegistry>>,
    ) -> Self {
        Self {
            buffer,
            canvases,
            content_canvases: HashMap::new(),
            canvas_clear: HashMap::new(),
            segments: Vec::new(),
            active_target: SCREEN_TARGET,
            active_routed: false,
            active_begin: 0,
            open_screen: 0,
            has_open_screen: false,
            has_ore_marker: false,
        }
    }
    fn stream_size(&self) -> u32 {
        self.buffer.lock().unwrap().command_bytes().len() as u32
    }
    fn reopen_unrouted_range(&mut self) {
        self.active_target = screen_target(self.open_screen);
        self.active_routed = self.has_open_screen;
        self.active_begin = self.stream_size();
    }
    fn close_active_range(&mut self) {
        if !self.active_routed {
            return;
        }
        if is_screen_target(self.active_target) {
            if self.stream_size() > self.active_begin {
                self.segments.push(DeferredSegment {
                    target: SegmentTarget::Screen,
                    target_id: screen_target_id(self.active_target),
                    begin: self.active_begin,
                    end: self.stream_size(),
                });
            }
            return;
        }
        let id = self.active_target as u32;
        self.buffer.lock().unwrap().append(
            RenderCmd::CanvasContentEnd,
            &ResIdPod {
                id: id | CANVAS_HANDLE_FLAG,
            },
        );
        self.segments.push(DeferredSegment {
            target: SegmentTarget::Canvas,
            target_id: self.active_target,
            begin: self.active_begin,
            end: self.stream_size(),
        });
    }
    pub fn close_open_range(&mut self) {
        self.close_active_range();
        self.reopen_unrouted_range();
    }
    pub fn end_canvas_content(&mut self) {
        if self.has_open_screen {
            self.route_to(screen_target(self.open_screen));
            return;
        }
        self.close_active_range();
        self.reopen_unrouted_range();
    }
    pub fn register_canvas(&mut self, canvas: RenderCanvasHandle) -> u32 {
        let image = canvas.borrow().render_image();
        let id = self.canvases.borrow_mut().image_draw_id(image.as_ref()) & CANVAS_HANDLE_MASK;
        self.content_canvases.insert(id, canvas);
        id
    }
    pub fn begin_canvas_content(&mut self, canvas: RenderCanvasHandle, clear_color: u32) -> u32 {
        let id = self.register_canvas(canvas);
        self.canvas_clear.insert(u64::from(id), clear_color);
        self.route_to(u64::from(id));
        id
    }
    pub fn scheduler_segments(&self) -> Vec<DeferredSegment> {
        let mut all = self.segments.clone();
        if self.active_routed
            && is_screen_target(self.active_target)
            && self.stream_size() > self.active_begin
        {
            all.push(DeferredSegment {
                target: SegmentTarget::Screen,
                target_id: screen_target_id(self.active_target),
                begin: self.active_begin,
                end: self.stream_size(),
            });
        }
        all
    }
    pub fn reset_frame(&mut self) {
        self.canvases.borrow_mut().reset();
        self.content_canvases.clear();
        self.canvas_clear.clear();
        self.active_target = SCREEN_TARGET;
        self.active_routed = false;
        self.active_begin = 0;
        self.open_screen = 0;
        self.has_open_screen = false;
        self.has_ore_marker = false;
        self.segments.clear();
    }
}
impl DeferredRouteHost for SessionRouting {
    fn route_to(&mut self, target: u64) {
        if self.active_routed && target == self.active_target {
            return;
        }
        self.close_active_range();
        self.active_target = target;
        self.active_routed = true;
        self.active_begin = self.stream_size();
        if is_screen_target(target) {
            self.open_screen = screen_target_id(target);
            self.has_open_screen = true;
            return;
        }
        let clear_color = *self.canvas_clear.entry(target).or_default();
        self.buffer.lock().unwrap().append(
            RenderCmd::CanvasContentBegin,
            &CanvasContentPod {
                canvas_id: target as u32 | CANVAS_HANDLE_FLAG,
                clear_color,
            },
        );
    }
}

#[derive(Default)]
struct SessionTargets {
    next: u64,
    free: Vec<u64>,
    open: Vec<u64>,
}
#[derive(Clone)]
pub struct DeferredSession {
    pub factory: Rc<RefCell<DeferredFactory>>,
    pub ore_context: Rc<RefCell<DeferredOreContext>>,
    pub routing: Rc<RefCell<SessionRouting>>,
    render_context: Rc<RefCell<Option<PersistentFactoryContext>>>,
    canvas_renderers: Rc<RefCell<HashMap<u64, RendererOwner>>>,
    screen_renderers: Rc<RefCell<HashMap<u64, RendererOwner>>>,
    targets: Rc<RefCell<SessionTargets>>,
}
impl DeferredSession {
    pub fn new(real_ore: Option<OreContextHandle>) -> Self {
        Self::with_ore_context(|| DeferredOreContext::fromReal(real_ore))
    }
    pub fn with_caps(caps: ReplayCaps) -> Self {
        Self::with_ore_context(|| DeferredOreContext::new(caps))
    }
    fn with_ore_context(make_ore: impl FnOnce() -> DeferredOreContext) -> Self {
        let factory = Rc::new(RefCell::new(DeferredFactory::new()));
        let canvases = Rc::new(RefCell::new(ForeignImageRegistry::default()));
        let routing = Rc::new(RefCell::new(SessionRouting::new(
            factory.borrow().buffer.clone(),
            canvases.clone(),
        )));
        let ore = make_ore();
        let mut out = Self {
            factory,
            ore_context: Rc::new(RefCell::new(ore)),
            routing,
            render_context: Rc::new(RefCell::new(None)),
            canvas_renderers: Rc::new(RefCell::new(HashMap::new())),
            screen_renderers: Rc::new(RefCell::new(HashMap::new())),
            targets: Rc::new(RefCell::new(SessionTargets::default())),
        };
        out.wire_ore_canvases();
        out
    }
    pub fn bind_real_ore(&mut self, real: Option<OreContextHandle>) {
        self.ore_context.borrow_mut().bindReal(real);
    }
    pub fn bind_replay_caps(&mut self, caps: ReplayCaps) {
        self.ore_context.borrow_mut().bindCaps(caps);
    }
    pub fn bind_render_context(&mut self, real: Option<PersistentFactoryContext>) {
        *self.render_context.borrow_mut() = real;
    }
    pub fn command_buffer(&self) -> SharedRenderCommandBuffer {
        self.factory.borrow().buffer.clone()
    }
    pub fn canvases(&self) -> Rc<RefCell<ForeignImageRegistry>> {
        self.routing.borrow().canvases.clone()
    }
    pub fn make_screen_renderer(&self, target: u64) -> Box<dyn Renderer> {
        Box::new(DeferredRenderer::new(
            self.command_buffer(),
            Some(self.canvases()),
            Some(self.routing.clone()),
            screen_target(target),
        ))
    }
    pub fn screen_renderer(&self, target: u64) -> RendererOwner {
        self.screen_renderers
            .borrow_mut()
            .entry(target)
            .or_insert_with(|| Rc::new(RefCell::new(self.make_screen_renderer(target))))
            .clone()
    }
    pub fn acquire_screen_target(&mut self) -> u64 {
        let mut targets = self.targets.borrow_mut();
        if let Some(id) = targets.free.pop() {
            return id;
        }
        let id = targets.next;
        targets.next = targets.next.wrapping_add(1);
        id
    }
    pub fn release_screen_target(&mut self, target: u64) {
        self.screen_renderers.borrow_mut().remove(&target);
        self.targets.borrow_mut().free.push(target);
    }
    pub fn attached_target_count(&self) -> usize {
        let targets = self.targets.borrow();
        targets.next as usize - targets.free.len()
    }
    pub fn begin_target_frame(&mut self, target: u64) {
        let mut targets = self.targets.borrow_mut();
        if !targets.open.contains(&target) {
            targets.open.push(target);
        }
    }
    pub fn end_target_frame(&mut self, target: u64) -> bool {
        let mut targets = self.targets.borrow_mut();
        if let Some(index) = targets.open.iter().position(|&id| id == target) {
            targets.open.remove(index);
        }
        targets.open.is_empty()
    }
    pub fn abandon_target_frame(&mut self, target: u64) {
        let mut targets = self.targets.borrow_mut();
        if let Some(index) = targets.open.iter().position(|&id| id == target) {
            targets.open.remove(index);
        }
    }
    pub fn close_open_range(&mut self) {
        self.routing.borrow_mut().close_open_range();
    }
    pub fn reset_frame(&mut self) {
        self.factory.borrow_mut().reset_frame();
        self.ore_context.borrow_mut().resetFrame();
        self.routing.borrow_mut().reset_frame();
        self.canvas_renderers.borrow_mut().clear();
    }
    pub fn stream_bytes(&self) -> u64 {
        let buffer = self.command_buffer();
        let buffer = buffer.lock().unwrap();
        let ore = self.ore_context.borrow().stream();
        let ore = ore.borrow();
        (buffer.command_bytes().len()
            + buffer.blob_bytes().len()
            + ore.command_bytes().len()
            + ore.blob_bytes().len()) as u64
    }
    pub fn recorded_this_frame(&self) -> bool {
        self.routing.borrow().has_ore_marker
            || !self.command_buffer().lock().unwrap().empty()
            || !self.ore_context.borrow().stream().borrow().empty()
    }
    pub fn recorded_segments(&self) -> Vec<DeferredSegment> {
        self.routing.borrow().segments.clone()
    }
    pub fn scheduler_segments(&self) -> Vec<DeferredSegment> {
        self.routing.borrow().scheduler_segments()
    }
    pub fn record_ore_replay_marker(&mut self) {
        self.routing.borrow_mut().has_ore_marker = true;
    }
    pub fn content_canvas_at(&self, id: u32) -> Option<RenderCanvasHandle> {
        self.routing.borrow().content_canvases.get(&id).cloned()
    }
    pub fn canvas_image_at(&self, id: u32) -> Option<Rc<dyn RenderImage>> {
        self.canvases().borrow().image_at(id).cloned()
    }
    pub fn content_canvases(&self) -> HashMap<u32, RenderCanvasHandle> {
        self.routing.borrow().content_canvases.clone()
    }
    fn wire_ore_canvases(&mut self) {
        let weak_routing = Rc::downgrade(&self.routing);
        self.ore_context
            .borrow_mut()
            .setCanvasIdProvider(Some(Box::new(move |canvas| {
                weak_routing
                    .upgrade()
                    .expect("live deferred session")
                    .borrow_mut()
                    .register_canvas(canvas)
            })));
        self.factory
            .borrow()
            .buffer
            .lock()
            .unwrap()
            .bind_recording_thread();
        self.ore_context
            .borrow_mut()
            .setCanvasRegistry(Some(self.canvases()));
    }
}
impl DeferredRouteHost for DeferredSession {
    fn route_to(&mut self, target: u64) {
        self.routing.borrow_mut().route_to(target);
    }
}
struct ScopedRenderer(RendererOwner);
impl Renderer for ScopedRenderer {
    fn save(&mut self) {
        self.0.borrow_mut().save();
    }
    fn restore(&mut self) {
        self.0.borrow_mut().restore();
    }
    fn transform(&mut self, t: Mat2D) {
        self.0.borrow_mut().transform(t);
    }
    fn draw_path(&mut self, p: &dyn RenderPath, paint: &dyn RenderPaint) {
        self.0.borrow_mut().draw_path(p, paint);
    }
    fn clip_path(&mut self, p: &dyn RenderPath) {
        self.0.borrow_mut().clip_path(p);
    }
    fn draw_image(&mut self, i: Option<&dyn RenderImage>, s: ImageSampler, b: BlendMode, o: f32) {
        self.0.borrow_mut().draw_image(i, s, b, o);
    }
    fn draw_image_mesh(
        &mut self,
        i: Option<&dyn RenderImage>,
        s: ImageSampler,
        v: Option<&dyn RenderBuffer>,
        uv: Option<&dyn RenderBuffer>,
        indices: Option<&dyn RenderBuffer>,
        vc: u32,
        ic: u32,
        b: BlendMode,
        o: f32,
    ) {
        self.0
            .borrow_mut()
            .draw_image_mesh(i, s, v, uv, indices, vc, ic, b, o);
    }
    fn modulate_opacity(&mut self, o: f32) {
        self.0.borrow_mut().modulate_opacity(o);
    }
}
impl DeferredCanvasHost for DeferredSession {
    fn begin_canvas_content(
        &mut self,
        canvas: RenderCanvasHandle,
        clear_color: ColorInt,
    ) -> Option<Box<dyn Renderer>> {
        let id = self
            .routing
            .borrow_mut()
            .begin_canvas_content(canvas, clear_color);
        let renderer = self
            .canvas_renderers
            .borrow_mut()
            .entry(u64::from(id))
            .or_insert_with(|| {
                Rc::new(RefCell::new(Box::new(DeferredRenderer::new(
                    self.command_buffer(),
                    Some(self.canvases()),
                    Some(self.routing.clone()),
                    u64::from(id),
                )) as Box<dyn Renderer>))
            })
            .clone();
        Some(Box::new(ScopedRenderer(renderer)))
    }
    fn end_canvas_content(&mut self, _canvas: &RenderCanvasHandle) {
        self.routing.borrow_mut().end_canvas_content();
    }
}
impl Factory for DeferredSession {
    fn ore(&mut self) -> Option<OreContextHandle> {
        Some(self.ore_context.clone())
    }
    fn render_context(&mut self) -> Option<PersistentFactoryContext> {
        self.render_context.borrow().clone()
    }
    fn deferred_canvas_host(&mut self) -> Option<DeferredCanvasHostHandle> {
        Some(Rc::new(RefCell::new(self.clone())))
    }
    fn gpu_canvas_shader_profile(&self) -> GpuCanvasShaderProfile {
        profile_for_target(self.ore_context.borrow().shaderTarget())
    }
    fn make_gpu_canvas_shader(
        &mut self,
        shader: &GpuCanvasShader,
    ) -> Result<std::sync::Arc<dyn RenderGpuCanvasShader>, GpuCanvasError> {
        self.make_gpu_canvas_shader_artifact(&GpuCanvasShaderArtifact::WebGpu(shader.clone()))
    }
    fn make_gpu_canvas_shader_artifact(
        &mut self,
        artifact: &GpuCanvasShaderArtifact,
    ) -> Result<std::sync::Arc<dyn RenderGpuCanvasShader>, GpuCanvasError> {
        let profile = self.gpu_canvas_shader_profile();
        // Only the recording context creates these modules. A bound real
        // device supplies capabilities/target, never eager driver creation.
        let occurrence = ExactGpuCanvasShaderOccurrence::compile(
            &mut *self.ore_context.borrow_mut(),
            profile,
            artifact,
            self.ore_context.clone(),
        )?;
        #[allow(clippy::arc_with_non_send_sync)]
        Ok(std::sync::Arc::new(occurrence))
    }
    fn make_gpu_canvas_shader_occurrence(
        &mut self,
        prepared: &std::sync::Arc<dyn RenderGpuCanvasShader>,
    ) -> Result<std::sync::Arc<dyn RenderGpuCanvasShader>, GpuCanvasError> {
        let prepared = prepared
            .as_any()
            .downcast_ref::<ExactGpuCanvasShaderOccurrence>()
            .ok_or_else(|| GpuCanvasError::new("prepared shader belongs to another renderer"))?;
        let anchor: Rc<dyn std::any::Any> = self.ore_context.clone();
        if !Rc::ptr_eq(&prepared.execution_anchor, &anchor) {
            return Err(GpuCanvasError::new(
                "prepared shader belongs to another recording context",
            ));
        }
        self.make_gpu_canvas_shader_artifact(&prepared.artifact)
    }
    fn make_render_buffer(
        &mut self,
        t: RenderBufferType,
        f: RenderBufferFlags,
        size: usize,
    ) -> Box<dyn RenderBuffer> {
        self.factory.borrow_mut().make_render_buffer(t, f, size)
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
        self.factory
            .borrow_mut()
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
        self.factory
            .borrow_mut()
            .make_radial_gradient(cx, cy, radius, colors, stops)
    }
    fn make_render_path(&mut self, path: RawPath, rule: FillRule) -> Box<dyn RenderPath> {
        self.factory.borrow_mut().make_render_path(path, rule)
    }
    fn make_empty_render_path(&mut self) -> Box<dyn RenderPath> {
        self.factory.borrow_mut().make_empty_render_path()
    }
    fn make_render_paint(&mut self) -> Box<dyn RenderPaint> {
        self.factory.borrow_mut().make_render_paint()
    }
    fn decode_image(&mut self, data: &[u8]) -> Result<Box<dyn RenderImage>, ImageDecodeError> {
        self.factory.borrow_mut().decode_image(data)
    }
}
