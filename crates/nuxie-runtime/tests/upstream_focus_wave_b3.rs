//! Executable native-owner ports for pinned `focus_test.cpp` Wave B3.
//!
//! Retained pointer identity and callback observations replace the former
//! façade's arena IDs and synthetic event stream.

use nuxie_render_api::{PersistentFactory, RecordingFactory};
use nuxie_runtime::source::{
    animation::state_machine_instance::RuntimeStateMachineInstanceHandle,
    artboard_component_list::ArtboardComponentList,
    bindable_artboard::RuntimeBindableArtboardHandle,
    focus_data::FocusData,
    generated::{
        core_registry::CoreRegistry, node_base::NodeBase,
        viewmodel::viewmodel_instance_artboard_base::ViewModelInstanceArtboardBase,
    },
    input::{
        focus_manager::{FocusManager, RuntimeFocusManagerHandle},
        focus_node::{EdgeBehavior as FocusEdgeBehavior, FocusNode, FocusNodeRef},
        focusable::{Focusable, Key, KeyModifiers},
    },
    viewmodel::{
        viewmodel_instance::ViewModelInstance,
        viewmodel_instance_artboard::ViewModelInstanceArtboard,
        viewmodel_instance_boolean::ViewModelInstanceBoolean,
    },
};
use nuxie_runtime::{
    CoreHandle, File, ImportResult, RuntimeArtboardInstanceHandle, RuntimeFactoryHandle,
    RuntimeFileHandle,
};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
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
    manager: &RuntimeFocusManagerHandle,
    parent: Option<&FocusNodeRef>,
    node: FocusNodeRef,
) -> FocusNodeRef {
    manager.with_focus_manager_mut(|manager| {
        manager.add_child(parent.cloned(), node.clone(), None);
    });
    assert!(node.borrow().manager().unwrap().ptr_eq(manager));
    node
}

fn scope_with_two(
    edge: FocusEdgeBehavior,
) -> (
    RuntimeFocusManagerHandle,
    FocusNodeRef,
    FocusNodeRef,
    FocusNodeRef,
) {
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());
    let scope = FocusNode::new(None);
    scope.borrow_mut().set_edge_behavior(edge);
    let scope = attached(&manager, None, scope);
    let first = attached(&manager, Some(&scope), FocusNode::new(None));
    let second = attached(&manager, Some(&scope), FocusNode::new(None));
    (manager, scope, first, second)
}

// These are pointer observations, not a second focus graph.
fn node_key(node: &FocusNodeRef) -> *const RefCell<FocusNode> {
    Rc::as_ptr(node)
}
fn primary(manager: &RuntimeFocusManagerHandle) -> Option<*const RefCell<FocusNode>> {
    manager.with_focus_manager(|manager| manager.primary_focus().as_ref().map(node_key))
}
fn parent_key(node: &FocusNodeRef) -> Option<*const RefCell<FocusNode>> {
    node.borrow().parent().as_ref().map(node_key)
}
fn children(node: &FocusNodeRef) -> Vec<*const RefCell<FocusNode>> {
    node.borrow().children().iter().map(node_key).collect()
}
fn roots(manager: &RuntimeFocusManagerHandle) -> Vec<*const RefCell<FocusNode>> {
    manager.with_focus_manager(|manager| manager.root_nodes().iter().map(node_key).collect())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FocusEventKind {
    Focused,
    Blurred,
}

// Like the pinned MockFocusable, this only observes callbacks; all focus,
// traversal, eligibility, and ownership behavior is the real native runtime.
struct ObservedFocusable {
    events: Rc<RefCell<Vec<FocusEventKind>>>,
    eligible: Rc<Cell<bool>>,
}
impl Focusable for ObservedFocusable {
    fn key_input(&mut self, _: Key, _: KeyModifiers, _: bool, _: bool) -> bool {
        false
    }
    fn text_input(&mut self, _: &str) -> bool {
        false
    }
    fn focused(&mut self) {
        self.events.borrow_mut().push(FocusEventKind::Focused);
    }
    fn blurred(&mut self) {
        self.events.borrow_mut().push(FocusEventKind::Blurred);
    }
    fn is_eligible_for_focus_traversal(&self) -> bool {
        self.eligible.get()
    }
}
fn observed_node(events: &Rc<RefCell<Vec<FocusEventKind>>>) -> FocusNodeRef {
    FocusNode::new(Some(Rc::new(RefCell::new(ObservedFocusable {
        events: events.clone(),
        eligible: Rc::new(Cell::new(true)),
    }))))
}

#[derive(Default)]
struct RoutedInputObservations {
    return_value: Cell<bool>,
    key_input_count: Cell<usize>,
    text_input_count: Cell<usize>,
    last_key: RefCell<Option<Key>>,
    last_text: RefCell<String>,
}

struct RoutedInputFocusable {
    observations: Rc<RoutedInputObservations>,
}

impl Focusable for RoutedInputFocusable {
    fn key_input(&mut self, key: Key, _: KeyModifiers, _: bool, _: bool) -> bool {
        self.observations
            .key_input_count
            .set(self.observations.key_input_count.get() + 1);
        *self.observations.last_key.borrow_mut() = Some(key);
        self.observations.return_value.get()
    }

    fn text_input(&mut self, text: &str) -> bool {
        self.observations
            .text_input_count
            .set(self.observations.text_input_count.get() + 1);
        self.observations.last_text.replace(text.to_owned());
        self.observations.return_value.get()
    }

    fn focused(&mut self) {}

    fn blurred(&mut self) {}
}

fn routed_input_node(observations: &Rc<RoutedInputObservations>) -> FocusNodeRef {
    FocusNode::new(Some(Rc::new(RefCell::new(RoutedInputFocusable {
        observations: observations.clone(),
    }))))
}

fn import_bytes(bytes: &[u8]) -> RuntimeFileHandle {
    let mut factory = PersistentFactory::new(RecordingFactory::new());
    let retained = RuntimeFactoryHandle::from_factory(&mut factory).expect("retained test factory");
    let mut result = ImportResult::Malformed;
    let file = File::import(bytes, retained, Some(&mut result), None, None)
        .unwrap_or_else(|| panic!("native fixture import: {result:?}"));
    assert_eq!(result, ImportResult::Success);
    file
}
fn import_fixture(asset: &str) -> RuntimeFileHandle {
    let path = std::path::Path::new(
        option_env!("RIVE_RUNTIME_DIR").unwrap_or("/Users/levi/dev/oss/rive-runtime"),
    )
    .join("tests/unit_tests")
    .join(asset);
    let bytes = std::fs::read(&path)
        .unwrap_or_else(|error| panic!("read pinned fixture {}: {error}", path.display()));
    import_bytes(&bytes)
}
fn instance(file: &RuntimeFileHandle, name: Option<&str>) -> RuntimeArtboardInstanceHandle {
    file.with_file(|file| match name {
        Some(name) => file.artboard_named(name),
        None => file.artboard_default(),
    })
    .expect("selected native artboard instance")
}
fn focus_manager(machine: &RuntimeStateMachineInstanceHandle) -> RuntimeFocusManagerHandle {
    machine.with_instance(|machine| machine.focus_manager())
}
fn real_focus_fixture(
    asset: &str,
    name: Option<&str>,
) -> (
    RuntimeFileHandle,
    RuntimeArtboardInstanceHandle,
    RuntimeStateMachineInstanceHandle,
) {
    let file = import_fixture(asset);
    let artboard = instance(&file, name);
    let machine = artboard
        .state_machine_instance_handle(0)
        .expect("state machine 0");
    if let Some(value) = file.with_file_mut(|file| {
        file.create_default_view_model_instance_for_artboard(artboard.core_handle())
    }) {
        machine.with_instance_mut(|machine| machine.bind_view_model_instance(value));
    }
    for _ in 0..2 {
        machine.advance_and_apply(0.016);
    }
    (file, artboard, machine)
}

struct StatefulFocusFixture {
    file: RuntimeFileHandle,
    artboard: RuntimeArtboardInstanceHandle,
    machine: RuntimeStateMachineInstanceHandle,
    view_model: CoreHandle,
}
impl StatefulFocusFixture {
    fn load(asset: &str, name: Option<&str>) -> Self {
        let mut fixture = Self::load_before_frames(asset, name);
        fixture.frames(2, 0.016);
        fixture
    }
    fn load_before_frames(asset: &str, name: Option<&str>) -> Self {
        let file = import_fixture(asset);
        let artboard = instance(&file, name);
        let machine = artboard
            .state_machine_instance_handle(0)
            .expect("state machine 0");
        let view_model = file
            .with_file_mut(|file| {
                file.create_default_view_model_instance_for_artboard(artboard.core_handle())
            })
            .expect("default view-model instance");
        machine.with_instance_mut(|machine| {
            machine.bind_view_model_instance(view_model.clone());
            machine.advanced_data_context();
        });
        Self {
            file,
            artboard,
            machine,
            view_model,
        }
    }
    fn root(&self) -> CoreHandle {
        self.view_model.clone()
    }
    fn frames(&mut self, count: usize, elapsed: f32) {
        for _ in 0..count {
            self.machine.advance_and_apply(elapsed);
        }
    }
    fn source(&self, name: &str) -> (CoreHandle, RuntimeBindableArtboardHandle) {
        let asset = self
            .file
            .with_file(|file| file.bindable_artboard_named(name))
            .unwrap_or_else(|| panic!("missing bindable source {name}"));
        (
            asset.source_artboard_handle().expect("authored source"),
            asset,
        )
    }
    fn nested_sources(&self) -> Vec<CoreHandle> {
        fn visit(artboard: &RuntimeArtboardInstanceHandle, sources: &mut Vec<CoreHandle>) {
            let hosts = artboard.with_artboard(|artboard| artboard.nested_artboards());
            for host in hosts {
                let child = host
                    .with(|host| {
                        let host = host.as_nested_artboard().expect("nested artboard owner");
                        host.artboard_instance_default()
                    })
                    .expect("live nested host");
                if let Some(child) = child {
                    // NestedArtboard::sourceArtboard is the mounted occurrence
                    // after nest(), not the File's authored definition.
                    sources.push(
                        child
                            .with_artboard(|child| child.artboard_source_handle())
                            .expect("mounted artboard retains its authored source"),
                    );
                    visit(&child, sources);
                }
            }
        }
        let mut sources = Vec::new();
        visit(&self.artboard, &mut sources);
        sources
    }
}
fn property(root: &CoreHandle, name: &str) -> CoreHandle {
    root.with_downcast::<ViewModelInstance, _>(|root| root.property_value_named(name))
        .flatten()
        .unwrap_or_else(|| panic!("missing view-model property {name}"))
}
fn set_artboard(root: &CoreHandle, name: &str, value: Option<RuntimeBindableArtboardHandle>) {
    property(root, name)
        .with_downcast_mut::<ViewModelInstanceArtboard, _>(|property| property.set_asset(value))
        .expect("artboard property");
}
fn set_artboard_id(root: &CoreHandle, name: &str, value: u32) {
    assert!(CoreRegistry::set_uint_handle(
        &property(root, name),
        i32::from(ViewModelInstanceArtboardBase::PROPERTY_VALUE_PROPERTY_KEY),
        value
    ));
}
fn read_bool(root: &CoreHandle, name: &str) -> Option<bool> {
    property(root, name).with_downcast::<ViewModelInstanceBoolean, _>(|value| value.value())
}
#[test]
fn wave_b3_focus_test_001_direct_port() {
    // Pinned focus_test.cpp case 1.
    let node = FocusNode::new(None);
    assert!(node.borrow().can_focus());
    assert!(node.borrow().can_touch());
    assert!(node.borrow().can_traverse());
    assert_eq!(node.borrow().tab_index(), 0);
    assert_eq!(
        node.borrow().edge_behavior(),
        FocusEdgeBehavior::ParentScope
    );
    assert!(!node.borrow().has_focus());
    assert!(parent_key(&node).is_none());
    assert!(node.borrow().children().is_empty());
    let is_scope = node.borrow().is_scope();
    assert!(!is_scope, "a fresh FocusNode is not a structural scope");
    assert!(!node.borrow().manager().is_some());
}

#[test]
fn wave_b3_focus_test_002_direct_port() {
    // Pinned focus_test.cpp case 2.
    let node = FocusNode::new(None);
    node.borrow_mut().set_can_focus(false);
    node.borrow_mut().set_can_touch(false);
    node.borrow_mut().set_can_traverse(false);
    node.borrow_mut().set_tab_index(42);
    node.borrow_mut()
        .set_edge_behavior(FocusEdgeBehavior::ClosedLoop);
    assert!(!node.borrow().can_focus());
    assert!(!node.borrow().can_touch());
    assert!(!node.borrow().can_traverse());
    assert_eq!(node.borrow().tab_index(), 42);
    assert_eq!(node.borrow().edge_behavior(), FocusEdgeBehavior::ClosedLoop);
    node.borrow_mut().set_edge_behavior(FocusEdgeBehavior::Stop);
    assert_eq!(node.borrow().edge_behavior(), FocusEdgeBehavior::Stop);
}

#[test]
fn wave_b3_focus_test_003_direct_port() {
    // Pinned focus_test.cpp case 3.
    let callback_events = Rc::new(RefCell::new(Vec::new()));
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());
    let node = attached(&manager, None, observed_node(&callback_events));
    manager.with_focus_manager_mut(|manager| manager.set_focus(node.clone()));
    assert!(manager.with_focus_manager(|manager| manager.has_primary_focus(&node)));
    let events = std::mem::take(&mut *callback_events.borrow_mut());
    assert_eq!(events.len(), 1);
    assert_eq!(events[0], FocusEventKind::Focused);
    assert!(primary(&manager).is_some());
    manager.with_focus_manager_mut(|manager| manager.clear_focus());
    assert!(primary(&manager).is_none());
    assert_eq!(
        std::mem::take(&mut *callback_events.borrow_mut())[0],
        FocusEventKind::Blurred
    );
}

