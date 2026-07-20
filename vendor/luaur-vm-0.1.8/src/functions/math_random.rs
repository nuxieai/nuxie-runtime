//! Node: `cxx:Function:Luau.VM:VM/src/lmathlib.cpp:234:math_random`
//!
//! `math.random` — 0 args: a double in [0,1) from two PCG32 draws via `ldexp`
//! (here `* 2^-64`); 1 arg `u`: integer in [1,u]; 2 args `l,u`: integer in
//! [l,u]. Bounds use the high 32 bits of a 64-bit multiply (Lemire-style) to
//! avoid modulo bias. Argument checks mirror the C++ exactly.

use crate::functions::lua_gettop::lua_gettop;
use crate::functions::lua_l_checkinteger::lua_l_checkinteger;
use crate::functions::lua_pushinteger::lua_pushinteger;
use crate::functions::lua_pushnumber::lua_pushnumber;
use crate::functions::pcg_32_random::pcg_32_random;
use crate::macros::lua_l_argcheck::luaL_argcheck;
use crate::macros::lua_l_error::luaL_error;
use crate::type_aliases::lua_state::lua_State;

pub unsafe fn math_random(L: *mut lua_State) -> i32 {
    let g = (*L).global;
    match lua_gettop(L) {
        0 => {
            let rl = pcg_32_random(&mut (*g).rngstate);
            let rh = pcg_32_random(&mut (*g).rngstate);
            let bits = (rl as u64) | ((rh as u64) << 32);
            let rd = (bits as f64) * 2.0f64.powi(-64);
            lua_pushnumber(L, rd);
        }
        1 => {
            let u = lua_l_checkinteger(L, 1);
            luaL_argcheck!(L, 1 <= u, 1, "interval is empty");

            let x = (u as u64).wrapping_mul(pcg_32_random(&mut (*g).rngstate) as u64);
            let r = (1 + (x >> 32)) as i32;
            lua_pushinteger(L, r);
        }
        2 => {
            let l = lua_l_checkinteger(L, 1);
            let u = lua_l_checkinteger(L, 2);
            luaL_argcheck!(L, l <= u, 2, "interval is empty");

            let ul = (u as u32).wrapping_sub(l as u32);
            luaL_argcheck!(L, ul < u32::MAX, 2, "interval is too large");
            let x = (ul as u64 + 1).wrapping_mul(pcg_32_random(&mut (*g).rngstate) as u64);
            let r = (l as i64 + (x >> 32) as i64) as i32;
            lua_pushinteger(L, r);
        }
        _ => {
            luaL_error!(L, "wrong number of arguments");
        }
    }
    1
}
