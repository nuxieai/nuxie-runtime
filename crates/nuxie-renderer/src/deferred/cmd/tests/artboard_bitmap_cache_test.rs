//! tests/unit_tests/runtime/artboard_bitmap_cache_test.cpp at a4dbc3ff.
//!
//! Cache-as-bitmap validation for an artboard carrying a BitmapCache child.
//! The feature only engages on the deferred/recording path (a factory that
//! hands back both a render context and a canvas content host), and only
//! through Artboard::draw_internal -- Artboard::draw() is the standalone-root
//! path and deliberately bypasses the cache. So this harness records into a
//! DeferredSession bound to a GPU-free null render context and draws through
//! draw_internal, which is what a nested/instanced draw reaches.
//!
//! Upstream's CountingSession subclasses DeferredSession to log each
//! beginCanvasContent. The Rust session is not subclassable, so the harness
//! reads the same facts off the recorded frame: its content canvases (one per
//! offscreen bracket) and its canvas segments (the bracket's stream bytes).
//!
//! Not ported: "a silver factory drives the cache only once enabled", which
//! exercises SerializingFactory::enableBitmapCache. The serializing utility
//! half of a4dbc3ff depends on the unported paintModulatedImage op and is out
//! of scope for this out-of-order port; see docs/cache-as-bitmap-port.md.
use super::super::{
    command_stream::CommandReader,
    deferred_replayer::*,
    deferred_session::{DeferredSession, SegmentTarget},
    render_commands::{payload_size_of, RenderCmd, TransformPod},
};
use super::render_context_null::*;
use super::*;
use nuxie_runtime::mechanical_port::source::{
    generated::{
        artboard_base::ArtboardBase, bitmap_cache_base::BitmapCacheBase,
        core_registry::CoreRegistry, node_base::NodeBase,
        transform_component_base::TransformComponentBase,
    },
    math::{mat2d::Mat2D as M, vec2d::Vec2D},
};
use nuxie_runtime::{CoreHandle, RuntimeArtboardInstanceHandle};

fn fixture() -> Vec<u8> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../fixtures/sync/cache_as_bitmap_test.riv");
    std::fs::read(&path).unwrap_or_else(|_| panic!("run `make fixtures`: {}", path.display()))
}

// The artboard carrying the BitmapCache, found rather than named so the tests
// do not encode which artboard the fixture happens to put it on. The file's
// default artboard is the root that nests this one.
fn cached_artboard_of(
    file: &nuxie_runtime::RuntimeFileHandle,
) -> Option<RuntimeArtboardInstanceHandle> {
    let count = file.with_file(|f| f.artboard_count());
    (0..count)
        .filter_map(|i| file.with_file(|f| f.artboard_at(i)))
        .find(|artboard| artboard.with_artboard(|a| a.bitmap_cache().is_some()))
}

// A node inside the cached artboard, used to force a content change. The
// fixture's state machine settles within a couple of frames, so any test that
// needs a *changed* frame has to make the change itself.
fn first_node_of(artboard: &RuntimeArtboardInstanceHandle) -> Option<CoreHandle> {
    artboard.with_artboard(|a| {
        a.objects()
            .iter()
            .flatten()
            .find(|object| object.is_type_of(NodeBase::TYPE_KEY))
            .cloned()
    })
}

fn node_x(node: &CoreHandle) -> f32 {
    CoreRegistry::get_double_handle(node, i32::from(NodeBase::X_PROPERTY_KEY)).unwrap()
}

fn set_node_x(node: &CoreHandle, value: f32) {
    CoreRegistry::set_double_handle(node, i32::from(NodeBase::X_PROPERTY_KEY), value);
}

// Mirrors the clamps in Artboard::draw_cached_as_bitmap. Duplicated on
// purpose: the test states the contract independently of the implementation.
const MIN_RES: f32 = 0.01;
const MAX_RES: f32 = 8.0;
const MAX_DIM: u32 = 2048;

// std::min/std::max argument order, so a NaN behaves as it does upstream.
fn cpp_min(a: f32, b: f32) -> f32 {
    if b < a {
        b
    } else {
        a
    }
}
fn cpp_max(a: f32, b: f32) -> f32 {
    if a < b {
        b
    } else {
        a
    }
}

fn expected_dim(natural: f32, resolution: f32) -> u32 {
    let res = cpp_min(cpp_max(resolution, MIN_RES), MAX_RES);
    cpp_min(cpp_max((natural * res).ceil(), 1.0), MAX_DIM as f32) as u32
}

// Optional: nothing here drives the cache through data binding, so a fixture
// that exposes no view model is fine. Mirrors upstream's
// `viewModelId == -1 ? createViewModelInstance(artboard)
//                    : createViewModelInstance(viewModelId, 0)`.
fn bind_view_model(
    file: &nuxie_runtime::RuntimeFileHandle,
    artboard: &RuntimeArtboardInstanceHandle,
    machine: &nuxie_runtime::RuntimeStateMachineInstanceHandle,
) {
    let view_model_id = artboard.with_artboard(|a| a.base.view_model_id());
    let model = if view_model_id == u32::MAX {
        file.with_file(|f| f.create_view_model_instance_for_artboard(artboard.core_handle()))
    } else {
        file.with_file(|f| f.create_view_model_instance_at(view_model_id as usize, 0))
    };
    if let Some(model) = model {
        artboard.bind_view_model_instance(Some(model.clone()));
        machine.with_instance_mut(|m| m.bind_view_model_instance(model));
    }
}

#[derive(Clone, Copy, Debug)]
struct Raster {
    width: u32,
    height: u32,
}
impl Raster {
    fn pixels(self) -> u64 {
        u64::from(self.width) * u64::from(self.height)
    }
}

const NUM_CMDS: usize = RenderCmd::ResourceNewVersion as usize + 1;

