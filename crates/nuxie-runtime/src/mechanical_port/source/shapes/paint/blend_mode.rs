#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum BlendMode {
    SrcOver = 3,
    Screen = 14,
    Overlay = 15,
    Darken = 16,
    Lighten = 17,
    ColorDodge = 18,
    ColorBurn = 19,
    HardLight = 20,
    SoftLight = 21,
    Difference = 22,
    Exclusion = 23,
    Multiply = 24,
    Hue = 25,
    Saturation = 26,
    Color = 27,
    Luminosity = 28,
}
pub const BLEND_MODE_BIT_COUNT: u32 = 5;

impl From<BlendMode> for nuxie_render_api::BlendMode {
    fn from(value: BlendMode) -> Self {
        match value {
            BlendMode::SrcOver => Self::SrcOver,
            BlendMode::Screen => Self::Screen,
            BlendMode::Overlay => Self::Overlay,
            BlendMode::Darken => Self::Darken,
            BlendMode::Lighten => Self::Lighten,
            BlendMode::ColorDodge => Self::ColorDodge,
            BlendMode::ColorBurn => Self::ColorBurn,
            BlendMode::HardLight => Self::HardLight,
            BlendMode::SoftLight => Self::SoftLight,
            BlendMode::Difference => Self::Difference,
            BlendMode::Exclusion => Self::Exclusion,
            BlendMode::Multiply => Self::Multiply,
            BlendMode::Hue => Self::Hue,
            BlendMode::Saturation => Self::Saturation,
            BlendMode::Color => Self::Color,
            BlendMode::Luminosity => Self::Luminosity,
        }
    }
}
