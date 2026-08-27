use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Not};

#[repr(transparent)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextModifierFlags(pub u8);

impl TextModifierFlags {
    pub const MODIFY_ORIGIN: Self = Self(1 << 0);
    pub const MODIFY_TRANSLATION: Self = Self(1 << 2);
    pub const MODIFY_ROTATION: Self = Self(1 << 3);
    pub const MODIFY_SCALE: Self = Self(1 << 4);
    pub const MODIFY_OPACITY: Self = Self(1 << 5);
    pub const INVERT_OPACITY: Self = Self(1 << 6);
}

macro_rules! bit_op {
    ($trait:ident, $method:ident, $op:tt) => {
        impl $trait for TextModifierFlags {
            type Output = Self;
            fn $method(self, rhs: Self) -> Self { Self(self.0 $op rhs.0) }
        }
    };
}
bit_op!(BitAnd, bitand, &);
bit_op!(BitOr, bitor, |);
bit_op!(BitXor, bitxor, ^);

impl Not for TextModifierFlags {
    type Output = Self;
    fn not(self) -> Self {
        Self(!self.0)
    }
}

macro_rules! bit_assign {
    ($trait:ident, $method:ident, $op:tt) => {
        impl $trait for TextModifierFlags {
            fn $method(&mut self, rhs: Self) { self.0 $op rhs.0; }
        }
    };
}
bit_assign!(BitAndAssign, bitand_assign, &=);
bit_assign!(BitOrAssign, bitor_assign, |=);
bit_assign!(BitXorAssign, bitxor_assign, ^=);
