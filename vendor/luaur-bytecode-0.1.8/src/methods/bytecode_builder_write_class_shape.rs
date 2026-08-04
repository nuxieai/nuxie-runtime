use crate::functions::write_var_int::writeVarInt;
use crate::records::bytecode_builder::BytecodeBuilder;
use crate::records::class_shape::ClassShape;
use alloc::string::String;

impl BytecodeBuilder {
    pub fn write_class_shape(&self, ss: &mut String, cs: &ClassShape) {
        writeVarInt(ss, cs.className as u64);
        writeVarInt(ss, cs.propertyNames.len() as u64);
        writeVarInt(ss, cs.methodNames.len() as u64);

        for &prop_name in &cs.propertyNames {
            writeVarInt(ss, prop_name as u64);
        }

        for &method_name in &cs.methodNames {
            writeVarInt(ss, method_name as u64);
        }
    }
}
