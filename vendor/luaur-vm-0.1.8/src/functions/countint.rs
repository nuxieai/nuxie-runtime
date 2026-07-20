use crate::functions::arrayindex::arrayindex;
use crate::macros::ceillog_2::ceillog2;
use crate::macros::maxbits::MAXSIZE;

pub(crate) fn countint(key: f64, nums: &mut [i32]) -> i32 {
    let k = arrayindex(key);
    if k > 0 && k <= MAXSIZE {
        nums[ceillog2(k as core::ffi::c_uint) as usize] += 1;
        1
    } else {
        0
    }
}
