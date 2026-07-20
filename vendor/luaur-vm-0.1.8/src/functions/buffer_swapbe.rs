#[inline]
pub fn buffer_swapbe<T>(v: T) -> T {
    let size = core::mem::size_of::<T>();
    if size == 8 {
        unsafe {
            let val = core::mem::transmute_copy::<T, u64>(&v);
            let swapped = val.swap_bytes();
            core::mem::transmute_copy::<u64, T>(&swapped)
        }
    } else if size == 4 {
        unsafe {
            let val = core::mem::transmute_copy::<T, u32>(&v);
            let swapped = val.swap_bytes();
            core::mem::transmute_copy::<u32, T>(&swapped)
        }
    } else if size == 2 {
        unsafe {
            let val = core::mem::transmute_copy::<T, u16>(&v);
            let swapped = val.swap_bytes();
            core::mem::transmute_copy::<u16, T>(&swapped)
        }
    } else {
        v
    }
}
