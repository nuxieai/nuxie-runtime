use crate::enums::bc_imm_kind::BcImmKind;
use crate::enums::bc_op_kind::BcOpKind;
use crate::enums::condition_state::ConditionState;
use crate::enums::constness::Constness;
use crate::records::bc_imm::BcImm;
use crate::records::bc_op::BcOp;
use crate::records::constness_lattice::ConstnessLattice;
use crate::records::sccp_interpreter::SccpInterpreter;
use luaur_common::enums::luau_opcode::LuauOpcode;
use luaur_common::macros::luau_assert::LUAU_ASSERT;

impl<'a> SccpInterpreter<'a> {
    pub fn new(
        impl_: &'a dyn crate::records::vm_const_ops::VmConstOps,
        state: *mut crate::records::sccp_state::SccpState,
    ) -> Self {
        Self { impl_, state }
    }

    pub fn evaluate_arith(
        &mut self,
        opcode: LuauOpcode,
        func: &crate::records::bc_function::BcFunction,
        inst_op: BcOp,
    ) -> ConstnessLattice {
        let inst = &func.instructions[inst_op.index as usize];
        let lhs = inst.ops[0];
        let rhs = inst.ops[1];
        let state = unsafe { &mut *self.state };
        let lhs_constness = state.operand_lattice(&lhs);
        let rhs_constness = state.operand_lattice(&rhs);

        if lhs_constness.kind == Constness::ImmConstant
            && rhs_constness.kind == Constness::ImmConstant
        {
            let lhs_imm = lhs_constness.imm_const.expect("immediate lattice");
            let rhs_imm = rhs_constness.imm_const.expect("immediate lattice");

            if lhs_imm.kind == BcImmKind::Int && rhs_imm.kind == BcImmKind::Int {
                let lv = unsafe { lhs_imm.value.valueInt };
                let rv = unsafe { rhs_imm.value.valueInt };

                if rv == 0
                    && matches!(
                        opcode,
                        LuauOpcode::LOP_DIV | LuauOpcode::LOP_MOD | LuauOpcode::LOP_IDIV
                    )
                {
                    return ConstnessLattice::from_kind(Constness::NotAConstant);
                }

                if matches!(opcode, LuauOpcode::LOP_DIV | LuauOpcode::LOP_POW) {
                    return ConstnessLattice::from_kind(Constness::NotAConstant);
                }

                let result = match opcode {
                    LuauOpcode::LOP_ADD => i64::from(lv) + i64::from(rv),
                    LuauOpcode::LOP_SUB => i64::from(lv) - i64::from(rv),
                    LuauOpcode::LOP_MUL => i64::from(lv) * i64::from(rv),
                    LuauOpcode::LOP_MOD => {
                        let mut remainder = i64::from(lv) % i64::from(rv);
                        if remainder != 0 && ((lv < 0) != (rv < 0)) {
                            remainder += i64::from(rv);
                        }
                        remainder
                    }
                    LuauOpcode::LOP_IDIV => {
                        let mut result = i64::from(lv) / i64::from(rv);
                        if result < 0 && (i64::from(lv) % i64::from(rv)) != 0 {
                            result -= 1;
                        }
                        result
                    }
                    _ => {
                        LUAU_ASSERT!(false, "Unhandled opcode");
                        return ConstnessLattice::from_kind(Constness::NotAConstant);
                    }
                };

                if result < i64::from(i16::MIN) || result > i64::from(i16::MAX) {
                    return ConstnessLattice::from_kind(Constness::NotAConstant);
                }

                return ConstnessLattice::from_imm_const(
                    Constness::ImmConstant,
                    self.impl_.make_imm_int(result as i32),
                );
            }
        } else if lhs_constness.kind == Constness::VmConstant
            && rhs_constness.kind == Constness::VmConstant
        {
            if let Some(vm_const) = self.impl_.evaluate(
                &lhs_constness.vm_const.expect("VM constant lattice"),
                &rhs_constness.vm_const.expect("VM constant lattice"),
                opcode,
            ) {
                return ConstnessLattice::from_vm_const(Constness::VmConstant, vm_const);
            }
            return ConstnessLattice::from_kind(Constness::NotAConstant);
        } else if lhs_constness.kind == Constness::Undetermined
            && rhs_constness.kind == Constness::Undetermined
        {
            return ConstnessLattice::from_kind(Constness::Undetermined);
        }

        ConstnessLattice::from_kind(Constness::NotAConstant)
    }

