use crate::records::location::Location;
use crate::records::parse_error::ParseError;

impl ParseError {
    pub fn get_location(&self) -> &Location {
        &self.location
    }
}
