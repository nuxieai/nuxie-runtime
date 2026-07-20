use crate::functions::lua_l_addlstring::lua_l_addlstring;
use crate::records::lua_l_strbuf::LuaLStrbuf;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

const MAXINTSIZE: i32 = 16;
const NB: i32 = 8;
const MC: i32 = 0xff;
const SZINT: i32 = core::mem::size_of::<core::ffi::c_longlong>() as i32;

pub fn packint(b: *mut LuaLStrbuf, mut n: u64, islittle: i32, size: i32, neg: i32) {
    LUAU_ASSERT!(size <= MAXINTSIZE);
    let mut buff = [0 as core::ffi::c_char; MAXINTSIZE as usize];
    let mut i: i32 = 0;

    buff[if islittle != 0 { 0 } else { size - 1 } as usize] = (n & MC as u64) as core::ffi::c_char;

    i = 1;
    while i < size {
        n >>= NB as u32;
        buff[if islittle != 0 { i } else { size - 1 - i } as usize] =
            (n & MC as u64) as core::ffi::c_char;
        i += 1;
    }

    if neg != 0 && size > SZINT {
        i = SZINT;
        while i < size {
            buff[if islittle != 0 { i } else { size - 1 - i } as usize] = MC as core::ffi::c_char;
            i += 1;
        }
    }

    unsafe {
        lua_l_addlstring(b, buff.as_ptr(), size as usize);
    }
}
