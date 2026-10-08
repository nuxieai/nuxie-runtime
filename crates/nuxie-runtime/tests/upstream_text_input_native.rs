//! Preserved text-input assertions, now exercising the actual native owners.
//! Pinned authority: tests/unit_tests/runtime/text_input_test.cpp.

use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::{
    core::{CoreHandle, CoreType},
    focus_data::FocusData,
    generated::core_registry::CoreRegistry,
    input::focusable::{Key, KeyModifiers},
    text::{
        cursor::{Cursor, CursorPosition},
        text_input::TextInput,
    },
};
use nuxie_runtime::{File, RuntimeArtboardInstanceHandle, RuntimeFactoryHandle, RuntimeFileHandle};
use std::path::PathBuf;

fn input_fixture() -> (RuntimeFileHandle, RuntimeArtboardInstanceHandle, CoreHandle) {
    named_input_fixture("text_input.riv", "Text Input - Multiline")
}

fn named_input_fixture(
    asset: &str,
    name: &str,
) -> (RuntimeFileHandle, RuntimeArtboardInstanceHandle, CoreHandle) {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let bytes = std::fs::read(
        PathBuf::from(root)
            .join("tests/unit_tests/assets")
            .join(asset),
    )
    .expect("pinned text input asset");
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let retained = RuntimeFactoryHandle::from_factory(&mut factory).expect("retained factory");
    let file = File::import(&bytes, retained, None, None, None).expect("native File import");
    let artboard = file
        .with_file(|file| file.artboard_named(name))
        .expect("named native artboard");
    let input = artboard
        .with_artboard(|artboard| {
            artboard
                .objects()
                .iter()
                .flatten()
                .find(|object| object.is_type_of(TextInput::TYPE_KEY))
                .cloned()
        })
        .expect("native TextInput");
    (file, artboard, input)
}

fn with_input<R>(handle: &CoreHandle, f: impl FnOnce(&mut TextInput) -> R) -> R {
    handle.with_downcast_mut(f).expect("live TextInput")
}

#[test]
fn caret_hides_while_text_is_selected() {
    use nuxie_runtime::source::text::text_input_cursor::TextInputCursor;
    let (_file, artboard, input) = input_fixture();
    let machine = artboard
        .state_machine_instance_handle(0)
        .expect("state machine");
    machine.advance_and_apply(0.0);
    let find = |key| {
        artboard
            .with_artboard(|a| {
                a.objects()
                    .iter()
                    .flatten()
                    .find(|o| o.is_type_of(key))
                    .cloned()
            })
            .expect("authored child")
    };
    let cursor = find(TextInputCursor::TYPE_KEY);
    machine.with_instance_mut(|m| m.set_focus(Some(find(FocusData::TYPE_KEY))));
    with_input(&input, |i| {
        i.raw_text_input().set_text("hello world".into())
    });
    machine.advance_and_apply(0.0);
    with_input(&input, |i| {
        i.raw_text_input().set_cursor(Cursor::new(
            CursorPosition::unresolved(0),
            CursorPosition::unresolved(5),
        ))
    });
    machine.advance_and_apply(0.0);
    assert!(!cursor_has_local_path(&cursor));
    with_input(&input, |i| {
        i.raw_text_input()
            .set_cursor(Cursor::collapsed(CursorPosition::unresolved(5)))
    });
    machine.advance_and_apply(0.0);
    assert!(cursor_has_local_path(&cursor));
}

#[test]
fn press_past_the_end_of_text_puts_caret_at_end() {
    use nuxie_runtime::source::{math::vec2d::Vec2D, pointer_button::PointerButton};
    for name in ["SingleLine", "Multiline"] {
        let (_file, artboard, input) = named_input_fixture("text_input_tray.riv", name);
        let machine = artboard
            .state_machine_instance_handle(0)
            .expect("state machine");
        CoreRegistry::set_string_handle(
            &input,
            i32::from(property_key_for_name("TextInput", "text")),
            "Hello world".into(),
        );
        machine.advance_and_apply(0.0);
        machine.with_instance_mut(|m| {
            m.pointer_down(Vec2D::new(70.0, 160.0), 0, PointerButton::Primary)
        });
        machine.with_instance_mut(|m| {
            m.pointer_up(Vec2D::new(70.0, 160.0), 0, PointerButton::Primary)
        });
        machine.advance_and_apply(0.0);
        assert!(input_cursor(&input).unwrap().1 < 11, "{name}");
        machine.with_instance_mut(|m| {
            m.pointer_down(Vec2D::new(420.0, 160.0), 0, PointerButton::Primary)
        });
        machine.with_instance_mut(|m| {
            m.pointer_up(Vec2D::new(420.0, 160.0), 0, PointerButton::Primary)
        });
        machine.advance_and_apply(0.0);
        assert_eq!(input_cursor(&input), Some((11, 11)), "{name}");
    }
}

#[test]
fn undo_reaches_first_edit_and_stops_at_initial_text() {
    let (_file, artboard, input) = named_input_fixture("text_input_tray.riv", "SingleLine");
    artboard.advance_default(0.0);
    let undo = || {
        with_input(&input, |i| {
            i.key_input(
                Key::from_raw(90),
                KeyModifiers::from_raw(8 | 2),
                true,
                false,
            )
        })
    };
    with_input(&input, |i| i.text_input("abc"));
    undo();
    assert_eq!(with_input(&input, |i| i.base.text().to_owned()), "");
    CoreRegistry::set_string_handle(
        &input,
        i32::from(property_key_for_name("TextInput", "text")),
        "Hello".into(),
    );
    artboard.advance_default(0.0);
    with_input(&input, |i| i.text_input("!"));
    assert_eq!(with_input(&input, |i| i.base.text().to_owned()), "!Hello");
    undo();
    assert_eq!(with_input(&input, |i| i.base.text().to_owned()), "Hello");
    undo();
    assert_eq!(with_input(&input, |i| i.base.text().to_owned()), "Hello");
}

#[test]
fn state_machine_selected_text_reports_the_focused_inputs_selection() {
    let (_file, artboard, input) = input_fixture();
    let machine = artboard
        .state_machine_instance_handle(0)
        .expect("authored state machine");
    machine.advance_and_apply(0.0);
    let focus = artboard
        .with_artboard(|artboard| {
            artboard
                .objects()
                .iter()
                .flatten()
                .find(|object| object.is_type_of(FocusData::TYPE_KEY))
                .cloned()
        })
        .expect("authored FocusData");

    with_input(&input, |input| {
        input.raw_text_input().set_text("hello world".into());
    });
    machine.advance_and_apply(0.0);

    // Nothing focused yet.
    assert!(machine.with_instance(|machine| machine.selected_text().is_empty()));

    machine.with_instance_mut(|machine| machine.set_focus(Some(focus)));
    with_input(&input, |input| input.raw_text_input().clear_selection());
    assert!(machine.with_instance(|machine| machine.selected_text().is_empty()));

    with_input(&input, |input| input.raw_text_input().select_all());
    assert_eq!(
        machine.with_instance(|machine| machine.selected_text()),
        "hello world"
    );

    // A partial selection reports just that range (2nd–4th characters).
    with_input(&input, |input| {
        input.raw_text_input().set_cursor(Cursor::new(
            CursorPosition::unresolved(1),
            CursorPosition::unresolved(4),
        ));
    });
    assert_eq!(
        machine.with_instance(|machine| machine.selected_text()),
        "ell"
    );

    // An obscured field keeps its selection off the clipboard.
    assert!(CoreRegistry::set_bool_handle(&input, 1095, true));
    assert!(machine.with_instance(|machine| machine.selected_text().is_empty()));
    assert!(CoreRegistry::set_bool_handle(&input, 1095, false));

    machine.with_instance_mut(|machine| machine.clear_focus());
    assert!(machine.with_instance(|machine| machine.selected_text().is_empty()));
}

