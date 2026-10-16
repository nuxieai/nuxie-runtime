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

fn assert_fixed_artboard_scroll_content(
    file_name: &str,
    board_name: &str,
    expected: (f32, f32),
    item: (f32, f32),
) {
    use nuxie_runtime::source::constraints::scrolling::scroll_constraint::ScrollConstraint;
    let mut factory = PersistentFactory::new(NullFactory);
    let file = File::import(
        &std::fs::read(fixture_path(file_name)).unwrap(),
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .unwrap();
    let board = file.with_file(|f| f.artboard_named(board_name)).unwrap();
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
    let content = scroll
        .with(|object| object.component_parent_handle())
        .flatten()
        .unwrap();
    let bounds = content
        .with(|object| {
            let layout = object.as_layout_component().unwrap();
            (layout.layout_width(), layout.layout_height())
        })
        .unwrap();
    assert_eq!(
        bounds, expected,
        "{file_name}: CSS allocation of the Hug content"
    );
    let children = content
        .with(|object| object.as_container_component().unwrap().children().to_vec())
        .unwrap();
    let list = children.iter().find_map(|handle| handle.with(|object| {
        object.as_any().downcast_ref::<nuxie_runtime::source::artboard_component_list::ArtboardComponentList>()
            .map(|list| (0..list.artboard_count()).filter_map(|i|list.artboard_instance(i as i32)).collect::<Vec<_>>())
    }).flatten()).unwrap();
    assert!(!list.is_empty());
    for board in list {
        let size = board.with_artboard(|board| (board.layout_width(), board.layout_height()));
        assert_eq!(
            size, item,
            "{file_name}: Fixed items share shrink by their equal bases"
        );
    }
    scroll
        .with_downcast::<ScrollConstraint, _>(|scroll| {
            assert!(
                scroll.max_offset_x() < 0.0 || scroll.max_offset_y() < 0.0,
                "{file_name}: scrolling must retain range"
            );
        })
        .unwrap();
}

#[test]
fn fixed_artboard_items_share_the_draw_index_list_shrink_and_still_loop() {
    // Ten equal 50-point bases share 100 + 9*10 available after negative gaps.
    // Hosted artboards have zero automatic minima, so each gets 19.
    assert_fixed_artboard_scroll_content(
        "draw_index_list.riv",
        "Main",
        (200.0, 100.0),
        (200.0, 19.0),
    );
}

#[test]
fn fixed_artboard_items_share_the_carousel_shrink_and_still_loop() {
    // Four equal 105-point bases share 415 - 3*12 = 379 after gaps.
    // The item allocation is 379/4 = 94.75 and the carousel still loops.
    assert_fixed_artboard_scroll_content(
        "layout/layout_scroll_snap.riv",
        "main",
        (415.0, 94.0),
        (94.75, 94.0),
    );
}
