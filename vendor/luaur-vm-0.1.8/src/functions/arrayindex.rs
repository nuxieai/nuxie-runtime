use crate::macros::cast_num::cast_num;
use crate::macros::luai_num_2_int::luai_num2int;
use crate::macros::luai_numeq::luai_numeq;

pub fn arrayindex(key: f64) -> i32 {
    let mut i: core::ffi::c_int = 0;
    luai_num2int!(i, key);

    if luai_numeq(cast_num!(i), key) {
        i
    } else {
        -1
    }
}
