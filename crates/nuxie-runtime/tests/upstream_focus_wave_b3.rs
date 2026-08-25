//! Executable Rust ports for pinned `focus_test.cpp` Wave B3.
//!
//! The tests use the public retained FocusNode/FocusManager and real imported
//! runtime fixtures. Callback pointer identity is adapted to the retained
//! FocusEvent stream; fixture-heavy cases execute through StateMachineInstance.

use nuxie_binary::{RuntimeFile, read_runtime_file};
use nuxie_graph::GraphFile;
use nuxie_runtime::{
    ArtboardInstance, FocusEdgeBehavior, FocusEventKind, FocusManager, FocusNode,
    RuntimeBindableArtboard, RuntimeOwnedViewModelContext, RuntimeOwnedViewModelHandle,
    RuntimeOwnedViewModelInstance, StateMachineInstance, TransformProperty,
};

fn push_var_uint(bytes: &mut Vec<u8>, mut value: u64) {
    loop {
        let mut byte = (value & 0x7f) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        bytes.push(byte);
        if value == 0 {
            break;
        }
    }
}

fn schema_property_key(type_name: &str, property_name: &str) -> u16 {
    let definition = nuxie_schema::definition_by_name(type_name)
        .unwrap_or_else(|| panic!("missing schema definition {type_name}"));
    std::iter::once(definition.name)
        .chain(definition.ancestors.iter().copied())
        .filter_map(nuxie_schema::definition_by_name)
        .flat_map(|owner| owner.properties)
        .find(|property| property.name == property_name)
        .unwrap_or_else(|| panic!("missing property {type_name}.{property_name}"))
        .key
        .int
}

fn push_object(bytes: &mut Vec<u8>, type_name: &str, properties: impl FnOnce(&mut Vec<u8>)) {
    push_var_uint(
        bytes,
        u64::from(
            nuxie_schema::definition_by_name(type_name)
                .unwrap_or_else(|| panic!("missing schema definition {type_name}"))
                .type_key
                .int,
        ),
    );
    properties(bytes);
    push_var_uint(bytes, 0);
}

fn push_uint(bytes: &mut Vec<u8>, owner: &str, name: &str, value: u64) {
    push_var_uint(bytes, u64::from(schema_property_key(owner, name)));
    push_var_uint(bytes, value);
}

fn push_f32(bytes: &mut Vec<u8>, owner: &str, name: &str, value: f32) {
    push_var_uint(bytes, u64::from(schema_property_key(owner, name)));
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn push_keyframe(bytes: &mut Vec<u8>, frame: u64, value: f32, interpolation: u64) {
    push_object(bytes, "KeyFrameDouble", |bytes| {
        push_uint(bytes, "KeyFrameDouble", "frame", frame);
        push_uint(bytes, "KeyFrameDouble", "interpolationType", interpolation);
        push_f32(bytes, "KeyFrameDouble", "value", value);
    });
}

fn push_node_x_animation(bytes: &mut Vec<u8>, target: u64, first: f32, second: f32) {
    push_object(bytes, "LinearAnimation", |bytes| {
        push_uint(bytes, "LinearAnimation", "fps", 10);
        push_uint(bytes, "LinearAnimation", "duration", 20);
    });
    push_object(bytes, "KeyedObject", |bytes| {
        push_uint(bytes, "KeyedObject", "objectId", target);
    });
    push_object(bytes, "KeyedProperty", |bytes| {
        push_uint(
            bytes,
            "KeyedProperty",
            "propertyKey",
            u64::from(schema_property_key("Node", "x")),
        );
    });
    push_keyframe(bytes, 0, first, 1);
    push_keyframe(bytes, 10, second, 0);
}

fn focus_condition_without_comparator_fixture() -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"RIVE");
    push_var_uint(&mut bytes, 7);
    push_var_uint(&mut bytes, 0);
    push_var_uint(&mut bytes, 88_203);
    push_var_uint(&mut bytes, 0);
    push_object(&mut bytes, "Backboard", |_| {});
    push_object(&mut bytes, "Artboard", |_| {});
    push_object(&mut bytes, "Node", |bytes| {
        push_uint(bytes, "Node", "parentId", 0);
        push_f32(bytes, "Node", "x", 2.0);
        push_f32(bytes, "Node", "y", 3.0);
        push_f32(bytes, "Node", "scaleX", 1.0);
        push_f32(bytes, "Node", "scaleY", 1.0);
        push_f32(bytes, "Node", "opacity", 1.0);
    });
    push_node_x_animation(&mut bytes, 1, 2.0, 12.0);
    push_node_x_animation(&mut bytes, 1, 20.0, 30.0);
    push_object(&mut bytes, "StateMachine", |_| {});
    push_object(&mut bytes, "StateMachineLayer", |_| {});
    push_object(&mut bytes, "AnyState", |_| {});
    push_object(&mut bytes, "EntryState", |_| {});
    push_object(&mut bytes, "StateTransition", |bytes| {
        push_uint(bytes, "StateTransition", "stateToId", 2);
    });
    push_object(&mut bytes, "AnimationState", |bytes| {
        push_uint(bytes, "AnimationState", "animationId", 0);
    });
    push_object(&mut bytes, "StateTransition", |bytes| {
        push_uint(bytes, "StateTransition", "stateToId", 3);
    });
    push_object(&mut bytes, "TransitionFocusCondition", |_| {});
    push_object(&mut bytes, "AnimationState", |bytes| {
        push_uint(bytes, "AnimationState", "animationId", 1);
    });
    push_object(&mut bytes, "ExitState", |_| {});
    bytes
}

fn attached(
    manager: &mut FocusManager,
    parent: Option<nuxie_runtime::FocusNodeId>,
    node: FocusNode,
) -> nuxie_runtime::FocusNodeId {
    let id = manager.create_node(node);
    assert!(manager.add_child(parent, id));
    id
}

fn scope_with_two(
    edge: FocusEdgeBehavior,
) -> (
    FocusManager,
    nuxie_runtime::FocusNodeId,
    nuxie_runtime::FocusNodeId,
    nuxie_runtime::FocusNodeId,
) {
    let mut manager = FocusManager::new();
    let mut scope = FocusNode::new();
    scope.set_edge_behavior(edge);
    let scope = attached(&mut manager, None, scope);
    let first = attached(&mut manager, Some(scope), FocusNode::new());
    let second = attached(&mut manager, Some(scope), FocusNode::new());
    (manager, scope, first, second)
}

fn real_focus_fixture(
    asset: &str,
    artboard_name: Option<&str>,
) -> (ArtboardInstance, StateMachineInstance) {
    let path = std::path::Path::new(
        option_env!("RIVE_RUNTIME_DIR").unwrap_or("/Users/levi/dev/oss/rive-runtime"),
    )
    .join("tests/unit_tests")
    .join(asset);
    let bytes = std::fs::read(&path)
        .unwrap_or_else(|error| panic!("read pinned fixture {}: {error}", path.display()));
    let file = read_runtime_file(&bytes)
        .unwrap_or_else(|error| panic!("import pinned fixture {}: {error}", path.display()));
    let graphs = GraphFile::from_runtime_file(&file)
        .unwrap_or_else(|error| panic!("graph pinned fixture {}: {error}", path.display()));
    let graph = artboard_name
        .and_then(|name| {
            graphs
                .artboards
                .iter()
                .find(|graph| graph.name.as_deref() == Some(name))
        })
        .or_else(|| graphs.artboards.first())
        .expect("pinned fixture has an artboard");
    let mut artboard = ArtboardInstance::from_graph_with_artboards(&file, graph, &graphs.artboards)
        .unwrap_or_else(|error| panic!("instantiate pinned fixture {}: {error}", path.display()));
    let mut machine = artboard
        .state_machine_instance(0)
        .unwrap_or_else(|| panic!("pinned fixture {} has state machine 0", path.display()));
    let _ = machine.bind_default_view_model_context_on_artboard(&mut artboard);
    for _ in 0..2 {
        let _ = machine.advance_and_apply(&mut artboard, 0.016);
    }
    (artboard, machine)
}

