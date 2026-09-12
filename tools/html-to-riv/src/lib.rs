//! HTML/CSS authoring compiler for the unchanged target defined in TARGET.md.
//! Output uses ordinary Rive objects only; unsupported semantics fail admission.
mod color;
mod css;
mod css_whitespace;
mod numeric_tokens;
mod request;
mod assets;
mod compiler;
mod variables;
#[allow(dead_code)]
mod wire;
#[cfg(target_arch = "wasm32")]
mod wasm;
use serde::{Deserialize, Serialize};
pub use assets::{Asset, AssetMap};

pub const BROWSER_RESET_CSS: &str = include_str!("reset.css");

pub const LANGUAGE_VERSION: &str = "nuxie-html-immutable-v1";

#[derive(Debug, Clone, Serialize)]
pub struct CompileInput {
    pub html: String,
    pub css: String,
    pub width: f32,
    pub height: f32,
    /// Explicit source-name to encoded-image mapping. Compilation performs no I/O.
    #[serde(skip_serializing_if = "AssetMap::is_empty")]
    pub assets: AssetMap,
}

impl<'de> Deserialize<'de> for CompileInput {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Fields { html: String, css: String, width: f32, height: f32,
            #[serde(default, deserialize_with = "assets::deserialize_asset_map")] assets: AssetMap }
        let Fields { html, css, width, height, assets } = request::object(deserializer, "struct CompileInput")?;
        Ok(Self { html, css, width, height, assets })
    }
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
