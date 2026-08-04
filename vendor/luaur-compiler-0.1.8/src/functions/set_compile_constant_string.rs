use crate::enums::type_constant_folding::Type;
use crate::records::compile_error::CompileError;
use crate::records::constant::Constant;
use crate::type_aliases::compile_constant::CompileConstant;
use core::ffi::c_char;
use core::ffi::c_uint;
use luaur_ast::records::location::Location;

pub fn set_compile_constant_string(constant: CompileConstant, s: *const c_char, l: usize) {
    let target = constant as *mut Constant;

    if l > c_uint::MAX as usize {
        CompileError::raise(
            &Location::default(),
            format_args!("Exceeded custom string constant length limit"),
        );
    }

    unsafe {
        (*target).r#type = Type::Type_String;
        (*target).string_length = l as c_uint;
        (*target).data.value_string = s;
    }
}
