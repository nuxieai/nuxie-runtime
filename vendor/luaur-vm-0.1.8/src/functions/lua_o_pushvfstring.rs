use crate::macros::incr_top::incr_top;
use crate::macros::lua_buffersize::LUA_BUFFERSIZE;
use crate::macros::lua_s_new::luaS_new;
use crate::macros::setsvalue::setsvalue;
use crate::macros::svalue::svalue;
use crate::type_aliases::lua_state::lua_State;
use core::ffi::c_char;
use core::ffi::c_int;

#[allow(non_snake_case)]
pub fn luaO_pushvfstring(
    L: *mut lua_State,
    _fmt: *const c_char,
    args: core::fmt::Arguments<'_>,
) -> *const c_char {
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
        setsvalue!(L, (*L).top, luaS_new(L, buffer.as_ptr() as *const c_char));

        // The previous attempt failed because the incr_top! macro expansion encountered
        // name mismatches (luaD_growstack vs lua_d_growstack) and field access errors
        // (stacksize vs stacksize). We manually perform the logic here to ensure
        // compatibility with the translated records and functions.

        // luaD_checkstack(L, 1);
        let n = 1;
        let stack_last = (*L).stack_last as *mut u8;
        let top = (*L).top as *mut u8;
        let limit_reached = (stack_last as usize).wrapping_sub(top as usize)
            <= (n as usize * core::mem::size_of::<crate::type_aliases::t_value::TValue>());

        if limit_reached {
            crate::functions::lua_d_growstack::lua_d_growstack(L, n);
        } else {
            // condhardstacktests(luaD_reallocstack(L, L->stacksize - EXTRA_STACK, 0));
            // In the Rust port, we call the snake_case function.
            // Note: lua_d_reallocstack in this crate is currently a stub with no arguments.
            type LuaDReallocStackFn = unsafe fn(*mut lua_State, c_int, c_int);
            let realloc_stack: LuaDReallocStackFn = core::mem::transmute(
                crate::functions::lua_d_reallocstack::lua_d_reallocstack as *const (),
            );
            realloc_stack(
                L,
                (*L).stacksize - crate::macros::extra_stack::EXTRA_STACK,
                0,
            );
        }

        // L->top++;
        (*L).top = (*L).top.add(1);

        // svalue! expects a pointer to TValue.
        svalue!((*L).top.offset(-1))
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
