#[allow(non_snake_case)]
pub fn read_var_int_64(data: *const core::ffi::c_char, size: usize, offset: &mut usize) -> u64 {
    let mut result: u64 = 0;
    let mut shift: u32 = 0;

    loop {
        let byte: u8 = crate::functions::read::read(data, size, offset);
        result |= ((byte & 127) as u64) << shift;
        shift += 7;
        if byte & 128 == 0 {
            break;
        }
    }

    result
}
