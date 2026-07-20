pub(crate) unsafe fn computesizes(
    nums: *const core::ffi::c_int,
    narray: *mut core::ffi::c_int,
) -> core::ffi::c_int {
    let mut twotoi: core::ffi::c_int;
    let mut a: core::ffi::c_int = 0;
    let mut na: core::ffi::c_int = 0;
    let mut n: core::ffi::c_int = 0;
    let mut i: core::ffi::c_int = 0;
    twotoi = 1;

    while twotoi / 2 < *narray {
        if *nums.add(i as usize) > 0 {
            a += *nums.add(i as usize);
            if a > twotoi / 2 {
                n = twotoi;
                na = a;
            }
        }
        if a == *narray {
            break;
        }
        i += 1;
        twotoi *= 2;
    }

    *narray = n;
    luaur_common::macros::luau_assert::LUAU_ASSERT!(*narray / 2 <= na && na <= *narray);
    na
}
