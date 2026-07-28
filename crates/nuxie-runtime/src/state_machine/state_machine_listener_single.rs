use super::listener_types::RuntimeListenerType;
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

pub(super) fn runtime_listener_single_view_model_property_path(
    file: &RuntimeFile,
    listener: &nuxie_binary::RuntimeStateMachineListener<'_>,
) -> Option<(usize, Vec<usize>)> {
    if runtime_listener_single_type(listener) != Some(RuntimeListenerType::ViewModel) {
        return None;
    }
    let data_bind_path = file.data_bind_path_for_referencer_object(listener.object)?;
    let encoded = if data_bind_path.is_relative {
        data_bind_path.resolved_path_ids
    } else {
        data_bind_path.path_ids
    };
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
