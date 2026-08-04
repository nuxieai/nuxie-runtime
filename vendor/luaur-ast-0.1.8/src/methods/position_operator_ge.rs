use crate::records::position::Position;

impl Position {
    #[inline]
    pub fn position_operator_ge(&self, rhs: &Position) -> bool {
        self >= rhs
    }
}
