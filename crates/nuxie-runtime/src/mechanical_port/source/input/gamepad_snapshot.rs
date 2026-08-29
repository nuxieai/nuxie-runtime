#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum GamepadInputChangeKind {
    #[default]
    Button = 0,
    Axis = 1,
}
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct GamepadInputChange {
    pub kind: GamepadInputChangeKind,
    pub index: u8,
    pub value: f32,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum GamepadMappingKind {
    #[default]
    Standard = 0,
    Unknown = 1,
}
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GamepadSnapshot {
    pub device_id: i32,
    pub button_mask: u64,
    pub button_values: Vec<f32>,
    pub axes: Vec<f32>,
    pub mapping: GamepadMappingKind,
}
