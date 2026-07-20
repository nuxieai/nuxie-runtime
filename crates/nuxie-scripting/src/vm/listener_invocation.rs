use luaur_rt::{
    AnyUserData, Error, Lua, Result, UserData, UserDataFields, UserDataMethods, Vector as LuaVector,
};
use nuxie_runtime::{ScriptListenerActionMethod, ScriptListenerInvocation, ScriptPointerEventKind};

#[derive(Clone)]
struct ScriptedInvocation(ScriptListenerInvocation);

#[derive(Clone, Copy)]
struct ScriptedPointerEvent {
    id: i32,
    x: f32,
    y: f32,
    previous_x: f32,
    previous_y: f32,
    timestamp_seconds: f32,
    event: Option<ScriptPointerEventKind>,
}

#[derive(Clone, Copy)]
struct ScriptedReportedEventInvocation {
    _event_local_index: usize,
    delay_seconds: f32,
}

struct ScriptedNoneInvocation;

pub(super) fn listener_action_argument(
    lua: &Lua,
    method: ScriptListenerActionMethod,
    invocation: &ScriptListenerInvocation,
) -> Result<AnyUserData> {
    match method {
        ScriptListenerActionMethod::PerformAction => {
            lua.create_userdata(ScriptedInvocation(invocation.clone()))
        }
        ScriptListenerActionMethod::Perform => {
            lua.create_userdata(ScriptedPointerEvent::from_invocation(invocation))
        }
    }
}

impl ScriptedPointerEvent {
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
                id: *pointer_id,
                x: *x,
                y: *y,
                previous_x: *previous_x,
                previous_y: *previous_y,
                timestamp_seconds: *timestamp_seconds,
                event: Some(*event),
            },
            ScriptListenerInvocation::ReportedEvent { .. } | ScriptListenerInvocation::None => {
                Self {
                    id: 0,
                    x: 0.0,
                    y: 0.0,
                    previous_x: 0.0,
                    previous_y: 0.0,
                    timestamp_seconds: 0.0,
                    event: None,
                }
            }
        }
    }

    fn event_name(self) -> &'static str {
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
        methods.add_method_mut("hit", |_, _, _: Option<bool>| Ok(()));
    }
}

impl UserData for ScriptedReportedEventInvocation {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("delaySeconds", |_, this| Ok(this.delay_seconds));
    }
}

impl UserData for ScriptedNoneInvocation {}

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
        for name in [
            "isKeyboardEvent",
            "isTextInput",
            "isFocus",
            "isViewModelChange",
            "isGamepadConnected",
            "isGamepadEvent",
            "isGamepadDisconnected",
        ] {
            methods.add_method(name, |_, _, ()| Ok(false));
        }

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
        for name in [
            "asKeyboardEvent",
            "asTextInput",
            "asFocus",
            "asViewModelChange",
            "asGamepadConnected",
            "asGamepadEvent",
            "asGamepadDisconnected",
        ] {
            methods.add_method(name, |_, _, ()| Ok::<Option<AnyUserData>, Error>(None));
        }
    }
}
