use crate::enums::bc_vm_const_kind::BcVmConstKind;
use crate::records::bc_vm_const::{BcVmConst, BcVmConstValue};
use crate::records::bytecode_builder::BytecodeBuilder;

#[test]
fn float_and_double_vector_constants_do_not_alias() {
    let mut builder = BytecodeBuilder::new(None);

    let vectorf = builder.add_constant_vectorf(1.0, 2.0, 3.0, 4.0);
    let vectorf_again = builder.add_constant_vectorf(1.0, 2.0, 3.0, 4.0);
    let vectord = builder.add_constant_vectord(1.0, 2.0, 3.0, 4.0);
    let vectord_again = builder.add_constant_vectord(1.0, 2.0, 3.0, 4.0);

    assert_eq!(vectorf, vectorf_again);
    assert_eq!(vectord, vectord_again);
    assert_ne!(vectorf, vectord);
    assert_eq!(
        unsafe { builder.constants[vectorf as usize].value.valueVectorf },
        [1.0, 2.0, 3.0, 4.0]
    );
    assert_eq!(
        unsafe { builder.constants[vectord as usize].value.valueVectord },
        [1.0, 2.0, 3.0, 4.0]
    );
}

#[test]
fn graph_vector_constants_compare_with_matching_precision_only() {
    let vectorf = BcVmConst {
        kind: BcVmConstKind::Vectorf,
        value: BcVmConstValue {
            valueVectorf: [1.0, 2.0, 3.0, 4.0],
        },
    };
    let vectord = BcVmConst {
        kind: BcVmConstKind::Vectord,
        value: BcVmConstValue {
            valueVectord: [1.0, 2.0, 3.0, 4.0],
        },
    };

    assert_eq!(vectorf, vectorf);
    assert_eq!(vectord, vectord);
    assert_ne!(vectorf, vectord);
}
