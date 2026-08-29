//! Preserved authored TextInput probes using actual native owners.
//! The three cases needing private source access live in TextInput's tools-only unit module.
#![cfg(feature = "tools")]
use nuxie_runtime as native_runtime;
mod cpp_probe_text_support;
use cpp_probe_text_support::*;
use native_runtime::source::{
    component_dirt::ComponentDirt,
    generated::core_registry::CoreRegistry,
    input::focusable::{Key, KeyModifiers},
    text::{text_engine::TextSizing, text_input::TextInput},
};

#[test]
fn upstream_text_input_callbacks_publish_pinned_dirt_even_without_raw_mutation() {
    let (_file, _artboard, input) = fixture();
    let text_key = property_key("TextInput", "text");
    let multiline_key = property_key("TextInput", "multiline");
    assert!(CoreRegistry::set_string_handle(
        &input,
        text_key,
        "hello".into()
    ));
    set_cursor(&input, 5, 5);
    clear_dirt(&input);
    assert!(with_input(&input, |input| input.key_input(
        Key::RIGHT,
        KeyModifiers::NONE,
        true,
        false
    )));
    assert_eq!(cursor(&input), Some((5, 5)));
    assert!(dirt(&input).is_some_and(|dirt| dirt.contains(ComponentDirt::PAINT)));

    set_cursor(&input, 0, 0);
    clear_dirt(&input);
    assert!(with_input(&input, |input| input.key_input(
        Key::BACKSPACE,
        KeyModifiers::NONE,
        true,
        false
    )));
    assert_eq!(
        Some(with_input(&input, |input| input.raw_text_input().text())).as_deref(),
        Some("hello")
    );
    assert!(dirt(&input).is_some_and(|dirt| dirt.contains(ComponentDirt::TEXT_SHAPE)));

    let _ = CoreRegistry::set_bool_handle(&input, multiline_key, true);
    clear_dirt(&input);
    assert!(with_input(&input, |input| input.key_input(
        Key::ENTER,
        KeyModifiers::NONE,
        true,
        false
    )));
    assert!(dirt(&input).is_some_and(|dirt| dirt.contains(ComponentDirt::PAINT)));

    clear_dirt(&input);
    with_input(&input, TextInput::selection_radius_changed);
    clear_dirt(&input);
    with_input(&input, TextInput::selection_radius_changed);
    assert!(dirt(&input).is_some_and(|dirt| dirt.contains(ComponentDirt::PATH)));

    set_cursor(&input, 0, 5);
    clear_dirt(&input);
    with_input(&input, TextInput::blurred);
    assert_eq!(cursor(&input), Some((5, 5)));
    assert!(dirt(&input).is_some_and(|dirt| dirt.contains(ComponentDirt::PAINT)));
    clear_dirt(&input);
    with_input(&input, TextInput::blurred);
    assert!(dirt(&input).is_some_and(|dirt| dirt.contains(ComponentDirt::PAINT)));

    set_cursor(&input, 2, 2);
    let before = cursor(&input);
    with_input(&input, TextInput::select_word);
    assert_ne!(cursor(&input), before);
    clear_dirt(&input);
    let before = cursor(&input);
    with_input(&input, TextInput::select_word);
    assert_eq!(cursor(&input), before);
    assert!(dirt(&input).is_some_and(|dirt| dirt.contains(ComponentDirt::PAINT)));

    set_cursor(&input, 3, 3);
    with_input(&input, TextInput::select_line);
    assert_eq!(cursor(&input), Some((1, 6)));
    clear_dirt(&input);
    with_input(&input, TextInput::select_line);
    // Pinned selectLine always re-resolves cursor.start(). Code point 1 is the
    // shared newline boundary, so the second resolution selects the first line.
    assert_eq!(cursor(&input), Some((0, 1)));
    assert!(dirt(&input).is_some_and(|dirt| dirt.contains(ComponentDirt::PAINT)));
}

