use super::RuntimeScheduledListenerAction;
use super::listener_types::RuntimeListenerType;
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
                    RuntimeListenerType::Event | RuntimeListenerType::ViewModel
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
