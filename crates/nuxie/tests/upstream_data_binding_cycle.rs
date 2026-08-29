//! Exact executable ports of pinned `data_binding_cycle_test.cpp`.

use std::path::PathBuf;

use nuxie::{
    CoreHandle, File, PersistentFactory, RuntimeArtboardInstanceHandle, RuntimeFactoryHandle,
    RuntimeFileHandle, RuntimeStateMachineInstanceHandle, RuntimeViewModelInstanceHandle, Vec2D,
    ViewModelInstanceRuntime,
};
use nuxie_render_api::SerializingFactory;
use nuxie_runtime::source::generated::{
    component_base::ComponentBase, core_registry::CoreRegistry,
};

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

fn object(artboard: &RuntimeArtboardInstanceHandle, name: &str) -> CoreHandle {
    let objects = artboard.with_artboard(|artboard| artboard.base.objects().to_vec());
    objects
        .into_iter()
        .flatten()
        .find(|component| {
            CoreRegistry::get_string_handle(component, i32::from(ComponentBase::NAME_PROPERTY_KEY))
                .as_deref()
                == Some(name)
        })
        .unwrap_or_else(|| panic!("component {name}"))
}

fn number(
    artboard: &RuntimeArtboardInstanceHandle,
    name: &str,
    owner: &str,
    property: &str,
) -> f32 {
    CoreRegistry::get_double_handle(
        &object(artboard, name),
        i32::from(property_key(owner, property)),
    )
    .unwrap_or_else(|| panic!("number {name}.{property}"))
}

fn text(artboard: &RuntimeArtboardInstanceHandle, name: &str) -> Vec<u8> {
    CoreRegistry::get_string_handle(
        &object(artboard, name),
        i32::from(property_key("TextValueRun", "text")),
    )
    .unwrap_or_else(|| panic!("text {name}"))
    .into_bytes()
}

fn nested_with_component(
    artboard: &RuntimeArtboardInstanceHandle,
    required_component: &str,
) -> RuntimeArtboardInstanceHandle {
    let children = artboard.with_artboard(|artboard| artboard.base.nested_artboards());
    for child in children.into_iter().filter_map(|nested| {
        nested
            .with(|nested| nested.nested_artboard_instance_handle())
            .flatten()
    }) {
        if child.with_artboard(|child| {
            child
                .base
                .find_handle::<nuxie_runtime::source::component::Component>(required_component)
                .is_some()
        }) {
            return child;
        }
        let descendants = child.with_artboard(|child| child.base.nested_artboards());
        if !descendants.is_empty() {
            // Only recurse when the child can contain another live occurrence.
            if let Some(found) = find_nested(&child, required_component) {
                return found;
            }
        }
    }
    panic!("live nested occurrence containing {required_component}")
}

fn find_nested(
    artboard: &RuntimeArtboardInstanceHandle,
    required_component: &str,
) -> Option<RuntimeArtboardInstanceHandle> {
    let children = artboard.with_artboard(|artboard| artboard.base.nested_artboards());
    for child in children.into_iter().filter_map(|nested| {
        nested
            .with(|nested| nested.nested_artboard_instance_handle())
            .flatten()
    }) {
        if child.with_artboard(|child| {
            child
                .base
                .find_handle::<nuxie_runtime::source::component::Component>(required_component)
                .is_some()
        }) {
            return Some(child);
        }
        if let Some(found) = find_nested(&child, required_component) {
            return Some(found);
        }
    }
    None
}

fn run(
    artboard_name: &str,
    body: impl FnOnce(
        &RuntimeFileHandle,
        &RuntimeArtboardInstanceHandle,
        &RuntimeStateMachineInstanceHandle,
        &RuntimeViewModelInstanceHandle,
    ),
) {
    let mut silver = PersistentFactory::new(SerializingFactory::new());
    let factory = RuntimeFactoryHandle::from_factory(&mut silver).expect("retained factory");
    let file = File::import(&pinned_fixture(), factory, None, None, None)
        .expect("data_binding_test_3 imports");
    let artboard = file
        .with_file(|file| file.artboard_named(artboard_name))
        .unwrap_or_else(|| panic!("artboard {artboard_name}"));
    let view_model = file
        .with_file_mut(|file| {
            file.create_default_view_model_instance_for_artboard(artboard.core_handle())
                .or_else(|| file.create_view_model_instance_for_artboard(artboard.core_handle()))
        })
        .map(ViewModelInstanceRuntime::new)
        .map(ViewModelInstanceRuntime::into_handle)
        .expect("default view model");
    let machine = artboard
        .default_state_machine_handle()
        .expect("default machine");
    machine.with_instance_mut(|machine| machine.bind_view_model_instance(view_model.instance()));
    artboard.bind_view_model_instance(Some(view_model.instance()));
    body(&file, &artboard, &machine, &view_model);
}

