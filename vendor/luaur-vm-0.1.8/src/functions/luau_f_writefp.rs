use crate::enums::lua_type::lua_Type;
use crate::macros::bufvalue::bufvalue;
use crate::macros::checkoutofbounds::checkoutofbounds;
use crate::macros::luai_num_2_int::luai_num2int;
use crate::macros::nvalue::nvalue;
use crate::macros::ttisbuffer::ttisbuffer;
use crate::macros::ttisnumber::ttisnumber;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;
use crate::type_aliases::t_value::TValue;
use luaur_common::macros::luau_big_endian::LUAU_BIG_ENDIAN;

#[allow(non_snake_case)]
pub unsafe fn luau_f_writefp<T>(
    _L: *mut lua_State,
    _res: StkId,
    arg0: *mut TValue,
    nresults: core::ffi::c_int,
    args: StkId,
    nparams: core::ffi::c_int,
) -> core::ffi::c_int
where
    T: Copy + From<f64>,
{
    if !LUAU_BIG_ENDIAN {
        if nparams >= 3
            && nresults <= 0
            && ttisbuffer!(arg0)
            && ttisnumber!(args)
            && ttisnumber!(args.add(1))
        {
            let mut offset: core::ffi::c_int = 0;
            luai_num2int!(offset, nvalue!(args));

            let buf = bufvalue!(arg0);
            if checkoutofbounds(offset, (*buf).len as usize, core::mem::size_of::<T>()) {
                return -1;
            }

            let val: T = T::from(nvalue!(args.add(1)));

            let dest = ((*buf).data.as_ptr() as *mut u8).add(offset as usize);
            core::ptr::write_unaligned(dest as *mut T, val);

            return 0;
        }
    }

    -1
}
