use luaur_rt::{
    AnyUserData, Lua, Result, UserData, UserDataFields, UserDataMethods, Value, Vector as LuaVector,
};
use nuxie_runtime::{
    ScriptGamepadInputChange, ScriptGamepadMappingKind, ScriptGamepadSnapshot,
    ScriptListenerActionMethod, ScriptListenerInvocation, ScriptPointerEventKind,
};
use std::{cell::Cell, rc::Rc};

pub(super) fn install_pointer_event_global(lua: &Lua) -> Result<()> {
    let pointer_event = lua.create_table();
    pointer_event.set(
        "new",
        lua.create_function(|lua, (id, position): (i64, LuaVector)| {
            lua.create_userdata(ScriptedPointerEvent::new(
                id as u8,
                position.x(),
                position.y(),
            ))
        })?,
    )?;
    lua.globals().set("PointerEvent", pointer_event)
}

#[derive(Clone)]
struct ScriptedInvocation(ScriptListenerInvocation);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ScriptedPointerHitResult {
    None,
    Hit,
    HitOpaque,
}

pub(super) type ScriptedPointerHitResultHandle = Rc<Cell<ScriptedPointerHitResult>>;

#[derive(Clone)]
struct ScriptedPointerEvent {
    id: u8,
    x: f32,
    y: f32,
    previous_x: f32,
    previous_y: f32,
    timestamp_seconds: f32,
    event: Option<ScriptPointerEventKind>,
    hit_result: ScriptedPointerHitResultHandle,
}

#[derive(Clone, Copy)]
struct ScriptedReportedEventInvocation {
    _event_local_index: usize,
    delay_seconds: f32,
}

struct ScriptedNoneInvocation;

#[derive(Clone, Copy)]
struct ScriptedKeyboardInvocation {
    key: u32,
    modifiers: u32,
    is_pressed: bool,
    is_repeat: bool,
}

#[derive(Clone)]
struct ScriptedTextInputInvocation(String);

#[derive(Clone, Copy)]
struct ScriptedFocusInvocation(bool);

struct ScriptedViewModelChangeInvocation;

#[derive(Clone)]
struct ScriptedGamepadConnected(ScriptGamepadSnapshot);

#[derive(Clone)]
struct ScriptedGamepadEvent {
    snapshot: ScriptGamepadSnapshot,
    change: ScriptGamepadInputChange,
    standard_button_intent: Option<u32>,
    standard_axis_intent: Option<u32>,
}

#[derive(Clone, Copy)]
struct ScriptedGamepadDisconnected(i32);

pub(super) fn listener_action_argument(
    lua: &Lua,
    method: ScriptListenerActionMethod,
    invocation: &ScriptListenerInvocation,
) -> Result<AnyUserData> {
    match method {
        ScriptListenerActionMethod::PerformAction => {
            lua.create_userdata(ScriptedInvocation(invocation.clone()))
        }
        ScriptListenerActionMethod::Perform => pointer_event_argument(lua, invocation).map(|v| v.0),
    }
}

/// Create the exact pointer userdata plus the result cell that the native
/// callback owner reads after Lua returns. The listener-action caller drops
/// the cell (matching C++, which ignores `hit()` there). A pointer-dispatch
/// caller can retain the cell through the callback and fold the resulting
/// tri-state into its hit traversal, as `HitScriptedDrawable` does natively.
pub(super) fn pointer_event_argument(
    lua: &Lua,
    invocation: &ScriptListenerInvocation,
) -> Result<(AnyUserData, ScriptedPointerHitResultHandle)> {
    let event = ScriptedPointerEvent::from_invocation(invocation);
    let result = event.hit_result.clone();
    Ok((lua.create_userdata(event)?, result))
}

pub(super) fn scripted_drawable_pointer_argument(
    lua: &Lua,
    pointer_id: i32,
    local_x: f32,
    local_y: f32,
) -> Result<(AnyUserData, ScriptedPointerHitResultHandle)> {
    let event = ScriptedPointerEvent::new(pointer_id as u8, local_x, local_y);
    let result = event.hit_result.clone();
    Ok((lua.create_userdata(event)?, result))
}

