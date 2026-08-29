#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArithmeticOperation {
    Add = 0,
    Subtract = 1,
    Multiply = 2,
    Divide = 3,
    Modulo = 4,
    SquareRoot = 5,
    Power = 6,
    Exp = 7,
    Log = 8,
    Cosine = 9,
    Sine = 10,
    Tangent = 11,
    Acosine = 12,
    Asine = 13,
    Atangent = 14,
    Atangent2 = 15,
    Round = 16,
    Floor = 17,
    Ceil = 18,
}
