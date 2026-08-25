//! Exact executable ports of pinned `data_binding_cycle_test.cpp`.

use std::path::PathBuf;

use nuxie::File;
use nuxie_runtime::ArtboardInstance as RawArtboardInstance;

fn pinned_fixture() -> Vec<u8> {
    let root = std::env::var_os("RIVE_RUNTIME_DIR")
        .unwrap_or_else(|| "/Users/levi/dev/oss/rive-runtime".into());
    let path = PathBuf::from(root).join("tests/unit_tests/assets/data_binding_test_3.riv");
    std::fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
}

fn property_key(type_name: &str, property_name: &str) -> u16 {
    let definition = nuxie_schema::definition_by_name(type_name).expect("schema definition");
    std::iter::once(definition.name)
        .chain(definition.ancestors.iter().copied())
        .filter_map(nuxie_schema::definition_by_name)
        .flat_map(|owner| owner.properties)
        .find(|property| property.name == property_name)
        .unwrap_or_else(|| panic!("{type_name}.{property_name}"))
        .key
        .int
}

fn named_local(graph: &nuxie_graph::ArtboardGraph, name: &str) -> usize {
    graph
        .component_named(name)
        .unwrap_or_else(|| panic!("component {name}"))
        .local_id
}

fn number(
    artboard: &RawArtboardInstance,
    graph: &nuxie_graph::ArtboardGraph,
    name: &str,
    owner: &str,
    property: &str,
) -> f32 {
    artboard
        .double_property(named_local(graph, name), property_key(owner, property))
        .unwrap_or_else(|| panic!("number {name}.{property}"))
}

fn text(artboard: &RawArtboardInstance, graph: &nuxie_graph::ArtboardGraph, name: &str) -> Vec<u8> {
    artboard
        .debug_string_property(
            named_local(graph, name),
            property_key("TextValueRun", "text"),
        )
        .unwrap_or_else(|| panic!("text {name}"))
        .to_vec()
}

fn visit_nested<T>(
    file: &File,
    artboard: &mut RawArtboardInstance,
    required_component: &str,
    read: impl FnOnce(&mut RawArtboardInstance, &nuxie_graph::ArtboardGraph) -> T,
) -> T {
    let mut read = Some(read);
    let mut value = None;
    let mut visitor = |_depth: usize, graph_global_id: u32, nested: &mut RawArtboardInstance| {
        let graph = file
            .graph()
            .artboards
            .iter()
            .find(|graph| graph.global_id == graph_global_id)
            .expect("nested graph");
        if value.is_none() && graph.component_named(required_component).is_some() {
            value = Some(read.take().expect("single matching occurrence")(
                nested, graph,
            ));
        }
        Ok(())
    };
    artboard
        .try_visit_nested_artboard_instances_mut::<()>(&mut visitor)
        .expect("nested occurrence visit");
    value.unwrap_or_else(|| panic!("live nested occurrence containing {required_component}"))
}

