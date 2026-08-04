use crate::records::position::Position;

impl Position {
    #[inline]
    pub fn operator_ne(&self, rhs: &Position) -> bool {
        self != rhs
    }
}
