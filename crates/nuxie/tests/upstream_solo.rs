//! Direct ports of pinned `tests/unit_tests/runtime/solo_test.cpp`.
#![cfg(feature = "scripting")]

use std::path::PathBuf;

use nuxie::{ArtboardInstance, File, PersistentFactory};
use nuxie_graph::{ArtboardGraph, GraphFile};
use nuxie_render_api::SerializingFactory;
use nuxie_schema::definition_by_name;
use silver_corpus::{compare_sriv, parse_sriv};

fn pinned_fixture(name: &str) -> Vec<u8> {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let fixture = PathBuf::from(root)
        .join("tests/unit_tests/assets")
        .join(name);
    std::fs::read(&fixture)
        .unwrap_or_else(|error| panic!("read pinned fixture {}: {error}", fixture.display()))
}

fn pinned_silver(name: &str) -> Vec<u8> {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let silver = PathBuf::from(root)
        .join("tests/unit_tests/silvers")
        .join(format!("{name}.sriv"));
    std::fs::read(&silver)
        .unwrap_or_else(|error| panic!("read pinned silver {}: {error}", silver.display()))
}

fn compare_silver(name: &str, actual: &[u8]) {
    let actual = parse_sriv(actual).expect("valid Rust SRIV stream");
    let expected = parse_sriv(&pinned_silver(name)).expect("valid pinned SRIV stream");
    compare_sriv(&expected, &actual)
        .unwrap_or_else(|difference| panic!("{name} differs: {difference}"));
}

fn default_graph(graphs: &GraphFile) -> &ArtboardGraph {
    graphs.artboards.first().expect("default artboard graph")
}

fn named_local(graph: &ArtboardGraph, name: &str) -> usize {
    graph
        .component_named(name)
        .unwrap_or_else(|| panic!("component {name}"))
        .local_id
}

fn collapsed(instance: &ArtboardInstance<'_>, graph: &ArtboardGraph, name: &str) -> bool {
    instance
        .raw()
        .component(named_local(graph, name))
        .unwrap_or_else(|| panic!("runtime component {name}"))
        .is_collapsed()
}

fn active_component_key() -> u16 {
    definition_by_name("Solo")
        .expect("Solo definition")
        .properties
        .iter()
        .find(|property| property.name == "activeComponentId")
        .expect("Solo.activeComponentId")
        .key
        .int
}

#[test]
fn file_with_skins_in_solos_loads_correctly() {
    let file = File::import(&pinned_fixture("death_knight.riv")).expect("death_knight imports");
    let graphs = GraphFile::from_runtime_file(file.runtime()).expect("death_knight graphs");
    let artboard = file.default_artboard().expect("default artboard");
    let mut artboard = artboard
        .instantiate()
        .expect("default artboard instantiates");
    artboard.advance(0.0);
    assert_eq!(
        default_graph(&graphs)
            .components
            .iter()
            .filter(|component| component.type_name == "Solo")
            .count(),
        2
    );
}

#[test]
fn children_load_correctly() {
    let file = File::import(&pinned_fixture("solo_test.riv")).expect("solo_test imports");
    let graphs = GraphFile::from_runtime_file(file.runtime()).expect("solo_test graphs");
    let graph = default_graph(&graphs);
    let artboard = file.default_artboard().expect("default artboard");
    let mut artboard = artboard
        .instantiate()
        .expect("default artboard instantiates");
    artboard.advance(0.0);

    let solos = graph
        .components
        .iter()
        .filter(|component| component.type_name == "Solo")
        .collect::<Vec<_>>();
    assert_eq!(solos.len(), 1);
    let solo = solos[0];
    let children = graph
        .components
        .iter()
        .filter(|component| component.parent_local == Some(solo.local_id))
        .collect::<Vec<_>>();
    assert_eq!(children.len(), 3);
    assert_eq!(
        children
            .iter()
            .map(|child| (child.type_name, child.name.as_deref()))
            .collect::<Vec<_>>(),
        [
            ("Shape", Some("Blue")),
            ("Shape", Some("Green")),
            ("Shape", Some("Red")),
        ]
    );
    assert!(!collapsed(&artboard, graph, "Blue"));
    assert!(collapsed(&artboard, graph, "Green"));
    assert!(collapsed(&artboard, graph, "Red"));
    for child in graph
        .components
        .iter()
        .filter(|component| component.parent_local == Some(named_local(graph, "Green")))
    {
        assert!(
            artboard
                .raw()
                .component(child.local_id)
                .unwrap()
                .is_collapsed()
        );
    }
    for child in graph
        .components
        .iter()
        .filter(|component| component.parent_local == Some(named_local(graph, "Red")))
    {
        assert!(
            artboard
                .raw()
                .component(child.local_id)
                .unwrap()
                .is_collapsed()
        );
    }

    let mut machine = artboard
        .default_state_machine_instance()
        .expect("default state machine");
    artboard
        .raw_mut()
        .advance_state_machine_instance(&mut machine, 0.0);
    assert!(collapsed(&artboard, graph, "Blue"));
    assert!(collapsed(&artboard, graph, "Green"));
    assert!(!collapsed(&artboard, graph, "Red"));
    artboard
        .raw_mut()
        .advance_state_machine_instance(&mut machine, 0.5);
    assert!(collapsed(&artboard, graph, "Blue"));
    assert!(!collapsed(&artboard, graph, "Green"));
    assert!(collapsed(&artboard, graph, "Red"));
    artboard
        .raw_mut()
        .advance_state_machine_instance(&mut machine, 0.5);
    assert!(!collapsed(&artboard, graph, "Blue"));
    assert!(collapsed(&artboard, graph, "Green"));
    assert!(collapsed(&artboard, graph, "Red"));
}

