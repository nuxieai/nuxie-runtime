#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LayoutAnimationStyle {
    None,
    Inherit,
    Custom,
}

impl From<u8> for LayoutAnimationStyle {
    fn from(value: u8) -> Self {
        Self::from(u32::from(value))
    }
}

impl From<u32> for LayoutAnimationStyle {
    fn from(value: u32) -> Self {
        match value {
            0 => Self::None,
            1 => Self::Inherit,
            2 => Self::Custom,
            _ => panic!("invalid LayoutAnimationStyle value: {value}"),
        }
    }
}
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LayoutStyleInterpolation {
    Hold,
    Linear,
    Cubic,
    Elastic,
}

impl From<u8> for LayoutStyleInterpolation {
    fn from(value: u8) -> Self {
        Self::from(u32::from(value))
    }
}

impl From<u32> for LayoutStyleInterpolation {
    fn from(value: u32) -> Self {
        match value {
            0 => Self::Hold,
            1 => Self::Linear,
            2 => Self::Cubic,
            3 => Self::Elastic,
            _ => panic!("invalid LayoutStyleInterpolation value: {value}"),
        }
    }
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

impl From<u8> for LayoutAlignmentType {
    fn from(value: u8) -> Self {
        Self::from(u32::from(value))
    }
}

impl From<u32> for LayoutAlignmentType {
    fn from(value: u32) -> Self {
        match value {
            0 => Self::TopLeft,
            1 => Self::TopCenter,
            2 => Self::TopRight,
            3 => Self::CenterLeft,
            4 => Self::Center,
            5 => Self::CenterRight,
            6 => Self::BottomLeft,
            7 => Self::BottomCenter,
            8 => Self::BottomRight,
            9 => Self::SpaceBetweenStart,
            10 => Self::SpaceBetweenCenter,
            11 => Self::SpaceBetweenEnd,
            _ => panic!("invalid LayoutAlignmentType value: {value}"),
        }
    }
}
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LayoutDirection {
    Inherit,
    Ltr,
    Rtl,
}

impl From<u8> for LayoutDirection {
    fn from(value: u8) -> Self {
        Self::from(u32::from(value))
    }
}

impl From<u32> for LayoutDirection {
    fn from(value: u32) -> Self {
        match value {
            0 => Self::Inherit,
            1 => Self::Ltr,
            2 => Self::Rtl,
            _ => panic!("invalid LayoutDirection value: {value}"),
        }
    }
}
#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum LayoutScaleType {
    Fixed,
    Fill,
    Hug,
}

impl From<u8> for LayoutScaleType {
    fn from(value: u8) -> Self {
        Self::from(u32::from(value))
    }
}

impl From<u32> for LayoutScaleType {
    fn from(value: u32) -> Self {
        match value {
            0 => Self::Fixed,
            1 => Self::Fill,
            2 => Self::Hug,
            _ => panic!("invalid LayoutScaleType value: {value}"),
        }
    }
}
