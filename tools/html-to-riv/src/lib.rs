//! HTML/CSS authoring compiler for the unchanged target defined in TARGET.md.
//! Output uses ordinary Rive objects only; unsupported semantics fail admission.
mod color;
mod css;
mod compiler;
mod variables;
#[allow(dead_code)]
mod wire;
#[cfg(target_arch = "wasm32")]
mod wasm;
use serde::{Deserialize, Serialize};

pub const BROWSER_RESET_CSS: &str = include_str!("reset.css");

pub const LANGUAGE_VERSION: &str = "nuxie-html-immutable-v1";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompileInput {
    pub html: String,
    pub css: String,
    pub width: f32,
    pub height: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SourceNode {
    pub id: String,
    pub path: String,
    pub object_id: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompileOutput {
    pub riv: Vec<u8>,
    pub source_map: Vec<SourceNode>,
}

pub fn compile(input: &CompileInput) -> Result<CompileOutput, Vec<Diagnostic>> {
    compiler::compile(input).map_err(|error| vec![error])
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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
