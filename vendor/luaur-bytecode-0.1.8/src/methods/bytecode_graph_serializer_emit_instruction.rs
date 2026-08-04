use crate::enums::bc_op_kind::BcOpKind;
use crate::records::bc_inst::BcInst;
use crate::records::bc_op::BcOp;
use crate::records::bytecode_builder::BytecodeBuilder;
use crate::records::bytecode_graph_serializer::BytecodeGraphSerializer;
use luaur_common::enums::luau_capture_type::LuauCaptureType;
use luaur_common::enums::luau_opcode::LuauOpcode;
use luaur_common::macros::luau_assert::LUAU_ASSERT;
use luaur_common::macros::luau_unreachable::LUAU_UNREACHABLE;

impl<'a> BytecodeGraphSerializer<'a> {
    pub fn emit_instruction(&mut self, insn_op: BcOp) {
        let mut insn: BcInst = self.func.inst_op(insn_op).clone();
        let insn: &mut BcInst = &mut insn;
        self.bcb.set_debug_line(insn.line as i32);
        match insn.op {
            LuauOpcode::LOP_NOP | LuauOpcode::LOP_BREAK | LuauOpcode::LOP_NATIVECALL => {
                self.bcb.emit_abc(insn.op, 0, 0, 0);
            }
            LuauOpcode::LOP_LOADNIL => {
                let out = self.get_register(insn_op);
                self.bcb.emit_abc(LuauOpcode::LOP_LOADNIL, out, 0, 0);
            }
            LuauOpcode::LOP_LOADB => {
                if insn.ops.len() > 1 {
                    self.record_jump(insn, 1);
                }
                let imm_bool = self.get_imm_bool(insn, 0);
                let out = self.get_register(insn_op);
                self.bcb
                    .emit_abc(LuauOpcode::LOP_LOADB, out, imm_bool as u8, 0);
            }
            LuauOpcode::LOP_LOADN => {
                let imm_int = self.get_imm_int(insn, 0);
                let out = self.get_register(insn_op);
                self.bcb.emit_ad(LuauOpcode::LOP_LOADN, out, imm_int as i16);
            }
            LuauOpcode::LOP_LOADK => {
                let vm_const_input_d = self.get_vm_const_input_d(insn, 0);
                let out = self.get_register(insn_op);
                self.bcb
                    .emit_ad(LuauOpcode::LOP_LOADK, out, vm_const_input_d as i16);
            }
            LuauOpcode::LOP_MOVE => {
                let reg_input = self.get_reg_input(insn, 0);
                let out = self.get_register(insn_op);
                self.bcb.emit_abc(LuauOpcode::LOP_MOVE, out, reg_input, 0);
            }
            LuauOpcode::LOP_GETGLOBAL => {
                let imm_int = self.get_imm_int(insn, 0);
                let out = self.get_register(insn_op);
                self.bcb
                    .emit_abc(LuauOpcode::LOP_GETGLOBAL, out, 0, imm_int as u8);
                let vm_const_input_aux = self.get_vm_const_input_aux(insn, 1);
                self.bcb.emit_aux(vm_const_input_aux);
            }
            LuauOpcode::LOP_SETGLOBAL => {
                let reg_input = self.get_reg_input(insn, 0);
                let imm_int = self.get_imm_int(insn, 1);
                self.bcb
                    .emit_abc(LuauOpcode::LOP_SETGLOBAL, reg_input, 0, imm_int as u8);
                let vm_const_input_aux = self.get_vm_const_input_aux(insn, 2);
                self.bcb.emit_aux(vm_const_input_aux);
            }
            LuauOpcode::LOP_GETUPVAL => {
                let upval_input = self.get_upval_input(insn, 0);
                let out = self.get_register(insn_op);
                self.bcb
                    .emit_abc(LuauOpcode::LOP_GETUPVAL, out, upval_input, 0);
            }
            LuauOpcode::LOP_SETUPVAL => {
                let reg_input = self.get_reg_input(insn, 0);
                let upval_input = self.get_upval_input(insn, 1);
                self.bcb
                    .emit_abc(LuauOpcode::LOP_SETUPVAL, reg_input, upval_input, 0);
            }
            LuauOpcode::LOP_CLOSEUPVALS => {
                LUAU_ASSERT!(insn.ops.len() == 1 && insn.ops[0].kind == BcOpKind::VmReg);
                self.bcb
                    .emit_abc(LuauOpcode::LOP_CLOSEUPVALS, insn.ops[0].index as u8, 0, 0);
            }
            LuauOpcode::LOP_GETIMPORT => {
                let vm_const_input_d = self.get_vm_const_input_d(insn, 0);
                let out = self.get_register(insn_op);
                self.bcb
                    .emit_ad(LuauOpcode::LOP_GETIMPORT, out, vm_const_input_d as i16);
                let imm_import = self.get_imm_import(insn, 1);
                self.bcb.emit_aux(imm_import);
            }
            LuauOpcode::LOP_GETTABLE => {
                let reg_input_0 = self.get_reg_input(insn, 0);
                let reg_input_1 = self.get_reg_input(insn, 1);
                let out = self.get_register(insn_op);
                self.bcb
                    .emit_abc(LuauOpcode::LOP_GETTABLE, out, reg_input_0, reg_input_1);
            }
            LuauOpcode::LOP_SETTABLE => {
                let reg_input_0 = self.get_reg_input(insn, 0);
                let reg_input_1 = self.get_reg_input(insn, 1);
                let reg_input_2 = self.get_reg_input(insn, 2);
                self.bcb.emit_abc(
                    LuauOpcode::LOP_SETTABLE,
                    reg_input_0,
                    reg_input_1,
                    reg_input_2,
                );
            }
            LuauOpcode::LOP_GETUDATAKS | LuauOpcode::LOP_GETTABLEKS => {
                let reg_input_0 = self.get_reg_input(insn, 0);
                let imm_int = self.get_imm_int(insn, 1);
                let out = self.get_register(insn_op);
                self.bcb.emit_abc(insn.op, out, reg_input_0, imm_int as u8);
                let vm_const_input_aux = self.get_vm_const_input_aux(insn, 2);
                self.bcb.emit_aux(vm_const_input_aux);
            }
            LuauOpcode::LOP_SETUDATAKS | LuauOpcode::LOP_SETTABLEKS => {
                let reg_input_0 = self.get_reg_input(insn, 0);
                let reg_input_1 = self.get_reg_input(insn, 1);
                let imm_int = self.get_imm_int(insn, 2);
                self.bcb
                    .emit_abc(insn.op, reg_input_0, reg_input_1, imm_int as u8);
                let vm_const_input_aux = self.get_vm_const_input_aux(insn, 3);
                self.bcb.emit_aux(vm_const_input_aux);
            }
            LuauOpcode::LOP_GETTABLEN => {
                let reg_input_0 = self.get_reg_input(insn, 0);
                let imm_int = self.get_imm_int(insn, 1);
                let out = self.get_register(insn_op);
                self.bcb.emit_abc(
                    LuauOpcode::LOP_GETTABLEN,
                    out,
                    reg_input_0,
                    (imm_int - 1) as u8,
                );
            }
            LuauOpcode::LOP_SETTABLEN => {
                let reg_input_0 = self.get_reg_input(insn, 0);
                let reg_input_1 = self.get_reg_input(insn, 1);
                let imm_int = self.get_imm_int(insn, 2);
                self.bcb.emit_abc(
                    LuauOpcode::LOP_SETTABLEN,
                    reg_input_0,
                    reg_input_1,
                    (imm_int - 1) as u8,
                );
            }
            LuauOpcode::LOP_NEWCLOSURE => {
                let proto_input = self.get_proto_input(insn, 0);
                let out = self.get_register(insn_op);
                self.bcb
                    .emit_ad(LuauOpcode::LOP_NEWCLOSURE, out, proto_input as i16);
            }
            LuauOpcode::LOP_NAMECALLUDATA | LuauOpcode::LOP_NAMECALL => {
                let reg_input_0 = self.get_reg_input(insn, 0);
                let imm_int = self.get_imm_int(insn, 1);
                let out = self.get_register(insn_op);
                self.bcb.emit_abc(insn.op, out, reg_input_0, imm_int as u8);
                let vm_const_input_aux = self.get_vm_const_input_aux(insn, 2);
                self.bcb.emit_aux(vm_const_input_aux);
            }
            LuauOpcode::LOP_CALL => {
                let reg_input_2 = self.get_reg_input(insn, 2);
                let imm_int_0 = self.get_imm_int(insn, 0);
                let imm_int_1 = self.get_imm_int(insn, 1);
                self.bcb.emit_abc(
                    LuauOpcode::LOP_CALL,
                    reg_input_2,
                    (imm_int_0 + 1) as u8,
                    (imm_int_1 + 1) as u8,
                );
            }
            LuauOpcode::LOP_CALLFB => {
                let reg_input_3 = self.get_reg_input(insn, 3);
                let imm_int_0 = self.get_imm_int(insn, 0);
                let imm_int_1 = self.get_imm_int(insn, 1);
                let imm_int_2 = self.get_imm_int(insn, 2);
                self.bcb.emit_abc(
                    LuauOpcode::LOP_CALLFB,
                    reg_input_3,
                    (imm_int_0 + 1) as u8,
                    (imm_int_1 + 1) as u8,
                );
                self.bcb.emit_aux(imm_int_2 as u32);
            }
            LuauOpcode::LOP_RETURN => {
                LUAU_ASSERT!(insn.ops.len() > 1);
                let reg_input_1 = self.get_reg_input(insn, 1);
                let imm_int_0 = self.get_imm_int(insn, 0);
                self.bcb.emit_abc(
                    LuauOpcode::LOP_RETURN,
                    reg_input_1,
                    (imm_int_0 + 1) as u8,
                    0,
                );
            }
            LuauOpcode::LOP_JUMP => {
                self.record_jump(insn, 0);
                self.bcb.emit_ad(LuauOpcode::LOP_JUMP, 0, 0);
            }
            LuauOpcode::LOP_JUMPBACK => {
                self.record_jump(insn, 0);
                self.bcb.emit_ad(LuauOpcode::LOP_JUMPBACK, 0, 0);
            }
            LuauOpcode::LOP_JUMPIFNOT | LuauOpcode::LOP_JUMPIF => {
                self.record_jump(insn, 1);
                let reg_input_0 = self.get_reg_input(insn, 0);
                self.bcb.emit_ad(insn.op, reg_input_0, 0);
            }
            LuauOpcode::LOP_JUMPIFEQ
            | LuauOpcode::LOP_JUMPIFLE
            | LuauOpcode::LOP_JUMPIFLT
            | LuauOpcode::LOP_JUMPIFNOTEQ
            | LuauOpcode::LOP_JUMPIFNOTLE
            | LuauOpcode::LOP_JUMPIFNOTLT => {
                self.record_jump(insn, 2);
                let reg_input_0 = self.get_reg_input(insn, 0);
                self.bcb.emit_ad(insn.op, reg_input_0, 0);
                let reg_input_1 = self.get_reg_input(insn, 1);
                self.bcb.emit_aux(reg_input_1 as u32);
            }
            LuauOpcode::LOP_ADD
            | LuauOpcode::LOP_SUB
            | LuauOpcode::LOP_MUL
            | LuauOpcode::LOP_DIV
            | LuauOpcode::LOP_MOD
            | LuauOpcode::LOP_POW
            | LuauOpcode::LOP_AND
            | LuauOpcode::LOP_OR => {
                let reg_input_0 = self.get_reg_input(insn, 0);
                let reg_input_1 = self.get_reg_input(insn, 1);
                let out = self.get_register(insn_op);
                self.bcb.emit_abc(insn.op, out, reg_input_0, reg_input_1);
            }
            LuauOpcode::LOP_ADDK
            | LuauOpcode::LOP_SUBK
            | LuauOpcode::LOP_MULK
            | LuauOpcode::LOP_DIVK
            | LuauOpcode::LOP_MODK
            | LuauOpcode::LOP_POWK
            | LuauOpcode::LOP_ANDK
            | LuauOpcode::LOP_ORK => {
                let reg_input_0 = self.get_reg_input(insn, 0);
                let vm_const_input_abc = self.get_vm_const_input_abc(insn, 1);
                let out = self.get_register(insn_op);
                self.bcb
                    .emit_abc(insn.op, out, reg_input_0, vm_const_input_abc);
            }
            LuauOpcode::LOP_CONCAT => {
                LUAU_ASSERT!(insn.ops.len() > 0);
                let reg_input_0 = self.get_reg_input(insn, 0);
                let last_idx = insn.ops.len() - 1;
                let reg_input_last = self.get_reg_input(insn, last_idx as u8);
                let out = self.get_register(insn_op);
                self.bcb
                    .emit_abc(LuauOpcode::LOP_CONCAT, out, reg_input_0, reg_input_last);
            }
            LuauOpcode::LOP_NOT | LuauOpcode::LOP_MINUS | LuauOpcode::LOP_LENGTH => {
                let reg_input_0 = self.get_reg_input(insn, 0);
                let out = self.get_register(insn_op);
                self.bcb.emit_abc(insn.op, out, reg_input_0, 0);
            }
            LuauOpcode::LOP_NEWTABLE => {
                let imm_int_0 = self.get_imm_int(insn, 0);
                let imm_int_1 = self.get_imm_int(insn, 1);
                let out = self.get_register(insn_op);
                self.bcb
                    .emit_abc(LuauOpcode::LOP_NEWTABLE, out, imm_int_0 as u8, 0);
                self.bcb.emit_aux(imm_int_1 as u32);
            }
            LuauOpcode::LOP_DUPTABLE => {
                let vm_const_input_d = self.get_vm_const_input_d(insn, 0);
                let out = self.get_register(insn_op);
                self.bcb
                    .emit_ad(LuauOpcode::LOP_DUPTABLE, out, vm_const_input_d as i16);
            }
            LuauOpcode::LOP_SETLIST => {
                LUAU_ASSERT!(insn.ops.len() > 2);
                let reg_input_2 = self.get_reg_input(insn, 2);
                let reg_input_3 = self.get_reg_input(insn, 3);
                let imm_int_1 = self.get_imm_int(insn, 1);
                let imm_int_0 = self.get_imm_int(insn, 0);
                self.bcb.emit_abc(
                    LuauOpcode::LOP_SETLIST,
                    reg_input_2,
                    reg_input_3,
                    (imm_int_1 + 1) as u8,
                );
                self.bcb.emit_aux(imm_int_0 as u32);
            }
            LuauOpcode::LOP_FORNPREP => {
                self.record_jump(insn, 3);
                let reg_input_0 = self.get_reg_input(insn, 0);
                self.bcb.emit_ad(LuauOpcode::LOP_FORNPREP, reg_input_0, 0);
            }
            LuauOpcode::LOP_FORNLOOP => {
                self.record_jump(insn, 3);
                let reg_input_0 = self.get_reg_input(insn, 0);
                self.bcb.emit_ad(LuauOpcode::LOP_FORNLOOP, reg_input_0, 0);
            }
            LuauOpcode::LOP_FORGPREP
            | LuauOpcode::LOP_FORGPREP_NEXT
            | LuauOpcode::LOP_FORGPREP_INEXT => {
                self.record_jump(insn, 3);
                let reg_input_0 = self.get_reg_input(insn, 0);
                self.bcb.emit_ad(insn.op, reg_input_0, 0);
            }
            LuauOpcode::LOP_FORGLOOP => {
                self.record_jump(insn, 5);
                let reg_input_0 = self.get_reg_input(insn, 0);
                self.bcb.emit_ad(LuauOpcode::LOP_FORGLOOP, reg_input_0, 0);
                let imm_bool_3 = self.get_imm_bool(insn, 3);
                let imm_int_4 = self.get_imm_int(insn, 4);
                let aux = (imm_bool_3 as u32) << 31 | (imm_int_4 as u32);
                self.bcb.emit_aux(aux);
            }
            LuauOpcode::LOP_FASTCALL => {
                let imm_int_0 = self.get_imm_int(insn, 0);
                let imm_int_1 = self.get_imm_int(insn, 1);
                self.bcb.emit_abc(
                    LuauOpcode::LOP_FASTCALL,
                    imm_int_0 as u8,
                    0,
                    imm_int_1 as u8,
                );
            }
            LuauOpcode::LOP_FASTCALL1 => {
                let imm_int_0 = self.get_imm_int(insn, 0);
                let reg_input_1 = self.get_reg_input(insn, 1);
                let imm_int_2 = self.get_imm_int(insn, 2);
                self.bcb.emit_abc(
                    LuauOpcode::LOP_FASTCALL1,
                    imm_int_0 as u8,
                    reg_input_1,
                    imm_int_2 as u8,
                );
            }
            LuauOpcode::LOP_FASTCALL2 => {
                let imm_int_0 = self.get_imm_int(insn, 0);
                let reg_input_1 = self.get_reg_input(insn, 1);
                let imm_int_3 = self.get_imm_int(insn, 3);
                let reg_input_2 = self.get_reg_input(insn, 2);
                self.bcb.emit_abc(
                    LuauOpcode::LOP_FASTCALL2,
                    imm_int_0 as u8,
                    reg_input_1,
                    imm_int_3 as u8,
                );
                self.bcb.emit_aux(reg_input_2 as u32);
            }
            LuauOpcode::LOP_FASTCALL2K => {
                let imm_int_0 = self.get_imm_int(insn, 0);
                let reg_input_1 = self.get_reg_input(insn, 1);
                let imm_int_3 = self.get_imm_int(insn, 3);
                let vm_const_input_aux = self.get_vm_const_input_aux(insn, 2);
                self.bcb.emit_abc(
                    LuauOpcode::LOP_FASTCALL2K,
                    imm_int_0 as u8,
                    reg_input_1,
                    imm_int_3 as u8,
                );
                self.bcb.emit_aux(vm_const_input_aux);
            }
            LuauOpcode::LOP_FASTCALL3 => {
                let imm_int_0 = self.get_imm_int(insn, 0);
                let reg_input_1 = self.get_reg_input(insn, 1);
                let imm_int_4 = self.get_imm_int(insn, 4);
                let reg_input_2 = self.get_reg_input(insn, 2);
                let reg_input_3 = self.get_reg_input(insn, 3);
                let aux = reg_input_2 as u32 | (reg_input_3 as u32) << 8;
                self.bcb.emit_abc(
                    LuauOpcode::LOP_FASTCALL3,
                    imm_int_0 as u8,
                    reg_input_1,
                    imm_int_4 as u8,
                );
                self.bcb.emit_aux(aux);
            }
            LuauOpcode::LOP_GETVARARGS => {
                LUAU_ASSERT!(insn.ops.len() == 2 && insn.ops[0].kind == BcOpKind::VmReg);
                let reg_index = insn.ops[0].index;
                let imm_int_1 = self.get_imm_int(insn, 1);
                self.bcb.emit_abc(
                    LuauOpcode::LOP_GETVARARGS,
                    reg_index as u8,
                    (imm_int_1 + 1) as u8,
                    0,
                );
            }
            LuauOpcode::LOP_DUPCLOSURE => {
                let vm_const_input_d = self.get_vm_const_input_d(insn, 0);
                let out = self.get_register(insn_op);
                self.bcb
                    .emit_ad(LuauOpcode::LOP_DUPCLOSURE, out, vm_const_input_d as i16);
            }
            LuauOpcode::LOP_PREPVARARGS => {
                let imm_int_0 = self.get_imm_int(insn, 0);
                self.bcb
                    .emit_ad(LuauOpcode::LOP_PREPVARARGS, imm_int_0 as u8, 0);
            }
            LuauOpcode::LOP_LOADKX => {
                let out = self.get_register(insn_op);
                self.bcb.emit_ad(LuauOpcode::LOP_LOADKX, out, 0);
                let vm_const_input_aux = self.get_vm_const_input_aux(insn, 0);
                self.bcb.emit_aux(vm_const_input_aux);
            }
            LuauOpcode::LOP_JUMPX => {
                self.record_jump(insn, 0);
                self.bcb.emit_e(LuauOpcode::LOP_JUMPX, 0);
            }
            LuauOpcode::LOP_COVERAGE => {
                let imm_int_0 = self.get_imm_int(insn, 0);
                self.bcb.emit_e(LuauOpcode::LOP_COVERAGE, imm_int_0);
            }
            LuauOpcode::LOP_CAPTURE => {
                let capture_type = self.get_imm_int(insn, 0) as u8;
                if capture_type == LuauCaptureType::LCT_VAL as u8
                    || capture_type == LuauCaptureType::LCT_REF as u8
                {
                    let reg_input_1 = self.get_reg_input(insn, 1);
                    let imm_int_2 = self.get_imm_int(insn, 2);
                    self.bcb.emit_abc(
                        LuauOpcode::LOP_CAPTURE,
                        capture_type,
                        reg_input_1,
                        imm_int_2 as u8,
                    );
                } else {
                    let upval_input = self.get_upval_input(insn, 1);
                    let imm_int_2 = self.get_imm_int(insn, 2);
                    self.bcb.emit_abc(
                        LuauOpcode::LOP_CAPTURE,
                        capture_type,
                        upval_input,
                        imm_int_2 as u8,
                    );
                }
            }
            LuauOpcode::LOP_SUBRK | LuauOpcode::LOP_DIVRK => {
                let vm_const_input_abc = self.get_vm_const_input_abc(insn, 0);
                let reg_input_1 = self.get_reg_input(insn, 1);
                let out = self.get_register(insn_op);
                self.bcb
                    .emit_abc(insn.op, out, vm_const_input_abc, reg_input_1);
            }
            LuauOpcode::LOP_JUMPXEQKNIL => {
                self.record_jump(insn, 2);
                let reg_input_0 = self.get_reg_input(insn, 0);
                let imm_bool_1 = self.get_imm_bool(insn, 1);
                let aux = (imm_bool_1 as u32) << 31;
                self.bcb
                    .emit_ad(LuauOpcode::LOP_JUMPXEQKNIL, reg_input_0, 0);
                self.bcb.emit_aux(aux);
            }
            LuauOpcode::LOP_JUMPXEQKB => {
                self.record_jump(insn, 2);
                let reg_input_0 = self.get_reg_input(insn, 0);
                let imm_bool_1 = self.get_imm_bool(insn, 1);
                let imm_bool_3 = self.get_imm_bool(insn, 3);
                let aux = (imm_bool_1 as u32) << 31 | (imm_bool_3 as u32);
                self.bcb.emit_ad(LuauOpcode::LOP_JUMPXEQKB, reg_input_0, 0);
                self.bcb.emit_aux(aux);
            }
            LuauOpcode::LOP_JUMPXEQKN | LuauOpcode::LOP_JUMPXEQKS => {
                self.record_jump(insn, 2);
                let reg_input_0 = self.get_reg_input(insn, 0);
                let imm_bool_1 = self.get_imm_bool(insn, 1);
                let vm_const_input_aux = self.get_vm_const_input_aux(insn, 3);
                let aux = (imm_bool_1 as u32) << 31 | vm_const_input_aux;
                self.bcb.emit_ad(insn.op, reg_input_0, 0);
                self.bcb.emit_aux(aux);
            }
            LuauOpcode::LOP_IDIV => {
                let reg_input_0 = self.get_reg_input(insn, 0);
                let reg_input_1 = self.get_reg_input(insn, 1);
                let out = self.get_register(insn_op);
                self.bcb
                    .emit_abc(LuauOpcode::LOP_IDIV, out, reg_input_0, reg_input_1);
            }
            LuauOpcode::LOP_IDIVK => {
                let reg_input_0 = self.get_reg_input(insn, 0);
                let vm_const_input_abc = self.get_vm_const_input_abc(insn, 1);
                let out = self.get_register(insn_op);
                self.bcb
                    .emit_abc(LuauOpcode::LOP_IDIVK, out, reg_input_0, vm_const_input_abc);
            }
            LuauOpcode::LOP_NEWCLASSMEMBER => {
                LUAU_ASSERT!(luaur_common::FFlag::DebugLuauUserDefinedClasses.get());
                let reg_input_0 = self.get_reg_input(insn, 0);
                let reg_input_1 = self.get_reg_input(insn, 1);
                self.bcb
                    .emit_abc(LuauOpcode::LOP_NEWCLASSMEMBER, reg_input_0, 0, reg_input_1);
                let vm_const_input_aux = self.get_vm_const_input_aux(insn, 2);
                self.bcb.emit_aux(vm_const_input_aux);
            }
            LuauOpcode::LOP_CMPPROTO => {
                self.record_jump(insn, 2);
                let reg_input_0 = self.get_reg_input(insn, 0);
                let imm_int_1 = self.get_imm_int(insn, 1);
                self.bcb.emit_ad(LuauOpcode::LOP_CMPPROTO, reg_input_0, 0);
                self.bcb.emit_aux(imm_int_1 as u32);
            }
            LuauOpcode::LOP__COUNT => {
                LUAU_UNREACHABLE!();
            }
        }
    }
}
