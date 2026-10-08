//! 0dd067f1: a press anywhere in the viewport reaches the text input.
//! Fixture source: tests/unit_tests/assets/rml/text_input_viewport.rml.

use nuxie_render_api::{PersistentFactory, RecordingFactory};
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
                *viewport.world_transform()
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
                *viewport.world_transform()
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
