//! Exact value-level ports for Wave B1 `data_binding_test.cpp` cases not in SRIV corpus.

use std::path::PathBuf;

use nuxie::File;

fn pinned(name: &str) -> Vec<u8> {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let path = PathBuf::from(root)
        .join("tests/unit_tests/assets")
        .join(name);
    std::fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
}

fn key(owner: &str, property: &str) -> u16 {
    let definition = nuxie_schema::definition_by_name(owner).expect("schema owner");
    std::iter::once(definition.name)
        .chain(definition.ancestors.iter().copied())
        .filter_map(nuxie_schema::definition_by_name)
        .flat_map(|owner| owner.properties)
        .find(|candidate| candidate.name == property)
        .unwrap_or_else(|| panic!("{owner}.{property}"))
        .key
        .int
}

fn local(artboard: &nuxie::ArtboardInstance<'_>, name: &str) -> usize {
    artboard
        .artboard()
        .graph()
        .component_named(name)
        .unwrap_or_else(|| panic!("component {name}"))
        .local_id
}

fn local_of_type(artboard: &nuxie::ArtboardInstance<'_>, type_name: &str) -> usize {
    artboard
        .artboard()
        .graph()
        .components
        .iter()
        .find(|component| component.type_name == type_name)
        .unwrap_or_else(|| panic!("component of type {type_name}"))
        .local_id
}

fn assert_live_bool(
    artboard: &mut nuxie::ArtboardInstance<'_>,
    local_id: usize,
    owner: &str,
    property: &str,
    expected: bool,
) {
    // The runtime exposes the exact generated bool setter but deliberately
    // keeps the raw getter internal. A generated setter reports `false` for a
    // no-op write, so an opposite write followed by restoration is an exact,
    // executable observation of the pre-write value.
    assert!(
        artboard
            .raw_mut()
            .set_bool_property(local_id, key(owner, property), !expected),
        "{owner}.{property} was not {expected} before the opposite write"
    );
    assert!(
        artboard
            .raw_mut()
            .set_bool_property(local_id, key(owner, property), expected),
        "{owner}.{property} did not restore to {expected}"
    );
}

fn number(artboard: &nuxie::ArtboardInstance<'_>, name: &str, owner: &str, property: &str) -> f32 {
    artboard
        .raw()
        .double_property(local(artboard, name), key(owner, property))
        .unwrap_or_else(|| panic!("{name}.{property}"))
}

fn string(
    artboard: &nuxie::ArtboardInstance<'_>,
    name: &str,
    owner: &str,
    property: &str,
) -> Vec<u8> {
    artboard
        .raw()
        .debug_string_property(local(artboard, name), key(owner, property))
        .unwrap_or_else(|| panic!("{name}.{property}"))
        .to_vec()
}

fn fixture(
    asset: &str,
    artboard_name: &str,
) -> (
    nuxie::ArtboardInstance<'static>,
    nuxie::StateMachineInstance,
    nuxie::ViewModelInstance,
) {
    let file = Box::leak(Box::new(
        File::import(&pinned(asset)).expect("fixture imports"),
    ));
    let mut artboard = file
        .artboard_named(artboard_name)
        .expect("named artboard")
        .instantiate()
        .expect("artboard instantiates");
    let view_model = artboard
        .instantiate_default_view_model_instance()
        .or_else(|| artboard.instantiate_view_model())
        .expect("view model");
    let mut machine = artboard
        .default_state_machine_instance()
        .expect("default machine");
    let _ = machine.bind_owned_view_model_handle(view_model.handle());
    let _ = artboard.bind_view_model(&view_model);
    (artboard, machine, view_model)
}

fn advance(
    artboard: &mut nuxie::ArtboardInstance<'_>,
    machine: &mut nuxie::StateMachineInstance,
    view_model: &mut nuxie::ViewModelInstance,
    seconds: f32,
) {
    artboard.advance_with_state_machines_and_view_model(
        std::slice::from_mut(machine),
        seconds,
        view_model,
    );
}

