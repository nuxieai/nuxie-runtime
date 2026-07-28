use super::instance::StateMachineInstance;
use super::listener_types::{
    RuntimeGamepadInputEvent, RuntimeListenerInputTypeGamepad, RuntimeListenerInputTypeKeyboard,
    RuntimeListenerInputTypeSemantic, RuntimeListenerInputTypeViewModel, RuntimeListenerType,
};
use super::state_machine_listener_single::{
    runtime_listener_single_event_local_indices, runtime_listener_single_type,
    runtime_listener_single_view_model_property_path,
};
use super::{RuntimeScheduledListenerAction, ScriptListenerInvocation, StateMachineEventContext};
use crate::ArtboardInstance;
use crate::properties::property_key_for_name;
use crate::{RuntimeOwnedViewModelInstance, ScriptError, ScriptHost};
use nuxie_binary::{RuntimeFile, RuntimeObject};
use nuxie_graph::{ArtboardGraph, ParametricPathNode};

#[derive(Debug, Clone)]
pub(crate) struct RuntimeStateMachineListener {
    pub(crate) target_local_id: usize,
    /// C++ gives `StateMachineListenerSingle` distinct event-loop semantics:
    /// one matching report ends the report scan, while a multi-input listener
    /// may fire once for every matching reported event.
    pub(crate) is_single: bool,
    pub(crate) listener_types: Vec<RuntimeListenerType>,
    pub(crate) event_local_indices: Vec<usize>,
    pub(crate) view_model_index: Option<usize>,
    pub(crate) view_model_property_path: Option<Vec<usize>>,
    pub(crate) view_model_input_types: Vec<RuntimeListenerInputTypeViewModel>,
    pub(crate) gamepad_input_types: Vec<RuntimeListenerInputTypeGamepad>,
    pub(crate) keyboard_input_types: Vec<RuntimeListenerInputTypeKeyboard>,
    pub(crate) semantic_input_types: Vec<RuntimeListenerInputTypeSemantic>,
    pub(crate) hit_paths: Vec<RuntimeListenerHitPath>,
    pub(crate) listener_actions: Vec<RuntimeScheduledListenerAction>,
}

impl RuntimeStateMachineListener {
    pub(crate) fn has_listener(&self, listener_type: RuntimeListenerType) -> bool {
        self.listener_types.contains(&listener_type)
    }

    pub(crate) fn hit_test(&self, artboard: &ArtboardInstance, x: f32, y: f32) -> bool {
        if artboard
            .component(self.target_local_id)
            .is_none_or(|component| component.is_collapsed())
        {
            return false;
        }

        self.hit_paths
            .iter()
            .any(|path| path.hit_test(artboard, x, y))
    }

    pub(crate) fn keyboard_constraints_met(
        &self,
        key: u32,
        modifiers: u32,
        is_pressed: bool,
        is_repeat: bool,
    ) -> bool {
        RuntimeListenerInputTypeKeyboard::constraints_met(
            &self.keyboard_input_types,
            key,
            modifiers,
            is_pressed,
            is_repeat,
        )
    }

    pub(crate) fn gamepad_constraints_met(&self, event: RuntimeGamepadInputEvent) -> bool {
        RuntimeListenerInputTypeGamepad::constraints_met(&self.gamepad_input_types, event)
    }

    pub(crate) fn semantic_constraints_met(&self, action_type: u32) -> bool {
        RuntimeListenerInputTypeSemantic::constraints_met(&self.semantic_input_types, action_type)
    }

    /// Run every retained action occurrence in authored order.
    ///
    /// This is the direct Rust owner for
    /// `StateMachineListener::performChanges`; action failures are handled by
    /// their concrete owners, so the listener itself never invents a
    /// rescan/reorder boundary (`state_machine_listener.cpp:85-92`).
    pub(crate) fn perform_changes(
        &self,
        instance: &mut StateMachineInstance,
        artboard: &mut ArtboardInstance,
        owned_context: Option<&mut RuntimeOwnedViewModelInstance>,
        invocation: &ScriptListenerInvocation,
        host: &mut dyn ScriptHost,
        event_context: Option<&StateMachineEventContext>,
    ) -> Result<bool, ScriptError> {
        instance.perform_listener_actions_with_event_context(
            artboard,
            &self.listener_actions,
            owned_context,
            invocation,
            host,
            event_context,
        )
    }
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeListenerHitPath {
    local_id: usize,
    kind: RuntimeListenerHitPathKind,
}

impl RuntimeListenerHitPath {
    fn hit_test(&self, artboard: &ArtboardInstance, x: f32, y: f32) -> bool {
        if artboard
            .component(self.local_id)
            .is_none_or(|component| component.is_collapsed())
        {
            return false;
        }

        match self.kind {
            RuntimeListenerHitPathKind::Rectangle => self.hit_test_rectangle(artboard, x, y),
        }
    }

