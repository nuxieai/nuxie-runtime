//! Node: `cxx:Function:Luau.VM:VM/src/ldebug.cpp:106:auxgetinfo`
//!
//! Fill a `lua_Debug` record from a closure + call-info, driven by the `what`
//! option string (`s` source/what/linedefined/short_src, `l` current line, `u`
//! upvalue count, `a` arity/vararg, `n` name, `f` push the function). Faithful
//! to the C++ field-by-field; returns the closure when `f` was requested.

use crate::functions::currentline::currentline;
use crate::functions::getfuncname::getfuncname;
use crate::functions::lua_o_chunkid::lua_o_chunkid;
use crate::macros::ci_func::ci_func;
use crate::macros::getstr::getstr;
use crate::macros::is_lua::isLua;
use crate::records::call_info::CallInfo;
use crate::records::closure::Closure;
use crate::records::lua_debug::LuaDebug;
use crate::type_aliases::lua_state::lua_State;
use core::ffi::c_char;

pub(crate) unsafe fn auxgetinfo(
    L: *mut lua_State,
    what: *const c_char,
    ar: *mut LuaDebug,
    f: *mut Closure,
    ci: *mut CallInfo,
) -> *mut Closure {
    let mut cl: *mut Closure = core::ptr::null_mut();

    let mut w = what;
    while *w != 0 {
        match *w as u8 {
            b's' => {
                if (*f).isC != 0 {
                    (*ar).source = c"=[C]".as_ptr();
                    (*ar).what = c"C".as_ptr();
                    (*ar).linedefined = -1;
                    (*ar).short_src = c"[C]".as_ptr();
                } else {
                    let proto = (&(*f).inner.l).p;
                    let source = (*proto).source;
                    (*ar).source = getstr(source);
                    (*ar).what = c"Lua".as_ptr();
                    (*ar).linedefined = (*proto).linedefined;
                    (*ar).short_src = lua_o_chunkid(
                        (*ar).ssbuf.as_mut_ptr(),
                        (*ar).ssbuf.len(),
                        getstr(source),
                        (*source).len as usize,
                    ) as *const c_char;
                }
            }
            b'l' => {
                if !ci.is_null() {
                    (*ar).currentline = if isLua!(ci) { currentline(L, ci) } else { -1 };
                } else {
                    (*ar).currentline = if (*f).isC != 0 {
                        -1
                    } else {
                        (*(&(*f).inner.l).p).linedefined
                    };
                }
            }
            b'u' => {
                (*ar).nupvals = (*f).nupvalues;
            }
            b'a' => {
                if (*f).isC != 0 {
                    (*ar).isvararg = 1;
                    (*ar).nparams = 0;
                } else {
                    let proto = (&(*f).inner.l).p;
                    (*ar).isvararg = (*proto).is_vararg as c_char;
                    (*ar).nparams = (*proto).numparams;
                }
            }
            b'n' => {
                (*ar).name = if !ci.is_null() {
                    getfuncname(ci_func!(ci))
                } else {
                    getfuncname(f)
                };
            }
            b'f' => {
                cl = f;
            }
            _ => {}
        }

        w = w.add(1);
    }

    cl
}
