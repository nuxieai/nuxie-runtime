use crate::records::location::Location;
use alloc::string::String;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct HotComment {
    pub header: bool,
    pub location: Location,
    pub content: String,
}
