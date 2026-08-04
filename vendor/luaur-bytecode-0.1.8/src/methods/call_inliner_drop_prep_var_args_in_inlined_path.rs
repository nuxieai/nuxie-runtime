use crate::records::bc_block::BcBlock;
use crate::records::bc_function::BcFunction;
use crate::records::bc_inst::BcInst;
use crate::records::bc_op::BcOp;
use crate::records::bc_ref::BcRef;
use crate::records::call_inliner::CallInliner;
use luaur_common::enums::luau_opcode::LuauOpcode::LOP_PREPVARARGS;

impl<'a> CallInliner<'a> {
    pub(crate) fn drop_prep_var_args_in_inlined_path(&mut self) {
        let target_entry_block = self.target.entry_block;
        let mapped_block_op = self.map_block_op(target_entry_block);
        let inlined_entry_block: BcRef<BcBlock> = self.caller.block(mapped_block_op);

        let block_mut = unsafe { &mut *BcRef::operator_arrow(&inlined_entry_block) };

        if !block_mut.ops.is_empty() {
            let first_op = block_mut.ops.front().unwrap();
            let first_inst = self.caller.inst_op(*first_op);

            if first_inst.op == LOP_PREPVARARGS {
                block_mut.ops.pop_front();
            }
        }
    }
}
