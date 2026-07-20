use luaur_common::macros::luau_big_endian::LUAU_BIG_ENDIAN;

use crate::functions::buffer_swapbe::buffer_swapbe;
use crate::functions::lua_l_checkbuffer::lua_l_checkbuffer;
use crate::functions::lua_l_checkinteger::lua_l_checkinteger;
use crate::functions::lua_pushnumber::lua_pushnumber;
use crate::macros::isoutofbounds::isoutofbounds;
use crate::macros::lua_l_error::luaL_error;
use crate::type_aliases::lua_state::lua_State;

#[allow(non_snake_case)]
pub unsafe fn buffer_readfp<T, StorageType>(L: *mut lua_State) -> core::ffi::c_int
where
    T: Copy + BufferReadableFloat,
    StorageType: Copy,
{
    let mut len: usize = 0;
    let buf = lua_l_checkbuffer(L, 1, &mut len) as *mut core::ffi::c_char;
    let offset = lua_l_checkinteger(L, 2);

    if isoutofbounds(offset, len, core::mem::size_of::<T>()) {
        luaL_error!(L, "buffer access out of bounds");
    }

    let mut val: T = core::mem::zeroed();

    if LUAU_BIG_ENDIAN {
        static_assert_size::<T, StorageType>();
        let mut tmp: StorageType = core::mem::zeroed();
        core::ptr::copy_nonoverlapping(
            buf.add(offset as usize),
            &mut tmp as *mut StorageType as *mut core::ffi::c_char,
            core::mem::size_of::<StorageType>(),
        );
        tmp = buffer_swapbe(tmp);
        core::ptr::copy_nonoverlapping(
            &tmp as *const StorageType as *const core::ffi::c_char,
            &mut val as *mut T as *mut core::ffi::c_char,
            core::mem::size_of::<T>(),
        );
    } else {
        core::ptr::copy_nonoverlapping(
            buf.add(offset as usize),
            &mut val as *mut T as *mut core::ffi::c_char,
            core::mem::size_of::<T>(),
        );
    }

    lua_pushnumber(L, val.to_f64());
    1
}

#[allow(dead_code)]
fn static_assert_size<T, StorageType>() {
    assert!(core::mem::size_of::<T>() == core::mem::size_of::<StorageType>());
}

pub trait BufferReadableFloat {
    fn to_f64(self) -> f64;
}

impl BufferReadableFloat for f32 {
    fn to_f64(self) -> f64 {
        self as f64
    }
}

impl BufferReadableFloat for f64 {
    fn to_f64(self) -> f64 {
        self
    }
}
