extern crate alloc;
use alloc::string::String;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct DeprecatedInfo {
    pub(crate) deprecated: bool,
    pub(crate) use_: Option<String>,
    pub(crate) reason: Option<String>,
}

impl DeprecatedInfo {
    pub fn use_suggestion(&self) -> Option<&str> {
        self.use_.as_deref()
    }

    pub fn reason(&self) -> Option<&str> {
        self.reason.as_deref()
    }
}
