use crate::records::bytecode_builder::BytecodeBuilder;
use alloc::string::String;
use core::ffi::c_int;
use luaur_common::enums::luau_opcode::LuauOpcode;
use luaur_common::functions::format_append::formatAppend;
use luaur_common::macros::luau_assert::LUAU_ASSERT;
use luaur_common::macros::luau_insn_a::LUAU_INSN_A;
use luaur_common::macros::luau_insn_aux_kv_16::LUAU_INSN_AUX_KV16;
use luaur_common::macros::luau_insn_b::LUAU_INSN_B;
use luaur_common::macros::luau_insn_c::LUAU_INSN_C;
use luaur_common::macros::luau_insn_d::LUAU_INSN_D;
use luaur_common::macros::luau_insn_op::LUAU_INSN_OP;

impl BytecodeBuilder {
    pub fn dump_instruction(&self, code: &[u32], result: &mut String, target_label: i32) -> usize {
        let insn = code[0];
        let op = LUAU_INSN_OP(insn);
        let op_enum = unsafe { core::mem::transmute::<u8, LuauOpcode>(op as u8) };

        match op_enum {
            LuauOpcode::LOP_LOADNIL => {
                formatAppend(result, format_args!("LOADNIL R{}\n", LUAU_INSN_A(insn)));
                1
            }
            LuauOpcode::LOP_LOADB => {
                if LUAU_INSN_C(insn) != 0 {
                    formatAppend(
                        result,
                        format_args!(
                            "LOADB R{} {} +{}\n",
                            LUAU_INSN_A(insn),
                            LUAU_INSN_B(insn),
                            LUAU_INSN_C(insn)
                        ),
                    );
                } else {
                    formatAppend(
                        result,
                        format_args!("LOADB R{} {}\n", LUAU_INSN_A(insn), LUAU_INSN_B(insn)),
                    );
                }
                1
            }
            LuauOpcode::LOP_LOADN => {
                formatAppend(
                    result,
                    format_args!("LOADN R{} {}\n", LUAU_INSN_A(insn), LUAU_INSN_D(insn)),
                );
                1
            }
            LuauOpcode::LOP_LOADK => {
                formatAppend(
                    result,
                    format_args!("LOADK R{} K{} [", LUAU_INSN_A(insn), LUAU_INSN_D(insn)),
                );
                self.dump_constant(result, LUAU_INSN_D(insn), false);
                result.push_str("]\n");
                1
            }
            LuauOpcode::LOP_MOVE => {
                formatAppend(
                    result,
                    format_args!("MOVE R{} R{}\n", LUAU_INSN_A(insn), LUAU_INSN_B(insn)),
                );
                1
            }
            LuauOpcode::LOP_GETGLOBAL => {
                formatAppend(
                    result,
                    format_args!("GETGLOBAL R{} K{} [", LUAU_INSN_A(insn), code[1]),
                );
                self.dump_constant(result, code[1] as i32, false);
                result.push_str("]\n");
                2
            }
            LuauOpcode::LOP_SETGLOBAL => {
                formatAppend(
                    result,
                    format_args!("SETGLOBAL R{} K{} [", LUAU_INSN_A(insn), code[1]),
                );
                self.dump_constant(result, code[1] as i32, false);
                result.push_str("]\n");
                2
            }
            LuauOpcode::LOP_GETUPVAL => {
                formatAppend(
                    result,
                    format_args!("GETUPVAL R{} {}\n", LUAU_INSN_A(insn), LUAU_INSN_B(insn)),
                );
                1
            }
            LuauOpcode::LOP_SETUPVAL => {
                formatAppend(
                    result,
                    format_args!("SETUPVAL R{} {}\n", LUAU_INSN_A(insn), LUAU_INSN_B(insn)),
                );
                1
            }
            LuauOpcode::LOP_CLOSEUPVALS => {
                formatAppend(result, format_args!("CLOSEUPVALS R{}\n", LUAU_INSN_A(insn)));
                1
            }
            LuauOpcode::LOP_GETIMPORT => {
                formatAppend(
                    result,
                    format_args!("GETIMPORT R{} {} [", LUAU_INSN_A(insn), LUAU_INSN_D(insn)),
                );
                self.dump_constant(result, LUAU_INSN_D(insn), false);
                result.push_str("]\n");
                2
            }
            LuauOpcode::LOP_GETTABLE => {
                formatAppend(
                    result,
                    format_args!(
                        "GETTABLE R{} R{} R{}\n",
                        LUAU_INSN_A(insn),
                        LUAU_INSN_B(insn),
                        LUAU_INSN_C(insn)
                    ),
                );
                1
            }
            LuauOpcode::LOP_SETTABLE => {
                formatAppend(
                    result,
                    format_args!(
                        "SETTABLE R{} R{} R{}\n",
                        LUAU_INSN_A(insn),
                        LUAU_INSN_B(insn),
                        LUAU_INSN_C(insn)
                    ),
                );
                1
            }
            LuauOpcode::LOP_GETTABLEKS => {
                formatAppend(
                    result,
                    format_args!(
                        "GETTABLEKS R{} R{} K{} [",
                        LUAU_INSN_A(insn),
                        LUAU_INSN_B(insn),
                        code[1]
                    ),
                );
                self.dump_constant(result, code[1] as i32, false);
                result.push_str("]\n");
                2
            }
            LuauOpcode::LOP_SETTABLEKS => {
                formatAppend(
                    result,
                    format_args!(
                        "SETTABLEKS R{} R{} K{} [",
                        LUAU_INSN_A(insn),
                        LUAU_INSN_B(insn),
                        code[1]
                    ),
                );
                self.dump_constant(result, code[1] as i32, false);
                result.push_str("]\n");
                2
            }
            LuauOpcode::LOP_GETTABLEN => {
                formatAppend(
                    result,
                    format_args!(
                        "GETTABLEN R{} R{} {}\n",
                        LUAU_INSN_A(insn),
                        LUAU_INSN_B(insn),
                        LUAU_INSN_C(insn) + 1
                    ),
                );
                1
            }
            LuauOpcode::LOP_SETTABLEN => {
                formatAppend(
                    result,
                    format_args!(
                        "SETTABLEN R{} R{} {}\n",
                        LUAU_INSN_A(insn),
                        LUAU_INSN_B(insn),
                        LUAU_INSN_C(insn) + 1
                    ),
                );
                1
            }
            LuauOpcode::LOP_NEWCLOSURE => {
                formatAppend(
                    result,
                    format_args!("NEWCLOSURE R{} P{}\n", LUAU_INSN_A(insn), LUAU_INSN_D(insn)),
                );
                1
            }
            LuauOpcode::LOP_NAMECALL => {
                formatAppend(
                    result,
                    format_args!(
                        "NAMECALL R{} R{} K{} [",
                        LUAU_INSN_A(insn),
                        LUAU_INSN_B(insn),
                        code[1]
                    ),
                );
                self.dump_constant(result, code[1] as i32, false);
                result.push_str("]\n");
                2
            }
            LuauOpcode::LOP_CALL => {
                formatAppend(
                    result,
                    format_args!(
                        "CALL R{} {} {}\n",
                        LUAU_INSN_A(insn),
                        LUAU_INSN_B(insn) as i32 - 1,
                        LUAU_INSN_C(insn) as i32 - 1
                    ),
                );
                1
            }
            LuauOpcode::LOP_CALLFB => {
                formatAppend(
                    result,
                    format_args!(
                        "CALLFB R{} {} {} [{}]\n",
                        LUAU_INSN_A(insn),
                        LUAU_INSN_B(insn) as i32 - 1,
                        LUAU_INSN_C(insn) as i32 - 1,
                        code[1] as c_int
                    ),
                );
                2
            }
            LuauOpcode::LOP_RETURN => {
                formatAppend(
                    result,
                    format_args!(
                        "RETURN R{} {}\n",
                        LUAU_INSN_A(insn),
                        LUAU_INSN_B(insn) as i32 - 1
                    ),
                );
                1
            }
            LuauOpcode::LOP_JUMP => {
                formatAppend(result, format_args!("JUMP L{}\n", target_label));
                1
            }
            LuauOpcode::LOP_JUMPIF => {
                formatAppend(
                    result,
                    format_args!("JUMPIF R{} L{}\n", LUAU_INSN_A(insn), target_label),
                );
                1
            }
            LuauOpcode::LOP_JUMPIFNOT => {
                formatAppend(
                    result,
                    format_args!("JUMPIFNOT R{} L{}\n", LUAU_INSN_A(insn), target_label),
                );
                1
            }
            LuauOpcode::LOP_JUMPIFEQ => {
                formatAppend(
                    result,
                    format_args!(
                        "JUMPIFEQ R{} R{} L{}\n",
                        LUAU_INSN_A(insn),
                        code[1],
                        target_label
                    ),
                );
                2
            }
            LuauOpcode::LOP_JUMPIFLE => {
                formatAppend(
                    result,
                    format_args!(
                        "JUMPIFLE R{} R{} L{}\n",
                        LUAU_INSN_A(insn),
                        code[1],
                        target_label
                    ),
                );
                2
            }
            LuauOpcode::LOP_JUMPIFLT => {
                formatAppend(
                    result,
                    format_args!(
                        "JUMPIFLT R{} R{} L{}\n",
                        LUAU_INSN_A(insn),
                        code[1],
                        target_label
                    ),
                );
                2
            }
            LuauOpcode::LOP_JUMPIFNOTEQ => {
                formatAppend(
                    result,
                    format_args!(
                        "JUMPIFNOTEQ R{} R{} L{}\n",
                        LUAU_INSN_A(insn),
                        code[1],
                        target_label
                    ),
                );
                2
            }
            LuauOpcode::LOP_JUMPIFNOTLE => {
                formatAppend(
                    result,
                    format_args!(
                        "JUMPIFNOTLE R{} R{} L{}\n",
                        LUAU_INSN_A(insn),
                        code[1],
                        target_label
                    ),
                );
                2
            }
            LuauOpcode::LOP_JUMPIFNOTLT => {
                formatAppend(
                    result,
                    format_args!(
                        "JUMPIFNOTLT R{} R{} L{}\n",
                        LUAU_INSN_A(insn),
                        code[1],
                        target_label
                    ),
                );
                2
            }
            LuauOpcode::LOP_ADD => {
                formatAppend(
                    result,
                    format_args!(
                        "ADD R{} R{} R{}\n",
                        LUAU_INSN_A(insn),
                        LUAU_INSN_B(insn),
                        LUAU_INSN_C(insn)
                    ),
                );
                1
            }
            LuauOpcode::LOP_SUB => {
                formatAppend(
                    result,
                    format_args!(
                        "SUB R{} R{} R{}\n",
                        LUAU_INSN_A(insn),
                        LUAU_INSN_B(insn),
                        LUAU_INSN_C(insn)
                    ),
                );
                1
            }
            LuauOpcode::LOP_MUL => {
                formatAppend(
                    result,
                    format_args!(
                        "MUL R{} R{} R{}\n",
                        LUAU_INSN_A(insn),
                        LUAU_INSN_B(insn),
                        LUAU_INSN_C(insn)
                    ),
                );
                1
            }
            LuauOpcode::LOP_DIV => {
                formatAppend(
                    result,
                    format_args!(
                        "DIV R{} R{} R{}\n",
                        LUAU_INSN_A(insn),
                        LUAU_INSN_B(insn),
                        LUAU_INSN_C(insn)
                    ),
                );
                1
            }
            LuauOpcode::LOP_IDIV => {
                formatAppend(
                    result,
                    format_args!(
                        "IDIV R{} R{} R{}\n",
                        LUAU_INSN_A(insn),
                        LUAU_INSN_B(insn),
                        LUAU_INSN_C(insn)
                    ),
                );
                1
            }
            LuauOpcode::LOP_MOD => {
                formatAppend(
                    result,
                    format_args!(
                        "MOD R{} R{} R{}\n",
                        LUAU_INSN_A(insn),
                        LUAU_INSN_B(insn),
                        LUAU_INSN_C(insn)
                    ),
                );
                1
            }
            LuauOpcode::LOP_POW => {
                formatAppend(
                    result,
                    format_args!(
                        "POW R{} R{} R{}\n",
                        LUAU_INSN_A(insn),
                        LUAU_INSN_B(insn),
                        LUAU_INSN_C(insn)
                    ),
                );
                1
            }
            LuauOpcode::LOP_ADDK => {
                formatAppend(
                    result,
                    format_args!(
                        "ADDK R{} R{} K{} [",
                        LUAU_INSN_A(insn),
                        LUAU_INSN_B(insn),
                        LUAU_INSN_C(insn)
                    ),
                );
                self.dump_constant(result, LUAU_INSN_C(insn) as i32, false);
                result.push_str("]\n");
                1
            }
            LuauOpcode::LOP_SUBK => {
                formatAppend(
                    result,
                    format_args!(
                        "SUBK R{} R{} K{} [",
                        LUAU_INSN_A(insn),
                        LUAU_INSN_B(insn),
                        LUAU_INSN_C(insn)
                    ),
                );
                self.dump_constant(result, LUAU_INSN_C(insn) as i32, false);
                result.push_str("]\n");
                1
            }
            LuauOpcode::LOP_MULK => {
                formatAppend(
                    result,
                    format_args!(
                        "MULK R{} R{} K{} [",
                        LUAU_INSN_A(insn),
                        LUAU_INSN_B(insn),
                        LUAU_INSN_C(insn)
                    ),
                );
                self.dump_constant(result, LUAU_INSN_C(insn) as i32, false);
                result.push_str("]\n");
                1
            }
            LuauOpcode::LOP_DIVK => {
                formatAppend(
                    result,
                    format_args!(
                        "DIVK R{} R{} K{} [",
                        LUAU_INSN_A(insn),
                        LUAU_INSN_B(insn),
                        LUAU_INSN_C(insn)
                    ),
                );
                self.dump_constant(result, LUAU_INSN_C(insn) as i32, false);
                result.push_str("]\n");
                1
            }
            LuauOpcode::LOP_IDIVK => {
                formatAppend(
                    result,
                    format_args!(
                        "IDIVK R{} R{} K{} [",
                        LUAU_INSN_A(insn),
                        LUAU_INSN_B(insn),
                        LUAU_INSN_C(insn)
                    ),
                );
                self.dump_constant(result, LUAU_INSN_C(insn) as i32, false);
                result.push_str("]\n");
                1
            }
            LuauOpcode::LOP_MODK => {
                formatAppend(
                    result,
                    format_args!(
                        "MODK R{} R{} K{} [",
                        LUAU_INSN_A(insn),
                        LUAU_INSN_B(insn),
                        LUAU_INSN_C(insn)
                    ),
                );
                self.dump_constant(result, LUAU_INSN_C(insn) as i32, false);
                result.push_str("]\n");
                1
            }
            LuauOpcode::LOP_POWK => {
                formatAppend(
                    result,
                    format_args!(
                        "POWK R{} R{} K{} [",
                        LUAU_INSN_A(insn),
                        LUAU_INSN_B(insn),
                        LUAU_INSN_C(insn)
                    ),
                );
                self.dump_constant(result, LUAU_INSN_C(insn) as i32, false);
                result.push_str("]\n");
                1
            }
            LuauOpcode::LOP_SUBRK => {
                formatAppend(
                    result,
                    format_args!("SUBRK R{} K{} [", LUAU_INSN_A(insn), LUAU_INSN_B(insn)),
                );
                self.dump_constant(result, LUAU_INSN_B(insn) as i32, false);
                formatAppend(result, format_args!("] R{}\n", LUAU_INSN_C(insn)));
                1
            }
            LuauOpcode::LOP_DIVRK => {
                formatAppend(
                    result,
                    format_args!("DIVRK R{} K{} [", LUAU_INSN_A(insn), LUAU_INSN_B(insn)),
                );
                self.dump_constant(result, LUAU_INSN_B(insn) as i32, false);
                formatAppend(result, format_args!("] R{}\n", LUAU_INSN_C(insn)));
                1
            }
            LuauOpcode::LOP_AND => {
                formatAppend(
                    result,
                    format_args!(
                        "AND R{} R{} R{}\n",
                        LUAU_INSN_A(insn),
                        LUAU_INSN_B(insn),
                        LUAU_INSN_C(insn)
                    ),
                );
                1
            }
            LuauOpcode::LOP_OR => {
                formatAppend(
                    result,
                    format_args!(
                        "OR R{} R{} R{}\n",
                        LUAU_INSN_A(insn),
                        LUAU_INSN_B(insn),
                        LUAU_INSN_C(insn)
                    ),
                );
                1
            }
            LuauOpcode::LOP_ANDK => {
                formatAppend(
                    result,
                    format_args!(
                        "ANDK R{} R{} K{} [",
                        LUAU_INSN_A(insn),
                        LUAU_INSN_B(insn),
                        LUAU_INSN_C(insn)
                    ),
                );
                self.dump_constant(result, LUAU_INSN_C(insn) as i32, false);
                result.push_str("]\n");
                1
            }
            LuauOpcode::LOP_ORK => {
                formatAppend(
                    result,
                    format_args!(
                        "ORK R{} R{} K{} [",
                        LUAU_INSN_A(insn),
                        LUAU_INSN_B(insn),
                        LUAU_INSN_C(insn)
                    ),
                );
                self.dump_constant(result, LUAU_INSN_C(insn) as i32, false);
                result.push_str("]\n");
                1
            }
            LuauOpcode::LOP_CONCAT => {
                formatAppend(
                    result,
                    format_args!(
                        "CONCAT R{} R{} R{}\n",
                        LUAU_INSN_A(insn),
                        LUAU_INSN_B(insn),
                        LUAU_INSN_C(insn)
                    ),
                );
                1
            }
            LuauOpcode::LOP_NOT => {
                formatAppend(
                    result,
                    format_args!("NOT R{} R{}\n", LUAU_INSN_A(insn), LUAU_INSN_B(insn)),
                );
                1
            }
            LuauOpcode::LOP_MINUS => {
                formatAppend(
                    result,
                    format_args!("MINUS R{} R{}\n", LUAU_INSN_A(insn), LUAU_INSN_B(insn)),
                );
                1
            }
            LuauOpcode::LOP_LENGTH => {
                formatAppend(
                    result,
                    format_args!("LENGTH R{} R{}\n", LUAU_INSN_A(insn), LUAU_INSN_B(insn)),
                );
                1
            }
            LuauOpcode::LOP_NEWTABLE => {
                formatAppend(
                    result,
                    format_args!(
                        "NEWTABLE R{} {} {}\n",
                        LUAU_INSN_A(insn),
                        if LUAU_INSN_B(insn) == 0 {
                            0
                        } else {
                            1 << (LUAU_INSN_B(insn) as i32 - 1)
                        },
                        code[1]
                    ),
                );
                2
            }
            LuauOpcode::LOP_DUPTABLE => {
                formatAppend(
                    result,
                    format_args!("DUPTABLE R{} {}\n", LUAU_INSN_A(insn), LUAU_INSN_D(insn)),
                );
                1
            }
            LuauOpcode::LOP_SETLIST => {
                formatAppend(
                    result,
                    format_args!(
                        "SETLIST R{} R{} {} [{}]\n",
                        LUAU_INSN_A(insn),
                        LUAU_INSN_B(insn),
                        LUAU_INSN_C(insn) as i32 - 1,
                        code[1]
                    ),
                );
                2
            }
            LuauOpcode::LOP_FORNPREP => {
                formatAppend(
                    result,
                    format_args!("FORNPREP R{} L{}\n", LUAU_INSN_A(insn), target_label),
                );
                1
            }
            LuauOpcode::LOP_FORNLOOP => {
                formatAppend(
                    result,
                    format_args!("FORNLOOP R{} L{}\n", LUAU_INSN_A(insn), target_label),
                );
                1
            }
            LuauOpcode::LOP_FORGPREP => {
                formatAppend(
                    result,
                    format_args!("FORGPREP R{} L{}\n", LUAU_INSN_A(insn), target_label),
                );
                1
            }
            LuauOpcode::LOP_FORGLOOP => {
                formatAppend(
                    result,
                    format_args!(
                        "FORGLOOP R{} L{} {}{}\n",
                        LUAU_INSN_A(insn),
                        target_label,
                        code[1] as u8,
                        if (code[1] as i32) < 0 { " [inext]" } else { "" }
                    ),
                );
                2
            }
            LuauOpcode::LOP_FORGPREP_INEXT => {
                formatAppend(
                    result,
                    format_args!("FORGPREP_INEXT R{} L{}\n", LUAU_INSN_A(insn), target_label),
                );
                1
            }
            LuauOpcode::LOP_FORGPREP_NEXT => {
                formatAppend(
                    result,
                    format_args!("FORGPREP_NEXT R{} L{}\n", LUAU_INSN_A(insn), target_label),
                );
                1
            }
            LuauOpcode::LOP_GETVARARGS => {
                formatAppend(
                    result,
                    format_args!(
                        "GETVARARGS R{} {}\n",
                        LUAU_INSN_A(insn),
                        LUAU_INSN_B(insn) as i32 - 1
                    ),
                );
                1
            }
            LuauOpcode::LOP_DUPCLOSURE => {
                formatAppend(
                    result,
                    format_args!("DUPCLOSURE R{} K{} [", LUAU_INSN_A(insn), LUAU_INSN_D(insn)),
                );
                self.dump_constant(result, LUAU_INSN_D(insn) as i32, false);
                result.push_str("]\n");
                1
            }
            LuauOpcode::LOP_BREAK => {
                formatAppend(result, format_args!("BREAK\n"));
                1
            }
            LuauOpcode::LOP_JUMPBACK => {
                formatAppend(result, format_args!("JUMPBACK L{}\n", target_label));
                1
            }
            LuauOpcode::LOP_LOADKX => {
                formatAppend(
                    result,
                    format_args!("LOADKX R{} K{} [", LUAU_INSN_A(insn), code[1]),
                );
                self.dump_constant(result, code[1] as i32, false);
                result.push_str("]\n");
                2
            }
            LuauOpcode::LOP_JUMPX => {
                formatAppend(result, format_args!("JUMPX L{}\n", target_label));
                1
            }
            LuauOpcode::LOP_FASTCALL => {
                formatAppend(
                    result,
                    format_args!("FASTCALL {} L{}\n", LUAU_INSN_A(insn), target_label),
                );
                1
            }
            LuauOpcode::LOP_FASTCALL1 => {
                formatAppend(
                    result,
                    format_args!(
                        "FASTCALL1 {} R{} L{}\n",
                        LUAU_INSN_A(insn),
                        LUAU_INSN_B(insn),
                        target_label
                    ),
                );
                1
            }
            LuauOpcode::LOP_FASTCALL2 => {
                formatAppend(
                    result,
                    format_args!(
                        "FASTCALL2 {} R{} R{} L{}\n",
                        LUAU_INSN_A(insn),
                        LUAU_INSN_B(insn),
                        code[1],
                        target_label
                    ),
                );
                2
            }
            LuauOpcode::LOP_FASTCALL2K => {
                formatAppend(
                    result,
                    format_args!(
                        "FASTCALL2K {} R{} K{} L{} [",
                        LUAU_INSN_A(insn),
                        LUAU_INSN_B(insn),
                        code[1],
                        target_label
                    ),
                );
                self.dump_constant(result, code[1] as i32, false);
                result.push_str("]\n");
                2
            }
            LuauOpcode::LOP_FASTCALL3 => {
                formatAppend(
                    result,
                    format_args!(
                        "FASTCALL3 {} R{} R{} R{} L{}\n",
                        LUAU_INSN_A(insn),
                        LUAU_INSN_B(insn),
                        code[1] & 0xff,
                        (code[1] >> 8) & 0xff,
                        target_label
                    ),
                );
                2
            }
            LuauOpcode::LOP_COVERAGE => {
                formatAppend(result, format_args!("COVERAGE\n"));
                1
            }
            LuauOpcode::LOP_CAPTURE => {
                let a = LUAU_INSN_A(insn);
                // LCT_VAL = 0, LCT_REF = 1, LCT_UPVAL = 2 (the port had VAL/UPVAL swapped).
                formatAppend(
                    result,
                    format_args!(
                        "CAPTURE {} {}{}\n",
                        if a == 2 {
                            "UPVAL"
                        } else if a == 1 {
                            "REF"
                        } else if a == 0 {
                            "VAL"
                        } else {
                            ""
                        },
                        if a == 2 { 'U' } else { 'R' },
                        LUAU_INSN_B(insn)
                    ),
                );
                1
            }
            LuauOpcode::LOP_JUMPXEQKNIL => {
                formatAppend(
                    result,
                    format_args!(
                        "JUMPXEQKNIL R{} L{}{}\n",
                        LUAU_INSN_A(insn),
                        target_label,
                        if (code[1] >> 31) != 0 { " NOT" } else { "" }
                    ),
                );
                2
            }
            LuauOpcode::LOP_JUMPXEQKB => {
                formatAppend(
                    result,
                    format_args!(
                        "JUMPXEQKB R{} {} L{}{}\n",
                        LUAU_INSN_A(insn),
                        code[1] & 1,
                        target_label,
                        if (code[1] >> 31) != 0 { " NOT" } else { "" }
                    ),
                );
                2
            }
            LuauOpcode::LOP_JUMPXEQKN => {
                formatAppend(
                    result,
                    format_args!(
                        "JUMPXEQKN R{} K{} L{}{} [",
                        LUAU_INSN_A(insn),
                        code[1] & 0xffffff,
                        target_label,
                        if (code[1] >> 31) != 0 { " NOT" } else { "" }
                    ),
                );
                self.dump_constant(result, (code[1] & 0xffffff) as i32, false);
                result.push_str("]\n");
                2
            }
            LuauOpcode::LOP_JUMPXEQKS => {
                formatAppend(
                    result,
                    format_args!(
                        "JUMPXEQKS R{} K{} L{}{} [",
                        LUAU_INSN_A(insn),
                        code[1] & 0xffffff,
                        target_label,
                        if (code[1] >> 31) != 0 { " NOT" } else { "" }
                    ),
                );
                self.dump_constant(result, (code[1] & 0xffffff) as i32, false);
                result.push_str("]\n");
                2
            }
            LuauOpcode::LOP_GETUDATAKS => {
                formatAppend(
                    result,
                    format_args!(
                        "GETUDATAKS R{} R{} K{} [",
                        LUAU_INSN_A(insn),
                        LUAU_INSN_B(insn),
                        LUAU_INSN_AUX_KV16(code[1])
                    ),
                );
                self.dump_constant(result, LUAU_INSN_AUX_KV16(code[1]) as i32, false);
                result.push_str("]\n");
                2
            }
            LuauOpcode::LOP_SETUDATAKS => {
                formatAppend(
                    result,
                    format_args!(
                        "SETUDATAKS R{} R{} K{} [",
                        LUAU_INSN_A(insn),
                        LUAU_INSN_B(insn),
                        LUAU_INSN_AUX_KV16(code[1])
                    ),
                );
                self.dump_constant(result, LUAU_INSN_AUX_KV16(code[1]) as i32, false);
                result.push_str("]\n");
                2
            }
            LuauOpcode::LOP_NAMECALLUDATA => {
                formatAppend(
                    result,
                    format_args!(
                        "NAMECALLUDATA R{} R{} K{} [",
                        LUAU_INSN_A(insn),
                        LUAU_INSN_B(insn),
                        LUAU_INSN_AUX_KV16(code[1])
                    ),
                );
                self.dump_constant(result, LUAU_INSN_AUX_KV16(code[1]) as i32, false);
                result.push_str("]\n");
                2
            }
            LuauOpcode::LOP_NEWCLASSMEMBER => {
                formatAppend(
                    result,
                    format_args!(
                        "NEWCLASSMEMBER R{} R{} [",
                        LUAU_INSN_A(insn),
                        LUAU_INSN_C(insn)
                    ),
                );
                self.dump_constant(result, code[1] as i32, false);
                result.push_str("]\n");
                2
            }
            LuauOpcode::LOP_CMPPROTO => {
                formatAppend(
                    result,
                    format_args!(
                        "CMPPROTO R{} #{} L{}\n",
                        LUAU_INSN_A(insn),
                        code[1],
                        target_label
                    ),
                );
                2
            }
            _ => {
                LUAU_ASSERT!(false);
                1
            }
        }
    }
}
