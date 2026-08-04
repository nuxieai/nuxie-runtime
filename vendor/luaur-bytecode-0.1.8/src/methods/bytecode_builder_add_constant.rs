use crate::records::bytecode_builder::BytecodeBuilder;
use crate::records::constant::Constant;
use crate::records::constant_key::ConstantKey;

impl BytecodeBuilder {
    pub fn add_constant(&mut self, key: ConstantKey, value: Constant) -> i32 {
        if let Some(cache) = self.constant_map.find(&key) {
            return *cache;
        }

        let id = self.constants.len() as u32;

        const K_MAX_CONSTANT_COUNT: u32 = 0x007f_ffff;
        if id >= K_MAX_CONSTANT_COUNT {
            return -1;
        }

        self.constant_map.try_insert(key, id as i32);
        self.constants.push(value);

        id as i32
    }
}