pub(super) fn scripted_drawable_input_argument(
    lua: &Lua,
    invocation: &ScriptListenerInvocation,
) -> Result<Option<AnyUserData>> {
    match invocation {
        ScriptListenerInvocation::Keyboard {
            key,
            modifiers,
            is_pressed,
            is_repeat,
        } => lua
            .create_userdata(ScriptedKeyboardInvocation {
                key: *key,
                modifiers: *modifiers,
                is_pressed: *is_pressed,
                is_repeat: *is_repeat,
            })
            .map(Some),
        ScriptListenerInvocation::TextInput { text } => lua
            .create_userdata(ScriptedTextInputInvocation(text.clone()))
            .map(Some),
        ScriptListenerInvocation::GamepadConnected { snapshot } => lua
            .create_userdata(ScriptedGamepadConnected(snapshot.clone()))
            .map(Some),
        ScriptListenerInvocation::GamepadEvent {
            full_state,
            change,
            standard_button_intent,
            standard_axis_intent,
        } => lua
            .create_userdata(ScriptedGamepadEvent {
                snapshot: full_state.clone(),
                change: *change,
                standard_button_intent: *standard_button_intent,
                standard_axis_intent: *standard_axis_intent,
            })
            .map(Some),
        ScriptListenerInvocation::GamepadDisconnected { device_id } => lua
            .create_userdata(ScriptedGamepadDisconnected(*device_id))
            .map(Some),
        ScriptListenerInvocation::Pointer { .. }
        | ScriptListenerInvocation::Focus { .. }
        | ScriptListenerInvocation::ReportedEvent { .. }
        | ScriptListenerInvocation::ViewModelChange { .. }
        | ScriptListenerInvocation::None
        | ScriptListenerInvocation::Semantic { .. } => Ok(None),
    }
}

impl ScriptedPointerEvent {
    fn new(id: u8, x: f32, y: f32) -> Self {
        Self {
            id,
            x,
            y,
            previous_x: 0.0,
            previous_y: 0.0,
            timestamp_seconds: 0.0,
            // `ScriptedPointerEvent`'s C++ constructor defaults the raw
            // ListenerType to `enter` (0). The non-pointer legacy listener
            // placeholder explicitly passes -1 and remains `unknown`.
            event: Some(ScriptPointerEventKind::Enter),
            hit_result: Rc::new(Cell::new(ScriptedPointerHitResult::None)),
        }
    }

    fn from_invocation(invocation: &ScriptListenerInvocation) -> Self {
        match invocation {
            ScriptListenerInvocation::Pointer {
                x,
                y,
                previous_x,
                previous_y,
                pointer_id,
                event,
                timestamp_seconds,
            } => Self {
                // Pinned Lua `ScriptedPointerEvent` stores the runtime's
                // signed pointer id in `uint8_t`, including modulo conversion
                // for out-of-range embedder ids.
                id: *pointer_id as u8,
                x: *x,
                y: *y,
                previous_x: *previous_x,
                previous_y: *previous_y,
                timestamp_seconds: *timestamp_seconds,
                event: Some(*event),
                hit_result: Rc::new(Cell::new(ScriptedPointerHitResult::None)),
            },
            ScriptListenerInvocation::ReportedEvent { .. }
            | ScriptListenerInvocation::Keyboard { .. }
            | ScriptListenerInvocation::TextInput { .. }
            | ScriptListenerInvocation::Focus { .. }
            | ScriptListenerInvocation::ViewModelChange { .. }
            | ScriptListenerInvocation::None
            | ScriptListenerInvocation::GamepadConnected { .. }
            | ScriptListenerInvocation::GamepadEvent { .. }
            | ScriptListenerInvocation::GamepadDisconnected { .. }
            | ScriptListenerInvocation::Semantic { .. } => Self {
                id: 0,
                x: 0.0,
                y: 0.0,
                previous_x: 0.0,
                previous_y: 0.0,
                timestamp_seconds: 0.0,
                event: None,
                hit_result: Rc::new(Cell::new(ScriptedPointerHitResult::None)),
            },
        }
    }

