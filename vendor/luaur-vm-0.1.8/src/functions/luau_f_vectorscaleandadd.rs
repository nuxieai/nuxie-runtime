use crate::enums::lua_type::lua_Type;
use crate::macros::nvalue::nvalue;
use crate::macros::setvvalue::setvvalue;
use crate::macros::ttisnumber::ttisnumber;
use crate::macros::ttisvector::ttisvector;
use crate::macros::vvalue::vvalue;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::lua_vector_type::LuaVectorType;
use crate::type_aliases::stk_id::StkId;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn luau_f_vectorscaleandadd(
    l: *mut lua_State,
    res: StkId,
    arg0: *mut TValue,
    nresults: core::ffi::c_int,
    args: StkId,
    nparams: core::ffi::c_int,
) -> core::ffi::c_int {
    if nparams >= 3
        && nresults <= 1
        && ttisvector!(arg0)
        && ttisvector!(args)
        && ttisnumber!(args.add(1))
    {
        let a = vvalue!(arg0).as_ptr();
        let b = vvalue!(args).as_ptr();
        let scale = nvalue!(args.add(1)) as LuaVectorType;
        let x = a.add(0).read() + b.add(0).read() * scale;
        let y = a.add(1).read() + b.add(1).read() * scale;
        let z = a.add(2).read() + b.add(2).read() * scale;
        setvvalue!(l, res, x, y, z, 0.0 as LuaVectorType);
        return 1;
    }

    -1
}
