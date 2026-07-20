//! Node: `cxx:Function:Luau.VM:VM/src/lmathlib.cpp:517:luaopen_math`
//! Source: `VM/src/lmathlib.cpp:473-541` (hand-ported)

use crate::functions::lua_encodepointer::lua_encodepointer;
use crate::functions::lua_l_register::lua_l_register;
use crate::functions::lua_pushnumber::lua_pushnumber;
use crate::functions::lua_setfield::lua_setfield;
use crate::functions::math_abs::math_abs;
use crate::functions::math_acos::math_acos;
use crate::functions::math_asin::math_asin;
use crate::functions::math_atan::math_atan;
use crate::functions::math_atan_2::math_atan2;
use crate::functions::math_ceil::math_ceil;
use crate::functions::math_clamp::math_clamp;
use crate::functions::math_cos::math_cos;
use crate::functions::math_cosh::math_cosh;
use crate::functions::math_deg::math_deg;
use crate::functions::math_exp::math_exp;
use crate::functions::math_floor::math_floor;
use crate::functions::math_fmod::math_fmod;
use crate::functions::math_frexp::math_frexp;
use crate::functions::math_isfinite::math_isfinite;
use crate::functions::math_isinf::math_isinf;
use crate::functions::math_isnan::math_isnan;
use crate::functions::math_ldexp::math_ldexp;
use crate::functions::math_lerp::math_lerp;
use crate::functions::math_log::math_log;
use crate::functions::math_log_10::math_log_10;
use crate::functions::math_map::math_map;
use crate::functions::math_max::math_max;
use crate::functions::math_min::math_min;
use crate::functions::math_modf::math_modf;
use crate::functions::math_noise::math_noise;
use crate::functions::math_pow::math_pow;
use crate::functions::math_rad::math_rad;
use crate::functions::math_random::math_random;
use crate::functions::math_randomseed::math_randomseed;
use crate::functions::math_round::math_round;
use crate::functions::math_sign::math_sign;
use crate::functions::math_sin::math_sin;
use crate::functions::math_sinh::math_sinh;
use crate::functions::math_sqrt::math_sqrt;
use crate::functions::math_tan::math_tan;
use crate::functions::math_tanh::math_tanh;
use crate::functions::pcg_32_seed::pcg_32_seed;
use crate::macros::luau_e::LUAU_E;
use crate::macros::luau_nan::LUAU_NAN;
use crate::macros::luau_phi::LUAU_PHI;
use crate::macros::luau_pi::LUAU_PI;
use crate::macros::luau_sqrt_2::LUAU_SQRT2;
use crate::macros::luau_tau::LUAU_TAU;
use crate::records::lua_l_reg::LuaLReg;
use crate::type_aliases::lua_state::lua_State;

struct MathFuncs([LuaLReg; 38]);
unsafe impl Sync for MathFuncs {}

