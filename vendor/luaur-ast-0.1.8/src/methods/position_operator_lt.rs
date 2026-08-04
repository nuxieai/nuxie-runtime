use crate::records::position::Position;

impl Position {
    #[inline]
    pub fn operator_lt(&self, rhs: &Position) -> bool {
        if self.line == rhs.line {
            self.column < rhs.column
        } else {
            self.line < rhs.line
        }
    }
}