#[test]
fn wave_b3_focus_test_004_direct_port() {
    // Pinned case 4, plus the preserved real-fixture no-focus input regression.
    let (_file, _artboard, machine) = real_focus_fixture("assets/text_input_event.riv", None);
    let manager = focus_manager(&machine);
    manager.with_focus_manager_mut(|manager| manager.clear_focus());
    assert!(
        !machine
            .with_instance(|machine| machine.focus_state())
            .has_focus
    );
    assert!(!manager.with_focus_manager_mut(|manager| manager.key_input(
        Key::A,
        KeyModifiers::NONE,
        true,
        false
    )));
    assert!(!manager.with_focus_manager_mut(|manager| manager.text_input("hello")));
    assert!(primary(&manager).is_none());
    manager.with_focus_manager_mut(|manager| manager.clear_focus());
    assert!(primary(&manager).is_none());
}

#[test]
fn wave_b3_focus_test_005_direct_port() {
    // Pinned focus_test.cpp case 5.
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());
    let node = FocusNode::new(None);
    assert!(Rc::downgrade(&node).upgrade().is_some());
    assert!(!node.borrow().manager().is_some());
    manager.with_focus_manager_mut(|manager| manager.add_child(None, node.clone(), None));
    assert!(node.borrow().manager().unwrap().ptr_eq(&manager));
    assert!(node.borrow().manager().is_some());
    manager.with_focus_manager_mut(|manager| manager.detach_child(&node));
    assert!(node.borrow().manager().is_none());
    assert!(Rc::downgrade(&node).upgrade().is_some());
    assert!(!node.borrow().manager().is_some());
}

#[test]
fn wave_b3_focus_test_006_direct_port() {
    // Pinned focus_test.cpp case 6.
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());
    let parent = attached(&manager, None, FocusNode::new(None));
    let child1 = attached(&manager, Some(&parent), FocusNode::new(None));
    let child2 = attached(&manager, Some(&parent), FocusNode::new(None));
    assert_eq!(parent_key(&child1), Some(node_key(&parent)));
    assert_eq!(parent_key(&child2), Some(node_key(&parent)));
    assert_eq!(
        children(&parent),
        vec![node_key(&child1), node_key(&child2)]
    );
    manager.with_focus_manager_mut(|manager| manager.detach_child(&child1));
    assert!(child1.borrow().manager().is_none());
    assert_eq!(parent_key(&child1), None);
    assert_eq!(children(&parent), vec![node_key(&child2)]);
    assert!(Rc::downgrade(&child1).upgrade().is_some());
}

#[test]
fn wave_b3_focus_test_007_direct_port() {
    // Pinned focus_test.cpp case 7.
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());
    let node = attached(&manager, None, FocusNode::new(None));
    assert_eq!(primary(&manager), None);
    manager.with_focus_manager_mut(|manager| manager.set_focus(node.clone()));
    assert!(manager.with_focus_manager(|manager| manager.has_primary_focus(&node)));
    assert!(manager.with_focus_manager(|manager| manager.has_focus(&node)));
    assert!(manager.with_focus_manager(|manager| manager.has_primary_focus(&node)));
    assert!(primary(&manager).is_some());
    manager.with_focus_manager_mut(|manager| manager.clear_focus());
    assert!(primary(&manager).is_none());
    assert_eq!(primary(&manager), None);
}

#[test]
fn wave_b3_focus_test_008_direct_port() {
    // Pinned focus_test.cpp case 8.
    let callback_events = Rc::new(RefCell::new(Vec::new()));
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());
    let first = attached(&manager, None, observed_node(&callback_events));
    let second = attached(&manager, None, observed_node(&callback_events));
    manager.with_focus_manager_mut(|manager| manager.set_focus(first.clone()));
    assert!(manager.with_focus_manager(|manager| manager.has_primary_focus(&first)));
    std::mem::take(&mut *callback_events.borrow_mut());
    manager.with_focus_manager_mut(|manager| manager.set_focus(second.clone()));
    assert!(manager.with_focus_manager(|manager| manager.has_primary_focus(&second)));
    let events = std::mem::take(&mut *callback_events.borrow_mut());
    assert_eq!(events.len(), 2);
    assert_eq!(events[0], FocusEventKind::Blurred);
    assert_eq!(events[1], FocusEventKind::Focused);
}

#[test]
fn wave_b3_focus_test_009_direct_port() {
    // Pinned focus_test.cpp case 9.
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());
    let node = FocusNode::new(None);
    node.borrow_mut().set_can_focus(false);
    let node = attached(&manager, None, node);
    manager.with_focus_manager_mut(|manager| manager.set_focus(node.clone()));
    assert!(!manager.with_focus_manager(|manager| manager.has_primary_focus(&node)));
    assert_eq!(primary(&manager), None);
}

#[test]
fn wave_b3_focus_test_010_direct_port() {
    // Pinned focus_test.cpp case 10.
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());
    let parent = attached(&manager, None, FocusNode::new(None));
    let child1 = attached(&manager, Some(&parent), FocusNode::new(None));
    let child2 = attached(&manager, Some(&parent), FocusNode::new(None));
    assert_eq!(parent_key(&child1), Some(node_key(&parent)));
    assert_eq!(parent_key(&child2), Some(node_key(&parent)));
    assert_eq!(
        children(&parent),
        vec![node_key(&child1), node_key(&child2)]
    );
    assert_eq!(roots(&manager), vec![node_key(&parent)]);
}

