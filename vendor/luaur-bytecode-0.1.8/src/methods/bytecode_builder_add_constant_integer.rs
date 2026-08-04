use crate::enums::r#type::Type;
use crate::records::bytecode_builder::BytecodeBuilder;
use crate::records::constant::Constant;
use crate::records::constant_key::ConstantKey;

impl BytecodeBuilder {
    pub fn add_constant_integer(&mut self, value: i64) -> i32 {
        let mut c = Constant {
            r#type: Type::Type_Integer,
            value: unsafe { core::mem::zeroed() },
        };
        unsafe {
            c.value.valueInteger64 = value;
        }

        let mut k = ConstantKey {
            r#type: Type::Type_Integer,
            value: 0,
            extra: 0,
        };

        // static_assert(sizeof(k.value) == sizeof(value), "Expecting integer to be 64-bit");
        // In Rust, k.value is u64 and value is i64, both are 8 bytes.
        unsafe {
            core::ptr::copy_nonoverlapping(
                &value as *const i64 as *const u8,
                &mut k.value as *mut u64 as *mut u8,
                core::mem::size_of::<i64>(),
            );
        }

        self.add_constant(k, c)
    }
}