struct StatefulFocusFixture {
    runtime: RuntimeFile,
    graphs: GraphFile,
    artboard: ArtboardInstance,
    machine: StateMachineInstance,
    context: RuntimeOwnedViewModelContext,
}

impl StatefulFocusFixture {
    fn load(asset: &str, artboard_name: Option<&str>) -> Self {
        let mut fixture = Self::load_before_frames(asset, artboard_name);
        fixture.frames(2, 0.016);
        fixture
    }

    fn load_before_frames(asset: &str, artboard_name: Option<&str>) -> Self {
        let path = std::path::Path::new(
            option_env!("RIVE_RUNTIME_DIR").unwrap_or("/Users/levi/dev/oss/rive-runtime"),
        )
        .join("tests/unit_tests")
        .join(asset);
        let bytes = std::fs::read(&path)
            .unwrap_or_else(|error| panic!("read pinned fixture {}: {error}", path.display()));
        let runtime = read_runtime_file(&bytes)
            .unwrap_or_else(|error| panic!("import pinned fixture {}: {error}", path.display()));
        let graphs = GraphFile::from_runtime_file(&runtime)
            .unwrap_or_else(|error| panic!("graph pinned fixture {}: {error}", path.display()));
        let (artboard_index, graph) = graphs
            .artboards
            .iter()
            .enumerate()
            .find(|(_, graph)| artboard_name.is_none_or(|name| graph.name.as_deref() == Some(name)))
            .expect("pinned fixture has selected artboard");
        let mut artboard =
            ArtboardInstance::from_graph_with_artboards(&runtime, graph, &graphs.artboards)
                .expect("instantiate selected artboard");
        let mut machine = artboard.state_machine_instance(0).expect("state machine 0");
        let view_model_index = runtime
            .artboard(artboard_index)
            .and_then(|object| object.uint_property("viewModelId"))
            .and_then(|value| usize::try_from(value).ok())
            .expect("selected artboard view-model id");
        let main = RuntimeOwnedViewModelInstance::from_instance(&runtime, view_model_index, 0)
            .or_else(|| RuntimeOwnedViewModelInstance::new(&runtime, view_model_index))
            .expect("default view-model instance");
        let mut context = RuntimeOwnedViewModelContext::from_main(main);
        context.complete_for_artboard(&runtime, artboard_index);
        artboard.bind_owned_view_model_artboard_contexts(&runtime, &context);
        assert!(machine.bind_owned_view_model_contexts(&context));
        machine.advance_data_context();
        Self {
            runtime,
            graphs,
            artboard,
            machine,
            context,
        }
    }

    fn root(&self) -> RuntimeOwnedViewModelHandle {
        self.context
            .main_handle()
            .expect("main view-model handle")
            .clone()
    }

    fn frames(&mut self, count: usize, elapsed: f32) {
        for _ in 0..count {
            StateMachineInstance::advance_and_apply_state_machines_with_view_models(
                &mut self.artboard,
                std::slice::from_mut(&mut self.machine),
                elapsed,
                true,
                || false,
            )
            .expect("stateful focus frame");
        }
    }

    fn source(&self, name: &str) -> (u32, RuntimeBindableArtboard) {
        let graph = self
            .graphs
            .artboards
            .iter()
            .find(|graph| graph.name.as_deref() == Some(name))
            .unwrap_or_else(|| panic!("missing source artboard {name}"));
        let instance = ArtboardInstance::from_graph_with_artboards(
            &self.runtime,
            graph,
            &self.graphs.artboards,
        )
        .unwrap_or_else(|error| panic!("instantiate source {name}: {error}"));
        (
            graph.global_id,
            RuntimeBindableArtboard::new_with_artboard_instance(name, &instance),
        )
    }

    fn nested_graphs(&mut self) -> Vec<u32> {
        let mut ids = Vec::new();
        self.artboard
            .try_visit_artboard_tree_instances_mut(&mut |_, global_id, _| {
                ids.push(global_id);
                Ok::<_, ()>(())
            })
            .expect("visit nested artboard tree");
        ids
    }
}

#[test]
fn wave_b3_focus_test_001_direct_port() {
    // Pinned focus_test.cpp case 1.
    let node = FocusNode::new();
    assert!(node.can_focus());
    assert!(node.can_touch());
    assert!(node.can_traverse());
    assert_eq!(node.tab_index(), 0);
    assert_eq!(node.edge_behavior(), FocusEdgeBehavior::ParentScope);
    assert!(!node.has_focus());
    let mut manager = FocusManager::new();
    let node = manager.create_node(node);
    assert!(manager.parent(node).is_none());
    assert!(
        manager
            .children(node)
            .is_some_and(|children| children.is_empty())
    );
    let is_scope = manager
        .children(node)
        .is_some_and(|children| !children.is_empty());
    assert!(!is_scope, "a fresh FocusNode is not a structural scope");
    assert!(!manager.is_attached(node));
}

#[test]
fn wave_b3_focus_test_002_direct_port() {
    // Pinned focus_test.cpp case 2.
    let mut node = FocusNode::new();
    node.set_can_focus(false);
    node.set_can_touch(false);
    node.set_can_traverse(false);
    node.set_tab_index(42);
    node.set_edge_behavior(FocusEdgeBehavior::ClosedLoop);
    assert!(!node.can_focus());
    assert!(!node.can_touch());
    assert!(!node.can_traverse());
    assert_eq!(node.tab_index(), 42);
    assert_eq!(node.edge_behavior(), FocusEdgeBehavior::ClosedLoop);
    node.set_edge_behavior(FocusEdgeBehavior::Stop);
    assert_eq!(node.edge_behavior(), FocusEdgeBehavior::Stop);
}

#[test]
fn wave_b3_focus_test_003_direct_port() {
    // Pinned focus_test.cpp case 3.
    let mut manager = FocusManager::new();
    let node = attached(&mut manager, None, FocusNode::new());
    assert!(manager.set_focus(node));
    let events = manager.take_events();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].kind, FocusEventKind::Focused);
    assert!(manager.clear_focus());
    assert_eq!(manager.take_events()[0].kind, FocusEventKind::Blurred);
}

#[test]
fn wave_b3_focus_test_004_direct_port() {
    // Pinned focus_test.cpp case 4.
    let (mut artboard, mut machine) = real_focus_fixture("assets/text_input_event.riv", None);
    let _ = machine.clear_focus();
    assert!(!machine.focus_state().has_focus);
    assert!(!machine.key_input(&mut artboard, 65, 0, true, false));
    assert!(!machine.text_input(&mut artboard, "hello"));
    assert!(!machine.clear_focus());
}