// Per-frame opcode tally over the recorded 2D stream.
#[derive(Clone, Copy)]
struct Census {
    count: [u64; NUM_CMDS],
    command_bytes: u64,
}
impl Census {
    fn draw_ops(&self) -> u64 {
        self.draw_paths() + self.draw_images() + self.count[RenderCmd::DrawImageMesh as usize]
    }
    fn draw_paths(&self) -> u64 {
        self.count[RenderCmd::DrawPath as usize]
    }
    fn draw_images(&self) -> u64 {
        self.count[RenderCmd::DrawImage as usize]
    }
    fn canvas_opens(&self) -> u64 {
        self.count[RenderCmd::CanvasContentBegin as usize]
    }
    fn of(commands: &[u8], blobs: &[u8]) -> Self {
        let mut census = Census {
            count: [0; NUM_CMDS],
            command_bytes: commands.len() as u64,
        };
        let mut reader = CommandReader::new(commands, blobs);
        while let Some(byte) = reader.next_u8() {
            let Some(command) = RenderCmd::from_byte(byte) else {
                break;
            };
            census.count[command as usize] += 1;
            reader.skip(payload_size_of(command));
        }
        census
    }
}

// Placement slack: pixel snapping moves the composite by up to half a device
// pixel, and the ceil()ed raster can overhang the box by up to one texel.
const SNAP_SLACK: f32 = 0.5;

fn texel_span(origin: Vec2D, corner: Vec2D, raster_dim: u32) -> f32 {
    (corner.x - origin.x).abs() / raster_dim as f32
}

fn approx(actual: f32, expected: f32, margin: f32) -> bool {
    (actual - expected).abs() <= margin
}

fn mat(m: M) -> Mat2D {
    Mat2D(*m.values())
}

fn from_scale_and_translation(sx: f32, sy: f32, tx: f32, ty: f32) -> M {
    M::new(sx, 0.0, 0.0, sy, tx, ty)
}

// Walks a recorded frame's screen segments, tracking the CTM, and returns the
// transform in force at the composite (the single drawImage the cache emits).
fn composite_transform(frame: &DeferredFrame) -> Option<M> {
    let mut found = None;
    // One stack across every screen segment: replay feeds them all to the
    // same renderer in record order.
    let mut stack = vec![M::identity()];
    for segment in &frame.segments {
        if segment.target != SegmentTarget::Screen {
            continue;
        }
        let commands = &frame.commands[segment.begin as usize..segment.end as usize];
        let mut reader = CommandReader::new(commands, &frame.blobs);
        while let Some(byte) = reader.next_u8() {
            let Some(command) = RenderCmd::from_byte(byte) else {
                break;
            };
            match command {
                RenderCmd::Save => {
                    let top = *stack.last().unwrap();
                    stack.push(top);
                }
                RenderCmd::Restore => {
                    if stack.len() > 1 {
                        stack.pop();
                    }
                }
                RenderCmd::Transform => {
                    let t: TransformPod = reader.read();
                    let top = stack.last_mut().unwrap();
                    *top = *top * M::new(t.xx, t.xy, t.yx, t.yy, t.tx, t.ty);
                }
                RenderCmd::DrawImage => {
                    reader.skip(payload_size_of(command));
                    found = Some(*stack.last().unwrap());
                }
                _ => reader.skip(payload_size_of(command)),
            }
        }
    }
    found
}

// Replays a recorded frame for real: opens the offscreen canvas frames on the
// observed null context and flushes each one, then the screen frame.
struct ReplaySink {
    ctx: ObservingFactory,
    width: u32,
    height: u32,
    screen: Option<RendererOwner>,
    canvas: Option<RendererOwner>,
}
impl ReplaySink {
    fn new(ctx: ObservingFactory, width: u32, height: u32) -> Self {
        Self {
            ctx,
            width,
            height,
            screen: None,
            canvas: None,
        }
    }
    fn flush_screen(&mut self) {
        if self.screen.take().is_some() {
            self.ctx
                .borrow()
                .with_backend_mut_for_test(NullBackend::flush);
        }
    }
    fn open_frame(&mut self, width: u32, height: u32) -> RendererOwner {
        self.ctx.borrow().resize(width, height).unwrap();
        Rc::new(RefCell::new(Box::new(
            self.ctx
                .borrow()
                .begin_frame(0, crate::RenderMode::RasterOrdering)
                .unwrap(),
        )))
    }
}
impl DeferredFrameSink for ReplaySink {
    fn factory(&mut self) -> PersistentFactoryContext {
        self.ctx.persistent_context().unwrap()
    }
    fn render_context(&mut self) -> Option<PersistentFactoryContext> {
        self.ctx.persistent_context()
    }
    fn ore_context(&mut self) -> Option<OreContextHandle> {
        None
    }
    fn begin_screen_frame(&mut self, _target: u64) -> Option<RendererOwner> {
        let screen = self.open_frame(self.width, self.height);
        self.screen = Some(screen.clone());
        Some(screen)
    }
    fn begin_canvas_content(
        &mut self,
        canvas: RenderCanvasHandle,
        _clear_color: u32,
    ) -> Option<RendererOwner> {
        let (width, height) = {
            let canvas = canvas.borrow();
            (canvas.width(), canvas.height())
        };
        let renderer = self.open_frame(width, height);
        self.canvas = Some(renderer.clone());
        Some(renderer)
    }
    fn end_canvas_content(&mut self) {
        self.ctx
            .borrow()
            .with_backend_mut_for_test(NullBackend::flush);
        self.canvas = None;
    }
}

struct Observed {
    sink: ReplaySink,
    stats: Rc<RefCell<FlushStats>>,
    features: Rc<Cell<u32>>,
}
impl Observed {
    fn new(width: f32, height: f32) -> Self {
        let (ctx, stats, features) = observing_factory(1, 1);
        Self {
            sink: ReplaySink::new(ctx, width.ceil() as u32, height.ceil() as u32),
            stats,
            features,
        }
    }
    fn work(&self) -> FlushStats {
        *self.stats.borrow()
    }
}