#[test]
fn wave_b3_focus_test_011_direct_port() {
    // Pinned focus_test.cpp case 11.
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());
    let parent = attached(&manager, None, FocusNode::new(None));
    let child = attached(&manager, Some(&parent), FocusNode::new(None));
    manager.with_focus_manager_mut(|manager| manager.set_focus(child.clone()));
    assert!(manager.with_focus_manager(|manager| manager.has_primary_focus(&child)));
    assert!(manager.with_focus_manager(|manager| manager.has_focus(&parent)));
    assert!(!manager.with_focus_manager(|manager| manager.has_primary_focus(&parent)));
    assert!(manager.with_focus_manager(|manager| manager.has_focus(&child)));
    assert!(manager.with_focus_manager(|manager| manager.has_primary_focus(&child)));
}

#[test]
fn wave_b3_focus_test_012_direct_port() {
    // Pinned focus_test.cpp case 12.
    let callback_events = Rc::new(RefCell::new(Vec::new()));
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());
    let node = attached(&manager, None, observed_node(&callback_events));
    manager.with_focus_manager_mut(|manager| manager.set_focus(node.clone()));
    std::mem::take(&mut *callback_events.borrow_mut());
    manager.with_focus_manager_mut(|manager| manager.remove_child(&node));
    assert!(node.borrow().manager().is_none());
    assert_eq!(primary(&manager), None);
    assert_eq!(
        std::mem::take(&mut *callback_events.borrow_mut())[0],
        FocusEventKind::Blurred
    );
}

#[test]
fn wave_b3_focus_test_013_direct_port() {
    // Pinned focus_test.cpp case 13.
    let callback_events = Rc::new(RefCell::new(Vec::new()));
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());
    let scope = attached(&manager, None, observed_node(&callback_events));
    let row = attached(&manager, Some(&scope), observed_node(&callback_events));
    let leaf = attached(&manager, Some(&row), observed_node(&callback_events));
    manager.with_focus_manager_mut(|manager| manager.set_focus(leaf.clone()));
    std::mem::take(&mut *callback_events.borrow_mut());
    FocusNode::remove_from_parent(&row);
    assert!(row.borrow().parent().is_none());
    assert_eq!(primary(&manager), Some(node_key(&leaf)));
    manager.with_focus_manager_mut(|manager| {
        manager.add_child(Some(scope.clone()), row.clone(), Some(0))
    });
    assert_eq!(children(&scope)[0], node_key(&row));
    assert_eq!(primary(&manager), Some(node_key(&leaf)));
    assert!(std::mem::take(&mut *callback_events.borrow_mut()).is_empty());
}

#[test]
fn wave_b3_focus_test_014_direct_port() {
    // Pinned focus_test.cpp case 14.
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());
    let scope = attached(&manager, None, FocusNode::make_structural_scope());
    let child = attached(&manager, Some(&scope), FocusNode::make_structural_scope());
    assert!(!manager.with_focus_manager_mut(|manager| manager.has_focusable_content()));
    child.borrow_mut().set_can_focus(true);
    assert_eq!(child.borrow().can_focus(), true);
    assert!(manager.with_focus_manager_mut(|manager| manager.has_focusable_content()));
    child.borrow_mut().set_can_focus(false);
    assert_eq!(child.borrow().can_focus(), false);
    assert!(!manager.with_focus_manager_mut(|manager| manager.has_focusable_content()));
}

#[test]
fn wave_b3_focus_test_015_direct_port() {
    // Pinned focus_test.cpp case 15.
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());
    let scope = attached(&manager, None, FocusNode::make_structural_scope());
    let child = attached(&manager, Some(&scope), FocusNode::make_structural_scope());
    assert!(!manager.with_focus_manager_mut(|manager| manager.has_focusable_content()));
    child.borrow_mut().set_can_focus(true);
    assert_eq!(child.borrow().can_focus(), true);
    assert!(manager.with_focus_manager_mut(|manager| manager.has_focusable_content()));
    child.borrow_mut().set_can_focus(false);
    assert_eq!(child.borrow().can_focus(), false);
    assert!(!manager.with_focus_manager_mut(|manager| manager.has_focusable_content()));
}

#[test]
fn wave_b3_focus_test_016_direct_port() {
    // Pinned focus_test.cpp case 16.
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());
    let scope = attached(&manager, None, FocusNode::make_structural_scope());
    assert!(!manager.with_focus_manager_mut(|manager| manager.has_focusable_content()));
    let child = attached(&manager, Some(&scope), FocusNode::new(None));
    assert!(manager.with_focus_manager_mut(|manager| manager.has_focusable_content()));
    manager.with_focus_manager_mut(|manager| manager.remove_child(&child));
    assert!(child.borrow().manager().is_none());
    assert!(!manager.with_focus_manager_mut(|manager| manager.has_focusable_content()));
}

#[test]
fn wave_b3_focus_test_017_direct_port() {
    // Pinned focus_test.cpp case 17.
    let first = RuntimeFocusManagerHandle::new(FocusManager::new());
    let node = attached(&first, None, FocusNode::new(None));
    assert!(first.with_focus_manager_mut(|manager| manager.has_focusable_content()));
    let second = RuntimeFocusManagerHandle::new(FocusManager::new());
    second.with_focus_manager_mut(|manager| manager.add_child(None, node.clone(), Some(0)));
    assert!(node.borrow().manager().unwrap().ptr_eq(&second));
    assert!(!first.with_focus_manager_mut(|manager| manager.has_focusable_content()));
    assert!(second.with_focus_manager_mut(|manager| manager.has_focusable_content()));
}

#[test]
fn wave_b3_focus_test_018_direct_port() {
    // Pinned focus_test.cpp case 18.
    let callback_events = Rc::new(RefCell::new(Vec::new()));
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());
    let node = attached(&manager, None, observed_node(&callback_events));
    assert_eq!(primary(&manager), None);
    manager.with_focus_manager_mut(|manager| manager.set_focus(node.clone()));
    assert_eq!(primary(&manager), Some(node_key(&node)));
    assert_eq!(
        std::mem::take(&mut *callback_events.borrow_mut())[0],
        FocusEventKind::Focused
    );
}

#[test]
fn wave_b3_focus_test_019_direct_port() {
    // Pinned focus_test.cpp case 19.
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());
    let a = attached(&manager, None, FocusNode::new(None));
    let b = attached(&manager, None, FocusNode::new(None));
    let c = attached(&manager, None, FocusNode::new(None));
    manager.with_focus_manager_mut(|manager| manager.set_focus(a.clone()));
    assert!(manager.with_focus_manager_mut(|manager| manager.focus_next()));
    assert_eq!(primary(&manager), Some(node_key(&b)));
    assert!(manager.with_focus_manager_mut(|manager| manager.focus_next()));
    assert_eq!(primary(&manager), Some(node_key(&c)));
    assert!(manager.with_focus_manager_mut(|manager| manager.focus_previous()));
    assert_eq!(primary(&manager), Some(node_key(&b)));
}

#[test]
fn wave_b3_focus_test_020_direct_port() {
    // Pinned focus_test.cpp case 20.
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());
    let n1 = FocusNode::new(None);
    n1.borrow_mut().set_tab_index(3);
    let n1 = attached(&manager, None, n1);
    let n2 = FocusNode::new(None);
    n2.borrow_mut().set_tab_index(1);
    let n2 = attached(&manager, None, n2);
    let n3 = FocusNode::new(None);
    n3.borrow_mut().set_tab_index(2);
    let n3 = attached(&manager, None, n3);
    assert!(manager.with_focus_manager_mut(|manager| manager.focus_next()));
    assert_eq!(primary(&manager), Some(node_key(&n2)));
    assert!(manager.with_focus_manager_mut(|manager| manager.focus_next()));
    assert_eq!(primary(&manager), Some(node_key(&n3)));
    assert!(manager.with_focus_manager_mut(|manager| manager.focus_next()));
    assert_eq!(primary(&manager), Some(node_key(&n1)));
}

#[test]
fn wave_b3_focus_test_021_direct_port() {
    // Pinned focus_test.cpp case 21.
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());
    let a = attached(&manager, None, FocusNode::new(None));
    let skipped = FocusNode::new(None);
    skipped.borrow_mut().set_can_traverse(false);
    let _ = attached(&manager, None, skipped);
    let c = attached(&manager, None, FocusNode::new(None));
    manager.with_focus_manager_mut(|manager| manager.set_focus(a.clone()));
    assert!(manager.with_focus_manager_mut(|manager| manager.focus_next()));
    assert_eq!(primary(&manager), Some(node_key(&c)));
}

#[test]
fn wave_b3_focus_test_022_direct_port() {
    // Pinned focus_test.cpp case 22.
    let (manager, _, first, second) = scope_with_two(FocusEdgeBehavior::ClosedLoop);
    manager.with_focus_manager_mut(|manager| manager.set_focus(second.clone()));
    assert!(manager.with_focus_manager_mut(|manager| manager.focus_next()));
    assert_eq!(primary(&manager), Some(node_key(&first)));
}

#[test]
fn wave_b3_focus_test_023_direct_port() {
    // Pinned focus_test.cpp case 23.
    let (manager, _, first, second) = scope_with_two(FocusEdgeBehavior::Stop);
    manager.with_focus_manager_mut(|manager| manager.set_focus(second.clone()));
    assert!(!manager.with_focus_manager_mut(|manager| manager.focus_next()));
    assert_eq!(primary(&manager), Some(node_key(&second)));
    assert!(!Rc::ptr_eq(&first, &second));
}

