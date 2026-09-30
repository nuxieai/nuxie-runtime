//! Source: rive/shapes/paint/layer_mask_mode.hpp at 8398db31.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum LayerMaskMode {
    #[default]
    Alpha = 0,
    InvertedAlpha = 1,
    Luminance = 2,
    InvertedLuminance = 3,
}
impl LayerMaskMode {
    pub fn from_value(value: u32) -> Option<Self> {
        match value {
            0 => Some(Self::Alpha),
            1 => Some(Self::InvertedAlpha),
            2 => Some(Self::Luminance),
            3 => Some(Self::InvertedLuminance),
            _ => None,
        }
    }
}
