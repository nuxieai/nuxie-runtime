extern crate alloc;
use alloc::string::String;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct UserdataType {
    pub(crate) name: String,
    pub(crate) nameRef: u32,
    pub(crate) used: bool,
}