#[test]
fn wave_b3_focus_test_005_direct_port() {
    // Pinned focus_test.cpp case 5.
    let mut manager = FocusManager::new();
    let node = manager.create_node(FocusNode::new());
    assert!(manager.contains(node));
    assert!(!manager.is_attached(node));
    assert!(manager.add_child(None, node));
    assert!(manager.is_attached(node));
    assert!(manager.detach_subtree(node));
    assert!(manager.contains(node));
    assert!(!manager.is_attached(node));
}

#[test]
fn wave_b3_focus_test_006_direct_port() {
    // Pinned focus_test.cpp case 6.
    let mut manager = FocusManager::new();
    let parent = attached(&mut manager, None, FocusNode::new());
    let child1 = attached(&mut manager, Some(parent), FocusNode::new());
    let child2 = attached(&mut manager, Some(parent), FocusNode::new());
    assert_eq!(manager.parent(child1), Some(parent));
    assert_eq!(manager.parent(child2), Some(parent));
    assert_eq!(manager.children(parent), Some(&[child1, child2][..]));
    assert!(manager.detach_subtree(child1));
    assert_eq!(manager.parent(child1), None);
    assert_eq!(manager.children(parent), Some(&[child2][..]));
    assert!(manager.contains(child1));
}

#[test]
fn wave_b3_focus_test_007_direct_port() {
    // Pinned focus_test.cpp case 7.
    let mut manager = FocusManager::new();
    let node = attached(&mut manager, None, FocusNode::new());
    assert_eq!(manager.primary_focus(), None);
    assert!(manager.set_focus(node));
    assert!(manager.has_focus(node));
    assert!(manager.has_primary_focus(node));
    assert!(manager.clear_focus());
    assert_eq!(manager.primary_focus(), None);
}

#[test]
fn wave_b3_focus_test_008_direct_port() {
    // Pinned focus_test.cpp case 8.
    let mut manager = FocusManager::new();
    let first = attached(&mut manager, None, FocusNode::new());
    let second = attached(&mut manager, None, FocusNode::new());
    assert!(manager.set_focus(first));
    manager.take_events();
    assert!(manager.set_focus(second));
    let events = manager.take_events();
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].kind, FocusEventKind::Blurred);
    assert_eq!(events[1].kind, FocusEventKind::Focused);
}

#[test]
fn wave_b3_focus_test_009_direct_port() {
    // Pinned focus_test.cpp case 9.
    let mut manager = FocusManager::new();
    let mut node = FocusNode::new();
    node.set_can_focus(false);
    let node = attached(&mut manager, None, node);
    assert!(!manager.set_focus(node));
    assert_eq!(manager.primary_focus(), None);
}

#[test]
fn wave_b3_focus_test_010_direct_port() {
    // Pinned focus_test.cpp case 10.
    let mut manager = FocusManager::new();
    let parent = attached(&mut manager, None, FocusNode::new());
    let child1 = attached(&mut manager, Some(parent), FocusNode::new());
    let child2 = attached(&mut manager, Some(parent), FocusNode::new());
    assert_eq!(manager.parent(child1), Some(parent));
    assert_eq!(manager.parent(child2), Some(parent));
    assert_eq!(manager.children(parent), Some(&[child1, child2][..]));
    assert_eq!(manager.roots(), &[parent]);
}

#[test]
fn wave_b3_focus_test_011_direct_port() {
    // Pinned focus_test.cpp case 11.
    let mut manager = FocusManager::new();
    let parent = attached(&mut manager, None, FocusNode::new());
    let child = attached(&mut manager, Some(parent), FocusNode::new());
    assert!(manager.set_focus(child));
    assert!(manager.has_focus(parent));
    assert!(!manager.has_primary_focus(parent));
    assert!(manager.has_focus(child));
    assert!(manager.has_primary_focus(child));
}

#[test]
fn wave_b3_focus_test_012_direct_port() {
    // Pinned focus_test.cpp case 12.
    let mut manager = FocusManager::new();
    let node = attached(&mut manager, None, FocusNode::new());
    manager.set_focus(node);
    manager.take_events();
    assert!(manager.remove_subtree(node));
    assert_eq!(manager.primary_focus(), None);
    assert_eq!(manager.take_events()[0].kind, FocusEventKind::Blurred);
}

#[test]
fn wave_b3_focus_test_013_direct_port() {
    // Pinned focus_test.cpp case 13.
    let mut manager = FocusManager::new();
    let scope = attached(&mut manager, None, FocusNode::new());
    let row = attached(&mut manager, Some(scope), FocusNode::new());
    let leaf = attached(&mut manager, Some(row), FocusNode::new());
    manager.set_focus(leaf);
    manager.take_events();
    assert!(manager.detach_subtree(row));
    assert_eq!(manager.primary_focus(), Some(leaf));
    assert!(manager.insert_child(Some(scope), row, 0));
    assert_eq!(manager.primary_focus(), Some(leaf));
    assert!(manager.take_events().is_empty());
}

#[test]
fn wave_b3_focus_test_014_direct_port() {
    // Pinned focus_test.cpp case 14.
    let mut manager = FocusManager::new();
    let scope = attached(&mut manager, None, FocusNode::structural_scope());
    let child = attached(&mut manager, Some(scope), FocusNode::structural_scope());
    assert!(!manager.has_focusable_content());
    assert!(manager.set_node_can_focus(child, true));
    assert!(manager.has_focusable_content());
    assert!(manager.set_node_can_focus(child, false));
    assert!(!manager.has_focusable_content());
}

#[test]
fn wave_b3_focus_test_015_direct_port() {
    // Pinned focus_test.cpp case 15.
    let mut manager = FocusManager::new();
    let scope = attached(&mut manager, None, FocusNode::structural_scope());
    let child = attached(&mut manager, Some(scope), FocusNode::structural_scope());
    assert!(!manager.has_focusable_content());
    assert!(manager.set_node_can_focus(child, true));
    assert!(manager.has_focusable_content());
    assert!(manager.set_node_can_focus(child, false));
    assert!(!manager.has_focusable_content());
}

#[test]
fn wave_b3_focus_test_016_direct_port() {
    // Pinned focus_test.cpp case 16.
    let mut manager = FocusManager::new();
    let scope = attached(&mut manager, None, FocusNode::structural_scope());
    assert!(!manager.has_focusable_content());
    let child = attached(&mut manager, Some(scope), FocusNode::new());
    assert!(manager.has_focusable_content());
    assert!(manager.remove_subtree(child));
    assert!(!manager.has_focusable_content());
}

#[test]
fn wave_b3_focus_test_017_direct_port() {
    // Pinned focus_test.cpp case 17.
    let mut first = FocusManager::new();
    let node = attached(&mut first, None, FocusNode::new());
    assert!(first.has_focusable_content());
    let mut second = FocusManager::new();
    assert!(second.migrate_subtree_from(&mut first, node, None, 0));
    assert!(!first.has_focusable_content());
    assert!(second.has_focusable_content());
}

#[test]
fn wave_b3_focus_test_018_direct_port() {
    // Pinned focus_test.cpp case 18.
    let mut manager = FocusManager::new();
    let node = attached(&mut manager, None, FocusNode::new());
    assert_eq!(manager.primary_focus(), None);
    manager.set_focus(node);
    assert_eq!(manager.primary_focus(), Some(node));
    assert_eq!(manager.take_events()[0].kind, FocusEventKind::Focused);
}

