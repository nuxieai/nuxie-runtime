#![cfg(feature = "rive_scripting")]

use crate::mechanical_port::source::{
    animation::listener_invocation::{ListenerInvocation, ListenerInvocationKind},
    input::{
        gamepad_snapshot::{GamepadInputChangeKind, GamepadMappingKind, GamepadSnapshot},
        standard_gamepad::{
            StandardGamepadAxis, StandardGamepadButton, standard_gamepad_axis_value,
            standard_gamepad_button_pressed,
        },
    },
    lua::rive_lua_libs::*,
    math::vec2d::Vec2D,
};

fn is_standard(snapshot: &GamepadSnapshot) -> bool {
    snapshot.mapping == GamepadMappingKind::Standard
}

fn push_standard_button(
    state: &mut LuaState,
    snapshot: &GamepadSnapshot,
    button: StandardGamepadButton,
) {
    state.push_boolean(
        is_standard(snapshot) && standard_gamepad_button_pressed(snapshot.button_mask, button),
    );
}

fn push_stick(
    state: &mut LuaState,
    snapshot: &GamepadSnapshot,
    x_axis: StandardGamepadAxis,
    y_axis: StandardGamepadAxis,
) {
    if is_standard(snapshot) {
        state.push_vector2(
            standard_gamepad_axis_value(&snapshot.axes, x_axis),
            standard_gamepad_axis_value(&snapshot.axes, y_axis),
        );
    } else {
        state.push_vector2(0.0, 0.0);
    }
}

fn push_trigger_axis(state: &mut LuaState, snapshot: &GamepadSnapshot, axis: StandardGamepadAxis) {
    let value = if is_standard(snapshot) {
        standard_gamepad_axis_value(&snapshot.axes, axis)
    } else {
        0.0
    };
    state.push_number(value as f64);
}

fn standard_button_label(button: StandardGamepadButton) -> &'static str {
    match button {
        StandardGamepadButton::South => "south",
        StandardGamepadButton::East => "east",
        StandardGamepadButton::West => "west",
        StandardGamepadButton::North => "north",
        StandardGamepadButton::LeftShoulder => "leftShoulder",
        StandardGamepadButton::RightShoulder => "rightShoulder",
        StandardGamepadButton::LeftTrigger => "leftTrigger",
        StandardGamepadButton::RightTrigger => "rightTrigger",
        StandardGamepadButton::Back => "back",
        StandardGamepadButton::Forward => "forward",
        StandardGamepadButton::LeftStick => "leftStick",
        StandardGamepadButton::RightStick => "rightStick",
        StandardGamepadButton::DpadUp => "dpadUp",
        StandardGamepadButton::DpadDown => "dpadDown",
        StandardGamepadButton::DpadLeft => "dpadLeft",
        StandardGamepadButton::DpadRight => "dpadRight",
        StandardGamepadButton::Start => "start",
    }
}

fn standard_axis_label(axis: StandardGamepadAxis) -> &'static str {
    match axis {
        StandardGamepadAxis::LeftX => "leftX",
        StandardGamepadAxis::LeftY => "leftY",
        StandardGamepadAxis::RightX => "rightX",
        StandardGamepadAxis::RightY => "rightY",
        StandardGamepadAxis::LeftTrigger => "leftTriggerAxis",
        StandardGamepadAxis::RightTrigger => "rightTriggerAxis",
    }
}

pub fn push_pointer_arg_for_perform(state: &mut LuaState, invocation: &ListenerInvocation) {
    if let Some(pointer) = invocation.as_pointer() {
        state.new_rive(ScriptedPointerEvent::new(
            pointer.pointer_id as u8,
            pointer.position,
            pointer.previous_position,
            pointer.hit_event as i32,
            pointer.time_stamp,
        ));
    } else {
        state.new_rive(ScriptedPointerEvent::new(
            0,
            Vec2D::new(0.0, 0.0),
            Vec2D::new(0.0, 0.0),
            -1,
            0.0,
        ));
    }
}

