use crate::functions::read_var_int::read_var_int;
use crate::records::temp_buffer::TempBuffer;
use crate::type_aliases::t_string::TString;
use core::ffi::c_char;

pub fn read_string(
    strings: &mut TempBuffer<*mut TString>,
    data: *const c_char,
    size: usize,
    offset: &mut usize,
) -> *mut TString {
    let id = read_var_int(data, size, offset);

    if id == 0 {
        core::ptr::null_mut()
    } else {
        unsafe { *strings.data.add((id - 1) as usize) }
    }
}
