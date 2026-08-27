use crate::mechanical_port::source::{
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

pub struct PointerInvocation {
    pub position: Vec2D,
    pub previous_position: Vec2D,
    pub pointer_id: i32,
    pub hit_event: u32,
    pub time_stamp: f32,
}
pub struct KeyboardInvocation {
    pub key: u32,
    pub modifiers: u32,
    pub is_pressed: bool,
    pub is_repeat: bool,
}
pub struct TextInputInvocation {
    pub text: String,
}
pub struct FocusInvocation {
    pub group: *mut (),
    pub is_focus: bool,
}
pub struct ReportedEventInvocation {
    pub reported_event: *mut (),
    pub delay_seconds: f32,
}
pub struct ViewModelChangeInvocation {
    pub source: *mut (),
}
pub struct NoneInvocation;
pub struct GamepadConnectedInvocation {
    pub snapshot: GamepadSnapshot,
}
pub struct GamepadEventInvocation {
    pub full_state: GamepadSnapshot,
    pub change: GamepadInputChange,
    pub has_standard_button_intent: bool,
    pub standard_button: StandardGamepadButton,
    pub has_standard_axis_intent: bool,
    pub standard_axis: StandardGamepadAxis,
}
pub struct GamepadDisconnectedInvocation {
    pub device_id: i32,
}
pub struct SemanticInvocation {
    pub group: *mut (),
    pub action_type: u8,
}

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
pub struct ListenerInvocation {
    storage: ListenerInvocationStorage,
}

impl ListenerInvocation {
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
    pub fn focus<T>(group: *mut T, is_focus: bool) -> Self {
        Self {
            storage: ListenerInvocationStorage::Focus(FocusInvocation {
                group: group.cast(),
                is_focus,
            }),
        }
    }
    pub fn reported_event<T>(reported_event: *mut T, delay_seconds: f32) -> Self {
        Self {
            storage: ListenerInvocationStorage::ReportedEvent(ReportedEventInvocation {
                reported_event: reported_event.cast(),
                delay_seconds,
            }),
        }
    }
    pub fn view_model_change<T>(source: *mut T) -> Self {
        Self {
            storage: ListenerInvocationStorage::ViewModelChange(ViewModelChangeInvocation {
                source: source.cast(),
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
    pub fn semantic<T>(group: *mut T, action_type: u8) -> Self {
        Self {
            storage: ListenerInvocationStorage::Semantic(SemanticInvocation {
                group: group.cast(),
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