pub fn push_scripted_invocation(state: &mut LuaState, invocation: &ListenerInvocation) {
    state.new_rive(ScriptedInvocation::new(invocation.clone()));
}

fn scripted_invocation_namecall(state: &mut LuaState) -> i32 {
    let (_, atom) = state.namecall_atom();
    let invocation = state.to_rive::<ScriptedInvocation>(1).invocation();
    let expected_kind = match atom {
        LuaAtoms::IsPointerEvent => Some(ListenerInvocationKind::Pointer),
        LuaAtoms::IsKeyboardEvent => Some(ListenerInvocationKind::Keyboard),
        LuaAtoms::IsTextInput => Some(ListenerInvocationKind::TextInput),
        LuaAtoms::IsFocus => Some(ListenerInvocationKind::Focus),
        LuaAtoms::IsReportedEvent => Some(ListenerInvocationKind::ReportedEvent),
        LuaAtoms::IsViewModelChange => Some(ListenerInvocationKind::ViewModelChange),
        LuaAtoms::IsNone => Some(ListenerInvocationKind::None),
        LuaAtoms::IsGamepadConnected => Some(ListenerInvocationKind::GamepadConnected),
        LuaAtoms::IsGamepadEvent => Some(ListenerInvocationKind::GamepadEvent),
        LuaAtoms::IsGamepadDisconnected => Some(ListenerInvocationKind::GamepadDisconnected),
        _ => None,
    };
    if let Some(kind) = expected_kind {
        state.push_boolean(invocation.kind() == kind);
        return 1;
    }
    match atom {
        LuaAtoms::AsPointerEvent => {
            if let Some(value) = invocation.as_pointer() {
                state.new_rive(ScriptedPointerEvent::new(
                    value.pointer_id as u8,
                    value.position,
                    value.previous_position,
                    value.hit_event as i32,
                    value.time_stamp,
                ));
            } else {
                state.push_nil();
            }
        }
        LuaAtoms::AsKeyboardEvent => {
            if let Some(value) = invocation.as_keyboard() {
                state.new_rive(ScriptedKeyboardInvocation::new(
                    value.key,
                    value.modifiers,
                    value.is_pressed,
                    value.is_repeat,
                ));
            } else {
                state.push_nil();
            }
        }
        LuaAtoms::AsTextInput => {
            if let Some(value) = invocation.as_text_input() {
                state.new_rive(ScriptedTextInputInvocation::new(value.text.clone()));
            } else {
                state.push_nil();
            }
        }
        LuaAtoms::AsFocus => {
            if let Some(value) = invocation.as_focus() {
                state.new_rive(ScriptedFocusInvocation::new(value.is_focus));
            } else {
                state.push_nil();
            }
        }
        LuaAtoms::AsReportedEvent => {
            if let Some(value) = invocation.as_reported_event() {
                state.new_rive(ScriptedReportedEventInvocation::new(
                    value.reported_event,
                    value.delay_seconds,
                ));
            } else {
                state.push_nil();
            }
        }
        LuaAtoms::AsViewModelChange => {
            if invocation.as_view_model_change().is_some() {
                state.new_rive(ScriptedViewModelChangeInvocation::new());
            } else {
                state.push_nil();
            }
        }
        LuaAtoms::AsGamepadConnected => {
            if let Some(value) = invocation.as_gamepad_connected() {
                state.new_rive(ScriptedGamepadConnected::new(value.snapshot.clone()));
            } else {
                state.push_nil();
            }
        }
        LuaAtoms::AsGamepadEvent => {
            if let Some(value) = invocation.as_gamepad_event() {
                state.new_rive(ScriptedGamepadEvent::new(value.clone()));
            } else {
                state.push_nil();
            }
        }
        LuaAtoms::AsGamepadDisconnected => {
            if let Some(value) = invocation.as_gamepad_disconnected() {
                state.new_rive(ScriptedGamepadDisconnected::new(value.device_id));
            } else {
                state.push_nil();
            }
        }
        LuaAtoms::AsNone => {
            if invocation.as_none().is_some() {
                state.new_rive(ScriptedNoneInvocation::new());
            } else {
                state.push_nil();
            }
        }
        _ => {
            return state.error(format!(
                "{} is not a valid method of {}",
                state.check_string(1),
                ScriptedInvocation::LUA_NAME
            ));
        }
    }
    1
}