// One artboard + the session recording it, wired for cache-as-bitmap.
struct Harness {
    session: PersistentFactory<DeferredSession>,
    _render_context: ObservingFactory,
    _file: nuxie_runtime::RuntimeFileHandle,
    artboard: RuntimeArtboardInstanceHandle,
    machine: nuxie_runtime::RuntimeStateMachineInstanceHandle,
    rasters: Vec<Raster>,
    raster_content_bytes: Vec<u64>,
    last_frame: DeferredFrame,
    replayer: DeferredReplayer,
    observed: Option<Observed>,
}
impl Harness {
    fn new() -> Self {
        Self::with_cache(true)
    }
    fn with_cache(cached: bool) -> Self {
        let (render_context, _, _) = observing_factory(1, 1);
        let mut session = PersistentFactory::new(DeferredSession::with_caps(Default::default()));
        // The artboard reads this off the factory to allocate the offscreen
        // canvas; without it draw_cached_as_bitmap falls back to a vector draw.
        session.borrow_mut().bind_render_context(if cached {
            render_context.persistent_context()
        } else {
            None
        });
        let file = nuxie_runtime::File::import(
            &fixture(),
            nuxie_runtime::RuntimeFactoryHandle::from_factory(&mut session).unwrap(),
            None,
            None,
            None,
        )
        .expect("fixture imports");
        let artboard = cached_artboard_of(&file).expect("cached artboard");
        let machine = artboard.state_machine_at(0).expect("state machine");
        bind_view_model(&file, &artboard, &machine);
        Self {
            session,
            _render_context: render_context,
            _file: file,
            artboard,
            machine,
            rasters: Vec::new(),
            raster_content_bytes: Vec::new(),
            last_frame: DeferredFrame::default(),
            replayer: DeferredReplayer::default(),
            observed: None,
        }
    }
    fn observe(&mut self) {
        let (w, h) = (self.width(), self.height());
        self.observed = Some(Observed::new(w, h));
    }
    fn cache(&self) -> CoreHandle {
        self.artboard
            .with_artboard(|a| a.bitmap_cache())
            .expect("bitmap cache")
    }
    fn width(&self) -> f32 {
        self.artboard.with_artboard(|a| a.width())
    }
    fn height(&self) -> f32 {
        self.artboard.with_artboard(|a| a.height())
    }
    fn did_change(&self) -> bool {
        self.artboard.with_artboard(|a| a.did_change())
    }
    fn has_self_transform(&self) -> bool {
        self.artboard.with_artboard(|a| a.has_self_transform())
    }
    fn set_artboard_double(&self, key: u16, value: f32) {
        CoreRegistry::set_double_handle(&self.artboard.core_handle(), i32::from(key), value);
    }
    // Written straight onto the core object rather than through a bound view
    // model property: what these tests are about is what the *cache* does
    // when resolution changes.
    fn set_resolution(&self, value: f32) {
        CoreRegistry::set_double_handle(
            &self.cache(),
            i32::from(BitmapCacheBase::RESOLUTION_PROPERTY_KEY),
            value,
        );
    }
    fn set_cache_enabled(&self, value: bool) {
        CoreRegistry::set_bool_handle(
            &self.cache(),
            i32::from(BitmapCacheBase::CACHE_ENABLED_PROPERTY_KEY),
            value,
        );
    }
    fn cache_enabled(&self) -> bool {
        CoreRegistry::get_bool_handle(
            &self.cache(),
            i32::from(BitmapCacheBase::CACHE_ENABLED_PROPERTY_KEY),
        )
        .unwrap()
    }
    fn dither(&self) -> bool {
        CoreRegistry::get_bool_handle(
            &self.cache(),
            i32::from(BitmapCacheBase::DITHER_PROPERTY_KEY),
        )
        .unwrap()
    }

    // Advance (optionally by zero, which is how a "nothing moved" frame is
    // expressed) and record one frame through the cache-aware draw path.
    fn frame(&mut self, dt: f32) -> Census {
        self.frame_under(dt, None)
    }
    fn frame_under(&mut self, dt: f32, outer: Option<M>) -> Census {
        self.machine.advance_and_apply(dt);
        let screen = self.session.borrow().screen_renderer(0);
        {
            let mut screen = screen.borrow_mut();
            screen.save();
            if let Some(outer) = outer {
                screen.transform(mat(outer));
            }
            self.artboard.draw_internal(screen.as_mut());
            screen.restore();
        }
        let frame = snapshot_frame(&mut self.session.borrow_mut());
        let census = Census::of(&frame.commands, &frame.blobs);
        for canvas in frame.content_canvases.values() {
            let canvas = canvas.borrow();
            self.rasters.push(Raster {
                width: canvas.width(),
                height: canvas.height(),
            });
        }
        for segment in &frame.segments {
            if segment.target == SegmentTarget::Canvas {
                self.raster_content_bytes
                    .push(u64::from(segment.end - segment.begin));
            }
        }
        self.session.borrow_mut().reset_frame();
        if let Some(observed) = self.observed.as_mut() {
            self.replayer.replay_frame(&frame, &mut observed.sink);
            observed.sink.flush_screen();
            // A cached frame composites a canvas rasterized in an earlier
            // frame; it must still resolve on the device, not be dropped.
            assert_eq!(self.replayer.dropped_draws(), 0, "replay dropped a draw");
        }
        self.last_frame = frame;
        census
    }

    fn raster_count(&self) -> usize {
        self.rasters.len()
    }
    fn last_raster(&self) -> Raster {
        *self.rasters.last().expect("a raster")
    }
}

#[test]
fn the_artboard_imports_its_bitmap_cache_child() {
    let h = Harness::new();
    let resolution = CoreRegistry::get_double_handle(
        &h.cache(),
        i32::from(BitmapCacheBase::RESOLUTION_PROPERTY_KEY),
    )
    .unwrap();
    // Whatever the file authored, so long as it is usable.
    assert!(resolution > 0.0);
    println!(
        "[bitmap-cache] artboard: {} x {}, authored resolution {}",
        h.width(),
        h.height(),
        resolution
    );
}

#[test]
fn a_cached_artboard_rasterizes_offscreen_and_composites_one_image() {
    let mut h = Harness::new();
    h.set_resolution(1.0);
    let first = h.frame(0.0);

    // The content went into an offscreen canvas...
    assert_eq!(h.raster_count(), 1);
    assert_eq!(first.canvas_opens(), 1);
    // ...and the screen got an image composite for it.
    assert!(first.draw_images() >= 1);

    // At resolution 1 the raster is the artboard's natural pixel size.
    let raster = h.last_raster();
    assert_eq!(raster.width, expected_dim(h.width(), 1.0));
    assert_eq!(raster.height, expected_dim(h.height(), 1.0));

    // The same artboard drawn uncached records its vector content straight to
    // the screen: no canvas, no composite image standing in for the content.
    let mut uncached = Harness::with_cache(false);
    let plain = uncached.frame(0.0);
    assert_eq!(uncached.raster_count(), 0);
    assert_eq!(plain.canvas_opens(), 0);
    assert!(plain.draw_paths() > 0);
}

