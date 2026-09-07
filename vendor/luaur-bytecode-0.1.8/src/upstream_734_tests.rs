//! Focused regression coverage for Luau 0.734 BytecodeOps and SCCP deltas.
#[test]
fn clear_strings_resets_string_ids_and_debug_names_together() {
    use crate::records::{bytecode_builder::BytecodeBuilder, string_ref::StringRef};
    let mut builder = BytecodeBuilder::new(None);
    builder.set_dump_flags(crate::enums::dump_flags::DumpFlags::Dump_Code as u32);
    let first = b"first";
    let second = b"second";
    builder.add_string_table_entry(StringRef::new(first.as_ptr().cast(), first.len()));
    assert_eq!(builder.debug_strings.len(), 1);
    builder.clear_strings();
    assert!(builder.debug_strings.is_empty());
    assert_eq!(builder.add_string_table_entry(StringRef::new(second.as_ptr().cast(), second.len())), 1);
    assert_eq!(builder.debug_strings.len(), 1);
}

#[test]
fn detailed_closure_constants_identify_named_and_anonymous_functions() {
    use crate::records::bytecode_builder::BytecodeBuilder;
    let mut builder = BytecodeBuilder::new(None);
    let id = builder.begin_function(0, false);
    let constant = builder.add_constant_closure(id);
    let mut dump = String::new();
    builder.dump_constant(&mut dump, constant, true);
    assert_eq!(dump, "function");
    builder.functions[id as usize].dumpname = "example".into();
    dump.clear();
    builder.dump_constant(&mut dump, constant, true);
    assert_eq!(dump, "function example");
    dump.clear();
    builder.dump_constant(&mut dump, constant, false);
    assert_eq!(dump, "'example'");
}

use crate::enums::{bc_imm_kind::BcImmKind, bc_op_kind::BcOpKind, bc_vm_const_kind::BcVmConstKind};
use crate::records::{
    bc_block::BcBlock,
    bc_function::BcFunction,
    bc_imm::{BcImm, BcImmValue},
    bc_inst_helper::BcInstHelper,
    bc_op::BcOp,
    bc_set_list::BcSetList,
    bc_vm_const::{BcVmConst, BcVmConstValue},
    bc_vm_const_impl::BcVmConstImpl,
};

#[test]
fn setlist_target_is_not_a_parameter_and_detach_removes_block_membership() {
    let mut graph = BcFunction::default();
    graph.blocks.push(BcBlock::default());
    let block = BcOp::bc_op_bc_op_kind_u32(BcOpKind::Block, 0);
    let inst = graph.add_inst();
    let table = BcOp::bc_op_bc_op_kind_u32(BcOpKind::VmReg, 2);
    let value = BcOp::bc_op_bc_op_kind_u32(BcOpKind::VmReg, 3);
    for operand in [BcOp::new(), BcOp::new(), table, value] {
        graph.instructions[0].ops.push_back(operand);
    }
    graph.instructions[0].block = block;
    graph.blocks[0].ops.push_back(inst);
    let ptr = &mut graph as *mut BcFunction;
    let reference = unsafe { (*ptr).inst(inst) };
    let mut setlist = BcSetList::<BcVmConst>::from(ptr, reference);
    assert_eq!(setlist.target(), table);
    assert_eq!(setlist.params(), vec![value]);
    drop(setlist);
    let reference = unsafe { (*ptr).inst(inst) };
    let mut helper = BcInstHelper::new(unsafe { &mut *ptr }, reference);
    helper.detach();
    helper.detach();
    drop(helper);
    assert!(graph.blocks[0].ops.is_empty());
    assert_eq!(graph.instructions[0].block, BcOp::new());
}

#[test]
fn sccp_incompatible_immediate_comparison_matches_upstream_zero() {
    let mut graph = BcFunction::default();
    graph.constants.push(BcVmConst {
        kind: BcVmConstKind::Boolean,
        value: BcVmConstValue { valueBoolean: true },
    });
    let implementation = BcVmConstImpl { func: &mut graph };
    let value = BcOp::bc_op_bc_op_kind_u32(BcOpKind::VmConst, 0);
    let immediate = BcImm {
        kind: BcImmKind::Int,
        value: BcImmValue { valueInt: 1 },
    };
    assert_eq!(implementation.cmp_bc_imm(&value, &immediate), 0);
}
