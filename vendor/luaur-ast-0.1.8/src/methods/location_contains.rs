use crate::records::location::Location;
use crate::records::position::Position;

impl Location {
    pub fn contains(&self, p: Position) -> bool {
        self.begin <= p && p < self.end
    }
}
