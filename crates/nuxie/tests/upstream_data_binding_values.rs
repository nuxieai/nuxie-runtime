//! Exact value-level ports for Wave B1 `data_binding_test.cpp` cases not in SRIV corpus.

use std::path::PathBuf;

use nuxie::{
    CoreHandle, File, PersistentFactory, RuntimeArtboardInstanceHandle, RuntimeFactoryHandle,
    RuntimeFileHandle, RuntimeStateMachineInstanceHandle, RuntimeViewModelInstanceHandle,
    ViewModelInstanceRuntime,
};
use nuxie_render_api::SerializingFactory;
use nuxie_runtime::{
    ViewModelRuntimeDataType,
    source::{
        generated::{component_base::ComponentBase, core_registry::CoreRegistry},
        text::text_value_run::TextValueRun,
    },
};
use silver_corpus::{compare_sriv, parse_sriv};

fn catch_approx_eq(actual: f32, expected: f32) -> bool {
    let actual = f64::from(actual);
    let expected = f64::from(expected);
    let scale = f64::from(f32::EPSILON) * 100.0 * expected.abs();
    let difference = (actual - expected).abs();
    difference <= scale
}

fn assert_catch_approx(actual: f32, expected: f32) {
    assert!(
        catch_approx_eq(actual, expected),
        "{actual} is not Catch Approx({expected})"
    );
}

fn pinned(name: &str) -> Vec<u8> {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let path = PathBuf::from(root)
        .join("tests/unit_tests/assets")
        .join(name);
    std::fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
}

fn pinned_silver(name: &str) -> Vec<u8> {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let path = PathBuf::from(root)
        .join("tests/unit_tests/silvers")
        .join(name);
    std::fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
}

#[test]
fn catch_approx_widens_float_operands_before_comparing() {
    let expected = f32::from_bits(0x0072_abfc);
    let actual = f32::from_bits(expected.to_bits() + 90);
    assert!(!catch_approx_eq(actual, expected));
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

fn object(artboard: &RuntimeArtboardInstanceHandle, local_id: usize) -> CoreHandle {
    artboard
        .with_artboard(|artboard| artboard.base.resolve_handle(local_id as u32))
        .unwrap_or_else(|| panic!("live object {local_id}"))
}

fn local(artboard: &RuntimeArtboardInstanceHandle, name: &str) -> usize {
    let objects = artboard.with_artboard(|artboard| artboard.base.objects().to_vec());
    objects
        .iter()
        .enumerate()
        .find_map(|(local_id, component)| {
            let component = component.as_ref()?;
            (CoreRegistry::get_string_handle(
                component,
                i32::from(ComponentBase::NAME_PROPERTY_KEY),
            )
            .as_deref()
                == Some(name))
            .then_some(local_id)
        })
        .unwrap_or_else(|| panic!("component {name}"))
}

fn local_of_type(artboard: &RuntimeArtboardInstanceHandle, type_name: &str) -> usize {
    let objects = artboard.with_artboard(|artboard| artboard.base.objects().to_vec());
    objects
        .iter()
        .enumerate()
        .find_map(|(local_id, component)| {
            let component = component.as_ref()?;
            let definition = nuxie_schema::definition_by_type_key(component.core_type()?)?;
            (definition.name == type_name).then_some(local_id)
        })
        .unwrap_or_else(|| panic!("component of type {type_name}"))
}

fn assert_live_bool(
    artboard: &RuntimeArtboardInstanceHandle,
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
        CoreRegistry::set_bool_handle(
            &object(artboard, local_id),
            i32::from(key(owner, property)),
            !expected
        ),
        "{owner}.{property} was not {expected} before the opposite write"
    );
    assert!(
        CoreRegistry::set_bool_handle(
            &object(artboard, local_id),
            i32::from(key(owner, property)),
            expected
        ),
        "{owner}.{property} did not restore to {expected}"
    );
}

fn number(
    artboard: &RuntimeArtboardInstanceHandle,
    name: &str,
    owner: &str,
    property: &str,
) -> f32 {
    CoreRegistry::get_double_handle(
        &object(artboard, local(artboard, name)),
        i32::from(key(owner, property)),
    )
    .unwrap_or_else(|| panic!("{name}.{property}"))
}

