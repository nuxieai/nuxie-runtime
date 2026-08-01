use super::*;

pub(super) fn imports_successfully(
    object: &RuntimeObject,
    definition: &'static Definition,
    context: &ImportContext,
) -> Option<bool> {
    if definition.is_a("StateMachineFireAction") {
        return Some(context.latest(ImportStackKey::StateMachineLayerComponent));
    }
    (definition.is_a("ListenerAction") && !listener_action_parent_kind_is_listener(object))
        .then(|| listener_action_imports_successfully(object, context))
}

pub(super) fn update_context(definition: &'static Definition, context: &mut ImportContext) {
    if definition.is_a("StateMachineLayerComponent") {
        context.make_latest(ImportStackKey::StateMachineLayerComponent);
    }
}
