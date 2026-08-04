use crate::records::location::Location;

impl Location {
    pub fn overlaps(&self, l: &Location) -> bool {
        (self.begin <= l.begin && self.end >= l.begin)
            || (self.begin <= l.end && self.end >= l.end)
            || (self.begin >= l.begin && self.end <= l.end)
    }
}
