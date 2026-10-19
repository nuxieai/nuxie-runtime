//! All eight non-canvas paint_artboard_test.cpp cases at fa2cd6e3.
#[path = "../src/test_support/paint_artboard.rs"]
mod support;
use nuxie_runtime::source::{
    advancing_component::QuietState,
    artboard_host::ArtboardHost,
    artboard_referencer::ArtboardReferencer,
    backboard::Backboard,
    generated::shapes::paint::paint_image_base::PaintImageBase,
    importers::{backboard_importer::BackboardImporter, import_stack::ImportStackObject},
    shapes::paint::paint_image::PaintImage,
    status_code::StatusCode,
};
use nuxie_runtime::{Artboard, RuntimeFactoryHandle};
use support::*;
fn factory() -> RuntimeFactoryHandle {
    RuntimeFactoryHandle::from_factory(&mut nuxie_render_api::PersistentFactory::new(
        nuxie_render_api::NullFactory,
    ))
    .unwrap()
}
#[test]
fn each_host_instance_plays_its_own_instance_of_the_source() {
    let f = factory();
    let source = Scene::source(f.clone(), 100.0, 50.0);
    let host = Scene::host(f, Some(source.artboard.clone()), 10.0, 10.0, 0);
    host.paint
        .as_ref()
        .unwrap()
        .with_downcast::<PaintImage, _>(|p| {
            assert!(p.source.is_none());
            assert_eq!(p.source_artboard(), Some(source.artboard.clone()));
        })
        .unwrap();
    let a = host.instance();
    let b = host.instance();
    let pa = paint_of(&a);
    let pb = paint_of(&b);
    assert_ne!(inner(&pa).core_handle(), inner(&pb).core_handle());
    pa.with_downcast::<PaintImage, _>(|p| {
        assert_eq!(p.source_artboard(), Some(source.artboard.clone()));
        let s = p.source.as_ref().unwrap();
        assert!(s.artboard().with_artboard(|a| a.is_instance()));
        assert!(s.has_state_machine());
        assert!(!s.has_animation());
        approx(s.bounds().width(), 100.0);
        approx(s.bounds().height(), 50.0);
    })
    .unwrap();
}
#[test]
fn a_change_inside_the_source_marks_the_host_changed() {
    let f = factory();
    let source = Scene::source(f.clone(), 100.0, 50.0);
    let host = Scene::host(f, Some(source.artboard.clone()), 10.0, 10.0, 0);
    let a = host.instance();
    let p = paint_of(&a);
    advance(&a, 0.0);
    a.draw_internal(&mut nuxie_render_api::NullRenderer);
    assert!(!a.with_artboard(|a| a.did_change()));
    let shape = first_shape(&inner(&p));
    set_x(&shape, x(&shape) + 5.0);
    advance(&a, 0.0);
    assert!(a.with_artboard(|a| a.did_change()));
    assert!(
        !p.with_downcast::<PaintImage, _>(|p| p.source.as_ref().unwrap().needs_update())
            .unwrap()
    );
}
#[test]
fn an_artboard_that_paints_itself_stops_nesting() {
    let h = Scene::host(factory(), None, 10.0, 10.0, 0);
    h.paint
        .as_ref()
        .unwrap()
        .with_downcast_mut::<PaintImage, _>(|p| {
            p.artboard_referencer
                .set_referenced_artboard(Some(h.artboard.clone()))
        })
        .unwrap();
    let mut level = Some(h.instance());
    let mut depth = 0;
    while let Some(a) = level {
        let p = paint_of(&a);
        level = p
            .with_downcast::<PaintImage, _>(|p| p.source.as_ref().map(|s| s.artboard()))
            .unwrap();
        depth += 1;
        assert!(depth < 100);
    }
    assert!(depth > 1);
    assert!(depth <= 17);
}
#[test]
fn the_importer_cuts_paint_artboard_cycles() {
    let f = factory();
    let a = Scene::host(f.clone(), None, 10.0, 10.0, 0);
    let b = Scene::host(f.clone(), None, 10.0, 10.0, 0);
    let s = Scene::host(f.clone(), None, 10.0, 10.0, 0);
    let plain = Scene::source(f.clone(), 10.0, 10.0);
    let user = Scene::host(f, None, 10.0, 10.0, 0);
    for (scene, id) in [(&a, 1), (&b, 0), (&s, 2), (&user, 3)] {
        uint(
            scene.paint.as_ref().unwrap(),
            PaintImageBase::ARTBOARD_ID_PROPERTY_KEY,
            id,
        );
    }
    let back = a.arena.insert(Backboard::default());
    let mut importer = BackboardImporter::new(back);
    for scene in [&a, &b, &s, &plain, &user] {
        scene
            .artboard
            .with_downcast_mut::<Artboard, _>(|a| importer.add_artboard(a))
            .unwrap();
    }
    for scene in [&a, &b, &s, &user] {
        importer.add_artboard_referencer(scene.paint.clone().unwrap());
    }
    assert_eq!(importer.resolve(), StatusCode::Ok);
    for scene in [&a, &b, &s] {
        assert!(
            scene
                .paint
                .as_ref()
                .unwrap()
                .with_downcast::<PaintImage, _>(PaintImage::source_artboard)
                .flatten()
                .is_none()
        );
    }
    assert_eq!(
        user.paint
            .as_ref()
            .unwrap()
            .with_downcast::<PaintImage, _>(PaintImage::source_artboard)
            .flatten(),
        Some(plain.artboard)
    );
}
#[test]
fn artboard_referencer_from_finds_a_paint_image() {
    let host = Scene::host(factory(), None, 10.0, 10.0, 0);
    host.paint
        .as_ref()
        .unwrap()
        .with_downcast_mut::<PaintImage, _>(|p| {
            assert!(ArtboardReferencer::from(p).is_some());
        })
        .unwrap();
    assert!(
        host.paint
            .as_ref()
            .unwrap()
            .with(|p| p.is_advancing_component())
            .unwrap()
    );
}
#[test]
fn the_source_binds_to_the_data_context_of_the_paints_artboard() {
    let file = read_file(factory());
    let host = file.with_file(|f| f.artboard_named("Host")).unwrap();
    let p = paint_of(&host);
    let red = bound_shape(&p);
    let vm = bind_instance(&file, &host);
    red_x(&vm, 60.0);
    host.bind_view_model_instance(Some(vm.clone()));
    advance(&host, 0.0);
    approx(x(&red), 60.0);
    host.draw_internal(&mut nuxie_render_api::NullRenderer);
    assert!(!host.with_artboard(|a| a.did_change()));
    red_x(&vm, 75.0);
    advance(&host, 0.0);
    approx(x(&red), 75.0);
    assert!(host.with_artboard(|a| a.did_change()));
}
#[test]
fn an_unbound_host_leaves_the_source_as_authored() {
    let file = read_file(factory());
    let host = file.with_file(|f| f.artboard_named("Host")).unwrap();
    let red = bound_shape(&paint_of(&host));
    advance(&host, 0.0);
    approx(x(&red), 25.0);
}
#[test]
fn only_a_paint_with_an_artboard_source_hosts_an_artboard() {
    let f = factory();
    let source = Scene::source(f.clone(), 100.0, 50.0);
    let image_only = Scene::host(f.clone(), None, 10.0, 10.0, 0);
    let artboard_paint = Scene::host(f, Some(source.artboard.clone()), 10.0, 10.0, 0);
    let plain = image_only.instance();
    let painted = artboard_paint.instance();
    paint_of(&plain)
        .with_downcast::<PaintImage, _>(|p| {
            assert_eq!(p.quiet_state(), QuietState::Quiet);
            assert_eq!(p.artboard_count(), 0);
        })
        .unwrap();
    let paint = paint_of(&painted);
    paint
        .with_downcast::<PaintImage, _>(|p| {
            assert_eq!(p.quiet_state(), QuietState::Never);
            assert_eq!(p.artboard_count(), 1);
            assert_eq!(
                p.source
                    .as_ref()
                    .unwrap()
                    .artboard()
                    .with_artboard(|a| a.host()),
                Some(paint.clone())
            );
            assert_eq!(p.parent_artboard(), Some(painted.core_handle()));
        })
        .unwrap();
}
