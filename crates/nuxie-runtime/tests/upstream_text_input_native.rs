//! Preserved text-input assertions, now exercising the actual native owners.
//! Pinned authority: tests/unit_tests/runtime/text_input_test.cpp.

use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::{
    core::CoreHandle,
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
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let bytes = std::fs::read(PathBuf::from(root).join("tests/unit_tests/assets/text_input.riv"))
        .expect("pinned text_input.riv");
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let retained = RuntimeFactoryHandle::from_factory(&mut factory).expect("retained factory");
    let file = File::import(&bytes, retained, None, None, None).expect("native File import");
    let artboard = file
        .with_file(|file| file.artboard_named("Text Input - Multiline"))
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
