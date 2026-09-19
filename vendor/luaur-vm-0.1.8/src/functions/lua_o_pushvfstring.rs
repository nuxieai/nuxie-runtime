use crate::macros::incr_top::incr_top;
use crate::macros::lua_buffersize::LUA_BUFFERSIZE;
use crate::macros::lua_s_new::luaS_new;
use crate::macros::setsvalue::setsvalue;
use crate::macros::svalue::svalue;
use crate::type_aliases::lua_state::lua_State;
use core::ffi::c_char;

#[allow(non_snake_case)]
pub fn luaO_pushvfstring(
    L: *mut lua_State,
    _fmt: *const c_char,
    args: core::fmt::Arguments<'_>,
) -> crate::records::lua_exception::LuaResult<*const c_char> {
    // Luau VM uses a fixed-size buffer for string formatting in luaO_pushvfstring.
    // Since we are translating to Rust's core::fmt::Arguments, we use a stack buffer
    // and a custom writer to mimic vsnprintf behavior.
    let mut buffer = [0u8; LUA_BUFFERSIZE as usize];
    let mut writer = BufferWriter {
        buf: &mut buffer,
        pos: 0,
    };

    let _ = core::fmt::write(&mut writer, args);

    // Ensure null termination for luaS_new which expects const char*
    let len = writer.pos;
    if len < buffer.len() {
        buffer[len] = 0;
    } else {
        buffer[buffer.len() - 1] = 0;
    }

    unsafe {
        // The macro setsvalue! expects a pointer to TValue. (*L).top is a StkId (TValue*).
        setsvalue!(L, (*L).top, luaS_new(L, buffer.as_ptr() as *const c_char)?);
        incr_top!(L);

        // svalue! expects a pointer to TValue.
        Ok(svalue!((*L).top.offset(-1)))
    }
}

struct BufferWriter<'a> {
    buf: &'a mut [u8],
    pos: usize,
}

impl<'a> core::fmt::Write for BufferWriter<'a> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        let bytes = s.as_bytes();
        let remain = self.buf.len().saturating_sub(self.pos);
        let to_copy = core::cmp::min(remain, bytes.len());
        if to_copy > 0 {
            self.buf[self.pos..self.pos + to_copy].copy_from_slice(&bytes[..to_copy]);
            self.pos += to_copy;
        }
        Ok(())
    }
}
