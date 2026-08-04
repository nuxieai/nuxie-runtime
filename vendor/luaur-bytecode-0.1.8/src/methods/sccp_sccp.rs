use alloc::boxed::Box;
use alloc::vec::Vec;

use crate::enums::bc_block_edge_kind::BcBlockEdgeKind;
use crate::enums::bc_block_flag::BcBlockFlag;
use crate::enums::bc_imm_kind::BcImmKind;
use crate::enums::bc_op_kind::BcOpKind;
use crate::enums::condition_state::ConditionState;
use crate::enums::constness::Constness;
use crate::records::bc_block_edge::BcBlockEdge;
use crate::records::bc_function::BcFunction;
use crate::records::bc_imm::{BcImm, BcImmValue};
use crate::records::bc_op::BcOp;
use crate::records::constness_lattice::ConstnessLattice;
use crate::records::jump_target::JumpTarget;
use crate::records::sccp::Sccp;
use crate::records::sccp_interpreter::SccpInterpreter;
use crate::records::sccp_state::SccpState;
use crate::records::vm_const_ops::VmConstOps;
use crate::type_aliases::bc_edges::BcEdges;
use crate::type_aliases::reg::Reg;
use luaur_common::enums::luau_capture_type::LuauCaptureType;
use luaur_common::enums::luau_opcode::LuauOpcode;
use luaur_common::functions::is_jump_d::isJumpD;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl<'a> Sccp<'a> {
    pub fn new(func: &mut BcFunction, impl_: &'a dyn VmConstOps) -> Self {
        let mut state = Box::new(SccpState::default());
        let state_ptr = &mut *state as *mut SccpState;
        Self {
            func,
            impl_,
            state,
            interpreter: SccpInterpreter::new(impl_, state_ptr),
            block_uses: Default::default(),
            flow_worklist: Default::default(),
            flow_worklist_set: Default::default(),
            ssa_worklist: Default::default(),
        }
    }

    fn func(&self) -> &BcFunction {
        unsafe { &*self.func }
    }

    fn func_mut(&mut self) -> &mut BcFunction {
        unsafe { &mut *self.func }
    }

    pub fn make_bool_imm(&self, value: bool) -> ConstnessLattice {
        ConstnessLattice::from_imm_const(
            Constness::ImmConstant,
            BcImm {
                kind: BcImmKind::Boolean,
                value: BcImmValue {
                    valueBoolean: value,
                },
            },
        )
    }

    pub fn get_fallthrough(&self, block_op: BcOp) -> Option<BcOp> {
        let block = &self.func().blocks[block_op.index as usize];
        let mut fallthrough = None;
        for successor in block.successors.iter() {
            if successor.kind == BcBlockEdgeKind::Fallthrough {
                if fallthrough.is_some() {
                    LUAU_ASSERT!(false, "Multiple fallthroughs");
                    return None;
                }
                fallthrough = Some(successor.target);
            }
        }
        fallthrough
    }

    pub fn conditional_targets(
        &self,
        inst_op: BcOp,
        target: BcOp,
        condition: ConditionState,
        target_taken_on_true: bool,
    ) -> Vec<JumpTarget> {
        LUAU_ASSERT!(target.kind == BcOpKind::Block);
        let inst = &self.func().instructions[inst_op.index as usize];
        let fallthrough = self
            .get_fallthrough(inst.block)
            .expect("conditional branch fallthrough");
        let mut target_dead = false;
        let mut fallthrough_dead = false;
        if condition == ConditionState::AlwaysTrue {
            target_dead = !target_taken_on_true;
            fallthrough_dead = target_taken_on_true;
        } else if condition == ConditionState::AlwaysFalse {
            target_dead = target_taken_on_true;
            fallthrough_dead = !target_taken_on_true;
        }
        vec![
            JumpTarget {
                dead: target_dead,
                block_op: target,
                condition,
            },
            JumpTarget {
                dead: fallthrough_dead,
                block_op: fallthrough,
                condition,
            },
        ]
    }

    pub fn jump_targets(&mut self, inst_op: BcOp) -> Vec<JumpTarget> {
        let (op, ops) = {
            let inst = &self.func().instructions[inst_op.index as usize];
            (inst.op, inst.ops.clone())
        };
        match op {
            LuauOpcode::LOP_JUMP | LuauOpcode::LOP_JUMPBACK => {
                LUAU_ASSERT!(ops[0].kind == BcOpKind::Block);
                vec![JumpTarget {
                    dead: false,
                    block_op: ops[0],
                    condition: ConditionState::AlwaysTrue,
                }]
            }
            LuauOpcode::LOP_JUMPIF | LuauOpcode::LOP_JUMPIFNOT => {
                let condition = self.interpreter.evaluate_condition(ops[0]);
                self.conditional_targets(inst_op, ops[1], condition, op == LuauOpcode::LOP_JUMPIF)
            }
            LuauOpcode::LOP_JUMPIFEQ
            | LuauOpcode::LOP_JUMPIFLE
            | LuauOpcode::LOP_JUMPIFLT
            | LuauOpcode::LOP_JUMPIFNOTEQ
            | LuauOpcode::LOP_JUMPIFNOTLE
            | LuauOpcode::LOP_JUMPIFNOTLT => {
                let condition = self
                    .interpreter
                    .evaluate_comparison_condition(op, ops[0], ops[1]);
                let negated = matches!(
                    op,
                    LuauOpcode::LOP_JUMPIFNOTEQ
                        | LuauOpcode::LOP_JUMPIFNOTLE
                        | LuauOpcode::LOP_JUMPIFNOTLT
                );
                self.conditional_targets(inst_op, ops[2], condition, !negated)
            }
            LuauOpcode::LOP_JUMPXEQKNIL
            | LuauOpcode::LOP_JUMPXEQKB
            | LuauOpcode::LOP_JUMPXEQKN
            | LuauOpcode::LOP_JUMPXEQKS => {
                let condition = self
                    .interpreter
                    .evaluate_xeqk_condition(unsafe { &*self.func }, inst_op);
                let negated = unsafe {
                    self.func().immediates[ops[1].index as usize]
                        .value
                        .valueBoolean
                };
                self.conditional_targets(inst_op, ops[2], condition, !negated)
            }
            LuauOpcode::LOP_FORNPREP
            | LuauOpcode::LOP_FORNLOOP
            | LuauOpcode::LOP_FORGPREP
            | LuauOpcode::LOP_FORGPREP_NEXT
            | LuauOpcode::LOP_FORGPREP_INEXT => {
                self.conditional_targets(inst_op, ops[3], ConditionState::Unknown, true)
            }
            LuauOpcode::LOP_FORGLOOP => {
                self.conditional_targets(inst_op, ops[5], ConditionState::Unknown, true)
            }
            LuauOpcode::LOP_CMPPROTO => {
                self.conditional_targets(inst_op, ops[2], ConditionState::Unknown, true)
            }
            LuauOpcode::LOP_JUMPX => {
                LUAU_ASSERT!(false, "Should have never parsed this");
                Vec::new()
            }
            _ => Vec::new(),
        }
    }

    pub fn visit_phi(&mut self, phi_op: BcOp) {
        let (ops, uses) = {
            let phi = &self.func().phis[phi_op.index as usize];
            LUAU_ASSERT!(!phi.ops.is_empty());
            (phi.ops.clone(), phi.uses.clone())
        };
        let mut fold = ConstnessLattice::default();
        for op in ops.iter() {
            let lattice = self.state.operand_lattice(op);
            fold = lattice.merge(&fold);
        }
        let previous = *self.state.op_constness.get_or_insert(phi_op);
        if fold != previous {
            self.ssa_worklist.extend(uses);
        }
        *self.state.op_constness.get_or_insert(phi_op) = fold;
    }

    pub fn visit_inst(&mut self, inst_op: BcOp) {
        let (opcode, ops, uses, block) = {
            let inst = &self.func().instructions[inst_op.index as usize];
            (inst.op, inst.ops.clone(), inst.uses.clone(), inst.block)
        };

        if opcode == LuauOpcode::LOP_CAPTURE && ops.len() >= 2 {
            LUAU_ASSERT!(ops[0].kind == BcOpKind::Imm);
            let capture = self.func().immediates[ops[0].index as usize];
            if capture.kind == BcImmKind::Int
                && unsafe { capture.value.valueInt } == LuauCaptureType::LCT_REF as i32
            {
                let source = ops[1];
                let previous = *self.state.op_constness.get_or_insert(source);
                if previous.kind != Constness::NotAConstant
                    && matches!(source.kind, BcOpKind::Inst | BcOpKind::Phi)
                {
                    *self.state.op_constness.get_or_insert(source) =
                        ConstnessLattice::from_kind(Constness::NotAConstant);
                    let source_uses = if source.kind == BcOpKind::Inst {
                        self.func().instructions[source.index as usize].uses.clone()
                    } else {
                        self.func().phis[source.index as usize].uses.clone()
                    };
                    self.ssa_worklist.extend(source_uses);
                }
            }
        }

        let lattice = self
            .interpreter
            .evaluate(opcode, unsafe { &*self.func }, inst_op);
        let previous = *self.state.op_constness.get_or_insert(inst_op);
        let new_value = lattice.merge(&previous);
        if new_value != previous {
            self.ssa_worklist.extend(uses);
        }

        for target in self.jump_targets(inst_op) {
            let block_index = target.block_op.index;
            if !target.dead {
                self.block_uses.get_or_insert(block_index).insert(block);
                if !self.flow_worklist_set.contains(&target.block_op) {
                    self.flow_worklist.push_back(target.block_op);
                }
            }
        }
        *self.state.op_constness.get_or_insert(inst_op) = new_value;
    }

    pub fn propagate(&mut self) {
        let entry = self.func().entry_block;
        let exit = self.func().exit_block;
        self.block_uses.get_or_insert(entry.index).insert(entry);
        self.block_uses.get_or_insert(exit.index).insert(exit);
        self.flow_worklist.push_back(entry);

        while !self.flow_worklist.is_empty() || !self.ssa_worklist.is_empty() {
            while let Some(block_op) = self.flow_worklist.pop_front() {
                if self.flow_worklist_set.contains(&block_op) {
                    continue;
                }
                let (phis, ops, successors) = {
                    let block = &self.func().blocks[block_op.index as usize];
                    (
                        block.phis.clone(),
                        block.ops.clone(),
                        block.successors.clone(),
                    )
                };
                for phi_op in phis {
                    LUAU_ASSERT!(phi_op.kind == BcOpKind::Phi);
                    self.visit_phi(phi_op);
                }
                for op in ops.iter() {
                    LUAU_ASSERT!(op.kind == BcOpKind::Inst);
                    self.visit_inst(*op);
                }
                let block_ends_with_branch = ops
                    .back()
                    .copied()
                    .map(|op| !self.jump_targets(op).is_empty())
                    .unwrap_or(false);

                for successor in successors.iter() {
                    if successor.kind == BcBlockEdgeKind::Fallthrough {
                        let successor_index = successor.target.index;
                        if block_ends_with_branch
                            && !self
                                .block_uses
                                .get_or_insert(successor_index)
                                .contains(&block_op)
                        {
                            continue;
                        }
                        self.block_uses
                            .get_or_insert(successor_index)
                            .insert(block_op);
                        if !self.flow_worklist_set.contains(&successor.target) {
                            self.flow_worklist.push_back(successor.target);
                        }
                    }
                }
                self.flow_worklist_set.insert(block_op);
            }

            while let Some(op) = self.ssa_worklist.pop_front() {
                if op.kind == BcOpKind::Inst {
                    self.visit_inst(op);
                } else if op.kind == BcOpKind::Phi {
                    self.visit_phi(op);
                }
            }
        }
    }

    pub fn make_constant_op(&mut self, lattice: ConstnessLattice) -> BcOp {
        if lattice.kind == Constness::VmConstant {
            let value =
                self.func().constants[lattice.vm_const.expect("VM constant").index as usize];
            self.func_mut().add_const(&value)
        } else if lattice.kind == Constness::ImmConstant {
            self.func_mut()
                .add_imm_value(&lattice.imm_const.expect("immediate constant"))
        } else {
            LUAU_ASSERT!(false, "makeConstantOp called on non-constant lattice value");
            BcOp::new()
        }
    }

    pub fn replace_operand(&mut self, inst_op: BcOp, old_op: BcOp, new_op: BcOp) {
        for operand in self.func_mut().instructions[inst_op.index as usize]
            .ops
            .iter_mut()
        {
            if *operand == old_op {
                *operand = new_op;
            }
        }
    }

    pub fn replace_phi_operand(&mut self, phi_op: BcOp, old_op: BcOp, new_op: BcOp) {
        for operand in self.func_mut().phis[phi_op.index as usize].ops.iter_mut() {
            if *operand == old_op {
                *operand = new_op;
            }
        }
    }

    pub fn is_load_inst(&self, op: BcOp) -> bool {
        op.kind == BcOpKind::Inst
            && matches!(
                self.func().instructions[op.index as usize].op,
                LuauOpcode::LOP_LOADK
                    | LuauOpcode::LOP_LOADKX
                    | LuauOpcode::LOP_LOADN
                    | LuauOpcode::LOP_LOADB
                    | LuauOpcode::LOP_LOADNIL
            )
    }

    pub fn erase_use(&mut self, user_op: BcOp, used_op: BcOp) {
        let uses = if used_op.kind == BcOpKind::Inst {
            &mut self.func_mut().instructions[used_op.index as usize].uses
        } else if used_op.kind == BcOpKind::Phi {
            &mut self.func_mut().phis[used_op.index as usize].uses
        } else {
            return;
        };
        uses.retain(|op| *op != user_op);
    }

    pub fn rewrite_to_load(&mut self, op: BcOp, lattice: ConstnessLattice) {
        let used_ops = self.func().instructions[op.index as usize].ops.clone();
        for used_op in used_ops.iter().copied() {
            self.erase_use(op, used_op);
        }
        let new_operand = if lattice.kind == Constness::VmConstant {
            lattice.vm_const.expect("VM constant lattice")
        } else {
            self.func_mut()
                .add_imm_value(&lattice.imm_const.expect("immediate lattice"))
        };
        let inst = &mut self.func_mut().instructions[op.index as usize];
        inst.ops.clear();
        if lattice.kind == Constness::VmConstant {
            inst.op = LuauOpcode::LOP_LOADK;
        } else {
            let imm = lattice.imm_const.expect("immediate lattice");
            inst.op = if imm.kind == BcImmKind::Boolean {
                LuauOpcode::LOP_LOADB
            } else {
                LuauOpcode::LOP_LOADN
            };
        }
        inst.ops.push(new_operand);
    }

    pub fn erase_op(&mut self, op: BcOp) {
        let block_op = self.func().instructions[op.index as usize].block;
        self.func_mut().blocks[block_op.index as usize]
            .ops
            .retain(|candidate| *candidate != op);
    }

    pub fn remove_dead_edges(&mut self, inst_op: BcOp) {
        let targets = self.jump_targets(inst_op);
        let block_op = self.func().instructions[inst_op.index as usize].block;
        let mut live_target = None;
        for target in targets {
            if target.dead {
                let successors = &self.func().blocks[block_op.index as usize].successors;
                let filtered: BcEdges = successors
                    .iter()
                    .copied()
                    .filter(|edge| edge.target != target.block_op)
                    .collect();
                self.func_mut().blocks[block_op.index as usize].successors = filtered;
            } else {
                live_target = Some(target.block_op);
            }
        }
        let Some(live_target) = live_target else {
            return;
        };
        let block = &mut self.func_mut().blocks[block_op.index as usize];
        if let Some(edge) = block
            .successors
            .iter_mut()
            .find(|edge| edge.target == live_target)
        {
            edge.kind = BcBlockEdgeKind::Fallthrough;
        } else {
            block.successors.push(BcBlockEdge {
                kind: BcBlockEdgeKind::Fallthrough,
                target: live_target,
            });
        }
    }

    pub fn replace_uses(&mut self) {
        let constants: Vec<(BcOp, ConstnessLattice)> = self
            .state
            .op_constness
            .iter()
            .map(|(op, lattice)| (*op, *lattice))
            .collect();
        for (op, lattice) in constants {
            if !matches!(lattice.kind, Constness::ImmConstant | Constness::VmConstant)
                || op.kind != BcOpKind::Inst
                || self.is_load_inst(op)
            {
                continue;
            }
            let opcode = self.func().instructions[op.index as usize].op;
            LUAU_ASSERT!(opcode != LuauOpcode::LOP_JUMPX);
            if isJumpD(opcode) {
                self.remove_dead_edges(op);
                self.erase_op(op);
            } else {
                self.rewrite_to_load(op, lattice);
            }
        }
    }

    fn uses_of(&self, op: BcOp) -> Vec<BcOp> {
        if op.kind == BcOpKind::Inst {
            self.func().instructions[op.index as usize].uses.clone()
        } else {
            LUAU_ASSERT!(op.kind == BcOpKind::Phi);
            self.func().phis[op.index as usize].uses.clone()
        }
    }

    fn push_use(&mut self, op: BcOp, user: BcOp) {
        if op.kind == BcOpKind::Inst {
            self.func_mut().instructions[op.index as usize]
                .uses
                .push(user);
        } else {
            LUAU_ASSERT!(op.kind == BcOpKind::Phi);
            self.func_mut().phis[op.index as usize].uses.push(user);
        }
    }

    pub fn simplify_phis(&mut self) {
        let blocks: Vec<BcOp> = self.flow_worklist_set.iter().copied().collect();
        for block_op in blocks {
            let phis: Vec<BcOp> = self.func().blocks[block_op.index as usize]
                .phis
                .iter()
                .copied()
                .collect();
            for op in phis {
                LUAU_ASSERT!(op.kind == BcOpKind::Phi);
                let operands = self.func().phis[op.index as usize].ops.clone();
                if operands.is_empty() {
                    continue;
                }
                let unique = operands[0];
                if operands.iter().skip(1).any(|operand| *operand != unique) {
                    continue;
                }
                for use_op in self.uses_of(op) {
                    if use_op.kind == BcOpKind::Inst {
                        self.replace_operand(use_op, op, unique);
                    } else if use_op.kind == BcOpKind::Phi {
                        self.replace_phi_operand(use_op, op, unique);
                    }
                    self.push_use(unique, use_op);
                }
                self.func_mut().phis[op.index as usize].uses.clear();
                self.func_mut().blocks[block_op.index as usize]
                    .phis
                    .retain(|candidate| *candidate != op);
            }
        }
    }

    pub fn update_block_uses(&mut self) {
        let entry = self.func().entry_block.index;
        let exit = self.func().exit_block.index;
        let mut reachable = luaur_common::records::dense_hash_set2::DenseHashSet2::<u32>::new();
        let mut worklist = vec![entry];
        reachable.insert(entry);
        reachable.insert(exit);
        while let Some(index) = worklist.pop() {
            let successors = self.func().blocks[index as usize].successors.clone();
            for edge in successors.iter() {
                let successor = edge.target.index;
                if !reachable.contains(&successor) {
                    reachable.insert(successor);
                    worklist.push(successor);
                }
            }
        }
        for index in 0..self.func().blocks.len() {
            let count = self.block_uses.get_or_insert(index as u32).size() as u32;
            let block = &mut self.func_mut().blocks[index];
            block.useCount = count;
            if !reachable.contains(&(index as u32)) {
                block.flags |= BcBlockFlag::Dead as u8;
            }
        }
    }

    pub fn arith_to_k_opcode(op: LuauOpcode) -> Option<LuauOpcode> {
        match op {
            LuauOpcode::LOP_ADD => Some(LuauOpcode::LOP_ADDK),
            LuauOpcode::LOP_SUB => Some(LuauOpcode::LOP_SUBK),
            LuauOpcode::LOP_MUL => Some(LuauOpcode::LOP_MULK),
            LuauOpcode::LOP_DIV => Some(LuauOpcode::LOP_DIVK),
            LuauOpcode::LOP_MOD => Some(LuauOpcode::LOP_MODK),
            LuauOpcode::LOP_POW => Some(LuauOpcode::LOP_POWK),
            _ => None,
        }
    }

    pub fn is_pure_producer(&self, op: LuauOpcode) -> bool {
        matches!(
            op,
            LuauOpcode::LOP_LOADK
                | LuauOpcode::LOP_LOADKX
                | LuauOpcode::LOP_LOADN
                | LuauOpcode::LOP_LOADB
                | LuauOpcode::LOP_LOADNIL
                | LuauOpcode::LOP_GETUPVAL
        )
    }

    pub fn register_of(&self, op: BcOp) -> Option<Reg> {
        if op.kind == BcOpKind::VmReg {
            Some(op.index as Reg)
        } else {
            self.func().regs.get(&op).copied()
        }
    }

    pub fn erase_dead_producer(&mut self, op: BcOp) {
        if op.kind != BcOpKind::Inst {
            return;
        }
        let inst = &self.func().instructions[op.index as usize];
        if self.is_pure_producer(inst.op) && inst.uses.is_empty() {
            self.erase_op(op);
        }
    }

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
                if value == 0.0 && matches!(opcode, LuauOpcode::LOP_ADD | LuauOpcode::LOP_SUB) {
                    let inst = &mut self.func_mut().instructions[op.index as usize];
                    inst.op = LuauOpcode::LOP_MOVE;
                    inst.ops.clear();
                    inst.ops.push(non_constant);
                } else if value == 0.0 && opcode == LuauOpcode::LOP_MUL {
                    let immediate = self.func_mut().add_imm_value(&BcImm {
                        kind: BcImmKind::Int,
                        value: BcImmValue { valueInt: 0 },
                    });
                    let inst = &mut self.func_mut().instructions[op.index as usize];
                    inst.op = LuauOpcode::LOP_LOADN;
                    inst.ops.clear();
                    inst.ops.push(immediate);
                } else if value == 0.0 && opcode == LuauOpcode::LOP_POW {
                    let immediate = self.func_mut().add_imm_value(&BcImm {
                        kind: BcImmKind::Int,
                        value: BcImmValue { valueInt: 1 },
                    });
                    let inst = &mut self.func_mut().instructions[op.index as usize];
                    inst.op = LuauOpcode::LOP_LOADN;
                    inst.ops.clear();
                    inst.ops.push(immediate);
                } else if value == 1.0
                    && matches!(
                        opcode,
                        LuauOpcode::LOP_MUL | LuauOpcode::LOP_POW | LuauOpcode::LOP_DIV
                    )
                {
                    let inst = &mut self.func_mut().instructions[op.index as usize];
                    inst.op = LuauOpcode::LOP_MOVE;
                    inst.ops.clear();
                    inst.ops.push(non_constant);
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

    pub fn rewrite(&mut self) {
        self.arith_to_k();
        self.replace_uses();
        self.simplify_phis();
        self.update_block_uses();
    }
}