#[test]
fn tab_traversal_into_a_text_input_selects_all() {
    let (_file, artboard, input) = input_fixture();
    let machine = artboard
        .state_machine_instance_handle(0)
        .expect("authored state machine");
    machine.advance_and_apply(0.0);
    with_input(&input, |input| {
        input.raw_text_input().set_text("hello world".into());
    });
    let focus = artboard
        .with_artboard(|artboard| {
            artboard
                .objects()
                .iter()
                .flatten()
                .find(|object| object.is_type_of(FocusData::TYPE_KEY))
                .cloned()
        })
        .expect("authored FocusData");

    // Target focus keeps the caret where it was.
    machine.with_instance_mut(|machine| machine.set_focus(Some(focus)));
    assert!(with_input(&input, |input| input.is_focused()));
    assert!(with_input(&input, |input| input
        .raw_text_input()
        .cursor()
        .is_collapsed()));

    machine.with_instance_mut(|machine| machine.clear_focus());
    assert!(machine.with_instance_mut(|machine| machine.focus_next()));
    assert!(with_input(&input, |input| input.is_focused()));
    assert_eq!(
        with_input(&input, |input| input.raw_text_input().selected_text()),
        "hello world"
    );
}

#[test]
fn select_all_on_focus_selects_on_target_focus_too() {
    let (_file, artboard, input) = input_fixture();
    let machine = artboard
        .state_machine_instance_handle(0)
        .expect("authored state machine");
    machine.advance_and_apply(0.0);
    with_input(&input, |input| {
        input.raw_text_input().set_text("hello world".into());
        input.set_select_all_on_focus(true);
    });
    let focus = artboard
        .with_artboard(|artboard| {
            artboard
                .objects()
                .iter()
                .flatten()
                .find(|object| object.is_type_of(FocusData::TYPE_KEY))
                .cloned()
        })
        .expect("authored FocusData");
    machine.with_instance_mut(|machine| machine.set_focus(Some(focus.clone())));
    assert_eq!(
        with_input(&input, |input| input.raw_text_input().selected_text()),
        "hello world"
    );

    // Losing focus drops the selection, so the next focus selects again.
    machine.with_instance_mut(|machine| machine.clear_focus());
    assert!(with_input(&input, |input| input
        .raw_text_input()
        .cursor()
        .is_collapsed()));
    machine.with_instance_mut(|machine| machine.set_focus(Some(focus)));
    assert_eq!(
        with_input(&input, |input| input.raw_text_input().selected_text()),
        "hello world"
    );
}

#[test]
fn select_all_on_focus_press_selects_all_later_press_places_the_caret() {
    use nuxie_runtime::source::math::{aabb::Aabb, vec2d::Vec2D};

    let (_file, artboard, input) = input_fixture();
    let machine = artboard
        .state_machine_instance_handle(0)
        .expect("authored state machine");
    machine.advance_and_apply(0.0);
    with_input(&input, |input| {
        input.raw_text_input().set_text("hello world".into());
        input.set_select_all_on_focus(true);
    });
    machine.advance_and_apply(0.0);

    // Press inside the input wherever the asset lays it out.
    let mut bounds = Aabb::default();
    assert!(with_input(&input, |input| input.world_bounds(&mut bounds)));
    let press_position = Vec2D::new(bounds.left() + 8.0, bounds.top() + 8.0);
    machine.with_instance_mut(|machine| {
        machine.pointer_down(
            press_position,
            0,
            nuxie_runtime::source::pointer_button::PointerButton::Primary,
        )
    });
    machine.with_instance_mut(|machine| {
        machine.pointer_up(
            press_position,
            0,
            nuxie_runtime::source::pointer_button::PointerButton::Primary,
        )
    });
    machine.advance_and_apply(0.0);
    assert!(with_input(&input, |input| input.is_focused()));
    assert_eq!(
        with_input(&input, |input| input.raw_text_input().selected_text()),
        "hello world"
    );

    // Already focused, a press places the caret and a drag extends from it.
    // Far enough from the first press not to count as a double click.
    let second_press = Vec2D::new(bounds.left() + 40.0, bounds.top() + 8.0);
    machine.with_instance_mut(|machine| {
        machine.pointer_down(
            second_press,
            0,
            nuxie_runtime::source::pointer_button::PointerButton::Primary,
        )
    });
    machine.advance_and_apply(0.0);
    assert!(with_input(&input, |input| input
        .raw_text_input()
        .cursor()
        .is_collapsed()));
    machine.with_instance_mut(|machine| machine.pointer_move(press_position, 0.0, 0));
    machine.advance_and_apply(0.0);
    assert!(with_input(&input, |input| input
        .raw_text_input()
        .cursor()
        .has_selection()));
    assert_ne!(
        with_input(&input, |input| input.raw_text_input().selected_text()),
        "hello world"
    );
    machine.with_instance_mut(|machine| {
        machine.pointer_up(
            press_position,
            0,
            nuxie_runtime::source::pointer_button::PointerButton::Primary,
        )
    });
}

#[test]
fn collapsing_an_artboard_ends_a_text_input_drag() {
    use nuxie_runtime::source::{
        generated::transform_component_base::TransformComponentBase, math::vec2d::Vec2D,
    };
    let (_file, artboard, input) = input_fixture();
    let machine = artboard.state_machine_instance_handle(0).unwrap();
    machine.advance_and_apply(0.0);
    with_input(&input, |input| {
        input.raw_text_input().set_text("hello world".into())
    });
    machine.advance_and_apply(0.0);
    machine.with_instance_mut(|m| {
        m.pointer_down(
            Vec2D::new(8.0, 8.0),
            0,
            nuxie_runtime::source::pointer_button::PointerButton::Primary,
        )
    });
    assert!(with_input(&input, |input| input.is_dragging()));
    for key in [
        TransformComponentBase::SCALE_X_PROPERTY_KEY,
        TransformComponentBase::SCALE_Y_PROPERTY_KEY,
    ] {
        assert!(CoreRegistry::set_double_handle(
            &artboard.core_handle(),
            i32::from(key),
            0.0
        ));
    }
    artboard.advance_default(0.0);
    machine.with_instance_mut(|m| m.pointer_move(Vec2D::new(20.0, 8.0), 0.0, 0));
    assert!(!with_input(&input, |input| input.is_dragging()));
}

#[test]
fn empty_shaped_text_has_no_selection_rectangles() {
    use nuxie_runtime::source::{math::aabb::Aabb, text::fully_shaped_text::FullyShapedText};
    let shape = FullyShapedText::default();
    let existing = Aabb::new(1.0, 2.0, 3.0, 4.0);
    let mut rectangles = vec![existing];
    for cursor in [
        Cursor::zero(),
        Cursor::new(CursorPosition::new(3, 9), CursorPosition::unresolved(40)),
    ] {
        cursor.selection_rects(&mut rectangles, &shape.layout_view());
    }
    assert_eq!(
        rectangles,
        vec![existing],
        "empty text appends no selection geometry"
    );
}

#[test]
fn native_input_point_hit_respects_drawable_hidden_flag() {
    let (_file, artboard, input) = input_fixture();
    artboard.advance_default(0.0);
    // 0dd067f1 makes the scroll viewport authoritative. This fixture's text
    // extends below its 50pt viewport, so its text center is not a hit point.
    let viewport = artboard
        .with_artboard(|artboard| {
            artboard.objects().iter().flatten().find_map(|object| {
                object
                    .with(|object| object.as_scroll_constraint()?.viewport_handle())
                    .flatten()
            })
        })
        .expect("fixture scroll viewport");
    let point = viewport
        .with(|viewport| {
            let viewport = viewport.as_layout_component().expect("layout viewport");
            let center = viewport.local_bounds().center();
            assert!(viewport.local_bounds().contains(center));
            *viewport.world_transform() * center
        })
        .expect("live viewport");
    assert!(with_input(&input, |input| input.hit_test_point(point, false, true)));
    let flags = CoreRegistry::get_uint_handle(&input, 129).unwrap();
    let hidden = nuxie_runtime::source::drawable_flag::DrawableFlag::HIDDEN.0;
    assert!(CoreRegistry::set_uint_handle(
        &input,
        129,
        flags | u32::from(hidden)
    ));
    assert!(!with_input(&input, |input| input.hit_test_point(point, false, true)));
    assert!(CoreRegistry::set_uint_handle(&input, 129, flags));
    assert!(with_input(&input, |input| input.hit_test_point(point, false, true)));
}

