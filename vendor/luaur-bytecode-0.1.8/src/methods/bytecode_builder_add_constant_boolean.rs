use crate::enums::r#type::Type;
use crate::records::bytecode_builder::BytecodeBuilder;
use crate::records::constant::Constant;
use crate::records::constant::ConstantValue;
use crate::records::constant_key::ConstantKey;

impl BytecodeBuilder {
    pub fn add_constant_boolean(&mut self, value: bool) -> i32 {
        let c = Constant {
            r#type: Type::Type_Boolean,
            value: ConstantValue {
                valueBoolean: value,
            },
        };

        let k = ConstantKey {
            r#type: Type::Type_Boolean,
            value: value as u64,
            extra: 0,
        };

        self.add_constant(k, c)
    }
}
