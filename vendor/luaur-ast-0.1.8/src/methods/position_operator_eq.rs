use crate::records::position::Position;

impl Position {
    #[inline]
    pub fn operator_eq(&self, rhs: &Position) -> bool {
        self.line == rhs.line && self.column == rhs.column
    }
}
