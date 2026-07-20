use luaur_common::macros::luau_big_endian::LUAU_BIG_ENDIAN;

use crate::functions::buffer_swapbe::buffer_swapbe;
use crate::functions::lua_l_checkbuffer::lua_l_checkbuffer;
use crate::functions::lua_l_checkinteger::lua_l_checkinteger;
use crate::functions::lua_l_checknumber::lua_l_checknumber;
use crate::macros::isoutofbounds::isoutofbounds;
use crate::macros::lua_l_error::luaL_error;
use crate::type_aliases::lua_state::lua_State;

#[allow(non_snake_case)]
pub unsafe fn buffer_writefp<T, StorageType>(L: *mut lua_State) -> core::ffi::c_int
where
    T: Copy + BufferFloat,
    StorageType: Copy,
{
    let mut len: usize = 0;
    let buf = lua_l_checkbuffer(L, 1, &mut len);

    let offset = lua_l_checkinteger(L, 2);
    let value = lua_l_checknumber(L, 3);

    if isoutofbounds(offset, len, core::mem::size_of::<T>()) {
        luaL_error!(L, "buffer access out of bounds");
    }

    let val: T = T::from_f64(value);

    if LUAU_BIG_ENDIAN {
        static_assert_size_match::<T, StorageType>();
        let mut tmp: StorageType =
            core::ptr::read_unaligned(&val as *const T as *const StorageType);
        tmp = buffer_swapbe(tmp);
        core::ptr::copy_nonoverlapping(
            &tmp as *const StorageType as *const u8,
            (buf as *mut u8).add(offset as usize),
            core::mem::size_of::<StorageType>(),
        );
    } else {
        core::ptr::copy_nonoverlapping(
            &val as *const T as *const u8,
            (buf as *mut u8).add(offset as usize),
            core::mem::size_of::<T>(),
        );
    }

    0
}

// Helper to enforce sizeof(T) == sizeof(StorageType) at compile time
#[allow(dead_code)]
fn static_assert_size_match<T, StorageType>() {
    assert!(core::mem::size_of::<T>() == core::mem::size_of::<StorageType>());
}

pub trait BufferFloat {
    fn from_f64(value: f64) -> Self;
}

impl BufferFloat for f32 {
    fn from_f64(value: f64) -> Self {
        value as f32
    }
}

impl BufferFloat for f64 {
    fn from_f64(value: f64) -> Self {
        value
    }
}
