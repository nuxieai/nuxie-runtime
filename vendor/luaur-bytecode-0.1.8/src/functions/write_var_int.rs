use crate::functions::write_byte::write_byte;
use alloc::string::String;

pub(crate) fn write_var_int(ss: &mut String, mut value: u64) {
    loop {
        write_byte(ss, (value & 127) as u8 | (((value > 127) as u8) << 7));
        value >>= 7;
        if value == 0 {
            break;
        }
    }
}

#[allow(non_snake_case)]
pub(crate) fn writeVarInt(ss: &mut String, value: u64) {
    write_var_int(ss, value);
}
