use crate::macros::bit_2_mask::bit2mask;

pub const WHITE0BIT: i32 = 0;
pub const WHITE1BIT: i32 = 1;

pub const WHITEBITS: i32 = bit2mask(WHITE0BIT, WHITE1BIT);
