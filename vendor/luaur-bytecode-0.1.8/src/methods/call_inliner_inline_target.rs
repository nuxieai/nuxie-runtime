use crate::enums::bc_block_edge_kind::BcBlockEdgeKind;
use crate::enums::bc_block_edge_kind::BcBlockEdgeKind::Fallthrough;
use crate::enums::bc_block_edge_kind::BcBlockEdgeKind::Fallthrough as FallthroughKind;
use crate::enums::bc_block_edge_kind::BcBlockEdgeKind::{Branch, Loop};
use crate::enums::bc_block_flag::BcBlockFlag;
use crate::enums::bc_op_kind::BcOpKind;
use crate::enums::bc_op_kind::BcOpKind::Block;
use crate::records::bc_block::BcBlock;
use crate::records::bc_block_edge::BcBlockEdge;
use crate::records::bc_function::BcFunction;
use crate::records::bc_inst::BcInst;
use crate::records::bc_op::BcOp;
use crate::records::bc_ref::BcRef;
use crate::records::call_inliner::CallInliner;
use luaur_common::enums::luau_opcode::LuauOpcode;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

const k_max_inliner_combined_stack_size: u8 = 250;

impl<'a> CallInliner<'a> {
    pub fn inline_target(&mut self, target_proto_id: u32) -> bool {
        let mut new_max_stack_size: u32 =
            self.caller.maxstacksize as u32 + self.target.maxstacksize as u32;

        if self.target.is_vararg {
            new_max_stack_size = new_max_stack_size.wrapping_add(self.call_params.len() as u32);
        }

        if new_max_stack_size >= k_max_inliner_combined_stack_size as u32 {
            return false;
        }

        if self.call.param_count() < 0 || self.call.return_count() < 0 {
            return false;
        }

        // inlining of upvalues is not supported yet
        if self.target.nups > 0 {
            return false;
        }

        self.caller.maxstacksize = new_max_stack_size as u8;

        let (mut prev_block, mut next_block) = self.split_block_on_op(self.call.op());

        let mut target_op: BcOp = self.call.target();

        if prev_block.operator_deref().ops.len() > 0 {
            let last_inst = self
                .caller
                .inst(prev_block.operator_deref().ops.back().copied().unwrap());
            if last_inst.operator_deref().op == LuauOpcode::LOP_NAMECALL {
                target_op = self.replace_namecall(last_inst.op, &mut prev_block);
            }
        }

        self.append_cmp_proto(&mut prev_block, target_op, target_proto_id);

        self.allocate_graph_entities_for_target();

        self.fill_under_call_arguments();

        self.find_target_call_projections();

        if !self.migrate_blocks(&mut next_block) {
            return false;
        }

        let caller_inlined_entry_op = self.map_block_op(self.target.entry_block);

        // Remove prevBlock fallthrough to call block from its predecessors
        let call_block = self.call.base.operator_deref().block;
        let mut call_block_ref = self.caller.block(call_block);
        let insn_preds: &mut crate::type_aliases::bc_edges::BcEdges =
            &mut call_block_ref.operator_deref_mut().predecessors;

        let prev_block_op = prev_block.op;

        let retained: Vec<_> = insn_preds
            .iter()
            .cloned()
            .filter(|p| !(p.kind == BcBlockEdgeKind::Fallthrough && p.target == prev_block_op))
            .collect();

        insn_preds.clear();
        for edge in retained {
            insn_preds.push_back(edge);
        }

        {
            let edges = &mut prev_block.operator_deref_mut().successors;
            let mut found = false;
            for edge in edges.iter_mut() {
                if edge.kind == BcBlockEdgeKind::Fallthrough {
                    edge.target = caller_inlined_entry_op;
                    found = true;
                    break;
                }
            }
            if !found {
                edges.push_back(BcBlockEdge {
                    kind: BcBlockEdgeKind::Fallthrough,
                    target: caller_inlined_entry_op,
                });
            }
        }
        {
            let edges =
                &mut self.caller.blocks[caller_inlined_entry_op.index as usize].predecessors;
            let mut found = false;
            for edge in edges.iter_mut() {
                if edge.kind == BcBlockEdgeKind::Fallthrough {
                    edge.target = prev_block.op;
                    found = true;
                    break;
                }
            }
            if !found {
                edges.push_back(BcBlockEdge {
                    kind: BcBlockEdgeKind::Fallthrough,
                    target: prev_block.op,
                });
            }
        }

        self.migrate_instructions();

        self.replace_call_usages_with_return_phis();

        self.drop_prep_var_args_in_inlined_path();

        LUAU_ASSERT!(self.validate_cfg());

        true
    }
}
