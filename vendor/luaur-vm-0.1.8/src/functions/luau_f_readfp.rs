use luaur_common::macros::luau_big_endian::LUAU_BIG_ENDIAN;

use crate::enums::lua_type::lua_Type;
use crate::macros::bufvalue::bufvalue;
use crate::macros::checkoutofbounds::checkoutofbounds;
use crate::macros::luai_num_2_int::luai_num2int;
use crate::macros::nvalue::nvalue;
use crate::macros::setnvalue::setnvalue;
use crate::macros::ttisbuffer::ttisbuffer;
use crate::macros::ttisnumber::ttisnumber;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn luau_f_readfp<T>(
    _L: *mut lua_State,
    res: StkId,
    arg0: *mut TValue,
    nresults: core::ffi::c_int,
    args: StkId,
    nparams: core::ffi::c_int,
) -> core::ffi::c_int
where
    T: Copy + Into<f64>,
{
    if !LUAU_BIG_ENDIAN {
        if nparams >= 2 && nresults <= 1 && ttisbuffer!(arg0) && ttisnumber!(args) {
            let mut offset: core::ffi::c_int = 0;
            luai_num2int!(offset, nvalue!(args));

            let buf = bufvalue!(arg0);
            if checkoutofbounds(offset, (*buf).len as usize, core::mem::size_of::<T>()) {
                return -1;
            }

            let val: T = {
                let src = ((*buf).data.as_ptr() as *const u8).add(offset as usize);
                core::ptr::read_unaligned(src as *const T)
            };

            setnvalue!(res, val.into());
            return 1;
        }
    }

    -1
}
