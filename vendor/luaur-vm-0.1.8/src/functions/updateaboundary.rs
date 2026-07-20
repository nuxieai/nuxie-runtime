use crate::enums::lua_type::lua_Type;
use crate::macros::ttisnil::ttisnil;
use crate::records::lua_table::LuaTable;

pub unsafe fn updateaboundary(t: *mut LuaTable, boundary: core::ffi::c_int) -> core::ffi::c_int {
    if boundary < (*t).sizearray && ttisnil!((*t).array.offset((boundary - 1) as isize)) {
        if boundary >= 2 && !ttisnil!((*t).array.offset((boundary - 2) as isize)) {
            if (*t).union.aboundary <= 0 {
                (*t).union.aboundary = -(boundary - 1);
            }
            return boundary - 1;
        }
    } else if boundary + 1 < (*t).sizearray
        && !ttisnil!((*t).array.offset(boundary as isize))
        && ttisnil!((*t).array.offset((boundary + 1) as isize))
    {
        if (*t).union.aboundary <= 0 {
            (*t).union.aboundary = -(boundary + 1);
        }
        return boundary + 1;
    }

    0
}
