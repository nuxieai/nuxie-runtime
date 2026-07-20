use crate::functions::buffer_copy::buffer_copy;
use crate::functions::buffer_create::buffer_create;
use crate::functions::buffer_fill::buffer_fill;
use crate::functions::buffer_fromstring::buffer_fromstring;
use crate::functions::buffer_len::buffer_len;
use crate::functions::buffer_readbits::buffer_readbits;
use crate::functions::buffer_readfp::buffer_readfp;
use crate::functions::buffer_readinteger::buffer_readinteger;
use crate::functions::buffer_readlong::buffer_readlong;
use crate::functions::buffer_readstring::buffer_readstring;
use crate::functions::buffer_tostring::buffer_tostring;
use crate::functions::buffer_writebits::buffer_writebits;
use crate::functions::buffer_writefp::buffer_writefp;
use crate::functions::buffer_writeinteger::buffer_writeinteger;
use crate::functions::buffer_writelong::buffer_writelong;
use crate::functions::buffer_writestring::buffer_writestring;
use crate::functions::lua_l_register::lua_l_register;
use crate::records::lua_l_reg::LuaLReg;
use crate::type_aliases::lua_state::lua_State;
use luaur_common::FFlag;

struct SyncLuaLReg<const N: usize>([LuaLReg; N]);
unsafe impl<const N: usize> Sync for SyncLuaLReg<N> {}

static BUFFER_LIB: SyncLuaLReg<29> = SyncLuaLReg([
    LuaLReg {
        name: c"create".as_ptr(),
        func: Some(buffer_create),
    },
    LuaLReg {
        name: c"fromstring".as_ptr(),
        func: Some(buffer_fromstring),
    },
    LuaLReg {
        name: c"tostring".as_ptr(),
        func: Some(buffer_tostring),
    },
    LuaLReg {
        name: c"readi8".as_ptr(),
        func: Some(buffer_readinteger::<i8>),
    },
    LuaLReg {
        name: c"readu8".as_ptr(),
        func: Some(buffer_readinteger::<u8>),
    },
    LuaLReg {
        name: c"readi16".as_ptr(),
        func: Some(buffer_readinteger::<i16>),
    },
    LuaLReg {
        name: c"readu16".as_ptr(),
        func: Some(buffer_readinteger::<u16>),
    },
    LuaLReg {
        name: c"readi32".as_ptr(),
        func: Some(buffer_readinteger::<i32>),
    },
    LuaLReg {
        name: c"readu32".as_ptr(),
        func: Some(buffer_readinteger::<u32>),
    },
    LuaLReg {
        name: c"readf32".as_ptr(),
        func: Some(buffer_readfp::<f32, u32>),
    },
    LuaLReg {
        name: c"readf64".as_ptr(),
        func: Some(buffer_readfp::<f64, u64>),
    },
    LuaLReg {
        name: c"writei8".as_ptr(),
        func: Some(buffer_writeinteger::<i8>),
    },
    LuaLReg {
        name: c"writeu8".as_ptr(),
        func: Some(buffer_writeinteger::<u8>),
    },
    LuaLReg {
        name: c"writei16".as_ptr(),
        func: Some(buffer_writeinteger::<i16>),
    },
    LuaLReg {
        name: c"writeu16".as_ptr(),
        func: Some(buffer_writeinteger::<u16>),
    },
    LuaLReg {
        name: c"writei32".as_ptr(),
        func: Some(buffer_writeinteger::<i32>),
    },
    LuaLReg {
        name: c"writeu32".as_ptr(),
        func: Some(buffer_writeinteger::<u32>),
    },
    LuaLReg {
        name: c"writef32".as_ptr(),
        func: Some(buffer_writefp::<f32, u32>),
    },
    LuaLReg {
        name: c"writef64".as_ptr(),
        func: Some(buffer_writefp::<f64, u64>),
    },
    LuaLReg {
        name: c"readstring".as_ptr(),
        func: Some(buffer_readstring),
    },
    LuaLReg {
        name: c"writestring".as_ptr(),
        func: Some(buffer_writestring),
    },
    LuaLReg {
        name: c"len".as_ptr(),
        func: Some(buffer_len),
    },
    LuaLReg {
        name: c"copy".as_ptr(),
        func: Some(buffer_copy),
    },
    LuaLReg {
        name: c"fill".as_ptr(),
        func: Some(buffer_fill),
    },
    LuaLReg {
        name: c"readbits".as_ptr(),
        func: Some(buffer_readbits),
    },
    LuaLReg {
        name: c"writebits".as_ptr(),
        func: Some(buffer_writebits),
    },
    LuaLReg {
        name: c"readinteger".as_ptr(),
        func: Some(buffer_readlong),
    },
    LuaLReg {
        name: c"writeinteger".as_ptr(),
        func: Some(buffer_writelong),
    },
    LuaLReg {
        name: core::ptr::null(),
        func: None,
    },
]);

