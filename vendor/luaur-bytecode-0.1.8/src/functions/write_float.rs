extern crate alloc;
use alloc::string::String;

pub(crate) fn write_float(ss: &mut String, value: f32) {
    let bytes = value.to_ne_bytes();
    unsafe {
        ss.as_mut_vec().extend_from_slice(&bytes);
    }
}

#[allow(non_snake_case)]
pub(crate) fn writeFloat(ss: &mut String, value: f32) {
    write_float(ss, value);
}