    fn event_name(&self) -> &'static str {
        match self.event {
            Some(ScriptPointerEventKind::Enter) => "pointerEnter",
            Some(ScriptPointerEventKind::Exit) => "pointerExit",
            Some(ScriptPointerEventKind::Down) => "pointerDown",
            Some(ScriptPointerEventKind::Move) => "pointerMove",
            Some(ScriptPointerEventKind::Up) => "pointerUp",
            Some(ScriptPointerEventKind::Click) => "click",
            Some(ScriptPointerEventKind::Drag) => "pointerDrag",
            Some(ScriptPointerEventKind::DragStart | ScriptPointerEventKind::DragEnd) | None => {
                "unknown"
            }
        }
    }
}

impl UserData for ScriptedPointerEvent {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("id", |_, this| Ok(this.id));
        fields.add_field_method_get("position", |_, this| {
            Ok(LuaVector::new(this.x, this.y, 0.0))
        });
        fields.add_field_method_get("previousPosition", |_, this| {
            Ok(LuaVector::new(this.previous_x, this.previous_y, 0.0))
        });
        fields.add_field_method_get("type", |_, this| Ok(this.event_name()));
        fields.add_field_method_get("timeStamp", |_, this| Ok(this.timestamp_seconds));
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("hit", |_, this, value: Value| {
            // Pinned `pointer_event_hit` distinguishes an actual Lua boolean
            // from every other value. `true` is a transparent hit; `false`,
            // nil/missing, and non-booleans are opaque.
            this.hit_result.set(match value {
                Value::Boolean(true) => ScriptedPointerHitResult::Hit,
                _ => ScriptedPointerHitResult::HitOpaque,
            });
            Ok(())
        });
    }
}

impl UserData for ScriptedReportedEventInvocation {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("delaySeconds", |_, this| Ok(this.delay_seconds));
    }
}

impl UserData for ScriptedNoneInvocation {}
impl UserData for ScriptedViewModelChangeInvocation {}

impl UserData for ScriptedKeyboardInvocation {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("key", |_, this| Ok(this.key));
        fields.add_field_method_get("shift", |_, this| Ok(this.modifiers & 1 != 0));
        fields.add_field_method_get("control", |_, this| Ok(this.modifiers & 2 != 0));
        fields.add_field_method_get("alt", |_, this| Ok(this.modifiers & 4 != 0));
        fields.add_field_method_get("meta", |_, this| Ok(this.modifiers & 8 != 0));
        fields.add_field_method_get("phase", |_, this| {
            Ok(if !this.is_pressed {
                "up"
            } else if this.is_repeat {
                "repeat"
            } else {
                "down"
            })
        });
    }
}

impl UserData for ScriptedTextInputInvocation {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("text", |_, this| {
            // Pinned C++ stores the owned std::string but exposes it through
            // `lua_pushstring(text.c_str())`, so Lua observes only the prefix
            // before an embedded NUL.
            Ok(this.0.split('\0').next().unwrap_or_default().to_owned())
        });
    }
}

impl UserData for ScriptedFocusInvocation {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("isFocus", |_, this| Ok(this.0));
    }
}

impl UserData for ScriptedGamepadConnected {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        add_gamepad_fields(fields, |this| &this.0);
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        add_gamepad_methods(methods, |this| &this.0);
    }
}

impl UserData for ScriptedGamepadEvent {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        add_gamepad_fields(fields, |this| &this.snapshot);
        fields.add_field_method_get("changeKind", |_, this| {
            Ok(match this.change {
                ScriptGamepadInputChange::Button { .. } => "button",
                ScriptGamepadInputChange::Axis { .. } => "axis",
            })
        });
        fields.add_field_method_get("changeIndex", |_, this| {
            Ok(match this.change {
                ScriptGamepadInputChange::Button { index, .. }
                | ScriptGamepadInputChange::Axis { index, .. } => u32::from(index) + 1,
            })
        });
        fields.add_field_method_get("changeValue", |_, this| {
            Ok(match this.change {
                ScriptGamepadInputChange::Button { value, .. }
                | ScriptGamepadInputChange::Axis { value, .. } => value,
            })
        });
        fields.add_field_method_get("hasStandardButtonIntent", |_, this| {
            Ok(this.standard_button_intent.is_some())
        });
        fields.add_field_method_get("hasStandardAxisIntent", |_, this| {
            Ok(this.standard_axis_intent.is_some())
        });
        fields.add_field_method_get("intentButton", |_, this| {
            Ok(this.standard_button_intent.map(standard_button_label))
        });
        fields.add_field_method_get("intentAxis", |_, this| {
            Ok(this.standard_axis_intent.map(standard_axis_label))
        });
    }

    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        add_gamepad_methods(methods, |this| &this.snapshot);
    }
}

