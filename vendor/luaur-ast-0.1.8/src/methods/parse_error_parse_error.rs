use crate::records::location::Location;
use crate::records::parse_error::ParseError;

impl ParseError {
    pub fn new(location: Location, message: String) -> Self {
        Self { location, message }
    }
}

#[allow(non_snake_case)]
pub fn parse_error_parse_error(location: Location, message: String) -> ParseError {
    ParseError::new(location, message)
}
