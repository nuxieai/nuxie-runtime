//! Browser automatic minima through real layout owners and hosted content.
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
fn nested_artboard_floors_the_paint_clipper_at_its_content_minimum() {
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
        (width - 738.65234).abs() < 0.001,
        "parent-clipper width {width}"
    );
}

#[test]
fn a_nested_artboard_in_a_fill_box_keeps_a_zero_automatic_minimum() {
    let mut factory = PersistentFactory::new(NullFactory);
    let file = File::import(
        &std::fs::read(fixture_path("hit_test_test.riv")).unwrap(),
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .unwrap();
    let board = file
        .with_file(|f| f.artboard_named("ab-shape-parent"))
        .unwrap();
    board.advance_default(0.0);
    // Two Fill boxes share a 500-point row; the second holds a Cover-fitted
    // nested artboard. An opaque host in a Fill box has neither a definite
    // nor a hugging size, so it adds no minimum: both keep C++'s 250.
    let widths = [12, 15].map(|index| {
        board
            .with_artboard(|a| a.objects()[index].clone())
            .unwrap()
            .with(|o| o.as_layout_component().unwrap().layout_width())
            .unwrap()
    });
    assert_eq!(widths, [250.0, 250.0]);
}

#[test]
fn definite_list_artboards_keep_their_height_in_a_fill_column() {
    let mut factory = PersistentFactory::new(NullFactory);
    let file = File::import(
        &std::fs::read(fixture_path("component_list_child_origin.riv")).unwrap(),
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .unwrap();
    let board = file.with_file(|f| f.artboard_named("Artboard")).unwrap();
    let model_id = board.with_artboard(|a| a.view_model_id());
    let model = file
        .with_file_mut(|f| f.create_view_model_instance_at(model_id as usize, 0))
        .unwrap();
    board.bind_view_model_instance(Some(model.clone()));
    let machine = board.state_machine_at(0).unwrap();
    machine.with_instance_mut(|m| m.bind_view_model_instance(model));
    machine.advance_and_apply(0.1);
    // Every list item is an artboard with a definite 58-point height. As an
    // opaque host it keeps that size, as in C++, even where six items share
    // a 200-point Fill column and overflow it.
    let heights = board
        .with_artboard(|a| {
            a.objects()
                .iter()
                .flatten()
                .filter_map(|handle| {
                    handle
                        .with(|object| {
                            object
                                .as_any()
                                .downcast_ref::<nuxie_runtime::source::artboard_component_list::ArtboardComponentList>()
                                .map(|list| {
                                    (0..list.artboard_count())
                                        .filter_map(|i| list.artboard_instance(i as i32))
                                        .collect::<Vec<_>>()
                                })
                        })
                        .flatten()
                })
                .flatten()
                .collect::<Vec<_>>()
        })
        .iter()
        .map(|item| item.with_artboard(|item| item.layout_height()))
        .collect::<Vec<_>>();
    assert!(heights.len() >= 6, "{heights:?}");
    assert!(heights.iter().all(|height| *height == 58.0), "{heights:?}");
}

#[test]
fn a_definite_10000_point_artboard_contributes_to_the_panel_minimum() {
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
    // C++ and main give it 1008 points. The browser minimum includes the
    // definite 10000-point child and 174 points of surrounding content.
    let panel = board.with_artboard(|a| a.objects()[5].clone()).unwrap();
    let width = panel
        .with(|o| o.as_layout_component().unwrap().layout_width())
        .unwrap();
    assert!((width - 10174.0).abs() < 0.001, "panel width {width}");
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
    let children = content
        .with(|object| object.as_container_component().unwrap().children().to_vec())
        .unwrap();
    let (list, virtual_size) = children.iter().find_map(|handle| handle.with(|object| {
        object.as_any().downcast_ref::<nuxie_runtime::source::artboard_component_list::ArtboardComponentList>()
            .map(|list| ((0..list.artboard_count()).filter_map(|i|list.artboard_instance(i as i32)).collect::<Vec<_>>(), list.size()))
    }).flatten()).unwrap();
    assert!(!list.is_empty());
    let sizes = list
        .iter()
        .map(|board| board.with_artboard(|board| (board.layout_width(), board.layout_height())))
        .collect::<Vec<_>>();
    let total = if file_name == "draw_index_list.riv" {
        (200.0, 410.0)
    } else {
        // Twenty virtual rows: 20*105 + 19*12 = 2328 points.
        (2328.0, 94.0)
    };
    assert_eq!(
        (virtual_size.x, virtual_size.y),
        total,
        "{file_name}: full virtual content"
    );
    assert_eq!(
        bounds, expected,
        "{file_name}: CSS allocation of the Hug content"
    );
    for size in sizes {
        assert_eq!(size, item, "{file_name}: definite artboard suggestion");
    }
    scroll
        .with_downcast::<ScrollConstraint, _>(|scroll| {
            assert!(
                if file_name == "draw_index_list.riv" {
                    scroll.max_offset_y() < 0.0
                } else {
                    scroll.max_offset_x() < 0.0
                },
                "{file_name}: scrolling must retain range"
            );
        })
        .unwrap();
}

#[test]
fn fixed_artboard_items_keep_the_draw_index_list_minimum_and_still_loop() {
    // Ten definite 50-point items and nine -10 gaps give 410 virtual points.
    // C++ mounts three rows: their native content box is 3*50 - 2*10 = 130.
    assert_fixed_artboard_scroll_content(
        "draw_index_list.riv",
        "Main",
        (200.0, 130.0),
        (200.0, 50.0),
    );
}

#[test]
fn fixed_artboard_items_keep_the_carousel_minimum_and_still_loop() {
    // Four definite 105-point items and three 12 gaps contribute 456 points.
    assert_fixed_artboard_scroll_content(
        "layout/layout_scroll_snap.riv",
        "main",
        (456.0, 94.0),
        (105.0, 94.0),
    );
}

#[test]
fn css_fixed_clock_graphic_keeps_its_natural_minimum_through_solo() {
    let mut widths = Vec::new();
    for asset in [
        "layout/text_layout_7_3.riv",
        "layout/text_layout_pre_7_3.riv",
    ] {
        let mut factory = PersistentFactory::new(NullFactory);
        let file = File::import(
            &std::fs::read(fixture_path(asset)).unwrap(),
            RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
            None,
            None,
            None,
        )
        .unwrap();
        let board = file.with_file(|file| file.artboard_default()).unwrap();
        if let Some(machine) = board.state_machine_at(0) {
            machine.advance_and_apply(0.0);
        } else {
            board.advance_default(0.0);
        }
        let graphic = board
            .with_artboard(|board| {
                board
                    .objects()
                    .iter()
                    .flatten()
                    .find(|handle| {
                        handle
                            .with(|object| {
                                object.as_layout_component().is_some_and(|layout| {
                                    (layout.base.width() - 17.300781).abs() < 0.00001
                                })
                            })
                            .unwrap_or(false)
                    })
                    .cloned()
            })
            .expect("the authored 17.300781-point vector box");
        widths.push(
            graphic
                .with(|object| object.as_layout_component().unwrap().layout_width())
                .unwrap(),
        );
    }
    assert_eq!(widths, vec![17.300781; 2]);
}

mod natural_minima {
    use nuxie_runtime::source::{
        core::{CoreArena, CoreHandle},
        core_context::{CoreContext, StatusCode},
        generated::core_registry::CoreField,
        layout::{
            layout_component_style::LayoutComponentStyle,
            layout_enums::{LayoutAlignmentType, LayoutScaleType},
            layout_style_applier::{YGFlexDirection, YGUnit},
        },
        layout_component::LayoutComponent,
    };
    struct ParentContext<'a> {
        arena: &'a CoreArena,
        parent: CoreHandle,
    }
    impl CoreContext for ParentContext<'_> {
        fn core_arena(&self) -> &CoreArena {
            self.arena
        }
        fn resolve_handle(&self, id: u32) -> Option<CoreHandle> {
            (id == 1).then(|| self.parent.clone())
        }
    }
    fn uint(owner: &CoreHandle, field: CoreField, value: u32) {
        owner
            .with_mut(|o| {
                o.set_uint(field, value);
                assert_eq!(o.get_uint(field), value);
            })
            .unwrap();
    }
    fn float(owner: &CoreHandle, field: CoreField, value: f32) {
        owner
            .with_mut(|o| {
                o.set_double(field, value);
                assert_eq!(o.get_double(field), value);
            })
            .unwrap();
    }
    fn attach(arena: &CoreArena, parent: &CoreHandle, child: &CoreHandle) {
        uint(child, CoreField::ComponentParentId, 1);
        let mut context = ParentContext {
            arena,
            parent: parent.clone(),
        };
        assert_eq!(
            child.with_mut(|o| o.as_component_mut().unwrap().on_added_dirty(&mut context)),
            Some(StatusCode::Ok)
        );
    }
    fn layout(
        arena: &CoreArena,
        parent: Option<&CoreHandle>,
        width: Option<f32>,
        row: bool,
    ) -> CoreHandle {
        let owner = arena.insert(LayoutComponent::default());
        if let Some(parent) = parent {
            attach(arena, parent, &owner);
        }
        let style = arena.insert(LayoutComponentStyle::default());
        uint(
            &style,
            CoreField::LayoutSizingStyleLayoutWidthScaleType,
            if width.is_some() {
                LayoutScaleType::Fixed
            } else {
                LayoutScaleType::Hug
            } as u32,
        );
        uint(
            &style,
            CoreField::LayoutSizingStyleLayoutHeightScaleType,
            LayoutScaleType::Hug as u32,
        );
        uint(
            &style,
            CoreField::LayoutSizingStyleWidthUnitsValue,
            if width.is_some() {
                YGUnit::Point
            } else {
                YGUnit::Auto
            } as u32,
        );
        uint(
            &style,
            CoreField::LayoutSizingStyleHeightUnitsValue,
            YGUnit::Auto as u32,
        );
        uint(
            &style,
            CoreField::LayoutComponentStyleFlexDirectionValue,
            if row {
                YGFlexDirection::Row
            } else {
                YGFlexDirection::Column
            } as u32,
        );
        uint(
            &style,
            CoreField::LayoutComponentStyleLayoutAlignmentType,
            LayoutAlignmentType::TopCenter as u32,
        );
        style
            .with_mut(|o| {
                o.set_bool(
                    CoreField::LayoutComponentStyleIntrinsicallySizedValue,
                    width.is_none(),
                )
            })
            .unwrap();
        float(
            &owner,
            CoreField::LayoutComponentWidth,
            width.unwrap_or(0.0),
        );
        float(&owner, CoreField::LayoutComponentHeight, 800.0);
        owner
            .with_downcast_mut::<LayoutComponent, _>(|o| {
                o.set_style(Some(style.clone()));
                o.add_layout_style_applier(owner.clone());
                o.add_layout_style_applier(style.clone());
            })
            .unwrap();
        owner
    }

    fn natural_sizes(kind: u8, zero_minimum: bool) -> Vec<f32> {
        use nuxie_runtime::source::shapes::{rectangle::Rectangle, shape::Shape};
        let arena = CoreArena::default();
        let root = layout(&arena, None, Some(100.0), true);
        let subject = layout(&arena, Some(&root), None, false);
        match kind {
            0 => {
                let shape = arena.insert(Shape::default());
                attach(&arena, &subject, &shape);
                let rectangle = arena.insert(Rectangle::default());
                float(&rectangle, CoreField::ParametricPathWidth, 80.0);
                float(&rectangle, CoreField::ParametricPathHeight, 30.0);
                attach(&arena, &shape, &rectangle);
                shape
                    .with_downcast_mut::<Shape, _>(|s| s.add_path(rectangle.clone()))
                    .unwrap();
            }
            1 => {
                let host = arena
                    .insert(nuxie_runtime::source::layout::n_sliced_node::NSlicedNode::default());
                float(&host, CoreField::NSlicedNodeWidth, 80.0);
                float(&host, CoreField::NSlicedNodeHeight, 30.0);
                attach(&arena, &subject, &host);
            }
            2 => {
                let host = arena.insert(nuxie_runtime::source::joystick::Joystick::default());
                float(&host, CoreField::JoystickWidth, 80.0);
                float(&host, CoreField::JoystickHeight, 30.0);
                attach(&arena, &subject, &host);
            }
            _ => unreachable!(),
        }
        let sibling = layout(&arena, Some(&root), Some(60.0), false);
        for node in [&sibling, &subject, &root] {
            LayoutComponent::sync_style_occurrence(node);
            LayoutComponent::sync_layout_children_occurrence(node);
        }
        let mut sizes = Vec::new();
        let style = subject
            .with_downcast::<LayoutComponent, _>(LayoutComponent::style_handle)
            .flatten()
            .unwrap();
        if zero_minimum {
            uint(
                &style,
                CoreField::LayoutSizingStyleMinWidthUnitsValue,
                YGUnit::Point as u32,
            );
            float(&style, CoreField::LayoutSizingStyleMinWidth, 0.0);
        }
        for (offer, cap) in [
            (100.0, None),
            (100.0, None),
            (100.0, Some(40.0)),
            (200.0, None),
            (100.0, None),
            (200.0, None),
        ] {
            uint(
                &style,
                CoreField::LayoutSizingStyleMaxWidthUnitsValue,
                if cap.is_some() {
                    YGUnit::Point
                } else {
                    YGUnit::Auto
                } as u32,
            );
            float(
                &style,
                CoreField::LayoutSizingStyleMaxWidth,
                cap.unwrap_or(0.0),
            );
            LayoutComponent::sync_style_occurrence(&subject);
            float(&root, CoreField::LayoutComponentWidth, offer);
            LayoutComponent::sync_style_occurrence(&root);
            LayoutComponent::calculate_layout_occurrence(&root, f32::NAN, f32::NAN);
            LayoutComponent::update_layout_bounds_occurrence(&root, false);
            sizes.push(
                subject
                    .with_downcast::<LayoutComponent, _>(|l| l.layout_bounds().width())
                    .unwrap(),
            );
        }
        sizes
    }

    #[test]
    fn css_natural_shape_minimum_is_stable_and_restores_after_widening() {
        let sizes = natural_sizes(0, false);
        // A non-reflowing 80-point graphic keeps its natural min-content.
        // An authored maximum may cap it, but removing that cap restores 80.
        assert_eq!(sizes, vec![80.0, 80.0, 40.0, 80.0, 80.0, 80.0]);
    }
    #[test]
    fn css_shape_with_zero_minimum_keeps_its_authored_flex_basis() {
        let sizes = natural_sizes(0, true);
        // With min-width: 0, the 80 and 60 bases share the 40-point deficit.
        let expected = [57.142857, 57.142857, 40.0, 80.0, 57.142857, 80.0];
        for (actual, expected) in sizes.iter().zip(expected) {
            assert!((actual - expected).abs() < 0.0001, "{sizes:?}");
        }
    }
    #[test]
    fn css_nonreflowing_hosts_keep_their_natural_minimum() {
        let sizes: Vec<_> = (1..=2).map(|kind| natural_sizes(kind, false)).collect();
        assert_eq!(sizes, vec![vec![80.0, 80.0, 40.0, 80.0, 80.0, 80.0]; 2]);
    }
}