impl UserData for ScriptedGamepadDisconnected {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("deviceId", |_, this| Ok(this.0));
    }
}

fn add_gamepad_fields<T, F>(fields: &mut F, snapshot: fn(&T) -> &ScriptGamepadSnapshot)
where
    T: 'static,
    F: UserDataFields<T>,
{
    fields.add_field_method_get("deviceId", move |_, this| Ok(snapshot(this).device_id));
    fields.add_field_method_get("buttonMask", move |_, this| {
        Ok(snapshot(this).button_mask as f64)
    });
    fields.add_field_method_get("buttons", move |lua, this| {
        let table = lua.create_table();
        for (index, value) in snapshot(this).button_values.iter().enumerate() {
            table.raw_set(index + 1, *value)?;
        }
        Ok(table)
    });
    fields.add_field_method_get("axes", move |lua, this| {
        let table = lua.create_table();
        for (index, value) in snapshot(this).axes.iter().enumerate() {
            table.raw_set(index + 1, *value)?;
        }
        Ok(table)
    });
    fields.add_field_method_get("gamepadMapping", move |_, this| {
        Ok(match snapshot(this).mapping {
            ScriptGamepadMappingKind::Standard => 0,
            ScriptGamepadMappingKind::Unknown => 1,
        })
    });
    fields.add_field_method_get("mapping", move |_, this| {
        Ok(match snapshot(this).mapping {
            ScriptGamepadMappingKind::Standard => "standard",
            ScriptGamepadMappingKind::Unknown => "unknown",
        })
    });
    fields.add_field_method_get("isStandardMapping", move |_, this| {
        Ok(snapshot(this).mapping == ScriptGamepadMappingKind::Standard)
    });
    for (name, index) in [
        ("south", 0),
        ("east", 1),
        ("west", 2),
        ("north", 3),
        ("leftShoulder", 4),
        ("rightShoulder", 5),
        ("leftTriggerPressed", 6),
        ("rightTriggerPressed", 7),
        ("gamepadBack", 8),
        ("gamepadForward", 9),
        ("leftStickButton", 10),
        ("rightStickButton", 11),
        ("dpadUp", 12),
        ("dpadDown", 13),
        ("dpadLeft", 14),
        ("dpadRight", 15),
        ("start", 16),
    ] {
        fields.add_field_method_get(name, move |_, this| {
            Ok(standard_button_pressed(snapshot(this), index))
        });
    }
    fields.add_field_method_get("leftStick", move |_, this| {
        let snapshot = snapshot(this);
        let (x, y) = if snapshot.mapping == ScriptGamepadMappingKind::Standard {
            (axis_value(snapshot, 0), axis_value(snapshot, 1))
        } else {
            (0.0, 0.0)
        };
        Ok(LuaVector::new(x, y, 0.0))
    });
    fields.add_field_method_get("rightStick", move |_, this| {
        let snapshot = snapshot(this);
        let (x, y) = if snapshot.mapping == ScriptGamepadMappingKind::Standard {
            (axis_value(snapshot, 2), axis_value(snapshot, 3))
        } else {
            (0.0, 0.0)
        };
        Ok(LuaVector::new(x, y, 0.0))
    });
    fields.add_field_method_get("leftTrigger", move |_, this| {
        let snapshot = snapshot(this);
        Ok(if snapshot.mapping == ScriptGamepadMappingKind::Standard {
            axis_value(snapshot, 4)
        } else {
            0.0
        })
    });
    fields.add_field_method_get("rightTrigger", move |_, this| {
        let snapshot = snapshot(this);
        Ok(if snapshot.mapping == ScriptGamepadMappingKind::Standard {
            axis_value(snapshot, 5)
        } else {
            0.0
        })
    });
}

