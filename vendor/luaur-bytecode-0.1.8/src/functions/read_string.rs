extern crate alloc;
use luaur_common::functions::read_var_int::read_var_int;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

pub(crate) fn read_string<'a>(
    strings: &'a [&'a [u8]],
    data: &[u8],
    offset: &mut usize,
) -> &'a [u8] {
    let string_id = read_var_int(data, offset);
    LUAU_ASSERT!(string_id as usize <= strings.len());

    if string_id == 0 {
        return &[];
    }

    strings[(string_id - 1) as usize]
}