fn keyboard_index(state: &mut LuaState) -> i32 {
    let (key, atom) = state.to_string_atom(2);
    if key.is_none() {
        return state.type_error(2, state.type_name(LuaType::String));
    }
    let value = state.to_rive::<ScriptedKeyboardInvocation>(1);
    match atom {
        LuaAtoms::Key => state.push_integer(value.key as i64),
        LuaAtoms::Shift => state.push_boolean(value.modifiers.contains(KeyModifiers::SHIFT)),
        LuaAtoms::Control => state.push_boolean(value.modifiers.contains(KeyModifiers::CTRL)),
        LuaAtoms::Alt => state.push_boolean(value.modifiers.contains(KeyModifiers::ALT)),
        LuaAtoms::Meta => state.push_boolean(value.modifiers.contains(KeyModifiers::META)),
        LuaAtoms::Phase => state.push_string(if !value.is_pressed {
            "up"
        } else if value.is_repeat {
            "repeat"
        } else {
            "down"
        }),
        _ => {
            return state.error(format!(
                "{} is not a valid field of {}",
                state.check_string(1),
                ScriptedKeyboardInvocation::LUA_NAME
            ));
        }
    }
    1
}

fn text_input_index(state: &mut LuaState) -> i32 {
    let (_, atom) = state.to_string_atom(2);
    if atom == LuaAtoms::Text {
        state.push_string(state.to_rive::<ScriptedTextInputInvocation>(1).text());
        1
    } else {
        state.error(format!(
            "{} is not a valid field of {}",
            state.check_string(1),
            ScriptedTextInputInvocation::LUA_NAME
        ))
    }
}

fn focus_index(state: &mut LuaState) -> i32 {
    let (_, atom) = state.to_string_atom(2);
    if atom == LuaAtoms::IsFocus {
        state.push_boolean(state.to_rive::<ScriptedFocusInvocation>(1).is_focus);
        1
    } else {
        state.error(format!(
            "{} is not a valid field of {}",
            state.check_string(1),
            ScriptedFocusInvocation::LUA_NAME
        ))
    }
}

fn reported_event_index(state: &mut LuaState) -> i32 {
    let (_, atom) = state.to_string_atom(2);
    if atom == LuaAtoms::DelaySeconds {
        state.push_number(
            state
                .to_rive::<ScriptedReportedEventInvocation>(1)
                .delay_seconds as f64,
        );
        1
    } else {
        state.error(format!(
            "{} is not a valid field of {}",
            state.check_string(1),
            ScriptedReportedEventInvocation::LUA_NAME
        ))
    }
}

fn gamepad_snapshot_namecall(state: &mut LuaState, snapshot: &GamepadSnapshot) -> i32 {
    let (_, atom) = state.namecall_atom();
    let index = state.check_integer(2);
    match atom {
        LuaAtoms::ButtonPressed => state.push_boolean(
            (1..=64).contains(&index) && snapshot.button_mask & (1u64 << (index as u32 - 1)) != 0,
        ),
        LuaAtoms::ButtonValue => state.push_number(
            snapshot
                .button_values
                .get((index - 1) as usize)
                .copied()
                .unwrap_or(0.0) as f64,
        ),
        LuaAtoms::Axis => state.push_number(
            snapshot
                .axes
                .get((index - 1) as usize)
                .copied()
                .unwrap_or(0.0) as f64,
        ),
        _ => {
            return state.error(format!(
                "{} is not a valid method of gamepad data",
                state.check_string(1)
            ));
        }
    }
    1
}

