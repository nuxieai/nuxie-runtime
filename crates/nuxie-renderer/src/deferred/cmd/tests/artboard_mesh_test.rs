//! Complete tests/unit_tests/runtime/artboard_mesh_test.cpp at 1b9b7381.
use super::super::{
    command_stream::CommandReader,
    deferred_replayer::snapshot_frame,
    deferred_session::DeferredSession,
    render_commands::{payload_size_of, RenderCmd},
};
use super::render_context_null::*;
use super::*;
use nuxie_runtime::source::{
    advance_flags::AdvanceFlags,
    artboard::Artboard,
    bitmap_cache::BitmapCache,
    component_dirt::ComponentDirt,
    core::{CoreArena, CoreHandle},
    drawable::Drawable,
    generated::{
        artboard_base::ArtboardBase,
        bitmap_cache_base::BitmapCacheBase,
        component_base::ComponentBase,
        core_registry::CoreRegistry,
        layout_component_base::LayoutComponentBase,
        node_base::NodeBase,
        shapes::{
            clipping_shape_base::ClippingShapeBase, mesh_vertex_base::MeshVertexBase,
            parametric_path_base::ParametricPathBase, vertex_base::VertexBase,
        },
        transform_component_base::TransformComponentBase,
        world_transform_component_base::WorldTransformComponentBase,
    },
    layout::n_slicer::NSlicer,
    nested_artboard::NestedArtboard,
    nested_artboard_layout::NestedArtboardLayout,
    nested_artboard_leaf::NestedArtboardLeaf,
    offscreen_raster as offscreen,
    shapes::{
        clipping_shape::ClippingShape,
        mesh::Mesh,
        mesh_host::MeshHost,
        mesh_vertex::MeshVertex,
        paint::{fill::Fill, solid_color::SolidColor},
        rectangle::Rectangle,
        shape::Shape,
    },
    status_code::StatusCode,
};
use nuxie_runtime::{RuntimeArtboardInstanceHandle, RuntimeFactoryHandle};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    sync::Mutex,
};

// The upstream runner is serial; Rust executes tests concurrently. All cases
// in this module share this lock because two mutate the global display scale.
pub(super) static DISPLAY_SCALE_TESTS: Mutex<()> = Mutex::new(());
macro_rules! mesh_test {
    ($name:ident, $body:block) => {
        #[test]
        fn $name() {
            let _guard = DISPLAY_SCALE_TESTS
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            $body
        }
    };
}
fn number(owner: &CoreHandle, key: u16, value: f32) {
    assert!(CoreRegistry::set_double_handle(owner, key.into(), value));
}
fn boolean(owner: &CoreHandle, key: u16, value: bool) {
    assert!(CoreRegistry::set_bool_handle(owner, key.into(), value));
}
fn uint(owner: &CoreHandle, key: u16, value: u32) {
    assert!(CoreRegistry::set_uint_handle(owner, key.into(), value));
}
fn advance(owner: &CoreHandle) {
    Artboard::advance_handle(
        owner,
        0.0,
        AdvanceFlags::ADVANCE_NESTED | AdvanceFlags::ANIMATE | AdvanceFlags::NEW_FRAME,
    );
}
fn add(artboard: &CoreHandle, owner: &CoreHandle, parent: &CoreHandle) {
    artboard
        .with_downcast_mut::<Artboard, _>(|a| a.add_object(Some(owner.clone())))
        .unwrap();
    let id = artboard
        .with_downcast::<Artboard, _>(|a| a.id_of(parent))
        .unwrap();
    uint(owner, ComponentBase::PARENT_ID_PROPERTY_KEY, id);
}
fn make_artboard(arena: &CoreArena, factory: &RuntimeFactoryHandle) -> CoreHandle {
    let root = arena.insert(Artboard::with_factory(factory.clone()));
    root.with_downcast_mut::<Artboard, _>(|a| {
        a.set_core_arena(arena.clone());
        a.add_object(Some(root.clone()));
    })
    .unwrap();
    root
}
fn make_source(
    arena: &CoreArena,
    factory: &RuntimeFactoryHandle,
    width: f32,
    height: f32,
    cached: bool,
) -> (CoreHandle, Option<CoreHandle>) {
    let root = make_artboard(arena, factory);
    let cache = cached.then(|| {
        let cache = arena.insert(BitmapCache::default());
        add(&root, &cache, &root);
        cache
    });
    let shape = arena.insert(Shape::default());
    add(&root, &shape, &root);
    let rect = arena.insert(Rectangle::default());
    add(&root, &rect, &shape);
    let fill = arena.insert(Fill::default());
    add(&root, &fill, &shape);
    add(&root, &arena.insert(SolidColor::default()), &fill);
    assert_eq!(Artboard::initialize_handle(&root), StatusCode::Ok);
    number(&root, LayoutComponentBase::WIDTH_PROPERTY_KEY, width);
    number(&root, LayoutComponentBase::HEIGHT_PROPERTY_KEY, height);
    number(&root, ArtboardBase::ORIGIN_X_PROPERTY_KEY, 0.5);
    number(&root, ArtboardBase::ORIGIN_Y_PROPERTY_KEY, 0.5);
    number(&rect, ParametricPathBase::WIDTH_PROPERTY_KEY, width);
    number(&rect, ParametricPathBase::HEIGHT_PROPERTY_KEY, height);
    advance(&root);
    (root, cache)
}

