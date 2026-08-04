use crate::records::parse_error::ParseError;
use alloc::string::String;

impl ParseError {
    pub fn get_message(&self) -> &String {
        &self.message
    }
}
