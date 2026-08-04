use crate::enums::bc_imm_kind::BcImmKind;
use crate::enums::bc_vm_const_kind::BcVmConstKind;
use crate::enums::constness::Constness;
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
