use crate::enums::lua_type::lua_Type;
use crate::functions::lua_l_checktype::lua_l_checktype;
use crate::functions::lua_pushnumber::lua_pushnumber;
use crate::macros::gkey::gkey;
use crate::macros::gkey::gval;
use crate::macros::gnode::gnode;
use crate::macros::hvalue::hvalue;
use crate::macros::nvalue::nvalue;
use crate::macros::sizenode::sizenode;
use crate::macros::ttisnil::ttisnil;
use crate::macros::ttisnumber::ttisnumber;
use crate::type_aliases::lua_state::lua_State;

pub unsafe fn maxn(l: *mut lua_State) -> core::ffi::c_int {
    let mut max: f64 = 0.0;
    unsafe {
        lua_l_checktype(l, 1, lua_Type::LUA_TTABLE as core::ffi::c_int);

        let t = hvalue!((*l).base);

        for i in 0..(*t).sizearray {
            if !ttisnil!((*t).array.add(i as usize)) {
                max = (i + 1) as f64;
            }
        }

        let node_size = sizenode!(t);
        for i in 0..node_size {
            let n = gnode!(t, i);

            if !ttisnil!(gval!(n)) && ttisnumber!(gkey!(n)) {
                let v = nvalue!(gkey!(n));

                if v > max {
                    max = v;
                }
            }
        }

        lua_pushnumber(l, max);
    }
    1
}
