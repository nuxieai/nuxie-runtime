use crate::enums::r#type::Type;
use crate::records::bytecode_builder::BytecodeBuilder;
use crate::records::class_shape::ClassShape;
use crate::records::constant::Constant;

impl BytecodeBuilder {
    pub fn add_class_shape(&mut self, shape: ClassShape) -> i32 {
        let id = self.constants.len() as u32;

        const K_MAX_CONSTANT_COUNT: u32 = 0x007f_ffff;
        if id >= K_MAX_CONSTANT_COUNT {
            return -1;
        }

        let mut c = Constant {
            r#type: Type::Type_ClassShape,
            value: unsafe { core::mem::zeroed() },
        };

        unsafe {
            c.value.valueClassShape = self.class_shapes.len() as u32;
        }

        self.class_shapes.push(shape);
        self.constants.push(c);

        id as i32
    }
}
