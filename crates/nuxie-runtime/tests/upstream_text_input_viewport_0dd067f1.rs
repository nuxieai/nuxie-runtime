//! 0dd067f1: a press anywhere in the viewport reaches the text input.
//! Fixture source: tests/unit_tests/assets/rml/text_input_viewport.rml.

use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::{
    animation::state_machine_instance::RuntimeStateMachineInstanceHandle,
    nested_artboard::NestedArtboard, shapes::paint::shape_paint::ShapePaintPathKind,
    text::text_input_cursor::TextInputCursor,
};
use nuxie_runtime::source::{
    constraints::scrolling::scroll_constraint::ScrollConstraint,
    core::{CoreHandle, CoreType},
    generated::{core_registry::CoreRegistry, text::text_input_base::TextInputBase},
    layout_component::LayoutComponent,
    math::{mat2d::Mat2D, vec2d::Vec2D},
    pointer_button::PointerButton,
    text::{
        cursor::{Cursor, CursorPosition},
        text_input::TextInput,
    },
};
use nuxie_runtime::{File, RuntimeFactoryHandle};
use nuxie_runtime::{RuntimeArtboardInstanceHandle, RuntimeFileHandle};

fn viewport_file() -> RuntimeFileHandle {
    let root =
        std::path::PathBuf::from(std::env::var_os("RIVE_RUNTIME_DIR").expect("pinned upstream"));
    let bytes =
        std::fs::read(root.join("tests/unit_tests/assets/text_input_viewport.riv")).unwrap();
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    File::import(
        &bytes,
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .unwrap()
}

fn first_object(artboard: &RuntimeArtboardInstanceHandle, type_key: u16) -> CoreHandle {
    artboard
        .with_artboard(|a| {
            a.objects()
                .iter()
                .flatten()
                .find(|o| o.is_type_of(type_key))
                .cloned()
        })
        .expect("fixture object")
}

fn field_point(input: &CoreHandle, x_offset: f32) -> Vec2D {
    let mut viewport = input.clone();
    for _ in 0..3 {
        viewport = viewport
            .with(|v| v.component_parent_handle())
            .flatten()
            .expect("viewport ancestor");
    }
    viewport
        .with_downcast::<LayoutComponent, _>(|v| {
            v.world_transform() * Vec2D::new(v.layout_width() + x_offset, v.layout_height() * 0.5)
        })
        .expect("LayoutComponent viewport")
}

fn press(machine: &RuntimeStateMachineInstanceHandle, point: Vec2D, button: PointerButton) {
    machine.with_instance_mut(|m| {
        m.pointer_down(point, 0, button);
        m.pointer_up(point, 0, button);
    });
    machine.advance_and_apply(0.0);
}

#[test]
fn a_press_outside_a_focused_text_input_blurs_it() {
    let file = viewport_file();
    let a = file.with_file(|f| f.artboard_named("SingleLine")).unwrap();
    let m = a.state_machine_at(0).unwrap();
    let input = first_object(&a, TextInput::TYPE_KEY);
    let cursor = first_object(&a, TextInputCursor::TYPE_KEY);
    m.advance_and_apply(0.0);
    let in_field = field_point(&input, -2.0);
    let past_field = field_point(&input, 20.0);
    assert!(a.with_artboard(|a| a.bounds().contains(past_field)));
    press(&m, in_field, PointerButton::Primary);
    assert!(with_input(&input, |i| i.is_focused()));
    assert!(
        cursor
            .with_downcast::<TextInputCursor, _>(
                |c| c.with_path_mut(ShapePaintPathKind::LocalClockwise, &mut |_| {})
            )
            .unwrap()
    );
    press(&m, past_field, PointerButton::Primary);
    assert!(!with_input(&input, |i| i.is_focused()));
    assert!(
        !cursor
            .with_downcast::<TextInputCursor, _>(
                |c| c.with_path_mut(ShapePaintPathKind::LocalClockwise, &mut |_| {})
            )
            .unwrap()
    );
    press(&m, in_field, PointerButton::Primary);
    assert!(with_input(&input, |i| i.is_focused()));
    press(&m, in_field, PointerButton::Primary);
    assert!(with_input(&input, |i| i.is_focused()));
    press(&m, past_field, PointerButton::Secondary);
    assert!(with_input(&input, |i| i.is_focused()));
    press(&m, Vec2D::new(-10.0, -10.0), PointerButton::Primary);
    assert!(!with_input(&input, |i| i.is_focused()));
}

#[test]
fn a_press_in_the_host_blurs_a_nested_text_input() {
    let file = viewport_file();
    let a = file.with_file(|f| f.artboard_named("Host")).unwrap();
    let m = a.state_machine_at(0).unwrap();
    m.advance_and_apply(0.0);
    let hosts = a.with_artboard(|a| a.nested_artboards());
    assert_eq!(hosts.len(), 2);
    let nested_input = |host: &CoreHandle| {
        let instance = host
            .with_downcast::<NestedArtboard, _>(|h| h.artboard_instance_handle(0))
            .flatten()
            .unwrap();
        first_object(&instance, TextInput::TYPE_KEY)
    };
    let first = nested_input(&hosts[0]);
    let second = nested_input(&hosts[1]);
    let host_point = |host: &CoreHandle, input: &CoreHandle| {
        let local = field_point(input, -2.0);
        host.with_downcast::<NestedArtboard, _>(|h| h.base.base.world_transform() * local)
            .unwrap()
    };
    let first_field = host_point(&hosts[0], &first);
    let second_field = host_point(&hosts[1], &second);
    let host_empty = Vec2D::new(700.0, 50.0);
    assert!(a.with_artboard(|a| a.bounds().contains(host_empty)));
    press(&m, first_field, PointerButton::Primary);
    assert!(with_input(&first, |i| i.is_focused()));
    press(&m, host_empty, PointerButton::Primary);
    assert!(!with_input(&first, |i| i.is_focused()));
    press(&m, first_field, PointerButton::Primary);
    assert!(with_input(&first, |i| i.is_focused()));
    press(&m, second_field, PointerButton::Primary);
    assert!(!with_input(&first, |i| i.is_focused()));
    assert!(with_input(&second, |i| i.is_focused()));
}

fn with_input<R>(handle: &CoreHandle, use_input: impl FnOnce(&mut TextInput) -> R) -> R {
    handle.with_downcast_mut(use_input).expect("live TextInput")
}

#[test]
fn a_press_anywhere_in_the_viewport_reaches_the_text_input() {
    let root = std::path::PathBuf::from(
        std::env::var_os("RIVE_RUNTIME_DIR").expect("RIVE_RUNTIME_DIR points to pinned upstream"),
    );
    let path = root.join("tests/unit_tests/assets/text_input_viewport.riv");
    let bytes = std::fs::read(&path)
        .unwrap_or_else(|error| panic!("read pinned fixture {}: {error}", path.display()));
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let factory = RuntimeFactoryHandle::from_factory(&mut factory).expect("retained factory");
    let file = File::import(&bytes, factory, None, None, None).expect("fixture imports");

    for name in ["SingleLine", "Multiline"] {
        let artboard = file
            .with_file(|file| file.artboard_named(name))
            .expect("named artboard");
        let machine = artboard
            .state_machine_instance_handle(0)
            .expect("state machine");
        let input = artboard
            .with_artboard(|artboard| {
                artboard
                    .objects()
                    .iter()
                    .flatten()
                    .find(|object| object.is_type_of(TextInput::TYPE_KEY))
                    .cloned()
            })
            .expect("TextInput");

        // TextInput -> Text Container -> Scroll Content (+Scroll) -> Viewport.
        let container = with_input(&input, |input| input.parent_handle()).expect("container");
        assert!(container.is_type_of(LayoutComponent::TYPE_KEY), "{name}");
        let content = container
            .with(|container| container.component_parent_handle())
            .flatten()
            .expect("content");
        assert!(content.is_type_of(LayoutComponent::TYPE_KEY), "{name}");
        assert!(
            content
                .with_downcast::<LayoutComponent, _>(|content| {
                    content
                        .constraints()
                        .first()
                        .expect("scroll constraint")
                        .is_type_of(ScrollConstraint::TYPE_KEY)
                })
                .unwrap(),
            "{name}"
        );
        let viewport = content
            .with(|content| content.component_parent_handle())
            .flatten()
            .expect("viewport");
        assert!(viewport.is_type_of(LayoutComponent::TYPE_KEY), "{name}");

        assert!(CoreRegistry::set_string_handle(
            &input,
            i32::from(TextInputBase::TEXT_PROPERTY_KEY),
            "HI".into(),
        ));
        machine.advance_and_apply(0.0);
        let far_corner = viewport
            .with_downcast::<LayoutComponent, _>(|viewport| {
                assert!(
                    (viewport.layout_width() - 300.0).abs() <= 100.0 * f32::EPSILON * 300.0,
                    "{name}"
                );
                viewport.world_transform()
                    * Vec2D::new(
                        viewport.layout_width() - 2.0,
                        viewport.layout_height() - 2.0,
                    )
            })
            .unwrap();
        container
            .with_downcast::<LayoutComponent, _>(|container| {
                let mut inverse = Mat2D::default();
                assert!(container.world_transform().invert(&mut inverse), "{name}");
                assert!(
                    !container.local_bounds().contains(inverse * far_corner),
                    "{name}"
                );
            })
            .unwrap();

        with_input(&input, |input| {
            input
                .raw_text_input()
                .set_cursor(Cursor::collapsed(CursorPosition::unresolved(0)));
        });
        machine.with_instance_mut(|machine| {
            machine.pointer_down(far_corner, 0, PointerButton::Primary);
            machine.pointer_up(far_corner, 0, PointerButton::Primary);
        });
        machine.advance_and_apply(0.0);
        with_input(&input, |input| {
            let caret = input.raw_text_input().cursor();
            assert_eq!(caret.start().code_point_index(), 2, "{name}");
            assert_eq!(caret.end().code_point_index(), 2, "{name}");
            assert!(input.is_focused(), "{name}");
        });

        // Just outside the viewport the press misses the input.
        with_input(&input, |input| {
            input
                .raw_text_input()
                .set_cursor(Cursor::collapsed(CursorPosition::unresolved(0)));
        });
        let outside = viewport
            .with_downcast::<LayoutComponent, _>(|viewport| {
                viewport.world_transform()
                    * Vec2D::new(
                        viewport.layout_width() + 5.0,
                        viewport.layout_height() * 0.5,
                    )
            })
            .unwrap();
        machine.with_instance_mut(|machine| {
            machine.pointer_down(outside, 0, PointerButton::Primary);
            machine.pointer_up(outside, 0, PointerButton::Primary);
        });
        machine.advance_and_apply(0.0);
        assert_eq!(
            with_input(&input, |input| {
                input.raw_text_input().cursor().end().code_point_index()
            }),
            0,
            "{name}"
        );
    }
}
