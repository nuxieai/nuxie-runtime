use crate::records::bytecode_builder::BytecodeBuilder;
use crate::records::typed_upval::TypedUpval;
use luaur_common::enums::luau_bytecode_type::LuauBytecodeType;

impl BytecodeBuilder {
    pub fn push_upval_type_info(&mut self, r#type: LuauBytecodeType) {
        let upval = TypedUpval { r#type };
        self.typed_upvals.push(upval);
    }
}
