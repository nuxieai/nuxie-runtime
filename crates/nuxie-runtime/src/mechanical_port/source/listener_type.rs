#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum ListenerType {
    Enter = 0,
    Exit = 1,
    Down = 2,
    Up = 3,
    Move = 4,
    Event = 5,
    Click = 6,
    ComponentProvided = 7,
    TextInput = 8,
    DragStart = 9,
    DragEnd = 10,
    ViewModel = 11,
    Drag = 12,
    Focus = 13,
    Blur = 14,
    Keyboard = 15,
    SemanticAction = 16,
    Gamepad = 17,
}
