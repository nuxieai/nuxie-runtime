use crate::records::bytecode_builder::BytecodeBuilder;
use crate::records::userdata_type::UserdataType;

impl BytecodeBuilder {
    pub fn add_userdata_type(&mut self, name: &str) -> u32 {
        let mut ty = UserdataType::default();

        ty.name = name.to_string();

        self.userdata_types.push(ty);
        (self.userdata_types.len() - 1) as u32
    }
}
