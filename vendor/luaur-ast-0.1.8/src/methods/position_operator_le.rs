use crate::records::position::Position;

impl Position {
    #[inline]
    pub fn operator_le(&self, rhs: &Position) -> bool {
        self <= rhs
    }
}
