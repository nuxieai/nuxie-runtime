use crate::records::bytecode_builder::BytecodeBuilder;
use crate::records::bytecode_graph_serializer::BytecodeGraphSerializer;
use crate::records::comp_time_bytecode_graph_serializer::CompTimeBytecodeGraphSerializer;
use crate::type_aliases::comp_time_bc_function::CompTimeBcFunction;
use alloc::vec::Vec;

// This implementation is disabled via #[cfg(any())] to avoid E0592 (duplicate definitions)
// because the record file `comp_time_bytecode_graph_serializer.rs` already contains
// the definition of this constructor.
#[cfg(any())]
impl CompTimeBytecodeGraphSerializer {
    pub fn comp_time_bytecode_graph_serializer_comp_time_bytecode_graph_serializer(
        bcb: &mut BytecodeBuilder,
        fn_: &mut CompTimeBcFunction,
        consts: &mut Vec<u16>,
    ) -> Self {
        Self {
            base: BytecodeGraphSerializer::bytecode_graph_serializer_bytecode_graph_serializer(
                bcb, fn_,
            ),
            consts: consts.clone(),
        }
    }
}
