use crate::records::location::Location;

impl Location {
    pub fn encloses(&self, l: &Location) -> bool {
        self.begin <= l.begin && self.end >= l.end
    }
}
