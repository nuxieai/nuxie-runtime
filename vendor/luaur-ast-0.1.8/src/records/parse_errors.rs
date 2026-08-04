use crate::records::parse_error::ParseError;
use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug, Clone)]
pub struct ParseErrors {
    pub(crate) errors: Vec<ParseError>,
    pub(crate) message: String,
}

impl core::fmt::Display for ParseErrors {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for ParseErrors {}
