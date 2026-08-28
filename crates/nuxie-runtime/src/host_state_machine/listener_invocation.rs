use nuxie_render_api::Vec2D;

/// Pinned C++ `ListenerInvocationKind` discriminants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
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

impl ScriptListenerInvocation {
    pub fn pointer(
        position: Vec2D,
        previous_position: Vec2D,
        pointer_id: i32,
        hit_event: ScriptPointerEventKind,
        time_stamp: f32,
    ) -> Self {
        Self::Pointer {
            pointer_id,
            x: position.x,
            y: position.y,
            previous_x: previous_position.x,
            previous_y: previous_position.y,
            event: hit_event,
            timestamp_seconds: time_stamp,
        }
    }

    pub fn keyboard(key: u32, modifiers: u32, is_pressed: bool, is_repeat: bool) -> Self {
        let key = crate::mechanical_port::source::input::focusable::Key::from_raw(key).raw();
        let modifiers =
            crate::mechanical_port::source::input::focusable::KeyModifiers::from_raw(modifiers)
                .bits();
        Self::Keyboard {
            key,
            modifiers,
            is_pressed,
            is_repeat,
        }
    }

    pub fn text_input(text: String) -> Self {
        Self::TextInput { text }
    }

    /// Stable listener indices replace pinned C++ listener-group pointers.
    pub fn focus(listener_index: usize, is_focus: bool) -> Self {
        Self::Focus {
            listener_index,
            is_focus,
        }
    }

    /// Stable local indices replace pinned C++ file-event pointers.
    pub fn reported_event(event_local_index: usize, delay_seconds: f32) -> Self {
        Self::ReportedEvent {
            event_local_index,
            seconds_delay: delay_seconds,
        }
    }

    /// Stable listener indices replace pinned C++ listener-view-model pointers.
    pub fn view_model_change(listener_index: usize) -> Self {
        Self::ViewModelChange { listener_index }
    }

    pub fn none() -> Self {
        Self::None
    }

    pub fn gamepad_connected(snapshot: ScriptGamepadSnapshot) -> Self {
        Self::GamepadConnected { snapshot }
    }

    pub fn gamepad_event(
        full_state: ScriptGamepadSnapshot,
        change: ScriptGamepadInputChange,
        standard_button_intent: Option<u32>,
        standard_axis_intent: Option<u32>,
    ) -> Self {
        Self::GamepadEvent {
            full_state,
            change,
            standard_button_intent,
            standard_axis_intent,
        }
    }

    pub fn gamepad_disconnected(device_id: i32) -> Self {
        Self::GamepadDisconnected { device_id }
    }

    /// Stable listener indices replace pinned C++ semantic-group pointers.
    pub fn semantic(listener_index: usize, action_type: u32) -> Self {
        Self::Semantic {
            listener_index,
            action_type,
        }
    }

    pub const fn kind(&self) -> ListenerInvocationKind {
        match self {
            Self::Pointer { .. } => ListenerInvocationKind::Pointer,
            Self::Keyboard { .. } => ListenerInvocationKind::Keyboard,
            Self::TextInput { .. } => ListenerInvocationKind::TextInput,
            Self::Focus { .. } => ListenerInvocationKind::Focus,
            Self::ReportedEvent { .. } => ListenerInvocationKind::ReportedEvent,
            Self::ViewModelChange { .. } => ListenerInvocationKind::ViewModelChange,
            Self::None => ListenerInvocationKind::None,
            Self::GamepadConnected { .. } => ListenerInvocationKind::GamepadConnected,
            Self::GamepadEvent { .. } => ListenerInvocationKind::GamepadEvent,
            Self::GamepadDisconnected { .. } => ListenerInvocationKind::GamepadDisconnected,
            Self::Semantic { .. } => ListenerInvocationKind::Semantic,
        }
    }

    pub const fn as_pointer(&self) -> Option<&Self> {
        if matches!(self, Self::Pointer { .. }) {
            Some(self)
        } else {
            None
        }
    }

    pub const fn as_keyboard(&self) -> Option<&Self> {
        if matches!(self, Self::Keyboard { .. }) {
            Some(self)
        } else {
            None
        }
    }

    pub const fn as_text_input(&self) -> Option<&Self> {
        if matches!(self, Self::TextInput { .. }) {
            Some(self)
        } else {
            None
        }
    }

    pub const fn as_focus(&self) -> Option<&Self> {
        if matches!(self, Self::Focus { .. }) {
            Some(self)
        } else {
            None
        }
    }

    pub const fn as_reported_event(&self) -> Option<&Self> {
        if matches!(self, Self::ReportedEvent { .. }) {
            Some(self)
        } else {
            None
        }
    }

    pub const fn as_view_model_change(&self) -> Option<&Self> {
        if matches!(self, Self::ViewModelChange { .. }) {
            Some(self)
        } else {
            None
        }
    }

    pub const fn as_none(&self) -> Option<&Self> {
        if matches!(self, Self::None) {
            Some(self)
        } else {
            None
        }
    }

    pub const fn as_gamepad_connected(&self) -> Option<&Self> {
        if matches!(self, Self::GamepadConnected { .. }) {
            Some(self)
        } else {
            None
        }
    }

    pub const fn as_gamepad_event(&self) -> Option<&Self> {
        if matches!(self, Self::GamepadEvent { .. }) {
            Some(self)
        } else {
            None
        }
    }

    pub const fn as_gamepad_disconnected(&self) -> Option<&Self> {
        if matches!(self, Self::GamepadDisconnected { .. }) {
            Some(self)
        } else {
            None
        }
    }

    pub const fn as_semantic(&self) -> Option<&Self> {
        if matches!(self, Self::Semantic { .. }) {
            Some(self)
        } else {
            None
        }
    }

    pub const fn storage(&self) -> &Self {
        self
    }
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
