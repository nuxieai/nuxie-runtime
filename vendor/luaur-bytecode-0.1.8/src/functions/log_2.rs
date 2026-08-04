use luaur_common::LUAU_ASSERT;

#[allow(non_snake_case)]
pub(crate) fn log2(mut v: i32) -> i32 {
    LUAU_ASSERT!(v != 0);

    let mut r = 0;

    while v >= (2 << r) {
        r += 1;
    }

    r
}
