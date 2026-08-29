use crate::mechanical_port::source::{
    core::CoreHandle,
    input::{
        gamepad_snapshot::{GamepadInputChange, GamepadSnapshot},
        standard_gamepad::{StandardGamepadAxis, StandardGamepadButton},
    },
    math::vec2d::Vec2D,
};

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ListenerInvocationKind {
    Pointer = 0,
    Keyboard = 1,
    TextInput = 2,
    Focus = 3,
    ReportedEvent = 4,
    ViewModelChange = 5,
    None = 6,
    GamepadConnected = 7,
    GamepadEvent = 8,
    GamepadDisconnected = 9,
    Semantic = 10,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PointerInvocation {
    pub position: Vec2D,
    pub previous_position: Vec2D,
    pub pointer_id: i32,
    pub hit_event: u32,
    pub time_stamp: f32,
}
#[derive(Clone, Debug, PartialEq)]
pub struct KeyboardInvocation {
    pub key: u32,
    pub modifiers: u32,
    pub is_pressed: bool,
    pub is_repeat: bool,
}
#[derive(Clone, Debug, PartialEq)]
pub struct TextInputInvocation {
    pub text: String,
}
#[derive(Clone, Debug, PartialEq)]
pub struct FocusInvocation {
    pub listener_index: usize,
    pub is_focus: bool,
}
#[derive(Clone, Debug, PartialEq)]
pub struct ReportedEventInvocation {
    pub event: CoreHandle,
    pub delay_seconds: f32,
}
#[derive(Clone, Debug, PartialEq)]
pub struct ViewModelChangeInvocation {
    pub listener_index: usize,
}
#[derive(Clone, Debug, PartialEq)]
pub struct NoneInvocation;
#[derive(Clone, Debug, PartialEq)]
pub struct GamepadConnectedInvocation {
    pub snapshot: GamepadSnapshot,
}
#[derive(Clone, Debug, PartialEq)]
pub struct GamepadEventInvocation {
    pub full_state: GamepadSnapshot,
    pub change: GamepadInputChange,
    pub standard_button: Option<StandardGamepadButton>,
    pub standard_axis: Option<StandardGamepadAxis>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct GamepadDisconnectedInvocation {
    pub device_id: i32,
}
#[derive(Clone, Debug, PartialEq)]
pub struct SemanticInvocation {
    pub listener_index: usize,
    pub action_type: u8,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ListenerInvocationStorage {
    Pointer(PointerInvocation),
    Keyboard(KeyboardInvocation),
    TextInput(TextInputInvocation),
    Focus(FocusInvocation),
    ReportedEvent(ReportedEventInvocation),
    ViewModelChange(ViewModelChangeInvocation),
    None(NoneInvocation),
    GamepadConnected(GamepadConnectedInvocation),
    GamepadEvent(GamepadEventInvocation),
    GamepadDisconnected(GamepadDisconnectedInvocation),
    Semantic(SemanticInvocation),
}
#[derive(Clone, Debug, PartialEq)]
pub struct ListenerInvocation {
    storage: ListenerInvocationStorage,
}

impl ListenerInvocation {
    pub fn to_script_invocation(&self) -> crate::state_machine::ScriptListenerInvocation {
        use crate::mechanical_port::source::input::gamepad_snapshot::{
            GamepadInputChangeKind, GamepadMappingKind,
        };
        use crate::state_machine::{
            ScriptGamepadInputChange, ScriptGamepadMappingKind, ScriptGamepadSnapshot,
            ScriptListenerInvocation as Invocation, ScriptPointerEventKind,
        };
        fn snapshot(value: &GamepadSnapshot) -> ScriptGamepadSnapshot {
            ScriptGamepadSnapshot {
                device_id: value.device_id,
                button_mask: value.button_mask,
                button_values: value.button_values.clone(),
                axes: value.axes.clone(),
                mapping: match value.mapping {
                    GamepadMappingKind::Standard => ScriptGamepadMappingKind::Standard,
                    GamepadMappingKind::Unknown => ScriptGamepadMappingKind::Unknown,
                },
            }
        }
        match &self.storage {
            ListenerInvocationStorage::Pointer(value) => Invocation::Pointer {
                pointer_id: value.pointer_id,
                x: value.position.x,
                y: value.position.y,
                previous_x: value.previous_position.x,
                previous_y: value.previous_position.y,
                event: match value.hit_event {
                    0 => ScriptPointerEventKind::Enter,
                    1 => ScriptPointerEventKind::Exit,
                    2 => ScriptPointerEventKind::Down,
                    3 => ScriptPointerEventKind::Up,
                    4 => ScriptPointerEventKind::Move,
                    6 => ScriptPointerEventKind::Click,
                    9 => ScriptPointerEventKind::DragStart,
                    10 => ScriptPointerEventKind::DragEnd,
                    12 => ScriptPointerEventKind::Drag,
                    _ => panic!("a pointer invocation carries a pointer listener type"),
                },
                timestamp_seconds: value.time_stamp,
            },
            ListenerInvocationStorage::Keyboard(value) => Invocation::Keyboard {
                key: value.key,
                modifiers: value.modifiers,
                is_pressed: value.is_pressed,
                is_repeat: value.is_repeat,
            },
            ListenerInvocationStorage::TextInput(value) => Invocation::TextInput {
                text: value.text.clone(),
            },
            ListenerInvocationStorage::Focus(value) => Invocation::Focus {
                listener_index: value.listener_index,
                is_focus: value.is_focus,
            },
            ListenerInvocationStorage::ReportedEvent(value) => {
                let artboard = value
                    .event
                    .with(|event| {
                        event
                            .as_component()
                            .and_then(|event| event.artboard_handle())
                    })
                    .flatten()
                    .expect("a reported event retains its owning artboard");
                let index = artboard
                    .with_downcast::<crate::mechanical_port::source::artboard::Artboard, _>(
                        |artboard| artboard.object_index(&value.event),
                    )
                    .expect("an event's owning artboard remains live");
                Invocation::ReportedEvent {
                    event_local_index: usize::try_from(index)
                        .expect("a reported event belongs to its artboard"),
                    seconds_delay: value.delay_seconds,
                }
            }
            ListenerInvocationStorage::ViewModelChange(value) => Invocation::ViewModelChange {
                listener_index: value.listener_index,
            },
            ListenerInvocationStorage::None(_) => Invocation::None,
            ListenerInvocationStorage::GamepadConnected(value) => Invocation::GamepadConnected {
                snapshot: snapshot(&value.snapshot),
            },
            ListenerInvocationStorage::GamepadEvent(value) => Invocation::GamepadEvent {
                full_state: snapshot(&value.full_state),
                change: match value.change.kind {
                    GamepadInputChangeKind::Button => ScriptGamepadInputChange::Button {
                        index: value.change.index,
                        value: value.change.value,
                    },
                    GamepadInputChangeKind::Axis => ScriptGamepadInputChange::Axis {
                        index: value.change.index,
                        value: value.change.value,
                    },
                },
                standard_button_intent: value.standard_button.map(|button| button as u32),
                standard_axis_intent: value.standard_axis.map(|axis| axis as u32),
            },
            ListenerInvocationStorage::GamepadDisconnected(value) => {
                Invocation::GamepadDisconnected {
                    device_id: value.device_id,
                }
            }
            ListenerInvocationStorage::Semantic(value) => Invocation::Semantic {
                listener_index: value.listener_index,
                action_type: value.action_type as u32,
            },
        }
    }

    pub fn pointer(
        position: Vec2D,
        previous_position: Vec2D,
        pointer_id: i32,
        hit_event: u32,
        time_stamp: f32,
    ) -> Self {
        Self {
            storage: ListenerInvocationStorage::Pointer(PointerInvocation {
                position,
                previous_position,
                pointer_id,
                hit_event,
                time_stamp,
            }),
        }
    }
    pub fn keyboard(key: u32, modifiers: u32, is_pressed: bool, is_repeat: bool) -> Self {
        Self {
            storage: ListenerInvocationStorage::Keyboard(KeyboardInvocation {
                key,
                modifiers,
                is_pressed,
                is_repeat,
            }),
        }
    }
    pub fn text_input(text: String) -> Self {
        Self {
            storage: ListenerInvocationStorage::TextInput(TextInputInvocation { text }),
        }
    }
    pub fn focus(listener_index: usize, is_focus: bool) -> Self {
        Self {
            storage: ListenerInvocationStorage::Focus(FocusInvocation {
                listener_index,
                is_focus,
            }),
        }
    }
    pub fn reported_event(event: CoreHandle, delay_seconds: f32) -> Self {
        Self {
            storage: ListenerInvocationStorage::ReportedEvent(ReportedEventInvocation {
                event,
                delay_seconds,
            }),
        }
    }
    pub fn view_model_change(listener_index: usize) -> Self {
        Self {
            storage: ListenerInvocationStorage::ViewModelChange(ViewModelChangeInvocation {
                listener_index,
            }),
        }
    }
    pub fn none() -> Self {
        Self {
            storage: ListenerInvocationStorage::None(NoneInvocation),
        }
    }
    pub fn gamepad_connected(snapshot: &GamepadSnapshot) -> Self {
        Self {
            storage: ListenerInvocationStorage::GamepadConnected(GamepadConnectedInvocation {
                snapshot: snapshot.clone(),
            }),
        }
    }
    pub fn gamepad_event(value: GamepadEventInvocation) -> Self {
        Self {
            storage: ListenerInvocationStorage::GamepadEvent(value),
        }
    }
    pub fn gamepad_disconnected(device_id: i32) -> Self {
        Self {
            storage: ListenerInvocationStorage::GamepadDisconnected(
                GamepadDisconnectedInvocation { device_id },
            ),
        }
    }
    pub fn semantic(listener_index: usize, action_type: u8) -> Self {
        Self {
            storage: ListenerInvocationStorage::Semantic(SemanticInvocation {
                listener_index,
                action_type,
            }),
        }
    }
    pub fn kind(&self) -> ListenerInvocationKind {
        match self.storage {
            ListenerInvocationStorage::Pointer(_) => ListenerInvocationKind::Pointer,
            ListenerInvocationStorage::Keyboard(_) => ListenerInvocationKind::Keyboard,
            ListenerInvocationStorage::TextInput(_) => ListenerInvocationKind::TextInput,
            ListenerInvocationStorage::Focus(_) => ListenerInvocationKind::Focus,
            ListenerInvocationStorage::ReportedEvent(_) => ListenerInvocationKind::ReportedEvent,
            ListenerInvocationStorage::ViewModelChange(_) => {
                ListenerInvocationKind::ViewModelChange
            }
            ListenerInvocationStorage::None(_) => ListenerInvocationKind::None,
            ListenerInvocationStorage::GamepadConnected(_) => {
                ListenerInvocationKind::GamepadConnected
            }
            ListenerInvocationStorage::GamepadEvent(_) => ListenerInvocationKind::GamepadEvent,
            ListenerInvocationStorage::GamepadDisconnected(_) => {
                ListenerInvocationKind::GamepadDisconnected
            }
            ListenerInvocationStorage::Semantic(_) => ListenerInvocationKind::Semantic,
        }
    }
    pub fn storage(&self) -> &ListenerInvocationStorage {
        &self.storage
    }
    pub fn as_pointer(&self) -> Option<&PointerInvocation> {
        if let ListenerInvocationStorage::Pointer(value) = &self.storage {
            Some(value)
        } else {
            None
        }
    }
    pub fn as_keyboard(&self) -> Option<&KeyboardInvocation> {
        if let ListenerInvocationStorage::Keyboard(value) = &self.storage {
            Some(value)
        } else {
            None
        }
    }
    pub fn as_text_input(&self) -> Option<&TextInputInvocation> {
        if let ListenerInvocationStorage::TextInput(value) = &self.storage {
            Some(value)
        } else {
            None
        }
    }
    pub fn as_focus(&self) -> Option<&FocusInvocation> {
        if let ListenerInvocationStorage::Focus(value) = &self.storage {
            Some(value)
        } else {
            None
        }
    }
    pub fn as_reported_event(&self) -> Option<&ReportedEventInvocation> {
        if let ListenerInvocationStorage::ReportedEvent(value) = &self.storage {
            Some(value)
        } else {
            None
        }
    }
    pub fn as_view_model_change(&self) -> Option<&ViewModelChangeInvocation> {
        if let ListenerInvocationStorage::ViewModelChange(value) = &self.storage {
            Some(value)
        } else {
            None
        }
    }
    pub fn as_none(&self) -> Option<&NoneInvocation> {
        if let ListenerInvocationStorage::None(value) = &self.storage {
            Some(value)
        } else {
            None
        }
    }
    pub fn as_gamepad_connected(&self) -> Option<&GamepadConnectedInvocation> {
        if let ListenerInvocationStorage::GamepadConnected(value) = &self.storage {
            Some(value)
        } else {
            None
        }
    }
    pub fn as_gamepad_event(&self) -> Option<&GamepadEventInvocation> {
        if let ListenerInvocationStorage::GamepadEvent(value) = &self.storage {
            Some(value)
        } else {
            None
        }
    }
    pub fn as_gamepad_disconnected(&self) -> Option<&GamepadDisconnectedInvocation> {
        if let ListenerInvocationStorage::GamepadDisconnected(value) = &self.storage {
            Some(value)
        } else {
            None
        }
    }
    pub fn as_semantic(&self) -> Option<&SemanticInvocation> {
        if let ListenerInvocationStorage::Semantic(value) = &self.storage {
            Some(value)
        } else {
            None
        }
    }
}