fn string(
    artboard: &RuntimeArtboardInstanceHandle,
    name: &str,
    owner: &str,
    property: &str,
) -> Vec<u8> {
    CoreRegistry::get_string_handle(
        &object(artboard, local(artboard, name)),
        i32::from(key(owner, property)),
    )
    .unwrap_or_else(|| panic!("{name}.{property}"))
    .into_bytes()
}

fn import(asset: &str) -> RuntimeFileHandle {
    let mut factory = PersistentFactory::new(SerializingFactory::new());
    let factory = RuntimeFactoryHandle::from_factory(&mut factory).expect("retained factory");
    File::import(&pinned(asset), factory, None, None, None).expect("fixture imports")
}

fn artboard_named(file: &RuntimeFileHandle, name: &str) -> RuntimeArtboardInstanceHandle {
    file.with_file(|file| file.artboard_named(name))
        .unwrap_or_else(|| panic!("named artboard {name}"))
}

fn fresh_view_model(
    file: &RuntimeFileHandle,
    artboard: &RuntimeArtboardInstanceHandle,
    default_instance: bool,
) -> RuntimeViewModelInstanceHandle {
    let instance = file
        .with_file_mut(|file| {
            if default_instance {
                file.create_default_view_model_instance_for_artboard(artboard.core_handle())
            } else {
                file.create_view_model_instance_for_artboard(artboard.core_handle())
            }
        })
        .expect("artboard view model");
    ViewModelInstanceRuntime::new(instance).into_handle()
}

fn bind_view_model(
    artboard: &RuntimeArtboardInstanceHandle,
    machine: &RuntimeStateMachineInstanceHandle,
    view_model: &RuntimeViewModelInstanceHandle,
) {
    machine.with_instance_mut(|machine| machine.bind_view_model_instance(view_model.instance()));
    artboard.bind_view_model_instance(Some(view_model.instance()));
}

fn fixture(
    asset: &str,
    artboard_name: &str,
) -> (
    RuntimeArtboardInstanceHandle,
    RuntimeStateMachineInstanceHandle,
    RuntimeViewModelInstanceHandle,
) {
    let file = import(asset);
    let artboard = artboard_named(&file, artboard_name);
    let view_model = file
        .with_file_mut(|file| {
            file.create_default_view_model_instance_for_artboard(artboard.core_handle())
                .or_else(|| file.create_view_model_instance_for_artboard(artboard.core_handle()))
        })
        .map(ViewModelInstanceRuntime::new)
        .map(ViewModelInstanceRuntime::into_handle)
        .expect("view model");
    let machine = artboard
        .default_state_machine_handle()
        .expect("default machine");
    bind_view_model(&artboard, &machine, &view_model);
    (artboard, machine, view_model)
}

fn advance(
    _artboard: &RuntimeArtboardInstanceHandle,
    machine: &RuntimeStateMachineInstanceHandle,
    _view_model: &RuntimeViewModelInstanceHandle,
    seconds: f32,
) {
    machine.advance_and_apply(seconds);
}

fn shared_fixture(
    artboard_name: &str,
    default_instance: bool,
) -> (
    RuntimeArtboardInstanceHandle,
    RuntimeStateMachineInstanceHandle,
    RuntimeViewModelInstanceHandle,
) {
    let file = import("shared_viewmodel_instance.riv");
    let artboard = artboard_named(&file, artboard_name);
    let view_model = fresh_view_model(&file, &artboard, default_instance);
    let machine = artboard
        .default_state_machine_handle()
        .expect("default machine");
    bind_view_model(&artboard, &machine, &view_model);
    (artboard, machine, view_model)
}

fn nested_texts(artboard: &RuntimeArtboardInstanceHandle) -> Vec<Vec<u8>> {
    artboard
        .with_artboard(|artboard| artboard.base.nested_artboards())
        .into_iter()
        .filter_map(|nested| nested.with(|nested| nested.nested_artboard_instance_handle()).flatten())
        .filter_map(|child| {
            child.with_artboard(|child| child.base.find_handle::<TextValueRun>("text_run"))
        })
        .filter_map(|run| {
            CoreRegistry::get_string_handle(
                &run,
                i32::from(nuxie_runtime::source::generated::text::text_value_run_base::TextValueRunBase::TEXT_PROPERTY_KEY),
            )
        })
        .map(String::into_bytes)
        .collect()
}