    pub fn evaluate_comparison_condition(
        &mut self,
        op: LuauOpcode,
        lhs: BcOp,
        rhs: BcOp,
    ) -> ConditionState {
        let state = unsafe { &mut *self.state };
        let lhs_const = state.operand_lattice(&lhs);
        let rhs_const = state.operand_lattice(&rhs);
        let is_ordering_op = matches!(
            op,
            LuauOpcode::LOP_JUMPIFLT
                | LuauOpcode::LOP_JUMPIFLE
                | LuauOpcode::LOP_JUMPIFNOTLT
                | LuauOpcode::LOP_JUMPIFNOTLE
        );
        let is_orderable = |lattice: &ConstnessLattice| {
            if lattice.kind == Constness::VmConstant {
                self.impl_
                    .is_orderable(&lattice.vm_const.expect("VM constant lattice"))
            } else if lattice.kind == Constness::ImmConstant {
                lattice.imm_const.expect("immediate lattice").kind == BcImmKind::Int
            } else {
                false
            }
        };

        if is_ordering_op && (!is_orderable(&lhs_const) || !is_orderable(&rhs_const)) {
            return ConditionState::Unknown;
        }

        if lhs_const.kind == Constness::VmConstant
            && rhs_const.kind == Constness::VmConstant
            && !self.impl_.kind_equals(
                &lhs_const.vm_const.expect("VM constant lattice"),
                &rhs_const.vm_const.expect("VM constant lattice"),
            )
        {
            return ConditionState::Unknown;
        }

        let apply_op = |cmp: i32| match op {
            LuauOpcode::LOP_JUMPIFEQ | LuauOpcode::LOP_JUMPIFNOTEQ => cmp == 0,
            LuauOpcode::LOP_JUMPIFLT | LuauOpcode::LOP_JUMPIFNOTLT => cmp < 0,
            LuauOpcode::LOP_JUMPIFLE | LuauOpcode::LOP_JUMPIFNOTLE => cmp <= 0,
            _ => {
                LUAU_ASSERT!(false, "Unhandled comparison opcode");
                false
            }
        };
        let condition = |value| {
            if value {
                ConditionState::AlwaysTrue
            } else {
                ConditionState::AlwaysFalse
            }
        };

        if lhs_const.kind == Constness::VmConstant && rhs_const.kind == Constness::VmConstant {
            let cmp = self.impl_.cmp_bc_op(
                &lhs_const.vm_const.expect("VM constant lattice"),
                &rhs_const.vm_const.expect("VM constant lattice"),
            );
            return condition(apply_op(cmp));
        } else if lhs_const.kind == Constness::ImmConstant
            && rhs_const.kind == Constness::ImmConstant
        {
            let lhs_imm = lhs_const.imm_const.expect("immediate lattice");
            let rhs_imm = rhs_const.imm_const.expect("immediate lattice");
            let cmp = if lhs_imm.kind == BcImmKind::Int && rhs_imm.kind == BcImmKind::Int {
                let lv = unsafe { lhs_imm.value.valueInt };
                let rv = unsafe { rhs_imm.value.valueInt };
                i32::from(lv > rv) - i32::from(lv < rv)
            } else if lhs_imm.kind == BcImmKind::Boolean && rhs_imm.kind == BcImmKind::Boolean {
                i32::from(
                    unsafe { lhs_imm.value.valueBoolean } != unsafe { rhs_imm.value.valueBoolean },
                )
            } else {
                return ConditionState::Unknown;
            };
            return condition(apply_op(cmp));
        } else if lhs_const.kind == Constness::VmConstant
            && rhs_const.kind == Constness::ImmConstant
        {
            let cmp = self.impl_.cmp_bc_imm(
                &lhs_const.vm_const.expect("VM constant lattice"),
                &rhs_const.imm_const.expect("immediate lattice"),
            );
            return condition(apply_op(cmp));
        } else if lhs_const.kind == Constness::ImmConstant
            && rhs_const.kind == Constness::VmConstant
        {
            let cmp = -self.impl_.cmp_bc_imm(
                &rhs_const.vm_const.expect("VM constant lattice"),
                &lhs_const.imm_const.expect("immediate lattice"),
            );
            return condition(apply_op(cmp));
        }

        ConditionState::Unknown
    }

