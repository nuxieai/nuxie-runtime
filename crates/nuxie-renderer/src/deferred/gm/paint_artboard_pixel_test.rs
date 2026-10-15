//! tests/unit_tests/renderer/paint_artboard_pixel_test.cpp at fa2cd6e3.
use super::ore_gm_helper::*;
use crate::deferred::cmd::{
    deferred_replayer::{DeferredReplayer, snapshot_frame},
    deferred_session::DeferredSession,
};
use nuxie_runtime::{
    File, RuntimeArtboardInstanceHandle, RuntimeFactoryHandle,
    source::{
        artboard::Artboard,
        artboard_referencer::ArtboardReferencerBehavior,
        core::{CoreArena, CoreHandle},
        generated::{
            component_base::ComponentBase,
            core_registry::CoreRegistry,
            layout_component_base::LayoutComponentBase,
            node_base::NodeBase,
            shapes::{
                paint::solid_color_base::SolidColorBase, parametric_path_base::ParametricPathBase,
            },
        },
        shapes::{
            paint::{fill::Fill, paint_image::PaintImage, solid_color::SolidColor},
            rectangle::Rectangle,
            shape::Shape,
        },
        status_code::StatusCode,
    },
};

const RED: u32 = 0xffff0000;
const BLUE: u32 = 0xff0000ff;
fn number(object: &CoreHandle, key: u16, value: f32) {
    assert!(CoreRegistry::set_double_handle(object, key.into(), value));
}
fn add(root: &CoreHandle, object: &CoreHandle, parent: &CoreHandle) {
    let id = root
        .with_downcast::<Artboard, _>(|a| a.id_of(parent))
        .unwrap();
    assert!(CoreRegistry::set_uint_handle(
        object,
        ComponentBase::PARENT_ID_PROPERTY_KEY.into(),
        id
    ));
    root.with_downcast_mut::<Artboard, _>(|a| a.add_object(Some(object.clone())));
}
fn artboard(arena: &CoreArena, factory: RuntimeFactoryHandle) -> CoreHandle {
    let root = arena.insert(Artboard::with_factory(factory));
    root.with_downcast_mut::<Artboard, _>(|a| {
        a.set_core_arena(arena.clone());
        a.add_object(Some(root.clone()));
    });
    root
}
fn add_rect(
    arena: &CoreArena,
    root: &CoreHandle,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    color: u32,
) -> (CoreHandle, CoreHandle) {
    let shape = arena.insert(Shape::default());
    add(root, &shape, root);
    number(&shape, NodeBase::X_PROPERTY_KEY, x);
    number(&shape, NodeBase::Y_PROPERTY_KEY, y);
    let rect = arena.insert(Rectangle::default());
    number(&rect, ParametricPathBase::WIDTH_PROPERTY_KEY, width);
    number(&rect, ParametricPathBase::HEIGHT_PROPERTY_KEY, height);
    add(root, &rect, &shape);
    let fill = arena.insert(Fill::default());
    add(root, &fill, &shape);
    let solid = arena.insert(SolidColor::default());
    assert!(CoreRegistry::set_color_handle(
        &solid,
        SolidColorBase::COLOR_VALUE_PROPERTY_KEY.into(),
        color as i32
    ));
    add(root, &solid, &fill);
    (shape, fill)
}
fn initialize(root: &CoreHandle, size: f32) {
    assert_eq!(Artboard::initialize_handle(root), StatusCode::Ok);
    number(root, LayoutComponentBase::WIDTH_PROPERTY_KEY, size);
    number(root, LayoutComponentBase::HEIGHT_PROPERTY_KEY, size);
}
fn red(pixels: &[u8], x: usize, y: usize) -> bool {
    let p = &pixels[(y * 200 + x) * 4..];
    p[0] > 200 && p[1] < 40 && p[2] < 40
}
fn blue(pixels: &[u8], x: usize, y: usize) -> bool {
    let p = &pixels[(y * 200 + x) * 4..];
    p[2] > 200 && p[0] < 40 && p[1] < 40
}
fn render(
    session: &PersistentFactory<DeferredSession>,
    instance: &RuntimeArtboardInstanceHandle,
    host: &mut GmHost,
    replayer: &mut DeferredReplayer,
) -> Vec<u8> {
    let screen = session.borrow().screen_renderer(0);
    instance.draw_internal(screen.borrow_mut().as_mut());
    let frame = snapshot_frame(&mut session.borrow_mut());
    session.borrow_mut().reset_frame();
    replayer.replay_frame(&frame, host);
    host.finish_frame()
}
struct PaintedScene {
    session: PersistentFactory<DeferredSession>,
    _arena: CoreArena,
    instance: RuntimeArtboardInstanceHandle,
}
impl PaintedScene {
    fn new(host: &mut GmHost) -> Self {
        let mut session = PersistentFactory::new(DeferredSession::with_caps(Default::default()));
        session
            .borrow_mut()
            .bind_render_context(host.factory.persistent_context());
        let factory = RuntimeFactoryHandle::from_factory(&mut session).unwrap();
        let arena = CoreArena::default();
        let source = artboard(&arena, factory.clone());
        add_rect(&arena, &source, 25.0, 50.0, 50.0, 100.0, RED);
        add_rect(&arena, &source, 75.0, 50.0, 50.0, 100.0, BLUE);
        initialize(&source, 100.0);
        let root = artboard(&arena, factory.clone());
        let (_, fill) = add_rect(&arena, &root, 100.0, 100.0, 200.0, 200.0, 0xffffffff);
        let paint = arena.insert(PaintImage::default());
        add(&root, &paint, &fill);
        initialize(&root, 200.0);
        paint.with_downcast_mut::<PaintImage, _>(|paint| {
            paint.set_referenced_artboard(Some(source))
        });
        let instance = Artboard::instance_from_handle_with_factory(&root, Some(factory)).unwrap();
        instance.advance_default(0.0);
        Self {
            session,
            _arena: arena,
            instance,
        }
    }
    fn inner_red(&self) -> CoreHandle {
        let paints = self
            .instance
            .with_artboard(|a| a.find_all_handles::<PaintImage>());
        for paint in paints {
            let inner = paint
                .with_downcast::<PaintImage, _>(|paint| {
                    let source = paint.artboard_source().unwrap();
                    source
                        .artboard()
                        .with_artboard(|a| a.find_all_handles::<Shape>())
                })
                .unwrap();
            if let Some(shape) = inner.first() {
                return shape.clone();
            }
        }
        panic!("source red shape");
    }
    fn render(&self, host: &mut GmHost, replayer: &mut DeferredReplayer) -> Vec<u8> {
        render(&self.session, &self.instance, host, replayer)
    }
}
#[test]
fn a_shape_painted_with_an_artboard_shows_it_on_the_first_frame() {
    let mut host = GmHost::with_size(0xff000000, false, 200, 200);
    let scene = PaintedScene::new(&mut host);
    let pixels = scene.render(&mut host, &mut DeferredReplayer::default());
    assert!(red(&pixels, 50, 100));
    assert!(blue(&pixels, 150, 100));
}
#[test]
fn a_source_bound_to_the_hosts_view_model_shows_its_value() {
    let mut host = GmHost::with_size(0xff000000, false, 200, 200);
    let mut session = PersistentFactory::new(DeferredSession::with_caps(Default::default()));
    session
        .borrow_mut()
        .bind_render_context(host.factory.persistent_context());
    let file = File::import(
        &fixture("sync/paint_artboard_bind.riv"),
        RuntimeFactoryHandle::from_factory(&mut session).unwrap(),
        None,
        None,
        None,
    )
    .unwrap();
    let artboard = file.with_file(|f| f.artboard_named("Host")).unwrap();
    let instance = file
        .with_file(|f| f.view_model_by_name("vm"))
        .unwrap()
        .create_default_instance();
    artboard.bind_view_model_instance(Some(instance.instance()));
    let mut replayer = DeferredReplayer::default();
    artboard.advance_default(0.016);
    let first = render(&session, &artboard, &mut host, &mut replayer);
    assert!(red(&first, 50, 100));
    assert!(blue(&first, 150, 100));
    instance.property_number("redX").unwrap().set_value(75.0);
    artboard.advance_default(0.016);
    let second = render(&session, &artboard, &mut host, &mut replayer);
    assert!(red(&second, 150, 100));
}
#[test]
fn a_changed_source_shows_on_the_next_frame() {
    let mut host = GmHost::with_size(0xff000000, false, 200, 200);
    let scene = PaintedScene::new(&mut host);
    let mut replayer = DeferredReplayer::default();
    let first = scene.render(&mut host, &mut replayer);
    assert!(blue(&first, 150, 100));
    number(&scene.inner_red(), NodeBase::X_PROPERTY_KEY, 75.0);
    scene.instance.advance_default(0.016);
    let second = scene.render(&mut host, &mut replayer);
    assert!(red(&second, 150, 100));
    scene.instance.advance_default(0.016);
    let third = scene.render(&mut host, &mut replayer);
    assert!(red(&third, 150, 100));
}