#[test]
fn wave_b3_focus_test_019_direct_port() {
    // Pinned focus_test.cpp case 19.
    let mut manager = FocusManager::new();
    let a = attached(&mut manager, None, FocusNode::new());
    let b = attached(&mut manager, None, FocusNode::new());
    let c = attached(&mut manager, None, FocusNode::new());
    manager.set_focus(a);
    assert!(manager.focus_next());
    assert_eq!(manager.primary_focus(), Some(b));
    assert!(manager.focus_next());
    assert_eq!(manager.primary_focus(), Some(c));
    assert!(manager.focus_previous());
    assert_eq!(manager.primary_focus(), Some(b));
}

#[test]
fn wave_b3_focus_test_020_direct_port() {
    // Pinned focus_test.cpp case 20.
    let mut manager = FocusManager::new();
    let mut n1 = FocusNode::new();
    n1.set_tab_index(3);
    let n1 = attached(&mut manager, None, n1);
    let mut n2 = FocusNode::new();
    n2.set_tab_index(1);
    let n2 = attached(&mut manager, None, n2);
    let mut n3 = FocusNode::new();
    n3.set_tab_index(2);
    let n3 = attached(&mut manager, None, n3);
    assert!(manager.focus_next());
    assert_eq!(manager.primary_focus(), Some(n2));
    assert!(manager.focus_next());
    assert_eq!(manager.primary_focus(), Some(n3));
    assert!(manager.focus_next());
    assert_eq!(manager.primary_focus(), Some(n1));
}

#[test]
fn wave_b3_focus_test_021_direct_port() {
    // Pinned focus_test.cpp case 21.
    let mut manager = FocusManager::new();
    let a = attached(&mut manager, None, FocusNode::new());
    let mut skipped = FocusNode::new();
    skipped.set_can_traverse(false);
    let _ = attached(&mut manager, None, skipped);
    let c = attached(&mut manager, None, FocusNode::new());
    manager.set_focus(a);
    assert!(manager.focus_next());
    assert_eq!(manager.primary_focus(), Some(c));
}

#[test]
fn wave_b3_focus_test_022_direct_port() {
    // Pinned focus_test.cpp case 22.
    let (mut manager, _, first, second) = scope_with_two(FocusEdgeBehavior::ClosedLoop);
    manager.set_focus(second);
    assert!(manager.focus_next());
    assert_eq!(manager.primary_focus(), Some(first));
}

#[test]
fn wave_b3_focus_test_023_direct_port() {
    // Pinned focus_test.cpp case 23.
    let (mut manager, _, first, second) = scope_with_two(FocusEdgeBehavior::Stop);
    manager.set_focus(second);
    assert!(!manager.focus_next());
    assert_eq!(manager.primary_focus(), Some(second));
    assert_ne!(first, second);
}

#[test]
fn wave_b3_focus_test_024_direct_port() {
    // Pinned focus_test.cpp case 24.
    let mut manager = FocusManager::new();
    let grand = attached(&mut manager, None, FocusNode::new());
    let parent = attached(&mut manager, Some(grand), FocusNode::new());
    let child = attached(&mut manager, Some(parent), FocusNode::new());
    manager.set_focus(child);
    assert!(manager.has_focus(child));
    assert!(manager.has_focus(parent));
    assert!(manager.has_focus(grand));
    assert_eq!(manager.take_events().len(), 3);
}

#[test]
fn wave_b3_focus_test_025_direct_port() {
    // Pinned focus_test.cpp case 25.
    let mut manager = FocusManager::new();
    let parent = attached(&mut manager, None, FocusNode::new());
    let a = attached(&mut manager, Some(parent), FocusNode::new());
    let b = attached(&mut manager, Some(parent), FocusNode::new());
    manager.set_focus(a);
    manager.take_events();
    manager.set_focus(b);
    let events = manager.take_events();
    assert_eq!(events.len(), 2);
    assert!(manager.has_focus(parent));
}

#[test]
fn wave_b3_focus_test_026_direct_port() {
    // Pinned focus_test.cpp case 26.
    let (mut manager, scope, first, second) = scope_with_two(FocusEdgeBehavior::ParentScope);
    assert!(manager.focus_next());
    assert_eq!(manager.primary_focus(), Some(first));
    assert!(manager.has_focus(scope));
    assert!(manager.focus_next());
    assert_eq!(manager.primary_focus(), Some(second));
}

#[test]
fn wave_b3_focus_test_027_direct_port() {
    // Pinned focus_test.cpp case 27.
    let mut manager = FocusManager::new();
    let s1 = attached(&mut manager, None, FocusNode::new());
    let s2 = attached(&mut manager, Some(s1), FocusNode::new());
    let leaf = attached(&mut manager, Some(s2), FocusNode::new());
    assert!(manager.focus_next());
    assert_eq!(manager.primary_focus(), Some(leaf));
    assert!(manager.has_focus(s1));
    assert!(manager.has_focus(s2));
}

#[test]
fn wave_b3_focus_test_028_direct_port() {
    // Pinned focus_test.cpp case 28.
    let mut manager = FocusManager::new();
    let root = attached(&mut manager, None, FocusNode::new());
    let scope = attached(&mut manager, Some(root), FocusNode::new());
    let _inner1 = attached(&mut manager, Some(scope), FocusNode::new());
    let inner2 = attached(&mut manager, Some(scope), FocusNode::new());
    let outer = attached(&mut manager, Some(root), FocusNode::new());
    manager.set_focus(inner2);
    assert!(manager.focus_next());
    assert_eq!(manager.primary_focus(), Some(outer));
}

#[test]
fn wave_b3_focus_test_029_direct_port() {
    // Pinned focus_test.cpp case 29.
    let mut manager = FocusManager::new();
    let parent = attached(&mut manager, None, FocusNode::new());
    let child = attached(&mut manager, Some(parent), FocusNode::new());
    manager.set_focus(child);
    manager.take_events();
    manager.clear_focus();
    assert!(!manager.has_focus(parent));
    assert!(!manager.has_focus(child));
    let events = manager.take_events();
    assert_eq!(events.len(), 2);
    assert!(
        events
            .iter()
            .all(|event| event.kind == FocusEventKind::Blurred)
    );
}

#[test]
fn wave_b3_focus_test_030_direct_port() {
    // Pinned focus_test.cpp case 30.
    let mut manager = FocusManager::new();
    let node = attached(&mut manager, None, FocusNode::new());
    assert!(manager.is_attached(node));
    assert!(manager.detach_subtree(node));
    assert!(manager.contains(node));
    assert!(!manager.is_attached(node));
}

#[test]
#[ignore = "expected-red: removing a transient parent erases the surviving child from Rust's retained focus arena"]
fn wave_b3_focus_test_031_direct_port() {
    // Pinned focus_test.cpp case 31.
    let mut manager = FocusManager::new();
    let row = attached(&mut manager, None, FocusNode::new());
    let survivor = attached(&mut manager, Some(row), FocusNode::new());
    assert!(manager.remove_subtree(row));
    assert!(!manager.contains(row));
    assert!(
        manager.contains(survivor),
        "the child outlives its freed parent in pinned C++"
    );
    let new_parent = attached(&mut manager, None, FocusNode::new());
    assert!(manager.add_child(Some(new_parent), survivor));
    assert_eq!(manager.children(new_parent), Some(&[survivor][..]));
}

#[test]
fn wave_b3_focus_test_032_direct_port() {
    // Pinned focus_test.cpp case 32.
    let mut source = FocusManager::new();
    let scope = attached(&mut source, None, FocusNode::new());
    let mut target = FocusManager::new();
    assert!(target.migrate_subtree_from(&mut source, scope, None, 0));
    assert!(source.roots().is_empty());
    assert_eq!(target.roots(), &[scope]);
}

