use super::RuntimeScheduledListenerAction;
use super::listener_types::{RuntimeListenerInputTypeKeyboard, RuntimeListenerType};
use super::state_machine_listener_single::{
    runtime_listener_single_event_local_indices, runtime_listener_single_type,
    runtime_listener_single_view_model_property_path,
};
use crate::ArtboardInstance;
use crate::properties::property_key_for_name;
use nuxie_binary::{RuntimeFile, RuntimeObject};
use nuxie_graph::{ArtboardGraph, ParametricPathNode};

#[derive(Debug, Clone)]
pub(crate) struct RuntimeStateMachineListener {
    pub(crate) target_local_id: usize,
    pub(crate) listener_types: Vec<RuntimeListenerType>,
    pub(crate) event_local_indices: Vec<usize>,
    pub(crate) view_model_index: Option<usize>,
    pub(crate) view_model_property_path: Option<Vec<usize>>,
    pub(crate) keyboard_input_types: Vec<RuntimeListenerInputTypeKeyboard>,
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
    state_machine_data_binds: &[&RuntimeObject],
    listener: &nuxie_binary::RuntimeStateMachineListener<'_>,
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
                )
        })
        .collect::<Vec<_>>();
    if listener_types.is_empty() {
        return None;
    }

    let hit_paths = if listener_types
        .iter()
        .any(|listener_type| listener_type.is_pointer_hit())
    {
        let hit_paths = runtime_listener_hit_paths(graph, target_local_id);
        if hit_paths.is_empty() {
            return None;
        }
        hit_paths
    } else {
        Vec::new()
    };
    let event_local_indices = runtime_listener_event_local_indices(listener);
    let (view_model_index, view_model_property_path) =
        runtime_listener_view_model_property_path(listener)
            .map(|(view_model_index, property_path)| (Some(view_model_index), Some(property_path)))
            .unwrap_or((None, None));

    Some(RuntimeStateMachineListener {
        target_local_id,
        listener_types,
        event_local_indices,
        view_model_index,
        view_model_property_path,
        keyboard_input_types: runtime_listener_input_type_keyboards(listener),
        hit_paths,
        listener_actions: listener
            .actions
            .iter()
            .filter_map(|action| {
                RuntimeScheduledListenerAction::from_imported(
                    file,
                    state_machine_data_binds,
                    action,
                )
            })
            .collect(),
    })
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

fn runtime_listener_view_model_property_path(
    listener: &nuxie_binary::RuntimeStateMachineListener<'_>,
) -> Option<(usize, Vec<usize>)> {
    if listener.object.type_name == "StateMachineListenerSingle" {
        return runtime_listener_single_view_model_property_path(listener);
    }

    let encoded = listener
        .listener_input_types
        .iter()
        .find(|input_type| {
            input_type
                .uint_property("listenerTypeValue")
                .and_then(RuntimeListenerType::from_value)
                == Some(RuntimeListenerType::ViewModel)
        })
        .and_then(|input_type| input_type.id_list_property("viewModelPathIds"))?;
    let (view_model_index, property_path) = encoded.split_first()?;
    let view_model_index = usize::try_from(*view_model_index).ok()?;
    let property_path = property_path
        .iter()
        .copied()
        .map(usize::try_from)
        .collect::<Result<Vec<_>, _>>()
        .ok()?;
    (!property_path.is_empty()).then_some((view_model_index, property_path))
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
            &[],
            &authored[0].listeners[0],
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

    const KEY_PHASE_DOWN_FOR_TEST: u64 = 1;
    const KEY_PHASE_UP_FOR_TEST: u64 = 4;
}
