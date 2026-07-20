use crate::enums::lua_type::lua_Type;
use crate::functions::countint::countint;
use crate::macros::gkey::gkey;
use crate::macros::gkey::gval;
use crate::macros::maxbits::MAXBITS;
use crate::macros::nvalue::nvalue;
use crate::macros::sizenode::sizenode;
use crate::macros::ttisnil::ttisnil;
use crate::macros::ttisnumber::ttisnumber;
use crate::records::lua_node::LuaNode;
use crate::records::lua_table::LuaTable;

pub fn numusehash(
    t: *const LuaTable,
    nums: *mut core::ffi::c_int,
    pnasize: *mut core::ffi::c_int,
) -> core::ffi::c_int {
    let mut totaluse: core::ffi::c_int = 0; // total number of elements
    let mut ause: core::ffi::c_int = 0; // summation of `nums'
    let mut i: core::ffi::c_int = unsafe { sizenode!(t) };

    while i != 0 {
        i -= 1;

        let n: *mut LuaNode = unsafe { (*t).node.add(i as usize) };
        unsafe {
            if !ttisnil!(gval!(n)) {
                if ttisnumber!(gkey!(n)) {
                    let key = nvalue!(gkey!(n));
                    ause += countint(
                        key,
                        core::slice::from_raw_parts_mut(nums, (MAXBITS + 1) as usize),
                    );
                }
                totaluse += 1;
            }
        }
    }

    unsafe {
        *pnasize += ause;
    }
    totaluse
}