fn shared_fixture(
    artboard_name: &str,
    default_instance: bool,
) -> (
    nuxie::ArtboardInstance<'static>,
    nuxie::StateMachineInstance,
    nuxie::ViewModelInstance,
) {
    let file = Box::leak(Box::new(
        File::import(&pinned("shared_viewmodel_instance.riv")).expect("fixture imports"),
    ));
    let mut artboard = file
        .artboard_named(artboard_name)
        .expect("named artboard")
        .instantiate()
        .expect("artboard instantiates");
    let view_model = if default_instance {
        artboard
            .instantiate_default_view_model_instance()
            .expect("default view model")
    } else {
        artboard.instantiate_view_model().expect("new view model")
    };
    let mut machine = artboard
        .default_state_machine_instance()
        .expect("default machine");
    assert!(machine.bind_owned_view_model_handle(view_model.handle()));
    let _ = artboard.bind_view_model(&view_model);
    (artboard, machine, view_model)
}

fn nested_texts(artboard: &mut nuxie::ArtboardInstance<'_>) -> Vec<Vec<u8>> {
    let mut texts = Vec::new();
    artboard
        .raw_mut()
        .try_visit_nested_artboard_instances_mut(&mut |_depth, _graph_id, child| {
            if let Some(text) = child.root_text_value_run("text_run") {
                texts.push(text.to_vec());
            }
            Ok::<_, ()>(())
        })
        .expect("nested occurrence traversal");
    texts
}

fn descendant_of_type(
    artboard: &nuxie::ArtboardInstance<'_>,
    root_name: &str,
    type_name: &str,
) -> usize {
    let graph = artboard.artboard().graph();
    let root = graph
        .component_named(root_name)
        .unwrap_or_else(|| panic!("component {root_name}"))
        .local_id;
    graph
        .components
        .iter()
        .find(|candidate| {
            if candidate.type_name != type_name {
                return false;
            }
            let mut parent = candidate.parent_local;
            while let Some(local) = parent {
                if local == root {
                    return true;
                }
                parent = graph
                    .components
                    .iter()
                    .find(|component| component.local_id == local)
                    .and_then(|component| component.parent_local);
            }
            false
        })
        .unwrap_or_else(|| panic!("{root_name} descendant {type_name}"))
        .local_id
}

fn set_number(view_model: &mut nuxie::ViewModelInstance, name: &str, value: f32) {
    let _ = view_model.set_number(name, value);
    assert_eq!(
        view_model.raw().number_value_by_property_name_path(name),
        Some(value)
    );
}

fn set_string(view_model: &mut nuxie::ViewModelInstance, name: &str, value: &str) {
    let _ = view_model.set_string(name, value);
    assert_eq!(
        view_model
            .raw()
            .string_value_by_property_name_path(name)
            .as_deref(),
        Some(value.as_bytes())
    );
}

#[test]
fn calculate_and_to_string_converters_with_numbers() {
    let (mut artboard, mut machine, mut view_model) =
        fixture("data_binding_test.riv", "artboard-3");
    assert_eq!(
        number(
            &artboard,
            "num_prop",
            "CustomPropertyNumber",
            "propertyValue"
        ),
        0.0
    );
    assert_eq!(
        string(&artboard, "text_run_bound", "TextValueRun", "text"),
        b"text"
    );
    advance(&mut artboard, &mut machine, &mut view_model, 0.0);
    assert_eq!(
        number(
            &artboard,
            "num_prop",
            "CustomPropertyNumber",
            "propertyValue"
        ),
        34.0
    );
    assert_eq!(
        string(&artboard, "text_run_bound", "TextValueRun", "text"),
        b"6"
    );
    set_number(&mut view_model, "num1", -10.0);
    advance(&mut artboard, &mut machine, &mut view_model, 0.0);
    assert_eq!(
        number(
            &artboard,
            "num_prop",
            "CustomPropertyNumber",
            "propertyValue"
        ),
        -20.0
    );
    assert_eq!(
        string(&artboard, "text_run_bound", "TextValueRun", "text"),
        b"-3"
    );
}