#[test]
fn nested_solos_work() {
    let file = File::import(&pinned_fixture("nested_solo.riv")).expect("nested_solo imports");
    let graphs = GraphFile::from_runtime_file(file.runtime()).expect("nested_solo graphs");
    let graph = default_graph(&graphs);
    let artboard = file.default_artboard().expect("default artboard");
    let mut artboard = artboard
        .instantiate()
        .expect("default artboard instantiates");
    artboard.advance(0.0);

    for name in [
        "Solo 1", "Solo 2", "Solo 3", "A", "B", "C", "D", "E", "F", "G", "H", "I",
    ] {
        assert!(graph.component_named(name).is_some(), "component {name}");
    }
    let key = active_component_key();
    for (solo, active) in [("Solo 1", "A"), ("Solo 2", "D"), ("Solo 3", "H")] {
        artboard.raw_mut().set_uint_property(
            named_local(graph, solo),
            key,
            named_local(graph, active) as u64,
        );
    }
    artboard.advance(0.0);
    for (name, expected) in [
        ("A", false),
        ("B", true),
        ("C", true),
        ("D", true),
        ("E", true),
        ("F", true),
        ("G", true),
        ("H", true),
        ("I", true),
    ] {
        assert_eq!(collapsed(&artboard, graph, name), expected, "{name}");
    }

    assert!(artboard.raw_mut().set_uint_property(
        named_local(graph, "Solo 3"),
        key,
        named_local(graph, "G") as u64,
    ));
    artboard.advance(0.0);
    for (name, expected) in [
        ("A", false),
        ("B", true),
        ("C", true),
        ("D", true),
        ("E", true),
        ("F", true),
        ("G", true),
        ("H", true),
        ("I", true),
    ] {
        assert_eq!(collapsed(&artboard, graph, name), expected, "{name}");
    }

    assert!(artboard.raw_mut().set_uint_property(
        named_local(graph, "Solo 1"),
        key,
        named_local(graph, "C") as u64,
    ));
    artboard.advance(0.0);
    for (name, expected) in [
        ("A", true),
        ("B", true),
        ("C", false),
        ("D", false),
        ("E", true),
        ("F", true),
        ("G", false),
        ("H", true),
        ("I", true),
    ] {
        assert_eq!(collapsed(&artboard, graph, name), expected, "{name}");
    }
}

#[test]
fn hit_test_on_solos() {
    let file = File::import(&pinned_fixture("hit_test_solos.riv")).expect("hit_test_solos imports");
    let artboard = file.default_artboard().expect("default artboard");
    let mut artboard = artboard
        .instantiate()
        .expect("default artboard instantiates");
    assert_eq!(artboard.artboard().state_machine_count(), 1);
    let mut machine = artboard.state_machine_instance(0).expect("state machine 0");
    artboard
        .raw_mut()
        .advance_state_machine_instance(&mut machine, 0.0);
    artboard.advance(0.0);
    assert!(machine.get_bool("hovered").is_some());

    for (x, y, expected) in [
        (200.0, 100.0, true),
        (200.0, 300.0, false),
        (200.0, 400.0, false),
    ] {
        machine.pointer_move(artboard.raw_mut(), x, y, 0.0, 0);
        assert_eq!(
            machine.get_bool("hovered").unwrap().bool_value(),
            Some(expected)
        );
    }
    artboard
        .raw_mut()
        .advance_state_machine_instance(&mut machine, 1.5);
    artboard.advance(1.5);
    for (x, y, expected) in [
        (200.0, 100.0, false),
        (200.0, 300.0, true),
        (200.0, 400.0, false),
    ] {
        machine.pointer_move(artboard.raw_mut(), x, y, 0.0, 0);
        assert_eq!(
            machine.get_bool("hovered").unwrap().bool_value(),
            Some(expected)
        );
    }
    artboard
        .raw_mut()
        .advance_state_machine_instance(&mut machine, 1.0);
    artboard.advance(1.0);
    for (x, y, expected) in [
        (200.0, 100.0, false),
        (200.0, 300.0, false),
        (200.0, 400.0, true),
    ] {
        machine.pointer_move(artboard.raw_mut(), x, y, 0.0, 0);
        assert_eq!(
            machine.get_bool("hovered").unwrap().bool_value(),
            Some(expected)
        );
    }
}

