use super::*;

pub(super) fn imports_successfully(
    _object: &RuntimeObject,
    definition: &'static Definition,
    context: &ImportContext,
) -> Option<bool> {
    if definition.is_a("BlendAnimation") {
        return Some(
            context.latest(ImportStackKey::Artboard) && context.latest(ImportStackKey::LayerState),
        );
    }
    definition
        .is_a("LayerState")
        .then(|| context.latest(ImportStackKey::StateMachineLayer))
}

pub(super) fn update_context(definition: &'static Definition, context: &mut ImportContext) {
    if definition.is_a("LayerState") {
        context.make_latest(ImportStackKey::LayerState);
        context.latest_layer_state_accepts_blend_animation = definition.is_a("BlendState");
    }
}
