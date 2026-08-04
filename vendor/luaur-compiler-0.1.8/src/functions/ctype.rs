use crate::enums::type_constant_folding::Type;
use crate::functions::cstring_builtin_folding::cstring_c_char;
use crate::functions::cvar::cvar;
use crate::records::constant::Constant;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

pub fn ctype(c: &Constant) -> Constant {
    LUAU_ASSERT!(c.r#type != Type::Type_Unknown);

    match c.r#type {
        Type::Type_Nil => cstring_c_char(c"nil".as_ptr()),
        Type::Type_Boolean => cstring_c_char(c"boolean".as_ptr()),
        Type::Type_Number => cstring_c_char(c"number".as_ptr()),
        Type::Type_Integer => cstring_c_char(c"integer".as_ptr()),
        Type::Type_Vector => cstring_c_char(c"vector".as_ptr()),
        Type::Type_String => cstring_c_char(c"string".as_ptr()),
        _ => {
            LUAU_ASSERT!(false);
            cvar()
        }
    }
}