#[test]
fn trim_string_converter() {
    let (mut artboard, mut machine, mut view_model) =
        fixture("data_binding_test.riv", "artboard-3");
    let reads = |artboard: &nuxie::ArtboardInstance<'_>| {
        [
            string(artboard, "second_text_run_no_trim", "TextValueRun", "text"),
            string(
                artboard,
                "second_text_run_trim_both",
                "TextValueRun",
                "text",
            ),
            string(
                artboard,
                "second_text_run_trim_start",
                "TextValueRun",
                "text",
            ),
            string(artboard, "second_text_run_trim_end", "TextValueRun", "text"),
        ]
    };
    assert_eq!(
        reads(&artboard),
        [
            b"text".to_vec(),
            b"text".to_vec(),
            b"text".to_vec(),
            b"text".to_vec()
        ]
    );
    advance(&mut artboard, &mut machine, &mut view_model, 0.0);
    assert_eq!(
        reads(&artboard),
        [
            b"     abc    ".to_vec(),
            b"abc".to_vec(),
            b"abc    ".to_vec(),
            b"     abc".to_vec()
        ]
    );
    set_string(&mut view_model, "text", "a b c ");
    advance(&mut artboard, &mut machine, &mut view_model, 0.0);
    assert_eq!(
        reads(&artboard),
        [
            b"a b c ".to_vec(),
            b"a b c".to_vec(),
            b"a b c ".to_vec(),
            b"a b c".to_vec()
        ]
    );
}

#[test]
fn to_string_converter_with_color_formatters() {
    let (mut artboard, mut machine, mut view_model) =
        fixture("data_binding_test.riv", "artboard-4");
    let reads = |artboard: &nuxie::ArtboardInstance<'_>| {
        [
            string(artboard, "RGBA_formatted_color_run", "TextValueRun", "text"),
            string(artboard, "rgba_formatted_color_run", "TextValueRun", "text"),
            string(artboard, "hls_formatted_color_run", "TextValueRun", "text"),
            string(artboard, "escaped_characters_run", "TextValueRun", "text"),
        ]
    };
    advance(&mut artboard, &mut machine, &mut view_model, 0.0);
    assert_eq!(
        reads(&artboard),
        [
            b"color: {red: 1E, green: 5A, blue: C8, alpha: FF}".to_vec(),
            b"color: {red: 30, green: 90, blue: 200, alpha: 255}".to_vec(),
            b"color: {hue: 219, luminance: 45, saturation: 74}".to_vec(),
            b"%r %g %b %a \\a".to_vec()
        ]
    );
    for (color, expected) in [
        (
            0x64c8_6432,
            [
                "color: {red: C8, green: 64, blue: 32, alpha: 64}",
                "color: {red: 200, green: 100, blue: 50, alpha: 100}",
                "color: {hue: 20, luminance: 49, saturation: 60}",
                "%r %g %b %a \\a",
            ],
        ),
        (
            0x6400_0a0f,
            [
                "color: {red: 00, green: 0A, blue: 0F, alpha: 64}",
                "color: {red: 0, green: 10, blue: 15, alpha: 100}",
                "color: {hue: 200, luminance: 3, saturation: 100}",
                "%r %g %b %a \\a",
            ],
        ),
    ] {
        let _ = view_model.set_color("col", color);
        advance(&mut artboard, &mut machine, &mut view_model, 0.0);
        assert_eq!(
            reads(&artboard),
            expected.map(|text| text.as_bytes().to_vec())
        );
    }
}

