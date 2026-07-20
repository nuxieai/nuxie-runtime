use luaur_common::macros::luau_big_endian::LUAU_BIG_ENDIAN;

use crate::macros::bufvalue::bufvalue;
use crate::macros::checkoutofbounds::checkoutofbounds;
use crate::macros::luai_num_2_int::luai_num2int;
use crate::macros::luai_num_2_unsigned::luai_num2unsigned;
use crate::macros::nvalue::nvalue;
use crate::macros::ttisbuffer::ttisbuffer;
use crate::macros::ttisnumber::ttisnumber;
use crate::type_aliases::lua_state::LuaState;
use crate::type_aliases::stk_id::StkId;
use crate::type_aliases::t_value::TValue;

#[allow(non_snake_case)]
pub unsafe fn luauF_writeinteger<T: Copy>(
    _L: *mut LuaState,
    _res: StkId,
    arg0: *mut TValue,
    nresults: core::ffi::c_int,
    args: StkId,
    nparams: core::ffi::c_int,
) -> core::ffi::c_int {
    if !LUAU_BIG_ENDIAN
        && nparams >= 3
        && nresults <= 0
        && ttisbuffer!(arg0)
        && ttisnumber!(args)
        && ttisnumber!(args.add(1))
    {
        let mut offset: core::ffi::c_int = 0;
        luai_num2int!(offset, nvalue!(args));

        let len = (*bufvalue!(arg0)).len as usize;
        let access_size = core::mem::size_of::<T>() as usize;
        if checkoutofbounds(offset, len, access_size) {
            return -1;
        }

        let mut value: u32 = 0;
        let incoming = nvalue!(args.add(1));
        luai_num2unsigned(&mut value, incoming);

        let val: T = core::mem::transmute_copy::<u32, T>(&value);

        let dst = (*bufvalue!(arg0)).data.as_ptr().add(offset as usize) as *mut u8;
        core::ptr::copy_nonoverlapping(
            &val as *const T as *const u8,
            dst,
            core::mem::size_of::<T>(),
        );
        return 0;
    }

    -1
}
