use crate::macros::clvalue::clvalue;
use crate::macros::lua_callinfo_opyield::LUA_CALLINFO_OPYIELD;
use crate::macros::setobj_2_s::setobj_2_s;
use crate::macros::ttisnil::ttisnil;
use crate::macros::vm_reg::VM_REG;
use crate::type_aliases::instruction::Instruction;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::stk_id::StkId;
use luaur_common::enums::luau_opcode::LuauOpcode;
use luaur_common::macros::luau_assert::LUAU_ASSERT;
use luaur_common::macros::luau_insn_a::LUAU_INSN_A;
use luaur_common::macros::luau_insn_d::LUAU_INSN_D;
use luaur_common::macros::luau_insn_op::LUAU_INSN_OP;
use luaur_common::macros::luau_unreachable::LUAU_UNREACHABLE;

#[allow(non_snake_case)]
pub unsafe fn luau_finishop(L: *mut lua_State) {
    let ci = &mut *(*L).ci;
    ci.flags &= !(LUA_CALLINFO_OPYIELD as u32);

    let cl = clvalue!((*(*L).ci).func);
    let _base = (*L).base;

    let pc_ptr = ci.savedpc;
    let insn: Instruction = *pc_ptr.offset(-1); // the interrupted instruction

    let mut pc = pc_ptr;
    match LUAU_INSN_OP(insn) {
        op if op == LuauOpcode::LOP_FORGLOOP as u32 => {
            let ra: StkId = VM_REG!(LUAU_INSN_A(insn), L, (*L).base);

            // copy first variable back into the iteration index
            setobj_2_s!(L, ra.add(2), ra.add(3));

            // note that we need to increment pc by 1 to exit the loop since we need to skip over aux
            if ttisnil!(ra.add(3)) {
                pc = pc.offset(1);
            } else {
                pc = pc.offset(LUAU_INSN_D(insn) as isize);
            }

            let lcl =
                core::ptr::addr_of!((*cl).inner.l).cast::<crate::records::closure::LClosure>();
            let proto = (*lcl).p;
            LUAU_ASSERT!(
                (pc as usize).wrapping_sub((*proto).code as usize)
                    / core::mem::size_of::<Instruction>()
                    < ((*proto).sizecode as usize)
            );
        }
        _ => {
            LUAU_ASSERT!(false);
            LUAU_UNREACHABLE!();
        }
    }

    (*(*L).ci).savedpc = pc;
}
