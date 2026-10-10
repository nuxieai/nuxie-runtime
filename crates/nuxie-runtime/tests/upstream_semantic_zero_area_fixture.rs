//! All four cases from upstream 293eaf00 semantic_zero_area_fixture_test.cpp.
use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::{
    animation::semantic_listener_group::SemanticActionType,
    semantic::semantic_snapshot::{Bounds, SemanticsDiff},
    viewmodel::{
        viewmodel_instance::ViewModelInstance, viewmodel_instance_string::ViewModelInstanceString,
    },
};
use nuxie_runtime::{
    CoreHandle, File, RuntimeArtboardInstanceHandle, RuntimeFactoryHandle, RuntimeFileHandle,
    RuntimeStateMachineInstanceHandle,
};
use std::{collections::HashMap, path::PathBuf};

struct Node {
    label: String,
    role: u32,
    bounds: Bounds,
}
struct Fixture {
    _file: RuntimeFileHandle,
    _artboard: RuntimeArtboardInstanceHandle,
    sm: RuntimeStateMachineInstanceHandle,
    vmi: CoreHandle,
    nodes: HashMap<u32, Node>,
}
impl Fixture {
    fn apply(&mut self, diff: SemanticsDiff) {
        for id in diff.removed {
            self.nodes.remove(&id);
        }
        for n in diff.added {
            let bounds = n.bounds();
            self.nodes.insert(
                n.id,
                Node {
                    label: n.label,
                    role: n.role,
                    bounds,
                },
            );
        }
        for n in diff.updated_semantic {
            if let Some(node) = self.nodes.get_mut(&n.id) {
                node.label = n.label;
                node.role = n.role;
            }
        }
        for b in diff.updated_geometry {
            if let Some(node) = self.nodes.get_mut(&b.id) {
                node.bounds = b.bounds();
            }
        }
        for n in diff.moved {
            if let Some(node) = self.nodes.get_mut(&n.id) {
                node.bounds = n.bounds();
            }
        }
    }
    fn drain(&mut self) {
        let manager = self.sm.with_instance(|sm| sm.semantic_manager());
        if let Some(manager) = manager {
            self.apply(manager.with_semantic_manager_mut(|manager| manager.drain_diff()));
        }
    }
    fn by_label(&self, label: &str) -> Option<&Node> {
        self.nodes.values().find(|node| node.label == label)
    }
    fn id_by_label(&self, label: &str) -> u32 {
        self.nodes
            .iter()
            .find(|(_, node)| node.label == label)
            .map_or(0, |(id, _)| *id)
    }
    fn last_pressed(&self) -> String {
        self.vmi
            .with_downcast::<ViewModelInstance, _>(|vmi| vmi.property_value_named("lastPressed"))
            .flatten()
            .and_then(|value| {
                value.with_downcast::<ViewModelInstanceString, _>(|value| value.value())
            })
            .unwrap_or_default()
    }
    fn advance(&self, frames: usize, dt: f32) {
        for _ in 0..frames {
            self.sm.advance_and_apply(dt);
        }
    }
}
fn make_fixture() -> Fixture {
    let path = std::env::var_os("RIVE_RUNTIME_DIR").map_or_else(
        || {
            PathBuf::from(
                option_env!("BAZEL_CARGO_MANIFEST_DIR").unwrap_or(env!("CARGO_MANIFEST_DIR")),
            )
            .join("../../fixtures/semantic/zero_area_semantics.riv")
        },
        |root| PathBuf::from(root).join("tests/unit_tests/assets/semantic/zero_area_semantics.riv"),
    );
    let bytes =
        std::fs::read(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()));
    let mut factory = PersistentFactory::new(RecordingFactory::default());
    let file = File::import(
        &bytes,
        RuntimeFactoryHandle::from_factory(&mut factory).unwrap(),
        None,
        None,
        None,
    )
    .expect("zero_area_semantics imports");
    let artboard = file
        .with_file(|file| file.artboard_default())
        .expect("default artboard");
    let sm = artboard
        .state_machine_instance_handle(0)
        .expect("state machine zero");
    sm.with_instance_mut(|sm| sm.enable_semantics());
    let vmi = file
        .with_file(|file| {
            file.create_default_view_model_instance_for_artboard(artboard.core_handle())
        })
        .expect("default view model");
    artboard.bind_view_model_instance(Some(vmi.clone()));
    sm.with_instance_mut(|sm| sm.bind_view_model_instance(vmi.clone()));
    sm.advance_and_apply(0.001);
    let mut fixture = Fixture {
        _file: file,
        _artboard: artboard,
        sm,
        vmi,
        nodes: HashMap::new(),
    };
    fixture.drain();
    fixture
}

#[test]
fn tree_shape_labels_and_hidden_pruning() {
    let f = make_fixture();
    for label in [
        "Invisible group",
        "Button A",
        "Button B",
        "Collapsing card",
        "Inside collapsing",
        "Scaling group",
        "Scaled button",
        "Empty root",
        "Root child button",
    ] {
        assert!(f.by_label(label).is_some(), "{label}");
    }
    assert_eq!(f.nodes.len(), 9);
    assert!(f.by_label("Hidden group").is_none());
    assert!(f.by_label("Hidden button").is_none());
}

#[test]
fn empty_groups_get_container_bounds_from_their_children() {
    let f = make_fixture();
    let group = &f.by_label("Invisible group").expect("group").bounds;
    let a = &f.by_label("Button A").expect("A").bounds;
    let b = &f.by_label("Button B").expect("B").bounds;
    assert_eq!(group.min_x, a.min_x.min(b.min_x));
    assert_eq!(group.min_y, a.min_y.min(b.min_y));
    assert_eq!(group.max_x, a.max_x.max(b.max_x));
    assert_eq!(group.max_y, a.max_y.max(b.max_y));
    assert!(group.max_x > group.min_x);
    assert!(group.max_y > group.min_y);
    let root = &f.by_label("Empty root").expect("root").bounds;
    let child = &f.by_label("Root child button").expect("child").bounds;
    assert_eq!(root.min_x, child.min_x);
    assert_eq!(root.max_x, child.max_x);
}

#[test]
fn semantic_tap_drives_listeners_into_the_view_model() {
    let f = make_fixture();
    assert_eq!(f.last_pressed(), "none");
    for (label, expected) in [
        ("Button A", "A"),
        ("Root child button", "root-child"),
        ("Inside collapsing", "inside"),
    ] {
        f.sm.fire_semantic_action(f.id_by_label(label), SemanticActionType::Tap as u8);
        f.advance(2, 0.001);
        assert_eq!(f.last_pressed(), expected);
    }
}

#[test]
fn node_scale_animation_collapses_bounds_through_geometry_diffs() {
    let mut f = make_fixture();
    let scaled = &f.by_label("Scaled button").expect("scaled").bounds;
    let initial_width = scaled.max_x - scaled.min_x;
    assert!(initial_width > 100.0);
    f.advance(150, 0.01);
    f.drain();
    let at_zero = &f.by_label("Scaled button").expect("at zero").bounds;
    assert!(at_zero.max_x - at_zero.min_x < 2.0);
    f.advance(150, 0.01);
    f.drain();
    let restored = &f.by_label("Scaled button").expect("restored").bounds;
    assert!(restored.max_x - restored.min_x > initial_width - 2.0);
}