#[test]
fn cache_enabled_off_falls_back_to_a_vector_draw() {
    let mut h = Harness::new();
    h.set_resolution(1.0);
    // Bit 0 of the mask is on by default.
    assert!(h.cache_enabled());
    assert_eq!(h.frame(0.0).canvas_opens(), 1);
    assert_eq!(h.raster_count(), 1);

    // Written through the passthrough key, which is the path a keyframe or a
    // data bind takes: CoreRegistry rewrites bit 0 and calls the mask setter.
    h.set_cache_enabled(false);
    assert!(!h.cache_enabled());
    // The sibling bit is untouched by a write to this one.
    assert!(!h.dither());

    let off = h.frame(0.0);
    assert_eq!(h.raster_count(), 1); // no new raster
    assert_eq!(off.canvas_opens(), 0);
    assert_eq!(off.draw_images(), 0);
    assert!(off.draw_paths() > 0);

    // Matches the uncached control exactly.
    let mut uncached = Harness::with_cache(false);
    let plain = uncached.frame(0.0);
    assert_eq!(off.draw_paths(), plain.draw_paths());

    // Turning it back on rasterizes afresh -- the disable released the
    // texture, so there is nothing left to reuse.
    h.set_cache_enabled(true);
    assert!(h.cache_enabled());
    let back = h.frame(0.0);
    assert_eq!(h.raster_count(), 2);
    assert_eq!(back.canvas_opens(), 1);
    assert!(back.draw_images() >= 1);
}

#[test]
fn a_rotated_or_scaled_artboard_falls_back_to_a_vector_draw() {
    let mut h = Harness::new();
    h.set_resolution(1.0);
    // Baseline: an identity self transform caches as usual.
    assert_eq!(h.frame(0.0).canvas_opens(), 1);
    assert_eq!(h.raster_count(), 1);

    h.set_artboard_double(TransformComponentBase::ROTATION_PROPERTY_KEY, 0.4);
    assert!(h.has_self_transform());
    let rotated = h.frame(0.0);
    assert_eq!(h.raster_count(), 1); // no new raster
    assert_eq!(rotated.canvas_opens(), 0);
    assert_eq!(rotated.draw_images(), 0);
    assert!(rotated.draw_paths() > 0);

    // Scale on its own is enough; rotation is not the only way in.
    h.set_artboard_double(TransformComponentBase::ROTATION_PROPERTY_KEY, 0.0);
    h.set_artboard_double(TransformComponentBase::SCALE_X_PROPERTY_KEY, 2.0);
    assert!(h.has_self_transform());
    let scaled = h.frame(0.0);
    assert_eq!(h.raster_count(), 1);
    assert_eq!(scaled.canvas_opens(), 0);
    assert_eq!(scaled.draw_images(), 0);
    assert!(scaled.draw_paths() > 0);

    // The same artboard drawn uncached records the same vector content.
    let mut uncached = Harness::with_cache(false);
    uncached.set_artboard_double(TransformComponentBase::SCALE_X_PROPERTY_KEY, 2.0);
    assert_eq!(scaled.draw_paths(), uncached.frame(0.0).draw_paths());

    // Back at identity the cache engages again, rasterizing afresh because
    // changing the transform marks the artboard changed.
    h.set_artboard_double(TransformComponentBase::SCALE_X_PROPERTY_KEY, 1.0);
    assert!(!h.has_self_transform());
    let identity = h.frame(0.0);
    assert_eq!(h.raster_count(), 2);
    assert_eq!(identity.canvas_opens(), 1);
    assert!(identity.draw_images() >= 1);

    // ...and once it settles, back to one composite and no vector content.
    let settled = h.frame(0.0);
    assert_eq!(h.raster_count(), 2);
    assert_eq!(settled.canvas_opens(), 0);
    assert_eq!(settled.draw_paths(), 0);
    assert_eq!(settled.draw_ops(), 1);
}

#[test]
fn the_flag_bits_are_independent_within_the_mask() {
    let h = Harness::new();
    // dither defaults off, cacheEnabled defaults on: mask initial value is 1.
    assert!(h.cache_enabled());
    assert!(!h.dither());

    CoreRegistry::set_bool_handle(
        &h.cache(),
        i32::from(BitmapCacheBase::DITHER_PROPERTY_KEY),
        true,
    );
    assert!(h.dither());
    assert!(h.cache_enabled()); // untouched

    h.set_cache_enabled(false);
    assert!(!h.cache_enabled());
    assert!(h.dither()); // untouched

    // Both bits live in the one serialized property.
    assert_eq!(
        CoreRegistry::get_uint_handle(
            &h.cache(),
            i32::from(BitmapCacheBase::CACHE_FLAGS_PROPERTY_KEY)
        ),
        Some(BitmapCacheBase::DITHER_BITMASK)
    );
}

#[test]
fn a_static_frame_reuses_the_raster_instead_of_re_rasterizing() {
    let mut h = Harness::new();
    h.set_resolution(1.0);
    let first = h.frame(0.0);
    assert_eq!(h.raster_count(), 1);

    // Nothing changed: the second frame must hit the cache.
    let second = h.frame(0.0);
    assert_eq!(h.raster_count(), 1);
    assert_eq!(second.canvas_opens(), 0);
    assert_eq!(second.draw_images(), 1);
    // The whole artboard now costs one draw.
    assert_eq!(second.draw_paths(), 0);
    assert_eq!(second.draw_ops(), 1);

    // And it stays that way.
    for _ in 0..8 {
        assert_eq!(h.frame(0.0).draw_ops(), 1);
    }
    assert_eq!(h.raster_count(), 1);

    let mut uncached = Harness::with_cache(false);
    uncached.frame(0.0);
    let plain = uncached.frame(0.0);
    println!(
        "[bitmap-cache] steady frame: cached draws={} bytes={} vs uncached draws={} bytes={}",
        second.draw_ops(),
        second.command_bytes,
        plain.draw_ops(),
        plain.command_bytes
    );

    // The point of the feature: a steady frame issues strictly less work.
    assert!(second.draw_ops() < plain.draw_ops());
    assert!(second.command_bytes < plain.command_bytes);
    assert!(first.draw_ops() >= second.draw_ops());
}

