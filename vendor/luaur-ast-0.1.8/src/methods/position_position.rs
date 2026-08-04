use crate::records::position::Position;

impl Position {
    pub fn new(line: u32, column: u32) -> Self {
        Self { line, column }
    }
}

#[allow(non_snake_case)]
pub fn position_position(line: u32, column: u32) -> Position {
    Position::new(line, column)
}
