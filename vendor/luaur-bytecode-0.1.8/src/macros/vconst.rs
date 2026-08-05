#[allow(unused_macros)]
macro_rules! VCONST {
    (@kind Nil) => {
        crate::enums::r#type::Type::Type_Nil
    };
    (@kind Boolean) => {
        crate::enums::r#type::Type::Type_Boolean
    };
    (@kind Number) => {
        crate::enums::r#type::Type::Type_Number
    };
    (@kind Integer) => {
        crate::enums::r#type::Type::Type_Integer
    };
    (@kind Vectorf) => {
        crate::enums::r#type::Type::Type_Vectorf
    };
    (@kind Vectord) => {
        crate::enums::r#type::Type::Type_Vectord
    };
    (@kind String) => {
        crate::enums::r#type::Type::Type_String
    };
    (@kind Import) => {
        crate::enums::r#type::Type::Type_Import
    };
    (@kind Table) => {
        crate::enums::r#type::Type::Type_Table
    };
    (@kind Closure) => {
        crate::enums::r#type::Type::Type_Closure
    };
    (@kind ClassShape) => {
        crate::enums::r#type::Type::Type_ClassShape
    };
    ($v:expr, $kind:ident, $builder:expr) => {
        if luaur_common::FFlag::LuauVirtualBcBuilder.get() {
            $builder.validate_const_type($v as i32, VCONST!(@kind $kind));
        } else {
            LUAU_ASSERT!(
                ($v as usize) < $builder.constants.len()
                    && $builder.constants[$v as usize].r#type == VCONST!(@kind $kind)
            );
        }
    };
}

pub(crate) use VCONST;