fn add_gamepad_methods<T, M>(methods: &mut M, snapshot: fn(&T) -> &ScriptGamepadSnapshot)
where
    T: 'static,
    M: UserDataMethods<T>,
{
    methods.add_method("buttonPressed", move |_, this, index: i64| {
        if !(1..=64).contains(&index) {
            return Ok(false);
        }
        Ok(snapshot(this).button_mask & (1_u64 << (index - 1)) != 0)
    });
    methods.add_method("buttonValue", move |_, this, index: i64| {
        Ok(index
            .checked_sub(1)
            .and_then(|index| usize::try_from(index).ok())
            .and_then(|index| snapshot(this).button_values.get(index))
            .copied()
            .unwrap_or(0.0))
    });
    methods.add_method("axis", move |_, this, index: i64| {
        Ok(index
            .checked_sub(1)
            .and_then(|index| usize::try_from(index).ok())
            .and_then(|index| snapshot(this).axes.get(index))
            .copied()
            .unwrap_or(0.0))
    });
}

fn standard_button_pressed(snapshot: &ScriptGamepadSnapshot, index: u32) -> bool {
    snapshot.mapping == ScriptGamepadMappingKind::Standard
        && snapshot.button_mask & (1_u64 << index) != 0
}

fn axis_value(snapshot: &ScriptGamepadSnapshot, index: usize) -> f32 {
    snapshot.axes.get(index).copied().unwrap_or(0.0)
}

fn standard_button_label(value: u32) -> &'static str {
    [
        "south",
        "east",
        "west",
        "north",
        "leftShoulder",
        "rightShoulder",
        "leftTrigger",
        "rightTrigger",
        "back",
        "forward",
        "leftStick",
        "rightStick",
        "dpadUp",
        "dpadDown",
        "dpadLeft",
        "dpadRight",
        "start",
    ]
    .get(value as usize)
    .copied()
    .unwrap_or("unknown")
}

fn standard_axis_label(value: u32) -> &'static str {
    [
        "leftX",
        "leftY",
        "rightX",
        "rightY",
        "leftTriggerAxis",
        "rightTriggerAxis",
    ]
    .get(value as usize)
    .copied()
    .unwrap_or("unknown")
}

impl UserData for ScriptedInvocation {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("isPointerEvent", |_, this, ()| {
            Ok(matches!(this.0, ScriptListenerInvocation::Pointer { .. }))
        });
        methods.add_method("isReportedEvent", |_, this, ()| {
            Ok(matches!(
                this.0,
                ScriptListenerInvocation::ReportedEvent { .. }
            ))
        });
        methods.add_method("isNone", |_, this, ()| {
            Ok(matches!(this.0, ScriptListenerInvocation::None))
        });
        methods.add_method("isKeyboardEvent", |_, this, ()| {
            Ok(matches!(this.0, ScriptListenerInvocation::Keyboard { .. }))
        });
        methods.add_method("isTextInput", |_, this, ()| {
            Ok(matches!(this.0, ScriptListenerInvocation::TextInput { .. }))
        });
        methods.add_method("isFocus", |_, this, ()| {
            Ok(matches!(this.0, ScriptListenerInvocation::Focus { .. }))
        });
        methods.add_method("isViewModelChange", |_, this, ()| {
            Ok(matches!(
                this.0,
                ScriptListenerInvocation::ViewModelChange { .. }
            ))
        });
        methods.add_method("isGamepadConnected", |_, this, ()| {
            Ok(matches!(
                this.0,
                ScriptListenerInvocation::GamepadConnected { .. }
            ))
        });
        methods.add_method("isGamepadEvent", |_, this, ()| {
            Ok(matches!(
                this.0,
                ScriptListenerInvocation::GamepadEvent { .. }
            ))
        });
        methods.add_method("isGamepadDisconnected", |_, this, ()| {
            Ok(matches!(
                this.0,
                ScriptListenerInvocation::GamepadDisconnected { .. }
            ))
        });

