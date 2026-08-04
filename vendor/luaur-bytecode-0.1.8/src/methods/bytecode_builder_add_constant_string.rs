use crate::enums::r#type::Type;
use crate::records::bytecode_builder::BytecodeBuilder;
use crate::records::constant::Constant;
use crate::records::constant::ConstantValue;
use crate::records::constant_key::ConstantKey;
use crate::records::string_ref::StringRef;

impl BytecodeBuilder {
    pub fn add_constant_string(&mut self, value: StringRef) -> i32 {
        let index = self.add_string_table_entry(value);

        let c = Constant {
            r#type: Type::Type_String,
            value: ConstantValue { valueString: index },
        };

        let k = ConstantKey {
            r#type: Type::Type_String,
            value: index as u64,
            extra: 0,
        };

        self.add_constant(k, c)
    }
}
