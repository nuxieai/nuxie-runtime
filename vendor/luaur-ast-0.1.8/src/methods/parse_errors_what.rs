use crate::records::parse_errors::ParseErrors;

impl ParseErrors {
    pub fn what(&self) -> &str {
        &self.message
    }
}
