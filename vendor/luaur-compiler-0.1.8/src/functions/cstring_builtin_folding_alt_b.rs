use crate::enums::type_constant_folding::Type;
use crate::records::constant::Constant;
use core::ffi::c_char;

#[allow(non_snake_case)]
pub fn cstring_c_char_usize(v: *const c_char, len: usize) -> Constant {
    let mut res = Constant {
        r#type: Type::Type_String,
        string_length: len as u32,
        data: unsafe { core::mem::zeroed() },
    };

    unsafe {
        res.data.value_string = v;
    }

    res
}