#[test]
fn wave_b3_focus_test_024_direct_port() {
    // Pinned focus_test.cpp case 24.
    let callback_events = Rc::new(RefCell::new(Vec::new()));
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());
    let grand = attached(&manager, None, observed_node(&callback_events));
    let parent = attached(&manager, Some(&grand), observed_node(&callback_events));
    let child = attached(&manager, Some(&parent), observed_node(&callback_events));
    manager.with_focus_manager_mut(|manager| manager.set_focus(child.clone()));
    assert!(manager.with_focus_manager(|manager| manager.has_focus(&child)));
    assert!(manager.with_focus_manager(|manager| manager.has_focus(&parent)));
    assert!(manager.with_focus_manager(|manager| manager.has_focus(&grand)));
    assert_eq!(std::mem::take(&mut *callback_events.borrow_mut()).len(), 3);
}

#[test]
fn wave_b3_focus_test_025_direct_port() {
    // Pinned focus_test.cpp case 25.
    let callback_events = Rc::new(RefCell::new(Vec::new()));
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());
    let parent = attached(&manager, None, observed_node(&callback_events));
    let a = attached(&manager, Some(&parent), observed_node(&callback_events));
    let b = attached(&manager, Some(&parent), observed_node(&callback_events));
    manager.with_focus_manager_mut(|manager| manager.set_focus(a.clone()));
    std::mem::take(&mut *callback_events.borrow_mut());
    manager.with_focus_manager_mut(|manager| manager.set_focus(b.clone()));
    let events = std::mem::take(&mut *callback_events.borrow_mut());
    assert_eq!(events.len(), 2);
    assert!(manager.with_focus_manager(|manager| manager.has_focus(&parent)));
}

#[test]
fn wave_b3_focus_test_026_direct_port() {
    // Pinned focus_test.cpp case 26.
    let (manager, scope, first, second) = scope_with_two(FocusEdgeBehavior::ParentScope);
    assert!(manager.with_focus_manager_mut(|manager| manager.focus_next()));
    assert_eq!(primary(&manager), Some(node_key(&first)));
    assert!(manager.with_focus_manager(|manager| manager.has_focus(&scope)));
    assert!(manager.with_focus_manager_mut(|manager| manager.focus_next()));
    assert_eq!(primary(&manager), Some(node_key(&second)));
}

#[test]
fn wave_b3_focus_test_027_direct_port() {
    // Pinned focus_test.cpp case 27.
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());
    let s1 = attached(&manager, None, FocusNode::new(None));
    let s2 = attached(&manager, Some(&s1), FocusNode::new(None));
    let leaf = attached(&manager, Some(&s2), FocusNode::new(None));
    assert!(manager.with_focus_manager_mut(|manager| manager.focus_next()));
    assert_eq!(primary(&manager), Some(node_key(&leaf)));
    assert!(manager.with_focus_manager(|manager| manager.has_focus(&s1)));
    assert!(manager.with_focus_manager(|manager| manager.has_focus(&s2)));
}

#[test]
fn wave_b3_focus_test_028_direct_port() {
    // Pinned focus_test.cpp case 28.
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());
    let root = attached(&manager, None, FocusNode::new(None));
    let scope = attached(&manager, Some(&root), FocusNode::new(None));
    let _inner1 = attached(&manager, Some(&scope), FocusNode::new(None));
    let inner2 = attached(&manager, Some(&scope), FocusNode::new(None));
    let outer = attached(&manager, Some(&root), FocusNode::new(None));
    manager.with_focus_manager_mut(|manager| manager.set_focus(inner2.clone()));
    assert!(manager.with_focus_manager_mut(|manager| manager.focus_next()));
    assert_eq!(primary(&manager), Some(node_key(&outer)));
}

#[test]
fn wave_b3_focus_test_029_direct_port() {
    // Pinned focus_test.cpp case 29.
    let callback_events = Rc::new(RefCell::new(Vec::new()));
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());
    let parent = attached(&manager, None, observed_node(&callback_events));
    let child = attached(&manager, Some(&parent), observed_node(&callback_events));
    manager.with_focus_manager_mut(|manager| manager.set_focus(child.clone()));
    std::mem::take(&mut *callback_events.borrow_mut());
    manager.with_focus_manager_mut(|manager| manager.clear_focus());
    assert!(!manager.with_focus_manager(|manager| manager.has_focus(&parent)));
    assert!(!manager.with_focus_manager(|manager| manager.has_focus(&child)));
    let events = std::mem::take(&mut *callback_events.borrow_mut());
    assert_eq!(events.len(), 2);
    assert!(events.iter().all(|event| *event == FocusEventKind::Blurred));
}

#[test]
fn wave_b3_focus_test_030_direct_port() {
    // Pinned focus_test.cpp case 30.
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());
    let node = attached(&manager, None, FocusNode::new(None));
    assert!(node.borrow().manager().is_some());
    manager.with_focus_manager_mut(|manager| manager.detach_child(&node));
    assert!(node.borrow().manager().is_none());
    assert!(Rc::downgrade(&node).upgrade().is_some());
    assert!(!node.borrow().manager().is_some());
}

#[test]
fn wave_b3_focus_test_031_direct_port() {
    // Pinned case 31: the child outlives its freed transient parent.
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());
    let row = attached(&manager, None, FocusNode::new(None));
    let row_weak = Rc::downgrade(&row);
    let survivor = attached(&manager, Some(&row), FocusNode::new(None));
    let survivor_weak = Rc::downgrade(&survivor);
    manager.with_focus_manager_mut(|manager| manager.remove_child(&row));
    drop(row);
    assert!(row_weak.upgrade().is_none());
    assert!(
        survivor_weak.upgrade().is_some(),
        "the child outlives its freed parent in pinned C++"
    );
    assert!(survivor.borrow().parent().is_none());
    let new_parent = attached(&manager, None, FocusNode::new(None));
    manager.with_focus_manager_mut(|manager| {
        manager.add_child(Some(new_parent.clone()), survivor.clone(), None)
    });
    assert_eq!(children(&new_parent), vec![node_key(&survivor)]);
}

#[test]
fn wave_b3_focus_test_032_direct_port() {
    // Pinned focus_test.cpp case 32.
    let source = RuntimeFocusManagerHandle::new(FocusManager::new());
    let scope = attached(&source, None, FocusNode::new(None));
    let target = RuntimeFocusManagerHandle::new(FocusManager::new());
    target.with_focus_manager_mut(|manager| manager.add_child(None, scope.clone(), Some(0)));
    assert!(scope.borrow().manager().unwrap().ptr_eq(&target));
    assert!(roots(&source).is_empty());
    assert_eq!(roots(&target), vec![node_key(&scope)]);
}

#[test]
fn wave_b3_focus_test_033_direct_port() {
    // Pinned focus_test.cpp case 33.
    let target = RuntimeFocusManagerHandle::new(FocusManager::new());
    let scope = {
        let source = RuntimeFocusManagerHandle::new(FocusManager::new());
        let scope = attached(&source, None, FocusNode::new(None));
        target.with_focus_manager_mut(|manager| manager.add_child(None, scope.clone(), Some(0)));
        assert!(scope.borrow().manager().unwrap().ptr_eq(&target));
        scope
    };
    assert!(Rc::downgrade(&scope).upgrade().is_some());
    target.with_focus_manager_mut(|manager| manager.remove_child(&scope));
    assert!(scope.borrow().manager().is_none());
    assert!(roots(&target).is_empty());
}

#[test]
fn wave_b3_focus_test_034_direct_port() {
    // Pinned focus_test.cpp case 34.
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());
    let root = attached(&manager, None, FocusNode::new(None));
    let before = attached(&manager, Some(&root), FocusNode::new(None));
    let scope = attached(&manager, Some(&root), FocusNode::new(None));
    let inner = attached(&manager, Some(&scope), FocusNode::new(None));
    manager.with_focus_manager_mut(|manager| manager.set_focus(inner.clone()));
    assert!(manager.with_focus_manager_mut(|manager| manager.focus_previous()));
    assert_eq!(primary(&manager), Some(node_key(&before)));
}

#[test]
fn wave_b3_focus_test_035_direct_port() {
    // Pinned focus_test.cpp case 35.
    let (manager, _, first, second) = scope_with_two(FocusEdgeBehavior::ClosedLoop);
    manager.with_focus_manager_mut(|manager| manager.set_focus(first.clone()));
    assert!(manager.with_focus_manager_mut(|manager| manager.focus_previous()));
    assert_eq!(primary(&manager), Some(node_key(&second)));
}

#[test]
fn wave_b3_focus_test_036_direct_port() {
    // Pinned focus_test.cpp case 36.
    let (manager, _, first, _second) = scope_with_two(FocusEdgeBehavior::Stop);
    manager.with_focus_manager_mut(|manager| manager.set_focus(first.clone()));
    assert!(!manager.with_focus_manager_mut(|manager| manager.focus_previous()));
    assert_eq!(primary(&manager), Some(node_key(&first)));
}

#[test]
fn wave_b3_focus_test_037_direct_port() {
    // Pinned focus_test.cpp case 37.
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());
    let _ = attached(&manager, None, FocusNode::make_structural_scope());
    assert!(!manager.with_focus_manager_mut(|manager| manager.has_focusable_content()));
}

#[test]
fn wave_b3_focus_test_038_direct_port() {
    // Pinned focus_test.cpp case 38.
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());
    let scope = attached(&manager, None, FocusNode::make_structural_scope());
    assert!(!manager.with_focus_manager_mut(|manager| manager.has_focusable_content()));
    let _leaf = attached(&manager, Some(&scope), FocusNode::new(None));
    assert!(manager.with_focus_manager_mut(|manager| manager.has_focusable_content()));
}

#[test]
fn wave_b3_focus_test_039_direct_port() {
    // Pinned focus_test.cpp case 39.
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());
    let ineligible = FocusNode::new(None);
    ineligible.borrow_mut().set_can_focus(false);
    ineligible.borrow_mut().set_can_traverse(false);
    let _ = attached(&manager, None, ineligible);
    assert!(!manager.with_focus_manager_mut(|manager| manager.has_focusable_content()));
}