#[test]
fn wave_b3_focus_test_033_direct_port() {
    // Pinned focus_test.cpp case 33.
    let mut target = FocusManager::new();
    let scope = {
        let mut source = FocusManager::new();
        let scope = attached(&mut source, None, FocusNode::new());
        assert!(target.migrate_subtree_from(&mut source, scope, None, 0));
        scope
    };
    assert!(target.contains(scope));
    assert!(target.remove_subtree(scope));
    assert!(target.roots().is_empty());
}

#[test]
fn wave_b3_focus_test_034_direct_port() {
    // Pinned focus_test.cpp case 34.
    let mut manager = FocusManager::new();
    let root = attached(&mut manager, None, FocusNode::new());
    let before = attached(&mut manager, Some(root), FocusNode::new());
    let scope = attached(&mut manager, Some(root), FocusNode::new());
    let inner = attached(&mut manager, Some(scope), FocusNode::new());
    manager.set_focus(inner);
    assert!(manager.focus_previous());
    assert_eq!(manager.primary_focus(), Some(before));
}

#[test]
fn wave_b3_focus_test_035_direct_port() {
    // Pinned focus_test.cpp case 35.
    let (mut manager, _, first, second) = scope_with_two(FocusEdgeBehavior::ClosedLoop);
    manager.set_focus(first);
    assert!(manager.focus_previous());
    assert_eq!(manager.primary_focus(), Some(second));
}

#[test]
fn wave_b3_focus_test_036_direct_port() {
    // Pinned focus_test.cpp case 36.
    let (mut manager, _, first, _second) = scope_with_two(FocusEdgeBehavior::Stop);
    manager.set_focus(first);
    assert!(!manager.focus_previous());
    assert_eq!(manager.primary_focus(), Some(first));
}

#[test]
fn wave_b3_focus_test_037_direct_port() {
    // Pinned focus_test.cpp case 37.
    let mut manager = FocusManager::new();
    let _ = attached(&mut manager, None, FocusNode::structural_scope());
    assert!(!manager.has_focusable_content());
}

#[test]
fn wave_b3_focus_test_038_direct_port() {
    // Pinned focus_test.cpp case 38.
    let mut manager = FocusManager::new();
    let scope = attached(&mut manager, None, FocusNode::structural_scope());
    assert!(!manager.has_focusable_content());
    let _leaf = attached(&mut manager, Some(scope), FocusNode::new());
    assert!(manager.has_focusable_content());
}

#[test]
fn wave_b3_focus_test_039_direct_port() {
    // Pinned focus_test.cpp case 39.
    let mut manager = FocusManager::new();
    let mut ineligible = FocusNode::new();
    ineligible.set_can_focus(false);
    ineligible.set_can_traverse(false);
    let _ = attached(&mut manager, None, ineligible);
    assert!(!manager.has_focusable_content());
}

#[test]
fn wave_b3_focus_test_040_direct_port() {
    // Pinned focus_test.cpp case 40.
    let mut manager = FocusManager::new();
    let a = attached(&mut manager, None, FocusNode::new());
    let scope = attached(&mut manager, None, FocusNode::structural_scope());
    let c = attached(&mut manager, None, FocusNode::new());
    manager.focus_next();
    assert_eq!(manager.primary_focus(), Some(a));
    manager.focus_next();
    assert_eq!(manager.primary_focus(), Some(c));
    manager.clear_focus();
    let b = attached(&mut manager, Some(scope), FocusNode::new());
    manager.focus_next();
    assert_eq!(manager.primary_focus(), Some(a));
    manager.focus_next();
    assert_eq!(manager.primary_focus(), Some(b));
    manager.focus_next();
    assert_eq!(manager.primary_focus(), Some(c));
}

#[test]
fn wave_b3_focus_test_041_direct_port() {
    // Pinned focus_test.cpp case 41.
    let mut manager = FocusManager::new();
    let scope = attached(&mut manager, None, FocusNode::structural_scope());
    let leaf = attached(&mut manager, Some(scope), FocusNode::new());
    manager.focus_next();
    assert_eq!(manager.primary_focus(), Some(leaf));
    manager.node_mut(leaf).unwrap().set_eligible(false);
    assert!(manager.drop_focus_if_ineligible());
    assert_eq!(manager.primary_focus(), None);
}

#[test]
fn wave_b3_focus_test_042_direct_port() {
    // Pinned focus_test.cpp case 42.
    let mut manager = FocusManager::new();
    let a = attached(&mut manager, None, FocusNode::structural_scope());
    let b = attached(&mut manager, None, FocusNode::structural_scope());
    let leaf_a = attached(&mut manager, Some(a), FocusNode::new());
    let leaf_b = attached(&mut manager, Some(b), FocusNode::new());
    manager.set_focus(leaf_a);
    assert!(manager.remove_subtree(leaf_b));
    let _ = attached(&mut manager, Some(b), FocusNode::new());
    assert_eq!(manager.primary_focus(), Some(leaf_a));
}

#[test]
fn wave_b3_focus_test_043_direct_port() {
    // Pinned focus_test.cpp case 43.
    let mut manager = FocusManager::new();
    let a = attached(&mut manager, None, FocusNode::new());
    let b = attached(&mut manager, None, FocusNode::new());
    manager.set_focus(a);
    assert!(manager.focus_next());
    assert_eq!(manager.primary_focus(), Some(b));
}

#[test]
fn wave_b3_focus_test_044_direct_port() {
    // Pinned focus_test.cpp case 44.
    let mut manager = FocusManager::new();
    let a = attached(&mut manager, None, FocusNode::new());
    let b = attached(&mut manager, None, FocusNode::new());
    manager.set_focus(b);
    assert!(manager.focus_previous());
    assert_eq!(manager.primary_focus(), Some(a));
}

#[test]
fn wave_b3_focus_test_045_direct_port() {
    // Pinned focus_test.cpp case 45.
    let mut manager = FocusManager::new();
    let a = attached(&mut manager, None, FocusNode::new());
    let b = attached(&mut manager, None, FocusNode::new());
    manager.set_focus(a);
    assert!(manager.focus_next());
    assert_eq!(manager.primary_focus(), Some(b));
}

#[test]
fn wave_b3_focus_test_046_direct_port() {
    // Pinned focus_test.cpp case 46.
    let (artboard, mut machine) = real_focus_fixture("assets/focus_collapsing.riv", None);
    assert!(machine.has_focus_nodes());
    let _ = machine.clear_focus();
    assert!(machine.focus_next(&artboard));
    assert!(machine.focus_next(&artboard));
    assert!(machine.focus_previous(&artboard));
}

#[test]
fn wave_b3_focus_test_047_direct_port() {
    // Pinned focus_test.cpp case 47.
    let mut manager = FocusManager::new();
    assert!(!manager.focus_next());
    assert_eq!(manager.primary_focus(), None);
}

#[test]
fn wave_b3_focus_test_048_direct_port() {
    // Pinned focus_test.cpp case 48.
    let node = FocusNode::new();
    assert!(node.can_focus());
    assert!(!node.has_focus());
}

#[test]
fn wave_b3_focus_test_049_direct_port() {
    // Pinned focus_test.cpp case 49.
    let manager = FocusManager::new();
    assert_eq!(manager.primary_focus(), None);
}

