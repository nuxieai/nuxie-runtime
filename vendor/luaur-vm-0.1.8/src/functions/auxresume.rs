use crate::enums::lua_co_status::lua_CoStatus;
use crate::enums::lua_status::lua_Status;
use crate::functions::lua_checkstack::lua_checkstack;
use crate::functions::lua_costatus::lua_costatus;
use crate::functions::lua_l_error_l::lua_l_error_l;
use crate::functions::lua_pushfstring_l::lua_pushfstring_l;
use crate::functions::lua_resume::lua_resume;
use crate::functions::lua_xmove::lua_xmove;
use crate::macros::cast_int::cast_int;
use crate::macros::co_status_break::CO_STATUS_BREAK;
use crate::macros::co_status_error::CO_STATUS_ERROR;
use crate::macros::lua_minstack::LUA_MINSTACK;
use crate::macros::luai_maxcstack::LUAI_MAXCSTACK;
use crate::records::lua_state::lua_State;
use core::ffi::c_int;

const STATNAMES: [&str; 5] = ["running", "suspended", "normal", "dead", "dead"];

#[allow(non_snake_case)]
pub unsafe fn auxresume(l: *mut lua_State, co: *mut lua_State, narg: c_int) -> c_int {
    // error handling for edge cases
    if (*co).status != lua_Status::LUA_YIELD as u8 {
        let status = lua_costatus(l, co);
        if status != lua_CoStatus::LUA_COSUS as c_int {
            lua_pushfstring_l(
                l,
                c"cannot resume %s coroutine".as_ptr(),
                format_args!("cannot resume {} coroutine", STATNAMES[status as usize]),
            );
            return CO_STATUS_ERROR;
        }
    }

    if narg != 0 {
        if lua_checkstack(co, narg) == 0 {
            lua_l_error_l(
                l,
                c"too many arguments to resume".as_ptr(),
                format_args!("too many arguments to resume"),
            );
        }
        lua_xmove(l, co, narg);
    } else {
        // coroutine might be completely full already
        if ((*co).top.offset_from((*co).base) as i32) > LUAI_MAXCSTACK {
            lua_l_error_l(
                l,
                c"too many arguments to resume".as_ptr(),
                format_args!("too many arguments to resume"),
            );
        }
    }

    (*co).singlestep = (*l).singlestep;

    let status = lua_resume(co, l, narg);
    if status == 0 || status == lua_Status::LUA_YIELD as c_int {
        let nres = cast_int!((*co).top.offset_from((*co).base));
        if nres != 0 {
            // +1 accounts for true/false status in resumefinish
            if nres + 1 > LUA_MINSTACK && lua_checkstack(l, nres + 1) == 0 {
                lua_l_error_l(
                    l,
                    c"too many results to resume".as_ptr(),
                    format_args!("too many results to resume"),
                );
            }
            lua_xmove(co, l, nres); // move yielded values
        }
        return nres;
    } else if status == lua_Status::LUA_BREAK as c_int {
        return CO_STATUS_BREAK;
    } else {
        lua_xmove(co, l, 1); // move error message
        return CO_STATUS_ERROR;
    }
}
