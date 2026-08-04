use crate::records::location::Location;
use crate::records::position::Position;

impl Location {
    pub fn containsClosed(&self, p: Position) -> bool {
        self.begin <= p && p <= self.end
    }
}