fn run(
    artboard_name: &str,
    body: impl FnOnce(
        &File,
        &mut nuxie::ArtboardInstance<'_>,
        &mut nuxie::StateMachineInstance,
        &mut nuxie::ViewModelInstance,
    ),
) {
    let file = File::import(&pinned_fixture()).expect("data_binding_test_3 imports");
    let mut artboard = file
        .artboard_named(artboard_name)
        .unwrap_or_else(|| panic!("artboard {artboard_name}"))
        .instantiate()
        .expect("artboard instantiates");
    let mut view_model = artboard
        .instantiate_default_view_model_instance()
        .or_else(|| artboard.instantiate_view_model())
        .expect("default view model");
    let mut machine = artboard
        .default_state_machine_instance()
        .expect("default machine");
    assert!(machine.bind_owned_view_model_handle(view_model.handle()));
    let _ = artboard.bind_view_model(&view_model);
    body(&file, &mut artboard, &mut machine, &mut view_model);
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

fn click(
    artboard: &mut nuxie::ArtboardInstance<'_>,
    machine: &mut nuxie::StateMachineInstance,
    x: f32,
    y: f32,
) {
    machine.pointer_down(artboard.raw_mut(), x, y, 0);
    machine.pointer_up(artboard.raw_mut(), x, y, 0);
}

#[test]
fn child_updates_parent_on_next_frame() {
    run("main-1", |_file, artboard, machine, view_model| {
        advance(artboard, machine, view_model, 0.0);
        assert_eq!(
            number(
                artboard.raw(),
                artboard.artboard().graph(),
                "sized-rect-path",
                "Rectangle",
                "width"
            ),
            100.0
        );
        click(artboard, machine, 75.0, 75.0);
        advance(artboard, machine, view_model, 0.0);
        assert_eq!(
            number(
                artboard.raw(),
                artboard.artboard().graph(),
                "sized-rect-path",
                "Rectangle",
                "width"
            ),
            200.0
        );
    });
}

#[test]
fn parent_updates_child_on_next_frame() {
    run("main-2", |file, artboard, machine, view_model| {
        advance(artboard, machine, view_model, 0.0);
        assert_eq!(
            visit_nested(
                file,
                artboard.raw_mut(),
                "child-rect-path",
                |child, graph| number(child, graph, "child-rect-path", "Rectangle", "width")
            ),
            100.0
        );
        click(artboard, machine, 250.0, 250.0);
        advance(artboard, machine, view_model, 0.0);
        assert_eq!(
            visit_nested(
                file,
                artboard.raw_mut(),
                "child-rect-path",
                |child, graph| number(child, graph, "child-rect-path", "Rectangle", "width")
            ),
            200.0
        );
    });
}

#[test]
fn child_event_updates_parent_on_next_frame() {
    run("main-3", |_file, artboard, machine, view_model| {
        advance(artboard, machine, view_model, 0.0);
        assert_eq!(
            number(
                artboard.raw(),
                artboard.artboard().graph(),
                "sized-rect-path",
                "Rectangle",
                "width"
            ),
            100.0
        );
        advance(artboard, machine, view_model, 0.5);
        assert_eq!(
            number(
                artboard.raw(),
                artboard.artboard().graph(),
                "sized-rect-path",
                "Rectangle",
                "width"
            ),
            100.0
        );
        advance(artboard, machine, view_model, 0.0);
        assert_eq!(
            number(
                artboard.raw(),
                artboard.artboard().graph(),
                "sized-rect-path",
                "Rectangle",
                "width"
            ),
            200.0
        );
    });
}

#[test]
fn parent_event_updates_child_on_next_frame() {
    run("main-4", |file, artboard, machine, view_model| {
        advance(artboard, machine, view_model, 0.0);
        assert_eq!(
            visit_nested(
                file,
                artboard.raw_mut(),
                "child-rect-path",
                |child, graph| number(child, graph, "child-rect-path", "Rectangle", "width")
            ),
            100.0
        );
        advance(artboard, machine, view_model, 0.5);
        assert_eq!(
            visit_nested(
                file,
                artboard.raw_mut(),
                "child-rect-path",
                |child, graph| number(child, graph, "child-rect-path", "Rectangle", "width")
            ),
            100.0
        );
        advance(artboard, machine, view_model, 0.0);
        assert_eq!(
            visit_nested(
                file,
                artboard.raw_mut(),
                "child-rect-path",
                |child, graph| number(child, graph, "child-rect-path", "Rectangle", "width")
            ),
            200.0
        );
    });
}

#[test]
fn child_target_to_source_reaches_parent_same_frame() {
    run("main-5", |_file, artboard, machine, view_model| {
        advance(artboard, machine, view_model, 0.0);
        assert_eq!(
            text(artboard.raw(), artboard.artboard().graph(), "text-run-test"),
            b"before"
        );
        advance(artboard, machine, view_model, 0.5);
        assert_eq!(
            text(artboard.raw(), artboard.artboard().graph(), "text-run-test"),
            b"after"
        );
    });
}

#[test]
fn parent_target_to_source_reaches_child_same_frame() {
    run("main-6", |file, artboard, machine, view_model| {
        advance(artboard, machine, view_model, 0.0);
        assert_eq!(
            visit_nested(
                file,
                artboard.raw_mut(),
                "child-text-run",
                |child, graph| text(child, graph, "child-text-run")
            ),
            b"parent-before"
        );
        advance(artboard, machine, view_model, 0.5);
        assert_eq!(
            visit_nested(
                file,
                artboard.raw_mut(),
                "child-text-run",
                |child, graph| text(child, graph, "child-text-run")
            ),
            b"parent-after"
        );
    });
}

#[test]
#[ignore = "expected-red: second 1.5s advance leaves main-run at main-test-2 instead of child-text-1 across the three-level occurrence"]
fn view_model_changes_propagate_through_three_artboard_levels() {
    run("main-7", |file, artboard, machine, view_model| {
        advance(artboard, machine, view_model, 0.0);
        for (seconds, expected) in [(0.5, &b"main-test-2"[..]), (1.5, &b"child-text-1"[..])] {
            advance(artboard, machine, view_model, seconds);
            assert_eq!(
                text(artboard.raw(), artboard.artboard().graph(), "main-run"),
                expected
            );
            let (child, grandchild) =
                visit_nested(file, artboard.raw_mut(), "child-run", |child, graph| {
                    let child_text = text(child, graph, "child-run");
                    let grandchild_text = visit_nested(
                        file,
                        child,
                        "grand-child-run",
                        |grandchild, grandchild_graph| {
                            text(grandchild, grandchild_graph, "grand-child-run")
                        },
                    );
                    (child_text, grandchild_text)
                });
            assert_eq!(child, expected);
            assert_eq!(grandchild, expected);
        }
    });
}
