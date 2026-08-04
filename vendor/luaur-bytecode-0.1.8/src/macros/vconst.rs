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
    (@kind Vector) => {
        crate::enums::r#type::Type::Type_Vector
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
    ($v:expr, $kind:ident, $constants:expr) => {
        LUAU_ASSERT!(
            ($v as usize) < $constants.len()
                && $constants[$v as usize].r#type == VCONST!(@kind $kind)
        )
    };
}

pub(crate) use VCONST;
