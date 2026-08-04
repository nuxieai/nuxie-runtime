use crate::enums::r#type::Type;
use crate::records::bytecode_builder::BytecodeBuilder;
use crate::records::constant::{Constant, ConstantValue};
use crate::records::constant_key::ConstantKey;

impl BytecodeBuilder {
    pub fn add_import(&mut self, iid: u32) -> i32 {
        let c = Constant {
            r#type: Type::Type_Import,
            value: ConstantValue { valueImport: iid },
        };

        let k = ConstantKey {
            r#type: Type::Type_Import,
            value: iid as u64,
            extra: 0,
        };

        self.add_constant(k, c)
    }
}