#[derive(Clone)]
struct CountingSession {
    session: DeferredSession,
    rasters: Rc<RefCell<Vec<(u32, u32)>>>,
    drops_clips: Rc<Cell<bool>>,
    on_content_drawn: Rc<RefCell<Option<Rc<dyn Fn()>>>>,
}
impl CountingSession {
    fn new() -> Self {
        Self {
            session: DeferredSession::with_caps(Default::default()),
            rasters: Default::default(),
            drops_clips: Default::default(),
            on_content_drawn: Default::default(),
        }
    }
}
impl DeferredCanvasHost for CountingSession {
    fn composite_drops_clips(&self) -> bool {
        self.drops_clips.get()
    }
    fn supports_layer_mask(&self) -> bool {
        DeferredCanvasHost::supports_layer_mask(&self.session)
    }
    fn make_content_canvas(&mut self, w: u32, h: u32) -> Option<RenderCanvasHandle> {
        self.session.make_content_canvas(w, h)
    }
    fn content_canvas_image(&mut self, c: &RenderCanvasHandle) -> Option<Rc<dyn RenderImage>> {
        self.session.content_canvas_image(c)
    }
    fn begin_canvas_content(
        &mut self,
        c: RenderCanvasHandle,
        color: ColorInt,
    ) -> Option<Box<dyn Renderer>> {
        self.rasters
            .borrow_mut()
            .push((c.borrow().width(), c.borrow().height()));
        DeferredCanvasHost::begin_canvas_content(&mut self.session, c, color)
    }
    fn end_canvas_content(&mut self, c: &RenderCanvasHandle) {
        DeferredCanvasHost::end_canvas_content(&mut self.session, c);
        let callback = self.on_content_drawn.borrow().clone();
        if let Some(callback) = callback {
            callback();
        }
    }
}
impl Factory for CountingSession {
    fn render_context(&mut self) -> Option<PersistentFactoryContext> {
        self.session.render_context()
    }
    fn deferred_canvas_host(&mut self) -> Option<DeferredCanvasHostHandle> {
        Some(Rc::new(RefCell::new(self.clone())))
    }
    fn make_render_buffer(
        &mut self,
        t: RenderBufferType,
        f: RenderBufferFlags,
        n: usize,
    ) -> Box<dyn RenderBuffer> {
        self.session.make_render_buffer(t, f, n)
    }
    fn make_linear_gradient(
        &mut self,
        sx: f32,
        sy: f32,
        ex: f32,
        ey: f32,
        c: &[ColorInt],
        s: &[f32],
    ) -> Box<dyn RenderShader> {
        self.session.make_linear_gradient(sx, sy, ex, ey, c, s)
    }
    fn make_radial_gradient(
        &mut self,
        x: f32,
        y: f32,
        r: f32,
        c: &[ColorInt],
        s: &[f32],
    ) -> Box<dyn RenderShader> {
        self.session.make_radial_gradient(x, y, r, c, s)
    }
    fn make_render_path(&mut self, p: RawPath, r: FillRule) -> Box<dyn RenderPath> {
        self.session.make_render_path(p, r)
    }
    fn make_empty_render_path(&mut self) -> Box<dyn RenderPath> {
        self.session.make_empty_render_path()
    }
    fn make_render_paint(&mut self) -> Box<dyn RenderPaint> {
        self.session.make_render_paint()
    }
    fn decode_image(&mut self, d: &[u8]) -> Result<Box<dyn RenderImage>, ImageDecodeError> {
        self.session.decode_image(d)
    }
}
struct Census([u64; RenderCmd::ResourceNewVersion as usize + 1]);
impl Census {
    fn of(&self, command: RenderCmd) -> u64 {
        self.0[command as usize]
    }
    fn read(commands: &[u8], blobs: &[u8]) -> Self {
        let mut result = Self([0; RenderCmd::ResourceNewVersion as usize + 1]);
        let mut reader = CommandReader::new(commands, blobs);
        while let Some(byte) = reader.next_u8() {
            let Some(cmd) = RenderCmd::from_byte(byte) else {
                break;
            };
            result.0[cmd as usize] += 1;
            reader.skip(payload_size_of(cmd));
        }
        result
    }
}
#[derive(Clone, Copy)]
enum HostDeformer {
    Mesh,
    Nslice,
    None,
}
#[derive(Clone, Copy)]
enum HostType {
    Node,
    Leaf,
    Layout,
}
struct MeshScene {
    arena: CoreArena,
    factory: RuntimeFactoryHandle,
    _render_context: ObservingFactory,
    session: CountingSession,
    source: CoreHandle,
    outer: CoreHandle,
    nested: CoreHandle,
    mesh: Option<CoreHandle>,
    vertices: Vec<CoreHandle>,
    init_status: StatusCode,
}
impl MeshScene {
    fn new() -> Self {
        Self::with(HostDeformer::Mesh, HostType::Node, true, false)
    }
    fn with(deformer: HostDeformer, host: HostType, with_host: bool, clipped: bool) -> Self {
        let (render_context, _, _) = observing_factory(1, 1);
        let mut session = CountingSession::new();
        session.session.bind_render_context(if with_host {
            render_context.persistent_context()
        } else {
            None
        });
        let mut persistent = PersistentFactory::new(session.clone());
        let factory = RuntimeFactoryHandle::from_factory(&mut persistent).unwrap();
        let arena = CoreArena::default();
        let (source, _) = make_source(&arena, &factory, 200.0, 100.0, false);
        let outer = make_artboard(&arena, &factory);
        let nested = match host {
            HostType::Node => arena.insert(NestedArtboard::new()),
            HostType::Leaf => arena.insert(NestedArtboardLeaf::default()),
            HostType::Layout => arena.insert(NestedArtboardLayout::default()),
        };
        add(&outer, &nested, &outer);
        number(&nested, NodeBase::X_PROPERTY_KEY, 250.0);
        number(&nested, NodeBase::Y_PROPERTY_KEY, 250.0);
        let mut mesh = None;
        let mut vertices = Vec::new();
        match deformer {
            HostDeformer::Mesh => {
                let m = arena.insert(Mesh::default());
                add(&outer, &m, &nested);
                for (u, v) in [(0.0, 0.0), (1.0, 0.0), (1.0, 1.0), (0.0, 1.0)] {
                    let vertex = arena.insert(MeshVertex::default());
                    number(&vertex, MeshVertexBase::U_PROPERTY_KEY, u);
                    number(&vertex, MeshVertexBase::V_PROPERTY_KEY, v);
                    add(&outer, &vertex, &m);
                    vertices.push(vertex);
                }
                // All six varuint indices fit in one byte, exactly as VectorBinaryWriter emits.
                m.with_downcast_mut::<Mesh, _>(|m| {
                    m.decode_triangle_index_bytes(&[0, 1, 2, 0, 2, 3])
                })
                .unwrap();
                mesh = Some(m);
            }
            HostDeformer::Nslice => {
                add(&outer, &arena.insert(NSlicer::default()), &nested);
            }
            HostDeformer::None => {}
        }
        if clipped {
            let shape = arena.insert(Shape::default());
            add(&outer, &shape, &outer);
            let rect = arena.insert(Rectangle::default());
            add(&outer, &rect, &shape);
            number(&rect, ParametricPathBase::WIDTH_PROPERTY_KEY, 50.0);
            number(&rect, ParametricPathBase::HEIGHT_PROPERTY_KEY, 50.0);
            let clip = arena.insert(ClippingShape::default());
            add(&outer, &clip, &nested);
            uint(
                &clip,
                ClippingShapeBase::SOURCE_ID_PROPERTY_KEY,
                outer
                    .with_downcast::<Artboard, _>(|a| a.id_of(&shape))
                    .unwrap(),
            );
        }
        let init_status = Artboard::initialize_handle(&outer);
        if init_status == StatusCode::Ok {
            number(&outer, LayoutComponentBase::WIDTH_PROPERTY_KEY, 500.0);
            number(&outer, LayoutComponentBase::HEIGHT_PROPERTY_KEY, 500.0);
            for v in &vertices {
                let u = CoreRegistry::get_double_handle(v, MeshVertexBase::U_PROPERTY_KEY.into())
                    .unwrap();
                let y = CoreRegistry::get_double_handle(v, MeshVertexBase::V_PROPERTY_KEY.into())
                    .unwrap();
                number(v, VertexBase::X_PROPERTY_KEY, (u - 0.5) * 200.0);
                number(v, VertexBase::Y_PROPERTY_KEY, (y - 0.5) * 100.0);
            }
            let instance = Artboard::instance_from_handle(&source).unwrap();
            nested
                .with_mut(|n| {
                    n.as_nested_artboard_mut()
                        .unwrap()
                        .referenced_artboard_instance(instance)
                })
                .unwrap();
            advance(&outer);
        }
        Self {
            arena,
            factory,
            _render_context: render_context,
            session,
            source,
            outer,
            nested,
            mesh,
            vertices,
            init_status,
        }
    }
    fn frame(&mut self) -> Census {
        self.frame_with_visitor(false)
    }
    fn frame_with_visitor(&mut self, visitor: bool) -> Census {
        advance(&self.outer);
        let screen = self.session.session.screen_renderer(0);
        {
            let mut screen = screen.borrow_mut();
            screen.save();
            if visitor {
                Artboard::draw_internal_with_visitor_handle(
                    &self.outer,
                    screen.as_mut(),
                    Some(Rc::new(|drawable, renderer| {
                        Drawable::draw_handle(drawable, renderer)
                    })),
                    None,
                );
            } else {
                Artboard::draw_internal_handle(&self.outer, screen.as_mut());
            }
            screen.restore();
        }
        let frame = snapshot_frame(&mut self.session.session);
        let census = Census::read(&frame.commands, &frame.blobs);
        self.session.session.reset_frame();
        census
    }
    fn instance(&self) -> RuntimeArtboardInstanceHandle {
        self.nested
            .with(|n| n.as_nested_artboard().unwrap().artboard_instance_handle(0))
            .flatten()
            .unwrap()
    }
    fn host(&self) -> MeshHost {
        MeshHost::from_handle(Some(self.nested.clone())).unwrap()
    }
    fn has_host(&self) -> bool {
        self.nested
            .with(|n| n.as_nested_artboard().unwrap().mesh_host_if_any().is_some())
            .unwrap()
    }
    fn mount(&self, source: &CoreHandle) {
        let instance = Artboard::instance_from_handle(source).unwrap();
        self.nested
            .with_mut(|n| {
                n.as_nested_artboard_mut()
                    .unwrap()
                    .referenced_artboard_instance(instance)
            })
            .unwrap();
    }
    fn raster_count(&self) -> usize {
        self.session.rasters.borrow().len()
    }
    fn cached_source(&mut self, resolution: f32, enabled: bool) {
        let (source, cache) = make_source(&self.arena, &self.factory, 200.0, 100.0, true);
        let cache = cache.unwrap();
        number(&cache, BitmapCacheBase::RESOLUTION_PROPERTY_KEY, resolution);
        boolean(&cache, BitmapCacheBase::CACHE_ENABLED_PROPERTY_KEY, enabled);
        self.source = source;
        self.mount(&self.source);
        advance(&self.outer);
        let cache = self.instance().with_artboard(|a| a.bitmap_cache()).unwrap();
        assert_eq!(
            CoreRegistry::get_double_handle(
                &cache,
                BitmapCacheBase::RESOLUTION_PROPERTY_KEY.into()
            ),
            Some(resolution)
        );
        assert_eq!(
            CoreRegistry::get_bool_handle(
                &cache,
                BitmapCacheBase::CACHE_ENABLED_PROPERTY_KEY.into()
            ),
            Some(enabled)
        );
    }
}
fn approx(actual: f32, expected: f32) {
    assert!(
        (actual - expected).abs() <= 100.0 * f32::EPSILON * expected.abs(),
        "{actual} != Approx({expected})"
    );
}