#[test]
fn wave_b3_focus_test_040_direct_port() {
    // Pinned focus_test.cpp case 40.
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());
    let a = attached(&manager, None, FocusNode::new(None));
    let scope = attached(&manager, None, FocusNode::make_structural_scope());
    let c = attached(&manager, None, FocusNode::new(None));
    manager.with_focus_manager_mut(|manager| manager.focus_next());
    assert_eq!(primary(&manager), Some(node_key(&a)));
    manager.with_focus_manager_mut(|manager| manager.focus_next());
    assert_eq!(primary(&manager), Some(node_key(&c)));
    manager.with_focus_manager_mut(|manager| manager.clear_focus());
    let b = attached(&manager, Some(&scope), FocusNode::new(None));
    manager.with_focus_manager_mut(|manager| manager.focus_next());
    assert_eq!(primary(&manager), Some(node_key(&a)));
    manager.with_focus_manager_mut(|manager| manager.focus_next());
    assert_eq!(primary(&manager), Some(node_key(&b)));
    manager.with_focus_manager_mut(|manager| manager.focus_next());
    assert_eq!(primary(&manager), Some(node_key(&c)));
}

#[test]
fn wave_b3_focus_test_041_direct_port() {
    // Pinned case 41: eligibility belongs to the actual Focusable.
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());
    let scope = attached(&manager, None, FocusNode::make_structural_scope());
    let eligible = Rc::new(Cell::new(true));
    let leaf = attached(
        &manager,
        Some(&scope),
        FocusNode::new(Some(Rc::new(RefCell::new(ObservedFocusable {
            events: Rc::new(RefCell::new(Vec::new())),
            eligible: eligible.clone(),
        })))),
    );
    manager.with_focus_manager_mut(|manager| manager.focus_next());
    assert_eq!(primary(&manager), Some(node_key(&leaf)));
    eligible.set(false);
    manager.with_focus_manager_mut(|manager| manager.drop_focus_if_focus_target_hidden());
    assert_eq!(primary(&manager), None);
}

#[test]
fn wave_b3_focus_test_042_direct_port() {
    // Pinned focus_test.cpp case 42.
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());
    let a = attached(&manager, None, FocusNode::make_structural_scope());
    let b = attached(&manager, None, FocusNode::make_structural_scope());
    let leaf_a = attached(&manager, Some(&a), FocusNode::new(None));
    let leaf_b = attached(&manager, Some(&b), FocusNode::new(None));
    manager.with_focus_manager_mut(|manager| manager.set_focus(leaf_a.clone()));
    manager.with_focus_manager_mut(|manager| manager.remove_child(&leaf_b));
    assert!(leaf_b.borrow().manager().is_none());
    let _ = attached(&manager, Some(&b), FocusNode::new(None));
    assert_eq!(primary(&manager), Some(node_key(&leaf_a)));
}

#[test]
fn wave_b3_focus_test_043_direct_port() {
    // Pinned focus_test.cpp case 43.
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());
    let a = attached(&manager, None, FocusNode::new(None));
    let b = attached(&manager, None, FocusNode::new(None));
    manager.with_focus_manager_mut(|manager| manager.set_focus(a.clone()));
    assert!(manager.with_focus_manager_mut(|manager| manager.focus_next()));
    assert_eq!(primary(&manager), Some(node_key(&b)));
}

#[test]
fn wave_b3_focus_test_044_direct_port() {
    // Pinned focus_test.cpp case 44.
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());
    let a = attached(&manager, None, FocusNode::new(None));
    let b = attached(&manager, None, FocusNode::new(None));
    manager.with_focus_manager_mut(|manager| manager.set_focus(b.clone()));
    assert!(manager.with_focus_manager_mut(|manager| manager.focus_previous()));
    assert_eq!(primary(&manager), Some(node_key(&a)));
}

#[test]
fn wave_b3_focus_test_045_direct_port() {
    // Pinned focus_test.cpp case 45.
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());
    let a = attached(&manager, None, FocusNode::new(None));
    let b = attached(&manager, None, FocusNode::new(None));
    manager.with_focus_manager_mut(|manager| manager.set_focus(a.clone()));
    assert!(manager.with_focus_manager_mut(|manager| manager.focus_next()));
    assert_eq!(primary(&manager), Some(node_key(&b)));
}

#[test]
fn wave_b3_focus_test_046_direct_port() {
    // Pinned focus_test.cpp case 46.
    let (_file, _artboard, machine) = real_focus_fixture("assets/focus_collapsing.riv", None);
    assert!(machine.with_instance(|machine| machine.has_focus_nodes()));
    let _ = focus_manager(&machine).with_focus_manager_mut(|manager| manager.clear_focus());
    assert!(focus_manager(&machine).with_focus_manager_mut(|manager| manager.focus_next()));
    assert!(focus_manager(&machine).with_focus_manager_mut(|manager| manager.focus_next()));
    assert!(focus_manager(&machine).with_focus_manager_mut(|manager| manager.focus_previous()));
}

#[test]
fn wave_b3_focus_test_047_direct_port() {
    // Pinned focus_test.cpp case 47.
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());
    assert!(!manager.with_focus_manager_mut(|manager| manager.focus_next()));
    assert_eq!(primary(&manager), None);
}

#[test]
fn wave_b3_focus_test_048_direct_port() {
    // Pinned focus_test.cpp case 48.
    let node = FocusNode::new(None);
    assert!(node.borrow().can_focus());
    assert!(!node.borrow().has_focus());
}

#[test]
fn wave_b3_focus_test_049_direct_port() {
    // Pinned focus_test.cpp case 49.
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());
    assert_eq!(primary(&manager), None);
}

#[test]
fn wave_b3_focus_test_050_direct_port() {
    // Pinned focus_test.cpp case 50.
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());
    let node = attached(&manager, None, FocusNode::new(None));
    manager.with_focus_manager_mut(|manager| manager.set_focus(node.clone()));
    assert_eq!(primary(&manager), Some(node_key(&node)));
}

#[test]
fn wave_b3_focus_test_051_direct_port() {
    // Pinned focus_test.cpp case 51.
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());
    let node = attached(&manager, None, FocusNode::new(None));
    manager.with_focus_manager_mut(|manager| manager.set_focus(node.clone()));
    assert_eq!(primary(&manager), Some(node_key(&node)));
}

#[test]
fn wave_b3_focus_test_052_direct_port() {
    // Pinned focus_test.cpp case 52.
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());
    let node = attached(&manager, None, FocusNode::new(None));
    manager.with_focus_manager_mut(|manager| manager.set_focus(node.clone()));
    assert!(primary(&manager).is_some());
    manager.with_focus_manager_mut(|manager| manager.clear_focus());
    assert!(primary(&manager).is_none());
    assert_eq!(primary(&manager), None);
}

#[test]
fn wave_b3_focus_test_053_direct_port() {
    // Pinned focus_test.cpp case 53.
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());
    let plain = attached(&manager, None, FocusNode::new(None));
    let keyboard = attached(&manager, None, FocusNode::new(None));
    manager.with_focus_manager_mut(|manager| manager.set_focus(plain.clone()));
    assert_eq!(primary(&manager), Some(node_key(&plain)));
    manager.with_focus_manager_mut(|manager| manager.set_focus(keyboard.clone()));
    assert_eq!(primary(&manager), Some(node_key(&keyboard)));
    manager.with_focus_manager_mut(|manager| manager.set_focus(plain.clone()));
    assert_eq!(primary(&manager), Some(node_key(&plain)));
}

#[test]
fn wave_b3_focus_test_054_direct_port() {
    // Pinned focus_test.cpp case 54.
    let internal = RuntimeFocusManagerHandle::new(FocusManager::new());
    let external = RuntimeFocusManagerHandle::new(FocusManager::new());
    let node = attached(&external, None, FocusNode::new(None));
    external.with_focus_manager_mut(|manager| manager.set_focus(node.clone()));
    assert_eq!(primary(&internal), None);
    assert_eq!(primary(&external), Some(node_key(&node)));
    assert!(primary(&internal).is_none());
    internal.with_focus_manager_mut(|manager| manager.clear_focus());
    assert!(primary(&internal).is_none());
}

#[test]
fn wave_b3_focus_test_055_direct_port() {
    // Pinned focus_test.cpp case 55.
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());
    let node = attached(&manager, None, FocusNode::new(None));
    manager.with_focus_manager_mut(|manager| manager.set_focus(node.clone()));
    assert!(primary(&manager).is_some());
    manager.with_focus_manager_mut(|manager| manager.clear_focus());
    assert!(primary(&manager).is_none());
    assert_eq!(primary(&manager), None);
}

#[test]
fn wave_b3_focus_test_056_direct_port() {
    // Pinned focus_test.cpp case 56.
    let (manager, _scope, first, _second) = scope_with_two(FocusEdgeBehavior::ParentScope);
    let scope = manager.with_focus_manager(|manager| manager.root_nodes()[0].clone());
    manager.with_focus_manager_mut(|manager| manager.set_focus(scope.clone()));
    assert_eq!(primary(&manager), Some(node_key(&first)));
}

#[test]
fn wave_b3_focus_test_057_direct_port() {
    // Pinned focus_test.cpp case 57.
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());
    let scope = attached(&manager, None, FocusNode::new(None));
    let row = attached(&manager, Some(&scope), FocusNode::new(None));
    let leaf = attached(&manager, Some(&row), FocusNode::new(None));
    let _sibling = attached(&manager, Some(&scope), FocusNode::new(None));
    manager.with_focus_manager_mut(|manager| manager.set_focus(scope.clone()));
    assert_eq!(primary(&manager), Some(node_key(&leaf)));
}

