use crate::enums::r#type::Type;
use crate::records::bytecode_builder::BytecodeBuilder;
use crate::records::constant::Constant;
use crate::records::table_shape::TableShape;

impl BytecodeBuilder {
    pub fn add_constant_table(&mut self, shape: &TableShape) -> i32 {
        if let Some(cache) = self.table_shape_map.find(shape) {
            return *cache;
        }

        let id = self.constants.len() as u32;

        const K_MAX_CONSTANT_COUNT: u32 = 0x007f_ffff;
        if id >= K_MAX_CONSTANT_COUNT {
            return -1;
        }

        let value = Constant {
            r#type: Type::Type_Table,
            value: crate::records::constant::ConstantValue {
                // C++ `value.valueTable = uint32_t(tableShapes.size())`: valueTable
                // indexes table_shapes, NOT constants. The previous `id` (= constants
                // length) over-indexed table_shapes and panicked in write_function.
                valueTable: self.table_shapes.len() as u32,
            },
        };

        self.table_shape_map.try_insert(shape.clone(), id as i32);
        self.table_shapes.push(shape.clone());
        self.constants.push(value);

        id as i32
    }
}