mesh_test!(mesh_host_accepts_images_and_node_mode_instances_only, {
    let arena = CoreArena::default();
    assert!(MeshHost::from_handle(Some(arena.insert(NestedArtboard::new()))).is_some());
    assert!(MeshHost::from_handle(Some(arena.insert(NestedArtboardLeaf::default()))).is_none());
    assert!(MeshHost::from_handle(Some(arena.insert(NestedArtboardLayout::default()))).is_none());
});
mesh_test!(an_instance_without_a_mesh_allocates_no_mesh_state, {
    let mut s = MeshScene::with(HostDeformer::None, HostType::Node, true, false);
    assert_eq!(s.init_status, StatusCode::Ok);
    s.frame();
    assert!(!s.has_host());
});
mesh_test!(a_mesh_under_a_leaf_or_layout_instance_is_rejected, {
    for host in [HostType::Leaf, HostType::Layout] {
        let s = MeshScene::with(HostDeformer::Mesh, host, true, false);
        assert!(!s.has_host());
    }
});
mesh_test!(a_meshed_instance_draws_its_artboard_through_the_mesh, {
    let mut s = MeshScene::new();
    assert_eq!(s.init_status, StatusCode::Ok);
    assert_eq!(s.host().hosted_mesh(), s.mesh);
    let c = s.frame();
    assert_eq!(s.raster_count(), 1);
    assert_eq!(c.of(RenderCmd::CanvasContentBegin), 1);
    assert_eq!(c.of(RenderCmd::DrawImageMesh), 1);
    assert_eq!(c.of(RenderCmd::DrawImage), 0);
});
mesh_test!(the_raster_is_the_artboard_size_times_resolution, {
    let mut s = MeshScene::new();
    s.frame();
    assert_eq!(&*s.session.rasters.borrow(), &[(200, 100)]);
});
mesh_test!(a_bitmap_cache_resolution_sets_the_mesh_raster_size, {
    let mut s = MeshScene::new();
    s.cached_source(2.0, true);
    assert_eq!(
        s.nested
            .with(|n| n
                .as_nested_artboard()
                .unwrap()
                .mesh_host_if_any()
                .unwrap()
                .borrow()
                .mesh_raster_resolution())
            .unwrap(),
        2.0
    );
    s.frame();
    assert_eq!(&*s.session.rasters.borrow(), &[(400, 200)]);
});
mesh_test!(a_meshed_instance_rasterizes_with_the_cache_switched_off, {
    let mut s = MeshScene::new();
    s.cached_source(1.0, false);
    let c = s.frame();
    assert_eq!(s.raster_count(), 1);
    assert_eq!(c.of(RenderCmd::DrawImageMesh), 1);
});
mesh_test!(a_static_meshed_instance_reuses_its_raster, {
    let mut s = MeshScene::new();
    s.frame();
    for _ in 0..4 {
        assert_eq!(s.frame().of(RenderCmd::DrawImageMesh), 1);
    }
    assert_eq!(s.raster_count(), 1);
});
mesh_test!(moving_a_vertex_redraws_without_rerasterizing, {
    let mut s = MeshScene::new();
    s.frame();
    let x =
        CoreRegistry::get_double_handle(&s.vertices[2], VertexBase::X_PROPERTY_KEY.into()).unwrap();
    number(&s.vertices[2], VertexBase::X_PROPERTY_KEY, x + 40.0);
    assert_eq!(s.frame().of(RenderCmd::DrawImageMesh), 1);
    assert_eq!(s.raster_count(), 1);
});
mesh_test!(
    swapping_the_instances_artboard_rerasterizes_at_the_new_size,
    {
        let mut s = MeshScene::new();
        s.frame();
        assert_eq!(s.raster_count(), 1);
        let (other, _) = make_source(&s.arena, &s.factory, 120.0, 60.0, false);
        s.mount(&other);
        advance(&s.outer);
        let c = s.frame();
        assert_eq!(s.raster_count(), 2);
        assert_eq!(s.session.rasters.borrow()[1], (120, 60));
        assert_eq!(c.of(RenderCmd::DrawImageMesh), 1);
        assert_eq!(s.host().hosted_mesh(), s.mesh);
    }
);
mesh_test!(uvs_map_onto_the_part_of_the_raster_the_box_covers, {
    let exact = MeshScene::new();
    assert_eq!(exact.host().uv_transform(), Mat2D::IDENTITY);
    let s = MeshScene::new();
    let (other, _) = make_source(&s.arena, &s.factory, 200.5, 100.0, false);
    s.mount(&other);
    let uv = s.host().uv_transform();
    approx(uv.0[0], 200.5 / 201.0);
    approx(uv.0[3], 1.0);
});
mesh_test!(the_mesh_host_reports_the_instances_box, {
    let s = MeshScene::new();
    assert_eq!(s.host().width(), 200.0);
    assert_eq!(s.host().height(), 100.0);
    let origin = s.host().origin();
    approx(origin.x, 0.5);
    approx(origin.y, 0.5);
});
mesh_test!(
    an_instance_without_a_canvas_host_draws_vectors_undeformed,
    {
        let mut s = MeshScene::with(HostDeformer::Mesh, HostType::Node, false, false);
        let c = s.frame();
        assert_eq!(s.raster_count(), 0);
        assert_eq!(c.of(RenderCmd::DrawImageMesh), 0);
        assert!(c.of(RenderCmd::DrawPath) > 0);
    }
);
mesh_test!(a_transparent_meshed_instance_draws_nothing, {
    let mut s = MeshScene::new();
    number(
        &s.nested,
        WorldTransformComponentBase::OPACITY_PROPERTY_KEY,
        0.0,
    );
    assert_eq!(s.frame().of(RenderCmd::DrawImageMesh), 0);
    assert_eq!(s.raster_count(), 0);
    number(
        &s.nested,
        WorldTransformComponentBase::OPACITY_PROPERTY_KEY,
        1.0,
    );
    assert_eq!(s.frame().of(RenderCmd::DrawImageMesh), 1);
    assert_eq!(s.raster_count(), 1);
});
mesh_test!(an_opacity_change_rerasterizes_the_instance, {
    let mut s = MeshScene::new();
    s.frame();
    number(
        &s.nested,
        WorldTransformComponentBase::OPACITY_PROPERTY_KEY,
        0.5,
    );
    s.frame();
    assert_eq!(s.raster_count(), 2);
});
mesh_test!(an_nslice_under_an_instance_draws_through_its_slice_mesh, {
    let mut s = MeshScene::with(HostDeformer::Nslice, HostType::Node, true, false);
    assert_eq!(s.init_status, StatusCode::Ok);
    assert!(s.host().hosted_mesh().is_some());
    let c = s.frame();
    assert_eq!(s.raster_count(), 1);
    assert_eq!(c.of(RenderCmd::DrawImageMesh), 1);
});
mesh_test!(a_clipped_meshed_instance_draws_inside_its_clip, {
    let mut s = MeshScene::with(HostDeformer::Mesh, HostType::Node, true, true);
    assert_eq!(s.init_status, StatusCode::Ok);
    let c = s.frame();
    assert_eq!(c.of(RenderCmd::DrawImageMesh), 1);
    let mut unclipped = MeshScene::new();
    assert_eq!(
        c.of(RenderCmd::ClipPath),
        unclipped.frame().of(RenderCmd::ClipPath) + 1
    );
});
struct ScaleScope;
impl Drop for ScaleScope {
    fn drop(&mut self) {
        offscreen::set_display_scale(1.0);
    }
}
mesh_test!(a_two_x_display_doubles_the_mesh_raster, {
    let _restore = ScaleScope;
    offscreen::set_display_scale(2.0);
    let mut s = MeshScene::new();
    s.frame();
    assert_eq!(&*s.session.rasters.borrow(), &[(400, 200)]);
});
mesh_test!(a_bad_display_scale_is_ignored, {
    let _restore = ScaleScope;
    offscreen::set_display_scale(2.0);
    offscreen::set_display_scale(0.0);
    offscreen::set_display_scale(f32::NAN);
    assert_eq!(offscreen::display_scale(), 2.0);
});
mesh_test!(a_draw_visitor_reaches_the_drawables_of_a_meshed_instance, {
    let mut s = MeshScene::new();
    assert_eq!(s.init_status, StatusCode::Ok);
    let c = s.frame_with_visitor(true);
    assert_eq!(c.of(RenderCmd::DrawImageMesh), 0);
    assert!(c.of(RenderCmd::DrawPath) >= 1);
    assert_eq!(s.frame().of(RenderCmd::DrawImageMesh), 1);
});
mesh_test!(
    a_change_drawn_through_the_mesh_refreshes_the_bitmap_cache_too,
    {
        let mut s = MeshScene::new();
        assert_eq!(s.init_status, StatusCode::Ok);
        let instance = s.instance();
        s.frame();
        let cache = Rc::new(RefCell::new(offscreen::CachedRaster::default()));
        let screen = s.session.session.screen_renderer(0);
        assert!(Artboard::raster_image_handle(
            &instance.core_handle(),
            screen.borrow_mut().as_mut(),
            &cache,
            1.0,
            true
        )
        .is_some());
        let before = cache.borrow().content_revision;
        s.session.session.reset_frame();
        number(
            &instance.core_handle(),
            WorldTransformComponentBase::OPACITY_PROPERTY_KEY,
            0.5,
        );
        s.frame();
        assert!(Artboard::raster_image_handle(
            &instance.core_handle(),
            screen.borrow_mut().as_mut(),
            &cache,
            1.0,
            true
        )
        .is_some());
        assert_ne!(cache.borrow().content_revision, before);
        s.session.session.reset_frame();
    }
);
mesh_test!(
    a_clipped_meshed_instance_draws_vectors_where_composite_drops_clips,
    {
        let mut s = MeshScene::with(HostDeformer::Mesh, HostType::Node, true, true);
        assert_eq!(s.init_status, StatusCode::Ok);
        s.session.drops_clips.set(true);
        let c = s.frame();
        assert_eq!(c.of(RenderCmd::DrawImageMesh), 0);
        assert!(c.of(RenderCmd::DrawPath) >= 1);
        let mut unclipped = MeshScene::new();
        boolean(
            &unclipped.outer,
            LayoutComponentBase::CLIP_PROPERTY_KEY,
            false,
        );
        unclipped.session.drops_clips.set(true);
        assert_eq!(unclipped.frame().of(RenderCmd::DrawImageMesh), 1);
    }
);
mesh_test!(
    a_meshed_instance_in_a_clipping_artboard_draws_vectors_where_composite_drops_clips,
    {
        let mut s = MeshScene::new();
        assert_eq!(s.init_status, StatusCode::Ok);
        s.session.drops_clips.set(true);
        boolean(&s.outer, LayoutComponentBase::CLIP_PROPERTY_KEY, false);
        assert_eq!(s.frame().of(RenderCmd::DrawImageMesh), 1);
        boolean(&s.outer, LayoutComponentBase::CLIP_PROPERTY_KEY, true);
        let c = s.frame();
        assert_eq!(c.of(RenderCmd::DrawImageMesh), 0);
        assert!(c.of(RenderCmd::DrawPath) >= 1);
    }
);
mesh_test!(
    an_instances_own_rotation_tells_its_host_the_transform_moved,
    {
        let mut s = MeshScene::new();
        assert_eq!(s.init_status, StatusCode::Ok);
        s.frame();
        let instance = s.instance();
        assert!(!s
            .nested
            .with(|n| n.as_component().unwrap().has_dirt(ComponentDirt::TRANSFORM))
            .unwrap());
        number(
            &instance.core_handle(),
            TransformComponentBase::ROTATION_PROPERTY_KEY,
            0.5,
        );
        assert!(s
            .nested
            .with(|n| n.as_component().unwrap().has_dirt(ComponentDirt::TRANSFORM))
            .unwrap());
    }
);
thread_local! {static TRANSFORM_DIRTY_CALLS: Cell<i32> = const {Cell::new(0)};}
fn transform_dirty(_: *mut ()) {
    TRANSFORM_DIRTY_CALLS.with(|c| c.set(c.get() + 1));
}
mesh_test!(every_own_rotation_frame_notifies_the_editor_host, {
    let mut s = MeshScene::new();
    assert_eq!(s.init_status, StatusCode::Ok);
    s.frame();
    let instance = s.instance();
    TRANSFORM_DIRTY_CALLS.with(|c| c.set(0));
    instance.with_artboard_mut(|a| {
        a.callback_user_data = std::ptr::null_mut();
        a.on_transform_dirty(Some(transform_dirty));
    });
    instance.update_pass(false);
    for frame in 1..=3 {
        number(
            &instance.core_handle(),
            TransformComponentBase::ROTATION_PROPERTY_KEY,
            0.1 * frame as f32,
        );
        instance.advance_internal(0.0, AdvanceFlags::NONE);
        assert_eq!(TRANSFORM_DIRTY_CALLS.with(Cell::get), frame);
    }
});
mesh_test!(
    a_component_mesh_host_reports_its_source_size_before_an_instance_exists,
    {
        let s = MeshScene::new();
        assert_eq!(s.init_status, StatusCode::Ok);
        let probe = s.arena.insert(NestedArtboard::new());
        probe
            .with_downcast_mut::<NestedArtboard, _>(|p| {
                p.referenced_artboard(Some(s.source.clone()))
            })
            .unwrap();
        assert!(probe
            .with_downcast::<NestedArtboard, _>(|p| p.artboard_instance_handle(0))
            .flatten()
            .is_none());
        let host = MeshHost::from_handle(Some(probe)).unwrap();
        approx(host.width(), 200.0);
        approx(host.height(), 100.0);
    }
);
mesh_test!(mounting_a_new_instance_releases_the_mesh_raster_at_once, {
    let mut s = MeshScene::new();
    assert_eq!(s.init_status, StatusCode::Ok);
    s.frame();
    let host = s
        .nested
        .with(|n| n.as_nested_artboard().unwrap().mesh_host_if_any())
        .flatten()
        .unwrap();
    assert!(host.borrow().raster_held());
    s.mount(&s.source);
    assert!(!host.borrow().raster_held());
});
mesh_test!(
    a_clipped_meshed_instance_with_a_cached_source_draws_vectors_where_composite_drops_clips,
    {
        let mut s = MeshScene::with(HostDeformer::Mesh, HostType::Node, true, true);
        assert_eq!(s.init_status, StatusCode::Ok);
        s.cached_source(1.0, true);
        s.session.drops_clips.set(true);
        let c = s.frame();
        assert_eq!(c.of(RenderCmd::DrawImageMesh), 0);
        assert_eq!(c.of(RenderCmd::DrawImage), 0);
        assert!(c.of(RenderCmd::DrawPath) >= 1);
    }
);
mesh_test!(a_change_raised_while_the_mesh_rasterizes_stays_pending, {
    let mut s = MeshScene::new();
    assert_eq!(s.init_status, StatusCode::Ok);
    s.frame();
    let instance = s.instance();
    let shape = instance
        .with_artboard(|a| a.objects_typed::<Shape>().iter().next())
        .unwrap();
    let move_shape = move || {
        let x = CoreRegistry::get_double_handle(&shape, NodeBase::X_PROPERTY_KEY.into()).unwrap();
        number(&shape, NodeBase::X_PROPERTY_KEY, x + 1.0);
    };
    move_shape();
    *s.session.on_content_drawn.borrow_mut() = Some(Rc::new(move_shape));
    s.frame();
    *s.session.on_content_drawn.borrow_mut() = None;
    assert_eq!(s.raster_count(), 2);
    assert!(instance.with_artboard(|a| a.did_change()));
});