#[test]
fn a_cache_property_change_marks_the_artboard_changed() {
    let mut h = Harness::new();
    h.set_resolution(1.0);
    h.frame(0.0);
    assert_eq!(h.raster_count(), 1);
    // A static frame settles the artboard: nothing moved, nothing changed.
    h.frame(0.0);
    assert!(!h.did_change());

    h.set_resolution(2.0);
    assert!(h.did_change());
    let resized = h.frame(0.0);
    assert_eq!(h.raster_count(), 2);
    assert_eq!(resized.canvas_opens(), 1);

    // And the same for a flag write, which arrives by the same route a
    // keyframe or a data bind would take.
    h.frame(0.0);
    assert!(!h.did_change());
    h.set_cache_enabled(false);
    assert!(h.did_change());
}

#[test]
fn a_transparent_cached_artboard_still_consumes_its_change() {
    let mut h = Harness::new();
    h.set_resolution(1.0);
    h.frame(0.0);
    assert_eq!(h.raster_count(), 1);
    h.frame(0.0);
    assert!(!h.did_change());

    let node = first_node_of(&h.artboard).expect("a node");

    // Invisible *and* genuinely changed: the case where the flag would leak.
    h.artboard.with_artboard_mut(|a| a.set_host_opacity(0.0));
    set_node_x(&node, node_x(&node) + 10.0);
    assert!(h.did_change());

    let hidden = h.frame(0.0);
    // Nothing drawn and nothing rasterized...
    assert_eq!(hidden.canvas_opens(), 0);
    assert_eq!(h.raster_count(), 1);
    // ...but the change is consumed, so the host settles instead of spinning.
    assert!(!h.did_change());

    // And it stays settled while it remains transparent.
    h.frame(0.0);
    assert!(!h.did_change());

    // The frame that brings it back re-rasterizes.
    h.artboard.with_artboard_mut(|a| a.set_host_opacity(1.0));
    let shown = h.frame(0.0);
    assert_eq!(shown.canvas_opens(), 1);
    assert_eq!(h.raster_count(), 2);
}

#[test]
fn an_artboard_re_rasterizes_every_frame_its_content_changes() {
    let mut h = Harness::new();
    h.set_resolution(0.5);
    let node = first_node_of(&h.artboard).expect("a node");

    h.frame(0.0);
    assert_eq!(h.raster_count(), 1);

    const FRAMES: usize = 30;
    for _ in 0..FRAMES {
        // Moves the content, which marks the artboard changed.
        set_node_x(&node, node_x(&node) + 1.0);
        h.frame(1.0 / 60.0);
    }
    // One per changed frame, plus the opening one. Anything less means a
    // changed artboard composited a stale texture.
    assert_eq!(h.raster_count(), FRAMES + 1);
}

#[test]
fn writing_resolution_re_rasterizes_at_the_new_size() {
    let mut h = Harness::new();
    let (w, ht) = (h.width(), h.height());

    h.set_resolution(1.0);
    h.frame(0.0);
    assert_eq!(h.raster_count(), 1);
    let at_one = h.last_raster();
    assert_eq!(at_one.width, expected_dim(w, 1.0));
    assert_eq!(at_one.height, expected_dim(ht, 1.0));

    // A static frame in between proves the next re-raster is the property
    // write's doing and not just frame churn.
    h.frame(0.0);
    assert_eq!(h.raster_count(), 1);

    h.set_resolution(2.0);
    let bumped = h.frame(0.0);
    assert_eq!(h.raster_count(), 2);
    assert_eq!(bumped.canvas_opens(), 1);
    let at_two = h.last_raster();
    assert_eq!(at_two.width, expected_dim(w, 2.0));
    assert_eq!(at_two.height, expected_dim(ht, 2.0));
    assert!(at_two.pixels() > at_one.pixels());

    h.set_resolution(0.5);
    h.frame(0.0);
    assert_eq!(h.raster_count(), 3);
    let at_half = h.last_raster();
    assert_eq!(at_half.width, expected_dim(w, 0.5));
    assert_eq!(at_half.height, expected_dim(ht, 0.5));
    assert!(at_half.pixels() < at_one.pixels());
}

#[test]
fn the_composite_lands_exactly_where_the_vector_content_would() {
    let outer = from_scale_and_translation(0.75, 0.75, 40.0, -12.0);
    for resolution in [0.1, 0.5, 1.0, 2.0, 8.0] {
        let mut h = Harness::new();
        h.set_resolution(resolution);
        h.frame_under(0.0, Some(outer));
        assert_eq!(h.raster_count(), 1);
        let raster = h.last_raster();
        let ctm = composite_transform(&h.last_frame).expect("a composite");

        // The image spans [0,0] .. [width_px,height_px] in its local space.
        let origin = ctm * Vec2D::new(0.0, 0.0);
        let corner = ctm * Vec2D::new(raster.width as f32, raster.height as f32);
        // Which must land on the artboard's own box under the same outer
        // transform the vector draw would have used.
        let want_origin = outer * Vec2D::new(0.0, 0.0);
        let want_corner = outer * Vec2D::new(h.width(), h.height());

        let texel_x = texel_span(origin, corner, raster.width);
        let texel_y = (corner.y - origin.y).abs() / raster.height as f32;

        // The origin only ever moves by the snap.
        assert!(
            approx(origin.x, want_origin.x, SNAP_SLACK),
            "res {resolution}"
        );
        assert!(
            approx(origin.y, want_origin.y, SNAP_SLACK),
            "res {resolution}"
        );
        // The far corner may overhang by up to one texel, but must never fall
        // short of the box by more than the snap.
        assert!(corner.x >= want_corner.x - SNAP_SLACK, "res {resolution}");
        assert!(corner.y >= want_corner.y - SNAP_SLACK, "res {resolution}");
        assert!(
            corner.x <= want_corner.x + SNAP_SLACK + texel_x,
            "res {resolution}"
        );
        assert!(
            corner.y <= want_corner.y + SNAP_SLACK + texel_y,
            "res {resolution}"
        );
    }
}

