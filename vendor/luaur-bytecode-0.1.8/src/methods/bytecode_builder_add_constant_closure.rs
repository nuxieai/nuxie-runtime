use crate::records::bytecode_builder::BytecodeBuilder;
use crate::records::constant::Constant;
use crate::records::constant::ConstantValue;
use crate::records::constant_key::ConstantKey;

impl BytecodeBuilder {
    pub fn add_constant_closure(&mut self, fid: u32) -> i32 {
        let c = Constant {
            r#type: crate::enums::r#type::Type::Type_Closure,
            value: ConstantValue { valueClosure: fid },
        };

        let k = ConstantKey {
            r#type: crate::enums::r#type::Type::Type_Closure,
            value: fid as u64,
            extra: 0,
        };

        self.add_constant(k, c)
    }
}
