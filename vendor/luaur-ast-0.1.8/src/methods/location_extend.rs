use crate::records::location::Location;

impl Location {
    pub fn extend(&mut self, other: &Location) {
        if other.begin < self.begin {
            self.begin = other.begin;
        }
        if other.end > self.end {
            self.end = other.end;
        }
    }
}
