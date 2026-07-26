use super::listener_types::RuntimeListenerType;

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
    listener: &nuxie_binary::RuntimeStateMachineListener<'_>,
) -> Option<(usize, Vec<usize>)> {
    if runtime_listener_single_type(listener) != Some(RuntimeListenerType::ViewModel) {
        return None;
    }
    let encoded = listener.object.id_list_property("viewModelPathIds")?;
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