    fn hit_test_rectangle(&self, artboard: &ArtboardInstance, x: f32, y: f32) -> bool {
        let Some(component) = artboard.component(self.local_id) else {
            return false;
        };
        let local = component.transform.world_transform.invert_or_identity();
        let (local_x, local_y) = local.transform_point(x, y);
        let width = artboard_double_property(artboard, self.local_id, "Rectangle", "width", 0.0);
        let height = artboard_double_property(artboard, self.local_id, "Rectangle", "height", 0.0);
        let origin_x =
            artboard_double_property(artboard, self.local_id, "Rectangle", "originX", 0.5);
        let origin_y =
            artboard_double_property(artboard, self.local_id, "Rectangle", "originY", 0.5);
        let left = -width * origin_x;
        let top = -height * origin_y;
        local_x >= left && local_x <= left + width && local_y >= top && local_y <= top + height
    }
}

#[derive(Debug, Clone, Copy)]
enum RuntimeListenerHitPathKind {
    Rectangle,
}

pub(super) fn runtime_state_machine_listener(
    file: &RuntimeFile,
    graph: &ArtboardGraph,
    state_machine_inputs: &[Option<&RuntimeObject>],
    state_machine_data_binds: &[&RuntimeObject],
    listener: &nuxie_binary::RuntimeStateMachineListener<'_>,
    action_owners: &super::RuntimeActionCoreArena,
) -> Option<RuntimeStateMachineListener> {
    // Mirrors C++ StateMachineListener import/action wiring and the first
    // simple-shape branch of src/animation/state_machine_instance.cpp.
    let target_local_id = usize::try_from(listener.object.uint_property("targetId")?).ok()?;
    let listener_types = runtime_listener_types(listener)
        .into_iter()
        .filter(|listener_type| {
            listener_type.is_pointer_hit()
                || matches!(
                    listener_type,
                    RuntimeListenerType::Event
                        | RuntimeListenerType::ViewModel
                        | RuntimeListenerType::Keyboard
                        | RuntimeListenerType::Gamepad
                        | RuntimeListenerType::Focus
                        | RuntimeListenerType::Blur
                        | RuntimeListenerType::TextInput
                        | RuntimeListenerType::SemanticAction
                )
        })
        .collect::<Vec<_>>();
    // Pinned C++ transfers the StateMachineListener owner and its ordered
    // actions before inspecting any ListenerInputType
    // (`state_machine_listener.cpp:55-66`). An unknown or absent dispatch
    // type therefore leaves an inert listener occurrence; it must not compact
    // the listener/action arrays.

    let hit_paths = if listener_types
        .iter()
        .any(|listener_type| listener_type.is_pointer_hit())
    {
        // C++ retains the listener occurrence and constructs all non-pointer
        // groups even when `addToHitLookup` cannot produce a pointer target.
        // An empty pointer lookup therefore disables only pointer hits.
        runtime_listener_hit_paths(graph, target_local_id)
    } else {
        Vec::new()
    };
    let event_local_indices = runtime_listener_event_local_indices(listener);
    let (view_model_index, view_model_property_path) =
        runtime_listener_single_view_model_property_path(listener)
            .map(|(view_model_index, property_path)| (Some(view_model_index), Some(property_path)))
            .unwrap_or((None, None));

    Some(RuntimeStateMachineListener {
        target_local_id,
        is_single: listener.object.type_name == "StateMachineListenerSingle",
        listener_types,
        event_local_indices,
        view_model_index,
        view_model_property_path,
        view_model_input_types: runtime_listener_input_type_viewmodels(listener),
        gamepad_input_types: runtime_listener_input_type_gamepads(listener),
        keyboard_input_types: runtime_listener_input_type_keyboards(listener),
        semantic_input_types: runtime_listener_input_type_semantics(listener),
        hit_paths,
        listener_actions: listener
            .actions
            .iter()
            .map(|action| {
                RuntimeScheduledListenerAction::from_imported(
                    file,
                    graph,
                    state_machine_inputs,
                    state_machine_data_binds,
                    action,
                    action_owners
                        .handle(action.object.id)
                        .expect("accepted listener action has an owner"),
                )
            })
            .collect(),
    })
}

fn runtime_listener_input_type_gamepads(
    listener: &nuxie_binary::RuntimeStateMachineListener<'_>,
) -> Vec<RuntimeListenerInputTypeGamepad> {
    listener
        .listener_input_types
        .iter()
        .zip(&listener.listener_input_type_inputs)
        .filter(|(input_type, _)| input_type.type_name == "ListenerInputTypeGamepad")
        .map(|(input_type, inputs)| {
            RuntimeListenerInputTypeGamepad::from_imported(input_type, inputs)
        })
        .collect()
}

fn runtime_listener_input_type_keyboards(
    listener: &nuxie_binary::RuntimeStateMachineListener<'_>,
) -> Vec<RuntimeListenerInputTypeKeyboard> {
    listener
        .listener_input_types
        .iter()
        .zip(&listener.listener_input_type_inputs)
        .filter(|(input_type, _)| input_type.type_name == "ListenerInputTypeKeyboard")
        .map(|(input_type, inputs)| {
            RuntimeListenerInputTypeKeyboard::from_imported(input_type, inputs)
        })
        .collect()
}

fn runtime_listener_input_type_semantics(
    listener: &nuxie_binary::RuntimeStateMachineListener<'_>,
) -> Vec<RuntimeListenerInputTypeSemantic> {
    listener
        .listener_input_types
        .iter()
        .zip(&listener.listener_input_type_inputs)
        .filter(|(input_type, _)| input_type.type_name == "ListenerInputTypeSemantic")
        .map(|(input_type, inputs)| {
            RuntimeListenerInputTypeSemantic::from_imported(input_type, inputs)
        })
        .collect()
}

fn runtime_listener_input_type_viewmodels(
    listener: &nuxie_binary::RuntimeStateMachineListener<'_>,
) -> Vec<RuntimeListenerInputTypeViewModel> {
    if listener.object.type_name == "StateMachineListenerSingle" {
        return Vec::new();
    }

    listener
        .listener_input_types
        .iter()
        .filter(|input_type| input_type.type_name == "ListenerInputTypeViewModel")
        .map(|input_type| RuntimeListenerInputTypeViewModel::from_imported(input_type))
        .collect()
}

fn runtime_listener_types(
    listener: &nuxie_binary::RuntimeStateMachineListener<'_>,
) -> Vec<RuntimeListenerType> {
    if listener.object.type_name == "StateMachineListenerSingle" {
        return runtime_listener_single_type(listener).into_iter().collect();
    }

    listener
        .listener_input_types
        .iter()
        .map(|input_type| input_type.uint_property("listenerTypeValue").unwrap_or(0))
        .filter_map(RuntimeListenerType::from_value)
        .collect()
}

fn runtime_listener_event_local_indices(
    listener: &nuxie_binary::RuntimeStateMachineListener<'_>,
) -> Vec<usize> {
    if listener.object.type_name == "StateMachineListenerSingle" {
        return runtime_listener_single_event_local_indices(listener);
    }

    listener
        .listener_input_types
        .iter()
        .filter(|input_type| {
            input_type
                .uint_property("listenerTypeValue")
                .and_then(RuntimeListenerType::from_value)
                == Some(RuntimeListenerType::Event)
        })
        .filter_map(|input_type| {
            input_type
                .uint_property("eventId")
                .and_then(|event_id| usize::try_from(event_id).ok())
        })
        .collect()
}

fn runtime_listener_hit_paths(
    graph: &ArtboardGraph,
    target_local_id: usize,
) -> Vec<RuntimeListenerHitPath> {
    let Some(target) = graph
        .components
        .iter()
        .find(|component| component.local_id == target_local_id)
    else {
        return Vec::new();
    };
    if target.type_name != "Shape" {
        // TODO(golden): port C++ StateMachineInstance::addToHitLookup for
        // containers, layout proxies, text runs, and component-provided groups.
        return Vec::new();
    }

    let Some(composer) = graph
        .path_composers
        .iter()
        .find(|composer| composer.shape_local == target_local_id)
    else {
        return Vec::new();
    };

    composer
        .path_locals
        .iter()
        .filter_map(|path_local| {
            let path = graph
                .paths
                .iter()
                .find(|path| path.local_id == *path_local)?;
            match path.parametric {
                Some(ParametricPathNode::Rectangle { .. }) => Some(RuntimeListenerHitPath {
                    local_id: *path_local,
                    kind: RuntimeListenerHitPathKind::Rectangle,
                }),
                _ => None,
            }
        })
        .collect()
}

fn artboard_double_property(
    artboard: &ArtboardInstance,
    local_id: usize,
    type_name: &str,
    property_name: &str,
    default: f32,
) -> f32 {
    property_key_for_name(type_name, property_name)
        .and_then(|key| artboard.double_property(local_id, key))
        .unwrap_or(default)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nuxie_binary::{AuthoringProperty, AuthoringRecord, AuthoringValue, RuntimeFile};
    use nuxie_graph::GraphFile;

    fn record(type_name: &str, properties: Vec<AuthoringProperty>) -> AuthoringRecord {
        AuthoringRecord {
            type_key: nuxie_schema::definition_by_name(type_name)
                .unwrap_or_else(|| panic!("missing schema definition {type_name}"))
                .type_key
                .int,
            properties,
        }
    }

    fn property(type_name: &str, name: &str, value: u64) -> AuthoringProperty {
        AuthoringProperty {
            key: property_key_for_name(type_name, name)
                .unwrap_or_else(|| panic!("missing property {type_name}.{name}")),
            value: AuthoringValue::Uint(value),
        }
    }

    fn action_owners(
        file: &RuntimeFile,
        state_machine_global_id: u32,
    ) -> super::super::RuntimeActionCoreArena {
        super::super::RuntimeFileStateMachineActionCatalog::new(file)
            .arena(state_machine_global_id)
            .expect("state-machine action owners")
    }

    #[test]
    fn imported_listener_retains_keyboard_owner_and_constraints() {
        let file = RuntimeFile::from_authoring_records(vec![
            record("Backboard", Vec::new()),
            record("Artboard", Vec::new()),
            record("StateMachine", Vec::new()),
            record(
                "StateMachineListener",
                vec![property("StateMachineListener", "targetId", 0)],
            ),
            record(
                "ListenerInputTypeKeyboard",
                vec![property(
                    "ListenerInputTypeKeyboard",
                    "listenerTypeValue",
                    RuntimeListenerType::Keyboard as u64,
                )],
            ),
            record(
                "KeyboardInput",
                vec![
                    property("KeyboardInput", "keyType", 65),
                    property("KeyboardInput", "keyPhase", KEY_PHASE_DOWN_FOR_TEST),
                    property("KeyboardInput", "modifiers", 2),
                ],
            ),
            record(
                "KeyboardInput",
                vec![
                    property("KeyboardInput", "keyType", 66),
                    property("KeyboardInput", "keyPhase", KEY_PHASE_UP_FOR_TEST),
                ],
            ),
        ])
        .expect("keyboard listener authoring records import");
        let graph = GraphFile::from_runtime_file(&file).expect("keyboard listener graph builds");
        let authored = file.artboard_state_machine_graphs(0);
        let listener = runtime_state_machine_listener(
            &file,
            graph.artboards.first().expect("artboard graph"),
            &authored[0].inputs,
            &[],
            &authored[0].listeners[0],
            &action_owners(&file, authored[0].object.id),
        )
        .expect("keyboard listener is retained");

        assert!(listener.has_listener(RuntimeListenerType::Keyboard));
        assert_eq!(listener.keyboard_input_types.len(), 1);
        assert_eq!(listener.keyboard_input_types[0].global_id, 4);
        assert_eq!(
            listener.keyboard_input_types[0]
                .keyboard_input(0)
                .map(|input| input.global_id),
            Some(5)
        );
        assert_eq!(
            listener.keyboard_input_types[0]
                .keyboard_input(1)
                .map(|input| input.global_id),
            Some(6)
        );
        assert!(listener.keyboard_constraints_met(65, 2, true, false));
        assert!(listener.keyboard_constraints_met(66, 0, false, false));
        assert!(!listener.keyboard_constraints_met(65, 0, true, false));
    }

    #[test]
    fn imported_listener_retains_gamepad_and_semantic_owners() {
        let file = RuntimeFile::from_authoring_records(vec![
            record("Backboard", Vec::new()),
            record("Artboard", Vec::new()),
            record("StateMachine", Vec::new()),
            record(
                "StateMachineListener",
                vec![property("StateMachineListener", "targetId", 0)],
            ),
            record(
                "ListenerInputTypeGamepad",
                vec![property(
                    "ListenerInputTypeGamepad",
                    "listenerTypeValue",
                    RuntimeListenerType::Gamepad as u64,
                )],
            ),
            record(
                "GamepadInput",
                vec![
                    property("GamepadInput", "kind", 0),
                    property("GamepadInput", "mapping", 1),
                    property("GamepadInput", "inputIndex", 3),
                    property("GamepadInput", "buttonPhase", 2),
                ],
            ),
            record(
                "ListenerInputTypeSemantic",
                vec![property(
                    "ListenerInputTypeSemantic",
                    "listenerTypeValue",
                    RuntimeListenerType::SemanticAction as u64,
                )],
            ),
            record(
                "SemanticInput",
                vec![property("SemanticInput", "actionType", 2)],
            ),
        ])
        .expect("gamepad and semantic listener records import");
        let graph = GraphFile::from_runtime_file(&file).expect("listener graph builds");
        let authored = file.artboard_state_machine_graphs(0);
        let listener = runtime_state_machine_listener(
            &file,
            graph.artboards.first().expect("artboard graph"),
            &authored[0].inputs,
            &[],
            &authored[0].listeners[0],
            &action_owners(&file, authored[0].object.id),
        )
        .expect("gamepad and semantic listener is retained");

        assert!(listener.has_listener(RuntimeListenerType::Gamepad));
        assert!(listener.has_listener(RuntimeListenerType::SemanticAction));
        assert_eq!(listener.gamepad_input_types[0].global_id, 4);
        assert_eq!(
            listener.gamepad_input_types[0]
                .gamepad_input(0)
                .map(|input| input.global_id),
            Some(5)
        );
        assert_eq!(listener.semantic_input_types[0].global_id, 6);
        assert_eq!(
            listener.semantic_input_types[0]
                .semantic_input(0)
                .map(|input| input.global_id),
            Some(7)
        );
        assert!(
            listener.gamepad_constraints_met(RuntimeGamepadInputEvent::Button {
                index: 3,
                value: 0.0,
                standard_intent: None,
            })
        );
        assert!(listener.semantic_constraints_met(2));
        assert!(!listener.semantic_constraints_met(1));
    }

    #[test]
    fn missing_pointer_hit_shape_does_not_drop_keyboard_channel() {
        let file = RuntimeFile::from_authoring_records(vec![
            record("Backboard", Vec::new()),
            record("Artboard", Vec::new()),
            record("Node", vec![property("Node", "parentId", 0)]),
            record("FocusData", vec![property("FocusData", "parentId", 1)]),
            record("StateMachine", Vec::new()),
            record(
                "StateMachineListener",
                vec![property("StateMachineListener", "targetId", 1)],
            ),
            record(
                "ListenerInputType",
                vec![property(
                    "ListenerInputType",
                    "listenerTypeValue",
                    RuntimeListenerType::Down as u64,
                )],
            ),
            record(
                "ListenerInputTypeKeyboard",
                vec![property(
                    "ListenerInputTypeKeyboard",
                    "listenerTypeValue",
                    RuntimeListenerType::Keyboard as u64,
                )],
            ),
        ])
        .expect("mixed pointer and keyboard listener records import");
        let graph = GraphFile::from_runtime_file(&file).expect("mixed listener graph builds");
        let authored = file.artboard_state_machine_graphs(0);
        let listener = runtime_state_machine_listener(
            &file,
            graph.artboards.first().expect("artboard graph"),
            &authored[0].inputs,
            &[],
            &authored[0].listeners[0],
            &action_owners(&file, authored[0].object.id),
        )
        .expect("non-pointer channels retain the listener occurrence");

        assert!(listener.has_listener(RuntimeListenerType::Down));
        assert!(listener.has_listener(RuntimeListenerType::Keyboard));
        assert!(listener.hit_paths.is_empty());
    }

    #[test]
    fn imported_listener_retains_every_view_model_input_in_authored_order() {
        let file = RuntimeFile::from_authoring_records(vec![
            record("Backboard", Vec::new()),
            record("Artboard", Vec::new()),
            record("StateMachine", Vec::new()),
            record(
                "StateMachineListener",
                vec![property("StateMachineListener", "targetId", 0)],
            ),
            record(
                "ListenerInputTypeViewModel",
                vec![
                    property(
                        "ListenerInputTypeViewModel",
                        "listenerTypeValue",
                        RuntimeListenerType::ViewModel as u64,
                    ),
                    bytes_property("ListenerInputTypeViewModel", "viewModelPathIds", vec![0, 0]),
                ],
            ),
            record(
                "ListenerInputTypeViewModel",
                vec![
                    property(
                        "ListenerInputTypeViewModel",
                        "listenerTypeValue",
                        RuntimeListenerType::ViewModel as u64,
                    ),
                    bytes_property("ListenerInputTypeViewModel", "viewModelPathIds", vec![0, 1]),
                ],
            ),
            record(
                "ListenerInputTypeViewModel",
                vec![property(
                    "ListenerInputTypeViewModel",
                    "listenerTypeValue",
                    RuntimeListenerType::ViewModel as u64,
                )],
            ),
        ])
        .expect("view-model listener authoring records import");
        let graph = GraphFile::from_runtime_file(&file).expect("view-model listener graph builds");
        let authored = file.artboard_state_machine_graphs(0);
        let listener = runtime_state_machine_listener(
            &file,
            graph.artboards.first().expect("artboard graph"),
            &authored[0].inputs,
            &[],
            &authored[0].listeners[0],
            &action_owners(&file, authored[0].object.id),
        )
        .expect("view-model listener is retained");

        assert!(listener.has_listener(RuntimeListenerType::ViewModel));
        assert_eq!(listener.view_model_input_types.len(), 3);
        assert_eq!(listener.view_model_input_types[0].global_id, 4);
        assert_eq!(
            listener.view_model_input_types[0].source_path(),
            Some((0, [0].as_slice()))
        );
        assert_eq!(listener.view_model_input_types[1].global_id, 5);
        assert_eq!(
            listener.view_model_input_types[1].source_path(),
            Some((0, [1].as_slice()))
        );
        assert_eq!(listener.view_model_input_types[2].global_id, 6);
        assert_eq!(listener.view_model_input_types[2].source_path(), None);
    }

    #[test]
    fn listeners_without_a_recognized_dispatch_type_remain_ordered_inert_owners() {
        let file = RuntimeFile::from_authoring_records(vec![
            record("Backboard", Vec::new()),
            record("Artboard", Vec::new()),
            record("StateMachine", Vec::new()),
            record("StateMachineBool", Vec::new()),
            record(
                "StateMachineListener",
                vec![property("StateMachineListener", "targetId", 0)],
            ),
            record(
                "ListenerBoolChange",
                vec![property("ListenerBoolChange", "inputId", 0)],
            ),
            record(
                "StateMachineListenerSingle",
                vec![
                    property("StateMachineListenerSingle", "targetId", 0),
                    property(
                        "StateMachineListenerSingle",
                        "listenerTypeValue",
                        u32::MAX as u64,
                    ),
                ],
            ),
            record(
                "ListenerBoolChange",
                vec![property("ListenerBoolChange", "inputId", 0)],
            ),
        ])
        .expect("inert listener authoring records import");
        let graph = GraphFile::from_runtime_file(&file).expect("inert listener graph builds");
        let artboard = ArtboardInstance::from_graph_with_artboards(
            &file,
            graph.artboards.first().expect("inert listener artboard"),
            &graph.artboards,
        )
        .expect("inert listener artboard instantiates");
        let state_machine = artboard.state_machine(0).expect("state machine");

        assert_eq!(
            state_machine.listeners.len(),
            2,
            "StateMachineListener::import retains every authored owner even when it has no recognized dispatch type"
        );
        assert!(state_machine.listeners[0].listener_types.is_empty());
        assert!(!state_machine.listeners[0].is_single);
        assert_eq!(state_machine.listeners[0].listener_actions.len(), 1);
        assert!(state_machine.listeners[1].listener_types.is_empty());
        assert!(state_machine.listeners[1].is_single);
        assert_eq!(state_machine.listeners[1].listener_actions.len(), 1);

        let cold_clone = state_machine.clone();
        assert_eq!(cold_clone.listeners.len(), 2);
        assert!(!cold_clone.listeners[0].is_single);
        assert!(cold_clone.listeners[1].is_single);
        assert_eq!(cold_clone.listeners[0].listener_actions.len(), 1);
        assert_eq!(cold_clone.listeners[1].listener_actions.len(), 1);
    }

    fn bytes_property(type_name: &str, name: &str, value: Vec<u8>) -> AuthoringProperty {
        AuthoringProperty {
            key: property_key_for_name(type_name, name)
                .unwrap_or_else(|| panic!("missing property {type_name}.{name}")),
            value: AuthoringValue::Bytes(value),
        }
    }

    const KEY_PHASE_DOWN_FOR_TEST: u64 = 1;
    const KEY_PHASE_UP_FOR_TEST: u64 = 4;
}
