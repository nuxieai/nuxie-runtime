use crate::records::location::Location;

impl Location {
    #[inline]
    pub fn operator_ne(&self, rhs: &Location) -> bool {
        self != rhs
    }
}
