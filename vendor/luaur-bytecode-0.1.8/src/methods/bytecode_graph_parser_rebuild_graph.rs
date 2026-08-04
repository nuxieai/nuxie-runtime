use crate::enums::bc_block_edge_kind::BcBlockEdgeKind;
use crate::enums::bc_imm_kind::BcImmKind;
use crate::enums::bc_op_kind::BcOpKind;
use crate::records::bc_imm::BcImm;
use crate::records::bc_inst::BcInst;
use crate::records::bc_op::BcOp;
use crate::records::bc_op_hash::BcOpHash;
use crate::records::bytecode_graph_parser::BytecodeGraphParser;
use crate::type_aliases::instruction::Instruction;
use crate::type_aliases::reg::Reg;
use alloc::vec::Vec;
use luaur_common::enums::luau_opcode::LuauOpcode;
use luaur_common::functions::get_jump_target::get_jump_target;
use luaur_common::functions::get_op_length::get_op_length;
use luaur_common::macros::luau_assert::LUAU_ASSERT;
use luaur_common::macros::luau_insn_a::LUAU_INSN_A;
use luaur_common::macros::luau_insn_b::LUAU_INSN_B;
use luaur_common::macros::luau_insn_c::LUAU_INSN_C;
use luaur_common::macros::luau_insn_d::LUAU_INSN_D;
use luaur_common::macros::luau_insn_e::LUAU_INSN_E;
use luaur_common::macros::luau_insn_op::LUAU_INSN_OP;
use std::collections::HashSet;
impl<'a> BytecodeGraphParser<'a> {
    pub fn rebuild_graph(
        &mut self,
        code: *const Instruction,
        codesize: u32,
        lines: &mut Vec<u32>,
        pcs: &mut Vec<u32>,
    ) -> bool {
        let instructions_count = self.rebuild_blocks(code, codesize);
        if self.block_by_pc.size() > Self::K_MAX_CFG_BLOCKS as usize {
            return false;
        }

        let mut loops: Vec<crate::records::loop_info::LoopInfo> = Vec::new();

        self.producers
            .resize(self.func.blocks.len(), Default::default());
        pcs.resize(codesize as usize, 0);

        self.current_block = self.func.entry_block;

        for i in 0..self.func.numparams {
            self.add_producer(i, BcOp::bc_op_bc_op_kind_u32(BcOpKind::VmReg, i as u32));
        }

        // Create instructions.
        self.current_block = self.func.entry_block;
        self.func.instructions.reserve(instructions_count);

        let mut i: u32 = 0;
        while i < codesize {
            let insn = unsafe { *code.add(i as usize) };
            let op: LuauOpcode = unsafe { core::mem::transmute((LUAU_INSN_OP(insn) & 0xff) as u8) };
            let op_length = get_op_length(op) as u32;
            let aux = if op_length > 1 && i + 1 < codesize {
                unsafe { *code.add((i + 1) as usize) }
            } else {
                0
            };
            let node_op = self.func.add_inst();
            self.func
                .block_op(self.current_block)
                .append_instruction(node_op);
            let node: *mut BcInst = self.func.inst_op(node_op);
            unsafe {
                (*node).block = self.current_block;
            }
            if (i as usize) < lines.len() {
                unsafe {
                    (*node).line = lines[i as usize];
                }
            }
            unsafe {
                (*node).op = op;
            }

            pcs[i as usize] = node_op.index;

            let parse_jump = |parser: &mut BytecodeGraphParser,
                              op: LuauOpcode,
                              jump_target: i32,
                              insn: u32,
                              aux: u32,
                              node_op: BcOp| {
                let node: *mut BcInst = parser.func.inst_op(node_op);
                unsafe {
                    (*node).op = op;
                }
                match op {
                    LuauOpcode::LOP_JUMPXEQKNIL => {
                        parser.add_vm_reg_input(node, LUAU_INSN_A(insn) as u8);
                        parser.add_imm_input_bc_inst_bool(node, (aux >> 31) != 0);
                        parser.add_jump_input(node, jump_target);
                    }
                    LuauOpcode::LOP_JUMPXEQKB => {
                        parser.add_vm_reg_input(node, LUAU_INSN_A(insn) as u8);
                        parser.add_imm_input_bc_inst_bool(node, (aux >> 31) != 0);
                        parser.add_jump_input(node, jump_target);
                        parser.add_imm_input_bc_inst_bool(node, (aux & 0x1) != 0);
                    }
                    LuauOpcode::LOP_JUMPXEQKN | LuauOpcode::LOP_JUMPXEQKS => {
                        parser.add_vm_reg_input(node, LUAU_INSN_A(insn) as u8);
                        parser.add_imm_input_bc_inst_bool(node, (aux >> 31) != 0);
                        parser.add_jump_input(node, jump_target);
                        parser.add_vm_const_input(node, aux & 0xFFFFFF);
                    }
                    LuauOpcode::LOP_JUMPIF | LuauOpcode::LOP_JUMPIFNOT => {
                        parser.add_vm_reg_input(node, LUAU_INSN_A(insn) as u8);
                        parser.add_jump_input(node, jump_target);
                    }
                    LuauOpcode::LOP_JUMPIFEQ
                    | LuauOpcode::LOP_JUMPIFLE
                    | LuauOpcode::LOP_JUMPIFLT
                    | LuauOpcode::LOP_JUMPIFNOTEQ
                    | LuauOpcode::LOP_JUMPIFNOTLE
                    | LuauOpcode::LOP_JUMPIFNOTLT => {
                        parser.add_vm_reg_input(node, LUAU_INSN_A(insn) as u8);
                        parser.add_vm_reg_input(node, aux as u8);
                        parser.add_jump_input(node, jump_target);
                    }
                    LuauOpcode::LOP_FORNPREP => {
                        // forg loop protocol: A, A+1, A+2 are used for iteration protocol; A+3, ... are loop variables
                        parser.add_vm_reg_input(node, LUAU_INSN_A(insn) as u8);
                        parser.add_vm_reg_input(node, (LUAU_INSN_A(insn) + 1) as u8);
                        parser.add_vm_reg_input(node, (LUAU_INSN_A(insn) + 2) as u8);
                        parser.add_jump_input(node, jump_target);
                        let node: *mut BcInst = parser.func.inst_op(node_op);
                        parser.func.regs.insert(node_op, LUAU_INSN_A(insn) as u8);
                        let __proj1 = parser.func.add_proj(node_op, 0);
                        parser.add_producer(LUAU_INSN_A(insn) as u8, __proj1);
                        let __proj2 = parser.func.add_proj(node_op, 1);
                        parser.add_producer((LUAU_INSN_A(insn) + 1) as u8, __proj2);
                        let __proj3 = parser.func.add_proj(node_op, 2);
                        parser.add_producer((LUAU_INSN_A(insn) + 2) as u8, __proj3);
                    }
                    LuauOpcode::LOP_FORNLOOP => {
                        parser.add_vm_reg_input(node, LUAU_INSN_A(insn) as u8);
                        parser.add_vm_reg_input(node, (LUAU_INSN_A(insn) + 1) as u8);
                        parser.add_vm_reg_input(node, (LUAU_INSN_A(insn) + 2) as u8);
                        parser.add_jump_input(node, jump_target);
                    }
                    _ => {
                        LUAU_ASSERT!(false);
                    }
                }
            };

            match op {
                LuauOpcode::LOP_NOP | LuauOpcode::LOP_BREAK | LuauOpcode::LOP_NATIVECALL => {}

                LuauOpcode::LOP_LOADNIL => {
                    self.add_producer(LUAU_INSN_A(insn) as u8, node_op);
                }

                LuauOpcode::LOP_LOADB => {
                    self.add_imm_input_bc_inst_bool(node, LUAU_INSN_B(insn) != 0);
                    self.add_jump_input(node, get_jump_target(insn, i));
                    self.add_producer(LUAU_INSN_A(insn) as u8, node_op);
                }

                LuauOpcode::LOP_LOADN => {
                    self.add_imm_input_bc_inst_i32(node, LUAU_INSN_D(insn));
                    self.add_producer(LUAU_INSN_A(insn) as u8, node_op);
                }

                LuauOpcode::LOP_LOADK => {
                    self.add_vm_const_input(node, LUAU_INSN_D(insn) as u32);
                    self.add_producer(LUAU_INSN_A(insn) as u8, node_op);
                }

                LuauOpcode::LOP_MOVE => {
                    self.add_vm_reg_input(node, LUAU_INSN_B(insn) as u8);
                    self.add_producer(LUAU_INSN_A(insn) as u8, node_op);
                }

                LuauOpcode::LOP_GETGLOBAL => {
                    self.add_imm_input_bc_inst_i32(node, LUAU_INSN_C(insn) as i32);
                    self.add_vm_const_input(node, aux);
                    self.add_producer(LUAU_INSN_A(insn) as u8, node_op);
                }

                LuauOpcode::LOP_SETGLOBAL => {
                    self.add_vm_reg_input(node, LUAU_INSN_A(insn) as u8);
                    self.add_imm_input_bc_inst_i32(node, LUAU_INSN_C(insn) as u8 as i32);
                    self.add_vm_const_input(node, aux);
                }

                LuauOpcode::LOP_GETUPVAL => {
                    self.add_upval_input(node, LUAU_INSN_B(insn));
                    self.add_producer(LUAU_INSN_A(insn) as u8, node_op);
                }

                LuauOpcode::LOP_SETUPVAL => {
                    self.add_vm_reg_input(node, LUAU_INSN_A(insn) as u8);
                    self.add_upval_input(node, LUAU_INSN_B(insn));
                }

                LuauOpcode::LOP_CLOSEUPVALS => unsafe {
                    (*node).ops.push_back(BcOp::bc_op_bc_op_kind_u32(
                        BcOpKind::VmReg,
                        LUAU_INSN_A(insn),
                    ));
                },

                LuauOpcode::LOP_GETIMPORT => {
                    self.add_vm_const_input(node, LUAU_INSN_D(insn) as u32);
                    self.add_imm_input_bc_inst_u32(node, aux);
                    self.add_producer(LUAU_INSN_A(insn) as u8, node_op);
                }

                LuauOpcode::LOP_GETTABLE => {
                    self.add_vm_reg_input(node, LUAU_INSN_B(insn) as u8);
                    self.add_vm_reg_input(node, LUAU_INSN_C(insn) as u8);
                    self.add_producer(LUAU_INSN_A(insn) as u8, node_op);
                }

                LuauOpcode::LOP_SETTABLE => {
                    self.add_vm_reg_input(node, LUAU_INSN_A(insn) as u8);
                    self.add_vm_reg_input(node, LUAU_INSN_B(insn) as u8);
                    self.add_vm_reg_input(node, LUAU_INSN_C(insn) as u8);
                }

                LuauOpcode::LOP_GETUDATAKS | LuauOpcode::LOP_GETTABLEKS => {
                    self.add_vm_reg_input(node, LUAU_INSN_B(insn) as u8);
                    self.add_imm_input_bc_inst_i32(node, LUAU_INSN_C(insn) as i32);
                    self.add_vm_const_input(node, aux);
                    self.add_producer(LUAU_INSN_A(insn) as u8, node_op);
                }

                LuauOpcode::LOP_SETUDATAKS | LuauOpcode::LOP_SETTABLEKS => {
                    self.add_vm_reg_input(node, LUAU_INSN_A(insn) as u8);
                    self.add_vm_reg_input(node, LUAU_INSN_B(insn) as u8);
                    self.add_imm_input_bc_inst_i32(node, LUAU_INSN_C(insn) as i32);
                    self.add_vm_const_input(node, aux);
                }

                LuauOpcode::LOP_GETTABLEN => {
                    self.add_vm_reg_input(node, LUAU_INSN_B(insn) as u8);
                    self.add_imm_input_bc_inst_i32(node, (LUAU_INSN_C(insn) + 1) as i32);
                    self.add_producer(LUAU_INSN_A(insn) as u8, node_op);
                }

                LuauOpcode::LOP_SETTABLEN => {
                    self.add_vm_reg_input(node, LUAU_INSN_A(insn) as u8);
                    self.add_vm_reg_input(node, LUAU_INSN_B(insn) as u8);
                    self.add_imm_input_bc_inst_i32(node, (LUAU_INSN_C(insn) + 1) as i32);
                }

                LuauOpcode::LOP_NEWCLOSURE => {
                    self.add_proto_input(node, LUAU_INSN_D(insn) as u32);
                    self.add_producer(LUAU_INSN_A(insn) as u8, node_op);
                }

                LuauOpcode::LOP_NAMECALLUDATA | LuauOpcode::LOP_NAMECALL => {
                    self.add_vm_reg_input(node, LUAU_INSN_B(insn) as u8);
                    self.add_imm_input_bc_inst_i32(node, LUAU_INSN_C(insn) as i32);
                    self.add_vm_const_input(node, aux);
                    self.func.regs.insert(node_op, LUAU_INSN_A(insn) as u8);
                    let __proj4 = self.func.add_proj(node_op, 0);
                    self.add_producer(LUAU_INSN_A(insn) as u8, __proj4);
                    let __proj5 = self.func.add_proj(node_op, 1);
                    self.add_producer((LUAU_INSN_A(insn) + 1) as u8, __proj5);
                }

                LuauOpcode::LOP_CALL | LuauOpcode::LOP_CALLFB => {
                    let nparams = LUAU_INSN_B(insn) as i32 - 1;
                    let nresults = LUAU_INSN_C(insn) as i32 - 1;
                    let node: *mut BcInst = self.func.inst_op(node_op);
                    self.add_imm_input_bc_inst_i32(node, nparams);
                    self.add_imm_input_bc_inst_i32(node, nresults);
                    if op == LuauOpcode::LOP_CALLFB {
                        self.add_imm_input_bc_inst_i32(node, aux as i32);
                    }

                    // Call target.
                    self.add_vm_reg_input(node, LUAU_INSN_A(insn) as u8);
                    // Fixed arguments.
                    let mut j = 1i32;
                    while j <= nparams {
                        self.add_vm_reg_input(node, (LUAU_INSN_A(insn) as i32 + j) as u8);
                        j += 1;
                    }

                    if nparams < 0 {
                        let producers_up_to_top = self.find_producers_up_to_top(
                            self.current_block,
                            (LUAU_INSN_A(insn) + 1) as u8,
                        );
                        let node: *mut BcInst = self.func.inst_op(node_op);
                        for inp in producers_up_to_top {
                            unsafe {
                                (*node).ops.push_back(inp);
                            }
                        }
                    }

                    let block_producers: *mut _ =
                        &mut self.producers[self.current_block.index as usize];
                    self.apply_call(
                        unsafe { &mut *block_producers },
                        node_op,
                        LUAU_INSN_A(insn) as u8,
                        nresults,
                    );

                    self.func.regs.insert(node_op, LUAU_INSN_A(insn) as u8);
                    let mut j = 0i32;
                    while j < nresults {
                        let __proj6 = self.func.add_proj(node_op, j as u32);
                        self.add_producer((LUAU_INSN_A(insn) as i32 + j) as u8, __proj6);
                        j += 1;
                    }
                }

                LuauOpcode::LOP_RETURN => {
                    let nresults = LUAU_INSN_B(insn) as i32 - 1;
                    let node: *mut BcInst = self.func.inst_op(node_op);
                    self.add_imm_input_bc_inst_i32(node, nresults);
                    let mut j = 0i32;
                    while j < nresults {
                        self.add_vm_reg_input(node, (LUAU_INSN_A(insn) as i32 + j) as u8);
                        j += 1;
                    }
                    if nresults < 0 {
                        let producers_up_to_top = self
                            .find_producers_up_to_top(self.current_block, LUAU_INSN_A(insn) as u8);
                        let node: *mut BcInst = self.func.inst_op(node_op);
                        for inp in producers_up_to_top {
                            unsafe {
                                (*node).ops.push_back(inp);
                            }
                        }
                    }
                    if nresults == 0 {
                        let node: *mut BcInst = self.func.inst_op(node_op);
                        unsafe {
                            (*node).ops.push_back(BcOp::bc_op_bc_op_kind_u32(
                                BcOpKind::VmReg,
                                LUAU_INSN_A(insn),
                            ));
                        }
                    }
                }

                LuauOpcode::LOP_JUMP => {
                    if self.is_jump_trampoline(i, code, codesize) {
                        // it is long jump trampoline
                        let long_offset = LUAU_INSN_E(unsafe { *code.add((i + 1) as usize) });
                        i += get_op_length(LuauOpcode::LOP_JUMP) as u32
                            + get_op_length(LuauOpcode::LOP_JUMPX) as u32;
                        let next_insn = unsafe { *code.add(i as usize) };
                        let next_op: LuauOpcode =
                            unsafe { core::mem::transmute((LUAU_INSN_OP(next_insn) & 0xff) as u8) };
                        let next_op_length = get_op_length(next_op) as u32;
                        let next_aux = if next_op_length > 1 && i + 1 < codesize {
                            unsafe { *code.add((i + 1) as usize) }
                        } else {
                            0
                        };
                        parse_jump(
                            self,
                            next_op,
                            (i as i32) + long_offset,
                            next_insn,
                            next_aux,
                            node_op,
                        );
                    } else {
                        self.add_jump_input(node, get_jump_target(insn, i));
                    }
                }

                LuauOpcode::LOP_JUMPBACK => {
                    // repeat .. until loops use it for back edge.
                    self.add_jump_input(node, get_jump_target(insn, i));
                }

                LuauOpcode::LOP_JUMPXEQKNIL
                | LuauOpcode::LOP_JUMPXEQKB
                | LuauOpcode::LOP_JUMPXEQKN
                | LuauOpcode::LOP_JUMPXEQKS
                | LuauOpcode::LOP_JUMPIF
                | LuauOpcode::LOP_JUMPIFNOT
                | LuauOpcode::LOP_JUMPIFEQ
                | LuauOpcode::LOP_JUMPIFLE
                | LuauOpcode::LOP_JUMPIFLT
                | LuauOpcode::LOP_JUMPIFNOTEQ
                | LuauOpcode::LOP_JUMPIFNOTLE
                | LuauOpcode::LOP_JUMPIFNOTLT
                | LuauOpcode::LOP_FORNPREP
                | LuauOpcode::LOP_FORNLOOP => {
                    parse_jump(self, op, get_jump_target(insn, i), insn, aux, node_op);
                }

                LuauOpcode::LOP_ADD
                | LuauOpcode::LOP_SUB
                | LuauOpcode::LOP_MUL
                | LuauOpcode::LOP_DIV
                | LuauOpcode::LOP_MOD
                | LuauOpcode::LOP_POW
                | LuauOpcode::LOP_AND
                | LuauOpcode::LOP_OR => {
                    self.add_vm_reg_input(node, LUAU_INSN_B(insn) as u8);
                    self.add_vm_reg_input(node, LUAU_INSN_C(insn) as u8);
                    self.add_producer(LUAU_INSN_A(insn) as u8, node_op);
                }

                LuauOpcode::LOP_ADDK
                | LuauOpcode::LOP_SUBK
                | LuauOpcode::LOP_MULK
                | LuauOpcode::LOP_DIVK
                | LuauOpcode::LOP_MODK
                | LuauOpcode::LOP_POWK
                | LuauOpcode::LOP_ANDK
                | LuauOpcode::LOP_ORK => {
                    self.add_vm_reg_input(node, LUAU_INSN_B(insn) as u8);
                    self.add_vm_const_input(node, LUAU_INSN_C(insn));
                    self.add_producer(LUAU_INSN_A(insn) as u8, node_op);
                }

                LuauOpcode::LOP_CONCAT => {
                    LUAU_ASSERT!(LUAU_INSN_B(insn) <= LUAU_INSN_C(insn));
                    let mut param = LUAU_INSN_B(insn);
                    while param <= LUAU_INSN_C(insn) {
                        self.add_vm_reg_input(node, param as u8);
                        param += 1;
                    }
                    self.add_producer(LUAU_INSN_A(insn) as u8, node_op);
                }

                LuauOpcode::LOP_NOT | LuauOpcode::LOP_MINUS | LuauOpcode::LOP_LENGTH => {
                    self.add_vm_reg_input(node, LUAU_INSN_B(insn) as u8);
                    self.add_producer(LUAU_INSN_A(insn) as u8, node_op);
                }

                LuauOpcode::LOP_NEWTABLE => {
                    self.add_imm_input_bc_inst_i32(node, LUAU_INSN_B(insn) as i32);
                    self.add_imm_input_bc_inst_i32(node, aux as i32);
                    self.add_producer(LUAU_INSN_A(insn) as u8, node_op);
                }

                LuauOpcode::LOP_DUPTABLE => {
                    self.add_producer(LUAU_INSN_A(insn) as u8, node_op);
                    self.add_vm_const_input(node, LUAU_INSN_D(insn) as u32);
                }

                LuauOpcode::LOP_SETLIST => {
                    let count = LUAU_INSN_C(insn) as i32 - 1;
                    self.add_imm_input_bc_inst_i32(node, aux as i32);
                    self.add_imm_input_bc_inst_i32(node, count);
                    self.add_vm_reg_input(node, LUAU_INSN_A(insn) as u8);
                    let mut param = 0i32;
                    while param < count {
                        self.add_vm_reg_input(node, (LUAU_INSN_B(insn) as i32 + param) as u8);
                        param += 1;
                    }
                    if count < 0 {
                        let producers_up_to_top = self
                            .find_producers_up_to_top(self.current_block, LUAU_INSN_B(insn) as u8);
                        let node: *mut BcInst = self.func.inst_op(node_op);
                        for inp in producers_up_to_top {
                            unsafe {
                                (*node).ops.push_back(inp);
                            }
                        }
                    }
                }

                LuauOpcode::LOP_FORGPREP
                | LuauOpcode::LOP_FORGPREP_NEXT
                | LuauOpcode::LOP_FORGPREP_INEXT => {
                    self.add_vm_reg_input(node, LUAU_INSN_A(insn) as u8);
                    self.add_vm_reg_input(node, (LUAU_INSN_A(insn) + 1) as u8);
                    self.add_vm_reg_input(node, (LUAU_INSN_A(insn) + 2) as u8);
                    let loop_insn_pc = get_jump_target(insn, i);
                    self.add_jump_input(node, loop_insn_pc);
                    let loop_insn = unsafe { *code.add(loop_insn_pc as usize) };
                    let loop_insn_op: LuauOpcode =
                        unsafe { core::mem::transmute((LUAU_INSN_OP(loop_insn) & 0xff) as u8) };
                    LUAU_ASSERT!(
                        loop_insn_pc + 1 < codesize as i32
                            && loop_insn_op == LuauOpcode::LOP_FORGLOOP
                    );
                    let vars = unsafe { *code.add((loop_insn_pc + 1) as usize) } as i32 & 0xFF;
                    self.func.regs.insert(node_op, LUAU_INSN_A(insn) as u8);
                    let mut idx = 0i32;
                    while idx <= core::cmp::max(vars, 2) {
                        let __proj7 = self.func.add_proj(node_op, (2 + idx) as u32);
                        self.add_producer((LUAU_INSN_A(insn) as i32 + 2 + idx) as u8, __proj7);
                        idx += 1;
                    }
                }

                LuauOpcode::LOP_FORGLOOP => {
                    self.add_vm_reg_input(node, LUAU_INSN_A(insn) as u8);
                    self.add_vm_reg_input(node, (LUAU_INSN_A(insn) + 1) as u8);
                    self.add_vm_reg_input(node, (LUAU_INSN_A(insn) + 2) as u8);
                    self.add_imm_input_bc_inst_bool(node, (aux >> 31) != 0);
                    let vars = (aux & 0xFF) as i32;
                    self.add_imm_input_bc_inst_i32(node, vars);
                    self.add_jump_input(node, get_jump_target(insn, i));
                }

                LuauOpcode::LOP_FASTCALL => {
                    // Note that FASTCALL will read the actual call arguments, such as argument/result registers and counts, from the CALL instruction
                    self.add_imm_input_bc_inst_i32(node, LUAU_INSN_A(insn) as i32);
                    // turn it in BcOp to CALL BcInst&.
                    self.add_imm_input_bc_inst_i32(node, LUAU_INSN_C(insn) as i32);
                }

                LuauOpcode::LOP_FASTCALL1 => {
                    self.add_imm_input_bc_inst_i32(node, LUAU_INSN_A(insn) as i32);
                    self.add_vm_reg_input(node, LUAU_INSN_B(insn) as u8);
                    // turn it in BcOp to CALL BcInst&.
                    self.add_imm_input_bc_inst_i32(node, LUAU_INSN_C(insn) as i32);
                }

                LuauOpcode::LOP_FASTCALL2 => {
                    self.add_imm_input_bc_inst_i32(node, LUAU_INSN_A(insn) as i32);
                    self.add_vm_reg_input(node, LUAU_INSN_B(insn) as u8);
                    self.add_vm_reg_input(node, (aux & 0xFF) as u8);
                    // turn it in BcOp to CALL BcInst&.
                    self.add_imm_input_bc_inst_i32(node, LUAU_INSN_C(insn) as i32);
                }

                LuauOpcode::LOP_FASTCALL2K => {
                    self.add_imm_input_bc_inst_i32(node, LUAU_INSN_A(insn) as i32);
                    self.add_vm_reg_input(node, LUAU_INSN_B(insn) as u8);
                    self.add_vm_const_input(node, aux);
                    // turn it in BcOp to CALL BcInst&.
                    self.add_imm_input_bc_inst_i32(node, LUAU_INSN_C(insn) as i32);
                }

                LuauOpcode::LOP_FASTCALL3 => {
                    self.add_imm_input_bc_inst_i32(node, LUAU_INSN_A(insn) as i32);
                    self.add_vm_reg_input(node, LUAU_INSN_B(insn) as u8);
                    self.add_vm_reg_input(node, (aux & 0xFF) as u8);
                    self.add_vm_reg_input(node, ((aux >> 8) & 0xFF) as u8);
                    // turn it in BcOp to CALL BcInst&.
                    self.add_imm_input_bc_inst_i32(node, LUAU_INSN_C(insn) as i32);
                }

                LuauOpcode::LOP_GETVARARGS => {
                    unsafe {
                        (*node).ops.push_back(BcOp::bc_op_bc_op_kind_u32(
                            BcOpKind::VmReg,
                            LUAU_INSN_A(insn),
                        ));
                    }
                    let count = LUAU_INSN_B(insn) as i32 - 1;
                    self.add_imm_input_bc_inst_i32(node, count);
                    self.func.regs.insert(node_op, LUAU_INSN_A(insn) as u8);
                    if count < 0 {
                        let block_producers =
                            &mut self.producers[self.current_block.index as usize];
                        block_producers.multiReturn = node_op;
                        block_producers.multiReturnStart = LUAU_INSN_A(insn) as u8;
                        block_producers.invalidAfter = 255;
                    } else {
                        let mut j = 0i32;
                        while j < count {
                            let __proj8 = self.func.add_proj(node_op, j as u32);
                            self.add_producer((LUAU_INSN_A(insn) as i32 + j) as u8, __proj8);
                            j += 1;
                        }
                    }
                }

                LuauOpcode::LOP_DUPCLOSURE => {
                    self.add_vm_const_input(node, LUAU_INSN_D(insn) as u32);
                    self.add_producer(LUAU_INSN_A(insn) as u8, node_op);
                }

                LuauOpcode::LOP_PREPVARARGS => {
                    self.add_imm_input_bc_inst_i32(node, LUAU_INSN_A(insn) as i32);
                }

                LuauOpcode::LOP_LOADKX => {
                    self.add_vm_const_input(node, aux);
                    self.add_producer(LUAU_INSN_A(insn) as u8, node_op);
                }

                LuauOpcode::LOP_JUMPX => {
                    LUAU_ASSERT!(false);
                    self.add_jump_input(node, get_jump_target(insn, i));
                }

                LuauOpcode::LOP_COVERAGE => {
                    self.add_imm_input_bc_inst_i32(node, LUAU_INSN_E(insn));
                }

                LuauOpcode::LOP_CAPTURE => {
                    let capture_type = LUAU_INSN_A(insn);
                    self.add_imm_input_bc_inst_i32(node, capture_type as i32);
                    if capture_type
                        == luaur_common::enums::luau_capture_type::LuauCaptureType::LCT_VAL as u32
                        || capture_type
                            == luaur_common::enums::luau_capture_type::LuauCaptureType::LCT_REF
                                as u32
                    {
                        self.add_vm_reg_input(node, LUAU_INSN_B(insn) as u8);
                    } else {
                        self.add_upval_input(node, LUAU_INSN_B(insn));
                    }
                    self.add_imm_input_bc_inst_i32(node, LUAU_INSN_C(insn) as i32);
                }

                LuauOpcode::LOP_SUBRK | LuauOpcode::LOP_DIVRK => {
                    self.add_vm_const_input(node, LUAU_INSN_B(insn));
                    self.add_vm_reg_input(node, LUAU_INSN_C(insn) as u8);
                    self.add_producer(LUAU_INSN_A(insn) as u8, node_op);
                }

                LuauOpcode::LOP_IDIV => {
                    self.add_vm_reg_input(node, LUAU_INSN_B(insn) as u8);
                    self.add_vm_reg_input(node, LUAU_INSN_C(insn) as u8);
                    self.add_producer(LUAU_INSN_A(insn) as u8, node_op);
                }

                LuauOpcode::LOP_IDIVK => {
                    self.add_vm_reg_input(node, LUAU_INSN_B(insn) as u8);
                    self.add_vm_const_input(node, LUAU_INSN_C(insn));
                    self.add_producer(LUAU_INSN_A(insn) as u8, node_op);
                }

                LuauOpcode::LOP_CMPPROTO => {
                    self.add_vm_reg_input(node, LUAU_INSN_A(insn) as u8);
                    self.add_imm_input_bc_inst_i32(node, aux as i32);
                    self.add_jump_input(node, get_jump_target(insn, i));
                }

                LuauOpcode::LOP_NEWCLASSMEMBER => {
                    LUAU_ASSERT!(luaur_common::FFlag::DebugLuauUserDefinedClasses.get());
                    self.add_vm_reg_input(node, LUAU_INSN_A(insn) as u8);
                    self.add_vm_reg_input(node, LUAU_INSN_C(insn) as u8);
                    self.add_vm_const_input(node, aux);
                }

                LuauOpcode::LOP__COUNT => {
                    LUAU_ASSERT!(false);
                }

                _ => {}
            }

            if luaur_common::functions::is_loop_jump::is_loop_jump(op) {
                let target = get_jump_target(insn, i);
                LUAU_ASSERT!(target >= 0 && self.block_by_pc.contains_key(&(target as u32)));
                loops.push(crate::records::loop_info::LoopInfo {
                    entry: *self.block_by_pc.get(&(target as u32)).unwrap(),
                    exit: self.current_block,
                });
            }

            i += op_length;
            if self.block_by_pc.contains_key(&i) {
                self.current_block = *self.block_by_pc.get(&i).unwrap();
            }
        }

        for loop_ in &loops {
            let mut visited: HashSet<BcOp, BcOpHash> = HashSet::default();
            let mut queue: Vec<BcOp> = Vec::new();
            queue.push(loop_.exit);
            while !queue.is_empty() {
                let cur = queue.pop().unwrap();
                if visited.contains(&cur) {
                    continue;
                }
                visited.insert(cur);
                let predecessors: alloc::collections::VecDeque<(BcBlockEdgeKind, BcOp)> = {
                    let bl = self.func.block_op(cur);
                    bl.predecessors.iter().map(|e| (e.kind, e.target)).collect()
                };
                let ops: alloc::collections::VecDeque<BcOp> = {
                    let bl = self.func.block_op(cur);
                    bl.ops.clone()
                };

                for op in &ops {
                    let inst_ops: crate::type_aliases::bc_ops::BcOps = {
                        let inst = self.func.inst_op(*op);
                        inst.ops.clone()
                    };
                    for inp_idx in 0..inst_ops.len() {
                        let inp = inst_ops[inp_idx];
                        let reg_it = self.func.regs.get(&inp);
                        let Some(reg) = reg_it.copied() else {
                            continue;
                        };
                        // try to find it in the same loop before
                        if self.has_producer_before_bc_op_bc_op_bc_op_reg(
                            loop_.entry,
                            cur,
                            *op,
                            reg,
                        ) {
                            continue;
                        }
                        if let Some(forward_input) = self
                            .find_forward_producer_in_range_bc_op_bc_op_bc_op_reg(
                                cur, loop_.exit, *op, reg,
                            )
                        {
                            let inst: *mut BcInst = self.func.inst_op(*op);
                            let op_val = unsafe {
                                let ops = &(*inst).ops;
                                ops[inp_idx]
                            };
                            let new_val = self.add_to_phi(op_val, forward_input);
                            unsafe {
                                let ops = &mut (*inst).ops;
                                ops[inp_idx] = new_val;
                            }
                            self.func.regs.insert(new_val, reg);
                        }
                    }
                }

                for &(ctrl, pred) in &predecessors {
                    if ctrl != BcBlockEdgeKind::Loop && !visited.contains(&pred) {
                        queue.push(pred);
                    }
                }
            }
        }
        true
    }
}
