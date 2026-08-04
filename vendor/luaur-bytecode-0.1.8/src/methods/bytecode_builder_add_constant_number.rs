use crate::enums::r#type::Type;
use crate::records::bytecode_builder::BytecodeBuilder;
use crate::records::constant::Constant;
use crate::records::constant_key::ConstantKey;

impl BytecodeBuilder {
    pub fn add_constant_number(&mut self, value: f64) -> i32 {
        let mut c = Constant {
            r#type: Type::Type_Number,
            value: unsafe { core::mem::zeroed() },
        };
        // The field name in the translated ConstantValue union is valueNumber, matching the C++ union field name.
        c.value.valueNumber = value;

        let mut k = ConstantKey {
            r#type: Type::Type_Number,
            value: 0,
            extra: 0,
        };

        // Expecting double to be 64-bit
        unsafe {
            core::ptr::copy_nonoverlapping(
                &value as *const f64 as *const u8,
                &mut k.value as *mut u64 as *mut u8,
                core::mem::size_of::<f64>(),
            );
        }

        self.add_constant(k, c)
    }
}
