//! Timed plain-text cues. Host loaders parse the retained caption asset; this
//! clock projection is shared by authored text and accessibility consumers.
use serde::{Deserialize, Serialize};
use std::sync::Arc;
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Cue {
    pub start: f64,
    pub end: f64,
    pub text: String,
}
#[derive(Debug, Clone)]
pub struct CaptionTrack {
    language: String,
    cues: Arc<[Cue]>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptionError {
    InvalidEncoding,
    UnsupportedVersion,
    InvalidTime,
    OutOfOrder,
    TooManyCues,
    TextTooLarge,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SerializedTrack {
    version: u32,
    language: String,
    cues: Vec<Cue>,
}
impl CaptionTrack {
    /// Canonical scene representation. This is normalized plain text, not a
    /// browser VTT/HTML parser; upload tooling converts those source formats.
    pub fn from_json(json: &str) -> Result<Self, CaptionError> {
        if json.len() > 8 * 1024 * 1024 {
            return Err(CaptionError::TextTooLarge);
        }
        let track: SerializedTrack =
            serde_json::from_str(json).map_err(|_| CaptionError::InvalidEncoding)?;
        if track.version != 1 {
            return Err(CaptionError::UnsupportedVersion);
        }
        Self::new(track.language, track.cues)
    }
    pub fn to_json(&self) -> Result<String, CaptionError> {
        let json = serde_json::to_string(&SerializedTrack {
            version: 1,
            language: self.language.clone(),
            cues: self.cues.to_vec(),
        })
        .map_err(|_| CaptionError::InvalidEncoding)?;
        if json.len() > 8 * 1024 * 1024 {
            return Err(CaptionError::TextTooLarge);
        }
        Ok(json)
    }
    pub fn new(language: String, cues: Vec<Cue>) -> Result<Self, CaptionError> {
        if language.len() > 256 {
            return Err(CaptionError::TextTooLarge);
        }
        if cues.len() > 100_000 {
            return Err(CaptionError::TooManyCues);
        }
        let mut previous = 0.0;
        let mut bytes = 0usize;
        for cue in &cues {
            if !cue.start.is_finite()
                || !cue.end.is_finite()
                || cue.start < 0.0
                || cue.end <= cue.start
            {
                return Err(CaptionError::InvalidTime);
            }
            if cue.start < previous {
                return Err(CaptionError::OutOfOrder);
            }
            previous = cue.start;
            bytes = bytes
                .checked_add(cue.text.len())
                .ok_or(CaptionError::TextTooLarge)?;
            if bytes > 8 * 1024 * 1024 {
                return Err(CaptionError::TextTooLarge);
            }
        }
        Ok(Self {
            language,
            cues: cues.into(),
        })
    }
    pub fn language(&self) -> &str {
        &self.language
    }
    /// Start is inclusive, end exclusive. No mutable cursor means seeking
    /// backwards and independent instances cannot retain a stale caption.
    pub fn active(&self, time: f64) -> impl Iterator<Item = &Cue> {
        self.cues
            .iter()
            .take_while(move |cue| cue.start <= time)
            .filter(move |cue| time < cue.end)
    }
    pub fn text(&self, time: f64) -> String {
        self.active(time)
            .map(|cue| cue.text.as_str())
            .collect::<Vec<_>>()
            .join("\n")
    }
}