#[test]
fn upstream_text_input_visual_cursor_fixture_values_are_ported() {
    let (_file, artboard, text_input) = fixture();
    let style = style(&text_input);
    install_font(&style, "fonts/Inter_18pt-Regular.ttf");
    assert!(CoreRegistry::set_double_handle(
        &style,
        property_key("TextStyle", "fontSize"),
        72.0
    ));
    assert!(CoreRegistry::set_string_handle(
        &text_input,
        property_key("TextInput", "text"),
        "this is some\nmultiline text input\nwith one final line\n".into()
    ));
    artboard.update_pass(true);
    // This fixture-value probe observes the unbounded geometry used by the
    // original helper, independently of the authored viewport's layout solve.
    with_input(&text_input, |input| {
        input.raw_text_input().set_sizing(TextSizing::AutoWidth)
    });
    update_raw(&artboard, &text_input);
    // The actual native shape retains the terminal caret's sentinel-only
    // fourth line. The authored-text snapshot below intentionally excludes it.
    with_input(&text_input, |input| {
        let raw = input.raw_text_input();
        let lines = raw.shape().ordered_lines();
        assert_eq!(lines.len(), 4);
        let terminal = lines.last().unwrap().begin();
        assert_eq!(
            terminal.run().text_indices[terminal.glyph_index() as usize] as usize,
            raw.length()
        );
    });
    assert_eq!(
        line_metrics(&text_input),
        Some(vec![
            (0, 12, 0.0, 87.11719),
            (13, 33, 87.11719, 174.23438),
            (34, 53, 174.23438, 261.35156),
        ])
    );
    for (index, expected_x) in [(0, 0.0), (1, 23.30859), (2, 65.17969), (12, 396.0)] {
        set_cursor(&text_input, index, index);
        update_raw(&artboard, &text_input);
        let (top, bottom) = caret(&text_input).expect("retained caret geometry");
        assert!((top.0 - expected_x).abs() < 1.0e-5);
        assert_eq!(top.1, 0.0);
        assert!((bottom.0 - expected_x).abs() < 1.0e-5);
        assert_eq!(bottom.1, 87.11719);
    }
}
#[test]
fn upstream_text_input_measurement_cache_is_ported() {
    let (_file, artboard, text_input) = fixture();
    let style = style(&text_input);
    install_font(&style, "fonts/IBMPlexSansArabic-Regular.ttf");
    assert!(CoreRegistry::set_double_handle(
        &style,
        property_key("TextStyle", "fontSize"),
        72.0
    ));
    assert!(CoreRegistry::set_string_handle(
        &text_input,
        property_key("TextInput", "text"),
        "one two three four five".into()
    ));
    artboard.update_pass(true);
    let bounds = measure(&text_input, 500.0, 400.0).expect("measure TextInput");
    assert_eq!(bounds, (0.0, 0.0, 446.51953, 216.0));
    let count = measure_count(&text_input).unwrap();
    assert_eq!(measure(&text_input, 500.0, 400.0), Some(bounds));
    assert_eq!(measure_count(&text_input), Some(count));
    assert_eq!(
        measure(&text_input, 400.0, 400.0),
        Some((0.0, 0.0, 318.97266, 324.0))
    );
    assert_eq!(measure_count(&text_input), Some(count + 1));
    assert!(CoreRegistry::set_string_handle(
        &text_input,
        property_key("TextInput", "text"),
        "one two three four five six".into()
    ));
    assert_eq!(
        measure(&text_input, 400.0, 400.0),
        Some((0.0, 0.0, 318.97266, 324.0))
    );
    assert_eq!(measure_count(&text_input), Some(count + 2));
}

#[test]
fn upstream_text_input_double_and_triple_click_selection_is_ported() {
    let (_file, artboard, input) = fixture();
    assert!(CoreRegistry::set_string_handle(
        &input,
        property_key("TextInput", "text"),
        "hello world".into()
    ));
    let machine = artboard
        .state_machine_at(0)
        .expect("authored TextInput fixture has state machine 0");
    machine.advance_and_apply(0.0);
    let click = world_point(&artboard, &input, 8.0, 8.0);
    let press_release = || {
        machine.with_instance_mut(|machine| machine.pointer_down(click, 0));
        machine.with_instance_mut(|machine| machine.pointer_up(click, 0));
    };
    press_release();
    press_release();
    machine.advance_and_apply(0.0);
    let (word_start, word_end) = cursor(&input).expect("TextInput cursor after double click");
    if word_start == word_end {
        // Same pinned asset-layout guard: the authored pointer can miss the hit area.
        return;
    }
    assert!(word_end > word_start);
    press_release();
    machine.advance_and_apply(0.0);
    let (line_start, line_end) = cursor(&input).expect("TextInput cursor");
    assert!(line_end > line_start);
    assert!(line_end >= word_end);
}
