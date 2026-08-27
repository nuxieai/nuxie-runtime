#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum GamepadInputKind {
    Button = 0,
    Axis = 1,
    Connected = 2,
    Disconnected = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum GamepadInputMapping {
    Standard = 0,
    Index = 1,
}
