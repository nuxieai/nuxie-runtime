use crate::macros::bitmask::bitmask;
use crate::macros::cast_byte::cast_byte;

pub const WHITE0BIT: i32 = 0;
pub const WHITE1BIT: i32 = 1;
pub const BLACKBIT: i32 = 2;

#[inline(always)]
pub const fn WHITEBITS() -> i32 {
    bitmask(WHITE0BIT) | bitmask(WHITE1BIT)
}

#[allow(non_snake_case)]
#[macro_export]
macro_rules! maskmarks {
    () => {
        $crate::macros::cast_byte::cast_byte!(
            !($crate::macros::bitmask::bitmask($crate::macros::maskmarks::BLACKBIT)
                | $crate::macros::maskmarks::WHITEBITS())
        )
    };
}

pub use maskmarks;