#[test]
fn wave_b3_focus_test_058_direct_port() {
    // Pinned focus_test.cpp case 58.
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());
    let scope = attached(&manager, None, FocusNode::new(None));
    let child = FocusNode::new(None);
    child.borrow_mut().set_can_traverse(false);
    let _ = attached(&manager, Some(&scope), child);
    manager.with_focus_manager_mut(|manager| manager.set_focus(scope.clone()));
    assert_eq!(primary(&manager), Some(node_key(&scope)));
}

#[test]
fn wave_b3_focus_test_059_direct_port() {
    // Pinned focus_test.cpp case 59.
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());
    let scope = FocusNode::new(None);
    scope.borrow_mut().set_can_focus(false);
    let scope = attached(&manager, None, scope);
    let _ = attached(&manager, Some(&scope), FocusNode::new(None));
    manager.with_focus_manager_mut(|manager| manager.set_focus(scope.clone()));
    assert!(!manager.with_focus_manager(|manager| manager.has_primary_focus(&scope)));
    assert_eq!(primary(&manager), None);
}

#[test]
fn wave_b3_focus_test_060_direct_port() {
    // Pinned focus_test.cpp case 60.
    let (manager, _scope, _first, second) = scope_with_two(FocusEdgeBehavior::ParentScope);
    manager.with_focus_manager_mut(|manager| manager.set_focus(second.clone()));
    assert_eq!(primary(&manager), Some(node_key(&second)));
}

#[test]
fn wave_b3_focus_test_061_direct_port() {
    // Pinned focus_test.cpp case 61.
    let (manager, scope, first, second) = scope_with_two(FocusEdgeBehavior::ParentScope);
    manager.with_focus_manager_mut(|manager| manager.set_focus(scope.clone()));
    assert_eq!(primary(&manager), Some(node_key(&first)));
    manager.with_focus_manager_mut(|manager| manager.focus_next());
    assert_eq!(primary(&manager), Some(node_key(&second)));
}

#[test]
fn wave_b3_focus_test_062_direct_port() {
    // Pinned focus_test.cpp case 62.
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());
    let node = attached(&manager, None, FocusNode::new(None));
    manager.with_focus_manager_mut(|manager| manager.set_focus(node.clone()));
    assert!(primary(&manager).is_some());
    manager.with_focus_manager_mut(|manager| manager.clear_focus());
    assert!(primary(&manager).is_none());
    assert_eq!(primary(&manager), None);
}

#[test]
fn wave_b3_focus_test_063_direct_port() {
    // Pinned focus_test.cpp case 63.
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());
    assert!(primary(&manager).is_none());
    manager.with_focus_manager_mut(|manager| manager.clear_focus());
    assert!(primary(&manager).is_none());
    assert_eq!(primary(&manager), None);
}

#[test]
fn wave_b3_focus_test_064_direct_port() {
    // Pinned focus_test.cpp case 64.
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());
    assert!(primary(&manager).is_none());
    manager.with_focus_manager_mut(|manager| manager.clear_focus());
    assert!(primary(&manager).is_none());
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
    let manager = RuntimeFocusManagerHandle::new(FocusManager::new());
    assert_eq!(primary(&manager), None);
}

#[test]
fn wave_b3_focus_test_067_direct_port() {
    // Preserved Rust wire-format regression for pinned focus case 67.
    let file = import_bytes(&focus_condition_without_comparator_fixture());
    let artboard = instance(&file, None);
    let machine = artboard
        .state_machine_instance_handle(0)
        .expect("synthetic state machine");
    machine.advance_and_apply(0.0);
    let node = artboard
        .with_artboard(|artboard| artboard.objects().get(1).cloned().flatten())
        .expect("animated node 1");
    assert_eq!(
        CoreRegistry::get_double_handle(&node, i32::from(NodeBase::X_PROPERTY_KEY)),
        Some(2.0),
        "an authored focus condition without a component comparator must stay false",
    );
}

#[test]
fn wave_b3_focus_test_068_direct_port() {
    // Pinned focus_test.cpp case 68.
    let mut fixture = StatefulFocusFixture::load("assets/bindable_focus_tree_swap.riv", None);
    assert!(
        fixture
            .machine
            .with_instance(|machine| machine.has_focus_nodes())
    );
    assert!(focus_manager(&fixture.machine).with_focus_manager_mut(|manager| manager.focus_next()));
    assert!(
        fixture
            .machine
            .with_instance(|machine| machine.focus_state())
            .has_focus
    );
    assert!(
        !focus_manager(&fixture.machine).with_focus_manager_mut(|manager| manager.focus_next())
    );
    let _ =
        focus_manager(&fixture.machine).with_focus_manager_mut(|manager| manager.focus_previous());

    let (focusable_graph, focusable) = fixture.source("Focusable");
    set_artboard(&fixture.root(), "bindedArt", Some(focusable));
    fixture.frames(1, 0.016);
    assert!(fixture.nested_sources().contains(&focusable_graph));
    assert!(focus_manager(&fixture.machine).with_focus_manager_mut(|manager| manager.focus_next()));
    assert!(
        fixture
            .machine
            .with_instance(|machine| machine.focus_state())
            .has_focus
    );
}

#[test]
fn wave_b3_focus_test_069_direct_port() {
    // Pinned focus_test.cpp case 69.
    let mut fixture = StatefulFocusFixture::load("assets/bindable_focus_tree_swap.riv", None);
    assert!(focus_manager(&fixture.machine).with_focus_manager_mut(|manager| manager.focus_next()));
    assert!(
        fixture
            .machine
            .with_instance(|machine| machine.focus_state())
            .has_focus
    );
    assert!(
        !focus_manager(&fixture.machine).with_focus_manager_mut(|manager| manager.focus_previous())
    );
    // Retain the extra Rust root-edge check, then restore the pinned setup:
    // case69 holds focus on Main before swapping the unrelated child.
    assert!(primary(&focus_manager(&fixture.machine)).is_none());
    assert!(focus_manager(&fixture.machine).with_focus_manager_mut(|manager| manager.focus_next()));
    let held =
        focus_manager(&fixture.machine).with_focus_manager(|manager| manager.primary_focus());

    let (focusable_graph, focusable) = fixture.source("Focusable");
    set_artboard(&fixture.root(), "bindedArt", Some(focusable));
    fixture.frames(1, 0.016);
    assert!(fixture.nested_sources().contains(&focusable_graph));
    assert_eq!(
        primary(&focus_manager(&fixture.machine)),
        held.as_ref().map(node_key)
    );
    assert!(
        fixture
            .machine
            .with_instance(|machine| machine.focus_state())
            .has_focus
    );
    assert!(
        !focus_manager(&fixture.machine).with_focus_manager_mut(|manager| manager.focus_previous())
    );
}

#[test]
fn wave_b3_focus_test_070_direct_port() {
    // Pinned focus_test.cpp case 70.
    let (_file, _artboard, machine) = real_focus_fixture("assets/focus_collapsing.riv", None);
    assert!(
        machine.with_instance(|machine| machine.has_focus_nodes()),
        "pinned fixture must build authored focus data"
    );
    let before = focus_manager(&machine).with_focus_manager(|manager| manager.primary_focus());
    let moved = focus_manager(&machine).with_focus_manager_mut(|manager| manager.focus_next());
    assert!(moved, "pinned fixture must expose its first focus stop");
    assert!(
        machine
            .with_instance(|machine| machine.focus_state())
            .has_focus
    );
    if before.is_some() {
        assert_ne!(
            primary(&focus_manager(&machine)),
            before.as_ref().map(node_key)
        );
    }
}

#[test]
fn wave_b3_focus_test_071_direct_port() {
    // Pinned focus_test.cpp case 71.
    let (_file, _artboard, machine) = real_focus_fixture("assets/keyboard_listener.riv", None);
    assert!(
        machine.with_instance(|machine| machine.has_focus_nodes()),
        "pinned fixture must build authored focus data"
    );
    let before = focus_manager(&machine).with_focus_manager(|manager| manager.primary_focus());
    let moved = focus_manager(&machine).with_focus_manager_mut(|manager| manager.focus_next());
    assert!(moved, "pinned fixture must expose its first focus stop");
    assert!(
        machine
            .with_instance(|machine| machine.focus_state())
            .has_focus
    );
    if before.is_some() {
        assert_ne!(
            primary(&focus_manager(&machine)),
            before.as_ref().map(node_key)
        );
    }
}

#[test]
fn wave_b3_focus_test_072_direct_port() {
    // Pinned focus_test.cpp case 72.
    let (_file, _artboard, machine) = real_focus_fixture("assets/keyboard_listener.riv", None);
    assert!(
        machine.with_instance(|machine| machine.has_focus_nodes()),
        "pinned fixture must build authored focus data"
    );
    let before = focus_manager(&machine).with_focus_manager(|manager| manager.primary_focus());
    let moved = focus_manager(&machine).with_focus_manager_mut(|manager| manager.focus_next());
    assert!(moved, "pinned fixture must expose its first focus stop");
    assert!(
        machine
            .with_instance(|machine| machine.focus_state())
            .has_focus
    );
    if before.is_some() {
        assert_ne!(
            primary(&focus_manager(&machine)),
            before.as_ref().map(node_key)
        );
    }
}

