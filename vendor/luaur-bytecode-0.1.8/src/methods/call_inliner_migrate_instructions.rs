use crate::enums::bc_op_kind::BcOpKind;
use crate::records::bc_inst::BcInst;
use crate::records::bc_op::BcOp;
use crate::records::bc_ref::BcRef;
use crate::records::call_inliner::CallInliner;
use crate::type_aliases::bc_ops::BcOps;

use alloc::vec::Vec;
use luaur_common::enums::luau_opcode::LuauOpcode;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl<'a> CallInliner<'a> {
    pub fn migrate_instructions(&mut self) {
        for i in 0..self.target.instructions.len() as u32 {
            let target_insn_op = BcOp::bc_op_bc_op_kind_u32(BcOpKind::Inst, i);
            let caller_insn_op =
                BcOp::bc_op_bc_op_kind_u32(BcOpKind::Inst, self.caller_inst_size_before_inline + i);
            let target_inst_data = {
                let target_inst = self.target.inst(target_insn_op);
                target_inst.operator_deref().clone()
            };
            let target_ops: Vec<BcOp> = target_inst_data.ops.iter().copied().collect();
            let target_reg = self.target.regs.get(&target_insn_op).copied();
            let is_multi_consumer = match target_inst_data.op {
                LuauOpcode::LOP_SETLIST
                | LuauOpcode::LOP_RETURN
                | LuauOpcode::LOP_CALLFB
                | LuauOpcode::LOP_CALL => {
                    let imm_op =
                        target_inst_data.ops[if target_inst_data.op == LuauOpcode::LOP_SETLIST {
                            1
                        } else {
                            0
                        }];
                    unsafe { self.target.immediates[imm_op.index as usize].value.valueInt < 0 }
                }
                _ => false,
            };

            if target_inst_data.op == LuauOpcode::LOP_RETURN
                || target_inst_data.op == LuauOpcode::LOP_GETVARARGS
            {
                continue;
            }

            LUAU_ASSERT!(target_inst_data.block.kind == BcOpKind::Block);
            let mapped_block = BcOp::bc_op_bc_op_kind_u32(
                BcOpKind::Block,
                self.caller_blocks_size_before_inline + target_inst_data.block.index,
            );
            self.caller.instructions[caller_insn_op.index as usize].op = target_inst_data.op;
            self.caller.instructions[caller_insn_op.index as usize].block = mapped_block;

            let last = *target_ops.last().unwrap();
            let last_is_get_var_arg = last.kind == BcOpKind::Inst
                && self.target.instructions[last.index as usize].op == LuauOpcode::LOP_GETVARARGS;

            if self.target.is_vararg && is_multi_consumer && last_is_get_var_arg {
                for inp in target_ops {
                    if inp != last {
                        let mapped = self.map_to_caller_op(inp);
                        self.caller.instructions[caller_insn_op.index as usize]
                            .ops
                            .push_back(mapped);
                    } else {
                        LUAU_ASSERT!(self.var_arg_moves.contains_key(&inp));
                        let moves = self.var_arg_moves.get(&inp).unwrap().clone();
                        for move_op in moves {
                            self.caller.instructions[caller_insn_op.index as usize]
                                .ops
                                .push_back(move_op);
                        }
                    }
                }
                let instructions = &self.caller.instructions as *const Vec<BcInst>;
                let mut caller_inst = BcRef {
                    vec: unsafe { &*instructions },
                    op: caller_insn_op,
                };
                self.make_fixed_consumer(&mut caller_inst);
            } else {
                for inp in target_ops {
                    let mapped = self.map_to_caller_op(inp);
                    self.caller.instructions[caller_insn_op.index as usize]
                        .ops
                        .push_back(mapped);
                }
            }

            if let Some(reg) = target_reg {
                let mapped_reg = self.map_to_caller_reg(reg);
                self.caller.regs.insert(caller_insn_op, mapped_reg);
            }
        }
    }
}
