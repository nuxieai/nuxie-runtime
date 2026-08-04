#[allow(non_snake_case)]
impl crate::records::location::Location {
    pub fn operator_eq(&self, rhs: &Self) -> bool {
        self.begin == rhs.begin && self.end == rhs.end
    }
}
