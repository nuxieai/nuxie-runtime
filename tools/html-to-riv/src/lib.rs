//! Compiler-owned authoring primitives under requalification.
//! The public HTML/CSS compile interface is being rebuilt against TARGET.md.
//! No runtime policy installation or host dependency belongs in this library.
#[allow(dead_code)]
mod color;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub code: String,
    pub source: String,
    pub message: String,
}

impl Diagnostic {
    pub fn new(code: &str, source: &str, message: impl Into<String>) -> Self {
        Self { code: code.into(), source: source.into(), message: message.into() }
    }
}

impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} at {}: {}", self.code, self.source, self.message)
    }
}

impl std::error::Error for Diagnostic {}
