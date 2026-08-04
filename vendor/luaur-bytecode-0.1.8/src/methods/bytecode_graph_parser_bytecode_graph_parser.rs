use crate::records::bc_function::BcFunction;
use crate::records::bc_op::BcOp;
use crate::records::bytecode_graph_parser::BytecodeGraphParser;
use alloc::vec::Vec;
use luaur_common::records::dense_hash_map::DenseHashMap;

impl<'a> BytecodeGraphParser<'a> {
    pub fn bytecode_graph_parser_bytecode_graph_parser(func: &'a mut BcFunction) -> Self {
        Self {
            func,
            block_by_pc: DenseHashMap::new(u32::MAX - 1),
            producers: Vec::new(),
            current_block: BcOp::new(),
        }
    }

    pub fn new(func: &'a mut BcFunction) -> Self {
        Self::bytecode_graph_parser_bytecode_graph_parser(func)
    }
}
