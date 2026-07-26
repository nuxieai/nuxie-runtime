/// The state-machine invocation supplied to a scripted listener action.
///
/// Scheduled state/transition actions use [`Self::None`]. Pointer listeners
/// retain the concrete pointer payload so scripting backends can expose the
/// same legacy `PointerEvent` shape as the C++ runtime.
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
    ReportedEvent {
        event_local_index: usize,
        seconds_delay: f32,
    },
    None,
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
