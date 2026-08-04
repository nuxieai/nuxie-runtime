use crate::enums::r#type::Type;
use crate::records::bytecode_builder::BytecodeBuilder;
use crate::records::constant::Constant;
use crate::records::constant_key::ConstantKey;

impl BytecodeBuilder {
    pub fn add_constant_vector(&mut self, x: f32, y: f32, z: f32, w: f32) -> i32 {
        let mut c = Constant {
            r#type: Type::Type_Vector,
            value: unsafe { core::mem::zeroed() },
        };
        // Store the actual components: the original left c.value zeroed, so vector
        // constants always read back as (0,0,0,0), breaking vector-component folding.
        c.value.valueVector = [x, y, z, w];

        let mut k = ConstantKey {
            r#type: Type::Type_Vector,
            value: 0,
            extra: 0,
        };

        k.value = x.to_bits() as u64;
        k.value |= (y.to_bits() as u64) << 32;

        k.extra = z.to_bits() as u64;
        k.extra |= (w.to_bits() as u64) << 32;

        self.add_constant(k, c)
    }
}