#[test]
fn native_input_alignment_schema_deserialization_and_clone_agree() {
    use nuxie_runtime::source::core::{CoreObject, binary_reader::BinaryReader};
    let mut input = TextInput::default();
    for key in [222, 1094] {
        let (_, property) = nuxie_schema::property_by_key_in_hierarchy(569, key).unwrap();
        assert_eq!(property.key.int, key);
        assert_eq!(property.runtime_type, nuxie_schema::FieldKind::Uint);
        assert_eq!(
            nuxie_schema::core_registry_setter_field_kind_by_property_key(key),
            Some(nuxie_schema::FieldKind::Uint)
        );
        assert_eq!(
            nuxie_schema::core_registry_getter_field_kind_by_property_key(key),
            Some(nuxie_schema::FieldKind::Uint)
        );
        assert!(input.deserialize(key, &mut BinaryReader::new(&[2])));
    }
    let cloned = input.clone_boxed().unwrap();
    let cloned = cloned.as_text_input().unwrap();
    assert_eq!(cloned.base.align_value(), 2);
    assert_eq!(cloned.base.vertical_align_value(), 2);
    for key in [817, 818, 979, 1095] {
        assert_eq!(
            nuxie_schema::property_by_key_in_hierarchy(569, key)
                .unwrap()
                .1
                .key
                .int,
            key
        );
    }
}

// Upstream 7098a7c8: alignment moves text within the field, not its intrinsic
// dimensions. Exercise the registry entry points used by imported bindings.
#[test]
fn native_input_alignment_properties_drive_field_relative_geometry() {
    let (_file, artboard, input) = input_fixture();
    assert_eq!(CoreRegistry::get_uint_handle(&input, 222), Some(0));
    assert!(CoreRegistry::set_string_handle(&input, 817, "hi".into()));
    artboard.advance_default(0.0);
    let (width, height, natural) = with_input(&input, |input| {
        let bounds = input.local_bounds();
        let raw = input.raw_text_input();
        (raw.align_width(), raw.align_height(), bounds)
    });
    assert!(width > natural.width());
    assert!(width > 0.0);
    assert_eq!(natural.min_x, 0.0);
    assert!(height > natural.height());
    for (horizontal, x_factor) in [(0, 0.0), (1, 1.0), (2, 0.5)] {
        for (vertical, y_factor) in [(0, 0.0), (1, 1.0), (2, 0.5)] {
            assert!(CoreRegistry::set_uint_handle(&input, 222, horizontal));
            assert!(CoreRegistry::set_uint_handle(&input, 1094, vertical));
            assert_eq!(CoreRegistry::get_uint_handle(&input, 222), Some(horizontal));
            assert_eq!(CoreRegistry::get_uint_handle(&input, 1094), Some(vertical));
            artboard.advance_default(0.0);
            assert_eq!(
                with_input(&input, |input| input.raw_text_input().align()),
                match horizontal {
                    0 => nuxie_runtime::source::text::text_engine::TextAlign::Left,
                    1 => nuxie_runtime::source::text::text_engine::TextAlign::Right,
                    2 => nuxie_runtime::source::text::text_engine::TextAlign::Center,
                    _ => unreachable!(),
                }
            );
            let bounds = with_input(&input, |input| input.local_bounds());
            assert!((bounds.min_x - (width - natural.width()) * x_factor).abs() < 0.01);
            assert!((bounds.min_y - (height - natural.height()) * y_factor).abs() < 0.01);
            assert!((bounds.width() - natural.width()).abs() < 0.01);
            assert!((bounds.height() - natural.height()).abs() < 0.01);
        }
    }
}

// Upstream's viewport-padding regression, including a second change after the
// first layout to prove alignment is refreshed rather than captured at import.
#[test]
fn native_input_alignment_tracks_viewport_padding_changes() {
    let (_file, artboard, input) = input_fixture();
    artboard.advance_default(0.0);
    let size = with_input(&input, |input| {
        let raw = input.raw_text_input();
        (raw.align_width(), raw.align_height())
    });
    assert!(size.0 > 0.0 && size.1 > 0.0);
    let mut viewport = input.clone();
    for _ in 0..3 {
        viewport = viewport
            .with(|node| node.component_parent_handle())
            .flatten()
            .expect("viewport ancestry");
    }
    let style = viewport
        .with(|node| {
            node.as_layout_component()
                .and_then(|layout| layout.style_handle())
        })
        .flatten()
        .expect("viewport style");
    for (left, right, top, bottom) in [
        (12.0, 8.0, 5.0, 3.0),
        (3.0, 1.0, 2.0, 4.0),
        (0.0, 0.0, 0.0, 0.0),
    ] {
        for (property, value) in [(512, left), (513, right), (514, top), (515, bottom)] {
            assert!(CoreRegistry::set_double_handle(&style, property, value));
        }
        artboard.advance_default(0.0);
        let padding_left = viewport
            .with(|node| node.as_layout_component().unwrap().padding_left())
            .unwrap();
        assert!((padding_left - left).abs() < 0.01);
        let aligned = with_input(&input, |input| {
            let raw = input.raw_text_input();
            (raw.align_width(), raw.align_height())
        });
        assert!(
            (aligned.0 - (size.0 - left - right)).abs() < 0.01,
            "width: {aligned:?}"
        );
        assert!(
            (aligned.1 - (size.1 - top - bottom)).abs() < 0.01,
            "height: {aligned:?}"
        );
    }
}

// The upstream cursor-path assertions exercise the drawable's actual local
// path gate, not just the TextInput visibility state used to implement it.
fn cursor_has_local_path(cursor: &CoreHandle) -> bool {
    use nuxie_runtime::source::{
        shapes::paint::shape_paint::ShapePaintPathKind, text::text_input_cursor::TextInputCursor,
    };
    cursor
        .with_downcast::<TextInputCursor, _>(|cursor| {
            cursor.with_path_mut(ShapePaintPathKind::LocalClockwise, &mut |_| {})
        })
        .expect("live cursor")
}

#[test]
fn losing_focus_clears_the_text_input_selection() {
    use nuxie_runtime::source::text::text_input_cursor::TextInputCursor;
    let (_file, artboard, input) = input_fixture();
    let machine = artboard
        .state_machine_instance_handle(0)
        .expect("authored state machine");
    machine.advance_and_apply(0.0);
    let find = |type_key| {
        artboard
            .with_artboard(|artboard| {
                artboard
                    .objects()
                    .iter()
                    .flatten()
                    .find(|object| object.is_type_of(type_key))
                    .cloned()
            })
            .expect("authored child")
    };
    let cursor = find(TextInputCursor::TYPE_KEY);
    assert!(!with_input(&input, |input| input.is_focused()));
    assert!(!cursor_has_local_path(&cursor));
    let focus = find(FocusData::TYPE_KEY);
    machine.with_instance_mut(|machine| machine.set_focus(Some(focus)));
    assert!(with_input(&input, |input| input.is_focused()));
    assert!(cursor_has_local_path(&cursor));
    with_input(&input, |input| {
        input.raw_text_input().set_text("hello world".into());
        input.raw_text_input().select_all();
    });
    assert!(with_input(&input, |input| input
        .raw_text_input()
        .cursor()
        .has_selection()));
    machine.with_instance_mut(|machine| machine.clear_focus());
    assert!(with_input(&input, |input| input
        .raw_text_input()
        .cursor()
        .is_collapsed()));
    assert_eq!(
        with_input(&input, |input| input
            .raw_text_input()
            .cursor()
            .end()
            .code_point_index()),
        11
    );
    assert!(!with_input(&input, |input| input.is_focused()));
    assert!(!cursor_has_local_path(&cursor));
}

