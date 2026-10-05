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

#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LayoutMainDistribute {
    #[default]
    Start,
    Center,
    End,
    SpaceBetween,
}
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LayoutCrossAlign {
    #[default]
    Start,
    Center,
    End,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LayoutContainerAlignment {
    pub main: LayoutMainDistribute,
    pub cross: LayoutCrossAlign,
}
pub fn container_alignment(kind: LayoutAlignmentType, is_row: bool) -> LayoutContainerAlignment {
    use LayoutAlignmentType::*;
    let horizontal = match kind {
        SpaceBetweenStart | SpaceBetweenCenter | SpaceBetweenEnd => {
            return LayoutContainerAlignment {
                main: LayoutMainDistribute::SpaceBetween,
                cross: match kind {
                    SpaceBetweenStart => LayoutCrossAlign::Start,
                    SpaceBetweenCenter => LayoutCrossAlign::Center,
                    _ => LayoutCrossAlign::End,
                },
            };
        }
        TopLeft | CenterLeft | BottomLeft => LayoutMainDistribute::Start,
        TopCenter | Center | BottomCenter => LayoutMainDistribute::Center,
        _ => LayoutMainDistribute::End,
    };
    let vertical = match kind {
        TopLeft | TopCenter | TopRight => LayoutCrossAlign::Start,
        CenterLeft | Center | CenterRight => LayoutCrossAlign::Center,
        _ => LayoutCrossAlign::End,
    };
    if is_row {
        LayoutContainerAlignment {
            main: horizontal,
            cross: vertical,
        }
    } else {
        LayoutContainerAlignment {
            main: match vertical {
                LayoutCrossAlign::Start => LayoutMainDistribute::Start,
                LayoutCrossAlign::Center => LayoutMainDistribute::Center,
                LayoutCrossAlign::End => LayoutMainDistribute::End,
            },
            cross: match horizontal {
                LayoutMainDistribute::Start => LayoutCrossAlign::Start,
                LayoutMainDistribute::Center => LayoutCrossAlign::Center,
                _ => LayoutCrossAlign::End,
            },
        }
    }
}

#[cfg(test)]
#[path = "layout_container_alignment_test.rs"]
mod layout_container_alignment_test;

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
