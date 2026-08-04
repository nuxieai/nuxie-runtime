use crate::enums::bc_block_edge_kind::BcBlockEdgeKind;
use crate::enums::bc_imm_kind::BcImmKind;
use crate::enums::bc_vm_const_kind::BcVmConstKind;
use crate::enums::constness::Constness;
use crate::functions::fold_constants::fold_constants;
use crate::records::bc_block_edge::BcBlockEdge;
use crate::records::bc_function::BcFunction;
use crate::records::bc_imm::{BcImm, BcImmValue};
use crate::records::bc_vm_const::{BcVmConst, BcVmConstValue};
use crate::records::bc_vm_const_impl::BcVmConstImpl;
use crate::records::constness_lattice::ConstnessLattice;
use luaur_common::enums::luau_opcode::LuauOpcode;

fn number(value: f64) -> BcVmConst {
    BcVmConst {
        kind: BcVmConstKind::Number,
        value: BcVmConstValue { valueNumber: value },
    }
}

#[test]
fn evaluates_numeric_arithmetic_and_deduplicates_results() {
    let mut func = BcFunction::default();
    let lhs = func.add_const(&number(7.0));
    let rhs = func.add_const(&number(3.0));
    let existing = func.add_const(&number(10.0));
    let ops = BcVmConstImpl::new(&mut func);

    assert_eq!(
        ops.evaluate(&lhs, &rhs, LuauOpcode::LOP_ADD),
        Some(existing)
    );
    let quotient = ops
        .evaluate(&lhs, &rhs, LuauOpcode::LOP_IDIV)
        .expect("constant quotient");
    assert_eq!(unsafe { func.const_op(quotient).value.valueNumber }, 2.0);
}

#[test]
fn evaluates_division_by_zero_with_ieee_results() {
    let mut func = BcFunction::default();
    let one = func.add_const(&number(1.0));
    let zero = func.add_const(&number(0.0));
    let ops = BcVmConstImpl::new(&mut func);

    let div = ops
        .evaluate(&one, &zero, LuauOpcode::LOP_DIV)
        .expect("division by zero is folded");
    let idiv = ops
        .evaluate(&one, &zero, LuauOpcode::LOP_IDIV)
        .expect("floor division by zero is folded");

    assert_eq!(
        unsafe { func.const_op(div).value.valueNumber },
        f64::INFINITY
    );
    assert_eq!(
        unsafe { func.const_op(idiv).value.valueNumber },
        f64::INFINITY
    );
    assert_eq!(ops.evaluate(&one, &zero, LuauOpcode::LOP_MOD), None);
}

#[test]
fn set_ops_rebuilds_def_use_links() {
    let mut func = BcFunction::default();
    let block = func.add_block();
    let old = func.add_inst();
    let new = func.add_inst();
    let user = func.add_inst();

    for op in [old, new, user] {
        func.instructions[op.index as usize].block = block;
        func.blocks[block.index as usize].ops.push_back(op);
    }
    func.instructions[user.index as usize].ops.push(old);
    func.instructions[old.index as usize].uses.push(user);

    func.set_ops(user, &[new]);

    assert!(func.instructions[old.index as usize].uses.is_empty());
    assert_eq!(func.instructions[new.index as usize].uses, vec![user]);
    assert_eq!(
        func.instructions[user.index as usize].ops.as_slice(),
        &[new]
    );
}

#[test]
fn preserves_luau_truthiness_and_optional_comparisons() {
    let mut func = BcFunction::default();
    let nil = func.add_const(&BcVmConst::new());
    let number = func.add_const(&number(1.0));
    let false_imm = BcImm {
        kind: BcImmKind::Boolean,
        value: BcImmValue {
            valueBoolean: false,
        },
    };
    let false_op = func.add_imm_value(&false_imm);
    let ops = BcVmConstImpl::new(&mut func);

    assert!(ops.falsey(&nil));
    assert!(ops.falsey(&false_op));
    assert_eq!(ops.eq_int(&number, 1), Some(true));
    assert_eq!(ops.eq_bool(&number, true), None);
}

#[test]
fn lattice_meet_keeps_equal_constants_and_drops_disagreement() {
    let op = crate::records::bc_op::BcOp::new();
    let top = ConstnessLattice::default();
    let constant = ConstnessLattice::from_vm_const(Constness::VmConstant, op);

    assert_eq!(top.merge(&constant), constant);
    assert_eq!(constant.merge(&constant), constant);
    assert_eq!(
        constant
            .merge(&ConstnessLattice::from_kind(Constness::NotAConstant))
            .kind,
        Constness::NotAConstant
    );
}