#[test]
fn a_focused_text_input_reports_that_it_accepts_text() {
    let (_file, artboard, input) = input_fixture();
    let machine = artboard
        .state_machine_instance_handle(0)
        .expect("authored state machine");
    machine.advance_and_apply(0.0);
    let manager = machine
        .with_instance(|machine| machine.focus_manager())
        .expect("artboard focus manager");
    assert!(!manager.with_focus_manager(|manager| manager.primary_focus_accepts_text()));

    // The focus target is the FocusData child; the query sees its TextInput parent.
    let focus = artboard
        .with_artboard(|artboard| {
            artboard
                .objects()
                .iter()
                .flatten()
                .find(|object| object.is_type_of(FocusData::TYPE_KEY))
                .cloned()
        })
        .expect("authored FocusData");
    machine.with_instance_mut(|machine| machine.set_focus(Some(focus)));
    assert!(with_input(&input, |input| input.is_focused()));
    assert!(manager.with_focus_manager(|manager| manager.primary_focus_accepts_text()));

    machine.with_instance_mut(|machine| machine.clear_focus());
    assert!(!manager.with_focus_manager(|manager| manager.primary_focus_accepts_text()));
}

#[test]
fn text_input_cursor_blinks_while_focused() {
    use nuxie_runtime::source::text::text_input_cursor::TextInputCursor;
    let (_file, artboard, _input) = input_fixture();
    let machine = artboard
        .state_machine_instance_handle(0)
        .expect("authored state machine");
    machine.advance_and_apply(0.0);
    let find = |type_key| {
        artboard
            .with_artboard(|artboard| {
                artboard
                    .objects()
                    .iter()
                    .flatten()
                    .find(|object| object.is_type_of(type_key))
                    .cloned()
            })
            .expect("authored child")
    };
    let cursor = find(TextInputCursor::TYPE_KEY);
    let focus = find(FocusData::TYPE_KEY);
    machine.advance_and_apply(0.6);
    assert!(!cursor_has_local_path(&cursor));
    machine.with_instance_mut(|machine| machine.set_focus(Some(focus)));
    assert!(cursor_has_local_path(&cursor));
    machine.advance_and_apply(0.5);
    assert!(!cursor_has_local_path(&cursor));
    machine.advance_and_apply(0.5);
    assert!(cursor_has_local_path(&cursor));
    machine.advance_and_apply(0.4);
    machine.with_instance_mut(|machine| machine.text_input("a"));
    machine.advance_and_apply(0.2);
    assert!(cursor_has_local_path(&cursor));
    machine.advance_and_apply(0.4);
    machine.with_instance_mut(|machine| {
        machine.key_input(Key::from_raw(263), KeyModifiers::from_raw(0), true, false)
    });
    machine.advance_and_apply(0.2);
    assert!(cursor_has_local_path(&cursor));
    machine.with_instance_mut(|machine| machine.clear_focus());
    assert!(!cursor_has_local_path(&cursor));
}

#[test]
fn caret_blink_accounts_for_every_elapsed_phase() {
    use nuxie_runtime::source::text::text_input_cursor::TextInputCursor;
    let (_file, artboard, _input) = input_fixture();
    let machine = artboard
        .state_machine_instance_handle(0)
        .expect("authored state machine");
    machine.advance_and_apply(0.0);
    let find = |type_key| {
        artboard
            .with_artboard(|artboard| {
                artboard
                    .objects()
                    .iter()
                    .flatten()
                    .find(|object| object.is_type_of(type_key))
                    .cloned()
            })
            .expect("authored child")
    };
    let cursor = find(TextInputCursor::TYPE_KEY);
    let focus = find(FocusData::TYPE_KEY);
    machine.with_instance_mut(|machine| machine.set_focus(Some(focus)));
    assert!(cursor_has_local_path(&cursor));
    for (elapsed, visible) in [
        (1.0, true),
        (1.5, false),
        (2.0, false),
        (0.3, false),
        (0.2, true),
    ] {
        machine.advance_and_apply(elapsed);
        assert_eq!(cursor_has_local_path(&cursor), visible, "after {elapsed}s");
    }
}

#[test]
fn caret_blink_marks_the_artboard_changed() {
    use nuxie_render_api::NullRenderer;
    use nuxie_runtime::source::text::text_input_cursor::TextInputCursor;
    let (_file, artboard, input) = input_fixture();
    let machine = artboard
        .state_machine_instance_handle(0)
        .expect("authored state machine");
    let mut renderer = NullRenderer::new();
    machine.advance_and_apply(0.0);
    let find = |type_key| {
        artboard
            .with_artboard(|artboard| {
                artboard
                    .objects()
                    .iter()
                    .flatten()
                    .find(|object| object.is_type_of(type_key))
                    .cloned()
            })
            .expect("authored child")
    };
    let cursor = find(TextInputCursor::TYPE_KEY);
    let focus = find(FocusData::TYPE_KEY);
    machine.with_instance_mut(|machine| machine.set_focus(Some(focus)));
    machine.advance_and_apply(0.0);
    artboard.draw(&mut renderer);
    assert!(!artboard.with_artboard(|artboard| artboard.did_change()));

    // Mid-phase nothing moved, so there is nothing to redraw.
    machine.advance_and_apply(0.2);
    assert!(!artboard.with_artboard(|artboard| artboard.did_change()));

    // Hiding and showing the caret both count as a change.
    machine.advance_and_apply(0.3);
    assert!(!cursor_has_local_path(&cursor));
    assert!(artboard.with_artboard(|artboard| artboard.did_change()));
    artboard.draw(&mut renderer);
    machine.advance_and_apply(0.5);
    assert!(cursor_has_local_path(&cursor));
    assert!(artboard.with_artboard(|artboard| artboard.did_change()));
    artboard.draw(&mut renderer);

    // An even number of phases leaves the caret where it was.
    machine.advance_and_apply(1.0);
    assert!(cursor_has_local_path(&cursor));
    assert!(!artboard.with_artboard(|artboard| artboard.did_change()));

    // A selection hides the caret, so its toggles change nothing on screen.
    with_input(&input, |input| {
        input.raw_text_input().set_text("hello world".into());
        input.raw_text_input().select_all();
    });
    machine.advance_and_apply(0.0);
    artboard.draw(&mut renderer);
    assert!(!artboard.with_artboard(|artboard| artboard.did_change()));
    machine.advance_and_apply(0.5);
    assert!(!artboard.with_artboard(|artboard| artboard.did_change()));
}

#[test]
fn obscured_native_input_preserves_value_and_blocks_selection_export() {
    let (_file, artboard, input) = input_fixture();
    assert!(CoreRegistry::set_string_handle(
        &input,
        817,
        "hunter2".into()
    ));
    artboard.advance_default(0.0);
    with_input(&input, |input| input.raw_text_input().select_all());
    assert_eq!(
        with_input(&input, |input| input.selected_text()),
        Some("hunter2".into())
    );
    assert!(CoreRegistry::set_bool_handle(&input, 1095, true));
    assert_eq!(
        with_input(&input, |input| input.selected_text()),
        Some(String::new())
    );
    with_input(&input, |input| {
        assert_eq!(input.base.text(), "hunter2");
        assert!(input.raw_text_input().obscured());
    });
    assert!(CoreRegistry::set_bool_handle(&input, 1095, false));
    assert_eq!(
        with_input(&input, |input| input.selected_text()),
        Some("hunter2".into())
    );
}