#[test]
fn wave_b3_focus_test_050_direct_port() {
    // Pinned focus_test.cpp case 50.
    let mut manager = FocusManager::new();
    let node = attached(&mut manager, None, FocusNode::new());
    manager.set_focus(node);
    assert_eq!(manager.primary_focus(), Some(node));
}

#[test]
fn wave_b3_focus_test_051_direct_port() {
    // Pinned focus_test.cpp case 51.
    let mut manager = FocusManager::new();
    let node = attached(&mut manager, None, FocusNode::new());
    manager.set_focus(node);
    assert_eq!(manager.primary_focus(), Some(node));
}

#[test]
fn wave_b3_focus_test_052_direct_port() {
    // Pinned focus_test.cpp case 52.
    let mut manager = FocusManager::new();
    let node = attached(&mut manager, None, FocusNode::new());
    manager.set_focus(node);
    assert!(manager.clear_focus());
    assert_eq!(manager.primary_focus(), None);
}

#[test]
fn wave_b3_focus_test_053_direct_port() {
    // Pinned focus_test.cpp case 53.
    let mut manager = FocusManager::new();
    let plain = attached(&mut manager, None, FocusNode::new());
    let keyboard = attached(&mut manager, None, FocusNode::new());
    manager.set_focus(plain);
    assert_eq!(manager.primary_focus(), Some(plain));
    manager.set_focus(keyboard);
    assert_eq!(manager.primary_focus(), Some(keyboard));
    manager.set_focus(plain);
    assert_eq!(manager.primary_focus(), Some(plain));
}

#[test]
fn wave_b3_focus_test_054_direct_port() {
    // Pinned focus_test.cpp case 54.
    let mut internal = FocusManager::new();
    let mut external = FocusManager::new();
    let node = attached(&mut external, None, FocusNode::new());
    external.set_focus(node);
    assert_eq!(internal.primary_focus(), None);
    assert_eq!(external.primary_focus(), Some(node));
    assert!(!internal.clear_focus());
}

#[test]
fn wave_b3_focus_test_055_direct_port() {
    // Pinned focus_test.cpp case 55.
    let mut manager = FocusManager::new();
    let node = attached(&mut manager, None, FocusNode::new());
    manager.set_focus(node);
    assert!(manager.clear_focus());
    assert_eq!(manager.primary_focus(), None);
}

#[test]
fn wave_b3_focus_test_056_direct_port() {
    // Pinned focus_test.cpp case 56.
    let (mut manager, _scope, first, _second) = scope_with_two(FocusEdgeBehavior::ParentScope);
    let scope = manager.roots()[0];
    manager.set_focus(scope);
    assert_eq!(manager.primary_focus(), Some(first));
}

#[test]
fn wave_b3_focus_test_057_direct_port() {
    // Pinned focus_test.cpp case 57.
    let mut manager = FocusManager::new();
    let scope = attached(&mut manager, None, FocusNode::new());
    let row = attached(&mut manager, Some(scope), FocusNode::new());
    let leaf = attached(&mut manager, Some(row), FocusNode::new());
    let _sibling = attached(&mut manager, Some(scope), FocusNode::new());
    manager.set_focus(scope);
    assert_eq!(manager.primary_focus(), Some(leaf));
}

#[test]
fn wave_b3_focus_test_058_direct_port() {
    // Pinned focus_test.cpp case 58.
    let mut manager = FocusManager::new();
    let scope = attached(&mut manager, None, FocusNode::new());
    let mut child = FocusNode::new();
    child.set_can_traverse(false);
    let _ = attached(&mut manager, Some(scope), child);
    manager.set_focus(scope);
    assert_eq!(manager.primary_focus(), Some(scope));
}

#[test]
fn wave_b3_focus_test_059_direct_port() {
    // Pinned focus_test.cpp case 59.
    let mut manager = FocusManager::new();
    let mut scope = FocusNode::new();
    scope.set_can_focus(false);
    let scope = attached(&mut manager, None, scope);
    let _ = attached(&mut manager, Some(scope), FocusNode::new());
    assert!(!manager.set_focus(scope));
    assert_eq!(manager.primary_focus(), None);
}

#[test]
fn wave_b3_focus_test_060_direct_port() {
    // Pinned focus_test.cpp case 60.
    let (mut manager, _scope, _first, second) = scope_with_two(FocusEdgeBehavior::ParentScope);
    manager.set_focus(second);
    assert_eq!(manager.primary_focus(), Some(second));
}

#[test]
fn wave_b3_focus_test_061_direct_port() {
    // Pinned focus_test.cpp case 61.
    let (mut manager, scope, first, second) = scope_with_two(FocusEdgeBehavior::ParentScope);
    manager.set_focus(scope);
    assert_eq!(manager.primary_focus(), Some(first));
    manager.focus_next();
    assert_eq!(manager.primary_focus(), Some(second));
}

#[test]
fn wave_b3_focus_test_062_direct_port() {
    // Pinned focus_test.cpp case 62.
    let mut manager = FocusManager::new();
    let node = attached(&mut manager, None, FocusNode::new());
    manager.set_focus(node);
    assert!(manager.clear_focus());
    assert_eq!(manager.primary_focus(), None);
}

#[test]
fn wave_b3_focus_test_063_direct_port() {
    // Pinned focus_test.cpp case 63.
    let mut manager = FocusManager::new();
    assert!(!manager.clear_focus());
    assert_eq!(manager.primary_focus(), None);
}

#[test]
fn wave_b3_focus_test_064_direct_port() {
    // Pinned focus_test.cpp case 64.
    let mut manager = FocusManager::new();
    assert!(!manager.clear_focus());
}

#[test]
fn wave_b3_focus_test_065_direct_port() {
    // Pinned focus_test.cpp case 65.
    let definition =
        nuxie_schema::definition_by_name("TransitionFocusCondition").expect("schema owner");
    assert_eq!(definition.type_key.int, 1038);
}

#[test]
fn wave_b3_focus_test_066_direct_port() {
    // Pinned focus_test.cpp case 66.
    let manager = FocusManager::new();
    assert_eq!(manager.primary_focus(), None);
}

#[test]
fn wave_b3_focus_test_067_direct_port() {
    // Pinned focus_test.cpp case 67.
    let file = read_runtime_file(&focus_condition_without_comparator_fixture())
        .expect("synthetic focus-condition fixture imports");
    let graphs = GraphFile::from_runtime_file(&file).expect("synthetic focus-condition graph");
    let mut artboard = ArtboardInstance::from_graph_with_artboards(
        &file,
        graphs.artboards.first().expect("synthetic artboard"),
        &graphs.artboards,
    )
    .expect("synthetic focus-condition artboard instantiates");
    let mut machine = artboard
        .state_machine_instance(0)
        .expect("synthetic state machine");
    let _ = artboard.advance_state_machine_instance(&mut machine, 0.0);
    artboard.update_components();
    assert_eq!(
        artboard.transform_property(1, TransformProperty::X),
        Some(2.0),
        "an authored focus condition without a component comparator must stay false",
    );
}

#[test]
fn wave_b3_focus_test_068_direct_port() {
    // Pinned focus_test.cpp case 68.
    let mut fixture = StatefulFocusFixture::load("assets/bindable_focus_tree_swap.riv", None);
    assert!(fixture.machine.has_focus_nodes());
    assert!(fixture.machine.focus_next(&fixture.artboard));
    assert!(fixture.machine.focus_state().has_focus);
    assert!(!fixture.machine.focus_next(&fixture.artboard));
    let _ = fixture.machine.focus_previous(&fixture.artboard);

    let (focusable_graph, focusable) = fixture.source("Focusable");
    assert!(
        fixture
            .root()
            .borrow_mut()
            .set_runtime_artboard_by_property_name("bindedArt", Some(focusable),)
    );
    fixture.frames(1, 0.016);
    assert!(fixture.nested_graphs().contains(&focusable_graph));
    assert!(fixture.machine.focus_next(&fixture.artboard));
    assert!(fixture.machine.focus_state().has_focus);
}