#[test]
fn range_mapper() {
    let (mut artboard, mut machine, mut view_model) =
        fixture("data_binding_test_2.riv", "artboard-2");
    advance(&mut artboard, &mut machine, &mut view_model, 0.0);
    let read = |artboard: &nuxie::ArtboardInstance<'_>| {
        (1..=5)
            .map(|n| {
                number(
                    artboard,
                    &format!("mapped-range-{n}"),
                    "CustomPropertyNumber",
                    "propertyValue",
                )
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(read(&artboard), [6.0, 3.0, 2.0, 2.0, 2.0]);
    for (input, expected) in [
        (-1.0, [1.0, 2.0, 2.0, 3.0, 2.0]),
        (0.0, [2.0, 2.0, 2.0, 3.0, 2.0]),
        (0.25, [2.12916, 2.12916, 2.12916, 2.87084, 2.0]),
        (2.0, [4.0, 3.0, 2.0, 2.0, 2.0]),
        (2.25, [4.25, 3.0, 2.12916, 2.0, 2.0]),
    ] {
        set_number(&mut view_model, "map-range-num", input);
        advance(&mut artboard, &mut machine, &mut view_model, 0.0);
        for (actual, expected) in read(&artboard).into_iter().zip(expected) {
            assert!((actual - expected).abs() < 0.0001);
        }
    }
}

#[test]
#[ignore = "expected-red: pad-string-3 retains '-' instead of the pinned empty string after initial advance"]
fn pad_string() {
    let (mut artboard, mut machine, mut view_model) =
        fixture("data_binding_test_2.riv", "artboard-3");
    advance(&mut artboard, &mut machine, &mut view_model, 0.0);
    let read = |artboard: &nuxie::ArtboardInstance<'_>| {
        (1..=3)
            .map(|n| {
                string(
                    artboard,
                    &format!("pad-string-{n}"),
                    "CustomPropertyString",
                    "propertyValue",
                )
            })
            .collect::<Vec<_>>()
    };
    assert_eq!(
        read(&artboard),
        [
            b"abcabcatext".to_vec(),
            b"textabcabcab".to_vec(),
            Vec::new()
        ]
    );
    for (input, expected) in [
        ("text-text-text", ["text-text-text", "text-text-text", ""]),
        ("", ["abcabcabcab", "abcabcabcabc", ""]),
    ] {
        set_string(&mut view_model, "pad-string", input);
        advance(&mut artboard, &mut machine, &mut view_model, 0.0);
        assert_eq!(
            read(&artboard),
            expected.map(|text| text.as_bytes().to_vec())
        );
    }
}

#[test]
fn advance_and_apply_can_skip_view_model_reset() {
    let (mut artboard, mut machine, view_model) = fixture("data_binding_test.riv", "artboard-2");
    machine
        .advance_and_apply_with_view_models(artboard.raw_mut(), 0.0, true)
        .expect("settle");
    assert!(
        view_model
            .raw_mut()
            .set_trigger_by_property_name("trigger-prop", 1)
    );
    machine
        .advance_and_apply_with_view_models(artboard.raw_mut(), 0.0, false)
        .expect("skip view models");
    assert_eq!(
        view_model
            .raw()
            .trigger_value_by_property_name("trigger-prop"),
        Some(1)
    );
    machine
        .advance_and_apply_with_view_models(artboard.raw_mut(), 0.0, true)
        .expect("advance view models");
    assert_eq!(
        view_model
            .raw()
            .trigger_value_by_property_name("trigger-prop"),
        Some(0)
    );
}

#[test]
fn view_model_runtime_properties() {
    let file = File::import(&pinned("viewmodel_runtime_file.riv")).expect("fixture imports");
    let vm = file.view_model_named("vm").expect("vm");
    let instance = vm.instantiate_default().expect("default vm instance");
    let properties = vm
        .properties()
        .map(|property| (property.name().unwrap_or_default(), property.type_name()))
        .collect::<std::collections::BTreeMap<_, _>>();
    for (name, ty) in [
        ("num", "ViewModelPropertyNumber"),
        ("str", "ViewModelPropertyString"),
        ("boo", "ViewModelPropertyBoolean"),
        ("col", "ViewModelPropertyColor"),
        ("tri", "ViewModelPropertyTrigger"),
        ("enu", "ViewModelPropertyEnumCustom"),
        ("ima", "ViewModelPropertyAssetImage"),
        ("art", "ViewModelPropertyArtboard"),
        ("lis", "ViewModelPropertyList"),
    ] {
        assert_eq!(properties.get(name), Some(&ty));
    }
    assert!(
        instance
            .raw()
            .number_value_by_property_name_path("chi/chi-num")
            .is_some()
    );
    assert_eq!(
        vm.property_named("enu")
            .expect("enu")
            .descriptor()
            .uint_property("enumId"),
        Some(0)
    );
}

fn two_way_fixture(
    artboard_name: &str,
) -> (
    nuxie::ArtboardInstance<'static>,
    nuxie::StateMachineInstance,
    nuxie::ViewModelInstance,
    usize,
) {
    let (artboard, machine, view_model) = fixture("bidirectional_precedence.riv", artboard_name);
    let target = artboard
        .artboard()
        .graph()
        .data_binds
        .iter()
        .find_map(|bind| {
            let target = bind.target_local?;
            let definition = bind
                .target_type_name
                .and_then(nuxie_schema::definition_by_name)?;
            definition.is_a("Node").then_some(target)
        })
        .expect("two-way Node target");
    (artboard, machine, view_model, target)
}

#[test]
#[ignore = "expected-red: target-first source x=500 settles retained Node.x at 252.5 instead of 500"]
fn two_way_source_change_reaches_target_under_target_first_precedence() {
    let (mut artboard, mut machine, mut view_model, target) = two_way_fixture("target_first");
    set_number(&mut view_model, "x", 100.0);
    set_number(&mut view_model, "y", 100.0);
    for seconds in std::iter::once(0.0).chain(std::iter::repeat_n(0.016, 10)) {
        advance(&mut artboard, &mut machine, &mut view_model, seconds);
    }
    assert_eq!(
        view_model.raw().number_value_by_property_name("x"),
        artboard.raw().double_property(target, key("Node", "x"))
    );
    assert_eq!(
        view_model.raw().number_value_by_property_name("y"),
        artboard.raw().double_property(target, key("Node", "y"))
    );
    set_number(&mut view_model, "x", 500.0);
    set_number(&mut view_model, "y", 600.0);
    for _ in 0..20 {
        advance(&mut artboard, &mut machine, &mut view_model, 0.016);
    }
    assert_eq!(
        artboard.raw().double_property(target, key("Node", "x")),
        Some(500.0)
    );
    assert_eq!(
        artboard.raw().double_property(target, key("Node", "y")),
        Some(600.0)
    );
}

#[test]
#[ignore = "expected-red: source-first target Node.x=700 leaves bound source x at 100 instead of 700"]
fn two_way_target_change_reaches_source_under_source_first_precedence() {
    let (mut artboard, mut machine, mut view_model, target) = two_way_fixture("source_first");
    set_number(&mut view_model, "x", 100.0);
    set_number(&mut view_model, "y", 100.0);
    advance(&mut artboard, &mut machine, &mut view_model, 0.0);
    assert!(
        artboard
            .raw_mut()
            .set_double_property(target, key("Node", "x"), 700.0)
    );
    assert!(
        artboard
            .raw_mut()
            .set_double_property(target, key("Node", "y"), 800.0)
    );
    advance(&mut artboard, &mut machine, &mut view_model, 0.016);
    assert_eq!(
        view_model.raw().number_value_by_property_name("x"),
        Some(700.0)
    );
    assert_eq!(
        view_model.raw().number_value_by_property_name("y"),
        Some(800.0)
    );
}

#[test]
fn same_view_model_instance_is_shared_by_two_properties() {
    let (mut artboard, mut machine, mut view_model) = shared_fixture("main", true);
    advance(&mut artboard, &mut machine, &mut view_model, 0.0);
    assert_eq!(
        nested_texts(&mut artboard),
        [b"label-vmi-1".to_vec(), b"label-vmi-1".to_vec()]
    );

    let child = view_model
        .handle()
        .linked_view_model_by_property_name_path("child1")
        .expect("child1 linked view model");
    assert!(
        child
            .borrow_mut()
            .set_string_by_property_name_path("label", b"label-update")
    );
    advance(&mut artboard, &mut machine, &mut view_model, 0.0);
    assert_eq!(
        nested_texts(&mut artboard),
        [b"label-update".to_vec(), b"label-update".to_vec()]
    );
}

#[test]
fn different_view_model_instances_are_not_shared_by_two_properties() {
    let (mut artboard, mut machine, mut view_model) = shared_fixture("main_2", true);
    advance(&mut artboard, &mut machine, &mut view_model, 0.0);
    assert_eq!(
        nested_texts(&mut artboard),
        [b"label-vmi-1".to_vec(), b"label-vmi-2".to_vec()]
    );

    let child = view_model
        .handle()
        .linked_view_model_by_property_name_path("vm_2_child1")
        .expect("vm_2_child1 linked view model");
    assert!(
        child
            .borrow_mut()
            .set_string_by_property_name_path("label", b"label-update")
    );
    advance(&mut artboard, &mut machine, &mut view_model, 0.0);
    assert_eq!(
        nested_texts(&mut artboard),
        [b"label-update".to_vec(), b"label-vmi-2".to_vec()]
    );
}

#[test]
fn newly_created_view_model_instances_do_not_share_nested_instances() {
    let (mut artboard, mut machine, mut view_model) = shared_fixture("main_2", false);
    advance(&mut artboard, &mut machine, &mut view_model, 0.0);
    assert_eq!(nested_texts(&mut artboard), [Vec::new(), Vec::new()]);

    let child = view_model
        .handle()
        .linked_view_model_by_property_name_path("vm_2_child1")
        .expect("vm_2_child1 linked view model");
    assert!(
        child
            .borrow_mut()
            .set_string_by_property_name_path("label", b"label-update")
    );
    advance(&mut artboard, &mut machine, &mut view_model, 0.0);
    assert_eq!(
        nested_texts(&mut artboard),
        [b"label-update".to_vec(), Vec::new()]
    );
}

#[test]
fn triggers_updated_by_events_update_parent_state() {
    let file = Box::leak(Box::new(
        File::import(&pinned("data_binding_test_triggers.riv")).expect("fixture imports"),
    ));
    let mut artboard = file
        .artboard_named("root")
        .expect("root artboard")
        .instantiate()
        .expect("artboard instantiates");
    let mut view_model = artboard.instantiate_view_model().expect("new view model");
    let mut machine = artboard
        .default_state_machine_instance()
        .expect("default machine");
    assert!(machine.bind_owned_view_model_handle(view_model.handle()));
    let _ = artboard.bind_view_model(&view_model);
    let color = descendant_of_type(&artboard, "main_rect", "SolidColor");
    let read = |artboard: &nuxie::ArtboardInstance<'_>| {
        artboard
            .raw()
            .color_property(color, key("SolidColor", "colorValue"))
            .expect("main_rect SolidColor.colorValue")
    };

    advance(&mut artboard, &mut machine, &mut view_model, 0.0);
    assert_eq!(read(&artboard), 0xffff_0000);
    advance(&mut artboard, &mut machine, &mut view_model, 0.7);
    advance(&mut artboard, &mut machine, &mut view_model, 0.1);
    assert_eq!(read(&artboard), 0xff00_ff00);
}

