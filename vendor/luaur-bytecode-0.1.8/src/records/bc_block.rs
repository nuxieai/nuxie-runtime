use crate::records::bc_op::BcOp;
use crate::type_aliases::bc_edges::BcEdges;
use alloc::collections::VecDeque;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BcBlock {
    pub flags: u8,
    pub useCount: u32,
    pub ops: VecDeque<BcOp>,
    pub successors: BcEdges,
    pub predecessors: BcEdges,
    pub sortkey: u32,
    pub chainkey: u32,
    pub startpc: u32,
}

impl BcBlock {
    pub const K_BLOCK_NO_START_PC: u32 = !0u32;
}

impl Default for BcBlock {
    fn default() -> Self {
        Self {
            flags: 0,
            useCount: 0,
            ops: VecDeque::new(),
            successors: BcEdges::default(),
            predecessors: BcEdges::default(),
            sortkey: !0u32,
            chainkey: 0,
            startpc: Self::K_BLOCK_NO_START_PC,
        }
    }
}