        methods.add_method("asPointerEvent", |lua, this, ()| match &this.0 {
            invocation @ ScriptListenerInvocation::Pointer { .. } => lua
                .create_userdata(ScriptedPointerEvent::from_invocation(invocation))
                .map(Some),
            _ => Ok(None),
        });
        methods.add_method("asReportedEvent", |lua, this, ()| match this.0 {
            ScriptListenerInvocation::ReportedEvent {
                event_local_index,
                seconds_delay,
            } => lua
                .create_userdata(ScriptedReportedEventInvocation {
                    _event_local_index: event_local_index,
                    delay_seconds: seconds_delay,
                })
                .map(Some),
            _ => Ok(None),
        });
        methods.add_method("asNone", |lua, this, ()| {
            if matches!(this.0, ScriptListenerInvocation::None) {
                lua.create_userdata(ScriptedNoneInvocation).map(Some)
            } else {
                Ok(None)
            }
        });
        methods.add_method("asKeyboardEvent", |lua, this, ()| match this.0 {
            ScriptListenerInvocation::Keyboard {
                key,
                modifiers,
                is_pressed,
                is_repeat,
            } => lua
                .create_userdata(ScriptedKeyboardInvocation {
                    key,
                    modifiers,
                    is_pressed,
                    is_repeat,
                })
                .map(Some),
            _ => Ok(None),
        });
        methods.add_method("asTextInput", |lua, this, ()| match &this.0 {
            ScriptListenerInvocation::TextInput { text } => lua
                .create_userdata(ScriptedTextInputInvocation(text.clone()))
                .map(Some),
            _ => Ok(None),
        });
        methods.add_method("asFocus", |lua, this, ()| match this.0 {
            ScriptListenerInvocation::Focus { is_focus, .. } => lua
                .create_userdata(ScriptedFocusInvocation(is_focus))
                .map(Some),
            _ => Ok(None),
        });
        methods.add_method("asViewModelChange", |lua, this, ()| {
            if matches!(this.0, ScriptListenerInvocation::ViewModelChange { .. }) {
                lua.create_userdata(ScriptedViewModelChangeInvocation)
                    .map(Some)
            } else {
                Ok(None)
            }
        });
        methods.add_method("asGamepadConnected", |lua, this, ()| match &this.0 {
            ScriptListenerInvocation::GamepadConnected { snapshot } => lua
                .create_userdata(ScriptedGamepadConnected(snapshot.clone()))
                .map(Some),
            _ => Ok(None),
        });
        methods.add_method("asGamepadEvent", |lua, this, ()| match &this.0 {
            ScriptListenerInvocation::GamepadEvent {
                full_state,
                change,
                standard_button_intent,
                standard_axis_intent,
            } => lua
                .create_userdata(ScriptedGamepadEvent {
                    snapshot: full_state.clone(),
                    change: *change,
                    standard_button_intent: *standard_button_intent,
                    standard_axis_intent: *standard_axis_intent,
                })
                .map(Some),
            _ => Ok(None),
        });
        methods.add_method("asGamepadDisconnected", |lua, this, ()| match this.0 {
            ScriptListenerInvocation::GamepadDisconnected { device_id } => lua
                .create_userdata(ScriptedGamepadDisconnected(device_id))
                .map(Some),
            _ => Ok(None),
        });
    }
}

#[cfg(all(test, feature = "compiler"))]
mod tests {
    use super::*;

    fn install_invocation(lua: &Lua, invocation: ScriptListenerInvocation) {
        let value =
            listener_action_argument(lua, ScriptListenerActionMethod::PerformAction, &invocation)
                .expect("invocation userdata");
        lua.globals().set("invocation", value).expect("global");
    }

    #[test]
    fn pointer_wrapper_matches_cpp_uint8_id_and_owned_payload() {
        let lua = Lua::new();
        install_invocation(
            &lua,
            ScriptListenerInvocation::Pointer {
                pointer_id: 300,
                x: 4.0,
                y: 5.0,
                previous_x: 1.0,
                previous_y: 2.0,
                event: ScriptPointerEventKind::Move,
                timestamp_seconds: 7.5,
            },
        );

        let (is_pointer, id, event, timestamp): (bool, u32, String, f32) = lua
            .load(
                r#"
                local pointer = invocation:asPointerEvent()
                return invocation:isPointerEvent(),
                    pointer.id,
                    pointer.type,
                    pointer.timeStamp
                "#,
            )
            .eval()
            .expect("read pointer invocation");
        assert!(is_pointer);
        assert_eq!(id, 44);
        assert_eq!(event, "pointerMove");
        assert_eq!(timestamp, 7.5);
    }

