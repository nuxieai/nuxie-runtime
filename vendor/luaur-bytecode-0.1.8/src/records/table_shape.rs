#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TableShape {
    pub keys: [i32; 32],
    pub constants: [i32; 32],
    pub length: u32,
    pub hasConstants: bool,
}

impl TableShape {
    pub const kMaxLength: u32 = 32;
}

impl Default for TableShape {
    fn default() -> Self {
        Self {
            keys: [0; 32],
            constants: [-1; 32],
            length: 0,
            hasConstants: false,
        }
    }
}
