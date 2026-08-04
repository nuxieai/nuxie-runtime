extern crate alloc;
use alloc::string::String;

pub(crate) fn write_int(ss: &mut String, value: i32) {
    let bytes = value.to_ne_bytes();
    unsafe {
        ss.as_mut_vec().extend_from_slice(&bytes);
    }
}

#[allow(non_snake_case)]
pub(crate) fn writeInt(ss: &mut String, value: i32) {
    write_int(ss, value);
}
