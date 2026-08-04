extern crate alloc;
use alloc::string::String;

pub(crate) fn write_byte(ss: &mut String, value: u8) {
    unsafe {
        ss.as_mut_vec().push(value);
    }
}

#[allow(non_snake_case)]
pub(crate) fn writeByte(ss: &mut String, value: u8) {
    write_byte(ss, value);
}
