use crate::enums::bc_block_flag::BcBlockFlag;
use crate::enums::bc_op_kind::BcOpKind;
use crate::records::bc_block::BcBlock;
use crate::records::bc_block_edge::BcBlockEdge;
use crate::records::bc_op::BcOp;
use crate::records::bc_ref::BcRef;
use crate::records::call_inliner::CallInliner;
use luaur_common::enums::luau_opcode::LuauOpcode;

impl<'a> CallInliner<'a> {
    pub fn migrate_blocks(&mut self, next_block: &mut BcRef<'a, BcBlock>) -> bool {
        let call_block = self.call.base.operator_deref().block;
        let call_block_ref = self.caller.block(call_block);
        let insn_block_sort_key = call_block_ref.operator_deref().sortkey;
        let insn_block_chain_key = call_block_ref.operator_deref().chainkey;
        let mut max_chain_key = 0;

        for i in 0..self.target.blocks.len() {
            let target_block_sortkey = self.target.blocks[i].sortkey;
            let target_block_ops: Vec<BcOp> = self.target.blocks[i].ops.iter().cloned().collect();

            let caller_block_idx = (self.caller_blocks_size_before_inline + i as u32) as usize;
            let caller_block_op = BcOp::bc_op_bc_op_kind_u32(
                BcOpKind::Block,
                self.caller_blocks_size_before_inline + i as u32,
            );

            if i as u32 == self.target.exit_block.index {
                let caller_block = &mut self.caller.blocks[caller_block_idx];
                caller_block.sortkey = BcBlock::K_BLOCK_NO_START_PC;
                caller_block.flags |= BcBlockFlag::Dead as u8;
                continue;
            }

            {
                let caller_block = &mut self.caller.blocks[caller_block_idx];
                caller_block.sortkey = insn_block_sort_key;
                caller_block.chainkey = insn_block_chain_key + target_block_sortkey;
                max_chain_key = std::cmp::max(caller_block.chainkey, max_chain_key);
            }

            // Migrate successors
            let target_successors = self.target.blocks[i].successors.clone();
            for e in &target_successors {
                if e.target != self.target.exit_block {
                    let mapped_target = self.map_block_op(e.target);
                    let caller_block = &mut self.caller.blocks[caller_block_idx];
                    caller_block.successors.push_back(BcBlockEdge {
                        kind: e.kind,
                        target: mapped_target,
                    });
                }
            }

            // Migrate predecessors
            let target_predecessors = self.target.blocks[i].predecessors.clone();
            for e in &target_predecessors {
                let mapped_target = self.map_block_op(e.target);
                let caller_block = &mut self.caller.blocks[caller_block_idx];
                caller_block.predecessors.push_back(BcBlockEdge {
                    kind: e.kind,
                    target: mapped_target,
                });
            }

            // Migrate instructions
            for op in target_block_ops {
                let inst_op_code = self.target.inst_op(op).op;
                if inst_op_code == LuauOpcode::LOP_GETVARARGS {
                    self.replace_get_var_arg(caller_block_op, op);
                } else if inst_op_code == LuauOpcode::LOP_RETURN {
                    if !self.replace_return(next_block, caller_block_op, op) {
                        return false;
                    }
                } else {
                    let caller_inst_op = self.map_inst_op(op);
                    self.caller.blocks[caller_block_idx].append_instruction(caller_inst_op);
                    self.caller.inst_op(caller_inst_op).block = caller_block_op;
                }
            }
        }

        self.caller.block(call_block).operator_deref_mut().chainkey = max_chain_key + 1;
        next_block.operator_deref_mut().chainkey = max_chain_key + 2;

        true
    }
}
