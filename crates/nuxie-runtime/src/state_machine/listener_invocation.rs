/// One owned state-machine listener invocation.
///
/// This is the direct Rust counterpart of pinned C++
/// `ListenerInvocationStorage`: every alternative owns the values needed by a
/// retained scripted `Invocation` wrapper, so callbacks never borrow transient
/// embedder strings or gamepad buffers.
#[derive(Debug, Clone, PartialEq)]
pub enum ScriptListenerInvocation {
    Pointer {
        pointer_id: i32,
        x: f32,
        y: f32,
        previous_x: f32,
        previous_y: f32,
        event: ScriptPointerEventKind,
        timestamp_seconds: f32,
    },
    Keyboard {
        key: u32,
        modifiers: u32,
        is_pressed: bool,
        is_repeat: bool,
    },
    TextInput {
        text: String,
    },
    Focus {
        listener_index: usize,
        is_focus: bool,
    },
    ReportedEvent {
        event_local_index: usize,
        seconds_delay: f32,
    },
    ViewModelChange {
        listener_index: usize,
    },
    None,
    GamepadConnected {
        snapshot: ScriptGamepadSnapshot,
    },
    GamepadEvent {
        full_state: ScriptGamepadSnapshot,
        change: ScriptGamepadInputChange,
        standard_button_intent: Option<u32>,
        standard_axis_intent: Option<u32>,
    },
    GamepadDisconnected {
        device_id: i32,
    },
    Semantic {
        listener_index: usize,
        action_type: u32,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ScriptGamepadSnapshot {
    pub device_id: i32,
    pub button_mask: u64,
    pub button_values: Vec<f32>,
    pub axes: Vec<f32>,
    pub mapping: ScriptGamepadMappingKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptGamepadMappingKind {
    Standard,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScriptGamepadInputChange {
    Button { index: u8, value: f32 },
    Axis { index: u8, value: f32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptPointerEventKind {
    Enter,
    Exit,
    Down,
    Up,
    Move,
    Click,
    DragStart,
    DragEnd,
    Drag,
}

impl ScriptPointerEventKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Enter => "pointerEnter",
            Self::Exit => "pointerExit",
            Self::Down => "pointerDown",
            Self::Up => "pointerUp",
            Self::Move => "pointerMove",
            Self::Click => "click",
            Self::DragStart => "pointerDragStart",
            Self::DragEnd => "pointerDragEnd",
            Self::Drag => "pointerDrag",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snapshot() -> ScriptGamepadSnapshot {
        ScriptGamepadSnapshot {
            device_id: 7,
            button_mask: 3,
            button_values: vec![0.25, 1.0],
            axes: vec![-0.5],
            mapping: ScriptGamepadMappingKind::Standard,
        }
    }

    #[test]
    fn invocation_storage_owns_every_cpp_alternative_and_clone_payload() {
        let mut caller_text = String::from("owned");
        let mut caller_snapshot = snapshot();
        let invocations = vec![
            ScriptListenerInvocation::Pointer {
                pointer_id: 1,
                x: 2.0,
                y: 3.0,
                previous_x: 1.0,
                previous_y: 1.5,
                event: ScriptPointerEventKind::Drag,
                timestamp_seconds: 4.0,
            },
            ScriptListenerInvocation::Keyboard {
                key: 65,
                modifiers: 3,
                is_pressed: true,
                is_repeat: false,
            },
            ScriptListenerInvocation::TextInput {
                text: caller_text.clone(),
            },
            ScriptListenerInvocation::Focus {
                listener_index: 2,
                is_focus: true,
            },
            ScriptListenerInvocation::ReportedEvent {
                event_local_index: 3,
                seconds_delay: 0.25,
            },
            ScriptListenerInvocation::ViewModelChange { listener_index: 4 },
            ScriptListenerInvocation::None,
            ScriptListenerInvocation::GamepadConnected {
                snapshot: caller_snapshot.clone(),
            },
            ScriptListenerInvocation::GamepadEvent {
                full_state: caller_snapshot.clone(),
                change: ScriptGamepadInputChange::Button {
                    index: 1,
                    value: 1.0,
                },
                standard_button_intent: Some(1),
                standard_axis_intent: None,
            },
            ScriptListenerInvocation::GamepadDisconnected { device_id: 7 },
            ScriptListenerInvocation::Semantic {
                listener_index: 5,
                action_type: 2,
            },
        ];
        let retained = invocations.clone();

        caller_text.clear();
        caller_snapshot.button_values.clear();
        caller_snapshot.axes.clear();

        assert_eq!(retained, invocations);
        assert!(matches!(
            &retained[2],
            ScriptListenerInvocation::TextInput { text } if text == "owned"
        ));
        assert!(matches!(
            &retained[7],
            ScriptListenerInvocation::GamepadConnected { snapshot }
                if snapshot.button_values == [0.25, 1.0] && snapshot.axes == [-0.5]
        ));
    }
}