static BUFFER_LIB_NO_INTEGER: SyncLuaLReg<27> = SyncLuaLReg([
    LuaLReg {
        name: c"create".as_ptr(),
        func: Some(buffer_create),
    },
    LuaLReg {
        name: c"fromstring".as_ptr(),
        func: Some(buffer_fromstring),
    },
    LuaLReg {
        name: c"tostring".as_ptr(),
        func: Some(buffer_tostring),
    },
    LuaLReg {
        name: c"readi8".as_ptr(),
        func: Some(buffer_readinteger::<i8>),
    },
    LuaLReg {
        name: c"readu8".as_ptr(),
        func: Some(buffer_readinteger::<u8>),
    },
    LuaLReg {
        name: c"readi16".as_ptr(),
        func: Some(buffer_readinteger::<i16>),
    },
    LuaLReg {
        name: c"readu16".as_ptr(),
        func: Some(buffer_readinteger::<u16>),
    },
    LuaLReg {
        name: c"readi32".as_ptr(),
        func: Some(buffer_readinteger::<i32>),
    },
    LuaLReg {
        name: c"readu32".as_ptr(),
        func: Some(buffer_readinteger::<u32>),
    },
    LuaLReg {
        name: c"readf32".as_ptr(),
        func: Some(buffer_readfp::<f32, u32>),
    },
    LuaLReg {
        name: c"readf64".as_ptr(),
        func: Some(buffer_readfp::<f64, u64>),
    },
    LuaLReg {
        name: c"writei8".as_ptr(),
        func: Some(buffer_writeinteger::<i8>),
    },
    LuaLReg {
        name: c"writeu8".as_ptr(),
        func: Some(buffer_writeinteger::<u8>),
    },
    LuaLReg {
        name: c"writei16".as_ptr(),
        func: Some(buffer_writeinteger::<i16>),
    },
    LuaLReg {
        name: c"writeu16".as_ptr(),
        func: Some(buffer_writeinteger::<u16>),
    },
    LuaLReg {
        name: c"writei32".as_ptr(),
        func: Some(buffer_writeinteger::<i32>),
    },
    LuaLReg {
        name: c"writeu32".as_ptr(),
        func: Some(buffer_writeinteger::<u32>),
    },
    LuaLReg {
        name: c"writef32".as_ptr(),
        func: Some(buffer_writefp::<f32, u32>),
    },
    LuaLReg {
        name: c"writef64".as_ptr(),
        func: Some(buffer_writefp::<f64, u64>),
    },
    LuaLReg {
        name: c"readstring".as_ptr(),
        func: Some(buffer_readstring),
    },
    LuaLReg {
        name: c"writestring".as_ptr(),
        func: Some(buffer_writestring),
    },
    LuaLReg {
        name: c"len".as_ptr(),
        func: Some(buffer_len),
    },
    LuaLReg {
        name: c"copy".as_ptr(),
        func: Some(buffer_copy),
    },
    LuaLReg {
        name: c"fill".as_ptr(),
        func: Some(buffer_fill),
    },
    LuaLReg {
        name: c"readbits".as_ptr(),
        func: Some(buffer_readbits),
    },
    LuaLReg {
        name: c"writebits".as_ptr(),
        func: Some(buffer_writebits),
    },
    LuaLReg {
        name: core::ptr::null(),
        func: None,
    },
]);

pub unsafe fn luaopen_buffer(L: *mut lua_State) -> core::ffi::c_int {
    let buffer_lib = if FFlag::LuauIntegerLibrary.get() {
        BUFFER_LIB.0.as_ptr()
    } else {
        BUFFER_LIB_NO_INTEGER.0.as_ptr()
    };

    lua_l_register(L, c"buffer".as_ptr(), buffer_lib);
    1
}
