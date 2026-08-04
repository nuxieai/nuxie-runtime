use crate::enums::type_constant_folding::Type;
use crate::records::constant::Constant;
use luaur_common::macros::luau_assert::LUAU_ASSERT;
use luaur_common::FFlag::LuauCompilePropagateTableProps2;
use luaur_common::FFlag::LuauIntegerType2;

pub fn constants_equal(la: &Constant, ra: &Constant) -> bool {
    LUAU_ASSERT!(la.r#type != Type::Type_Unknown && ra.r#type != Type::Type_Unknown);

    match la.r#type {
        Type::Type_Nil => ra.r#type == Type::Type_Nil,
        Type::Type_Boolean => {
            ra.r#type == Type::Type_Boolean
                && unsafe { la.data.value_boolean == ra.data.value_boolean }
        }
        Type::Type_Number => {
            ra.r#type == Type::Type_Number
                && unsafe { la.data.value_number == ra.data.value_number }
        }
        Type::Type_Vector => {
            ra.r#type == Type::Type_Vector
                && unsafe { la.data.value_vector[0] == ra.data.value_vector[0] }
                && unsafe { la.data.value_vector[1] == ra.data.value_vector[1] }
                && unsafe { la.data.value_vector[2] == ra.data.value_vector[2] }
                && unsafe { la.data.value_vector[3] == ra.data.value_vector[3] }
        }
        Type::Type_String => {
            ra.r#type == Type::Type_String
                && la.string_length == ra.string_length
                && unsafe {
                    std::slice::from_raw_parts(
                        la.data.value_string as *const u8,
                        la.string_length as usize,
                    ) == std::slice::from_raw_parts(
                        ra.data.value_string as *const u8,
                        ra.string_length as usize,
                    )
                }
        }
        Type::Type_Table => {
            if LuauCompilePropagateTableProps2.get() {
                ra.r#type == Type::Type_Table
                    && unsafe { la.data.value_table == ra.data.value_table }
            } else {
                LUAU_ASSERT!(false);
                false
            }
        }
        Type::Type_Integer => {
            if LuauIntegerType2.get() {
                ra.r#type == Type::Type_Integer
                    && unsafe { la.data.value_integer64 == ra.data.value_integer64 }
            } else {
                LUAU_ASSERT!(false);
                false
            }
        }
        _ => {
            LUAU_ASSERT!(false);
            false
        }
    }
}
