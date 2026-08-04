use crate::records::location::Location;
use alloc::string::String;

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ParseError {
    pub(crate) location: Location,
    pub(crate) message: String,
}

impl core::fmt::Display for ParseError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.message)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ParseError {}