#[test]
fn a_lower_resolution_costs_proportionally_less_raster_work() {
    // Every step here stays under MAX_DIM for this artboard, so the ratios are
    // the resolution's doing and not the clamp's.
    let mut samples = Vec::new();
    for resolution in [0.125, 0.25, 0.5, 1.0] {
        let mut h = Harness::new();
        assert!(expected_dim(h.width(), resolution) < MAX_DIM);
        assert!(expected_dim(h.height(), resolution) < MAX_DIM);
        h.set_resolution(resolution);
        h.frame(0.0);
        assert_eq!(h.raster_count(), 1);
        let pixels = h.last_raster().pixels();
        samples.push((pixels, pixels * 4 /* RGBA8 */));
    }
    // Monotonic, and each doubling of resolution is ~4x the pixels.
    for pair in samples.windows(2) {
        assert!(pair[1].0 > pair[0].0);
        assert!(pair[1].1 > pair[0].1);
        let ratio = pair[1].0 as f64 / pair[0].0 as f64;
        assert!((ratio - 4.0).abs() <= 4.0 * 0.05, "ratio {ratio}");
    }
    // 1/8 resolution fills ~64x fewer pixels than native.
    assert!(samples.first().unwrap().0 * 60 <= samples.last().unwrap().0);
}

#[test]
fn a_lower_resolution_costs_less_real_gpu_work() {
    // Baseline: the same artboard with the cache never engaging.
    let uncached_frame = {
        let mut u = Harness::with_cache(false);
        u.observe();
        u.frame(0.0);
        let first = u.observed.as_ref().unwrap().work();
        u.frame(0.0);
        let work = u.observed.as_ref().unwrap().work() - first;
        assert_eq!(u.raster_count(), 0);
        work
    };

    let mut samples = Vec::new();
    for resolution in [0.25, 0.5, 1.0] {
        let mut h = Harness::new();
        h.observe();
        h.set_resolution(resolution);
        h.frame(0.0); // the rasterizing frame
        let after_raster = h.observed.as_ref().unwrap().work();
        assert_eq!(h.raster_count(), 1);
        // A canvas flush plus a screen flush.
        assert!(after_raster.flushes >= 2);

        h.frame(0.0); // the cached frame
        let cached_frame = h.observed.as_ref().unwrap().work() - after_raster;
        assert_eq!(h.raster_count(), 1);

        // A cached frame opens no offscreen frame at all: one flush, one
        // screen target, and less vector work than either the rasterizing
        // frame or the uncached baseline.
        assert_eq!(cached_frame.flushes, 1);
        assert!(cached_frame.flushes < after_raster.flushes);
        assert!(cached_frame.tess_vertex_spans < after_raster.tess_vertex_spans);
        assert!(cached_frame.tess_vertex_spans < uncached_frame.tess_vertex_spans);
        assert_eq!(cached_frame.target_pixels, uncached_frame.target_pixels);

        // The other side of the trade: a frame that has to rasterize costs
        // more than not caching at all.
        assert!(after_raster.flushes > uncached_frame.flushes);
        assert!(after_raster.target_pixels > uncached_frame.target_pixels);

        samples.push(after_raster);
    }

    // Real GPU work on the rasterizing frame rises with resolution.
    for pair in samples.windows(2) {
        assert!(pair[1].target_pixels > pair[0].target_pixels);
        assert!(pair[1].tess_data_height >= pair[0].tess_data_height);
    }
    let low_overhead = samples.first().unwrap().target_pixels - uncached_frame.target_pixels;
    let high_overhead = samples.last().unwrap().target_pixels - uncached_frame.target_pixels;
    assert!(low_overhead < high_overhead);
}

#[test]
fn resolution_is_clamped_to_a_sane_range() {
    let mut h = Harness::new();
    let (w, ht) = (h.width(), h.height());

    // Below the floor clamps to MIN_RES, not to zero-area.
    h.set_resolution(0.0);
    h.frame(0.0);
    let tiny = h.last_raster();
    assert!(tiny.width >= 1);
    assert!(tiny.height >= 1);
    assert_eq!(tiny.width, expected_dim(w, MIN_RES));
    assert_eq!(tiny.height, expected_dim(ht, MIN_RES));

    // Negative is the same clamp, not a wrap into something huge.
    h.set_resolution(-4.0);
    h.frame(0.0);
    assert_eq!(h.last_raster().width, tiny.width);
    assert_eq!(h.last_raster().height, tiny.height);

    // Above the ceiling clamps to MAX_RES, and no dimension exceeds MAX_DIM.
    h.set_resolution(64.0);
    h.frame(0.0);
    let huge = h.last_raster();
    assert!(huge.width <= MAX_DIM);
    assert!(huge.height <= MAX_DIM);
    assert_eq!(huge.width, expected_dim(w, MAX_RES));
    assert_eq!(huge.height, expected_dim(ht, MAX_RES));
    assert!(huge.pixels() > tiny.pixels());

    // Non-finite values are handled rather than assumed away.
    h.set_resolution(f32::INFINITY);
    let infinite = h.frame(0.0);
    assert_eq!(infinite.canvas_opens(), 1);
    assert_eq!(h.last_raster().width, huge.width);
    assert_eq!(h.last_raster().height, huge.height);

    // NaN has no size to rasterize at, so it takes the vector path.
    let rasters_before_nan = h.raster_count();
    h.set_resolution(f32::NAN);
    let not_a_number = h.frame(0.0);
    assert_eq!(h.raster_count(), rasters_before_nan);
    assert_eq!(not_a_number.canvas_opens(), 0);
    assert_eq!(not_a_number.draw_images(), 0);
    assert!(not_a_number.draw_paths() > 0);
}