fn descendant_of_type(
    artboard: &RuntimeArtboardInstanceHandle,
    root_name: &str,
    type_name: &str,
) -> usize {
    let root = local(artboard, root_name);
    let root_handle = object(artboard, root);
    let objects = artboard.with_artboard(|artboard| artboard.base.objects().to_vec());
    objects
        .iter()
        .enumerate()
        .find_map(|(local_id, candidate)| {
            let candidate = candidate.as_ref()?;
            let definition = nuxie_schema::definition_by_type_key(candidate.core_type()?)?;
            if definition.name != type_name {
                return None;
            }
            let mut parent = candidate
                .with(|candidate| candidate.as_component()?.parent_handle())
                .flatten();
            let mut remaining = objects.len();
            while let Some(parent_handle) = parent {
                if parent_handle == root_handle {
                    return Some(local_id);
                }
                if remaining == 0 {
                    return None;
                }
                remaining -= 1;
                parent = parent_handle
                    .with(|parent| parent.as_component()?.parent_handle())
                    .flatten();
            }
            None
        })
        .unwrap_or_else(|| panic!("{root_name} descendant {type_name}"))
}

fn set_number(view_model: &RuntimeViewModelInstanceHandle, name: &str, value: f32) {
    let property = view_model
        .property_number(name)
        .unwrap_or_else(|| panic!("number {name}"));
    property.set_value(value);
    assert_eq!(property.value(), value);
}