static MATH_FUNCS: MathFuncs = MathFuncs([
    LuaLReg {
        name: c"abs".as_ptr(),
        func: Some(math_abs),
    },
    LuaLReg {
        name: c"acos".as_ptr(),
        func: Some(math_acos),
    },
    LuaLReg {
        name: c"asin".as_ptr(),
        func: Some(math_asin),
    },
    LuaLReg {
        name: c"atan2".as_ptr(),
        func: Some(math_atan2),
    },
    LuaLReg {
        name: c"atan".as_ptr(),
        func: Some(math_atan),
    },
    LuaLReg {
        name: c"ceil".as_ptr(),
        func: Some(math_ceil),
    },
    LuaLReg {
        name: c"cosh".as_ptr(),
        func: Some(math_cosh),
    },
    LuaLReg {
        name: c"cos".as_ptr(),
        func: Some(math_cos),
    },
    LuaLReg {
        name: c"deg".as_ptr(),
        func: Some(math_deg),
    },
    LuaLReg {
        name: c"exp".as_ptr(),
        func: Some(math_exp),
    },
    LuaLReg {
        name: c"floor".as_ptr(),
        func: Some(math_floor),
    },
    LuaLReg {
        name: c"fmod".as_ptr(),
        func: Some(math_fmod),
    },
    LuaLReg {
        name: c"frexp".as_ptr(),
        func: Some(math_frexp),
    },
    LuaLReg {
        name: c"ldexp".as_ptr(),
        func: Some(math_ldexp),
    },
    LuaLReg {
        name: c"log10".as_ptr(),
        func: Some(math_log_10),
    },
    LuaLReg {
        name: c"log".as_ptr(),
        func: Some(math_log),
    },
    LuaLReg {
        name: c"max".as_ptr(),
        func: Some(math_max),
    },
    LuaLReg {
        name: c"min".as_ptr(),
        func: Some(math_min),
    },
    LuaLReg {
        name: c"modf".as_ptr(),
        func: Some(math_modf),
    },
    LuaLReg {
        name: c"pow".as_ptr(),
        func: Some(math_pow),
    },
    LuaLReg {
        name: c"rad".as_ptr(),
        func: Some(math_rad),
    },
    LuaLReg {
        name: c"random".as_ptr(),
        func: Some(math_random),
    },
    LuaLReg {
        name: c"randomseed".as_ptr(),
        func: Some(math_randomseed),
    },
    LuaLReg {
        name: c"sinh".as_ptr(),
        func: Some(math_sinh),
    },
    LuaLReg {
        name: c"sin".as_ptr(),
        func: Some(math_sin),
    },
    LuaLReg {
        name: c"sqrt".as_ptr(),
        func: Some(math_sqrt),
    },
    LuaLReg {
        name: c"tanh".as_ptr(),
        func: Some(math_tanh),
    },
    LuaLReg {
        name: c"tan".as_ptr(),
        func: Some(math_tan),
    },
    LuaLReg {
        name: c"noise".as_ptr(),
        func: Some(math_noise),
    },
    LuaLReg {
        name: c"clamp".as_ptr(),
        func: Some(math_clamp),
    },
    LuaLReg {
        name: c"sign".as_ptr(),
        func: Some(math_sign),
    },
    LuaLReg {
        name: c"round".as_ptr(),
        func: Some(math_round),
    },
    LuaLReg {
        name: c"map".as_ptr(),
        func: Some(math_map),
    },
    LuaLReg {
        name: c"lerp".as_ptr(),
        func: Some(math_lerp),
    },
    LuaLReg {
        name: c"isnan".as_ptr(),
        func: Some(math_isnan),
    },
    LuaLReg {
        name: c"isinf".as_ptr(),
        func: Some(math_isinf),
    },
    LuaLReg {
        name: c"isfinite".as_ptr(),
        func: Some(math_isfinite),
    },
    LuaLReg {
        name: core::ptr::null(),
        func: None,
    },
]);

#[allow(non_snake_case)]
pub unsafe fn luaopen_math(L: *mut lua_State) -> core::ffi::c_int {
    let mut seed = lua_encodepointer(L, L as usize) as u64;
    seed ^= 0;
    pcg_32_seed(&mut (*(*L).global).rngstate, seed);

    lua_l_register(L, c"math".as_ptr(), MATH_FUNCS.0.as_ptr());

    lua_pushnumber(L, LUAU_PI);
    lua_setfield(L, -2, c"pi".as_ptr());
    lua_pushnumber(L, f64::INFINITY);
    lua_setfield(L, -2, c"huge".as_ptr());
    lua_pushnumber(L, LUAU_NAN);
    lua_setfield(L, -2, c"nan".as_ptr());
    lua_pushnumber(L, LUAU_E);
    lua_setfield(L, -2, c"e".as_ptr());
    lua_pushnumber(L, LUAU_PHI);
    lua_setfield(L, -2, c"phi".as_ptr());
    lua_pushnumber(L, LUAU_SQRT2);
    lua_setfield(L, -2, c"sqrt2".as_ptr());
    lua_pushnumber(L, LUAU_TAU);
    lua_setfield(L, -2, c"tau".as_ptr());

    1
}
