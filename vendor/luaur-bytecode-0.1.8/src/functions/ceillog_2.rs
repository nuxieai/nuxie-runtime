use crate::functions::log_2::log2;
use luaur_common::LUAU_ASSERT;

#[allow(non_snake_case)]
pub(crate) fn ceillog2(v: i32) -> i32 {
    LUAU_ASSERT!(v > 0);

    if v == 1 {
        0
    } else {
        log2(v - 1) + 1
    }
}

#[allow(non_snake_case)]
pub(crate) fn ceillog_2(v: i32) -> i32 {
    ceillog2(v)
}
