extern crate alloc;
use alloc::string::String;

pub(crate) fn write_double(ss: &mut String, value: f64) {
    let bytes = value.to_ne_bytes();
    unsafe {
        ss.as_mut_vec().extend_from_slice(&bytes);
    }
}

#[allow(non_snake_case)]
pub(crate) fn writeDouble(ss: &mut String, value: f64) {
    write_double(ss, value);
}
