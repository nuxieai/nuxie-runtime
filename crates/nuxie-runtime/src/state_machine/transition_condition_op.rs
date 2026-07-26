#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum TransitionConditionOp {
    Equal,
    NotEqual,
    LessThanOrEqual,
    GreaterThanOrEqual,
    LessThan,
    GreaterThan,
}

impl TransitionConditionOp {
    pub(super) fn from_value(value: u64) -> Self {
        match value {
            1 => Self::NotEqual,
            2 => Self::LessThanOrEqual,
            3 => Self::GreaterThanOrEqual,
            4 => Self::LessThan,
            5 => Self::GreaterThan,
            _ => Self::Equal,
        }
    }

    pub(super) fn compare(self, input_value: f32, value: f32) -> bool {
        match self {
            Self::Equal => input_value == value,
            Self::NotEqual => input_value != value,
            Self::LessThanOrEqual => input_value <= value,
            Self::GreaterThanOrEqual => input_value >= value,
            Self::LessThan => input_value < value,
            Self::GreaterThan => input_value > value,
        }
    }

    pub(super) fn compare_bool(self, input_value: bool, value: bool) -> bool {
        match self {
            Self::Equal => input_value == value,
            Self::NotEqual => input_value != value,
            _ => false,
        }
    }

    pub(super) fn compare_u32_equal_only(self, input_value: u32, value: u32) -> bool {
        match self {
            Self::Equal => input_value == value,
            Self::NotEqual => input_value != value,
            _ => false,
        }
    }

    pub(super) fn compare_bytes_equal_only(self, input_value: &[u8], value: &[u8]) -> bool {
        match self {
            Self::Equal => input_value == value,
            Self::NotEqual => input_value != value,
            _ => false,
        }
    }

    pub(super) fn compare_u64_equal_only(self, input_value: u64, value: u64) -> bool {
        match self {
            Self::Equal => input_value == value,
            Self::NotEqual => input_value != value,
            _ => false,
        }
    }
}
