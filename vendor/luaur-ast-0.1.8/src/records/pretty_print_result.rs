extern crate alloc;

use crate::records::location::Location;
use alloc::string::String;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct PrettyPrintResult {
    pub code: String,
    pub error_location: Location,
    pub parse_error: String,
}