// bec99be4: Option<String> is the Rust equivalent of handled + out string.
#[test]
fn obscured_text_input_keeps_selected_text_off_the_clipboard() {
    let (_file, artboard, input) = input_fixture();
    with_input(&input, |input| {
        input.raw_text_input().set_text("hunter2".into())
    });
    artboard.advance_default(0.0);
    with_input(&input, |input| input.raw_text_input().select_all());
    let mut selected =
        with_input(&input, |input| input.selected_text()).expect("handled selection");
    assert_eq!(selected, "hunter2");
    assert!(CoreRegistry::set_bool_handle(&input, 1095, true));
    selected = "stale".into();
    assert_eq!(selected, "stale");
    // A handled empty result replaces stale clipboard contents, not None.
    selected =
        with_input(&input, |input| input.selected_text()).expect("handled obscured selection");
    assert!(selected.is_empty());
}

#[test]
fn obscured_text_input_stops_selection_lookup_at_itself() {
    use nuxie_runtime::source::input::{
        focus_manager::FocusManager, focus_node::FocusNode, focusable::Focusable,
    };
    use std::{cell::RefCell, rc::Rc};
    struct SelectionAncestor;
    impl Focusable for SelectionAncestor {
        fn key_input(&mut self, _: Key, _: KeyModifiers, _: bool, _: bool) -> bool {
            false
        }
        fn text_input(&mut self, _: &str) -> bool {
            false
        }
        fn focused(&mut self) {}
        fn blurred(&mut self) {}
        fn selected_text(&self) -> Option<String> {
            Some("ancestor selection".into())
        }
    }
    let (_file, artboard, input) = input_fixture();
    // Upstream creates a fresh node over TextInput's Focusable base, not its
    // authored FocusData node (which adds eligibility and tree ownership).
    struct InputFocusable(CoreHandle);
    impl Focusable for InputFocusable {
        fn key_input(
            &mut self,
            key: Key,
            modifiers: KeyModifiers,
            pressed: bool,
            repeat: bool,
        ) -> bool {
            with_input(&self.0, |input| {
                input.key_input(key, modifiers, pressed, repeat)
            })
        }
        fn text_input(&mut self, text: &str) -> bool {
            with_input(&self.0, |input| input.text_input(text))
        }
        fn focused(&mut self) {
            with_input(&self.0, TextInput::focused);
        }
        fn blurred(&mut self) {
            with_input(&self.0, TextInput::blurred);
        }
        fn selected_text(&self) -> Option<String> {
            self.0
                .with_downcast::<TextInput, _>(TextInput::selected_text)
                .expect("live TextInput")
        }
        fn focusable_artboard(&self) -> Option<CoreHandle> {
            self.0
                .with_downcast::<TextInput, _>(TextInput::focusable_artboard)
                .expect("live TextInput")
        }
        fn accepts_keyboard_input(&self) -> bool {
            self.0
                .with_downcast::<TextInput, _>(TextInput::accepts_keyboard_input)
                .expect("live TextInput")
        }
    }
    let input_node = FocusNode::new(Some(Rc::new(RefCell::new(InputFocusable(input.clone())))));
    let ancestor_node = FocusNode::new(Some(Rc::new(RefCell::new(SelectionAncestor))));
    FocusNode::add_child(&ancestor_node, input_node.clone());
    let mut manager = FocusManager::new();
    manager.set_focus(input_node);
    with_input(&input, |input| {
        input.raw_text_input().set_text("hunter2".into())
    });
    artboard.advance_default(0.0);
    with_input(&input, |input| input.raw_text_input().select_all());
    assert_eq!(manager.selected_text(), "hunter2");
    with_input(&input, |input| input.raw_text_input().clear_selection());
    assert_eq!(manager.selected_text(), "ancestor selection");
    with_input(&input, |input| input.raw_text_input().select_all());
    assert!(CoreRegistry::set_bool_handle(&input, 1095, true));
    assert!(manager.selected_text().is_empty());
    manager.clear_focus();
}

fn input_cursor(handle: &CoreHandle) -> Option<(u32, u32)> {
    handle.with_downcast_mut::<TextInput, _>(|input| {
        let cursor = input.raw_text_input().cursor();
        (
            cursor.start().code_point_index(),
            cursor.end().code_point_index(),
        )
    })
}

fn property_key_for_name(type_name: &str, property_name: &str) -> u16 {
    let definition = nuxie_schema::definition_by_name(type_name).expect("schema type");
    std::iter::once(definition.name)
        .chain(definition.ancestors.iter().copied())
        .filter_map(nuxie_schema::definition_by_name)
        .flat_map(|owner| owner.properties)
        .find(|property| property.name == property_name)
        .expect("schema property")
        .key
        .int
}

#[test]
fn selected_text_reaches_host_through_focus_data_and_structural_focus_child() {
    use nuxie_runtime::source::input::focus_node::FocusNode;

    let (_file, artboard, text_input) = input_fixture();
    let machine = artboard
        .state_machine_instance_handle(0)
        .expect("authored state machine");
    machine.advance_and_apply(0.0);
    let focus_data = artboard
        .with_artboard(|artboard| {
            artboard
                .objects()
                .iter()
                .flatten()
                .find(|object| {
                    object
                        .with_downcast::<FocusData, _>(|data| data.parent_handle())
                        .flatten()
                        .as_ref()
                        == Some(&text_input)
                })
                .cloned()
        })
        .expect("TextInput's FocusData child");
    let manager = machine
        .with_instance(|machine| machine.focus_manager())
        .expect("artboard focus manager");
    machine.with_instance_mut(|machine| machine.set_focus(Some(focus_data)));
    with_input(&text_input, |input| {
        let raw = input.raw_text_input();
        raw.set_text("aé🦀z".to_owned());
        raw.set_cursor(Cursor::new(
            CursorPosition::new(0, 1),
            CursorPosition::new(0, 3),
        ));
    });
    assert_eq!(
        with_input(&text_input, |input| input.selected_text()),
        Some("é🦀".into())
    );
    assert_eq!(
        manager.with_focus_manager(|manager| manager.selected_text()),
        "é🦀"
    );

    // Upstream skips a node without a Focusable and bubbles to the same
    // FocusData/TextInput selection. Reading selection must not mutate text.
    manager.with_focus_manager_mut(|manager| {
        let parent = manager.primary_focus().expect("focused input");
        let child = FocusNode::new(None);
        manager.add_child(Some(parent), child.clone(), None);
        manager.set_focus(child);
    });
    assert_eq!(
        manager.with_focus_manager(|manager| manager.selected_text()),
        "é🦀"
    );
    assert_eq!(
        with_input(&text_input, |input| input.raw_text_input().text()),
        "aé🦀z"
    );
    machine.with_instance_mut(|machine| machine.clear_focus());
    assert_eq!(
        manager.with_focus_manager(|manager| manager.selected_text()),
        ""
    );
}

#[test]
fn upstream_707c_state_machine_key_and_text_input_forward_to_text_input() {
    let (_file, artboard, text_input) = input_fixture();
    let Some(machine) = artboard.state_machine_instance_handle(0) else {
        return;
    };
    machine.advance_and_apply(0.0);

    let Some(focus_data) = artboard.with_artboard(|artboard| {
        artboard
            .objects()
            .iter()
            .flatten()
            .find(|object| object.is_type_of(FocusData::TYPE_KEY))
            .cloned()
    }) else {
        panic!("authored FocusData");
    };
    machine.with_instance_mut(|machine| machine.set_focus(Some(focus_data)));

    with_input(&text_input, |input| {
        input.raw_text_input().set_text(String::new());
        input.raw_text_input().set_cursor(Cursor::zero());
    });

    assert!(machine.with_instance_mut(|machine| machine.text_input("typed text")));
    assert_eq!(
        with_input(&text_input, |input| input.raw_text_input().text()),
        "typed text"
    );

    assert!(machine.with_instance_mut(|machine| {
        machine.key_input(Key::BACKSPACE, KeyModifiers::NONE, true, false)
    }));
    assert_eq!(
        with_input(&text_input, |input| input.raw_text_input().text()),
        "typed tex"
    );

    machine.with_instance_mut(|machine| machine.clear_focus());
    assert!(!machine.with_instance_mut(|machine| machine.text_input("more")));
    assert!(!machine.with_instance_mut(|machine| {
        machine.key_input(Key::BACKSPACE, KeyModifiers::NONE, true, false)
    }));
    assert_eq!(
        with_input(&text_input, |input| input.raw_text_input().text()),
        "typed tex"
    );
}

