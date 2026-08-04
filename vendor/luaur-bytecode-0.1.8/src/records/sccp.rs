use alloc::boxed::Box;
use alloc::collections::VecDeque;

use crate::records::bc_function::BcFunction;
use crate::records::bc_op::BcOp;
use crate::records::bc_op_hash::BcOpHash;
use crate::records::sccp_interpreter::SccpInterpreter;
use crate::records::sccp_state::SccpState;
use crate::records::vm_const_ops::VmConstOps;
use luaur_common::records::dense_hash_map2::DenseHashMap2;
use luaur_common::records::dense_hash_set2::DenseHashSet2;

pub struct Sccp<'a> {
    pub(crate) func: *mut BcFunction,
    pub(crate) impl_: &'a dyn VmConstOps,
    pub(crate) state: Box<SccpState>,
    pub(crate) interpreter: SccpInterpreter<'a>,
    pub(crate) block_uses: DenseHashMap2<u32, DenseHashSet2<BcOp, BcOpHash>>,
    pub(crate) flow_worklist: VecDeque<BcOp>,
    pub(crate) flow_worklist_set: DenseHashSet2<BcOp, BcOpHash>,
    pub(crate) ssa_worklist: VecDeque<BcOp>,
}