fn gamepad_snapshot_index(state: &mut LuaState, atom: LuaAtoms, snapshot: &GamepadSnapshot) -> i32 {
    match atom {
        LuaAtoms::DeviceId => state.push_integer(snapshot.device_id as i64),
        LuaAtoms::ButtonMask => state.push_number(snapshot.button_mask as f64),
        LuaAtoms::Buttons => state.push_number_table(&snapshot.button_values),
        LuaAtoms::Axes => state.push_number_table(&snapshot.axes),
        LuaAtoms::GamepadMapping => state.push_integer(snapshot.mapping as i64),
        LuaAtoms::Mapping => state.push_string(if is_standard(snapshot) {
            "standard"
        } else {
            "unknown"
        }),
        LuaAtoms::IsStandardMapping => state.push_boolean(is_standard(snapshot)),
        LuaAtoms::West => push_standard_button(state, snapshot, StandardGamepadButton::West),
        LuaAtoms::South => push_standard_button(state, snapshot, StandardGamepadButton::South),
        LuaAtoms::North => push_standard_button(state, snapshot, StandardGamepadButton::North),
        LuaAtoms::East => push_standard_button(state, snapshot, StandardGamepadButton::East),
        LuaAtoms::LeftShoulder => {
            push_standard_button(state, snapshot, StandardGamepadButton::LeftShoulder)
        }
        LuaAtoms::RightShoulder => {
            push_standard_button(state, snapshot, StandardGamepadButton::RightShoulder)
        }
        LuaAtoms::GamepadBack => push_standard_button(state, snapshot, StandardGamepadButton::Back),
        LuaAtoms::GamepadForward => {
            push_standard_button(state, snapshot, StandardGamepadButton::Forward)
        }
        LuaAtoms::LeftStickButton => {
            push_standard_button(state, snapshot, StandardGamepadButton::LeftStick)
        }
        LuaAtoms::RightStickButton => {
            push_standard_button(state, snapshot, StandardGamepadButton::RightStick)
        }
        LuaAtoms::DpadUp => push_standard_button(state, snapshot, StandardGamepadButton::DpadUp),
        LuaAtoms::DpadDown => {
            push_standard_button(state, snapshot, StandardGamepadButton::DpadDown)
        }
        LuaAtoms::DpadLeft => {
            push_standard_button(state, snapshot, StandardGamepadButton::DpadLeft)
        }
        LuaAtoms::DpadRight => {
            push_standard_button(state, snapshot, StandardGamepadButton::DpadRight)
        }
        LuaAtoms::Start => push_standard_button(state, snapshot, StandardGamepadButton::Start),
        LuaAtoms::LeftTriggerPressed => {
            push_standard_button(state, snapshot, StandardGamepadButton::LeftTrigger)
        }
        LuaAtoms::RightTriggerPressed => {
            push_standard_button(state, snapshot, StandardGamepadButton::RightTrigger)
        }
        LuaAtoms::LeftStick => push_stick(
            state,
            snapshot,
            StandardGamepadAxis::LeftX,
            StandardGamepadAxis::LeftY,
        ),
        LuaAtoms::RightStick => push_stick(
            state,
            snapshot,
            StandardGamepadAxis::RightX,
            StandardGamepadAxis::RightY,
        ),
        LuaAtoms::LeftTrigger => {
            push_trigger_axis(state, snapshot, StandardGamepadAxis::LeftTrigger)
        }
        LuaAtoms::RightTrigger => {
            push_trigger_axis(state, snapshot, StandardGamepadAxis::RightTrigger)
        }
        _ => {
            return state.error(format!(
                "{} is not a valid field of gamepad state",
                state.check_string(1)
            ));
        }
    }
    1
}

fn gamepad_connected_index(state: &mut LuaState) -> i32 {
    let (_, atom) = state.to_string_atom(2);
    let snapshot = &state.to_rive::<ScriptedGamepadConnected>(1).snapshot;
    gamepad_snapshot_index(state, atom, snapshot)
}