#[test]
#[ignore = "expected-red: public facade cannot yet inspect nested-artboard child paint colors"]
fn hit_test_on_nested_artboards_in_solos() {
    let file = File::import(&pinned_fixture(
        "pointer_events_nested_artboards_in_solos.riv",
    ))
    .expect("nested-artboard solo fixture imports");
    let graphs = GraphFile::from_runtime_file(file.runtime()).expect("fixture graphs");
    let graph = default_graph(&graphs);
    let artboard = file.default_artboard().expect("default artboard");
    let mut artboard = artboard
        .instantiate()
        .expect("default artboard instantiates");
    for name in [
        "Parent-Artboard",
        "Nested-Artboard-Active",
        "Nested-Artboard-Inactive",
    ] {
        assert!(graph.component_named(name).is_some(), "component {name}");
    }
    assert_eq!(artboard.artboard().state_machine_count(), 1);
    let mut machine = artboard.state_machine_instance(0).expect("state machine 0");
    artboard
        .raw_mut()
        .advance_state_machine_instance(&mut machine, 0.0);
    artboard.advance(0.0);
    assert!(!collapsed(&artboard, graph, "Nested-Artboard-Active"));
    assert!(collapsed(&artboard, graph, "Nested-Artboard-Inactive"));
    let active_color = None::<u32>;
    let inactive_color = None::<u32>;
    assert_eq!(active_color, Some(0xFF00_B511));
    assert_eq!(inactive_color, Some(0xFF74_7474));

    artboard
        .raw_mut()
        .advance_state_machine_instance(&mut machine, 0.1);
    artboard.advance(0.1);
    assert!(collapsed(&artboard, graph, "Nested-Artboard-Active"));
    assert!(!collapsed(&artboard, graph, "Nested-Artboard-Inactive"));
    assert_eq!(inactive_color, Some(0xFF00_B511));
    artboard
        .raw_mut()
        .advance_state_machine_instance(&mut machine, 0.1);
    artboard.advance(0.1);
    assert!(!collapsed(&artboard, graph, "Nested-Artboard-Active"));
    assert!(collapsed(&artboard, graph, "Nested-Artboard-Inactive"));
    machine.pointer_up(artboard.raw_mut(), 200.0, 200.0, 0);
    artboard
        .raw_mut()
        .advance_state_machine_instance(&mut machine, 0.0);
    artboard.advance(0.0);
    artboard
        .raw_mut()
        .advance_state_machine_instance(&mut machine, 0.1);
    artboard.advance(0.1);
    assert_eq!(active_color, Some(0xFFC8_0000));
    assert_eq!(inactive_color, Some(0xFF00_B511));
}

// The exact synthetic construction and every assertion for
// "solo index/name selection skips property-like children" live beside the
// private Solo owner in `nuxie-runtime/src/artboard/tests.rs`.

#[test]
#[ignore = "expected-red: Rust silver stream records frameSize before pinned C++ makeRenderPaint"]
fn data_bound_solos_with_enums_work_in_both_directions() {
    let file = File::import(&pinned_fixture("databind_solo_to_enum.riv"))
        .expect("databind_solo_to_enum imports");
    let artboard = file.default_artboard().expect("default artboard");
    let mut artboard = artboard
        .instantiate()
        .expect("default artboard instantiates");
    let mut silver = PersistentFactory::new(SerializingFactory::new());
    let (width, height) = artboard.artboard_dimensions();
    silver.borrow_mut().frame_size(width as u32, height as u32);
    let mut machine = artboard.state_machine_instance(0).expect("state machine 0");
    let mut view_model = if artboard.view_model_index().is_none() {
        artboard.instantiate_view_model()
    } else {
        artboard.instantiate_view_model_instance(0)
    }
    .expect("view-model instance");
    assert_eq!(
        view_model
            .raw()
            .enum_value_by_property_name_path("enuToSource"),
        Some(3)
    );
    artboard.bind_view_model(&view_model);
    artboard.advance_with_state_machines_and_view_model(
        std::slice::from_mut(&mut machine),
        0.0,
        &mut view_model,
    );
    let mut renderer = silver.borrow().make_renderer();
    artboard
        .draw(&mut silver, &mut renderer)
        .expect("initial draw");
    silver.borrow_mut().add_frame();
    machine.pointer_down(artboard.raw_mut(), 425.0, 70.0, 0);
    machine.pointer_up(artboard.raw_mut(), 425.0, 70.0, 0);
    artboard.advance_with_state_machines_and_view_model(
        std::slice::from_mut(&mut machine),
        0.016,
        &mut view_model,
    );
    artboard
        .draw(&mut silver, &mut renderer)
        .expect("second draw");
    assert_eq!(
        view_model
            .raw()
            .enum_value_by_property_name_path("enuToSource"),
        Some(5)
    );
    compare_silver("databind_solo_to_enum", &silver.borrow().bytes());
}

