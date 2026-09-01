use core::ffi::c_int;

use crate::enums::lua_type::lua_Type;
use crate::functions::lua_absindex::lua_absindex;
use crate::functions::lua_rawiter::lua_rawiter;
use crate::functions::lua_g_readonlyerror::lua_g_readonlyerror;
use crate::functions::lua_rawgeti::lua_rawgeti;
use crate::functions::lua_rawseti::lua_rawseti;
use crate::functions::lua_tointegerx::lua_tointegerx;
use crate::functions::lua_tonumberx::lua_tonumberx;
use crate::functions::lua_type::lua_type;
use crate::macros::hvalue::hvalue;
use crate::macros::lua_c_barrierfast::lua_c_barrierfast;
use crate::macros::lua_newtable::lua_newtable;
use crate::macros::lua_pop::lua_pop;
use crate::macros::sizenode::sizenode;
use crate::macros::setobj_2_t::setobj2t;
use crate::records::lua_table::LuaTable;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::t_value::TValue;

/// Decide whether the sparse `table.move` path from Luau 0.733 is needed.
///
/// The regular path performs one raw lookup per element in the requested
/// range.  For a very large, mostly empty range that can take long enough to
/// trip the VM timeout guard; the upstream implementation switches to a
/// bounded iteration over the actual table entries instead.
#[inline]
pub(crate) unsafe fn should_sparse_move(src: *mut LuaTable, dst: *mut LuaTable, n: c_int) -> bool {
    let srcelems = (*src).sizearray.wrapping_add(sizenode!(src));
    let dstelems = (*dst).sizearray.wrapping_add(sizenode!(dst));
    let maxelems = srcelems.max(dstelems);
    n > 32 && n / 2 > maxelems
}

/// Convert a stack key to an integer only when it is an exact integer in the
/// requested inclusive range.  This mirrors Luau's `tovalidintkey` helper.
#[inline]
unsafe fn to_valid_int_key(
    l: *mut lua_State,
    idx: c_int,
    first: c_int,
    last: c_int,
) -> Option<c_int> {
    if lua_type(l, idx) != lua_Type::LUA_TNUMBER as c_int {
        return None;
    }

    let number = lua_tonumberx(l, idx, core::ptr::null_mut());
    if number < first as f64 || number > last as f64 {
        return None;
    }

    // The range check above makes this conversion representable.  Rust's
    // float-to-int cast has the same truncating semantics as Luau's
    // `luai_num2int` here; the equality test rejects fractional values.
    let integer = number as c_int;
    (integer as f64 == number).then_some(integer)
}

#[allow(non_snake_case)]
pub unsafe fn moveelements(
    L: *mut lua_State,
    srct: i32,
    dstt: i32,
    f: i32,
    e: i32,
    t: i32,
    sparsemove: bool,
) {
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
    } else if luaur_common::DFFlag::LuauTableMoveTimeoutFix.get() && sparsemove {
        let srcta = lua_absindex(L, srct);
        let dstta = lua_absindex(L, dstt);
        let te = t + (n - 1);

        // Temporary table holding the source entries in the requested range.
        // Keeping the entries alive also makes overlapping source/destination
        // moves behave exactly like the C++ implementation.
        lua_newtable(L);

        // Collect only integer keys in [f, e]. `lua_rawiter` leaves key/value
        // at -2/-1; rawseti consumes the value and the explicit pop consumes
        // the key, restoring the temporary table at the top each iteration.
        let mut iter = 0;
        loop {
            iter = crate::functions::lua_rawiter::lua_rawiter(L, srcta, iter);
            if iter == -1 {
                break;
            }

            if let Some(ikey) = to_valid_int_key(L, -2, f, e) {
                lua_rawseti(L, -3, ikey);
            } else {
                lua_pop(L, 1);
            }
            lua_pop(L, 1);
        }

        // Clear destination entries in [t, te] before copying.  Iteration is
        // over the actual hash/array entries, so sparse ranges stay bounded.
        let mut iter = 0;
        loop {
            iter = lua_rawiter(L, dstta, iter);
            if iter == -1 {
                break;
            }

            if let Some(ikey) = to_valid_int_key(L, -2, t, te) {
                crate::functions::lua_pushnil::lua_pushnil(L);
                lua_rawseti(L, dstta, ikey);
            }
            lua_pop(L, 2);
        }

        // Copy the collected entries to their translated destination keys.
        let mut iter = 0;
        loop {
            iter = lua_rawiter(L, -1, iter);
            if iter == -1 {
                break;
            }
            let ikey = lua_tointegerx(L, -2, core::ptr::null_mut());
            lua_rawseti(L, dstta, ikey - f + t);
            lua_pop(L, 1);
        }
        lua_pop(L, 1);
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
