use crate::records::position::Position;

impl Position {
    #[inline]
    pub fn operator_gt(&self, rhs: &Position) -> bool {
        self > rhs
    }
}
