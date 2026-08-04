use crate::records::position::Position;

impl Position {
    pub fn shift(&mut self, start: &Position, old_end: &Position, new_end: &Position) {
        if self.position_operator_ge(start) {
            if self.line > old_end.line {
                self.line += new_end.line - old_end.line;
            } else {
                self.line = new_end.line;
                self.column += new_end.column - old_end.column;
            }
        }
    }
}

#[allow(non_snake_case)]
pub fn position_shift(
    position: &mut Position,
    start: &Position,
    old_end: &Position,
    new_end: &Position,
) {
    position.shift(start, old_end, new_end);
}