#[test]
fn state_machine_is_led_by_bound_enum_and_trigger() {
    let (mut artboard, mut machine, mut view_model) =
        fixture("data_binding_test.riv", "artboard-2");
    let color = descendant_of_type(&artboard, "color_rectangle", "SolidColor");
    let read_color = |artboard: &nuxie::ArtboardInstance<'_>| {
        artboard
            .raw()
            .color_property(color, key("SolidColor", "colorValue"))
            .expect("color_rectangle SolidColor.colorValue")
    };
    let read_position = |artboard: &nuxie::ArtboardInstance<'_>| {
        (
            number(artboard, "color_rectangle", "Node", "x"),
            number(artboard, "color_rectangle", "Node", "y"),
        )
    };

    assert_eq!(read_position(&artboard), (250.0, 250.0));
    assert_eq!(read_color(&artboard), 0xff74_7474);
    advance(&mut artboard, &mut machine, &mut view_model, 0.0);
    assert_eq!(read_color(&artboard), 0xffff_0000);

    assert!(view_model.set_enum("state", 1));
    advance(&mut artboard, &mut machine, &mut view_model, 0.0);
    assert_eq!(read_color(&artboard), 0xff00_ff00);
    assert_eq!(read_position(&artboard), (150.0, 250.0));

    // The pinned enum member named `state-blue` is the third member (index 2).
    assert!(view_model.set_enum("state", 2));
    assert!(view_model.fire_trigger("trigger-prop"));
    advance(&mut artboard, &mut machine, &mut view_model, 0.0);
    assert_eq!(read_color(&artboard), 0xff00_00ff);
    assert_eq!(read_position(&artboard), (350.0, 250.0));

    assert!(view_model.fire_trigger("trigger-prop"));
    advance(&mut artboard, &mut machine, &mut view_model, 0.0);
    assert_eq!(read_position(&artboard), (350.0, 350.0));
}

