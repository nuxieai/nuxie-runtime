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
