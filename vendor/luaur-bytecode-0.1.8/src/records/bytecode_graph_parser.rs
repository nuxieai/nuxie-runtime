use crate::records::bc_function::BcFunction;
use crate::records::bc_op::BcOp;
use crate::records::block_producers::BlockProducers;
use alloc::vec::Vec;
use luaur_common::records::dense_hash_map::DenseHashMap;

#[derive(Debug)]
pub struct BytecodeGraphParser<'a> {
    pub(crate) func: &'a mut BcFunction,
    pub(crate) block_by_pc: DenseHashMap<u32, BcOp>,
    pub(crate) producers: Vec<BlockProducers>,
    pub(crate) current_block: BcOp,
}

impl<'a> BytecodeGraphParser<'a> {
    pub(crate) const K_MAX_CFG_BLOCKS: u32 = 1000;
}
