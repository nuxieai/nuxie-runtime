use super::listener_types::{RuntimeListenerType, RuntimeListenerViewModelPath};
use nuxie_binary::RuntimeFile;

pub(super) fn runtime_listener_single_type(
    listener: &nuxie_binary::RuntimeStateMachineListener<'_>,
) -> Option<RuntimeListenerType> {
    RuntimeListenerType::from_value(
        listener
            .object
            .uint_property("listenerTypeValue")
            .unwrap_or(0),
    )
}

pub(super) fn runtime_listener_single_event_local_indices(
    listener: &nuxie_binary::RuntimeStateMachineListener<'_>,
) -> Vec<usize> {
    if runtime_listener_single_type(listener) != Some(RuntimeListenerType::Event) {
        return Vec::new();
    }
    listener
        .object
        .uint_property("eventId")
        .and_then(|event_id| usize::try_from(event_id).ok())
        .into_iter()
        .collect()
}

pub(super) fn runtime_listener_single_view_model_path(
    file: &RuntimeFile,
    listener: &nuxie_binary::RuntimeStateMachineListener<'_>,
) -> Option<RuntimeListenerViewModelPath> {
    if runtime_listener_single_type(listener) != Some(RuntimeListenerType::ViewModel) {
        return None;
    }
    file.data_bind_path_for_referencer_object(listener.object)
        .and_then(RuntimeListenerViewModelPath::from_data_bind_path)
}
