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
    pub(crate) fn value(self) -> u32 {
        match self {
            Self::Enter => 0,
            Self::Exit => 1,
            Self::Down => 2,
            Self::Up => 3,
            Self::Move => 4,
            Self::Event => 5,
            Self::Click => 6,
            Self::ComponentProvided => 7,
            Self::TextInput => 8,
            Self::DragStart => 9,
            Self::DragEnd => 10,
            Self::ViewModel => 11,
            Self::Drag => 12,
            Self::Focus => 13,
            Self::Blur => 14,
            Self::Keyboard => 15,
            Self::SemanticAction => 16,
            Self::Gamepad => 17,
        }
    }

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