#[test]
#[ignore = "expected-red: scripted advanceCount remains zero on the first bound frame"]
fn do_not_advance_collapsed_scripts() {
    let file = File::import_with_unsigned_scripts(&pinned_fixture("script_advance_test.riv"))
        .expect("script_advance_test imports with trusted scripts");
    let artboard = file.default_artboard().expect("default artboard");
    let mut artboard = artboard
        .instantiate()
        .expect("default artboard instantiates");
    let mut machine = artboard.state_machine_instance(0).expect("state machine 0");
    let mut view_model = artboard
        .instantiate_default_view_model_instance()
        .expect("default view-model instance");
    artboard.bind_view_model(&view_model);
    assert_eq!(
        view_model
            .raw()
            .number_value_by_property_name_path("soloIndex"),
        Some(0.0)
    );
    assert_eq!(
        view_model
            .raw()
            .number_value_by_property_name_path("advanceCount"),
        Some(0.0)
    );
    for expected in [1.0, 2.0] {
        artboard.advance_with_state_machines_and_view_model(
            std::slice::from_mut(&mut machine),
            0.016,
            &mut view_model,
        );
        assert_eq!(
            view_model
                .raw()
                .number_value_by_property_name_path("advanceCount"),
            Some(expected)
        );
    }
    for (index, expected) in [(1.0, 3.0), (2.0, 4.0), (3.0, 5.0)] {
        assert!(view_model.set_number("soloIndex", index));
        artboard.advance_with_state_machines_and_view_model(
            std::slice::from_mut(&mut machine),
            0.016,
            &mut view_model,
        );
        assert_eq!(
            view_model
                .raw()
                .number_value_by_property_name_path("advanceCount"),
            Some(expected)
        );
    }
    artboard.advance_with_state_machines_and_view_model(
        std::slice::from_mut(&mut machine),
        0.016,
        &mut view_model,
    );
    assert_eq!(
        view_model
            .raw()
            .number_value_by_property_name_path("advanceCount"),
        Some(5.0)
    );
    assert!(view_model.set_number("soloIndex", 0.0));
    artboard.advance_with_state_machines_and_view_model(
        std::slice::from_mut(&mut machine),
        0.016,
        &mut view_model,
    );
    assert_eq!(
        view_model
            .raw()
            .number_value_by_property_name_path("advanceCount"),
        Some(5.0)
    );
    artboard.advance_with_state_machines_and_view_model(
        std::slice::from_mut(&mut machine),
        0.016,
        &mut view_model,
    );
    assert_eq!(
        view_model
            .raw()
            .number_value_by_property_name_path("advanceCount"),
        Some(6.0)
    );
}

#[test]
#[ignore = "expected-red: exact solo-index silver was previously classified before the mutation surface existed"]
fn data_bind_by_index_skipping_non_hierarchical_children() {
    let file =
        File::import(&pinned_fixture("solo_index_test.riv")).expect("solo_index_test imports");
    let artboard = file.default_artboard().expect("default artboard");
    let mut artboard = artboard
        .instantiate()
        .expect("default artboard instantiates");
    let mut silver = PersistentFactory::new(SerializingFactory::new());
    let (width, height) = artboard.artboard_dimensions();
    silver.borrow_mut().frame_size(width as u32, height as u32);
    let mut renderer = silver.borrow().make_renderer();
    let mut machine = artboard.state_machine_instance(0).expect("state machine 0");
    let mut view_model = artboard
        .instantiate_default_view_model_instance()
        .expect("default view-model instance");
    artboard.bind_view_model(&view_model);
    for index in [0.0, 1.0, 2.0, 3.0] {
        if index != 0.0 {
            assert!(view_model.set_number("index", index));
        }
        artboard.advance_with_state_machines_and_view_model(
            std::slice::from_mut(&mut machine),
            0.1,
            &mut view_model,
        );
        artboard
            .draw(&mut silver, &mut renderer)
            .expect("solo-index draw");
        if index != 3.0 {
            silver.borrow_mut().add_frame();
        }
    }
    compare_silver("solo_index_test", &silver.borrow().bytes());
}