#[test]
fn the_cache_engages_through_the_real_nested_artboard_path() {
    // The production shape: the file's default artboard nests the cached one,
    // and a host draws the root with Artboard::draw(). The root itself is not
    // cached but the nested instance reaches draw_internal.
    let (render_context, _, _) = observing_factory(1, 1);
    let mut session = PersistentFactory::new(DeferredSession::with_caps(Default::default()));
    session
        .borrow_mut()
        .bind_render_context(render_context.persistent_context());
    let file = nuxie_runtime::File::import(
        &fixture(),
        nuxie_runtime::RuntimeFactoryHandle::from_factory(&mut session).unwrap(),
        None,
        None,
        None,
    )
    .expect("fixture imports");
    let root = file.with_file(|f| f.artboard_default()).expect("root");
    // The root is a plain artboard; the caching one is the instance inside it.
    assert!(root.with_artboard(|a| a.bitmap_cache().is_none()));
    let machine = root.state_machine_at(0).expect("state machine");
    bind_view_model(&file, &root, &machine);

    let mut rasters = 0;
    let mut draw_frame = |dt: f32| {
        machine.advance_and_apply(dt);
        let screen = session.borrow().screen_renderer(0);
        {
            let mut screen = screen.borrow_mut();
            screen.save();
            root.draw(screen.as_mut());
            screen.restore();
        }
        let frame = snapshot_frame(&mut session.borrow_mut());
        rasters += frame.content_canvases.len();
        session.borrow_mut().reset_frame();
        rasters
    };

    // The nested instance rasterized itself offscreen.
    assert_eq!(draw_frame(0.0), 1);
    // Static frames ride the cache: no re-raster from the root draw either.
    let mut count = 0;
    for _ in 0..4 {
        count = draw_frame(0.0);
    }
    assert_eq!(count, 1);
    // Advancing time changes nothing in this fixture's nested artboard.
    let before = count;
    for _ in 0..4 {
        count = draw_frame(1.0 / 60.0);
    }
    println!(
        "[bitmap-cache] nested root draw: {before} rasters over 5 static frames, {} more over 4 advanced ones",
        count - before
    );
    assert_eq!(count, before);
}

fn assert_raster_covers_bounds(h: &mut Harness, label: &str) {
    h.frame(0.0);
    assert_eq!(h.raster_count(), 1, "{label}");
    let ctm = composite_transform(&h.last_frame).expect("a composite");
    let raster = h.last_raster();
    let origin = ctm * Vec2D::new(0.0, 0.0);
    let corner = ctm * Vec2D::new(raster.width as f32, raster.height as f32);
    let want = h.artboard.with_artboard(|a| a.bounds());
    println!(
        "[bitmap-cache] {label}: bounds [{} {} {} {}], raster lands [{} {} {} {}]",
        want.left(),
        want.top(),
        want.right(),
        want.bottom(),
        origin.x,
        origin.y,
        corner.x,
        corner.y
    );
    let slack = SNAP_SLACK + (corner.x - origin.x) / raster.width as f32;
    assert!(approx(origin.x, want.left(), slack), "{label} left");
    assert!(approx(origin.y, want.top(), slack), "{label} top");
    assert!(approx(corner.x, want.right(), slack), "{label} right");
    assert!(approx(corner.y, want.bottom(), slack), "{label} bottom");
}

#[test]
fn the_raster_covers_the_artboards_own_bounds() {
    {
        let mut h = Harness::new();
        h.set_resolution(1.0);
        h.artboard.with_artboard_mut(|a| a.set_frame_origin(false)); // what a NestedArtboard host sets
        assert_raster_covers_bounds(&mut h, "origin 0,0");
    }
    {
        let mut h = Harness::new();
        h.set_resolution(1.0);
        h.artboard.with_artboard_mut(|a| a.set_frame_origin(false));
        h.set_artboard_double(ArtboardBase::ORIGIN_X_PROPERTY_KEY, 0.5);
        h.set_artboard_double(ArtboardBase::ORIGIN_Y_PROPERTY_KEY, 0.5);
        assert_raster_covers_bounds(&mut h, "origin 0.5,0.5 frameOrigin off");
    }
    {
        let mut h = Harness::new();
        h.set_resolution(1.0);
        h.set_artboard_double(ArtboardBase::ORIGIN_X_PROPERTY_KEY, 0.5);
        h.set_artboard_double(ArtboardBase::ORIGIN_Y_PROPERTY_KEY, 0.5);
        assert_raster_covers_bounds(&mut h, "origin 0.5,0.5 frameOrigin on");
    }
}

#[test]
fn the_recording_holds_vector_commands_never_pixels() {
    let mut bytes = Vec::new();
    for resolution in [0.25, 1.0] {
        let mut h = Harness::new();
        h.set_resolution(resolution);
        h.frame(0.0); // identical animation state both times
        assert_eq!(h.raster_count(), 1);
        bytes.push(h.raster_content_bytes[0]);
    }
    // 16x the pixels, byte for byte the same recording.
    assert_eq!(bytes[0], bytes[1]);
    assert!(bytes[0] > 0);
}

#[test]
fn what_in_the_offscreen_raster_scales_with_resolution() {
    let mut samples = Vec::new();
    for resolution in [0.125, 0.25, 0.5, 1.0] {
        let mut h = Harness::new();
        h.observe();
        h.set_resolution(resolution);
        let node = first_node_of(&h.artboard).expect("a node");
        // Settle first, replaying every frame so the session's resources stay
        // resident on the observed device.
        for _ in 0..60 {
            h.frame(1.0 / 60.0);
        }
        // Measure a frame that actually rasterizes.
        let rasters_before = h.raster_count();
        let before = h.observed.as_ref().unwrap().work();
        set_node_x(&node, node_x(&node) + 1.0);
        h.frame(1.0 / 60.0);
        assert_eq!(h.raster_count(), rasters_before + 1);
        let work = h.observed.as_ref().unwrap().work() - before;
        println!(
            "[bitmap-cache] {resolution:<6.2} targetPx={} paths={} tessSpans={} tessRows={} atlasArea={} features={:#x}",
            work.target_pixels,
            work.path_count,
            work.tess_vertex_spans,
            work.tess_data_height,
            work.atlas_content_area,
            h.observed.as_ref().unwrap().features.get()
        );
        samples.push(work);
    }

    for sample in &samples {
        assert!(sample.path_count > 0);
        assert!(sample.target_pixels > 0);
    }
    // Per-pixel, and quadratically so.
    for pair in samples.windows(2) {
        assert!(pair[1].target_pixels > pair[0].target_pixels);
    }
    for i in 2..samples.len() {
        let previous_step = samples[i - 1].target_pixels - samples[i - 2].target_pixels;
        let step = samples[i].target_pixels - samples[i - 1].target_pixels;
        assert!(step > previous_step * 3);
    }
    // Per-path: the floor.
    for sample in &samples {
        assert_eq!(sample.path_count, samples[0].path_count);
        assert_eq!(sample.tess_vertex_spans, samples[0].tess_vertex_spans);
    }
}