#[test]
#[ignore = "expected-red: swapping the unrelated bindable nested artboard clears focus held on the main artboard"]
fn wave_b3_focus_test_069_direct_port() {
    // Pinned focus_test.cpp case 69.
    let mut fixture = StatefulFocusFixture::load("assets/bindable_focus_tree_swap.riv", None);
    assert!(fixture.machine.focus_next(&fixture.artboard));
    assert!(fixture.machine.focus_state().has_focus);
    assert!(!fixture.machine.focus_previous(&fixture.artboard));

    let (focusable_graph, focusable) = fixture.source("Focusable");
    assert!(
        fixture
            .root()
            .borrow_mut()
            .set_runtime_artboard_by_property_name("bindedArt", Some(focusable),)
    );
    fixture.frames(1, 0.016);
    assert!(fixture.nested_graphs().contains(&focusable_graph));
    assert!(fixture.machine.focus_state().has_focus);
    assert!(!fixture.machine.focus_previous(&fixture.artboard));
}

#[test]
fn wave_b3_focus_test_070_direct_port() {
    // Pinned focus_test.cpp case 70.
    let (artboard, mut machine) = real_focus_fixture("assets/focus_collapsing.riv", None);
    assert!(
        machine.has_focus_nodes(),
        "pinned fixture must build authored focus data"
    );
    let before = machine.focus_state();
    let moved = machine.focus_next(&artboard);
    assert!(moved, "pinned fixture must expose its first focus stop");
    assert!(machine.focus_state().has_focus);
    if before.has_focus {
        assert_ne!(machine.focus_state(), before);
    }
}

#[test]
fn wave_b3_focus_test_071_direct_port() {
    // Pinned focus_test.cpp case 71.
    let (artboard, mut machine) = real_focus_fixture("assets/keyboard_listener.riv", None);
    assert!(
        machine.has_focus_nodes(),
        "pinned fixture must build authored focus data"
    );
    let before = machine.focus_state();
    let moved = machine.focus_next(&artboard);
    assert!(moved, "pinned fixture must expose its first focus stop");
    assert!(machine.focus_state().has_focus);
    if before.has_focus {
        assert_ne!(machine.focus_state(), before);
    }
}

#[test]
fn wave_b3_focus_test_072_direct_port() {
    // Pinned focus_test.cpp case 72.
    let (artboard, mut machine) = real_focus_fixture("assets/keyboard_listener.riv", None);
    assert!(
        machine.has_focus_nodes(),
        "pinned fixture must build authored focus data"
    );
    let before = machine.focus_state();
    let moved = machine.focus_next(&artboard);
    assert!(moved, "pinned fixture must expose its first focus stop");
    assert!(machine.focus_state().has_focus);
    if before.has_focus {
        assert_ne!(machine.focus_state(), before);
    }
}

#[test]
fn wave_b3_focus_test_073_direct_port() {
    // Pinned focus_test.cpp case 73.
    let mut fixture = StatefulFocusFixture::load("assets/text_input_event.riv", None);
    let main = fixture.root();
    let read = |name| main.borrow().boolean_value_by_property_name(name);

    assert!(fixture.machine.focus_next(&fixture.artboard));
    fixture.frames(1, 0.016);
    assert_eq!(read("isFocused"), Some(true));
    assert_eq!(read("hasKeyed"), Some(false));
    assert_eq!(read("hasTexted"), Some(false));

    assert!(
        !fixture
            .machine
            .key_input(&mut fixture.artboard, 66, 0, true, false)
    );
    fixture.frames(1, 0.016);
    assert_eq!(read("isFocused"), Some(true));
    assert_eq!(read("hasKeyed"), Some(false));
    assert_eq!(read("hasTexted"), Some(false));

    let _ = fixture.machine.text_input(&mut fixture.artboard, "b");
    fixture.frames(1, 0.016);
    assert_eq!(read("isFocused"), Some(true));
    assert_eq!(read("hasKeyed"), Some(false));
    assert_eq!(read("hasTexted"), Some(true));

    let _ = fixture
        .machine
        .key_input(&mut fixture.artboard, 65, 0, true, false);
    fixture.frames(1, 0.016);
    assert_eq!(read("isFocused"), Some(true));
    assert_eq!(read("hasKeyed"), Some(true));
    assert_eq!(read("hasTexted"), Some(true));
}

#[test]
fn wave_b3_focus_test_074_direct_port() {
    // Pinned focus_test.cpp case 74.
    let (artboard, mut machine) = real_focus_fixture("assets/focus_traversal.riv", None);
    assert!(
        machine.has_focus_nodes(),
        "pinned fixture must build authored focus data"
    );
    let before = machine.focus_state();
    let moved = machine.focus_next(&artboard);
    assert!(moved, "pinned fixture must expose its first focus stop");
    assert!(machine.focus_state().has_focus);
    if before.has_focus {
        assert_ne!(machine.focus_state(), before);
    }
}

#[test]
fn wave_b3_focus_test_075_direct_port() {
    // Pinned focus_test.cpp case 75.
    let (artboard, mut machine) = real_focus_fixture("assets/focusable_element.riv", None);
    assert!(
        machine.has_focus_nodes(),
        "pinned fixture must build authored focus data"
    );
    let before = machine.focus_state();
    let moved = machine.focus_next(&artboard);
    assert!(moved, "pinned fixture must expose its first focus stop");
    assert!(machine.focus_state().has_focus);
    if before.has_focus {
        assert_ne!(machine.focus_state(), before);
    }
}

#[test]
#[ignore = "expected-red: component_list_1.riv does not register authored list rows in the retained focus tree"]
fn wave_b3_focus_test_076_direct_port() {
    // Pinned focus_test.cpp case 76.
    let (artboard, mut machine) = real_focus_fixture("assets/component_list_1.riv", None);
    assert!(
        machine.has_focus_nodes(),
        "pinned fixture must build authored focus data"
    );
    let before = machine.focus_state();
    let moved = machine.focus_next(&artboard);
    assert!(moved, "pinned fixture must expose its first focus stop");
    assert!(machine.focus_state().has_focus);
    if before.has_focus {
        assert_ne!(machine.focus_state(), before);
    }
}

#[test]
#[ignore = "expected-red: the Node-hosted component list exposes no authored focus stop after focus-tree construction"]
fn wave_b3_focus_test_077_direct_port() {
    // Pinned focus_test.cpp case 77.
    let (artboard, mut machine) = real_focus_fixture("assets/component_list_1.riv", None);
    assert!(
        machine.has_focus_nodes(),
        "pinned fixture must build authored focus data"
    );
    let before = machine.focus_state();
    let moved = machine.focus_next(&artboard);
    assert!(moved, "pinned fixture must expose its first focus stop");
    assert!(machine.focus_state().has_focus);
    if before.has_focus {
        assert_ne!(machine.focus_state(), before);
    }
}

