use crate::records::location::Location;
use crate::records::position::Position;

impl Location {
    pub fn shift(&mut self, start: &Position, old_end: &Position, new_end: &Position) {
        self.begin.shift(start, old_end, new_end);
        self.end.shift(start, old_end, new_end);
    }
}