    #[test]
    fn pointer_hit_propagates_the_cpp_tristate_out_of_the_lua_callback() {
        let lua = Lua::new();
        let event = ScriptedPointerEvent::from_invocation(&ScriptListenerInvocation::Pointer {
            pointer_id: 1,
            x: 10.0,
            y: 20.0,
            previous_x: 0.0,
            previous_y: 0.0,
            event: ScriptPointerEventKind::Down,
            timestamp_seconds: 0.0,
        });
        let hit_result = event.hit_result.clone();
        lua.globals()
            .set(
                "event",
                lua.create_userdata(event).expect("pointer userdata"),
            )
            .expect("pointer global");

        lua.load("event:hit(true)").exec().expect("transparent hit");
        assert_eq!(hit_result.get(), ScriptedPointerHitResult::Hit);

        lua.load("event:hit(false)").exec().expect("opaque hit");
        assert_eq!(hit_result.get(), ScriptedPointerHitResult::HitOpaque);

        lua.load("event:hit()")
            .exec()
            .expect("missing argument defaults opaque");
        assert_eq!(hit_result.get(), ScriptedPointerHitResult::HitOpaque);

        lua.load("event:hit('not a boolean')")
            .exec()
            .expect("non-boolean argument defaults opaque");
        assert_eq!(hit_result.get(), ScriptedPointerHitResult::HitOpaque);
    }

    #[test]
    fn owned_text_and_gamepad_payloads_survive_source_drop() {
        let lua = Lua::new();
        install_invocation(
            &lua,
            ScriptListenerInvocation::TextInput {
                text: String::from("owned"),
            },
        );
        assert_eq!(
            lua.load("return invocation:asTextInput().text")
                .eval::<String>()
                .expect("owned text"),
            "owned"
        );

        install_invocation(
            &lua,
            ScriptListenerInvocation::GamepadEvent {
                full_state: ScriptGamepadSnapshot {
                    device_id: 9,
                    button_mask: 2,
                    button_values: vec![0.0, 0.75],
                    axes: vec![-0.5],
                    mapping: ScriptGamepadMappingKind::Standard,
                },
                change: ScriptGamepadInputChange::Button {
                    index: 1,
                    value: 0.75,
                },
                standard_button_intent: Some(1),
                standard_axis_intent: None,
            },
        );
        let (device, pressed, value, axis, intent): (i32, bool, f32, f32, String) = lua
            .load(
                r#"
                local event = invocation:asGamepadEvent()
                return event.deviceId,
                    event:buttonPressed(2),
                    event:buttonValue(2),
                    event:axis(1),
                    event.intentButton
                "#,
            )
            .eval()
            .expect("owned gamepad");
        assert_eq!(
            (device, pressed, value, axis, intent),
            (9, true, 0.75, -0.5, "east".into())
        );
    }

    #[test]
    fn semantic_variant_is_retained_but_not_exposed_by_pinned_lua_api() {
        let lua = Lua::new();
        install_invocation(
            &lua,
            ScriptListenerInvocation::Semantic {
                listener_index: 4,
                action_type: 2,
            },
        );
        let values: (bool, bool, bool, bool, bool, bool, bool, bool, bool, bool) = lua
            .load(
                r#"
                return invocation:isPointerEvent(),
                    invocation:isKeyboardEvent(),
                    invocation:isTextInput(),
                    invocation:isFocus(),
                    invocation:isReportedEvent(),
                    invocation:isViewModelChange(),
                    invocation:isNone(),
                    invocation:isGamepadConnected(),
                    invocation:isGamepadEvent(),
                    invocation:isGamepadDisconnected()
                "#,
            )
            .eval()
            .expect("semantic classification");
        assert_eq!(
            values,
            (
                false, false, false, false, false, false, false, false, false, false
            )
        );
    }
}
