use crate::enums::type_constant_folding::Type;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl crate::records::constant::Constant {
    pub fn is_truthful(&self) -> bool {
        LUAU_ASSERT!(self.r#type != Type::Type_Unknown);
        self.r#type != Type::Type_Nil
            && !(self.r#type == Type::Type_Boolean && unsafe { self.data.value_boolean } == false)
    }
}