    pub fn evaluate_xeqk_condition(
        &mut self,
        func: &crate::records::bc_function::BcFunction,
        inst_op: BcOp,
    ) -> ConditionState {
        let inst = &func.instructions[inst_op.index as usize];
        let val_const = unsafe { &mut *self.state }.operand_lattice(&inst.ops[0]);
        let condition = |value| {
            if value {
                ConditionState::AlwaysTrue
            } else {
                ConditionState::AlwaysFalse
            }
        };

        match inst.op {
            LuauOpcode::LOP_JUMPXEQKNIL => {
                let nil = self.impl_.make_nil();
                if val_const.kind == Constness::VmConstant
                    && self
                        .impl_
                        .falsey(&val_const.vm_const.expect("VM constant lattice"))
                    && self
                        .impl_
                        .kind_equals(&val_const.vm_const.expect("VM constant lattice"), &nil)
                {
                    return ConditionState::AlwaysTrue;
                } else if val_const.kind == Constness::ImmConstant
                    || (val_const.kind == Constness::VmConstant
                        && !self
                            .impl_
                            .kind_equals(&val_const.vm_const.expect("VM constant lattice"), &nil))
                {
                    return ConditionState::AlwaysFalse;
                }
            }
            LuauOpcode::LOP_JUMPXEQKB => {
                let cmp_imm_op = inst.ops[3];
                LUAU_ASSERT!(cmp_imm_op.kind == BcOpKind::Imm);
                if val_const.kind == Constness::ImmConstant
                    && val_const.imm_const.expect("immediate lattice").kind == BcImmKind::Boolean
                {
                    // Preserve the upstream 0.730 implementation, including
                    // its immediate-lattice access through vmConst.
                    if let Some(eq) = self.impl_.eq_bc_op(
                        &val_const.vm_const.expect("upstream vmConst access"),
                        &cmp_imm_op,
                    ) {
                        return condition(eq);
                    }
                } else if val_const.kind == Constness::VmConstant {
                    if let Some(eq) = self.impl_.eq_bc_op(
                        &val_const.vm_const.expect("VM constant lattice"),
                        &cmp_imm_op,
                    ) {
                        return condition(eq);
                    }
                }
            }
            LuauOpcode::LOP_JUMPXEQKN => {
                let cmp_const_op = inst.ops[3];
                LUAU_ASSERT!(cmp_const_op.kind == BcOpKind::VmConst);
                if val_const.kind == Constness::VmConstant {
                    if let Some(eq) = self.impl_.eq_bc_op(
                        &val_const.vm_const.expect("VM constant lattice"),
                        &cmp_const_op,
                    ) {
                        return condition(eq);
                    }
                } else if val_const.kind == Constness::ImmConstant
                    && val_const.imm_const.expect("immediate lattice").kind == BcImmKind::Int
                {
                    if let Some(eq) = self.impl_.eq_int(&cmp_const_op, unsafe {
                        val_const
                            .imm_const
                            .expect("immediate lattice")
                            .value
                            .valueInt
                    }) {
                        return condition(eq);
                    }
                }
            }
            LuauOpcode::LOP_JUMPXEQKS => {
                let cmp_const_op = inst.ops[3];
                LUAU_ASSERT!(cmp_const_op.kind == BcOpKind::VmConst);
                if val_const.kind == Constness::VmConstant {
                    if let Some(eq) = self.impl_.eq_bc_op(
                        &val_const.vm_const.expect("VM constant lattice"),
                        &cmp_const_op,
                    ) {
                        return condition(eq);
                    }
                }
            }
            _ => {}
        }

        ConditionState::Unknown
    }

    pub fn evaluate_condition(&mut self, op: BcOp) -> ConditionState {
        let lhs = unsafe { &mut *self.state }.operand_lattice(&op);
        if lhs.kind == Constness::VmConstant {
            if self
                .impl_
                .falsey(&lhs.vm_const.expect("VM constant lattice"))
            {
                ConditionState::AlwaysFalse
            } else {
                ConditionState::AlwaysTrue
            }
        } else if lhs.kind == Constness::ImmConstant {
            let imm = lhs.imm_const.expect("immediate lattice");
            if imm.kind == BcImmKind::Boolean {
                if unsafe { imm.value.valueBoolean } {
                    ConditionState::AlwaysTrue
                } else {
                    ConditionState::AlwaysFalse
                }
            } else {
                ConditionState::Unknown
            }
        } else {
            ConditionState::Unknown
        }
    }

