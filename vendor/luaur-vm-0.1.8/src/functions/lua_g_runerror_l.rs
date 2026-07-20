//! Node: `cxx:Function:Luau.VM:VM/src/ldebug.cpp:335:luaG_runerrorL`
//! Source: `VM/src/ldebug.cpp:335-347` (hand-ported; C varargs follow the
//! project convention of `core::fmt::Arguments` with the C fmt string unused)

use crate::enums::lua_status::lua_Status;
use crate::functions::lua_d_throw_ldo::lua_d_throw;
use crate::functions::lua_rawcheckstack::lua_rawcheckstack;
use crate::functions::pusherror::pusherror;
use crate::macros::lua_buffersize::LUA_BUFFERSIZE;
use crate::type_aliases::lua_state::lua_State;
use core::ffi::c_char;

struct BufWriter<'a> {
    buf: &'a mut [u8],
    pos: usize,
}

impl core::fmt::Write for BufWriter<'_> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let avail = self.buf.len().saturating_sub(self.pos + 1); // keep room for NUL
        let n = s.len().min(avail);
        self.buf[self.pos..self.pos + n].copy_from_slice(&s.as_bytes()[..n]);
        self.pos += n;
        Ok(())
    }
}

#[allow(non_snake_case)]
pub unsafe fn lua_g_runerror_l(
    L: *mut lua_State,
    _fmt: *const c_char,
    args: core::fmt::Arguments<'_>,
) -> ! {
    let mut result = [0u8; LUA_BUFFERSIZE as usize];
    let mut w = BufWriter {
        buf: &mut result,
        pos: 0,
    };
    let _ = core::fmt::write(&mut w, args);
    let len = w.pos;
    result[len] = 0;

    lua_rawcheckstack(L, 1);

    pusherror(L, result.as_ptr() as *const c_char);
    lua_d_throw(L, lua_Status::LUA_ERRRUN as i32);
}
