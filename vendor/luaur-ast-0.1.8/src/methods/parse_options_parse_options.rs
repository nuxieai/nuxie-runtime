use crate::records::parse_options::ParseOptions;

impl ParseOptions {
    /// C++ `ParseOptions{}` — default-initialized parse options.
    pub fn parse_options() -> Self {
        Self::default()
    }
}
