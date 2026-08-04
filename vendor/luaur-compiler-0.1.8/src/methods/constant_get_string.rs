use crate::enums::type_constant_folding::Type;
use crate::records::constant::Constant;
use core::ffi::c_char;
use core::mem::transmute;
use luaur_ast::records::ast_array::AstArray;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl Constant {
    pub fn get_string(&self) -> AstArray<c_char> {
        LUAU_ASSERT!(self.r#type == Type::Type_String);
        let data = unsafe { self.data.value_string as *mut c_char };
        let size = self.string_length as usize;
        unsafe { transmute::<(*mut c_char, usize), AstArray<c_char>>((data, size)) }
    }
}
