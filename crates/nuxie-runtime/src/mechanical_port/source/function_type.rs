#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum FunctionType {
    Min = 0,
    Max = 1,
    Round = 2,
    Ceil = 3,
    Floor = 4,
    Sqrt = 5,
    Pow = 6,
    Exp = 7,
    Log = 8,
    Cosine = 9,
    Sine = 10,
    Tangent = 11,
    Acosine = 12,
    Asine = 13,
    Atangent = 14,
    Atangent2 = 15,
    Random = 16,
}