#[test]
fn sccp_rewrites_folded_immediate_arithmetic_to_loadn() {
    let mut func = BcFunction::default();
    let entry = func.add_block();
    let exit = func.add_block();
    func.entry_block = entry;
    func.exit_block = exit;
    func.blocks[entry.index as usize]
        .successors
        .push(BcBlockEdge {
            kind: BcBlockEdgeKind::Fallthrough,
            target: exit,
        });

    let lhs_imm = func.add_imm_value(&BcImm {
        kind: BcImmKind::Int,
        value: BcImmValue { valueInt: -7 },
    });
    let rhs_imm = func.add_imm_value(&BcImm {
        kind: BcImmKind::Int,
        value: BcImmValue { valueInt: 3 },
    });
    let lhs = func.add_inst();
    func.instructions[lhs.index as usize].op = LuauOpcode::LOP_LOADN;
    func.instructions[lhs.index as usize].block = entry;
    func.instructions[lhs.index as usize].ops.push(lhs_imm);
    let rhs = func.add_inst();
    func.instructions[rhs.index as usize].op = LuauOpcode::LOP_LOADN;
    func.instructions[rhs.index as usize].block = entry;
    func.instructions[rhs.index as usize].ops.push(rhs_imm);
    let idiv = func.add_inst();
    func.instructions[idiv.index as usize].op = LuauOpcode::LOP_IDIV;
    func.instructions[idiv.index as usize].block = entry;
    func.instructions[idiv.index as usize].ops.push(lhs);
    func.instructions[idiv.index as usize].ops.push(rhs);
    func.instructions[lhs.index as usize].uses.push(idiv);
    func.instructions[rhs.index as usize].uses.push(idiv);
    func.blocks[entry.index as usize]
        .ops
        .extend([lhs, rhs, idiv]);

    let ops = BcVmConstImpl::new(&mut func);
    fold_constants(&mut func, &ops);

    let folded = &func.instructions[idiv.index as usize];
    assert_eq!(folded.op, LuauOpcode::LOP_LOADN);
    assert_eq!(folded.ops.len(), 1);
    assert_eq!(
        unsafe { func.imm_op(folded.ops[0]).value.valueInt },
        -3,
        "Luau integer division rounds toward negative infinity"
    );
}

fn fold_vm_constant_arithmetic(opcode: LuauOpcode, value: f64) -> LuauOpcode {
    let mut func = BcFunction::default();
    let entry = func.add_block();
    let exit = func.add_block();
    func.entry_block = entry;
    func.exit_block = exit;
    func.blocks[entry.index as usize]
        .successors
        .push(BcBlockEdge {
            kind: BcBlockEdgeKind::Fallthrough,
            target: exit,
        });

    let constant = func.add_const(&number(value));
    let load = func.add_inst();
    func.instructions[load.index as usize].op = LuauOpcode::LOP_LOADK;
    func.instructions[load.index as usize].block = entry;
    func.instructions[load.index as usize].ops.push(constant);
    let arithmetic = func.add_inst();
    func.instructions[arithmetic.index as usize].op = opcode;
    func.instructions[arithmetic.index as usize].block = entry;
    func.instructions[arithmetic.index as usize].ops.push(
        crate::records::bc_op::BcOp::bc_op_bc_op_kind_u32(
            crate::enums::bc_op_kind::BcOpKind::VmReg,
            0,
        ),
    );
    func.instructions[arithmetic.index as usize].ops.push(load);
    func.instructions[load.index as usize].uses.push(arithmetic);
    func.blocks[entry.index as usize]
        .ops
        .extend([load, arithmetic]);

    let ops = BcVmConstImpl::new(&mut func);
    fold_constants(&mut func, &ops);
    func.instructions[arithmetic.index as usize].op
}

#[test]
fn sccp_preserves_upstream_unmatched_zero_and_one_arithmetic_cases() {
    assert_eq!(
        fold_vm_constant_arithmetic(LuauOpcode::LOP_DIV, 0.0),
        LuauOpcode::LOP_DIV
    );
    assert_eq!(
        fold_vm_constant_arithmetic(LuauOpcode::LOP_ADD, 1.0),
        LuauOpcode::LOP_ADD
    );
}
