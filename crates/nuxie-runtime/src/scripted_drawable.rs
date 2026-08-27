use crate::components::Mat2D;
use crate::script_asset::RuntimeScriptImplementedMethods;
use crate::scripting::ScriptMethod;
use crate::state_machine::RuntimeListenerType;

/// `ScriptedDrawable::hitComponents`.
pub(crate) fn has_hit_component(methods: RuntimeScriptImplementedMethods) -> bool {
    methods.listens_to_pointer_events()
}

/// `HitScriptedDrawable::handlesEvent` followed by `methodName`.
pub(crate) fn method_for_event(
    methods: RuntimeScriptImplementedMethods,
    can_hit: bool,
    hit_event: RuntimeListenerType,
) -> Option<ScriptMethod> {
    if can_hit {
        match hit_event {
            RuntimeListenerType::Down if methods.wants_pointer_down() => {
                Some(ScriptMethod::PointerDown)
            }
            RuntimeListenerType::Up if methods.wants_pointer_up() => Some(ScriptMethod::PointerUp),
            RuntimeListenerType::DragStart | RuntimeListenerType::DragEnd => None,
            RuntimeListenerType::Down | RuntimeListenerType::Up => None,
            _ if methods.wants_pointer_move() => Some(ScriptMethod::PointerMove),
            _ => None,
        }
    } else if methods.wants_pointer_exit() {
        Some(ScriptMethod::PointerExit)
    } else {
        None
    }
}

/// `ScriptedDrawable::worldToLocal`.
pub(crate) fn world_to_local(world_transform: Mat2D, world: (f32, f32)) -> Option<(f32, f32)> {
    world_transform
        .invert()
        .map(|to_mounted_artboard| to_mounted_artboard.transform_point(world.0, world.1))
}
