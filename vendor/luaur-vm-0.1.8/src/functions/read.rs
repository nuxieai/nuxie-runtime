#[allow(non_snake_case)]
pub fn read<T: Copy>(data: *const core::ffi::c_char, _size: usize, offset: &mut usize) -> T {
    let result = unsafe {
        let src = data.add(*offset) as *const T;
        core::ptr::read_unaligned(src)
    };
    *offset += core::mem::size_of::<T>();
    result
}
