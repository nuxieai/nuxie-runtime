use crate::records::bc_call_fb::BcCallFB;
use crate::records::bc_function::BcFunction;
use crate::records::bc_move::BcMove;
use crate::records::bc_op::BcOp;
use crate::records::bc_op_hash::BcOpHash;
use crate::type_aliases::reg::Reg;
use alloc::vec::Vec;
use luaur_common::records::dense_hash_map::DenseHashMap;
use luaur_common::records::dense_hash_set::DenseHashSet;

#[derive(Debug)]
pub struct CallInliner<'a> {
    pub(crate) caller: &'a mut BcFunction,
    pub(crate) target: &'a mut BcFunction,
    pub(crate) call: BcCallFB<'a>,
    pub(crate) call_params: Vec<BcOp>,
    pub(crate) target_reg: Reg,

    pub(crate) caller_blocks_size_before_inline: u32,
    pub(crate) caller_inst_size_before_inline: u32,
    pub(crate) caller_vm_const_size_before_inline: u32,
    pub(crate) caller_proto_size_before_inline: u32,
    pub(crate) caller_up_val_size_before_inline: u8,

    pub(crate) return_ops: Vec<BcOp>,
    pub(crate) call_projections: DenseHashSet<BcOp, BcOpHash>,
    pub(crate) var_arg_moves: DenseHashMap<BcOp, Vec<BcOp>, BcOpHash>,
}

impl<'a> CallInliner<'a> {
    pub(crate) const K_MAX_INLINER_COMBINED_STACK_SIZE: u32 = 256;
}
