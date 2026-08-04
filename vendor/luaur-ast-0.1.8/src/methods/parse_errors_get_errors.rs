use crate::records::parse_error::ParseError;
use crate::records::parse_errors::ParseErrors;

impl ParseErrors {
    pub fn get_errors(&self) -> &Vec<ParseError> {
        &self.errors
    }
}