    pub fn evaluate(
        &mut self,
        op: LuauOpcode,
        func: &crate::records::bc_function::BcFunction,
        inst_op: BcOp,
    ) -> ConstnessLattice {
        let inst = &func.instructions[inst_op.index as usize];
        match op {
            LuauOpcode::LOP_LOADK | LuauOpcode::LOP_LOADKX => {
                let op = inst.ops[0];
                LUAU_ASSERT!(op.kind == BcOpKind::VmConst);
                ConstnessLattice::from_vm_const(Constness::VmConstant, op)
            }
            LuauOpcode::LOP_LOADB | LuauOpcode::LOP_LOADN => {
                let op = inst.ops[0];
                LUAU_ASSERT!(op.kind == BcOpKind::Imm);
                ConstnessLattice::from_imm_const(
                    Constness::ImmConstant,
                    *self.impl_.as_imm(op).operator_deref(),
                )
            }
            LuauOpcode::LOP_LOADNIL => {
                ConstnessLattice::from_vm_const(Constness::VmConstant, self.impl_.make_nil())
            }
            LuauOpcode::LOP_ADD
            | LuauOpcode::LOP_SUB
            | LuauOpcode::LOP_MUL
            | LuauOpcode::LOP_DIV
            | LuauOpcode::LOP_MOD
            | LuauOpcode::LOP_POW
            | LuauOpcode::LOP_IDIV => self.evaluate_arith(op, func, inst_op),
            LuauOpcode::LOP_MOVE => unsafe { &mut *self.state }.operand_lattice(&inst.ops[0]),
            LuauOpcode::LOP_JUMPIF | LuauOpcode::LOP_JUMPIFNOT => {
                let condition = self.evaluate_condition(inst.ops[0]);
                if condition == ConditionState::Unknown {
                    return ConstnessLattice::from_kind(
                        unsafe { &mut *self.state }.unknown_condition_constness([inst.ops[0]]),
                    );
                }
                let jumps_on_true = inst.op == LuauOpcode::LOP_JUMPIF;
                let takes_jump = (condition == ConditionState::AlwaysTrue) == jumps_on_true;
                ConstnessLattice::from_imm_const(
                    Constness::ImmConstant,
                    self.impl_.make_imm_bool(takes_jump),
                )
            }
            LuauOpcode::LOP_JUMPIFEQ
            | LuauOpcode::LOP_JUMPIFLE
            | LuauOpcode::LOP_JUMPIFLT
            | LuauOpcode::LOP_JUMPIFNOTEQ
            | LuauOpcode::LOP_JUMPIFNOTLE
            | LuauOpcode::LOP_JUMPIFNOTLT => {
                let condition =
                    self.evaluate_comparison_condition(inst.op, inst.ops[0], inst.ops[1]);
                if condition == ConditionState::Unknown {
                    return ConstnessLattice::from_kind(
                        unsafe { &mut *self.state }
                            .unknown_condition_constness([inst.ops[0], inst.ops[1]]),
                    );
                }
                let negated = matches!(
                    inst.op,
                    LuauOpcode::LOP_JUMPIFNOTEQ
                        | LuauOpcode::LOP_JUMPIFNOTLE
                        | LuauOpcode::LOP_JUMPIFNOTLT
                );
                let takes_jump = (condition == ConditionState::AlwaysTrue) != negated;
                ConstnessLattice::from_imm_const(
                    Constness::ImmConstant,
                    self.impl_.make_imm_bool(takes_jump),
                )
            }
            LuauOpcode::LOP_JUMPXEQKNIL
            | LuauOpcode::LOP_JUMPXEQKB
            | LuauOpcode::LOP_JUMPXEQKN
            | LuauOpcode::LOP_JUMPXEQKS => {
                let condition = self.evaluate_xeqk_condition(func, inst_op);
                if condition == ConditionState::Unknown {
                    return ConstnessLattice::from_kind(
                        unsafe { &mut *self.state }.unknown_condition_constness([inst.ops[0]]),
                    );
                }
                let negated = !self.impl_.falsey(&inst.ops[1]);
                let takes_jump = (condition == ConditionState::AlwaysTrue) != negated;
                ConstnessLattice::from_imm_const(
                    Constness::ImmConstant,
                    self.impl_.make_imm_bool(takes_jump),
                )
            }
            _ => ConstnessLattice::from_kind(Constness::NotAConstant),
        }
    }
}