fn gamepad_connected_namecall(state: &mut LuaState) -> i32 {
    let snapshot = &state.to_rive::<ScriptedGamepadConnected>(1).snapshot;
    gamepad_snapshot_namecall(state, snapshot)
}

fn gamepad_event_namecall(state: &mut LuaState) -> i32 {
    let snapshot = &state.to_rive::<ScriptedGamepadEvent>(1).data.full_state;
    gamepad_snapshot_namecall(state, snapshot)
}

fn gamepad_event_index(state: &mut LuaState) -> i32 {
    let (_, atom) = state.to_string_atom(2);
    let data = &state.to_rive::<ScriptedGamepadEvent>(1).data;
    match atom {
        LuaAtoms::ChangeKind => {
            state.push_string(if data.change.kind == GamepadInputChangeKind::Button {
                "button"
            } else {
                "axis"
            })
        }
        LuaAtoms::ChangeIndex => state.push_integer(data.change.index as i64 + 1),
        LuaAtoms::ChangeValue => state.push_number(data.change.value as f64),
        LuaAtoms::HasStandardButtonIntent => state.push_boolean(data.has_standard_button_intent),
        LuaAtoms::HasStandardAxisIntent => state.push_boolean(data.has_standard_axis_intent),
        LuaAtoms::IntentButton => {
            if data.has_standard_button_intent {
                state.push_string(standard_button_label(data.standard_button));
            } else {
                state.push_nil();
            }
        }
        LuaAtoms::IntentAxis => {
            if data.has_standard_axis_intent {
                state.push_string(standard_axis_label(data.standard_axis));
            } else {
                state.push_nil();
            }
        }
        _ => return gamepad_snapshot_index(state, atom, &data.full_state),
    }
    1
}

fn gamepad_disconnected_index(state: &mut LuaState) -> i32 {
    let (_, atom) = state.to_string_atom(2);
    if atom == LuaAtoms::DeviceId {
        state.push_integer(state.to_rive::<ScriptedGamepadDisconnected>(1).device_id as i64);
        1
    } else {
        state.error(format!(
            "{} is not a valid field of {}",
            state.check_string(1),
            ScriptedGamepadDisconnected::LUA_NAME
        ))
    }
}

fn register_index<T: LuaRive>(state: &mut LuaState, index: LuaFunction) {
    state.register_rive::<T>();
    state.push_function(index);
    state.set_field(-2, "__index");
    state.set_readonly(-1, true);
    state.pop(1);
}

pub fn register_listener_invocation_types(state: &mut LuaState) {
    state.register_rive::<ScriptedInvocation>();
    state.push_function(scripted_invocation_namecall);
    state.set_field(-2, "__namecall");
    state.set_readonly(-1, true);
    state.pop(1);

    register_index::<ScriptedKeyboardInvocation>(state, keyboard_index);
    for field in ["key", "shift", "control", "alt", "meta"] {
        state.register_invocation_direct_field::<ScriptedKeyboardInvocation>(field);
    }
    register_index::<ScriptedTextInputInvocation>(state, text_input_index);
    register_index::<ScriptedFocusInvocation>(state, focus_index);
    state.register_invocation_direct_field::<ScriptedFocusInvocation>("isFocus");
    register_index::<ScriptedReportedEventInvocation>(state, reported_event_index);
    state.register_invocation_direct_field::<ScriptedReportedEventInvocation>("delaySeconds");

    state.register_rive::<ScriptedViewModelChangeInvocation>();
    state.set_readonly(-1, true);
    state.pop(1);

    register_index::<ScriptedGamepadConnected>(state, gamepad_connected_index);
    state.push_function(gamepad_connected_namecall);
    state.set_field(-2, "__namecall");
    register_index::<ScriptedGamepadEvent>(state, gamepad_event_index);
    state.push_function(gamepad_event_namecall);
    state.set_field(-2, "__namecall");
    register_index::<ScriptedGamepadDisconnected>(state, gamepad_disconnected_index);

    state.register_rive::<ScriptedNoneInvocation>();
    state.set_readonly(-1, true);
    state.pop(1);
}