// Rust ownership regressions for the same live CachedRaster that C++ mutates
// from cacheFlagsChanged/resetRaster while the content bracket is closing.
mesh_test!(
    disabling_a_bitmap_cache_while_drawing_releases_the_live_raster,
    {
        for reenable_in_callback in [false, true] {
            let mut s = MeshScene::new();
            let (source, cache) = make_source(&s.arena, &s.factory, 200.0, 100.0, true);
            let cache = cache.unwrap();
            let callback_cache = cache.clone();
            *s.session.on_content_drawn.borrow_mut() = Some(Rc::new(move || {
                boolean(
                    &callback_cache,
                    BitmapCacheBase::CACHE_ENABLED_PROPERTY_KEY,
                    false,
                );
                if reenable_in_callback {
                    boolean(
                        &callback_cache,
                        BitmapCacheBase::CACHE_ENABLED_PROPERTY_KEY,
                        true,
                    );
                }
            }));
            let screen = s.session.session.screen_renderer(0);
            assert!(!Artboard::draw_cached_as_bitmap_handle(
                &source,
                screen.borrow_mut().as_mut()
            ));
            *s.session.on_content_drawn.borrow_mut() = None;
            let frame = snapshot_frame(&mut s.session.session);
            assert_eq!(
                Census::read(&frame.commands, &frame.blobs).of(RenderCmd::DrawImage),
                0
            );
            s.session.session.reset_frame();
            assert_eq!(s.raster_count(), 1);
            boolean(&cache, BitmapCacheBase::CACHE_ENABLED_PROPERTY_KEY, true);
            assert!(Artboard::draw_cached_as_bitmap_handle(
                &source,
                screen.borrow_mut().as_mut()
            ));
            assert_eq!(s.raster_count(), 2);
            s.session.session.reset_frame();
        }
    }
);

mesh_test!(
    resetting_or_removing_a_mesh_during_raster_draw_cannot_composite_released_pixels,
    {
        for remove_mesh in [false, true] {
            let mut s = MeshScene::new();
            let host = s
                .nested
                .with(|n| n.as_nested_artboard().unwrap().mesh_host_if_any())
                .flatten()
                .unwrap();
            let callback_host = host.clone();
            *s.session.on_content_drawn.borrow_mut() = Some(Rc::new(move || {
                if remove_mesh {
                    callback_host.borrow_mut().set_mesh(None);
                } else {
                    callback_host.borrow_mut().reset_raster();
                }
            }));
            let frame = s.frame();
            *s.session.on_content_drawn.borrow_mut() = None;
            assert_eq!(frame.of(RenderCmd::DrawImageMesh), 0);
            assert!(!host.borrow().raster_held());
            assert_eq!(s.raster_count(), 1);
            if remove_mesh {
                host.borrow_mut().set_mesh(s.mesh.clone());
            }
            assert_eq!(s.frame().of(RenderCmd::DrawImageMesh), 1);
            assert!(host.borrow().raster_held());
            assert_eq!(s.raster_count(), 2);
        }
    }
);