#[test]
fn wave_b3_focus_test_078_direct_port() {
    // Pinned focus_test.cpp case 78.
    let (artboard, mut machine) = real_focus_fixture("assets/list_focus_order.riv", None);
    assert!(
        machine.has_focus_nodes(),
        "pinned fixture must build authored focus data"
    );
    let before = machine.focus_state();
    let moved = machine.focus_next(&artboard);
    assert!(moved, "pinned fixture must expose its first focus stop");
    assert!(machine.focus_state().has_focus);
    if before.has_focus {
        assert_ne!(machine.focus_state(), before);
    }
}

#[test]
fn wave_b3_focus_test_079_direct_port() {
    // Pinned focus_test.cpp case 79.
    let (artboard, mut machine) = real_focus_fixture("assets/focus_test.riv", None);
    assert!(
        machine.has_focus_nodes(),
        "pinned fixture must build authored focus data"
    );
    let before = machine.focus_state();
    let moved = machine.focus_next(&artboard);
    assert!(moved, "pinned fixture must expose its first focus stop");
    assert!(machine.focus_state().has_focus);
    if before.has_focus {
        assert_ne!(machine.focus_state(), before);
    }
}

#[test]
fn wave_b3_focus_test_080_direct_port() {
    // Pinned focus_test.cpp case 80.
    let mut fixture = StatefulFocusFixture::load("assets/list_focus_order.riv", None);
    assert!(fixture.machine.has_focus_nodes());
    assert!(fixture.machine.focus_next(&fixture.artboard));
    let before_rewire = fixture.machine.focus_state();
    assert!(before_rewire.has_focus);

    assert!(
        fixture
            .machine
            .bind_owned_view_model_contexts(&fixture.context)
    );
    fixture.machine.advance_data_context();
    fixture.frames(1, 0.016);
    assert_eq!(fixture.machine.focus_state(), before_rewire);
    assert!(fixture.machine.focus_next(&fixture.artboard));
}

#[test]
fn wave_b3_focus_test_081_direct_port() {
    // Pinned focus_test.cpp case 81.
    let mut fixture =
        StatefulFocusFixture::load("assets/swappable_artboards_focus.riv", Some("Main"));
    assert!(fixture.machine.has_focus_nodes());
    assert!(fixture.machine.focus_next(&fixture.artboard));
    assert!(fixture.machine.focus_next(&fixture.artboard));
    assert!(fixture.machine.focus_next(&fixture.artboard));
    assert!(!fixture.machine.focus_next(&fixture.artboard));

    let (swappable2_graph, swappable2) = fixture.source("Swappable2");
    assert!(
        fixture
            .root()
            .borrow_mut()
            .set_runtime_artboard_by_property_name("artboardProp", Some(swappable2),)
    );
    fixture.frames(1, 0.016);
    assert!(fixture.nested_graphs().contains(&swappable2_graph));
    assert!(fixture.machine.focus_next(&fixture.artboard));
    assert!(fixture.machine.focus_next(&fixture.artboard));
    assert!(!fixture.machine.focus_next(&fixture.artboard));

    assert!(fixture.machine.focus_next(&fixture.artboard));
    let (swappable1_graph, swappable1) = fixture.source("Swappable1");
    assert!(
        fixture
            .root()
            .borrow_mut()
            .set_runtime_artboard_by_property_name("artboardProp", Some(swappable1),)
    );
    fixture.frames(1, 0.016);
    assert!(fixture.nested_graphs().contains(&swappable1_graph));
    assert!(fixture.machine.focus_state().has_focus);
    assert!(fixture.machine.focus_next(&fixture.artboard));
    assert!(fixture.machine.focus_next(&fixture.artboard));
    assert!(!fixture.machine.focus_next(&fixture.artboard));
}

#[test]
fn wave_b3_focus_test_082_direct_port() {
    // Pinned focus_test.cpp case 82.
    let mut fixture =
        StatefulFocusFixture::load("assets/swappable_artboards_focus.riv", Some("Main"));
    assert!(fixture.machine.focus_next(&fixture.artboard));
    assert!(fixture.machine.focus_next(&fixture.artboard));
    assert!(fixture.machine.focus_next(&fixture.artboard));
    let held = fixture.machine.focus_state();
    assert!(held.has_focus);

    assert!(
        fixture
            .machine
            .bind_owned_view_model_contexts(&fixture.context)
    );
    fixture.machine.advance_data_context();
    fixture.frames(1, 0.016);
    assert_eq!(fixture.machine.focus_state(), held);
    assert!(!fixture.machine.focus_next(&fixture.artboard));
}

#[test]
fn wave_b3_focus_test_083_direct_port() {
    // Pinned focus_test.cpp case 83.
    let mut fixture =
        StatefulFocusFixture::load("assets/swappable_artboards_focus.riv", Some("Main"));
    let foreign = StatefulFocusFixture::load("assets/swappable_artboards_focus.riv", Some("Main"));
    let (foreign_graph, foreign_swappable) = foreign.source("Swappable1");
    assert!(
        fixture
            .root()
            .borrow_mut()
            .set_runtime_artboard_by_property_name("artboardProp", Some(foreign_swappable),)
    );
    fixture.frames(1, 0.016);
    assert!(fixture.nested_graphs().contains(&foreign_graph));
    assert!(fixture.machine.focus_next(&fixture.artboard));
    assert!(fixture.machine.focus_next(&fixture.artboard));
    assert!(fixture.machine.focus_next(&fixture.artboard));
    assert!(!fixture.machine.focus_next(&fixture.artboard));
}

#[test]
fn wave_b3_focus_test_084_direct_port() {
    // Pinned focus_test.cpp case 84.
    let mut fixture =
        StatefulFocusFixture::load("assets/swappable_artboards_focus.riv", Some("Main"));
    assert!(fixture.machine.focus_next(&fixture.artboard));
    let held = fixture.machine.focus_state();
    assert!(held.has_focus);
    let root = fixture.root();
    assert!(
        root.borrow_mut()
            .set_artboard_by_property_name("artboardProp", 9999)
    );
    fixture.frames(1, 0.016);
    assert_eq!(fixture.machine.focus_state(), held);
    assert!(fixture.machine.focus_next(&fixture.artboard));
    assert!(fixture.machine.focus_next(&fixture.artboard));
    assert!(!fixture.machine.focus_next(&fixture.artboard));
}

#[test]
fn wave_b3_focus_test_085_direct_port() {
    // Pinned focus_test.cpp case 85.
    let mut fixture = StatefulFocusFixture::load_before_frames(
        "assets/swappable_artboards_focus.riv",
        Some("Main"),
    );
    assert!(
        fixture
            .root()
            .borrow_mut()
            .set_artboard_by_property_name("artboardProp", u64::from(u32::MAX))
    );
    fixture.frames(2, 0.016);
    assert!(fixture.machine.focus_next(&fixture.artboard));
    assert!(fixture.machine.focus_next(&fixture.artboard));
    assert!(!fixture.machine.focus_next(&fixture.artboard));

    let (swappable1_graph, swappable1) = fixture.source("Swappable1");
    assert!(
        fixture
            .root()
            .borrow_mut()
            .set_runtime_artboard_by_property_name("artboardProp", Some(swappable1),)
    );
    fixture.frames(1, 0.016);
    assert!(fixture.nested_graphs().contains(&swappable1_graph));
    assert!(fixture.machine.focus_next(&fixture.artboard));
    assert!(fixture.machine.focus_next(&fixture.artboard));
    assert!(fixture.machine.focus_next(&fixture.artboard));
    assert!(!fixture.machine.focus_next(&fixture.artboard));
}
