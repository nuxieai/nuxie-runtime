//! `aux_upvalue` — resolve the n-th upvalue slot in a function TValue.
//! C++ source: `VM/src/lapi.cpp:1487`
//!
//! Returns the C-string name of the upvalue (or `""`) and writes `*val` to the
//! corresponding `TValue*`.  Returns `NULL` when `fi` is not a function or `n`
//! is out of range.

use crate::enums::lua_type::lua_Type;
use crate::macros::getstr::getstr;
use crate::type_aliases::stk_id::StkId;
use crate::type_aliases::t_value::TValue;

/// `static const char* aux_upvalue(StkId fi, int n, TValue** val)`
#[allow(non_snake_case)]
pub(crate) unsafe fn aux_upvalue(
    fi: StkId,
    n: core::ffi::c_int,
    val: *mut *mut TValue,
) -> *const core::ffi::c_char {
    if !crate::ttisfunction!(fi) {
        return core::ptr::null();
    }
    let f = &*crate::clvalue!(fi);

    if f.isC != 0 {
        // C closure — upvalues stored inline in `c.upvals`
        if !(1 <= n && n <= f.nupvalues as core::ffi::c_int) {
            return core::ptr::null();
        }
        *val = f.inner.c.upvals.as_ptr().add((n - 1) as usize) as *mut TValue;
        b"\0".as_ptr() as *const core::ffi::c_char
    } else {
        // Lua closure
        let p = f.inner.l.p;
        if !(1 <= n && n <= (*p).nups as core::ffi::c_int) {
            return core::ptr::null();
        }
        // uprefs is a flexible array — use pointer arithmetic
        let r: *mut TValue = f.inner.l.uprefs.as_ptr().add((n - 1) as usize) as *mut TValue;
        // ttisupval(r): ttype(r) == LUA_TUPVAL
        *val = if (*r).tt() == lua_Type::LUA_TUPVAL as core::ffi::c_int {
            // upvalue(r)->v
            (*(core::ptr::addr_of!((*(*r).value.gc).uv) as *const crate::records::up_val::UpVal)).v
        } else {
            r
        };
        if !(1 <= n && n <= (*p).sizeupvalues) {
            return b"\0".as_ptr() as *const core::ffi::c_char;
        }
        getstr(*(*p).upvalues.add((n - 1) as usize))
    }
}