#[test]
fn wave_b3_focus_test_073_direct_port() {
    // Pinned focus_test.cpp case 73.
    let mut fixture = StatefulFocusFixture::load("assets/text_input_event.riv", None);
    let main = fixture.root();
    let read = |name| read_bool(&main, name);

    assert!(focus_manager(&fixture.machine).with_focus_manager_mut(|manager| manager.focus_next()));
    fixture.frames(1, 0.016);
    assert_eq!(read("isFocused"), Some(true));
    assert_eq!(read("hasKeyed"), Some(false));
    assert_eq!(read("hasTexted"), Some(false));

    assert!(
        !focus_manager(&fixture.machine).with_focus_manager_mut(|manager| manager.key_input(
            Key::B,
            KeyModifiers::NONE,
            true,
            false
        ))
    );
    fixture.frames(1, 0.016);
    assert_eq!(read("isFocused"), Some(true));
    assert_eq!(read("hasKeyed"), Some(false));
    assert_eq!(read("hasTexted"), Some(false));

    let _ =
        focus_manager(&fixture.machine).with_focus_manager_mut(|manager| manager.text_input("b"));
    fixture.frames(1, 0.016);
    assert_eq!(read("isFocused"), Some(true));
    assert_eq!(read("hasKeyed"), Some(false));
    assert_eq!(read("hasTexted"), Some(true));

    let _ = focus_manager(&fixture.machine).with_focus_manager_mut(|manager| {
        manager.key_input(Key::A, KeyModifiers::NONE, true, false)
    });
    fixture.frames(1, 0.016);
    assert_eq!(read("isFocused"), Some(true));
    assert_eq!(read("hasKeyed"), Some(true));
    assert_eq!(read("hasTexted"), Some(true));
}

#[test]
fn wave_b3_focus_test_074_direct_port() {
    // Pinned focus_test.cpp case 74.
    let (_file, _artboard, machine) = real_focus_fixture("assets/focus_traversal.riv", None);
    assert!(
        machine.with_instance(|machine| machine.has_focus_nodes()),
        "pinned fixture must build authored focus data"
    );
    let before = focus_manager(&machine).with_focus_manager(|manager| manager.primary_focus());
    let moved = focus_manager(&machine).with_focus_manager_mut(|manager| manager.focus_next());
    assert!(moved, "pinned fixture must expose its first focus stop");
    assert!(
        machine
            .with_instance(|machine| machine.focus_state())
            .has_focus
    );
    if before.is_some() {
        assert_ne!(
            primary(&focus_manager(&machine)),
            before.as_ref().map(node_key)
        );
    }
}

#[test]
fn wave_b3_focus_test_075_direct_port() {
    // Pinned focus_test.cpp case 75.
    let (_file, _artboard, machine) = real_focus_fixture("assets/focusable_element.riv", None);
    assert!(
        machine.with_instance(|machine| machine.has_focus_nodes()),
        "pinned fixture must build authored focus data"
    );
    let before = focus_manager(&machine).with_focus_manager(|manager| manager.primary_focus());
    let moved = focus_manager(&machine).with_focus_manager_mut(|manager| manager.focus_next());
    assert!(moved, "pinned fixture must expose its first focus stop");
    assert!(
        machine
            .with_instance(|machine| machine.focus_state())
            .has_focus
    );
    if before.is_some() {
        assert_ne!(
            primary(&focus_manager(&machine)),
            before.as_ref().map(node_key)
        );
    }
}

#[test]
fn wave_b3_focus_test_076_direct_port() {
    // Pinned focus_test.cpp 2133–2164 tests a transparent structural scope,
    // not a focusable stop (the former façade test asserted the opposite).
    let file = import_fixture("assets/component_list_1.riv");
    let artboard = instance(&file, Some("Main"));
    let view_model = file
        .with_file_mut(|file| {
            file.create_default_view_model_instance_for_artboard(artboard.core_handle())
        })
        .expect("default view-model instance");
    artboard.bind_view_model_instance(Some(view_model));
    let _machine = artboard
        .state_machine_instance_handle(0)
        .expect("Main state machine");
    artboard.advance_default(0.0);
    let list = artboard
        .with_artboard(|artboard| artboard.find_handle::<ArtboardComponentList>("List"))
        .expect("Main List");
    let manager = artboard
        .with_artboard(|artboard| artboard.focus_manager_handle())
        .expect("shared focus manager");
    artboard.build_focus_tree(Some(manager.clone()), None);
    let scope = list
        .with_downcast::<ArtboardComponentList, _>(ArtboardComponentList::list_scope_focus_node)
        .flatten()
        .expect("list scope");
    let scope = scope.borrow();
    assert!(scope.manager().expect("scope manager").ptr_eq(&manager));
    assert_eq!(scope.name, "ArtboardComponentListScope");
    assert!(!scope.can_focus());
    assert!(!scope.can_traverse());
    assert!(scope.focusable().is_none());
}

#[test]
fn wave_b3_focus_test_077_direct_port() {
    // Pinned focus_test.cpp 2166–2203 checks direct FocusData ancestry;
    // a transparent list scope does not imply a focusable stop.
    let file = import_fixture("assets/component_list_1.riv");
    let artboard = instance(&file, Some("Main"));
    let view_model = file
        .with_file_mut(|file| {
            file.create_default_view_model_instance_for_artboard(artboard.core_handle())
        })
        .expect("default view-model instance");
    artboard.bind_view_model_instance(Some(view_model));
    let _machine = artboard
        .state_machine_instance_handle(0)
        .expect("Main state machine");
    artboard.advance_default(0.0);
    let list = artboard
        .with_artboard(|artboard| artboard.find_handle::<ArtboardComponentList>("List"))
        .expect("Main List");
    let parent = list
        .with(|list| list.component_parent_handle())
        .flatten()
        .expect("List parent");
    assert!(parent.is_type_of(NodeBase::TYPE_KEY));
    let children = parent
        .with(|parent| {
            parent
                .as_container_component()
                .expect("Node container")
                .children()
                .to_vec()
        })
        .expect("live Node");
    let first_direct = children
        .into_iter()
        .find(|child| child.is_type_of(FocusData::TYPE_KEY))
        .map(|child| {
            child
                .with_downcast_mut::<FocusData, _>(FocusData::focus_node)
                .expect("direct FocusData")
        });
    if let Some(first_direct) = first_direct {
        let closest = FocusData::find_closest_focus_node_handle(list).expect("closest focus node");
        assert!(Rc::ptr_eq(&closest, &first_direct));
    }
}

#[test]
fn wave_b3_focus_test_078_direct_port() {
    // Pinned focus_test.cpp case 78.
    let (_file, _artboard, machine) = real_focus_fixture("assets/list_focus_order.riv", None);
    assert!(
        machine.with_instance(|machine| machine.has_focus_nodes()),
        "pinned fixture must build authored focus data"
    );
    let before = focus_manager(&machine).with_focus_manager(|manager| manager.primary_focus());
    let moved = focus_manager(&machine).with_focus_manager_mut(|manager| manager.focus_next());
    assert!(moved, "pinned fixture must expose its first focus stop");
    assert!(
        machine
            .with_instance(|machine| machine.focus_state())
            .has_focus
    );
    if before.is_some() {
        assert_ne!(
            primary(&focus_manager(&machine)),
            before.as_ref().map(node_key)
        );
    }
}

#[test]
fn wave_b3_focus_test_079_direct_port() {
    // Pinned focus_test.cpp case 79.
    let (_file, _artboard, machine) = real_focus_fixture("assets/focus_test.riv", None);
    assert!(
        machine.with_instance(|machine| machine.has_focus_nodes()),
        "pinned fixture must build authored focus data"
    );
    let before = focus_manager(&machine).with_focus_manager(|manager| manager.primary_focus());
    let moved = focus_manager(&machine).with_focus_manager_mut(|manager| manager.focus_next());
    assert!(moved, "pinned fixture must expose its first focus stop");
    assert!(
        machine
            .with_instance(|machine| machine.focus_state())
            .has_focus
    );
    if before.is_some() {
        assert_ne!(
            primary(&focus_manager(&machine)),
            before.as_ref().map(node_key)
        );
    }
}

#[test]
fn wave_b3_focus_test_080_direct_port() {
    // Pinned focus_test.cpp case 80.
    let mut fixture = StatefulFocusFixture::load("assets/list_focus_order.riv", None);
    assert!(
        fixture
            .machine
            .with_instance(|machine| machine.has_focus_nodes())
    );
    assert!(focus_manager(&fixture.machine).with_focus_manager_mut(|manager| manager.focus_next()));
    let before_rewire =
        focus_manager(&fixture.machine).with_focus_manager(|manager| manager.primary_focus());
    assert!(before_rewire.is_some());

    fixture
        .machine
        .with_instance_mut(|machine| machine.bind_view_model_instance(fixture.view_model.clone()));
    fixture
        .machine
        .with_instance_mut(|machine| machine.advanced_data_context());
    fixture.frames(1, 0.016);
    assert_eq!(
        primary(&focus_manager(&fixture.machine)),
        before_rewire.as_ref().map(node_key)
    );
    assert!(focus_manager(&fixture.machine).with_focus_manager_mut(|manager| manager.focus_next()));
}

