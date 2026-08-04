use crate::enums::bc_block_edge_kind::BcBlockEdgeKind;
use crate::enums::bc_op_kind::BcOpKind;
use crate::records::bc_block::BcBlock;
use crate::records::bc_function::VmConst;
use crate::records::bc_inst_helper::BcInstHelper;
use crate::records::bc_jump::BcJump;
use crate::records::bc_op::BcOp;
use crate::records::bytecode_graph_serializer::BytecodeGraphSerializer;
use alloc::vec::Vec;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl<'a> BytecodeGraphSerializer<'a> {
    pub fn emit_bytecode(&mut self) -> Vec<u32> {
        let schedule = self.reschedule();
        let mut insns_pc: Vec<u32> = Vec::new();
        insns_pc.resize(self.func.instructions.len(), 0);

        for i in 0..schedule.len() {
            let block_op = schedule[i as usize];
            let fallthrough = {
                let block: &BcBlock = &self.func.blocks[block_op.index as usize];
                block
                    .successors
                    .iter()
                    .find(|edge| edge.kind == BcBlockEdgeKind::Fallthrough)
                    .map(|edge| edge.target)
            };
            if let Some(fallthrough_op) = fallthrough {
                if fallthrough_op != self.func.exit_block
                    && (i + 1 >= schedule.len() || fallthrough_op != schedule[(i + 1) as usize])
                {
                    let mut jump = BcJump::<VmConst>::create(self.func);
                    jump.set_target(fallthrough_op);
                    jump.append_to(block_op);
                    insns_pc.resize(self.func.instructions.len(), 0);
                }
            }
            let ops = {
                let block: &mut BcBlock = self.func.block_op(block_op);
                block.startpc = self.bcb.get_debug_pc();
                block.ops.iter().cloned().collect::<Vec<_>>()
            };
            for op in ops {
                LUAU_ASSERT!(op.kind == BcOpKind::Inst);
                insns_pc[op.index as usize] = self.bcb.get_debug_pc();
                self.emit_instruction(op);
            }
        }

        let mut jumps = core::mem::take(&mut self.jumps);
        for jump in jumps.iter_mut() {
            self.patch_jump(jump);
        }
        self.jumps = jumps;

        if self.error {
            Vec::new()
        } else {
            insns_pc
        }
    }
}
