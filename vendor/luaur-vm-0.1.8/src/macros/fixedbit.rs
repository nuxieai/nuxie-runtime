pub const FIXEDBIT: i32 = 3;

#[inline(always)]
pub const fn bit2mask(b1: i32, b2: i32) -> i32 {
    (1 << b1) | (1 << b2)
}

pub const WHITE0BIT: i32 = 0;
pub const WHITE1BIT: i32 = 1;

pub const WHITEBITS: i32 = bit2mask(WHITE0BIT, WHITE1BIT);
