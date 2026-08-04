//! `Location::Location(const Position& begin, unsigned int length)` — Location.h:80.

use crate::records::location::Location;
use crate::records::position::Position;

impl Location {
    /// `end = Position(begin.line, begin.column + length)`.
    pub fn with_length(begin: Position, length: u32) -> Location {
        Location {
            begin,
            end: Position {
                line: begin.line,
                column: begin.column + length,
            },
        }
    }
}