#[test]
fn wave_b3_focus_test_081_direct_port() {
    // Pinned focus_test.cpp case 81.
    let mut fixture =
        StatefulFocusFixture::load("assets/swappable_artboards_focus.riv", Some("Main"));
    assert!(
        fixture
            .machine
            .with_instance(|machine| machine.has_focus_nodes())
    );
    assert!(focus_manager(&fixture.machine).with_focus_manager_mut(|manager| manager.focus_next()));
    assert!(focus_manager(&fixture.machine).with_focus_manager_mut(|manager| manager.focus_next()));
    assert!(focus_manager(&fixture.machine).with_focus_manager_mut(|manager| manager.focus_next()));
    assert!(
        !focus_manager(&fixture.machine).with_focus_manager_mut(|manager| manager.focus_next())
    );

    let (swappable2_graph, swappable2) = fixture.source("Swappable2");
    set_artboard(&fixture.root(), "artboardProp", Some(swappable2));
    fixture.frames(1, 0.016);
    assert!(fixture.nested_sources().contains(&swappable2_graph));
    assert!(focus_manager(&fixture.machine).with_focus_manager_mut(|manager| manager.focus_next()));
    assert!(focus_manager(&fixture.machine).with_focus_manager_mut(|manager| manager.focus_next()));
    assert!(
        !focus_manager(&fixture.machine).with_focus_manager_mut(|manager| manager.focus_next())
    );

    assert!(focus_manager(&fixture.machine).with_focus_manager_mut(|manager| manager.focus_next()));
    let (swappable1_graph, swappable1) = fixture.source("Swappable1");
    set_artboard(&fixture.root(), "artboardProp", Some(swappable1));
    fixture.frames(1, 0.016);
    assert!(fixture.nested_sources().contains(&swappable1_graph));
    assert!(
        fixture
            .machine
            .with_instance(|machine| machine.focus_state())
            .has_focus
    );
    assert!(focus_manager(&fixture.machine).with_focus_manager_mut(|manager| manager.focus_next()));
    assert!(focus_manager(&fixture.machine).with_focus_manager_mut(|manager| manager.focus_next()));
    assert!(
        !focus_manager(&fixture.machine).with_focus_manager_mut(|manager| manager.focus_next())
    );
}

#[test]
fn wave_b3_focus_test_082_direct_port() {
    // Pinned focus_test.cpp case 82.
    let mut fixture =
        StatefulFocusFixture::load("assets/swappable_artboards_focus.riv", Some("Main"));
    assert!(focus_manager(&fixture.machine).with_focus_manager_mut(|manager| manager.focus_next()));
    assert!(focus_manager(&fixture.machine).with_focus_manager_mut(|manager| manager.focus_next()));
    assert!(focus_manager(&fixture.machine).with_focus_manager_mut(|manager| manager.focus_next()));
    let held =
        focus_manager(&fixture.machine).with_focus_manager(|manager| manager.primary_focus());
    assert!(held.is_some());

    fixture
        .machine
        .with_instance_mut(|machine| machine.bind_view_model_instance(fixture.view_model.clone()));
    fixture
        .machine
        .with_instance_mut(|machine| machine.advanced_data_context());
    fixture.frames(1, 0.016);
    assert_eq!(
        primary(&focus_manager(&fixture.machine)),
        held.as_ref().map(node_key)
    );
    assert!(
        !focus_manager(&fixture.machine).with_focus_manager_mut(|manager| manager.focus_next())
    );
}

#[test]
fn wave_b3_focus_test_083_direct_port() {
    // Pinned focus_test.cpp case 83.
    let mut fixture =
        StatefulFocusFixture::load("assets/swappable_artboards_focus.riv", Some("Main"));
    let foreign = StatefulFocusFixture::load("assets/swappable_artboards_focus.riv", Some("Main"));
    let (foreign_graph, foreign_swappable) = foreign.source("Swappable1");
    set_artboard(&fixture.root(), "artboardProp", Some(foreign_swappable));
    fixture.frames(1, 0.016);
    assert!(fixture.nested_sources().contains(&foreign_graph));
    assert!(focus_manager(&fixture.machine).with_focus_manager_mut(|manager| manager.focus_next()));
    assert!(focus_manager(&fixture.machine).with_focus_manager_mut(|manager| manager.focus_next()));
    assert!(focus_manager(&fixture.machine).with_focus_manager_mut(|manager| manager.focus_next()));
    assert!(
        !focus_manager(&fixture.machine).with_focus_manager_mut(|manager| manager.focus_next())
    );
}

#[test]
fn wave_b3_focus_test_084_direct_port() {
    // Pinned focus_test.cpp case 84.
    let mut fixture =
        StatefulFocusFixture::load("assets/swappable_artboards_focus.riv", Some("Main"));
    assert!(focus_manager(&fixture.machine).with_focus_manager_mut(|manager| manager.focus_next()));
    let held =
        focus_manager(&fixture.machine).with_focus_manager(|manager| manager.primary_focus());
    assert!(held.is_some());
    let root = fixture.root();
    set_artboard_id(&root, "artboardProp", 9999);
    fixture.frames(1, 0.016);
    assert_eq!(
        primary(&focus_manager(&fixture.machine)),
        held.as_ref().map(node_key)
    );
    assert!(focus_manager(&fixture.machine).with_focus_manager_mut(|manager| manager.focus_next()));
    assert!(focus_manager(&fixture.machine).with_focus_manager_mut(|manager| manager.focus_next()));
    assert!(
        !focus_manager(&fixture.machine).with_focus_manager_mut(|manager| manager.focus_next())
    );
}

#[test]
fn wave_b3_focus_test_085_direct_port() {
    // Pinned focus_test.cpp case 85.
    let mut fixture = StatefulFocusFixture::load_before_frames(
        "assets/swappable_artboards_focus.riv",
        Some("Main"),
    );
    set_artboard_id(&fixture.root(), "artboardProp", u32::MAX);
    fixture.frames(2, 0.016);
    assert!(focus_manager(&fixture.machine).with_focus_manager_mut(|manager| manager.focus_next()));
    assert!(focus_manager(&fixture.machine).with_focus_manager_mut(|manager| manager.focus_next()));
    assert!(
        !focus_manager(&fixture.machine).with_focus_manager_mut(|manager| manager.focus_next())
    );

    let (swappable1_graph, swappable1) = fixture.source("Swappable1");
    set_artboard(&fixture.root(), "artboardProp", Some(swappable1));
    fixture.frames(1, 0.016);
    assert!(fixture.nested_sources().contains(&swappable1_graph));
    assert!(focus_manager(&fixture.machine).with_focus_manager_mut(|manager| manager.focus_next()));
    assert!(focus_manager(&fixture.machine).with_focus_manager_mut(|manager| manager.focus_next()));
    assert!(focus_manager(&fixture.machine).with_focus_manager_mut(|manager| manager.focus_next()));
    assert!(
        !focus_manager(&fixture.machine).with_focus_manager_mut(|manager| manager.focus_next())
    );
}

#[test]
fn upstream_707c_state_machine_routes_key_and_text_input_to_focus() {
    let (_file, _artboard, machine) = real_focus_fixture("assets/text_input_event.riv", None);
    let manager = focus_manager(&machine);
    manager.with_focus_manager_mut(FocusManager::clear_focus);

    let observations = Rc::new(RoutedInputObservations::default());
    observations.return_value.set(true);

    assert!(!machine.with_instance_mut(|machine| {
        machine.key_input(Key::A, KeyModifiers::NONE, true, false)
    }));
    assert!(!machine.with_instance_mut(|machine| machine.text_input("hello")));
    assert_eq!(observations.key_input_count.get(), 0);
    assert_eq!(observations.text_input_count.get(), 0);

    let node = attached(&manager, None, routed_input_node(&observations));
    manager.with_focus_manager_mut(|manager| manager.set_focus(node));

    assert!(machine.with_instance_mut(|machine| {
        machine.key_input(Key::B, KeyModifiers::SHIFT, true, false)
    }));
    assert_eq!(observations.key_input_count.get(), 1);
    assert_eq!(*observations.last_key.borrow(), Some(Key::B));
    assert!(machine.with_instance_mut(|machine| machine.text_input("world")));
    assert_eq!(observations.text_input_count.get(), 1);
    assert_eq!(&*observations.last_text.borrow(), "world");
}

#[test]
fn upstream_707c_state_machine_reports_unhandled_focused_input() {
    let (_file, _artboard, machine) = real_focus_fixture("assets/text_input_event.riv", None);
    let manager = focus_manager(&machine);
    manager.with_focus_manager_mut(FocusManager::clear_focus);

    let observations = Rc::new(RoutedInputObservations::default());
    let node = attached(&manager, None, routed_input_node(&observations));
    manager.with_focus_manager_mut(|manager| manager.set_focus(node));

    assert!(!machine.with_instance_mut(|machine| {
        machine.key_input(Key::ESCAPE, KeyModifiers::NONE, true, false)
    }));
    assert!(!machine.with_instance_mut(|machine| machine.text_input("ignored")));
    assert_eq!(observations.key_input_count.get(), 1);
    assert_eq!(observations.text_input_count.get(), 1);
}

#[test]
fn upstream_707c_state_machine_routes_input_through_external_focus_manager() {
    let (_file, _artboard, machine) = real_focus_fixture("assets/text_input_event.riv", None);
    let internal = machine.with_instance(|machine| machine.internal_focus_manager());
    internal.with_focus_manager_mut(FocusManager::clear_focus);

    let internal_observations = Rc::new(RoutedInputObservations::default());
    internal_observations.return_value.set(true);
    let internal_node = attached(&internal, None, routed_input_node(&internal_observations));
    internal.with_focus_manager_mut(|manager| manager.set_focus(internal_node));

    let external = RuntimeFocusManagerHandle::new(FocusManager::new());
    let external_observations = Rc::new(RoutedInputObservations::default());
    external_observations.return_value.set(true);
    let external_node = attached(&external, None, routed_input_node(&external_observations));
    external.with_focus_manager_mut(|manager| manager.set_focus(external_node));
    machine.with_instance_mut(|machine| {
        machine.set_external_focus_manager(Some(external.clone()));
    });

    assert!(machine.with_instance_mut(|machine| {
        machine.key_input(Key::C, KeyModifiers::NONE, true, false)
    }));
    assert!(machine.with_instance_mut(|machine| machine.text_input("external")));
    assert_eq!(external_observations.key_input_count.get(), 1);
    assert_eq!(external_observations.text_input_count.get(), 1);
    assert_eq!(&*external_observations.last_text.borrow(), "external");
    assert_eq!(internal_observations.key_input_count.get(), 0);
    assert_eq!(internal_observations.text_input_count.get(), 0);

    machine.with_instance_mut(|machine| machine.set_external_focus_manager(None));
}