#[test]
fn artboard_has_bound_properties() {
    let file = Box::leak(Box::new(
        File::import(&pinned("data_binding_test.riv")).expect("fixture imports"),
    ));
    let mut artboard = file
        .artboard_named("artboard-1")
        .expect("artboard-1")
        .instantiate()
        .expect("artboard instantiates");
    let mut view_model = artboard
        .instantiate_default_view_model_instance()
        .expect("default view model");
    assert!(artboard.bind_view_model(&view_model));
    artboard.advance(0.0);

    let rectangle = local(&artboard, "bound_rect");
    let shape = local(&artboard, "bound_rect_shape");
    let solid = descendant_of_type(&artboard, "bound_rect_shape", "SolidColor");
    let text = local(&artboard, "bound_text_run");
    let follow = local_of_type(&artboard, "FollowPathConstraint");
    assert_eq!(
        artboard
            .raw()
            .double_property(rectangle, key("Rectangle", "width")),
        Some(100.0)
    );
    assert!(
        (artboard
            .raw()
            .double_property(shape, key("Node", "rotation"))
            .expect("bound_rect_shape rotation")
            - 1.5708)
            .abs()
            < 0.0001
    );
    assert_eq!(
        artboard
            .raw()
            .color_property(solid, key("SolidColor", "colorValue")),
        Some(0xffff_0000)
    );
    assert_eq!(
        artboard
            .raw()
            .debug_string_property(text, key("TextValueRun", "text")),
        Some(b"bound text".as_slice())
    );
    assert_live_bool(
        &mut artboard,
        follow,
        "FollowPathConstraint",
        "orient",
        false,
    );

    assert!(view_model.set_number("width", 200.0));
    assert!(view_model.set_number("rotation", 180.0));
    assert!(view_model.set_color("color", 0xff00_ff00));
    assert!(view_model.set_string("text", "New text"));
    assert!(view_model.set_bool("orient", true));
    artboard.advance(0.0);
    assert_eq!(
        artboard
            .raw()
            .double_property(rectangle, key("Rectangle", "width")),
        Some(200.0)
    );
    assert!(
        (artboard
            .raw()
            .double_property(shape, key("Node", "rotation"))
            .expect("bound_rect_shape rotation")
            - std::f32::consts::PI)
            .abs()
            < 0.0001
    );
    assert_eq!(
        artboard
            .raw()
            .color_property(solid, key("SolidColor", "colorValue")),
        Some(0xff00_ff00)
    );
    assert_eq!(
        artboard
            .raw()
            .debug_string_property(text, key("TextValueRun", "text")),
        Some(b"New text".as_slice())
    );
    assert_live_bool(
        &mut artboard,
        follow,
        "FollowPathConstraint",
        "orient",
        true,
    );
}

#[test]
fn boolean_toggle_converter_negates_bound_value() {
    let (mut artboard, mut machine, mut view_model) =
        fixture("data_binding_test_2.riv", "artboard-3");
    advance(&mut artboard, &mut machine, &mut view_model, 0.0);
    let target = local(&artboard, "negate-bool-1");
    assert_live_bool(
        &mut artboard,
        target,
        "CustomPropertyBoolean",
        "propertyValue",
        true,
    );
    assert_eq!(
        view_model
            .raw()
            .boolean_value_by_property_name_path("bool-prop"),
        Some(false)
    );

    assert!(view_model.set_bool("bool-prop", true));
    advance(&mut artboard, &mut machine, &mut view_model, 0.0);
    assert_live_bool(
        &mut artboard,
        target,
        "CustomPropertyBoolean",
        "propertyValue",
        false,
    );
}
