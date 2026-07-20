use crate::functions::read::read;

pub fn read_var_int(data: *const core::ffi::c_char, size: usize, offset: &mut usize) -> u32 {
    let mut result: u32 = 0;
    let mut shift: u32 = 0;

    loop {
        let byte: u8 = read(data, size, offset);
        result |= ((byte & 127) as u32) << shift;
        shift += 7;
        if (byte & 128) == 0 {
            break;
        }
    }

    result
}
