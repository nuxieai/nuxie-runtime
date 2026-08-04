use crate::enums::bc_imm_kind::BcImmKind;
use crate::enums::constness::Constness;
use crate::records::bc_imm::{BcImm, BcImmValue};
use crate::records::bc_op::BcOp;
use crate::records::constness_lattice::ConstnessLattice;
use crate::records::sccp::Sccp;
use alloc::vec::Vec;
use luaur_common::enums::luau_opcode::LuauOpcode;

impl<'func, 'ops> Sccp<'func, 'ops> {
    pub fn arith_to_k(&mut self) {
        for block_index in 0..self.func().blocks.len() {
            if self.block_uses.get_or_insert(block_index as u32).is_empty() {
                continue;
            }
            let ops: Vec<BcOp> = self.func().blocks[block_index]
                .ops
                .iter()
                .copied()
                .collect();
            let mut to_erase = Vec::new();
            for op in ops {
                let (opcode, operands) = {
                    let inst = &self.func().instructions[op.index as usize];
                    (inst.op, inst.ops.clone())
                };
                let Some(mut k_opcode) = Self::arith_to_k_opcode(opcode) else {
                    continue;
                };
                if operands.len() != 2 {
                    continue;
                }
                let lhs = operands[0];
                let rhs = operands[1];
                let lhs_lattice = self.state.operand_lattice(&lhs);
                let rhs_lattice = self.state.operand_lattice(&rhs);
                let is_const_number = |lattice: &ConstnessLattice| {
                    lattice.kind == Constness::VmConstant
                        && lattice.vm_const.is_some()
                        && self
                            .impl_
                            .is_arithmetic_constant(&lattice.vm_const.expect("VM constant"))
                };
                let (non_constant, constant, rk) = if is_const_number(&rhs_lattice)
                    && lhs_lattice.kind == Constness::NotAConstant
                {
                    (lhs, rhs_lattice, false)
                } else if is_const_number(&lhs_lattice)
                    && rhs_lattice.kind == Constness::NotAConstant
                    && matches!(
                        opcode,
                        LuauOpcode::LOP_ADD
                            | LuauOpcode::LOP_MUL
                            | LuauOpcode::LOP_SUB
                            | LuauOpcode::LOP_DIV
                    )
                {
                    let rk = if opcode == LuauOpcode::LOP_SUB {
                        k_opcode = LuauOpcode::LOP_SUBRK;
                        true
                    } else if opcode == LuauOpcode::LOP_DIV {
                        k_opcode = LuauOpcode::LOP_DIVRK;
                        true
                    } else {
                        false
                    };
                    (rhs, lhs_lattice, rk)
                } else {
                    continue;
                };
                let previous_constant = if non_constant == lhs { rhs } else { lhs };
                let value = self
                    .impl_
                    .as_number(&constant.vm_const.expect("VM constant"));
                if value == 0.0 {
                    if matches!(opcode, LuauOpcode::LOP_ADD | LuauOpcode::LOP_SUB) {
                        let inst = &mut self.func_mut().instructions[op.index as usize];
                        inst.op = LuauOpcode::LOP_MOVE;
                        inst.ops.clear();
                        inst.ops.push(non_constant);
                    } else if opcode == LuauOpcode::LOP_MUL {
                        let immediate = self.func_mut().add_imm_value(&BcImm {
                            kind: BcImmKind::Int,
                            value: BcImmValue { valueInt: 0 },
                        });
                        let inst = &mut self.func_mut().instructions[op.index as usize];
                        inst.op = LuauOpcode::LOP_LOADN;
                        inst.ops.clear();
                        inst.ops.push(immediate);
                    } else if opcode == LuauOpcode::LOP_POW {
                        let immediate = self.func_mut().add_imm_value(&BcImm {
                            kind: BcImmKind::Int,
                            value: BcImmValue { valueInt: 1 },
                        });
                        let inst = &mut self.func_mut().instructions[op.index as usize];
                        inst.op = LuauOpcode::LOP_LOADN;
                        inst.ops.clear();
                        inst.ops.push(immediate);
                    }
                } else if value == 1.0 {
                    if matches!(
                        opcode,
                        LuauOpcode::LOP_MUL | LuauOpcode::LOP_POW | LuauOpcode::LOP_DIV
                    ) {
                        let inst = &mut self.func_mut().instructions[op.index as usize];
                        inst.op = LuauOpcode::LOP_MOVE;
                        inst.ops.clear();
                        inst.ops.push(non_constant);
                    }
                } else {
                    let inst = &mut self.func_mut().instructions[op.index as usize];
                    inst.op = k_opcode;
                    inst.ops.clear();
                    if rk {
                        inst.ops.push(constant.vm_const.expect("VM constant"));
                        inst.ops.push(non_constant);
                    } else {
                        inst.ops.push(non_constant);
                        inst.ops.push(constant.vm_const.expect("VM constant"));
                    }
                }
                to_erase.push(previous_constant);
            }
            for op in to_erase {
                self.erase_dead_producer(op);
            }
        }
    }
}
