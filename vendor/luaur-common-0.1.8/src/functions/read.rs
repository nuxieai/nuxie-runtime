pub fn read<T: Copy>(data: &[u8], offset: &mut usize) -> T {
    let size = core::mem::size_of::<T>();
    assert!(*offset + size <= data.len(), "read out of bounds");

    let mut result = core::mem::MaybeUninit::<T>::uninit();
    unsafe {
        core::ptr::copy_nonoverlapping(
            data.as_ptr().add(*offset),
            result.as_mut_ptr() as *mut u8,
            size,
        );
        *offset += size;
        result.assume_init()
    }
}
