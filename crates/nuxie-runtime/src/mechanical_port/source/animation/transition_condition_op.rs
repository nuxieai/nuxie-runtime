#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransitionConditionOp {
    Equal = 0,
    NotEqual = 1,
    LessThanOrEqual = 2,
    GreaterThanOrEqual = 3,
    LessThan = 4,
    GreaterThan = 5,
}

impl TransitionConditionOp {
    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            0 => Some(Self::Equal),
            1 => Some(Self::NotEqual),
            2 => Some(Self::LessThanOrEqual),
            3 => Some(Self::GreaterThanOrEqual),
            4 => Some(Self::LessThan),
            5 => Some(Self::GreaterThan),
            _ => None,
        }
    }
}
