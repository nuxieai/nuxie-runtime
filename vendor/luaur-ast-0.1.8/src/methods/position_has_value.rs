use crate::records::position::Position;

impl Position {
    pub fn has_value(&self) -> bool {
        self.line != u32::MAX || self.column != u32::MAX
    }
}
