use crate::enums::r#type::Type;
use crate::records::bytecode_builder::BytecodeBuilder;
use crate::records::constant::{Constant, ConstantValue};
use crate::records::constant_key::ConstantKey;

impl BytecodeBuilder {
    pub fn add_constant_vectord(&mut self, x: f64, y: f64, z: f64, w: f64) -> i32 {
        let c = Constant {
            r#type: Type::Type_Vectord,
            value: ConstantValue {
                valueVectord: [x, y, z, w],
            },
        };

        let k = ConstantKey {
            r#type: Type::Type_Vectord,
            value: x.to_bits(),
            extra1: y.to_bits(),
            extra2: z.to_bits(),
            extra3: w.to_bits(),
        };

        self.add_constant(k, c)
    }
}