#[test]
fn upstream_text_input_load_and_drawable_children_are_ported() {
    let (_file, artboard, text_input) = input_fixture();
    assert_eq!(
        artboard.with_artboard(|artboard| artboard
            .objects()
            .iter()
            .flatten()
            .filter(|object| object.is_type_of(TextInput::TYPE_KEY))
            .count()),
        1
    );
    with_input(&text_input, |input| {
        let child_count = |name: &str| {
            let key = nuxie_schema::definition_by_name(name)
                .expect("child type")
                .type_key
                .int;
            input
                .base
                .children()
                .iter()
                .filter(|child| child.is_type_of(key))
                .count()
        };
        assert_eq!(child_count("TextInputText"), 1);
        assert_eq!(child_count("TextInputSelection"), 1);
        assert_eq!(child_count("TextInputCursor"), 1);
        assert_eq!(child_count("TextInputSelectedText"), 0);
        assert_eq!(child_count("TextInputDrawable"), 3);
    });
    artboard.advance_default(0.0);
    let parent = with_input(&text_input, |input| input.base.parent_handle())
        .expect("authored layout parent");
    assert!(
        parent
            .with(|object| object
                .as_layout_component()
                .is_some_and(|layout| layout.layout_node_key(0).is_some()))
            .unwrap()
    );
}

#[test]
fn wave_c7_text_input_004_key_input_handles_backspace_and_delete() {
    const BACKSPACE: u32 = 259;
    const DELETE: u32 = 261;

    let (_file, artboard, text_input) = input_fixture();
    let _ = with_input(&text_input, |input| {
        input.raw_text_input().set_text("hello".to_owned())
    });
    let _ = with_input(&text_input, |input| {
        input.raw_text_input().set_cursor(Cursor::new(
            CursorPosition::unresolved(3),
            CursorPosition::unresolved(3),
        ))
    });

    artboard.advance_default(0.0);

    let handled = with_input(&text_input, |input| {
        input.key_input(
            Key::from_raw(BACKSPACE),
            KeyModifiers::from_raw(0),
            true,
            false,
        )
    });
    assert!(handled);
    assert_eq!(
        text_input
            .with_downcast_mut::<TextInput, _>(|input| input.raw_text_input().text())
            .as_deref(),
        Some("helo")
    );

    let _ = with_input(&text_input, |input| {
        input.raw_text_input().set_cursor(Cursor::new(
            CursorPosition::unresolved(2),
            CursorPosition::unresolved(2),
        ))
    });
    let handled = with_input(&text_input, |input| {
        input.key_input(
            Key::from_raw(DELETE),
            KeyModifiers::from_raw(0),
            true,
            false,
        )
    });
    assert!(handled);
    assert_eq!(
        text_input
            .with_downcast_mut::<TextInput, _>(|input| input.raw_text_input().text())
            .as_deref(),
        Some("heo")
    );
}

#[test]
fn wave_c7_text_input_006_key_input_returns_false_for_unhandled_keys() {
    const ESCAPE: u32 = 256;
    const RIGHT: u32 = 262;

    let (_file, artboard, text_input) = input_fixture();

    artboard.advance_default(0.0);

    let handled = with_input(&text_input, |input| {
        input.key_input(
            Key::from_raw(ESCAPE),
            KeyModifiers::from_raw(0),
            true,
            false,
        )
    });
    assert!(!handled);

    let handled = with_input(&text_input, |input| {
        input.key_input(
            Key::from_raw(RIGHT),
            KeyModifiers::from_raw(0),
            false,
            false,
        )
    });
    assert!(!handled);
}

#[test]
fn wave_c7_text_input_009_key_input_handles_select_all() {
    const A: u32 = 65;
    const CTRL: u32 = 2;
    const META: u32 = 8;

    let (_file, artboard, text_input) = input_fixture();
    let _ = with_input(&text_input, |input| {
        input.raw_text_input().set_text("hello world".to_owned())
    });
    let _ = with_input(&text_input, |input| {
        input
            .raw_text_input()
            .set_cursor(Cursor::new(CursorPosition::zero(), CursorPosition::zero()))
    });

    artboard.advance_default(0.0);

    let system_modifier = if cfg!(windows) { CTRL } else { META };
    let handled = with_input(&text_input, |input| {
        input.key_input(
            Key::from_raw(A),
            KeyModifiers::from_raw(system_modifier),
            true,
            false,
        )
    });
    assert!(handled);
    assert_eq!(input_cursor(&text_input), Some((0, 11)));

    let handled = with_input(&text_input, |input| {
        input.key_input(Key::from_raw(A), KeyModifiers::from_raw(0), true, false)
    });
    assert!(!handled);
}

#[test]
fn wave_c7_text_input_014_text_input_method_inserts_text() {
    let (_file, artboard, text_input) = input_fixture();
    let _ = with_input(&text_input, |input| {
        input.raw_text_input().set_text("".to_owned())
    });
    let _ = with_input(&text_input, |input| {
        input
            .raw_text_input()
            .set_cursor(Cursor::new(CursorPosition::zero(), CursorPosition::zero()))
    });

    artboard.advance_default(0.0);

    let handled = with_input(&text_input, |input| input.text_input("hello"));
    assert!(handled);
    assert_eq!(
        text_input
            .with_downcast_mut::<TextInput, _>(|input| input.raw_text_input().text())
            .as_deref(),
        Some("hello")
    );

    let handled = with_input(&text_input, |input| input.text_input(" world"));
    assert!(handled);
    assert_eq!(
        text_input
            .with_downcast_mut::<TextInput, _>(|input| input.raw_text_input().text())
            .as_deref(),
        Some("hello world")
    );
}

#[test]
fn wave_c7_text_input_018_selection_radius_changed_updates_raw_text_input() {
    let (_file, artboard, text_input) = input_fixture();
    let radius = property_key_for_name("TextInput", "selectionRadius");

    let _ = CoreRegistry::set_double_handle(&text_input, i32::from(radius), 5.0);

    assert_eq!(
        CoreRegistry::get_double_handle(&text_input, i32::from(radius)),
        Some(5.0)
    );
}