// ---- device-scale sizing (the crispness fix) ----

struct Composite {
    raster: Raster,
    ctm: M,
    artboard_width: f32,
    artboard_height: f32,
}

fn composite_under(outer: M) -> Composite {
    let mut h = Harness::new();
    h.frame_under(0.0, Some(outer));
    assert_eq!(h.raster_count(), 1);
    Composite {
        raster: h.last_raster(),
        ctm: composite_transform(&h.last_frame).expect("a composite"),
        artboard_width: h.width(),
        artboard_height: h.height(),
    }
}

// Two device scales an octave apart that both stay inside the MAX_DIM clamp,
// both whole sixteenths so quantization is a no-op.
fn scales_inside_clamp(probe: &Composite) -> (f32, f32) {
    let ceiling_scale =
        (MAX_DIM as f32 / probe.artboard_width).min(MAX_DIM as f32 / probe.artboard_height);
    let mut sixteenths = (ceiling_scale * 8.0).floor() as i32;
    sixteenths -= sixteenths % 2;
    let hi = sixteenths as f32 / 16.0;
    (hi * 0.5, hi)
}

#[test]
fn the_raster_is_sized_from_the_device_scale_the_artboard_lands_at() {
    let probe = composite_under(M::identity());
    let (lo_scale, hi_scale) = scales_inside_clamp(&probe);
    assert!(lo_scale >= 0.125);

    let lo = composite_under(M::new(lo_scale, 0.0, 0.0, lo_scale, 0.0, 0.0));
    let hi = composite_under(M::new(hi_scale, 0.0, 0.0, hi_scale, 0.0, 0.0));

    assert_eq!(
        lo.raster.width,
        expected_dim(probe.artboard_width, lo_scale)
    );
    assert_eq!(
        lo.raster.height,
        expected_dim(probe.artboard_height, lo_scale)
    );
    assert_eq!(
        hi.raster.width,
        expected_dim(probe.artboard_width, hi_scale)
    );
    assert_eq!(
        hi.raster.height,
        expected_dim(probe.artboard_height, hi_scale)
    );
    // Doubling the device scale is 4x the texels.
    assert!(hi.raster.pixels() > lo.raster.pixels() * 3);
}

#[test]
fn resolution_1_composites_one_texel_per_device_pixel() {
    let probe = composite_under(M::identity());
    let (lo_scale, hi_scale) = scales_inside_clamp(&probe);
    assert!(lo_scale >= 0.125);
    for scale in [lo_scale, hi_scale] {
        let now = composite_under(M::new(scale, 0.0, 0.0, scale, 0.0, 0.0));
        assert!(
            approx(now.ctm.find_max_scale(), 1.0, 0.001),
            "scale {scale}"
        );
    }
}

#[test]
fn the_composite_still_lands_on_the_artboards_box_under_device_scaling() {
    for scale in [0.75, 1.0, 2.0] {
        let outer = from_scale_and_translation(scale, scale, 40.0, -12.0);
        let c = composite_under(outer);
        let origin = c.ctm * Vec2D::new(0.0, 0.0);
        let corner = c.ctm * Vec2D::new(c.raster.width as f32, c.raster.height as f32);
        let want_origin = outer * Vec2D::new(0.0, 0.0);
        let want_corner = outer * Vec2D::new(c.artboard_width, c.artboard_height);

        assert!(approx(origin.x, want_origin.x, 0.01), "scale {scale}");
        assert!(approx(origin.y, want_origin.y, 0.01), "scale {scale}");
        assert!(corner.x >= want_corner.x - 0.01, "scale {scale}");
        assert!(corner.y >= want_corner.y - 0.01, "scale {scale}");
        assert!(corner.x - want_corner.x < 1.0, "scale {scale}");
        assert!(corner.y - want_corner.y < 1.0, "scale {scale}");
    }
}

#[test]
fn pixel_snapping_puts_the_composited_raster_on_the_pixel_grid() {
    // Both offsets are deliberately off-grid: unsnapped this would composite
    // at (10.25, 7.4) device pixels.
    let outer = from_scale_and_translation(2.0, 2.0, 10.25, 7.4);
    let unsnapped = outer * Vec2D::new(0.0, 0.0);
    assert!((unsnapped.x - unsnapped.x.round()).abs() > 0.01);
    assert!((unsnapped.y - unsnapped.y.round()).abs() > 0.01);

    let c = composite_under(outer);
    let origin = c.ctm * Vec2D::new(0.0, 0.0);
    assert!(approx(origin.x, origin.x.round(), 0.001));
    assert!(approx(origin.y, origin.y.round(), 0.001));

    // Snapping moves the image by less than a pixel and changes nothing else.
    assert!((origin.x - unsnapped.x).abs() <= 0.5 + 0.001);
    assert!((origin.y - unsnapped.y).abs() <= 0.5 + 0.001);
}

#[test]
fn max_dim_caps_the_device_scale_a_large_artboard_can_be_cached_at() {
    let probe = composite_under(M::identity());
    // This fixture is small (225x225) and does not clamp until ~9x, hence the
    // deliberately large scale.
    let retina = 16.0;
    let c = composite_under(M::new(retina, 0.0, 0.0, retina, 0.0, 0.0));

    let ceiling_scale =
        (MAX_DIM as f32 / probe.artboard_width).min(MAX_DIM as f32 / probe.artboard_height);
    assert!(ceiling_scale < retina); // otherwise this artboard is not capped

    // The raster tops out on its long axis...
    assert_eq!(c.raster.width.max(c.raster.height), MAX_DIM);
    // ...and the composite is left magnifying by the shortfall.
    let magnification = c.ctm.find_max_scale();
    assert!(magnification > 1.0);
    let expected = retina / ceiling_scale;
    assert!((magnification - expected).abs() <= expected * 0.02);
}
