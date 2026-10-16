//! Opaque hosted artboards must not turn preferred sizes into content minima.
#![cfg(feature = "tools")]
use nuxie_render_api::{NullFactory, PersistentFactory};
use nuxie_runtime::source::layout_component::LayoutComponent;
use nuxie_runtime::{File, RuntimeFactoryHandle};

fn fixture_path(name: &str) -> std::path::PathBuf {
    std::path::PathBuf::from(std::env::var_os("RIVE_RUNTIME_DIR").unwrap())
        .join("tests/unit_tests/assets")
        .join(name)
}

#[test]
fn nested_artboard_keeps_the_parent_clip_at_its_572_point_offer() {
    let mut factory = PersistentFactory::new(NullFactory);
    let file = File::import(
        &std::fs::read(fixture_path("hit_test_test.riv")).unwrap(),
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .unwrap();
    let board = file.with_file(|f| f.artboard_named("ab1-parent")).unwrap();
    board.advance_default(0.0);
    let clipper = board
        .with_artboard(|a| a.find_handle::<LayoutComponent>("parent-clipper"))
        .unwrap();
    let width = clipper
        .with(|o| o.as_layout_component().unwrap().layout_width())
        .unwrap();
    assert!(
        (width - 572.0).abs() < 0.001,
        "parent-clipper width {width}"
    );
}

#[test]
fn an_opaque_10000_point_artboard_does_not_expand_the_1008_point_panel() {
    let mut factory = PersistentFactory::new(NullFactory);
    let vm = nuxie_runtime::source::lua::scripting_vm::RuntimeScriptingVmHandle::new(Box::new(
        nuxie_scripting::vm::ScriptVm::new(),
    ));
    let file = File::import(
        &std::fs::read(fixture_path("db_health_tracker.riv")).unwrap(),
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        Some(vm),
    )
    .unwrap();
    let board = file.with_file(|f| f.artboard_default()).unwrap();
    let model_id = board.with_artboard(|a| a.view_model_id());
    let model = file
        .with_file_mut(|f| f.create_view_model_instance_at(model_id as usize, 0))
        .unwrap();
    board.bind_view_model_instance(Some(model.clone()));
    let machine = board.state_machine_at(0).unwrap();
    machine.with_instance_mut(|m| m.bind_view_model_instance(model));
    machine.advance_and_apply(0.1);
    // This pinned panel has no name. Its stable source object is index 5;
    // C++ and main give it 1080 - 2*16 - 2*20 = 1008 points.
    let panel = board.with_artboard(|a| a.objects()[5].clone()).unwrap();
    let width = panel
        .with(|o| o.as_layout_component().unwrap().layout_width())
        .unwrap();
    assert!((width - 1008.0).abs() < 0.001, "panel width {width}");
}

#[test]
fn scroll_threshold_viewports_keep_their_content_and_scroll_range() {
    use nuxie_runtime::source::constraints::scrolling::scroll_constraint::ScrollConstraint;
    for (name, content_width) in [("all-scroll", 1500.0), ("horizontal-scroll", 1700.0)] {
        let mut factory = PersistentFactory::new(NullFactory);
        let file = File::import(
            &std::fs::read(fixture_path("scroll_threshold.riv")).unwrap(),
            RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
            None,
            None,
            None,
        )
        .unwrap();
        let board = file.with_file(|f| f.artboard_named(name)).unwrap();
        let model_id = board.with_artboard(|a| a.view_model_id());
        let model = file
            .with_file_mut(|f| f.create_view_model_instance_at(model_id as usize, 0))
            .unwrap();
        board.bind_view_model_instance(Some(model.clone()));
        let machine = board.state_machine_at(0).unwrap();
        machine.with_instance_mut(|m| m.bind_view_model_instance(model));
        machine.advance_and_apply(0.1);
        let scroll = board
            .with_artboard(|a| {
                a.objects()
                    .iter()
                    .flatten()
                    .find(|handle| {
                        handle
                            .with(|object| object.as_any().is::<ScrollConstraint>())
                            .unwrap_or(false)
                    })
                    .cloned()
            })
            .unwrap();
        scroll
            .with_downcast::<ScrollConstraint, _>(|scroll| {
                assert_eq!(scroll.viewport_width(), 500.0, "{name}");
                assert_eq!(scroll.content_width(), content_width, "{name}");
                assert_eq!(scroll.max_offset_x(), 500.0 - content_width, "{name}");
            })
            .unwrap();
    }
}
