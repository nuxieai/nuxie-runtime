/// Listener category decoded from the generated C++ `ListenerType` values.
///
/// The category is authored definition state. Per-device constraints and
/// occurrence-owned listener groups live in their matching sibling modules.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RuntimeListenerType {
    Enter,
    Exit,
    Down,
    Up,
    Move,
    Event,
    Click,
    ComponentProvided,
    TextInput,
    DragStart,
    DragEnd,
    ViewModel,
    Drag,
    Focus,
    Blur,
    Keyboard,
    SemanticAction,
    Gamepad,
}

impl RuntimeListenerType {
    pub(in crate::state_machine) fn from_value(value: u64) -> Option<Self> {
        match value {
            0 => Some(Self::Enter),
            1 => Some(Self::Exit),
            2 => Some(Self::Down),
            3 => Some(Self::Up),
            4 => Some(Self::Move),
            5 => Some(Self::Event),
            6 => Some(Self::Click),
            7 => Some(Self::ComponentProvided),
            8 => Some(Self::TextInput),
            9 => Some(Self::DragStart),
            10 => Some(Self::DragEnd),
            11 => Some(Self::ViewModel),
            12 => Some(Self::Drag),
            13 => Some(Self::Focus),
            14 => Some(Self::Blur),
            15 => Some(Self::Keyboard),
            16 => Some(Self::SemanticAction),
            17 => Some(Self::Gamepad),
            _ => None,
        }
    }

    pub(crate) fn is_pointer_hit(self) -> bool {
        matches!(
            self,
            Self::Enter
                | Self::Exit
                | Self::Down
                | Self::Up
                | Self::Move
                | Self::Click
                | Self::DragStart
                | Self::DragEnd
                | Self::Drag
        )
    }
}