#[test]
fn upstream_text_input_key_editing_and_selection_cases_are_ported() {
    const A: u32 = 65;
    const Z: u32 = 90;
    const ESCAPE: u32 = 256;
    const BACKSPACE: u32 = 259;
    const DELETE: u32 = 261;
    const RIGHT: u32 = 262;
    const LEFT: u32 = 263;
    const HOME: u32 = 268;
    const END: u32 = 269;
    const SHIFT: u32 = 1;
    const CTRL: u32 = 2;
    const ALT: u32 = 4;
    const META: u32 = 8;

    let (_file, artboard, text_input) = input_fixture();
    let text_key = property_key_for_name("TextInput", "text");
    assert!(CoreRegistry::set_string_handle(
        &text_input,
        i32::from(text_key),
        String::from_utf8(b"hello world".to_vec()).unwrap()
    ));
    with_input(&text_input, |input| {
        input
            .raw_text_input()
            .set_cursor(Cursor::new(CursorPosition::zero(), CursorPosition::zero()))
    });
    assert!(with_input(&text_input, |input| input.key_input(
        Key::from_raw(RIGHT),
        KeyModifiers::from_raw(0),
        true,
        false
    )));
    assert_eq!(input_cursor(&text_input), Some((1, 1)));
    assert!(with_input(&text_input, |input| input.key_input(
        Key::from_raw(LEFT),
        KeyModifiers::from_raw(0),
        true,
        false
    )));
    assert_eq!(input_cursor(&text_input), Some((0, 0)));

    with_input(&text_input, |input| {
        input.raw_text_input().set_cursor(Cursor::new(
            CursorPosition::unresolved(3),
            CursorPosition::unresolved(3),
        ))
    });
    assert!(with_input(&text_input, |input| input.key_input(
        Key::from_raw(BACKSPACE),
        KeyModifiers::from_raw(0),
        true,
        false
    )));
    assert_eq!(
        text_input
            .with_downcast_mut::<TextInput, _>(|input| input.raw_text_input().text())
            .as_deref(),
        Some("helo world")
    );
    with_input(&text_input, |input| {
        input.raw_text_input().set_cursor(Cursor::new(
            CursorPosition::unresolved(2),
            CursorPosition::unresolved(2),
        ))
    });
    assert!(with_input(&text_input, |input| input.key_input(
        Key::from_raw(DELETE),
        KeyModifiers::from_raw(0),
        true,
        false
    )));
    assert_eq!(
        text_input
            .with_downcast_mut::<TextInput, _>(|input| input.raw_text_input().text())
            .as_deref(),
        Some("heo world")
    );

    assert!(CoreRegistry::set_string_handle(
        &text_input,
        i32::from(text_key),
        String::from_utf8(Vec::new()).unwrap()
    ));
    with_input(&text_input, |input| {
        input
            .raw_text_input()
            .set_cursor(Cursor::new(CursorPosition::zero(), CursorPosition::zero()))
    });
    assert!(with_input(&text_input, |input| input.text_input("hello")));
    assert!(with_input(&text_input, |input| input.key_input(
        Key::from_raw(Z),
        KeyModifiers::from_raw(META),
        true,
        false
    )));
    assert_eq!(
        text_input
            .with_downcast_mut::<TextInput, _>(|input| input.raw_text_input().text())
            .as_deref(),
        Some("")
    );
    assert!(with_input(&text_input, |input| input.key_input(
        Key::from_raw(Z),
        KeyModifiers::from_raw(META | SHIFT),
        true,
        false
    )));
    assert_eq!(
        text_input
            .with_downcast_mut::<TextInput, _>(|input| input.raw_text_input().text())
            .as_deref(),
        Some("hello")
    );
    assert!(!with_input(&text_input, |input| input.key_input(
        Key::from_raw(ESCAPE),
        KeyModifiers::from_raw(0),
        true,
        false
    )));
    assert!(!with_input(&text_input, |input| input.key_input(
        Key::from_raw(RIGHT),
        KeyModifiers::from_raw(0),
        false,
        false
    )));

    assert!(CoreRegistry::set_string_handle(
        &text_input,
        i32::from(text_key),
        String::from_utf8(b"one two three".to_vec()).unwrap()
    ));
    with_input(&text_input, |input| {
        input
            .raw_text_input()
            .set_cursor(Cursor::new(CursorPosition::zero(), CursorPosition::zero()))
    });
    assert!(with_input(&text_input, |input| input.key_input(
        Key::from_raw(RIGHT),
        KeyModifiers::from_raw(ALT),
        true,
        false
    )));
    assert_eq!(input_cursor(&text_input), Some((3, 3)));
    with_input(&text_input, |input| {
        input
            .raw_text_input()
            .set_cursor(Cursor::new(CursorPosition::zero(), CursorPosition::zero()))
    });
    assert!(with_input(&text_input, |input| input.key_input(
        Key::from_raw(RIGHT),
        KeyModifiers::from_raw(META),
        true,
        false
    )));
    assert_eq!(input_cursor(&text_input), Some((13, 13)));
    assert!(CoreRegistry::set_string_handle(
        &text_input,
        i32::from(text_key),
        String::from_utf8(b"oneTwo threeF".to_vec()).unwrap()
    ));
    with_input(&text_input, |input| {
        input
            .raw_text_input()
            .set_cursor(Cursor::new(CursorPosition::zero(), CursorPosition::zero()))
    });
    assert!(with_input(&text_input, |input| input.key_input(
        Key::from_raw(RIGHT),
        KeyModifiers::from_raw(ALT | CTRL),
        true,
        false
    )));
    assert_eq!(input_cursor(&text_input), Some((3, 3)));

    assert!(CoreRegistry::set_string_handle(
        &text_input,
        i32::from(text_key),
        String::from_utf8(b"hello world".to_vec()).unwrap()
    ));
    with_input(&text_input, |input| {
        input
            .raw_text_input()
            .set_cursor(Cursor::new(CursorPosition::zero(), CursorPosition::zero()))
    });
    assert!(with_input(&text_input, |input| input.key_input(
        Key::from_raw(RIGHT),
        KeyModifiers::from_raw(SHIFT),
        true,
        false
    )));
    assert!(with_input(&text_input, |input| input.key_input(
        Key::from_raw(RIGHT),
        KeyModifiers::from_raw(SHIFT),
        true,
        false
    )));
    assert_eq!(input_cursor(&text_input), Some((0, 2)));
    with_input(&text_input, |input| {
        input
            .raw_text_input()
            .set_cursor(Cursor::new(CursorPosition::zero(), CursorPosition::zero()))
    });
    assert!(with_input(&text_input, |input| input.key_input(
        Key::from_raw(A),
        KeyModifiers::from_raw(META),
        true,
        false
    )));
    assert_eq!(input_cursor(&text_input), Some((0, 11)));
    assert!(!with_input(&text_input, |input| input.key_input(
        Key::from_raw(A),
        KeyModifiers::from_raw(0),
        true,
        false
    )));
    with_input(&text_input, |input| {
        input.raw_text_input().set_cursor(Cursor::new(
            CursorPosition::unresolved(5),
            CursorPosition::unresolved(5),
        ))
    });
    // Match the pinned home/end fixture's update after the raw cursor write.
    with_input(&text_input, |input| {
        input
            .raw_text_input()
            .update(&_file.with_file(|file| file.factory()));
    });
    assert!(with_input(&text_input, |input| input.key_input(
        Key::from_raw(END),
        KeyModifiers::from_raw(0),
        true,
        false
    )));
    assert_eq!(input_cursor(&text_input), Some((11, 11)));
    assert!(with_input(&text_input, |input| input.key_input(
        Key::from_raw(HOME),
        KeyModifiers::from_raw(0),
        true,
        false
    )));
    assert_eq!(input_cursor(&text_input), Some((0, 0)));
    assert!(with_input(&text_input, |input| input.key_input(
        Key::from_raw(END),
        KeyModifiers::from_raw(SHIFT),
        true,
        false
    )));
    assert_eq!(input_cursor(&text_input), Some((0, 11)));
}

