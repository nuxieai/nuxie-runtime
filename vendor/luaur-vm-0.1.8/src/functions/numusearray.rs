use crate::enums::lua_type::lua_Type;
use crate::macros::maxbits::MAXBITS;
use crate::macros::ttisnil::ttisnil;
use crate::records::lua_table::LuaTable;

pub(crate) unsafe fn numusearray(
    t: *const LuaTable,
    nums: *mut core::ffi::c_int,
) -> core::ffi::c_int {
    let mut lg: core::ffi::c_int;
    let mut ttlg: core::ffi::c_int; // 2^lg
    let mut ause: core::ffi::c_int = 0; // summation of `nums'
    let mut i: core::ffi::c_int = 1; // count to traverse all array keys

    lg = 0;
    ttlg = 1;
    while lg <= MAXBITS {
        let mut lc: core::ffi::c_int = 0; // counter
        let mut lim: core::ffi::c_int = ttlg;

        if lim > (*t).sizearray {
            lim = (*t).sizearray; // adjust upper limit
            if i > lim {
                break; // no more elements to count
            }
        }

        // count elements in range (2^(lg-1), 2^lg]
        while i <= lim {
            if !ttisnil!((*t).array.offset((i - 1) as isize)) {
                lc += 1;
            }
            i += 1;
        }

        *nums.offset(lg as isize) += lc;
        ause += lc;

        lg += 1;
        ttlg *= 2;
    }

    ause
}
