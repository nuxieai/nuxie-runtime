use crate::records::bytecode_builder::BytecodeBuilder;
use crate::records::constant::Constant;
use crate::records::constant::ConstantValue;
use crate::records::constant_key::ConstantKey;

impl BytecodeBuilder {
    pub fn add_constant_nil(&mut self) -> i32 {
        let c = Constant {
            r#type: crate::enums::r#type::Type::Type_Nil,
            value: ConstantValue {
                valueBoolean: false,
            },
        };

        let k = ConstantKey {
            r#type: crate::enums::r#type::Type::Type_Nil,
            value: 0,
            extra: 0,
        };

        self.add_constant(k, c)
    }
}