fn set_string(view_model: &RuntimeViewModelInstanceHandle, name: &str, value: &str) {
    let property = view_model
        .property_string(name)
        .unwrap_or_else(|| panic!("string {name}"));
    property.set_value(value);
    assert_eq!(property.value(), value);
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
    let reads = |artboard: &RuntimeArtboardInstanceHandle| {
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
    let reads = |artboard: &RuntimeArtboardInstanceHandle| {
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
        view_model
            .property_color("col")
            .expect("col")
            .set_value(color as i32);
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
    let read = |artboard: &RuntimeArtboardInstanceHandle| {
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
    assert_eq!(
        view_model
            .property_number("map-range-num")
            .expect("map-range-num")
            .value(),
        4.0
    );

    set_number(&mut view_model, "map-range-num", -1.0);
    advance(&mut artboard, &mut machine, &mut view_model, 0.0);
    assert_eq!(read(&artboard), [1.0, 2.0, 2.0, 3.0, 2.0]);

    set_number(&mut view_model, "map-range-num", 0.0);
    advance(&mut artboard, &mut machine, &mut view_model, 0.0);
    assert_eq!(read(&artboard), [2.0, 2.0, 2.0, 3.0, 2.0]);

    set_number(&mut view_model, "map-range-num", 0.25);
    advance(&mut artboard, &mut machine, &mut view_model, 0.0);
    let values = read(&artboard);
    assert_catch_approx(values[0], 2.12916);
    assert_catch_approx(values[1], 2.12916);
    assert_catch_approx(values[2], 2.12916);
    assert_catch_approx(values[3], 2.87084);
    assert_eq!(values[4], 2.0);

    set_number(&mut view_model, "map-range-num", 2.0);
    advance(&mut artboard, &mut machine, &mut view_model, 0.0);
    assert_eq!(read(&artboard), [4.0, 3.0, 2.0, 2.0, 2.0]);

    set_number(&mut view_model, "map-range-num", 2.25);
    advance(&mut artboard, &mut machine, &mut view_model, 0.0);
    let values = read(&artboard);
    assert_eq!(values[0], 4.25);
    assert_eq!(values[1], 3.0);
    assert_catch_approx(values[2], 2.12916);
    assert_eq!(values[3], 2.0);
    assert_eq!(values[4], 2.0);
}

#[test]
fn pad_string() {
    let (mut artboard, mut machine, mut view_model) =
        fixture("data_binding_test_2.riv", "artboard-3");
    advance(&mut artboard, &mut machine, &mut view_model, 0.0);
    let read = |artboard: &RuntimeArtboardInstanceHandle| {
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
    let (_artboard, machine, view_model) = fixture("data_binding_test.riv", "artboard-2");
    machine.advance_and_apply_view_models(0.0, true);
    let trigger = view_model
        .property_trigger("trigger-prop")
        .expect("trigger-prop");
    trigger.trigger();
    let trigger_value = || {
        trigger
            .value_runtime()
            .handle()
            .with(|property| {
                property
                    .as_view_model_instance_trigger()
                    .map(|property| property.base.property_value())
            })
            .flatten()
            .expect("trigger value")
    };
    machine.advance_and_apply_view_models(0.0, false);
    assert_eq!(trigger_value(), 1);
    machine.advance_and_apply_view_models(0.0, true);
    assert_eq!(trigger_value(), 0);
}

#[test]
fn view_model_runtime_properties() {
    let file = import("viewmodel_runtime_file.riv");
    let runtime = file
        .with_file(|file| file.view_model_by_name("vm"))
        .expect("vm runtime");
    let instance = runtime.create_default_instance();
    assert_eq!(instance.view_model_name(), "vm");
    let _ = instance.property_number("num").expect("num");
    assert_eq!(
        instance.property("num").expect("cached num").data_type(),
        ViewModelRuntimeDataType::Number
    );
    let _ = instance.property_string("str").expect("str");
    assert_eq!(
        instance.property("str").expect("cached str").data_type(),
        ViewModelRuntimeDataType::String
    );
    assert!(instance.property_number("str").is_none());
    let _ = instance.property_boolean("boo").expect("boo");
    assert_eq!(
        instance.property("boo").expect("cached boo").data_type(),
        ViewModelRuntimeDataType::Boolean
    );
    let _ = instance.property_color("col").expect("col");
    assert_eq!(
        instance.property("col").expect("cached col").data_type(),
        ViewModelRuntimeDataType::Color
    );
    let _ = instance.property_trigger("tri").expect("tri");
    assert_eq!(
        instance.property("tri").expect("cached tri").data_type(),
        ViewModelRuntimeDataType::Trigger
    );
    let _ = instance.property_enum("enu").expect("enu");
    assert_eq!(
        instance.property("enu").expect("cached enu").data_type(),
        ViewModelRuntimeDataType::Enum
    );
    let _ = instance.property_image("ima").expect("ima");
    assert_eq!(
        instance.property("ima").expect("cached ima").data_type(),
        ViewModelRuntimeDataType::AssetImage
    );
    let _ = instance.property_artboard("art").expect("art");
    assert_eq!(
        instance.property("art").expect("cached art").data_type(),
        ViewModelRuntimeDataType::Artboard
    );
    let _ = instance.property_list("lis").expect("lis");
    assert_eq!(
        instance.property("lis").expect("cached lis").data_type(),
        ViewModelRuntimeDataType::List
    );
    let _ = instance
        .property_number("chi/chi-num")
        .expect("chi/chi-num");
    assert_eq!(
        instance
            .property("chi/chi-num")
            .expect("cached chi/chi-num")
            .data_type(),
        ViewModelRuntimeDataType::Number
    );

    let properties = instance.properties();
    let enu = properties
        .iter()
        .find(|property| property.name == "enu")
        .expect("enu property data");
    assert_eq!(enu.data_type, ViewModelRuntimeDataType::Enum);
    assert_eq!(enu.enum_name, "Horizontal Align");
    let num = properties
        .iter()
        .find(|property| property.name == "num")
        .expect("num property data");
    assert_eq!(num.data_type, ViewModelRuntimeDataType::Number);
    assert!(num.enum_name.is_empty());
}

fn two_way_fixture(
    artboard_name: &str,
) -> (
    RuntimeArtboardInstanceHandle,
    RuntimeStateMachineInstanceHandle,
    RuntimeViewModelInstanceHandle,
    usize,
) {
    let (artboard, machine, view_model) = fixture("bidirectional_precedence.riv", artboard_name);
    let target = artboard
        .with_artboard(|artboard| artboard.base.data_bind_handles())
        .into_iter()
        .find_map(|bind| {
            let target = bind.with(|bind| bind.as_data_bind()?.target()).flatten()?;
            let definition = nuxie_schema::definition_by_type_key(target.core_type()?)?;
            definition.is_a("Node").then_some(target)
        })
        .expect("two-way Node target");
    let target = artboard.with_artboard(|artboard| artboard.base.object_index(&target)) as usize;
    (artboard, machine, view_model, target)
}

#[test]
fn two_way_source_change_reaches_target_under_target_first_precedence() {
    let (mut artboard, mut machine, mut view_model, target) = two_way_fixture("target_first");
    set_number(&mut view_model, "x", 100.0);
    set_number(&mut view_model, "y", 100.0);
    for seconds in std::iter::once(0.0).chain(std::iter::repeat_n(0.016, 10)) {
        advance(&mut artboard, &mut machine, &mut view_model, seconds);
    }
    assert_eq!(
        view_model.property_number("x").expect("x").value(),
        CoreRegistry::get_double_handle(&object(&artboard, target), i32::from(key("Node", "x")))
            .expect("Node.x")
    );
    assert_eq!(
        view_model.property_number("y").expect("y").value(),
        CoreRegistry::get_double_handle(&object(&artboard, target), i32::from(key("Node", "y")))
            .expect("Node.y")
    );
    set_number(&mut view_model, "x", 500.0);
    set_number(&mut view_model, "y", 600.0);
    for _ in 0..20 {
        advance(&mut artboard, &mut machine, &mut view_model, 0.016);
    }
    assert_eq!(view_model.property_number("x").expect("x").value(), 500.0);
    assert_eq!(view_model.property_number("y").expect("y").value(), 600.0);
    assert_eq!(
        CoreRegistry::get_double_handle(&object(&artboard, target), i32::from(key("Node", "x"))),
        Some(500.0)
    );
    assert_eq!(
        CoreRegistry::get_double_handle(&object(&artboard, target), i32::from(key("Node", "y"))),
        Some(600.0)
    );
}

#[test]
fn two_way_target_change_reaches_source_under_source_first_precedence() {
    let (mut artboard, mut machine, mut view_model, target) = two_way_fixture("source_first");
    set_number(&mut view_model, "x", 100.0);
    set_number(&mut view_model, "y", 100.0);
    advance(&mut artboard, &mut machine, &mut view_model, 0.0);
    assert!(CoreRegistry::set_double_handle(
        &object(&artboard, target),
        i32::from(key("Node", "x")),
        700.0,
    ));
    assert!(CoreRegistry::set_double_handle(
        &object(&artboard, target),
        i32::from(key("Node", "y")),
        800.0,
    ));
    advance(&mut artboard, &mut machine, &mut view_model, 0.016);
    assert_eq!(view_model.property_number("x").expect("x").value(), 700.0);
    assert_eq!(view_model.property_number("y").expect("y").value(), 800.0);
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
        .property_view_model("child1")
        .expect("child1 linked view model");
    child
        .property_string("label")
        .expect("label")
        .set_value("label-update");
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
        .property_view_model("vm_2_child1")
        .expect("vm_2_child1 linked view model");
    child
        .property_string("label")
        .expect("label")
        .set_value("label-update");
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
    assert_eq!(
        nested_texts(&mut artboard),
        [Vec::<u8>::new(), Vec::<u8>::new()]
    );

    let child = view_model
        .property_view_model("vm_2_child1")
        .expect("vm_2_child1 linked view model");
    child
        .property_string("label")
        .expect("label")
        .set_value("label-update");
    advance(&mut artboard, &mut machine, &mut view_model, 0.0);
    assert_eq!(
        nested_texts(&mut artboard),
        [b"label-update".to_vec(), Vec::new()]
    );
}

#[test]
fn triggers_updated_by_events_update_parent_state() {
    let file = import("data_binding_test_triggers.riv");
    let mut artboard = artboard_named(&file, "root");
    let mut view_model = fresh_view_model(&file, &artboard, false);
    let mut machine = artboard
        .default_state_machine_handle()
        .expect("default machine");
    bind_view_model(&artboard, &machine, &view_model);
    let color = descendant_of_type(&artboard, "main_rect", "SolidColor");
    let read = |artboard: &RuntimeArtboardInstanceHandle| {
        CoreRegistry::get_color_handle(
            &object(artboard, color),
            i32::from(key("SolidColor", "colorValue")),
        )
        .map(|color| color as u32)
        .expect("main_rect SolidColor.colorValue")
    };

    advance(&mut artboard, &mut machine, &mut view_model, 0.0);
    assert_eq!(read(&artboard), 0xffff_0000);
    advance(&mut artboard, &mut machine, &mut view_model, 0.7);
    advance(&mut artboard, &mut machine, &mut view_model, 0.1);
    assert_eq!(read(&artboard), 0xff00_ff00);
}

#[test]
fn custom_property_trigger_binding_has_exact_initial_owners() {
    let mut factory = PersistentFactory::new(SerializingFactory::new());
    let factory_handle =
        RuntimeFactoryHandle::from_factory(&mut factory).expect("retained factory");
    let file = File::import(
        &pinned("custom_property_trigger.riv"),
        factory_handle,
        None,
        None,
        None,
    )
    .expect("fixture imports");
    let mut artboard = artboard_named(&file, "Main");
    let mut view_model = file
        .with_file_mut(|file| {
            file.create_default_view_model_instance_for_artboard(artboard.core_handle())
                .or_else(|| file.create_view_model_instance_for_artboard(artboard.core_handle()))
        })
        .map(ViewModelInstanceRuntime::new)
        .map(ViewModelInstanceRuntime::into_handle)
        .expect("artboard view model");
    let mut machine = artboard
        .default_state_machine_handle()
        .expect("default state machine");
    bind_view_model(&artboard, &machine, &view_model);
    let (width, height) = artboard.with_artboard(|artboard| (artboard.width(), artboard.height()));
    factory.borrow_mut().frame_size(width as u32, height as u32);
    advance(&mut artboard, &mut machine, &mut view_model, 0.0);

    let circle_local = local(&artboard, "MainCircle");
    let circle = object(&artboard, circle_local);
    assert!(
        circle
            .core_type()
            .and_then(nuxie_schema::definition_by_type_key)
            .is_some_and(|definition| definition.is_a("Shape"))
    );
    assert_eq!(
        CoreRegistry::get_double_handle(&circle, i32::from(key("Node", "scaleX"))),
        Some(1.0)
    );
    assert_eq!(
        CoreRegistry::get_double_handle(&circle, i32::from(key("Node", "scaleY"))),
        Some(1.0)
    );

    let trigger = object(&artboard, local(&artboard, "Trig"));
    assert!(
        trigger
            .core_type()
            .and_then(nuxie_schema::definition_by_type_key)
            .is_some_and(|definition| definition.is_a("CustomPropertyTrigger"))
    );

    let mut renderer = factory.borrow().make_renderer();
    artboard.draw(&mut renderer);
    for _ in 0..(1.0_f32 / 0.16_f32) as usize {
        factory.borrow_mut().add_frame();
        advance(&mut artboard, &mut machine, &mut view_model, 0.16);
        let mut renderer = factory.borrow().make_renderer();
        artboard.draw(&mut renderer);
    }
    let expected =
        parse_sriv(&pinned_silver("custom_property_trigger_bind.sriv")).expect("pinned C++ silver");
    let actual = parse_sriv(&factory.borrow().bytes()).expect("Rust SRIV");
    compare_sriv(&expected, &actual)
        .unwrap_or_else(|difference| panic!("custom_property_trigger_bind differs: {difference}"));
}

#[test]
fn state_machine_is_led_by_bound_enum_and_trigger() {
    let (mut artboard, mut machine, mut view_model) =
        fixture("data_binding_test.riv", "artboard-2");
    let color = descendant_of_type(&artboard, "color_rectangle", "SolidColor");
    let read_color = |artboard: &RuntimeArtboardInstanceHandle| {
        CoreRegistry::get_color_handle(
            &object(artboard, color),
            i32::from(key("SolidColor", "colorValue")),
        )
        .map(|color| color as u32)
        .expect("color_rectangle SolidColor.colorValue")
    };
    let read_position = |artboard: &RuntimeArtboardInstanceHandle| {
        (
            number(artboard, "color_rectangle", "Node", "x"),
            number(artboard, "color_rectangle", "Node", "y"),
        )
    };

    assert_eq!(read_position(&artboard), (250.0, 250.0));
    assert_eq!(read_color(&artboard), 0xff74_7474);
    advance(&mut artboard, &mut machine, &mut view_model, 0.0);
    assert_eq!(read_color(&artboard), 0xffff_0000);

    assert!(
        view_model
            .property_enum("state")
            .expect("state")
            .set_value_index(1)
    );
    advance(&mut artboard, &mut machine, &mut view_model, 0.0);
    assert_eq!(read_color(&artboard), 0xff00_ff00);
    assert_eq!(read_position(&artboard), (150.0, 250.0));

    assert!(
        view_model
            .property_enum("state")
            .expect("state")
            .set_value("state-blue")
    );
    view_model
        .property_trigger("trigger-prop")
        .expect("trigger-prop")
        .trigger();
    advance(&mut artboard, &mut machine, &mut view_model, 0.0);
    assert_eq!(read_color(&artboard), 0xff00_00ff);
    assert_eq!(read_position(&artboard), (350.0, 250.0));

    view_model
        .property_trigger("trigger-prop")
        .expect("trigger-prop")
        .trigger();
    advance(&mut artboard, &mut machine, &mut view_model, 0.0);
    assert_eq!(read_position(&artboard), (350.0, 350.0));
}

#[test]
fn artboard_has_bound_properties() {
    let file = import("data_binding_test.riv");
    let mut artboard = artboard_named(&file, "artboard-1");
    let view_model = fresh_view_model(&file, &artboard, true);
    artboard.bind_view_model_instance(Some(view_model.instance()));
    artboard.advance_default(0.0);

    let rectangle = local(&artboard, "bound_rect");
    let shape = local(&artboard, "bound_rect_shape");
    let solid = descendant_of_type(&artboard, "bound_rect_shape", "SolidColor");
    let text = local(&artboard, "bound_text_run");
    let follow = local_of_type(&artboard, "FollowPathConstraint");
    assert_eq!(
        CoreRegistry::get_double_handle(
            &object(&artboard, rectangle),
            i32::from(key("Rectangle", "width"))
        ),
        Some(100.0)
    );
    assert_catch_approx(
        CoreRegistry::get_double_handle(
            &object(&artboard, shape),
            i32::from(key("Node", "rotation")),
        )
        .expect("bound_rect_shape rotation"),
        1.5708,
    );
    assert_eq!(
        CoreRegistry::get_color_handle(
            &object(&artboard, solid),
            i32::from(key("SolidColor", "colorValue"))
        )
        .map(|color| color as u32),
        Some(0xffff_0000_u32)
    );
    assert_eq!(
        CoreRegistry::get_string_handle(
            &object(&artboard, text),
            i32::from(key("TextValueRun", "text"))
        ),
        Some("bound text".to_owned())
    );
    assert_live_bool(
        &mut artboard,
        follow,
        "FollowPathConstraint",
        "orient",
        false,
    );

    set_number(&view_model, "width", 200.0);
    set_number(&view_model, "rotation", 180.0);
    view_model
        .property_color("color")
        .expect("color")
        .set_value(0xff00_ff00_u32 as i32);
    set_string(&view_model, "text", "New text");
    view_model
        .property_boolean("orient")
        .expect("orient")
        .set_value(true);
    artboard.advance_default(0.0);
    assert_eq!(
        CoreRegistry::get_double_handle(
            &object(&artboard, rectangle),
            i32::from(key("Rectangle", "width"))
        ),
        Some(200.0)
    );
    assert_catch_approx(
        CoreRegistry::get_double_handle(
            &object(&artboard, shape),
            i32::from(key("Node", "rotation")),
        )
        .expect("bound_rect_shape rotation"),
        3.14159,
    );
    assert_eq!(
        CoreRegistry::get_color_handle(
            &object(&artboard, solid),
            i32::from(key("SolidColor", "colorValue"))
        )
        .map(|color| color as u32),
        Some(0xff00_ff00_u32)
    );
    assert_eq!(
        CoreRegistry::get_string_handle(
            &object(&artboard, text),
            i32::from(key("TextValueRun", "text"))
        ),
        Some("New text".to_owned())
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
            .property_boolean("bool-prop")
            .expect("bool-prop")
            .value(),
        false
    );

    view_model
        .property_boolean("bool-prop")
        .expect("bool-prop")
        .set_value(true);
    advance(&mut artboard, &mut machine, &mut view_model, 0.0);
    assert_live_bool(
        &mut artboard,
        target,
        "CustomPropertyBoolean",
        "propertyValue",
        false,
    );
}
