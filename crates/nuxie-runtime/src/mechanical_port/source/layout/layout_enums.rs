#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LayoutAnimationStyle {
    None,
    Inherit,
    Custom,
}
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LayoutStyleInterpolation {
    Hold,
    Linear,
    Cubic,
    Elastic,
}
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LayoutAlignmentType {
    TopLeft,
    TopCenter,
    TopRight,
    CenterLeft,
    Center,
    CenterRight,
    BottomLeft,
    BottomCenter,
    BottomRight,
    SpaceBetweenStart,
    SpaceBetweenCenter,
    SpaceBetweenEnd,
}
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LayoutDirection {
    Inherit,
    Ltr,
    Rtl,
}
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LayoutScaleType {
    Fixed,
    Fill,
    Hug,
}
