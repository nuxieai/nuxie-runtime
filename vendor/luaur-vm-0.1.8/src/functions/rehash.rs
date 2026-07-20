use crate::functions::adjustasize::adjustasize;
use crate::functions::computesizes::computesizes;
use crate::functions::countint::countint;
use crate::functions::numusearray::numusearray;
use crate::functions::numusehash::numusehash;
use crate::functions::resize::resize;
use crate::macros::maxbits::MAXBITS;
use crate::macros::nvalue::nvalue;
use crate::macros::ttisnumber::ttisnumber;
use crate::records::lua_state::lua_State;
use crate::records::lua_table::LuaTable;
use crate::type_aliases::t_value::TValue;

pub(crate) unsafe fn rehash(l: *mut lua_State, t: *mut LuaTable, ek: *const TValue) {
    let mut nums = [0i32; (MAXBITS + 1) as usize];
    let nasize = numusearray(t, nums.as_mut_ptr());
    let mut totaluse = nasize;
    let mut nasize_mut = nasize;
    totaluse += numusehash(t, nums.as_mut_ptr(), &mut nasize_mut);

    if ttisnumber!(ek) {
        nasize_mut += countint(nvalue!(ek), &mut nums);
    }
    totaluse += 1;

    let na = computesizes(nums.as_ptr(), &mut nasize_mut);
    let mut nh = totaluse - na;

    let nadjusted = adjustasize(t, nasize_mut, ek);
    let aextra = nadjusted - nasize_mut;

    if aextra != 0 {
        nh -= aextra;
        nasize_mut = nadjusted + aextra;
        nasize_mut = adjustasize(t, nasize_mut, ek);
    }

    resize(l, t, nasize_mut, nh);
}
