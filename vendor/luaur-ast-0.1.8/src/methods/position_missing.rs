use crate::records::position::Position;

impl Position {
    pub fn missing() -> Position {
        Position {
            line: u32::MAX,
            column: u32::MAX,
        }
    }
}