fn advance(
    _artboard: &RuntimeArtboardInstanceHandle,
    machine: &RuntimeStateMachineInstanceHandle,
    _view_model: &RuntimeViewModelInstanceHandle,
    seconds: f32,
) {
    machine.advance_and_apply(seconds);
}

fn click(
    _artboard: &RuntimeArtboardInstanceHandle,
    machine: &RuntimeStateMachineInstanceHandle,
    x: f32,
    y: f32,
) {
    machine.with_instance_mut(|machine| {
        machine.pointer_down(Vec2D::new(x, y), 0);
        machine.pointer_up(Vec2D::new(x, y), 0);
    });
}

#[test]
fn child_updates_parent_on_next_frame() {
    run("main-1", |_file, artboard, machine, view_model| {
        advance(artboard, machine, view_model, 0.0);
        assert_eq!(
            number(artboard, "sized-rect-path", "Rectangle", "width"),
            100.0
        );
        click(artboard, machine, 75.0, 75.0);
        advance(artboard, machine, view_model, 0.0);
        assert_eq!(
            number(artboard, "sized-rect-path", "Rectangle", "width"),
            200.0
        );
    });
}

#[test]
fn parent_updates_child_on_next_frame() {
    run("main-2", |_file, artboard, machine, view_model| {
        advance(artboard, machine, view_model, 0.0);
        assert_eq!(
            number(
                &nested_with_component(artboard, "child-rect-path"),
                "child-rect-path",
                "Rectangle",
                "width",
            ),
            100.0
        );
        click(artboard, machine, 250.0, 250.0);
        advance(artboard, machine, view_model, 0.0);
        assert_eq!(
            number(
                &nested_with_component(artboard, "child-rect-path"),
                "child-rect-path",
                "Rectangle",
                "width",
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
            number(artboard, "sized-rect-path", "Rectangle", "width"),
            100.0
        );
        advance(artboard, machine, view_model, 0.5);
        assert_eq!(
            number(artboard, "sized-rect-path", "Rectangle", "width"),
            100.0
        );
        advance(artboard, machine, view_model, 0.0);
        assert_eq!(
            number(artboard, "sized-rect-path", "Rectangle", "width"),
            200.0
        );
    });
}

#[test]
fn parent_event_updates_child_on_next_frame() {
    run("main-4", |_file, artboard, machine, view_model| {
        advance(artboard, machine, view_model, 0.0);
        assert_eq!(
            number(
                &nested_with_component(artboard, "child-rect-path"),
                "child-rect-path",
                "Rectangle",
                "width"
            ),
            100.0
        );
        advance(artboard, machine, view_model, 0.5);
        assert_eq!(
            number(
                &nested_with_component(artboard, "child-rect-path"),
                "child-rect-path",
                "Rectangle",
                "width"
            ),
            100.0
        );
        advance(artboard, machine, view_model, 0.0);
        assert_eq!(
            number(
                &nested_with_component(artboard, "child-rect-path"),
                "child-rect-path",
                "Rectangle",
                "width"
            ),
            200.0
        );
    });
}

#[test]
fn child_target_to_source_reaches_parent_same_frame() {
    run("main-5", |_file, artboard, machine, view_model| {
        advance(artboard, machine, view_model, 0.0);
        assert_eq!(text(artboard, "text-run-test"), b"before");
        advance(artboard, machine, view_model, 0.5);
        assert_eq!(text(artboard, "text-run-test"), b"after");
    });
}

#[test]
fn parent_target_to_source_reaches_child_same_frame() {
    run("main-6", |_file, artboard, machine, view_model| {
        advance(artboard, machine, view_model, 0.0);
        assert_eq!(
            text(
                &nested_with_component(artboard, "child-text-run"),
                "child-text-run"
            ),
            b"parent-before"
        );
        advance(artboard, machine, view_model, 0.5);
        assert_eq!(
            text(
                &nested_with_component(artboard, "child-text-run"),
                "child-text-run"
            ),
            b"parent-after"
        );
    });
}

#[test]
fn view_model_changes_propagate_through_three_artboard_levels() {
    run("main-7", |_file, artboard, machine, view_model| {
        advance(artboard, machine, view_model, 0.0);
        for (seconds, expected) in [(0.5, &b"main-test-2"[..]), (1.5, &b"child-text-1"[..])] {
            advance(artboard, machine, view_model, seconds);
            assert_eq!(text(artboard, "main-run"), expected);
            let child_artboard = nested_with_component(artboard, "child-run");
            let child = text(&child_artboard, "child-run");
            let grandchild = text(
                &nested_with_component(&child_artboard, "grand-child-run"),
                "grand-child-run",
            );
            assert_eq!(child, expected);
            assert_eq!(grandchild, expected);
        }
    });
}
