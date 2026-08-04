use crate::records::bc_function::BcFunction;
use crate::records::bytecode_builder::BytecodeBuilder;
use crate::records::bytecode_graph_serializer::BytecodeGraphSerializer;

impl BytecodeGraphSerializer<'_> {
    pub fn bytecode_graph_serializer_bytecode_graph_serializer(
        bcb: &mut BytecodeBuilder,
        func: &mut BcFunction,
    ) -> Self {
        // SAFETY: Mirror the C++ constructor which stores references passed by the caller.
        // The returned serializer is parameterized to match the lifetime of those references.
        unsafe {
            core::mem::transmute::<BytecodeGraphSerializer<'_>, BytecodeGraphSerializer<'_>>(
                BytecodeGraphSerializer::new(bcb, func),
            )
        }
    }
}
