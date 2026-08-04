//! `Location::Location(const Location& begin, const Location& end)` — Location.h:86.

use crate::records::location::Location;

impl Location {
    /// Spans from one location's start to another's end.
    pub fn between(begin: Location, end: Location) -> Location {
        Location {
            begin: begin.begin,
            end: end.end,
        }
    }
}
