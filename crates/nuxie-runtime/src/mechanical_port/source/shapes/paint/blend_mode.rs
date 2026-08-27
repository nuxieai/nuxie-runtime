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
