use crate::enums::r#type::Type;
use crate::records::bytecode_builder::BytecodeBuilder;
use crate::records::constant::{Constant, ConstantValue};
use crate::records::constant_key::ConstantKey;

impl BytecodeBuilder {
    pub fn add_constant_vectorf(&mut self, x: f32, y: f32, z: f32, w: f32) -> i32 {
        let c = Constant {
            r#type: Type::Type_Vectorf,
            value: ConstantValue {
                valueVectorf: [x, y, z, w],
            },
        };

        let k = ConstantKey {
            r#type: Type::Type_Vectorf,
            value: x.to_bits() as u64 | (y.to_bits() as u64) << 32,
            extra1: z.to_bits() as u64 | (w.to_bits() as u64) << 32,
            extra2: 0,
            extra3: 0,
        };

        self.add_constant(k, c)
    }
}