#[test]
fn upstream_text_input_text_multiline_wrapper_and_radius_cases_are_ported() {
    const ENTER: u32 = 257;
    let (_file, artboard, text_input) = input_fixture();
    let text_key = property_key_for_name("TextInput", "text");
    let multiline_key = property_key_for_name("TextInput", "multiline");
    let radius_key = property_key_for_name("TextInput", "selectionRadius");

    assert!(CoreRegistry::set_string_handle(
        &text_input,
        i32::from(text_key),
        String::from_utf8(b"line1\nline2".to_vec()).unwrap()
    ));
    assert_eq!(
        text_input
            .with_downcast_mut::<TextInput, _>(|input| input.raw_text_input().text())
            .as_deref(),
        Some("line1\nline2")
    );
    assert!(CoreRegistry::set_bool_handle(
        &text_input,
        i32::from(multiline_key),
        false
    ));
    assert_eq!(
        text_input
            .with_downcast_mut::<TextInput, _>(|input| input.raw_text_input().text())
            .as_deref(),
        Some("line1 line2")
    );
    assert!(CoreRegistry::set_bool_handle(
        &text_input,
        i32::from(multiline_key),
        true
    ));
    assert_eq!(
        text_input
            .with_downcast_mut::<TextInput, _>(|input| input.raw_text_input().text())
            .as_deref(),
        Some("line1\nline2")
    );

    assert!(CoreRegistry::set_string_handle(
        &text_input,
        i32::from(text_key),
        String::from_utf8(Vec::new()).unwrap()
    ));
    assert!(CoreRegistry::set_bool_handle(
        &text_input,
        i32::from(multiline_key),
        false
    ));
    assert!(with_input(&text_input, |input| input.text_input("a\nb\r\nc\rd")));
    assert_eq!(
        text_input
            .with_downcast_mut::<TextInput, _>(|input| input.raw_text_input().text())
            .as_deref(),
        Some("a b c d")
    );
    assert!(CoreRegistry::set_bool_handle(
        &text_input,
        i32::from(multiline_key),
        true
    ));
    assert_eq!(
        text_input
            .with_downcast_mut::<TextInput, _>(|input| input.raw_text_input().text())
            .as_deref(),
        Some("a b c d")
    );

    assert!(CoreRegistry::set_string_handle(
        &text_input,
        i32::from(text_key),
        String::from_utf8(b"hello".to_vec()).unwrap()
    ));
    with_input(&text_input, |input| {
        input.raw_text_input().set_cursor(Cursor::new(
            CursorPosition::unresolved(3),
            CursorPosition::unresolved(3),
        ))
    });
    assert!(with_input(&text_input, |input| input.key_input(
        Key::from_raw(ENTER),
        KeyModifiers::from_raw(0),
        true,
        false
    )));
    assert_eq!(
        text_input
            .with_downcast_mut::<TextInput, _>(|input| input.raw_text_input().text())
            .as_deref(),
        Some("hel\nlo")
    );
    assert!(CoreRegistry::set_string_handle(
        &text_input,
        i32::from(text_key),
        String::from_utf8(b"hello".to_vec()).unwrap()
    ));
    assert!(CoreRegistry::set_bool_handle(
        &text_input,
        i32::from(multiline_key),
        false
    ));
    with_input(&text_input, |input| {
        input.raw_text_input().set_cursor(Cursor::new(
            CursorPosition::unresolved(3),
            CursorPosition::unresolved(3),
        ))
    });
    assert!(!with_input(&text_input, |input| input.key_input(
        Key::from_raw(ENTER),
        KeyModifiers::from_raw(0),
        true,
        false
    )));
    assert_eq!(
        text_input
            .with_downcast_mut::<TextInput, _>(|input| input.raw_text_input().text())
            .as_deref(),
        Some("hello")
    );

    assert!(CoreRegistry::set_bool_handle(
        &text_input,
        i32::from(multiline_key),
        true
    ));
    with_input(&text_input, |input| {
        input.raw_text_input().set_cursor(Cursor::new(
            CursorPosition::unresolved(2),
            CursorPosition::unresolved(2),
        ))
    });
    with_input(&text_input, |input| input.select_word());
    assert_eq!(input_cursor(&text_input), Some((0, 5)));
    with_input(&text_input, |input| {
        input.raw_text_input().set_cursor(Cursor::new(
            CursorPosition::unresolved(3),
            CursorPosition::unresolved(3),
        ))
    });
    with_input(&text_input, |input| input.select_line());
    assert_eq!(input_cursor(&text_input), Some((0, 5)));

    assert!(CoreRegistry::set_double_handle(
        &text_input,
        i32::from(radius_key),
        7.0
    ));
    assert_eq!(
        text_input.with_downcast_mut::<TextInput, _>(|input| input
            .raw_text_input()
            .selection_corner_radius()),
        Some(7.0)
    );
}

#[test]
fn upstream_text_input_vertical_cursor_retains_the_ideal_column() {
    const DOWN: u32 = 264;
    let (_file, artboard, text_input) = input_fixture();
    let text_key = property_key_for_name("TextInput", "text");
    assert!(CoreRegistry::set_string_handle(
        &text_input,
        i32::from(text_key),
        String::from_utf8(b"abcdefghij\nx\nabcdefghij".to_vec()).unwrap()
    ));
    with_input(&text_input, |input| {
        input.raw_text_input().set_cursor(Cursor::new(
            CursorPosition::unresolved(8),
            CursorPosition::unresolved(8),
        ))
    });
    // Raw cursor writes need the pinned update step to resolve their line and caret.
    with_input(&text_input, |input| {
        input
            .raw_text_input()
            .update(&_file.with_file(|file| file.factory()));
    });
    assert!(with_input(&text_input, |input| input.key_input(
        Key::from_raw(DOWN),
        KeyModifiers::from_raw(0),
        true,
        false
    )));
    assert_eq!(input_cursor(&text_input), Some((12, 12)));
    artboard.advance_default(0.0);
    assert!(with_input(&text_input, |input| input.key_input(
        Key::from_raw(DOWN),
        KeyModifiers::from_raw(0),
        true,
        false
    )));
    assert_eq!(input_cursor(&text_input), Some((21, 21)));
}

#[test]
fn upstream_text_input_multiline_cursor_sequence_is_ported() {
    const LEFT: u32 = 263;
    const RIGHT: u32 = 262;
    const UP: u32 = 265;
    const DOWN: u32 = 264;
    let (_file, artboard, text_input) = input_fixture();
    assert!(CoreRegistry::set_string_handle(
        &text_input,
        i32::from(property_key_for_name("TextInput", "text")),
        String::from_utf8(b"this is some\nmultiline text input\nwith one final line".to_vec())
            .unwrap()
    ));
    artboard.advance_default(0.0);
    with_input(&text_input, |input| {
        input.key_input(Key::from_raw(RIGHT), KeyModifiers::from_raw(0), true, false)
    });
    artboard.advance_default(0.0);
    assert_eq!(input_cursor(&text_input), Some((1, 1)));
    for _ in 0..14 {
        with_input(&text_input, |input| {
            input.key_input(Key::from_raw(RIGHT), KeyModifiers::from_raw(0), true, false)
        });
        artboard.advance_default(0.0);
    }
    assert_eq!(input_cursor(&text_input), Some((15, 15)));
    with_input(&text_input, |input| {
        input.key_input(Key::from_raw(UP), KeyModifiers::from_raw(0), true, false)
    });
    artboard.advance_default(0.0);
    assert_eq!(input_cursor(&text_input), Some((4, 4)));
    with_input(&text_input, |input| {
        input.key_input(Key::from_raw(UP), KeyModifiers::from_raw(0), true, false)
    });
    artboard.advance_default(0.0);
    assert_eq!(input_cursor(&text_input), Some((0, 0)));
    for _ in 0..3 {
        with_input(&text_input, |input| {
            input.key_input(Key::from_raw(RIGHT), KeyModifiers::from_raw(0), true, false)
        });
        artboard.advance_default(0.0);
    }
    with_input(&text_input, |input| {
        input.key_input(Key::from_raw(DOWN), KeyModifiers::from_raw(0), true, false)
    });
    artboard.advance_default(0.0);
    assert_eq!(input_cursor(&text_input), Some((14, 14)));
    with_input(&text_input, |input| {
        input.key_input(Key::from_raw(DOWN), KeyModifiers::from_raw(0), true, false)
    });
    artboard.advance_default(0.0);
    assert_eq!(input_cursor(&text_input), Some((36, 36)));
    with_input(&text_input, |input| {
        input.key_input(Key::from_raw(DOWN), KeyModifiers::from_raw(0), true, false)
    });
    artboard.advance_default(0.0);
    assert_eq!(input_cursor(&text_input), Some((53, 53)));
    with_input(&text_input, |input| {
        input.key_input(Key::from_raw(LEFT), KeyModifiers::from_raw(0), true, false)
    });
    artboard.advance_default(0.0);
    assert_eq!(input_cursor(&text_input), Some((52, 52)));
}
