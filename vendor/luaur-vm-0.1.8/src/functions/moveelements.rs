use crate::functions::lua_g_readonlyerror::lua_g_readonlyerror;
use crate::functions::lua_rawgeti::lua_rawgeti;
use crate::functions::lua_rawseti::lua_rawseti;
use crate::macros::hvalue::hvalue;
use crate::macros::lua_c_barrierfast::lua_c_barrierfast;
use crate::macros::setobj_2_t::setobj2t;
use crate::records::lua_table::LuaTable;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn moveelements(L: *mut lua_State, srct: i32, dstt: i32, f: i32, e: i32, t: i32) {
    let src = hvalue!((*L).base.offset((srct - 1) as isize));
    let dst = hvalue!((*L).base.offset((dstt - 1) as isize));

    if (*dst).readonly != 0 {
        lua_g_readonlyerror(L);
    }

    let n = e - f + 1;
    let f_index = (f as u32).wrapping_sub(1);
    let t_index = (t as u32).wrapping_sub(1);
    let n_unsigned = n as u32;

    if f_index < (*src).sizearray as u32
        && t_index < (*dst).sizearray as u32
        && f_index.wrapping_add(n_unsigned) <= (*src).sizearray as u32
        && t_index.wrapping_add(n_unsigned) <= (*dst).sizearray as u32
    {
        let srcarray = (*src).array;
        let dstarray = (*dst).array;

        if t > e || t <= f || (dstt != srct && dst != src) {
            for i in 0..n {
                let s: *mut TValue = srcarray.offset((f + i - 1) as isize);
                let d: *mut TValue = dstarray.offset((t + i - 1) as isize);
                setobj2t!(L, d, s);
            }
        } else {
            for i in (0..n).rev() {
                let s: *mut TValue = srcarray.offset((f + i - 1) as isize);
                let d: *mut TValue = dstarray.offset((t + i - 1) as isize);
                setobj2t!(L, d, s);
            }
        }

        lua_c_barrierfast!(L, dst);
    } else {
        if t > e || t <= f || dst != src {
            for i in 0..n {
                lua_rawgeti(L, srct, f + i);
                lua_rawseti(L, dstt, t + i);
            }
        } else {
            for i in (0..n).rev() {
                lua_rawgeti(L, srct, f + i);
                lua_rawseti(L, dstt, t + i);
            }
        }
    }
}
