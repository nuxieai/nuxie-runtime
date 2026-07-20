//! Node: `cxx:Function:Luau.VM:VM/src/lvmexecute.cpp:3716:luau_execute`
//! Source: `VM/src/lvmexecute.cpp:228-3722` (hand-port in progress; design
//! card at `translation/design-cards/lvmexecute.md`)
//!
//! ALL 89 `VM_CASE` opcode arms live HERE as match arms (the per-arm
//! `vm_case_lvmexecute*.rs` node files are macro-extraction artifacts and
//! become doc-pointers as their arms land). Control-flow mapping:
//! `VM_NEXT()` -> `continue 'dispatch`; `VM_CONTINUE(op)` -> set
//! `continue_op` + `continue 'dispatch`; `goto exit` -> `return`.

use crate::enums::lua_type::lua_Type;
use crate::functions::lua_d_call::lua_d_call;
use crate::functions::lua_d_check_cstack::luaD_checkCstack;
use crate::functions::lua_d_performcally::lua_d_performcally;
use crate::functions::lua_f_close::lua_f_close;
use crate::functions::lua_f_findupval::lua_f_findupval;
use crate::functions::lua_f_new_lclosure::lua_f_new_lclosure;
use crate::functions::lua_f_recordhit::luaF_recordhit;
use crate::functions::lua_g_methoderror::luaG_methoderror;
use crate::functions::lua_g_missingmembererror::luaG_missingmembererror;
use crate::functions::lua_g_typeerror_l::luaG_typeerrorL;
use crate::functions::lua_h_clone::lua_h_clone;
use crate::functions::lua_h_getn::lua_h_getn;
use crate::functions::lua_h_getstr::luaH_getstr;
use crate::functions::lua_h_new::lua_h_new;
use crate::functions::lua_h_resizearray::lua_h_resizearray;
use crate::functions::lua_h_setstr::lua_h_setstr as luaH_setstr;
use crate::functions::lua_o_rawequal_obj::luaO_rawequalObj;
use crate::functions::lua_r_addclassmember::lua_r_addclassmember;
use crate::functions::lua_t_gettmbyobj::lua_t_gettmbyobj;
use crate::functions::lua_v_call_tm::lua_v_call_tm;
use crate::functions::lua_v_concat::lua_v_concat;
use crate::functions::lua_v_doarithimpl::lua_v_doarithimpl;
use crate::functions::lua_v_dolen::lua_v_dolen;
use crate::functions::lua_v_equalval::lua_v_equalval;
use crate::functions::lua_v_getimport::lua_v_getimport;
use crate::functions::lua_v_gettable::lua_v_gettable;
use crate::functions::lua_v_lessequal::lua_v_lessequal;
use crate::functions::lua_v_lessthan::lua_v_lessthan;
use crate::functions::lua_v_prepare_forn::lua_v_prepare_forn;
use crate::functions::lua_v_settable::lua_v_settable;
use crate::functions::lua_v_strcmp::lua_v_strcmp;
use crate::functions::lua_v_tryfunc_tm::lua_v_tryfunc_tm;
use crate::functions::luai_numidiv::luai_numidiv;
use crate::functions::luai_nummod::luai_nummod;
use crate::functions::luai_veceq::luai_veceq;
use crate::functions::luau_callhook::luau_callhook;
use crate::functions::luau_setupcci::luau_setupcci;
use crate::functions::luau_skipstep::luau_skipstep;
use crate::macros::bvalue::bvalue;
use crate::macros::classvalue::classvalue;
use crate::macros::clvalue::clvalue;
use crate::macros::fastnotm::fastnotm;
use crate::macros::fasttm::fasttm;
use crate::macros::gcvalue::gcvalue;
use crate::macros::getnodekey::getnodekey;
use crate::macros::getstr::getstr;
use crate::macros::gkey::{gkey, gval};
use crate::macros::gnext::gnext;
use crate::macros::gval_2_slot::gval2slot;
use crate::macros::hvalue::hvalue;
use crate::macros::incr_ci::incr_ci;
use crate::macros::is_lua::isLua;
use crate::macros::l_isfalse::l_isfalse;
use crate::macros::lightuserdatatag::lightuserdatatag;
use crate::macros::lu_tag_iterator::LU_TAG_ITERATOR;
use crate::macros::lua_c_barrier::luaC_barrier;
use crate::macros::lua_c_barrierfast::luaC_barrierfast;
use crate::macros::lua_c_barriert::luaC_barriert;
use crate::macros::lua_c_check_gc::luaC_checkGC;
use crate::macros::lua_callinfo_native::LUA_CALLINFO_NATIVE;
use crate::macros::lua_callinfo_return::LUA_CALLINFO_RETURN;
use crate::macros::lua_d_checkstack::luaD_checkstack;
use crate::macros::lua_d_checkstackfornewci::luaD_checkstackfornewci;
use crate::macros::lua_multret::LUA_MULTRET;
use crate::macros::lua_o_nilobject::luaO_nilobject;
use crate::macros::lua_r_lookupmemberatoffset::luaR_lookupmemberatoffset;
use crate::macros::luai_maxccalls::LUAI_MAXCCALLS;
use crate::macros::luau_f_table::luauF_table;
use crate::macros::lvalue::lvalue;
use crate::macros::nvalue::nvalue;
use crate::macros::objectvalue::objectvalue;
use crate::macros::pvalue::pvalue;
use crate::macros::setbvalue::setbvalue;
use crate::macros::setclvalue::setclvalue;
use crate::macros::sethvalue::sethvalue;
use crate::macros::setnilvalue::setnilvalue;
use crate::macros::setnvalue::setnvalue;
use crate::macros::setobj::setobj;
use crate::macros::setobj_2_s::setobj_2_s;
use crate::macros::setobj_2_t::setobj2t;
use crate::macros::setpvalue::setpvalue;
use crate::macros::setupvalue::setupvalue;
use crate::macros::setvvalue::setvvalue;
use crate::macros::sizenode::sizenode;
use crate::macros::tsvalue::tsvalue;
use crate::macros::ttisboolean::ttisboolean;
use crate::macros::ttisfunction::ttisfunction;
use crate::macros::ttisnil::ttisnil;
use crate::macros::ttisnumber::ttisnumber;
use crate::macros::ttisobject::ttisobject;
use crate::macros::ttisstring::ttisstring;
use crate::macros::ttistable::ttistable;
use crate::macros::ttisupval::ttisupval;
use crate::macros::ttisuserdata::ttisuserdata;
use crate::macros::ttisvector::ttisvector;
use crate::macros::ttype::ttype;
use crate::macros::upvalue::upvalue;
use crate::macros::uvalue::uvalue;
use crate::macros::vm_interrupt::VM_INTERRUPT;
use crate::macros::vm_kv::VM_KV;
use crate::macros::vm_patch_aux::VM_PATCH_AUX;
use crate::macros::vm_patch_aux_slot::VM_PATCH_AUX_SLOT;
use crate::macros::vm_patch_c::VM_PATCH_C;
use crate::macros::vm_patch_e::VM_PATCH_E;
use crate::macros::vm_patch_op::VM_PATCH_OP;
use crate::macros::vm_protect::vm_protect;
use crate::macros::vm_reg::VM_REG;
use crate::macros::vm_uv::VM_UV;
use crate::macros::vvalue::vvalue;
use crate::records::closure::Closure;
use crate::records::lua_state::lua_State;
use crate::records::luau_class::LuauClass;
use crate::records::luau_object::LuauObject;
use crate::records::up_val::UpVal;
use crate::type_aliases::instruction::Instruction;
use crate::type_aliases::stk_id::StkId;
use crate::type_aliases::t_value::TValue;
use crate::type_aliases::tms::TMS;
use luaur_common::enums::luau_capture_type::LuauCaptureType;
use luaur_common::enums::luau_opcode::LuauOpcode;
use luaur_common::macros::luau_assert::LUAU_ASSERT;
use luaur_common::macros::luau_insn_a::LUAU_INSN_A;
use luaur_common::macros::luau_insn_aux_a::LUAU_INSN_AUX_A;
use luaur_common::macros::luau_insn_aux_b::LUAU_INSN_AUX_B;
use luaur_common::macros::luau_insn_aux_kb::LUAU_INSN_AUX_KB;
use luaur_common::macros::luau_insn_aux_kv::LUAU_INSN_AUX_KV;
use luaur_common::macros::luau_insn_aux_kv_16::LUAU_INSN_AUX_KV16;
use luaur_common::macros::luau_insn_aux_not::LUAU_INSN_AUX_NOT;
use luaur_common::macros::luau_insn_aux_slot::LUAU_INSN_AUX_SLOT;
use luaur_common::macros::luau_insn_b::LUAU_INSN_B;
use luaur_common::macros::luau_insn_c::LUAU_INSN_C;
use luaur_common::macros::luau_insn_d::LUAU_INSN_D;
use luaur_common::macros::luau_insn_e::LUAU_INSN_E;
use luaur_common::macros::luau_insn_fbslot_sealed::LUAU_INSN_FBSLOT_SEALED;
use luaur_common::macros::luau_insn_op::LUAU_INSN_OP;

/// C++ `void luau_execute(lua_State* L)` (lvmexecute.cpp:3716) — dispatches
/// to the `template<bool SingleStep>` monomorphs.
#[allow(non_snake_case)]
pub unsafe fn luau_execute(L: *mut lua_State) {
    if (*L).singlestep {
        luau_execute_impl::<true>(L)
    } else {
        luau_execute_impl::<false>(L)
    }
}

/// C++ `template<bool SingleStep> static void luau_execute(lua_State* L)`
/// (lvmexecute.cpp:228). The computed-goto dispatch table becomes the match
/// below (both blindly index by the opcode byte).
#[allow(non_snake_case, unused_assignments, unreachable_code, unused_variables)]
unsafe fn luau_execute_impl<const SINGLE_STEP: bool>(L: *mut lua_State) {
    // the critical interpreter state, stored in locals for performance
    let mut cl: *mut Closure;
    let mut base: StkId;
    let mut k: *mut TValue;
    let mut pc: *const Instruction;

    LUAU_ASSERT!(isLua!((*L).ci));
    LUAU_ASSERT!((*L).isactive);
    // C++ also asserts !isblack(obj2gco(L)) — active threads never turn black.

    // VM_HAS_NATIVE entry: execution may continue in native code.
    if ((*(*L).ci).flags as i32 & LUA_CALLINFO_NATIVE) != 0 && !SINGLE_STEP {
        let native_cl = clvalue!((*(*L).ci).func);
        let native_lcl =
            core::ptr::addr_of!((*native_cl).inner.l).cast::<crate::records::closure::LClosure>();
        let p = (*native_lcl).p;
        LUAU_ASSERT!(!(*p).execdata.is_null());
        if let Some(enter) = (*(*L).global).ecb.enter {
            if enter(L, p) == 0 {
                return;
            }
        }
    }

    // C++ `reentry:` label (goto target from NATIVECALL/RETURN native paths).
    'reentry: loop {
        LUAU_ASSERT!(isLua!((*L).ci));

        pc = (*(*L).ci).savedpc;
        cl = clvalue!((*(*L).ci).func);
        base = (*L).base;
        k = {
            let l = &(*cl).inner.l;
            (*l.p).k
        };

        // C++ `VM_CONTINUE(op)` re-dispatches WITHOUT refetching `*pc`.
        let mut continue_op: Option<u8> = None;

        // C++ `dispatch:` label; `VM_NEXT()` == `continue 'dispatch`.
        'dispatch: loop {
            // Note: in C++ this assert block is bypassed by computed goto
            // except in single-step mode; asserts only.
            if SINGLE_STEP && continue_op.is_none() {
                if (*(*L).global).cb.debugstep.is_some() && !luau_skipstep(LUAU_INSN_OP!(*pc) as u8)
                {
                    let debugstep = (*(*L).global).cb.debugstep;
                    vm_protect!(L, pc, base, {
                        luau_callhook(L, debugstep, core::ptr::null_mut());
                    });
                    // allow debugstep hook to put thread into error/yield state
                    if (*L).status != 0 {
                        return; // goto exit
                    }
                }
            }

            let op: u8 = match continue_op.take() {
                Some(op) => op,
                None => LUAU_INSN_OP!(*pc) as u8,
            };

            // The C++ jump table indexes the opcode byte blindly; so do we.
            match core::mem::transmute::<u8, LuauOpcode>(op) {
                LuauOpcode::LOP_NOP => {
                    // lvmexecute.cpp:319
                    let insn = *pc;
                    pc = pc.add(1);
                    LUAU_ASSERT!(insn == 0);
                    continue 'dispatch;
                }

                LuauOpcode::LOP_LOADNIL => {
                    // lvmexecute.cpp:326
                    let insn = *pc;
                    pc = pc.add(1);
                    let ra = VM_REG!(LUAU_INSN_A!(insn), L, base);

                    setnilvalue!(ra as *mut TValue);
                    continue 'dispatch;
                }

                LuauOpcode::LOP_LOADB => {
                    // lvmexecute.cpp:335
                    let insn = *pc;
                    pc = pc.add(1);
                    let ra = VM_REG!(LUAU_INSN_A!(insn), L, base);

                    setbvalue!(ra as *mut TValue, LUAU_INSN_B!(insn) as i32);

                    pc = pc.add(LUAU_INSN_C!(insn) as usize);
                    let p = {
                        let l = &(*cl).inner.l;
                        l.p
                    };
                    LUAU_ASSERT!((pc.offset_from((*p).code) as u32) < (*p).sizecode as u32);
                    continue 'dispatch;
                }

                LuauOpcode::LOP_LOADN => {
                    // lvmexecute.cpp:347
                    let insn = *pc;
                    pc = pc.add(1);
                    let ra = VM_REG!(LUAU_INSN_A!(insn), L, base);

                    setnvalue!(ra as *mut TValue, LUAU_INSN_D!(insn) as f64);
                    continue 'dispatch;
                }

                LuauOpcode::LOP_LOADK => {
                    // lvmexecute.cpp:356
                    let insn = *pc;
                    pc = pc.add(1);
                    let ra = VM_REG!(LUAU_INSN_A!(insn), L, base);
                    let kv = VM_KV!(LUAU_INSN_D!(insn), cl, k);

                    setobj_2_s!(L, ra as *mut TValue, kv as *const TValue);
                    continue 'dispatch;
                }

                LuauOpcode::LOP_MOVE => {
                    // lvmexecute.cpp:366
                    let insn = *pc;
                    pc = pc.add(1);
                    let ra = VM_REG!(LUAU_INSN_A!(insn), L, base);
                    let rb = VM_REG!(LUAU_INSN_B!(insn), L, base);

                    setobj_2_s!(L, ra as *mut TValue, rb as *const TValue);
                    continue 'dispatch;
                }

                LuauOpcode::LOP_GETGLOBAL => {
                    // lvmexecute.cpp:376
                    let insn = *pc;
                    pc = pc.add(1);
                    let ra = VM_REG!(LUAU_INSN_A!(insn), L, base);
                    let aux: u32 = *pc;
                    pc = pc.add(1);
                    let kv = VM_KV!(aux, cl, k);
                    LUAU_ASSERT!(ttisstring!(kv as *const TValue));

                    // fast-path: value is in expected slot
                    let h = (*cl).env;
                    let slot = (LUAU_INSN_C!(insn) as i32) & (*h).nodemask8 as i32;
                    let n = (*h).node.add(slot as usize);

                    if ttisstring!(gkey!(n) as *const TValue)
                        && tsvalue!(gkey!(n) as *const TValue) == tsvalue!(kv as *const TValue)
                        && !ttisnil!(gval!(n))
                    {
                        setobj_2_s!(L, ra as *mut TValue, gval!(n));
                        continue 'dispatch;
                    } else {
                        // slow-path, may invoke Lua calls via __index metamethod
                        let mut g: TValue = core::mem::zeroed();
                        sethvalue!(L, &mut g as *mut TValue, h);
                        (*L).cachedslot = slot;
                        vm_protect!(L, pc, base, {
                            lua_v_gettable(L, &g as *const TValue, kv as *mut TValue, ra);
                        });
                        // save cachedslot to accelerate future lookups; patches
                        // currently executing instruction since pc-2 rolls back two pc++
                        VM_PATCH_C(pc.sub(2), (*L).cachedslot);
                        continue 'dispatch;
                    }
                }

                LuauOpcode::LOP_SETGLOBAL => {
                    // lvmexecute.cpp:407
                    let insn = *pc;
                    pc = pc.add(1);
                    let ra = VM_REG!(LUAU_INSN_A!(insn), L, base);
                    let aux: u32 = *pc;
                    pc = pc.add(1);
                    let kv = VM_KV!(aux, cl, k);
                    LUAU_ASSERT!(ttisstring!(kv as *const TValue));

                    // fast-path: value is in expected slot
                    let h = (*cl).env;
                    let slot = (LUAU_INSN_C!(insn) as i32) & (*h).nodemask8 as i32;
                    let n = (*h).node.add(slot as usize);

                    if ttisstring!(gkey!(n) as *const TValue)
                        && tsvalue!(gkey!(n) as *const TValue) == tsvalue!(kv as *const TValue)
                        && !ttisnil!(gval!(n))
                        && (*h).readonly == 0
                    {
                        setobj2t!(L, gval!(n), ra as *const TValue);
                        luaC_barriert!(L, h, ra as *const TValue);
                        continue 'dispatch;
                    } else {
                        // slow-path, may invoke Lua calls via __newindex metamethod
                        let mut g: TValue = core::mem::zeroed();
                        sethvalue!(L, &mut g as *mut TValue, h);
                        (*L).cachedslot = slot;
                        vm_protect!(L, pc, base, {
                            lua_v_settable(L, &g as *const TValue, kv as *mut TValue, ra);
                        });
                        VM_PATCH_C(pc.sub(2), (*L).cachedslot);
                        continue 'dispatch;
                    }
                }

                LuauOpcode::LOP_GETUPVAL => {
                    // lvmexecute.cpp:439
                    let insn = *pc;
                    pc = pc.add(1);
                    let ra = VM_REG!(LUAU_INSN_A!(insn), L, base);
                    let ur = VM_UV!(LUAU_INSN_B!(insn), cl);
                    let v: *mut TValue = if ttisupval!(ur as *const TValue) {
                        (*upvalue!(ur as *mut TValue)).v
                    } else {
                        ur as *mut TValue
                    };

                    setobj_2_s!(L, ra as *mut TValue, v as *const TValue);
                    continue 'dispatch;
                }

                LuauOpcode::LOP_SETUPVAL => {
                    // lvmexecute.cpp:450
                    let insn = *pc;
                    pc = pc.add(1);
                    let ra = VM_REG!(LUAU_INSN_A!(insn), L, base);
                    let ur = VM_UV!(LUAU_INSN_B!(insn), cl);
                    let uv = &mut **upvalue!(ur as *mut TValue) as *mut UpVal;

                    setobj!(L, (*uv).v, ra as *const TValue);
                    luaC_barrier!(L, uv, ra as *const TValue);
                    continue 'dispatch;
                }

                LuauOpcode::LOP_CLOSEUPVALS => {
                    // lvmexecute.cpp:462
                    let insn = *pc;
                    pc = pc.add(1);
                    let ra = VM_REG!(LUAU_INSN_A!(insn), L, base);

                    if !(*L).openupval.is_null() && (*(*L).openupval).v >= ra as *mut TValue {
                        lua_f_close(L, ra as *mut TValue);
                    }
                    continue 'dispatch;
                }

                LuauOpcode::LOP_GETIMPORT => {
                    // lvmexecute.cpp:472
                    let insn = *pc;
                    pc = pc.add(1);
                    let ra = VM_REG!(LUAU_INSN_A!(insn), L, base);
                    let kv = VM_KV!(LUAU_INSN_D!(insn), cl, k);

                    // fast-path: import resolution was successful and closure
                    // environment is "safe" for import
                    if !ttisnil!(kv as *const TValue) && (*(*cl).env).safeenv != 0 {
                        setobj_2_s!(L, ra as *mut TValue, kv as *const TValue);
                        pc = pc.add(1); // skip over AUX
                        continue 'dispatch;
                    } else {
                        let aux: u32 = *pc;
                        pc = pc.add(1);

                        vm_protect!(L, pc, base, {
                            lua_v_getimport(
                                L,
                                (*cl).env,
                                k,
                                ra,
                                aux,
                                /* propagatenil= */ false,
                            );
                        });
                        continue 'dispatch;
                    }
                }
                LuauOpcode::LOP_GETTABLEKS => {
                    // lvmexecute.cpp:494
                    let insn = *pc;
                    pc = pc.add(1);
                    let ra = VM_REG!(LUAU_INSN_A!(insn), L, base) as *mut TValue;
                    let rb = VM_REG!(LUAU_INSN_B!(insn), L, base) as *mut TValue;
                    let aux: u32 = *pc;
                    pc = pc.add(1);
                    let kv = VM_KV!(aux, cl, k) as *mut TValue;
                    LUAU_ASSERT!(ttisstring!(kv as *const TValue));

                    // fast-path: built-in table
                    if ttistable!(rb as *const TValue) {
                        let h = hvalue!(rb as *const TValue);

                        let slot = (LUAU_INSN_C!(insn) as i32) & (*h).nodemask8 as i32;
                        let n = (*h).node.add(slot as usize);

                        // fast-path: value is in expected slot
                        if ttisstring!(gkey!(n) as *const TValue)
                            && tsvalue!(gkey!(n) as *const TValue) == tsvalue!(kv as *const TValue)
                            && !ttisnil!(gval!(n))
                        {
                            setobj_2_s!(L, ra, gval!(n));
                            continue 'dispatch;
                        } else if (*h).metatable.is_null() {
                            // fast-path: value is not in expected slot, but the table
                            // lookup doesn't involve metatable
                            let res = luaH_getstr(
                                h,
                                tsvalue!(kv as *const TValue)
                                    as *mut crate::records::t_string::TString,
                            );

                            if res != luaO_nilobject {
                                let cachedslot = gval2slot!(h, res);
                                // save cachedslot to accelerate future lookups; patches
                                // currently executing instruction since pc-2 rolls back two pc++
                                VM_PATCH_C(pc.sub(2), cachedslot);
                            }

                            setobj_2_s!(L, ra, res);
                            continue 'dispatch;
                        } else {
                            // slow-path, may invoke Lua calls via __index metamethod
                            (*L).cachedslot = slot;
                            vm_protect!(L, pc, base, {
                                lua_v_gettable(L, rb as *const TValue, kv, ra);
                            });
                            VM_PATCH_C(pc.sub(2), (*L).cachedslot);
                            continue 'dispatch;
                        }
                    } else {
                        // fast-path: registered direct field handler
                        if luaur_common::FFlag::LuauDirectFieldGet.get()
                            && ttisuserdata!(rb as *const TValue)
                        {
                            let dispatch_t = {
                                let u = uvalue!(rb as *const TValue);
                                (*(*L).global).udatadirectfields[u.tag as usize]
                            };
                            if !dispatch_t.is_null() {
                                let slot =
                                    (LUAU_INSN_C!(insn) as i32) & (*dispatch_t).nodemask8 as i32;
                                let n = (*dispatch_t).node.add(slot as usize);

                                if ttisstring!(gkey!(n) as *const TValue)
                                    && tsvalue!(gkey!(n) as *const TValue)
                                        == tsvalue!(kv as *const TValue)
                                    && !ttisnil!(gval!(n))
                                {
                                    let f: unsafe extern "C" fn(
                                        *mut core::ffi::c_void,
                                        *mut core::ffi::c_void,
                                    ) = core::mem::transmute(pvalue!(gval!(n) as *const TValue));
                                    let u = uvalue!(rb as *const TValue);
                                    f(
                                        u.data.as_ptr() as *mut core::ffi::c_void,
                                        ra as *mut core::ffi::c_void,
                                    );
                                    continue 'dispatch;
                                }

                                let fptr = luaH_getstr(
                                    dispatch_t,
                                    tsvalue!(kv as *const TValue)
                                        as *mut crate::records::t_string::TString,
                                );
                                if !ttisnil!(fptr) {
                                    // cache slot for future lookups
                                    VM_PATCH_C(pc.sub(2), gval2slot!(dispatch_t, fptr));
                                    let f: unsafe extern "C" fn(
                                        *mut core::ffi::c_void,
                                        *mut core::ffi::c_void,
                                    ) = core::mem::transmute(pvalue!(fptr));
                                    let u = uvalue!(rb as *const TValue);
                                    f(
                                        u.data.as_ptr() as *mut core::ffi::c_void,
                                        ra as *mut core::ffi::c_void,
                                    );
                                    continue 'dispatch;
                                }
                            }

                            // fall through to slow path
                        }

                        // fast-path: user data with C __index TM
                        let mut fn_tm: *const TValue = core::ptr::null();
                        if ttisuserdata!(rb as *const TValue)
                            && {
                                fn_tm = fasttm(
                                    L,
                                    (*uvalue!(rb as *const TValue)).metatable,
                                    TMS::TM_INDEX as i32,
                                );
                                !fn_tm.is_null()
                            }
                            && ttisfunction!(fn_tm)
                            && (*clvalue!(fn_tm)).isC != 0
                        {
                            // note: it's safe to push arguments past top for
                            // complicated reasons (see top of the file)
                            LUAU_ASSERT!((*L).top.add(3) < (*L).stack.add((*L).stacksize as usize));
                            let top = (*L).top;
                            setobj_2_s!(L, top.add(0), fn_tm);
                            setobj_2_s!(L, top.add(1), rb as *const TValue);
                            setobj_2_s!(L, top.add(2), kv as *const TValue);
                            (*L).top = top.add(3);

                            (*L).cachedslot = LUAU_INSN_C!(insn) as i32;
                            vm_protect!(L, pc, base, {
                                lua_v_call_tm(L, 2, LUAU_INSN_A!(insn) as i32);
                            });
                            VM_PATCH_C(pc.sub(2), (*L).cachedslot);
                            continue 'dispatch;
                        } else if ttisvector!(rb as *const TValue) {
                            // fast-path: quick case-insensitive comparison with "X"/"Y"/"Z"
                            let name = getstr(tsvalue!(kv as *const TValue));
                            let ic = ((*name.add(0)) as u8 | b' ') as i32 - b'x' as i32;
                            // (LUA_VECTOR_SIZE == 3 in this port; the C++ `== 4` branch
                            // maps 'w' -> 3 and is omitted)

                            if (ic as u32) < 3 && *name.add(1) == 0 {
                                let v = vvalue!(rb as *const TValue).as_ptr(); // silences ubsan when indexing v[]
                                setnvalue!(ra, *v.add(ic as usize) as f64);
                                continue 'dispatch;
                            }

                            let fn_tm = fasttm(
                                L,
                                (*(*L).global).mt[lua_Type::LUA_TVECTOR as usize],
                                TMS::TM_INDEX as i32,
                            );

                            if !fn_tm.is_null()
                                && ttisfunction!(fn_tm)
                                && (*clvalue!(fn_tm)).isC != 0
                            {
                                // note: it's safe to push arguments past top for
                                // complicated reasons (see top of the file)
                                LUAU_ASSERT!(
                                    (*L).top.add(3) < (*L).stack.add((*L).stacksize as usize)
                                );
                                let top = (*L).top;
                                setobj_2_s!(L, top.add(0), fn_tm);
                                setobj_2_s!(L, top.add(1), rb as *const TValue);
                                setobj_2_s!(L, top.add(2), kv as *const TValue);
                                (*L).top = top.add(3);

                                (*L).cachedslot = LUAU_INSN_C!(insn) as i32;
                                vm_protect!(L, pc, base, {
                                    lua_v_call_tm(L, 2, LUAU_INSN_A!(insn) as i32);
                                });
                                VM_PATCH_C(pc.sub(2), (*L).cachedslot);
                                continue 'dispatch;
                            }

                            // fall through to slow path
                        } else if luaur_common::FFlag::DebugLuauUserDefinedClassesRuntime.get()
                            && ttisobject!(rb as *const TValue)
                        {
                            // fast-path: the "hash line" is an offset that points
                            // to the class member with the same name.
                            let slot = LUAU_INSN_C!(insn) as u8;
                            let inst = &mut **objectvalue!(rb as *const TValue) as *mut LuauObject;
                            if (slot as i32) < (*(*inst).lclass).numberofallmembers
                                && tsvalue!(kv as *const TValue)
                                    == *(*(*inst).lclass).offsettomember.add(slot as usize)
                            {
                                setobj_2_s!(L, ra, luaR_lookupmemberatoffset!(inst, slot as i32));
                                continue 'dispatch;
                            } else {
                                // slow-er path: the slot mismatched so we fall back to
                                // looking up the offset from the string.
                                let offset = luaH_getstr(
                                    (*(*inst).lclass).memberstooffset,
                                    tsvalue!(kv as *const TValue)
                                        as *mut crate::records::t_string::TString,
                                );
                                if ttisnil!(offset) {
                                    luaG_missingmembererror(
                                        L,
                                        rb as *const TValue,
                                        kv as *const TValue,
                                    );
                                }
                                LUAU_ASSERT!(ttisnumber!(offset));
                                let offsetnum = nvalue!(offset) as i32;
                                setobj_2_s!(L, ra, luaR_lookupmemberatoffset!(inst, offsetnum));
                                VM_PATCH_C(pc.sub(2), offsetnum);
                                continue 'dispatch;
                            }
                        }

                        // fall through to slow path
                    }

                    // slow-path, may invoke Lua calls via __index metamethod
                    vm_protect!(L, pc, base, {
                        lua_v_gettable(L, rb as *const TValue, kv, ra);
                    });
                    continue 'dispatch;
                }
                LuauOpcode::LOP_SETTABLEKS => {
                    // lvmexecute.cpp:665
                    let insn = *pc;
                    pc = pc.add(1);
                    let ra = VM_REG!(LUAU_INSN_A!(insn), L, base) as *mut TValue;
                    let rb = VM_REG!(LUAU_INSN_B!(insn), L, base) as *mut TValue;
                    let aux: u32 = *pc;
                    pc = pc.add(1);
                    let kv = VM_KV!(aux, cl, k) as *mut TValue;
                    LUAU_ASSERT!(ttisstring!(kv as *const TValue));

                    // fast-path: built-in table
                    if ttistable!(rb as *const TValue) {
                        let h = hvalue!(rb as *const TValue);

                        let slot = (LUAU_INSN_C!(insn) as i32) & (*h).nodemask8 as i32;
                        let n = (*h).node.add(slot as usize);

                        // fast-path: value is in expected slot
                        if ttisstring!(gkey!(n) as *const TValue)
                            && tsvalue!(gkey!(n) as *const TValue) == tsvalue!(kv as *const TValue)
                            && !ttisnil!(gval!(n))
                            && (*h).readonly == 0
                        {
                            setobj2t!(L, gval!(n), ra as *const TValue);
                            luaC_barriert!(L, h, ra as *const TValue);
                            continue 'dispatch;
                        } else if fastnotm((*h).metatable, TMS::TM_NEWINDEX as i32)
                            && (*h).readonly == 0
                        {
                            (*(*L).ci).savedpc = pc; // VM_PROTECT_PC(): set may fail

                            let res = luaH_setstr(
                                L,
                                h,
                                tsvalue!(kv as *const TValue)
                                    as *mut crate::records::t_string::TString,
                            );
                            let cachedslot = gval2slot!(h, res as *const TValue);
                            // save cachedslot to accelerate future lookups; patches
                            // currently executing instruction since pc-2 rolls back two pc++
                            VM_PATCH_C(pc.sub(2), cachedslot);
                            setobj2t!(L, res, ra as *const TValue);
                            luaC_barriert!(L, h, ra as *const TValue);
                            continue 'dispatch;
                        } else {
                            // slow-path, may invoke Lua calls via __newindex metamethod
                            (*L).cachedslot = slot;
                            vm_protect!(L, pc, base, {
                                lua_v_settable(L, rb as *const TValue, kv, ra);
                            });
                            VM_PATCH_C(pc.sub(2), (*L).cachedslot);
                            continue 'dispatch;
                        }
                    } else {
                        // fast-path: user data with C __newindex TM
                        let mut fn_tm: *const TValue = core::ptr::null();
                        if ttisuserdata!(rb as *const TValue)
                            && {
                                fn_tm = fasttm(
                                    L,
                                    (*uvalue!(rb as *const TValue)).metatable,
                                    TMS::TM_NEWINDEX as i32,
                                );
                                !fn_tm.is_null()
                            }
                            && ttisfunction!(fn_tm)
                            && (*clvalue!(fn_tm)).isC != 0
                        {
                            // note: it's safe to push arguments past top for
                            // complicated reasons (see top of the file)
                            LUAU_ASSERT!((*L).top.add(4) < (*L).stack.add((*L).stacksize as usize));
                            let top = (*L).top;
                            setobj_2_s!(L, top.add(0), fn_tm);
                            setobj_2_s!(L, top.add(1), rb as *const TValue);
                            setobj_2_s!(L, top.add(2), kv as *const TValue);
                            setobj_2_s!(L, top.add(3), ra as *const TValue);
                            (*L).top = top.add(4);

                            (*L).cachedslot = LUAU_INSN_C!(insn) as i32;
                            vm_protect!(L, pc, base, {
                                lua_v_call_tm(L, 3, -1);
                            });
                            VM_PATCH_C(pc.sub(2), (*L).cachedslot);
                            continue 'dispatch;
                        } else {
                            // slow-path, may invoke Lua calls via __newindex metamethod
                            vm_protect!(L, pc, base, {
                                lua_v_settable(L, rb as *const TValue, kv, ra);
                            });
                            continue 'dispatch;
                        }
                    }
                }
                LuauOpcode::LOP_GETTABLE => {
                    // lvmexecute.cpp:741
                    let insn = *pc;
                    pc = pc.add(1);
                    let ra = VM_REG!(LUAU_INSN_A!(insn), L, base) as *mut TValue;
                    let rb = VM_REG!(LUAU_INSN_B!(insn), L, base) as *mut TValue;
                    let rc = VM_REG!(LUAU_INSN_C!(insn), L, base) as *mut TValue;

                    // fast-path: array lookup
                    if ttistable!(rb as *const TValue) && ttisnumber!(rc as *const TValue) {
                        let h = hvalue!(rb as *const TValue);

                        let indexd = nvalue!(rc as *const TValue);
                        let index = indexd as i32;

                        // index has to be an exact integer and in-bounds for the array portion
                        if ((index as u32).wrapping_sub(1)) < (*h).sizearray as u32
                            && (*h).metatable.is_null()
                            && index as f64 == indexd
                        {
                            setobj_2_s!(
                                L,
                                ra,
                                (*h).array.add((index - 1) as u32 as usize) as *const TValue
                            );
                            continue 'dispatch;
                        }

                        // fall through to slow path
                    }

                    // slow-path: handles out of bounds array lookups, non-integer
                    // numeric keys, non-array table lookup, __index MT calls
                    vm_protect!(L, pc, base, {
                        lua_v_gettable(L, rb as *const TValue, rc, ra);
                    });
                    continue 'dispatch;
                }
                LuauOpcode::LOP_SETTABLE => {
                    // lvmexecute.cpp:771
                    let insn = *pc;
                    pc = pc.add(1);
                    let ra = VM_REG!(LUAU_INSN_A!(insn), L, base) as *mut TValue;
                    let rb = VM_REG!(LUAU_INSN_B!(insn), L, base) as *mut TValue;
                    let rc = VM_REG!(LUAU_INSN_C!(insn), L, base) as *mut TValue;

                    // fast-path: array assign
                    if ttistable!(rb as *const TValue) && ttisnumber!(rc as *const TValue) {
                        let h = hvalue!(rb as *const TValue);

                        let indexd = nvalue!(rc as *const TValue);
                        let index = indexd as i32;

                        // index has to be an exact integer and in-bounds for the array portion
                        if ((index as u32).wrapping_sub(1)) < (*h).sizearray as u32
                            && (*h).metatable.is_null()
                            && (*h).readonly == 0
                            && index as f64 == indexd
                        {
                            setobj2t!(
                                L,
                                (*h).array.add((index - 1) as u32 as usize),
                                ra as *const TValue
                            );
                            luaC_barriert!(L, h, ra as *const TValue);
                            continue 'dispatch;
                        }

                        // fall through to slow path
                    }

                    // slow-path: handles out of bounds array assignments, non-integer
                    // numeric keys, non-array table access, __newindex MT calls
                    vm_protect!(L, pc, base, {
                        lua_v_settable(L, rb as *const TValue, rc, ra);
                    });
                    continue 'dispatch;
                }
                LuauOpcode::LOP_GETTABLEN => {
                    // lvmexecute.cpp:802
                    let insn = *pc;
                    pc = pc.add(1);
                    let ra = VM_REG!(LUAU_INSN_A!(insn), L, base) as *mut TValue;
                    let rb = VM_REG!(LUAU_INSN_B!(insn), L, base) as *mut TValue;
                    let c = LUAU_INSN_C!(insn) as i32;

                    // fast-path: array lookup
                    if ttistable!(rb as *const TValue) {
                        let h = hvalue!(rb as *const TValue);

                        if (c as u32) < (*h).sizearray as u32 && (*h).metatable.is_null() {
                            setobj_2_s!(L, ra, (*h).array.add(c as usize) as *const TValue);
                            continue 'dispatch;
                        }

                        // fall through to slow path
                    }

                    // slow-path: handles out of bounds array lookups
                    let mut n: TValue = core::mem::zeroed();
                    setnvalue!(&mut n as *mut TValue, (c + 1) as f64);
                    vm_protect!(L, pc, base, {
                        lua_v_gettable(L, rb as *const TValue, &mut n as *mut TValue, ra);
                    });
                    continue 'dispatch;
                }
                LuauOpcode::LOP_SETTABLEN => {
                    // lvmexecute.cpp:830
                    let insn = *pc;
                    pc = pc.add(1);
                    let ra = VM_REG!(LUAU_INSN_A!(insn), L, base) as *mut TValue;
                    let rb = VM_REG!(LUAU_INSN_B!(insn), L, base) as *mut TValue;
                    let c = LUAU_INSN_C!(insn) as i32;

                    // fast-path: array assign
                    if ttistable!(rb as *const TValue) {
                        let h = hvalue!(rb as *const TValue);

                        if (c as u32) < (*h).sizearray as u32
                            && (*h).metatable.is_null()
                            && (*h).readonly == 0
                        {
                            setobj2t!(L, (*h).array.add(c as usize), ra as *const TValue);
                            luaC_barriert!(L, h, ra as *const TValue);
                            continue 'dispatch;
                        }

                        // fall through to slow path
                    }

                    // slow-path: handles out of bounds array lookups
                    let mut n: TValue = core::mem::zeroed();
                    setnvalue!(&mut n as *mut TValue, (c + 1) as f64);
                    vm_protect!(L, pc, base, {
                        lua_v_settable(L, rb as *const TValue, &mut n as *mut TValue, ra);
                    });
                    continue 'dispatch;
                }
                LuauOpcode::LOP_NEWCLOSURE => {
                    // lvmexecute.cpp:859
                    let insn = *pc;
                    pc = pc.add(1);
                    let ra = VM_REG!(LUAU_INSN_A!(insn), L, base) as *mut TValue;

                    let (pv, sizep) = {
                        let l = &(*cl).inner.l;
                        (*(*l.p).p.add(LUAU_INSN_D!(insn) as usize), (*l.p).sizep)
                    };
                    LUAU_ASSERT!((LUAU_INSN_D!(insn) as u32) < sizep as u32);

                    (*(*L).ci).savedpc = pc; // VM_PROTECT_PC(): luaF_newLclosure may fail due to OOM

                    // note: we save closure to stack early in case the code below
                    // wants to capture it by value
                    let ncl = lua_f_new_lclosure(L, (*pv).nups as i32, (*cl).env, pv);
                    setclvalue!(L, ra, ncl);

                    for ui in 0..(*pv).nups as usize {
                        let uinsn = *pc;
                        pc = pc.add(1);
                        LUAU_ASSERT!(LUAU_INSN_OP!(uinsn) == LuauOpcode::LOP_CAPTURE as u32);

                        let uref = {
                            let l = &mut (*ncl).inner.l;
                            l.uprefs.as_mut_ptr().add(ui)
                        };
                        match LUAU_INSN_A!(uinsn) {
                            x if x == LuauCaptureType::LCT_VAL as u32 => {
                                setobj!(
                                    L,
                                    uref,
                                    VM_REG!(LUAU_INSN_B!(uinsn), L, base) as *const TValue
                                );
                            }
                            x if x == LuauCaptureType::LCT_REF as u32 => {
                                setupvalue!(
                                    L,
                                    uref,
                                    lua_f_findupval(
                                        L,
                                        VM_REG!(LUAU_INSN_B!(uinsn), L, base) as *mut TValue
                                    )
                                );
                            }
                            x if x == LuauCaptureType::LCT_UPVAL as u32 => {
                                setobj!(L, uref, VM_UV!(LUAU_INSN_B!(uinsn), cl) as *const TValue);
                            }
                            _ => {
                                // LUAU_ASSERT(!"Unknown upvalue capture type")
                                LUAU_ASSERT!(false);
                                unreachable!() // LUAU_UNREACHABLE()
                            }
                        }
                    }

                    vm_protect!(L, pc, base, {
                        luaC_checkGC!(L);
                    });
                    continue 'dispatch;
                }
                LuauOpcode::LOP_NAMECALL => {
                    // lvmexecute.cpp:902
                    let insn = *pc;
                    pc = pc.add(1);
                    let mut ra = VM_REG!(LUAU_INSN_A!(insn), L, base) as *mut TValue;
                    let rb = VM_REG!(LUAU_INSN_B!(insn), L, base) as *mut TValue;
                    let aux: u32 = *pc;
                    pc = pc.add(1);
                    let kv = VM_KV!(aux, cl, k) as *mut TValue;
                    LUAU_ASSERT!(ttisstring!(kv as *const TValue));

                    if ttistable!(rb as *const TValue) {
                        let h = hvalue!(rb as *const TValue);
                        // note: we can't use nodemask8 here because we need to query the
                        // main position of the table, and 8-bit nodemask8 only works for
                        // predictive lookups
                        let n = (*h).node.add(
                            ((*tsvalue!(kv as *const TValue)).hash & (sizenode!(h) - 1) as u32)
                                as usize,
                        );

                        // fast-path: key is in the table in expected slot
                        if ttisstring!(gkey!(n) as *const TValue)
                            && tsvalue!(gkey!(n) as *const TValue) == tsvalue!(kv as *const TValue)
                            && !ttisnil!(gval!(n))
                        {
                            // note: order of copies allows rb to alias ra+1 or ra
                            setobj_2_s!(L, ra.add(1), rb as *const TValue);
                            setobj_2_s!(L, ra, gval!(n));
                        } else {
                            // fast-path: key is absent from the base, table has an
                            // __index table, and it has the result in the expected slot
                            let mut hit_mt_fast = false;
                            if gnext!(n) == 0 {
                                let mt = fasttm(
                                    L,
                                    (*hvalue!(rb as *const TValue)).metatable,
                                    TMS::TM_INDEX as i32,
                                );
                                if !mt.is_null() && ttistable!(mt) {
                                    let mtn = (*hvalue!(mt)).node.add(
                                        ((LUAU_INSN_C!(insn) as i32)
                                            & (*hvalue!(mt)).nodemask8 as i32)
                                            as usize,
                                    );
                                    if ttisstring!(gkey!(mtn) as *const TValue)
                                        && tsvalue!(gkey!(mtn) as *const TValue)
                                            == tsvalue!(kv as *const TValue)
                                        && !ttisnil!(gval!(mtn))
                                    {
                                        // note: order of copies allows rb to alias ra+1 or ra
                                        setobj_2_s!(L, ra.add(1), rb as *const TValue);
                                        setobj_2_s!(L, ra, gval!(mtn));
                                        hit_mt_fast = true;
                                    }
                                }
                            }
                            if !hit_mt_fast {
                                // slow-path: handles full table lookup
                                setobj_2_s!(L, ra.add(1), rb as *const TValue);
                                (*L).cachedslot = LUAU_INSN_C!(insn) as i32;
                                vm_protect!(L, pc, base, {
                                    lua_v_gettable(L, rb as *const TValue, kv, ra);
                                });
                                VM_PATCH_C(pc.sub(2), (*L).cachedslot);
                                // recompute ra since stack might have been reallocated
                                ra = VM_REG!(LUAU_INSN_A!(insn), L, base) as *mut TValue;
                                if ttisnil!(ra as *const TValue) {
                                    luaG_methoderror(
                                        L,
                                        ra.add(1) as *const TValue,
                                        tsvalue!(kv as *const TValue),
                                    );
                                }
                            }
                        }
                    } else {
                        let mt = if ttisuserdata!(rb as *const TValue) {
                            (*uvalue!(rb as *const TValue)).metatable
                        } else {
                            (*(*L).global).mt[ttype!(rb as *const TValue) as usize]
                        };

                        // fast-path: metatable with __namecall
                        let fn_nc = fasttm(L, mt, TMS::TM_NAMECALL as i32);
                        if !fn_nc.is_null() {
                            // note: order of copies allows rb to alias ra+1 or ra
                            setobj_2_s!(L, ra.add(1), rb as *const TValue);
                            setobj_2_s!(L, ra, fn_nc);

                            (*L).namecall = tsvalue!(kv as *const TValue)
                                as *mut crate::records::t_string::TString;
                        } else {
                            let tmi = fasttm(L, mt, TMS::TM_INDEX as i32);
                            if !tmi.is_null() && ttistable!(tmi) {
                                let h = hvalue!(tmi);
                                let slot = (LUAU_INSN_C!(insn) as i32) & (*h).nodemask8 as i32;
                                let n = (*h).node.add(slot as usize);

                                // fast-path: metatable with __index that has method in expected slot
                                if ttisstring!(gkey!(n) as *const TValue)
                                    && tsvalue!(gkey!(n) as *const TValue)
                                        == tsvalue!(kv as *const TValue)
                                    && !ttisnil!(gval!(n))
                                {
                                    // note: order of copies allows rb to alias ra+1 or ra
                                    setobj_2_s!(L, ra.add(1), rb as *const TValue);
                                    setobj_2_s!(L, ra, gval!(n));
                                } else {
                                    // slow-path: handles slot mismatch
                                    setobj_2_s!(L, ra.add(1), rb as *const TValue);
                                    (*L).cachedslot = slot;
                                    vm_protect!(L, pc, base, {
                                        lua_v_gettable(L, rb as *const TValue, kv, ra);
                                    });
                                    VM_PATCH_C(pc.sub(2), (*L).cachedslot);
                                    // recompute ra since stack might have been reallocated
                                    ra = VM_REG!(LUAU_INSN_A!(insn), L, base) as *mut TValue;
                                    if ttisnil!(ra as *const TValue) {
                                        luaG_methoderror(
                                            L,
                                            ra.add(1) as *const TValue,
                                            tsvalue!(kv as *const TValue),
                                        );
                                    }
                                }
                            } else if luaur_common::FFlag::DebugLuauUserDefinedClassesRuntime.get()
                                && ttisobject!(rb as *const TValue)
                            {
                                let slot = LUAU_INSN_C!(insn) as i32;
                                let inst =
                                    &mut **objectvalue!(rb as *const TValue) as *mut LuauObject;
                                if slot < (*(*inst).lclass).numberofallmembers
                                    && tsvalue!(kv as *const TValue)
                                        == *(*(*inst).lclass).offsettomember.add(slot as usize)
                                {
                                    // note: order of copies allows rb to alias ra+1 or ra
                                    setobj_2_s!(L, ra.add(1), rb as *const TValue);
                                    setobj_2_s!(L, ra, luaR_lookupmemberatoffset!(inst, slot));
                                } else {
                                    // slow-er path: try to fetch the field manually.
                                    let offset = luaH_getstr(
                                        (*(*inst).lclass).memberstooffset,
                                        tsvalue!(kv as *const TValue)
                                            as *mut crate::records::t_string::TString,
                                    );
                                    if ttisnil!(offset) {
                                        luaG_missingmembererror(
                                            L,
                                            rb as *const TValue,
                                            kv as *const TValue,
                                        );
                                    }
                                    LUAU_ASSERT!(ttisnumber!(offset));
                                    let offsetnum = nvalue!(offset) as i32;
                                    setobj_2_s!(L, ra.add(1), rb as *const TValue);
                                    setobj_2_s!(L, ra, luaR_lookupmemberatoffset!(inst, offsetnum));
                                    VM_PATCH_C(pc.sub(2), offsetnum);
                                }
                            } else {
                                // slow-path: handles non-table __index
                                setobj_2_s!(L, ra.add(1), rb as *const TValue);
                                vm_protect!(L, pc, base, {
                                    lua_v_gettable(L, rb as *const TValue, kv, ra);
                                });
                                // recompute ra since stack might have been reallocated
                                ra = VM_REG!(LUAU_INSN_A!(insn), L, base) as *mut TValue;
                                if ttisnil!(ra as *const TValue) {
                                    luaG_methoderror(
                                        L,
                                        ra.add(1) as *const TValue,
                                        tsvalue!(kv as *const TValue),
                                    );
                                }
                            }
                        }
                    }

                    if luaur_common::FFlag::LuauCallFeedback.get() {
                        continue 'dispatch;
                    } else {
                        // intentional fallthrough to CALL (C++ case fallthrough; pc
                        // points at the CALL instruction, so forcing the dispatch op
                        // is semantically identical)
                        LUAU_ASSERT!(LUAU_INSN_OP!(*pc) == LuauOpcode::LOP_CALL as u32);
                        continue_op = Some(LuauOpcode::LOP_CALL as u8);
                        continue 'dispatch;
                    }
                }
                LuauOpcode::LOP_CALL => {
                    // lvmexecute.cpp:1038
                    VM_INTERRUPT!(L, pc, base);
                    let insn = *pc;
                    pc = pc.add(1);
                    let ra = VM_REG!(LUAU_INSN_A!(insn), L, base) as *mut TValue;

                    let nparams = LUAU_INSN_B!(insn) as i32 - 1;
                    let nresults = LUAU_INSN_C!(insn) as i32 - 1;

                    let mut argtop = (*L).top;
                    argtop = if nparams == LUA_MULTRET {
                        argtop
                    } else {
                        ra.add(1 + nparams as usize)
                    };

                    if !ttisfunction!(ra as *const TValue) {
                        // slow-path: not a function call
                        (*(*L).ci).savedpc = pc; // VM_PROTECT_PC(): luaV_tryfuncTM may fail

                        lua_v_tryfunc_tm(L, ra);
                        argtop = argtop.add(1); // __call adds an extra self
                    }

                    let ccl = clvalue!(ra as *const TValue);
                    (*(*L).ci).savedpc = pc;

                    incr_ci!(L);
                    let ci = (*L).ci;
                    (*ci).func = ra;
                    (*ci).base = ra.add(1);
                    // note: technically UB since we haven't reallocated the stack yet
                    (*ci).top = argtop.add((*ccl).stacksize as usize);
                    (*ci).savedpc = core::ptr::null();
                    (*ci).flags = 0;
                    (*ci).nresults = nresults;

                    (*L).base = (*ci).base;
                    (*L).top = argtop;

                    if luaur_common::FFlag::LuauClosureUsageCounter.get() {
                        (*ccl).usage += 1;
                    }

                    // note: this reallocs stack, but we don't need to VM_PROTECT this
                    // this is because we're going to modify base/savedpc manually anyhow
                    // crucially, we can't use ra/argtop after this line
                    luaD_checkstackfornewci(L, (*ccl).stacksize as i32);

                    LUAU_ASSERT!((*ci).top <= (*L).stack_last);

                    if (*ccl).isC == 0 {
                        let p = {
                            let l = &(*ccl).inner.l;
                            l.p
                        };

                        // fill unused parameters with nil
                        let mut argi = (*L).top;
                        let argend = (*L).base.add((*p).numparams as usize);
                        while argi < argend {
                            setnilvalue!(argi); // complete missing arguments
                            argi = argi.add(1);
                        }
                        (*L).top = if (*p).is_vararg != 0 { argi } else { (*ci).top };

                        // reentry
                        // codeentry may point to NATIVECALL instruction when proto is
                        // compiled to native code; execution continues in native code.
                        // note that p->codeentry may point *outside* of
                        // p->code..p->code+p->sizecode, but that pointer never gets
                        // saved to savedpc.
                        pc = if SINGLE_STEP {
                            (*p).code
                        } else {
                            (*p).codeentry
                        };
                        cl = ccl;
                        base = (*L).base;
                        k = (*p).k;
                        continue 'dispatch;
                    } else {
                        let func = {
                            let c = &(*ccl).inner.c;
                            c.f
                        };
                        let n = match func {
                            Some(f) => f(L),
                            None => 0,
                        };

                        // yield
                        if n < 0 {
                            return; // goto exit
                        }

                        // ci is our callinfo, cip is our parent
                        let ci = (*L).ci;
                        let cip = ci.sub(1);

                        if luaur_common::FFlag::LuauClosureUsageCounter.get() {
                            LUAU_ASSERT!((*ccl).usage > 0);
                            (*ccl).usage -= 1;
                        }

                        // copy return values into parent stack (but only up to
                        // nresults!), fill the rest with nil
                        // note: in MULTRET context nresults starts as -1 so i != 0
                        // condition never activates intentionally
                        let mut res = (*ci).func;
                        let mut vali = (*L).top.sub(n as usize);
                        let valend = (*L).top;

                        let mut i = nresults;
                        while i != 0 && vali < valend {
                            setobj_2_s!(L, res, vali as *const TValue);
                            res = res.add(1);
                            vali = vali.add(1);
                            i -= 1;
                        }
                        while i > 0 {
                            setnilvalue!(res);
                            res = res.add(1);
                            i -= 1;
                        }

                        // pop the stack frame
                        (*L).ci = cip;
                        (*L).base = (*cip).base;
                        (*L).top = if nresults == LUA_MULTRET {
                            res
                        } else {
                            (*cip).top
                        };

                        // stack may have been reallocated, so we need to refresh base ptr
                        base = (*L).base;
                        continue 'dispatch;
                    }
                }
                LuauOpcode::LOP_CALLFB => {
                    // lvmexecute.cpp:1145
                    VM_INTERRUPT!(L, pc, base);
                    let insn = *pc;
                    pc = pc.add(1);
                    let feedback_slot: Instruction = *pc;
                    pc = pc.add(1);
                    let ra = VM_REG!(LUAU_INSN_A!(insn), L, base) as *mut TValue;

                    let nparams = LUAU_INSN_B!(insn) as i32 - 1;
                    let nresults = LUAU_INSN_C!(insn) as i32 - 1;

                    let mut argtop = (*L).top;
                    argtop = if nparams == LUA_MULTRET {
                        argtop
                    } else {
                        ra.add(1 + nparams as usize)
                    };

                    // slow-path: not a function call
                    if !ttisfunction!(ra as *const TValue) {
                        if feedback_slot != LUAU_INSN_FBSLOT_SEALED {
                            VM_PATCH_AUX(pc.sub(1), LUAU_INSN_FBSLOT_SEALED as i32);
                        }

                        (*(*L).ci).savedpc = pc; // VM_PROTECT_PC(): luaV_tryfuncTM may fail

                        lua_v_tryfunc_tm(L, ra);
                        argtop = argtop.add(1); // __call adds an extra self
                    }

                    let ccl = clvalue!(ra as *const TValue);
                    (*(*L).ci).savedpc = pc;

                    incr_ci!(L);
                    let ci = (*L).ci;
                    (*ci).func = ra;
                    (*ci).base = ra.add(1);
                    // note: technically UB since we haven't reallocated the stack yet
                    (*ci).top = argtop.add((*ccl).stacksize as usize);
                    (*ci).savedpc = core::ptr::null();
                    (*ci).flags = 0;
                    (*ci).nresults = nresults;

                    (*L).base = (*ci).base;
                    (*L).top = argtop;

                    if luaur_common::FFlag::LuauClosureUsageCounter.get() {
                        (*ccl).usage += 1;
                    }

                    // note: this reallocs stack, but we don't need to VM_PROTECT this
                    // this is because we're going to modify base/savedpc manually anyhow
                    // crucially, we can't use ra/argtop after this line
                    luaD_checkstackfornewci(L, (*ccl).stacksize as i32);

                    LUAU_ASSERT!((*ci).top <= (*L).stack_last);

                    if (*ccl).isC == 0 {
                        let p = {
                            let l = &(*ccl).inner.l;
                            l.p
                        };

                        if feedback_slot != LUAU_INSN_FBSLOT_SEALED {
                            if !luaF_recordhit(L, cl, ccl, feedback_slot) {
                                VM_PATCH_AUX(pc.sub(1), LUAU_INSN_FBSLOT_SEALED as i32);
                            }
                        }

                        // fill unused parameters with nil
                        let mut argi = (*L).top;
                        let argend = (*L).base.add((*p).numparams as usize);
                        while argi < argend {
                            setnilvalue!(argi); // complete missing arguments
                            argi = argi.add(1);
                        }
                        (*L).top = if (*p).is_vararg != 0 { argi } else { (*ci).top };

                        // reentry (see LOP_CALL for the codeentry note)
                        pc = if SINGLE_STEP {
                            (*p).code
                        } else {
                            (*p).codeentry
                        };
                        cl = ccl;
                        base = (*L).base;
                        k = (*p).k;
                        continue 'dispatch;
                    } else {
                        if feedback_slot != LUAU_INSN_FBSLOT_SEALED {
                            VM_PATCH_AUX(pc.sub(1), LUAU_INSN_FBSLOT_SEALED as i32);
                        }

                        let func = {
                            let c = &(*ccl).inner.c;
                            c.f
                        };
                        let n = match func {
                            Some(f) => f(L),
                            None => 0,
                        };

                        // yield
                        if n < 0 {
                            return; // goto exit
                        }

                        // ci is our callinfo, cip is our parent
                        let ci = (*L).ci;
                        let cip = ci.sub(1);

                        if luaur_common::FFlag::LuauClosureUsageCounter.get() {
                            LUAU_ASSERT!((*ccl).usage > 0);
                            (*ccl).usage -= 1;
                        }

                        // copy return values into parent stack (but only up to
                        // nresults!), fill the rest with nil
                        let mut res = (*ci).func;
                        let mut vali = (*L).top.sub(n as usize);
                        let valend = (*L).top;

                        let mut i = nresults;
                        while i != 0 && vali < valend {
                            setobj_2_s!(L, res, vali as *const TValue);
                            res = res.add(1);
                            vali = vali.add(1);
                            i -= 1;
                        }
                        while i > 0 {
                            setnilvalue!(res);
                            res = res.add(1);
                            i -= 1;
                        }

                        // pop the stack frame
                        (*L).ci = cip;
                        (*L).base = (*cip).base;
                        (*L).top = if nresults == LUA_MULTRET {
                            res
                        } else {
                            (*cip).top
                        };

                        // stack may have been reallocated, so we need to refresh base ptr
                        base = (*L).base;
                        continue 'dispatch;
                    }
                }
                LuauOpcode::LOP_RETURN => {
                    // lvmexecute.cpp:1265
                    VM_INTERRUPT!(L, pc, base);
                    let insn = *pc;
                    pc = pc.add(1);
                    // note: this can point to L->top if b == LUA_MULTRET making VM_REG unsafe to use
                    let ra: StkId = base.add(LUAU_INSN_A!(insn) as usize);
                    let b = LUAU_INSN_B!(insn) as i32 - 1;

                    // ci is our callinfo, cip is our parent
                    let ci = (*L).ci;
                    let cip = ci.sub(1);

                    if luaur_common::FFlag::LuauClosureUsageCounter.get() {
                        let cicl = clvalue!((*ci).func);
                        LUAU_ASSERT!((*cicl).usage > 0);
                        (*cicl).usage -= 1;
                    }

                    // note: we assume CALL always puts func+args and expects results
                    // to start at func
                    let mut res = (*ci).func;

                    let mut vali = ra;
                    // copy as much as possible for MULTRET calls, and only as much as
                    // needed otherwise
                    let valend = if b == LUA_MULTRET {
                        (*L).top
                    } else {
                        ra.add(b as usize)
                    };

                    let nresults = (*ci).nresults;

                    // copy return values into parent stack (but only up to nresults!),
                    // fill the rest with nil
                    // note: in MULTRET context nresults starts as -1 so i != 0
                    // condition never activates intentionally
                    let mut i = nresults;
                    while i != 0 && vali < valend {
                        setobj_2_s!(L, res, vali as *const TValue);
                        res = res.add(1);
                        vali = vali.add(1);
                        i -= 1;
                    }
                    while i > 0 {
                        setnilvalue!(res);
                        res = res.add(1);
                        i -= 1;
                    }

                    // pop the stack frame
                    (*L).ci = cip;
                    (*L).base = (*cip).base;
                    (*L).top = if nresults == LUA_MULTRET {
                        res
                    } else {
                        (*cip).top
                    };

                    // we're done!
                    if ((*ci).flags as i32 & LUA_CALLINFO_RETURN) != 0 {
                        return; // goto exit
                    }

                    LUAU_ASSERT!(isLua!((*L).ci));

                    let nextcl = clvalue!((*cip).func);
                    let nextproto = {
                        let l = &(*nextcl).inner.l;
                        l.p
                    };

                    // VM_HAS_NATIVE
                    if ((*cip).flags as i32 & LUA_CALLINFO_NATIVE) != 0 && !SINGLE_STEP {
                        if let Some(enter) = (*(*L).global).ecb.enter {
                            if enter(L, nextproto) == 1 {
                                continue 'reentry; // goto reentry
                            } else {
                                return; // goto exit
                            }
                        }
                    }

                    // reentry
                    pc = (*cip).savedpc;
                    cl = nextcl;
                    base = (*L).base;
                    k = (*nextproto).k;
                    continue 'dispatch;
                }
                LuauOpcode::LOP_JUMP => {
                    // lvmexecute.cpp:1332
                    let insn = *pc;
                    pc = pc.add(1);

                    pc = pc.offset(LUAU_INSN_D!(insn) as isize);
                    let p = {
                        let l = &(*cl).inner.l;
                        l.p
                    };
                    LUAU_ASSERT!((pc.offset_from((*p).code) as u32) < (*p).sizecode as u32);
                    continue 'dispatch;
                }

                LuauOpcode::LOP_JUMPIF => {
                    // lvmexecute.cpp:1341
                    let insn = *pc;
                    pc = pc.add(1);
                    let ra = VM_REG!(LUAU_INSN_A!(insn), L, base);

                    pc = pc.offset(if l_isfalse!(ra as *const TValue) {
                        0
                    } else {
                        LUAU_INSN_D!(insn) as isize
                    });
                    let p = {
                        let l = &(*cl).inner.l;
                        l.p
                    };
                    LUAU_ASSERT!((pc.offset_from((*p).code) as u32) < (*p).sizecode as u32);
                    continue 'dispatch;
                }

                LuauOpcode::LOP_JUMPIFNOT => {
                    // lvmexecute.cpp:1351
                    let insn = *pc;
                    pc = pc.add(1);
                    let ra = VM_REG!(LUAU_INSN_A!(insn), L, base);

                    pc = pc.offset(if l_isfalse!(ra as *const TValue) {
                        LUAU_INSN_D!(insn) as isize
                    } else {
                        0
                    });
                    let p = {
                        let l = &(*cl).inner.l;
                        l.p
                    };
                    LUAU_ASSERT!((pc.offset_from((*p).code) as u32) < (*p).sizecode as u32);
                    continue 'dispatch;
                }
                LuauOpcode::LOP_JUMPIFEQ => {
                    // lvmexecute.cpp:1361
                    let insn = *pc;
                    pc = pc.add(1);
                    let aux: u32 = *pc;
                    let ra = VM_REG!(LUAU_INSN_A!(insn), L, base) as *mut TValue;
                    let rb = VM_REG!(aux, L, base) as *mut TValue;

                    macro_rules! jump_and_next {
                        ($cond:expr) => {{
                            pc = pc.offset(if $cond {
                                LUAU_INSN_D!(insn) as isize
                            } else {
                                1
                            });
                            let p = {
                                let l = &(*cl).inner.l;
                                l.p
                            };
                            LUAU_ASSERT!((pc.offset_from((*p).code) as u32) < (*p).sizecode as u32);
                            continue 'dispatch;
                        }};
                    }

                    // Note that all jumps below jump by 1 in the "false" case to skip over aux
                    if ttype!(ra as *const TValue) == ttype!(rb as *const TValue) {
                        let t = ttype!(ra as *const TValue);
                        // C++ switch over ttype; TABLE/USERDATA/OBJECT `break` out of
                        // the switch into the shared slow path below the match.
                        if t == lua_Type::LUA_TNIL as i32 {
                            jump_and_next!(true);
                        } else if t == lua_Type::LUA_TBOOLEAN as i32 {
                            jump_and_next!(
                                bvalue!(ra as *const TValue) == bvalue!(rb as *const TValue)
                            );
                        } else if t == lua_Type::LUA_TLIGHTUSERDATA as i32 {
                            jump_and_next!(
                                pvalue!(ra as *const TValue) == pvalue!(rb as *const TValue)
                                    && lightuserdatatag!(ra as *const TValue)
                                        == lightuserdatatag!(rb as *const TValue)
                            );
                        } else if t == lua_Type::LUA_TNUMBER as i32 {
                            jump_and_next!(
                                nvalue!(ra as *const TValue) == nvalue!(rb as *const TValue)
                            );
                        } else if t == lua_Type::LUA_TVECTOR as i32 {
                            jump_and_next!(luai_veceq(
                                vvalue!(ra as *const TValue).as_ptr(),
                                vvalue!(rb as *const TValue).as_ptr()
                            ));
                        } else if t == lua_Type::LUA_TSTRING as i32
                            || t == lua_Type::LUA_TFUNCTION as i32
                            || t == lua_Type::LUA_TTHREAD as i32
                            || t == lua_Type::LUA_TBUFFER as i32
                        {
                            jump_and_next!(
                                gcvalue!(ra as *const TValue) == gcvalue!(rb as *const TValue)
                            );
                        } else if t == lua_Type::LUA_TTABLE as i32 {
                            // fast-path: same metatable, no EQ metamethod
                            if (*hvalue!(ra as *const TValue)).metatable
                                == (*hvalue!(rb as *const TValue)).metatable
                            {
                                let fn_tm = fasttm(
                                    L,
                                    (*hvalue!(ra as *const TValue)).metatable,
                                    TMS::TM_EQ as i32,
                                );
                                if fn_tm.is_null() {
                                    jump_and_next!(
                                        hvalue!(ra as *const TValue)
                                            == hvalue!(rb as *const TValue)
                                    );
                                }
                            }
                            // slow path after switch()
                        } else if t == lua_Type::LUA_TUSERDATA as i32 {
                            // fast-path: same metatable, no EQ metamethod or C metamethod
                            if (*uvalue!(ra as *const TValue)).metatable
                                == (*uvalue!(rb as *const TValue)).metatable
                            {
                                let fn_tm = fasttm(
                                    L,
                                    (*uvalue!(ra as *const TValue)).metatable,
                                    TMS::TM_EQ as i32,
                                );
                                if fn_tm.is_null() {
                                    jump_and_next!(
                                        uvalue!(ra as *const TValue) as *const _
                                            as *const core::ffi::c_void
                                            == uvalue!(rb as *const TValue) as *const _
                                                as *const core::ffi::c_void
                                    );
                                } else if ttisfunction!(fn_tm) && (*clvalue!(fn_tm)).isC != 0 {
                                    // note: it's safe to push arguments past top for
                                    // complicated reasons (see top of the file)
                                    LUAU_ASSERT!(
                                        (*L).top.add(3) < (*L).stack.add((*L).stacksize as usize)
                                    );
                                    let top = (*L).top;
                                    setobj_2_s!(L, top.add(0), fn_tm);
                                    setobj_2_s!(L, top.add(1), ra as *const TValue);
                                    setobj_2_s!(L, top.add(2), rb as *const TValue);
                                    let res = top.offset_from(base) as i32;
                                    (*L).top = top.add(3);

                                    vm_protect!(L, pc, base, {
                                        lua_v_call_tm(L, 2, res);
                                    });
                                    jump_and_next!(!l_isfalse!(
                                        base.add(res as usize) as *const TValue
                                    ));
                                }
                            }
                            // slow path after switch()
                        } else if t == lua_Type::LUA_TCLASS as i32 {
                            // Class objects are only ever physically equal, so check
                            // for pointer equality.
                            jump_and_next!(
                                classvalue!(ra as *const TValue) as *const _
                                    as *const core::ffi::c_void
                                    == classvalue!(rb as *const TValue) as *const _
                                        as *const core::ffi::c_void
                            );
                        } else if t == lua_Type::LUA_TOBJECT as i32 {
                            // For now, hit the slow path after the switch (we may
                            // need to invoke metamethods).
                        } else if t == lua_Type::LUA_TINTEGER as i32 {
                            jump_and_next!(
                                lvalue!(ra as *const TValue) == lvalue!(rb as *const TValue)
                            );
                        } else {
                            // LUAU_ASSERT(!"Unknown value type")
                            LUAU_ASSERT!(false);
                            unreachable!() // LUAU_UNREACHABLE()
                        }

                        // slow-path: tables with metatables and userdata values
                        // note that we don't have a fast path for userdata values
                        // without metatables, since that's very rare
                        let mut res: i32 = 0;
                        vm_protect!(L, pc, base, {
                            res = lua_v_equalval(L, ra as *const TValue, rb as *const TValue);
                        });

                        jump_and_next!(res == 1);
                    } else {
                        pc = pc.offset(1);
                        let p = {
                            let l = &(*cl).inner.l;
                            l.p
                        };
                        LUAU_ASSERT!((pc.offset_from((*p).code) as u32) < (*p).sizecode as u32);
                        continue 'dispatch;
                    }
                }
                LuauOpcode::LOP_JUMPIFNOTEQ => {
                    // lvmexecute.cpp:1494
                    let insn = *pc;
                    pc = pc.add(1);
                    let aux: u32 = *pc;
                    let ra = VM_REG!(LUAU_INSN_A!(insn), L, base) as *mut TValue;
                    let rb = VM_REG!(aux, L, base) as *mut TValue;

                    macro_rules! jump_and_next {
                        ($cond:expr) => {{
                            pc = pc.offset(if $cond {
                                LUAU_INSN_D!(insn) as isize
                            } else {
                                1
                            });
                            let p = {
                                let l = &(*cl).inner.l;
                                l.p
                            };
                            LUAU_ASSERT!((pc.offset_from((*p).code) as u32) < (*p).sizecode as u32);
                            continue 'dispatch;
                        }};
                    }

                    // Note that all jumps below jump by 1 in the "true" case to skip over aux
                    if ttype!(ra as *const TValue) == ttype!(rb as *const TValue) {
                        let t = ttype!(ra as *const TValue);
                        // C++ switch over ttype; TABLE/USERDATA/OBJECT `break` out of
                        // the switch into the shared slow path below.
                        if t == lua_Type::LUA_TNIL as i32 {
                            jump_and_next!(false); // pc += 1
                        } else if t == lua_Type::LUA_TBOOLEAN as i32 {
                            jump_and_next!(
                                bvalue!(ra as *const TValue) != bvalue!(rb as *const TValue)
                            );
                        } else if t == lua_Type::LUA_TLIGHTUSERDATA as i32 {
                            jump_and_next!(
                                pvalue!(ra as *const TValue) != pvalue!(rb as *const TValue)
                                    || lightuserdatatag!(ra as *const TValue)
                                        != lightuserdatatag!(rb as *const TValue)
                            );
                        } else if t == lua_Type::LUA_TNUMBER as i32 {
                            jump_and_next!(
                                nvalue!(ra as *const TValue) != nvalue!(rb as *const TValue)
                            );
                        } else if t == lua_Type::LUA_TVECTOR as i32 {
                            jump_and_next!(!luai_veceq(
                                vvalue!(ra as *const TValue).as_ptr(),
                                vvalue!(rb as *const TValue).as_ptr()
                            ));
                        } else if t == lua_Type::LUA_TSTRING as i32
                            || t == lua_Type::LUA_TFUNCTION as i32
                            || t == lua_Type::LUA_TTHREAD as i32
                            || t == lua_Type::LUA_TBUFFER as i32
                        {
                            jump_and_next!(
                                gcvalue!(ra as *const TValue) != gcvalue!(rb as *const TValue)
                            );
                        } else if t == lua_Type::LUA_TTABLE as i32 {
                            // fast-path: same metatable, no EQ metamethod
                            if (*hvalue!(ra as *const TValue)).metatable
                                == (*hvalue!(rb as *const TValue)).metatable
                            {
                                let fn_tm = fasttm(
                                    L,
                                    (*hvalue!(ra as *const TValue)).metatable,
                                    TMS::TM_EQ as i32,
                                );
                                if fn_tm.is_null() {
                                    jump_and_next!(
                                        hvalue!(ra as *const TValue)
                                            != hvalue!(rb as *const TValue)
                                    );
                                }
                            }
                            // slow path after switch()
                        } else if t == lua_Type::LUA_TUSERDATA as i32 {
                            // fast-path: same metatable, no EQ metamethod or C metamethod
                            if (*uvalue!(ra as *const TValue)).metatable
                                == (*uvalue!(rb as *const TValue)).metatable
                            {
                                let fn_tm = fasttm(
                                    L,
                                    (*uvalue!(ra as *const TValue)).metatable,
                                    TMS::TM_EQ as i32,
                                );
                                if fn_tm.is_null() {
                                    jump_and_next!(
                                        uvalue!(ra as *const TValue) as *const _
                                            as *const core::ffi::c_void
                                            != uvalue!(rb as *const TValue) as *const _
                                                as *const core::ffi::c_void
                                    );
                                } else if ttisfunction!(fn_tm) && (*clvalue!(fn_tm)).isC != 0 {
                                    // note: it's safe to push arguments past top for
                                    // complicated reasons (see top of the file)
                                    LUAU_ASSERT!(
                                        (*L).top.add(3) < (*L).stack.add((*L).stacksize as usize)
                                    );
                                    let top = (*L).top;
                                    setobj_2_s!(L, top.add(0), fn_tm);
                                    setobj_2_s!(L, top.add(1), ra as *const TValue);
                                    setobj_2_s!(L, top.add(2), rb as *const TValue);
                                    let res = top.offset_from(base) as i32;
                                    (*L).top = top.add(3);

                                    vm_protect!(L, pc, base, {
                                        lua_v_call_tm(L, 2, res);
                                    });
                                    jump_and_next!(l_isfalse!(
                                        base.add(res as usize) as *const TValue
                                    ));
                                }
                            }
                            // slow path after switch()
                        } else if t == lua_Type::LUA_TCLASS as i32 {
                            // Class objects are only ever physically equal, so check
                            // for pointer inequality.
                            jump_and_next!(
                                classvalue!(ra as *const TValue) as *const _
                                    as *const core::ffi::c_void
                                    != classvalue!(rb as *const TValue) as *const _
                                        as *const core::ffi::c_void
                            );
                        } else if t == lua_Type::LUA_TOBJECT as i32 {
                            // For now, hit the slow path after the switch (we may
                            // need to invoke metamethods).
                        } else if t == lua_Type::LUA_TINTEGER as i32 {
                            jump_and_next!(
                                lvalue!(ra as *const TValue) != lvalue!(rb as *const TValue)
                            );
                        } else {
                            // LUAU_ASSERT(!"Unknown value type")
                            LUAU_ASSERT!(false);
                            unreachable!() // LUAU_UNREACHABLE()
                        }

                        // slow-path: tables with metatables and userdata values
                        // note that we don't have a fast path for userdata values
                        // without metatables, since that's very rare
                        let mut res: i32 = 0;
                        vm_protect!(L, pc, base, {
                            res = lua_v_equalval(L, ra as *const TValue, rb as *const TValue);
                        });

                        jump_and_next!(res == 0);
                    } else {
                        pc = pc.offset(LUAU_INSN_D!(insn) as isize);
                        let p = {
                            let l = &(*cl).inner.l;
                            l.p
                        };
                        LUAU_ASSERT!((pc.offset_from((*p).code) as u32) < (*p).sizecode as u32);
                        continue 'dispatch;
                    }
                }
                LuauOpcode::LOP_JUMPIFLE => {
                    // lvmexecute.cpp:1627
                    let insn = *pc;
                    pc = pc.add(1);
                    let aux: u32 = *pc;
                    let ra = VM_REG!(LUAU_INSN_A!(insn), L, base) as *mut TValue;
                    let rb = VM_REG!(aux, L, base) as *mut TValue;

                    macro_rules! jump_and_next {
                        ($cond:expr) => {{
                            pc = pc.offset(if $cond {
                                LUAU_INSN_D!(insn) as isize
                            } else {
                                1
                            });
                            let p = {
                                let l = &(*cl).inner.l;
                                l.p
                            };
                            LUAU_ASSERT!((pc.offset_from((*p).code) as u32) < (*p).sizecode as u32);
                            continue 'dispatch;
                        }};
                    }

                    // fast-path: number
                    // Note that all jumps below jump by 1 in the "false" case to skip over aux
                    if ttisnumber!(ra as *const TValue) && ttisnumber!(rb as *const TValue) {
                        jump_and_next!(
                            nvalue!(ra as *const TValue) <= nvalue!(rb as *const TValue)
                        );
                    }
                    // fast-path: string
                    else if ttisstring!(ra as *const TValue) && ttisstring!(rb as *const TValue) {
                        jump_and_next!(
                            lua_v_strcmp(
                                tsvalue!(ra as *const TValue),
                                tsvalue!(rb as *const TValue)
                            ) <= 0
                        );
                    } else {
                        let mut res: i32 = 0;
                        vm_protect!(L, pc, base, {
                            res = lua_v_lessequal(L, ra as *const TValue, rb as *const TValue);
                        });

                        jump_and_next!(res == 1);
                    }
                }
                LuauOpcode::LOP_JUMPIFNOTLE => {
                    // lvmexecute.cpp:1660
                    let insn = *pc;
                    pc = pc.add(1);
                    let aux: u32 = *pc;
                    let ra = VM_REG!(LUAU_INSN_A!(insn), L, base) as *mut TValue;
                    let rb = VM_REG!(aux, L, base) as *mut TValue;

                    macro_rules! jump_and_next {
                        ($cond:expr) => {{
                            pc = pc.offset(if $cond {
                                LUAU_INSN_D!(insn) as isize
                            } else {
                                1
                            });
                            let p = {
                                let l = &(*cl).inner.l;
                                l.p
                            };
                            LUAU_ASSERT!((pc.offset_from((*p).code) as u32) < (*p).sizecode as u32);
                            continue 'dispatch;
                        }};
                    }

                    // fast-path: number
                    // Note that all jumps below jump by 1 in the "true" case to skip over aux
                    if ttisnumber!(ra as *const TValue) && ttisnumber!(rb as *const TValue) {
                        jump_and_next!(
                            !(nvalue!(ra as *const TValue) <= nvalue!(rb as *const TValue))
                        );
                    }
                    // fast-path: string
                    else if ttisstring!(ra as *const TValue) && ttisstring!(rb as *const TValue) {
                        jump_and_next!(
                            !(lua_v_strcmp(
                                tsvalue!(ra as *const TValue),
                                tsvalue!(rb as *const TValue)
                            ) <= 0)
                        );
                    } else {
                        let mut res: i32 = 0;
                        vm_protect!(L, pc, base, {
                            res = lua_v_lessequal(L, ra as *const TValue, rb as *const TValue);
                        });

                        jump_and_next!(res == 0);
                    }
                }
                LuauOpcode::LOP_JUMPIFLT => {
                    // lvmexecute.cpp:1693
                    let insn = *pc;
                    pc = pc.add(1);
                    let aux: u32 = *pc;
                    let ra = VM_REG!(LUAU_INSN_A!(insn), L, base) as *mut TValue;
                    let rb = VM_REG!(aux, L, base) as *mut TValue;

                    macro_rules! jump_and_next {
                        ($cond:expr) => {{
                            pc = pc.offset(if $cond {
                                LUAU_INSN_D!(insn) as isize
                            } else {
                                1
                            });
                            let p = {
                                let l = &(*cl).inner.l;
                                l.p
                            };
                            LUAU_ASSERT!((pc.offset_from((*p).code) as u32) < (*p).sizecode as u32);
                            continue 'dispatch;
                        }};
                    }

                    // fast-path: number
                    // Note that all jumps below jump by 1 in the "false" case to skip over aux
                    if ttisnumber!(ra as *const TValue) && ttisnumber!(rb as *const TValue) {
                        jump_and_next!(nvalue!(ra as *const TValue) < nvalue!(rb as *const TValue));
                    }
                    // fast-path: string
                    else if ttisstring!(ra as *const TValue) && ttisstring!(rb as *const TValue) {
                        jump_and_next!(
                            lua_v_strcmp(
                                tsvalue!(ra as *const TValue),
                                tsvalue!(rb as *const TValue)
                            ) < 0
                        );
                    } else {
                        let mut res: i32 = 0;
                        vm_protect!(L, pc, base, {
                            res = lua_v_lessthan(L, ra as *const TValue, rb as *const TValue);
                        });

                        jump_and_next!(res == 1);
                    }
                }
                LuauOpcode::LOP_JUMPIFNOTLT => {
                    // lvmexecute.cpp:1726
                    let insn = *pc;
                    pc = pc.add(1);
                    let aux: u32 = *pc;
                    let ra = VM_REG!(LUAU_INSN_A!(insn), L, base) as *mut TValue;
                    let rb = VM_REG!(aux, L, base) as *mut TValue;

                    macro_rules! jump_and_next {
                        ($cond:expr) => {{
                            pc = pc.offset(if $cond {
                                LUAU_INSN_D!(insn) as isize
                            } else {
                                1
                            });
                            let p = {
                                let l = &(*cl).inner.l;
                                l.p
                            };
                            LUAU_ASSERT!((pc.offset_from((*p).code) as u32) < (*p).sizecode as u32);
                            continue 'dispatch;
                        }};
                    }

                    // fast-path: number
                    // Note that all jumps below jump by 1 in the "true" case to skip over aux
                    if ttisnumber!(ra as *const TValue) && ttisnumber!(rb as *const TValue) {
                        jump_and_next!(
                            !(nvalue!(ra as *const TValue) < nvalue!(rb as *const TValue))
                        );
                    }
                    // fast-path: string
                    else if ttisstring!(ra as *const TValue) && ttisstring!(rb as *const TValue) {
                        jump_and_next!(
                            !(lua_v_strcmp(
                                tsvalue!(ra as *const TValue),
                                tsvalue!(rb as *const TValue)
                            ) < 0)
                        );
                    } else {
                        let mut res: i32 = 0;
                        vm_protect!(L, pc, base, {
                            res = lua_v_lessthan(L, ra as *const TValue, rb as *const TValue);
                        });

                        jump_and_next!(res == 0);
                    }
                }
                LuauOpcode::LOP_ADD => {
                    // lvmexecute.cpp:1759
                    let insn = *pc;
                    pc = pc.add(1);
                    let ra = VM_REG!(LUAU_INSN_A!(insn), L, base) as *mut TValue;
                    let rb = VM_REG!(LUAU_INSN_B!(insn), L, base) as *mut TValue;
                    let rc = VM_REG!(LUAU_INSN_C!(insn), L, base) as *mut TValue;

                    // fast-path
                    if ttisnumber!(rb as *const TValue) && ttisnumber!(rc as *const TValue) {
                        setnvalue!(
                            ra,
                            nvalue!(rb as *const TValue) + nvalue!(rc as *const TValue)
                        );
                        continue 'dispatch;
                    } else if ttisvector!(rb as *const TValue) && ttisvector!(rc as *const TValue) {
                        let vb = vvalue!(rb as *const TValue).as_ptr();
                        let vc = vvalue!(rc as *const TValue).as_ptr();
                        setvvalue!(
                            ra,
                            *vb.add(0) + *vc.add(0),
                            *vb.add(1) + *vc.add(1),
                            *vb.add(2) + *vc.add(2),
                            *vb.add(3) + *vc.add(3)
                        );
                        continue 'dispatch;
                    } else {
                        // fast-path for userdata with C functions
                        let mut fn_tm: *const TValue = core::ptr::null();
                        if ttisuserdata!(rb as *const TValue)
                            && {
                                fn_tm = lua_t_gettmbyobj(L, rb as *const TValue, TMS::TM_ADD);
                                !fn_tm.is_null()
                            }
                            && ttisfunction!(fn_tm)
                            && (*clvalue!(fn_tm)).isC != 0
                        {
                            // note: it's safe to push arguments past top for
                            // complicated reasons (see top of the file)
                            LUAU_ASSERT!((*L).top.add(3) < (*L).stack.add((*L).stacksize as usize));
                            let top = (*L).top;
                            setobj_2_s!(L, top.add(0), fn_tm);
                            setobj_2_s!(L, top.add(1), rb as *const TValue);
                            setobj_2_s!(L, top.add(2), rc as *const TValue);
                            (*L).top = top.add(3);

                            vm_protect!(L, pc, base, {
                                lua_v_call_tm(L, 2, LUAU_INSN_A!(insn) as i32);
                            });
                            continue 'dispatch;
                        } else {
                            // slow-path, may invoke C/Lua via metamethods
                            vm_protect!(L, pc, base, {
                                lua_v_doarithimpl(
                                    L,
                                    ra,
                                    rb as *const TValue,
                                    rc as *const TValue,
                                    TMS::TM_ADD,
                                );
                            });
                            continue 'dispatch;
                        }
                    }
                }

                LuauOpcode::LOP_SUB => {
                    // lvmexecute.cpp:1805
                    let insn = *pc;
                    pc = pc.add(1);
                    let ra = VM_REG!(LUAU_INSN_A!(insn), L, base) as *mut TValue;
                    let rb = VM_REG!(LUAU_INSN_B!(insn), L, base) as *mut TValue;
                    let rc = VM_REG!(LUAU_INSN_C!(insn), L, base) as *mut TValue;

                    // fast-path
                    if ttisnumber!(rb as *const TValue) && ttisnumber!(rc as *const TValue) {
                        setnvalue!(
                            ra,
                            nvalue!(rb as *const TValue) - nvalue!(rc as *const TValue)
                        );
                        continue 'dispatch;
                    } else if ttisvector!(rb as *const TValue) && ttisvector!(rc as *const TValue) {
                        let vb = vvalue!(rb as *const TValue).as_ptr();
                        let vc = vvalue!(rc as *const TValue).as_ptr();
                        setvvalue!(
                            ra,
                            *vb.add(0) - *vc.add(0),
                            *vb.add(1) - *vc.add(1),
                            *vb.add(2) - *vc.add(2),
                            *vb.add(3) - *vc.add(3)
                        );
                        continue 'dispatch;
                    } else {
                        // fast-path for userdata with C functions
                        let mut fn_tm: *const TValue = core::ptr::null();
                        if ttisuserdata!(rb as *const TValue)
                            && {
                                fn_tm = lua_t_gettmbyobj(L, rb as *const TValue, TMS::TM_SUB);
                                !fn_tm.is_null()
                            }
                            && ttisfunction!(fn_tm)
                            && (*clvalue!(fn_tm)).isC != 0
                        {
                            // note: it's safe to push arguments past top for
                            // complicated reasons (see top of the file)
                            LUAU_ASSERT!((*L).top.add(3) < (*L).stack.add((*L).stacksize as usize));
                            let top = (*L).top;
                            setobj_2_s!(L, top.add(0), fn_tm);
                            setobj_2_s!(L, top.add(1), rb as *const TValue);
                            setobj_2_s!(L, top.add(2), rc as *const TValue);
                            (*L).top = top.add(3);

                            vm_protect!(L, pc, base, {
                                lua_v_call_tm(L, 2, LUAU_INSN_A!(insn) as i32);
                            });
                            continue 'dispatch;
                        } else {
                            // slow-path, may invoke C/Lua via metamethods
                            vm_protect!(L, pc, base, {
                                lua_v_doarithimpl(
                                    L,
                                    ra,
                                    rb as *const TValue,
                                    rc as *const TValue,
                                    TMS::TM_SUB,
                                );
                            });
                            continue 'dispatch;
                        }
                    }
                }
                LuauOpcode::LOP_MUL => {
                    // lvmexecute.cpp:1851
                    let insn = *pc;
                    pc = pc.add(1);
                    let ra = VM_REG!(LUAU_INSN_A!(insn), L, base) as *mut TValue;
                    let rb = VM_REG!(LUAU_INSN_B!(insn), L, base) as *mut TValue;
                    let rc = VM_REG!(LUAU_INSN_C!(insn), L, base) as *mut TValue;

                    // fast-path
                    if ttisnumber!(rb as *const TValue) && ttisnumber!(rc as *const TValue) {
                        setnvalue!(
                            ra,
                            nvalue!(rb as *const TValue) * nvalue!(rc as *const TValue)
                        );
                        continue 'dispatch;
                    } else if ttisvector!(rb as *const TValue) && ttisnumber!(rc as *const TValue) {
                        let vb = vvalue!(rb as *const TValue).as_ptr();
                        let vc = nvalue!(rc as *const TValue) as f32;
                        setvvalue!(
                            ra,
                            *vb.add(0) * vc,
                            *vb.add(1) * vc,
                            *vb.add(2) * vc,
                            *vb.add(3) * vc
                        );
                        continue 'dispatch;
                    } else if ttisvector!(rb as *const TValue) && ttisvector!(rc as *const TValue) {
                        let vb = vvalue!(rb as *const TValue).as_ptr();
                        let vc = vvalue!(rc as *const TValue).as_ptr();
                        setvvalue!(
                            ra,
                            *vb.add(0) * *vc.add(0),
                            *vb.add(1) * *vc.add(1),
                            *vb.add(2) * *vc.add(2),
                            *vb.add(3) * *vc.add(3)
                        );
                        continue 'dispatch;
                    } else if ttisnumber!(rb as *const TValue) && ttisvector!(rc as *const TValue) {
                        let vb = nvalue!(rb as *const TValue) as f32;
                        let vc = vvalue!(rc as *const TValue).as_ptr();
                        setvvalue!(
                            ra,
                            vb * *vc.add(0),
                            vb * *vc.add(1),
                            vb * *vc.add(2),
                            vb * *vc.add(3)
                        );
                        continue 'dispatch;
                    } else {
                        // fast-path for userdata with C functions
                        let rbc = if ttisnumber!(rb as *const TValue) {
                            rc
                        } else {
                            rb
                        };
                        let mut fn_tm: *const TValue = core::ptr::null();
                        if ttisuserdata!(rbc as *const TValue)
                            && {
                                fn_tm = lua_t_gettmbyobj(L, rbc as *const TValue, TMS::TM_MUL);
                                !fn_tm.is_null()
                            }
                            && ttisfunction!(fn_tm)
                            && (*clvalue!(fn_tm)).isC != 0
                        {
                            // note: it's safe to push arguments past top for
                            // complicated reasons (see top of the file)
                            LUAU_ASSERT!((*L).top.add(3) < (*L).stack.add((*L).stacksize as usize));
                            let top = (*L).top;
                            setobj_2_s!(L, top.add(0), fn_tm);
                            setobj_2_s!(L, top.add(1), rb as *const TValue);
                            setobj_2_s!(L, top.add(2), rc as *const TValue);
                            (*L).top = top.add(3);

                            vm_protect!(L, pc, base, {
                                lua_v_call_tm(L, 2, LUAU_INSN_A!(insn) as i32);
                            });
                            continue 'dispatch;
                        } else {
                            // slow-path, may invoke C/Lua via metamethods
                            vm_protect!(L, pc, base, {
                                lua_v_doarithimpl(
                                    L,
                                    ra,
                                    rb as *const TValue,
                                    rc as *const TValue,
                                    TMS::TM_MUL,
                                );
                            });
                            continue 'dispatch;
                        }
                    }
                }
                LuauOpcode::LOP_DIV => {
                    // lvmexecute.cpp:1912
                    let insn = *pc;
                    pc = pc.add(1);
                    let ra = VM_REG!(LUAU_INSN_A!(insn), L, base) as *mut TValue;
                    let rb = VM_REG!(LUAU_INSN_B!(insn), L, base) as *mut TValue;
                    let rc = VM_REG!(LUAU_INSN_C!(insn), L, base) as *mut TValue;

                    // fast-path
                    if ttisnumber!(rb as *const TValue) && ttisnumber!(rc as *const TValue) {
                        setnvalue!(
                            ra,
                            nvalue!(rb as *const TValue) / nvalue!(rc as *const TValue)
                        );
                        continue 'dispatch;
                    } else if ttisvector!(rb as *const TValue) && ttisnumber!(rc as *const TValue) {
                        let vb = vvalue!(rb as *const TValue).as_ptr();
                        let vc = nvalue!(rc as *const TValue) as f32;
                        setvvalue!(
                            ra,
                            *vb.add(0) / vc,
                            *vb.add(1) / vc,
                            *vb.add(2) / vc,
                            *vb.add(3) / vc
                        );
                        continue 'dispatch;
                    } else if ttisvector!(rb as *const TValue) && ttisvector!(rc as *const TValue) {
                        let vb = vvalue!(rb as *const TValue).as_ptr();
                        let vc = vvalue!(rc as *const TValue).as_ptr();
                        setvvalue!(
                            ra,
                            *vb.add(0) / *vc.add(0),
                            *vb.add(1) / *vc.add(1),
                            *vb.add(2) / *vc.add(2),
                            *vb.add(3) / *vc.add(3)
                        );
                        continue 'dispatch;
                    } else if ttisnumber!(rb as *const TValue) && ttisvector!(rc as *const TValue) {
                        let vb = nvalue!(rb as *const TValue) as f32;
                        let vc = vvalue!(rc as *const TValue).as_ptr();
                        setvvalue!(
                            ra,
                            vb / *vc.add(0),
                            vb / *vc.add(1),
                            vb / *vc.add(2),
                            vb / *vc.add(3)
                        );
                        continue 'dispatch;
                    } else {
                        // fast-path for userdata with C functions
                        let rbc = if ttisnumber!(rb as *const TValue) {
                            rc
                        } else {
                            rb
                        };
                        let mut fn_tm: *const TValue = core::ptr::null();
                        if ttisuserdata!(rbc as *const TValue)
                            && {
                                fn_tm = lua_t_gettmbyobj(L, rbc as *const TValue, TMS::TM_DIV);
                                !fn_tm.is_null()
                            }
                            && ttisfunction!(fn_tm)
                            && (*clvalue!(fn_tm)).isC != 0
                        {
                            // note: it's safe to push arguments past top for
                            // complicated reasons (see top of the file)
                            LUAU_ASSERT!((*L).top.add(3) < (*L).stack.add((*L).stacksize as usize));
                            let top = (*L).top;
                            setobj_2_s!(L, top.add(0), fn_tm);
                            setobj_2_s!(L, top.add(1), rb as *const TValue);
                            setobj_2_s!(L, top.add(2), rc as *const TValue);
                            (*L).top = top.add(3);

                            vm_protect!(L, pc, base, {
                                lua_v_call_tm(L, 2, LUAU_INSN_A!(insn) as i32);
                            });
                            continue 'dispatch;
                        } else {
                            // slow-path, may invoke C/Lua via metamethods
                            vm_protect!(L, pc, base, {
                                lua_v_doarithimpl(
                                    L,
                                    ra,
                                    rb as *const TValue,
                                    rc as *const TValue,
                                    TMS::TM_DIV,
                                );
                            });
                            continue 'dispatch;
                        }
                    }
                }
                LuauOpcode::LOP_IDIV => {
                    // lvmexecute.cpp:1973
                    let insn = *pc;
                    pc = pc.add(1);
                    let ra = VM_REG!(LUAU_INSN_A!(insn), L, base) as *mut TValue;
                    let rb = VM_REG!(LUAU_INSN_B!(insn), L, base) as *mut TValue;
                    let rc = VM_REG!(LUAU_INSN_C!(insn), L, base) as *mut TValue;

                    // fast-path
                    if ttisnumber!(rb as *const TValue) && ttisnumber!(rc as *const TValue) {
                        setnvalue!(
                            ra,
                            luai_numidiv(
                                nvalue!(rb as *const TValue),
                                nvalue!(rc as *const TValue)
                            )
                        );
                        continue 'dispatch;
                    } else if ttisvector!(rb as *const TValue) && ttisnumber!(rc as *const TValue) {
                        let vb = vvalue!(rb as *const TValue).as_ptr();
                        let vc = nvalue!(rc as *const TValue) as f32;
                        setvvalue!(
                            ra,
                            luai_numidiv(*vb.add(0) as f64, vc as f64) as f32,
                            luai_numidiv(*vb.add(1) as f64, vc as f64) as f32,
                            luai_numidiv(*vb.add(2) as f64, vc as f64) as f32,
                            luai_numidiv(*vb.add(3) as f64, vc as f64) as f32
                        );
                        continue 'dispatch;
                    } else {
                        // fast-path for userdata with C functions
                        let rbc = if ttisnumber!(rb as *const TValue) {
                            rc
                        } else {
                            rb
                        };
                        let mut fn_tm: *const TValue = core::ptr::null();
                        if ttisuserdata!(rbc as *const TValue)
                            && {
                                fn_tm = lua_t_gettmbyobj(L, rbc as *const TValue, TMS::TM_IDIV);
                                !fn_tm.is_null()
                            }
                            && ttisfunction!(fn_tm)
                            && (*clvalue!(fn_tm)).isC != 0
                        {
                            // note: it's safe to push arguments past top for
                            // complicated reasons (see top of the file)
                            LUAU_ASSERT!((*L).top.add(3) < (*L).stack.add((*L).stacksize as usize));
                            let top = (*L).top;
                            setobj_2_s!(L, top.add(0), fn_tm);
                            setobj_2_s!(L, top.add(1), rb as *const TValue);
                            setobj_2_s!(L, top.add(2), rc as *const TValue);
                            (*L).top = top.add(3);

                            vm_protect!(L, pc, base, {
                                lua_v_call_tm(L, 2, LUAU_INSN_A!(insn) as i32);
                            });
                            continue 'dispatch;
                        } else {
                            // slow-path, may invoke C/Lua via metamethods
                            vm_protect!(L, pc, base, {
                                lua_v_doarithimpl(
                                    L,
                                    ra,
                                    rb as *const TValue,
                                    rc as *const TValue,
                                    TMS::TM_IDIV,
                                );
                            });
                            continue 'dispatch;
                        }
                    }
                }
                LuauOpcode::LOP_MOD => {
                    // lvmexecute.cpp:2026
                    let insn = *pc;
                    pc = pc.add(1);
                    let ra = VM_REG!(LUAU_INSN_A!(insn), L, base) as *mut TValue;
                    let rb = VM_REG!(LUAU_INSN_B!(insn), L, base) as *mut TValue;
                    let rc = VM_REG!(LUAU_INSN_C!(insn), L, base) as *mut TValue;

                    // fast-path
                    if ttisnumber!(rb as *const TValue) && ttisnumber!(rc as *const TValue) {
                        let nb = nvalue!(rb as *const TValue);
                        let nc = nvalue!(rc as *const TValue);
                        setnvalue!(ra, luai_nummod(nb, nc));
                        continue 'dispatch;
                    } else {
                        // slow-path, may invoke C/Lua via metamethods
                        vm_protect!(L, pc, base, {
                            lua_v_doarithimpl(
                                L,
                                ra,
                                rb as *const TValue,
                                rc as *const TValue,
                                TMS::TM_MOD,
                            );
                        });
                        continue 'dispatch;
                    }
                }
                LuauOpcode::LOP_POW => {
                    // lvmexecute.cpp:2049
                    let insn = *pc;
                    pc = pc.add(1);
                    let ra = VM_REG!(LUAU_INSN_A!(insn), L, base) as *mut TValue;
                    let rb = VM_REG!(LUAU_INSN_B!(insn), L, base) as *mut TValue;
                    let rc = VM_REG!(LUAU_INSN_C!(insn), L, base) as *mut TValue;

                    // fast-path
                    if ttisnumber!(rb as *const TValue) && ttisnumber!(rc as *const TValue) {
                        setnvalue!(
                            ra,
                            nvalue!(rb as *const TValue).powf(nvalue!(rc as *const TValue))
                        );
                        continue 'dispatch;
                    } else {
                        // slow-path, may invoke C/Lua via metamethods
                        vm_protect!(L, pc, base, {
                            lua_v_doarithimpl(
                                L,
                                ra,
                                rb as *const TValue,
                                rc as *const TValue,
                                TMS::TM_POW,
                            );
                        });
                        continue 'dispatch;
                    }
                }
                LuauOpcode::LOP_ADDK => {
                    // lvmexecute.cpp:2070
                    let insn = *pc;
                    pc = pc.add(1);
                    let ra = VM_REG!(LUAU_INSN_A!(insn), L, base) as *mut TValue;
                    let rb = VM_REG!(LUAU_INSN_B!(insn), L, base) as *mut TValue;
                    let kv = VM_KV!(LUAU_INSN_C!(insn), cl, k) as *mut TValue;

                    // fast-path
                    if ttisnumber!(rb as *const TValue) {
                        setnvalue!(
                            ra,
                            nvalue!(rb as *const TValue) + nvalue!(kv as *const TValue)
                        );
                        continue 'dispatch;
                    } else {
                        // slow-path, may invoke C/Lua via metamethods
                        vm_protect!(L, pc, base, {
                            lua_v_doarithimpl(
                                L,
                                ra,
                                rb as *const TValue,
                                kv as *const TValue,
                                TMS::TM_ADD,
                            );
                        });
                        continue 'dispatch;
                    }
                }
                LuauOpcode::LOP_SUBK => {
                    // lvmexecute.cpp:2091
                    let insn = *pc;
                    pc = pc.add(1);
                    let ra = VM_REG!(LUAU_INSN_A!(insn), L, base) as *mut TValue;
                    let rb = VM_REG!(LUAU_INSN_B!(insn), L, base) as *mut TValue;
                    let kv = VM_KV!(LUAU_INSN_C!(insn), cl, k) as *mut TValue;

                    // fast-path
                    if ttisnumber!(rb as *const TValue) {
                        setnvalue!(
                            ra,
                            nvalue!(rb as *const TValue) - nvalue!(kv as *const TValue)
                        );
                        continue 'dispatch;
                    } else {
                        // slow-path, may invoke C/Lua via metamethods
                        vm_protect!(L, pc, base, {
                            lua_v_doarithimpl(
                                L,
                                ra,
                                rb as *const TValue,
                                kv as *const TValue,
                                TMS::TM_SUB,
                            );
                        });
                        continue 'dispatch;
                    }
                }
                LuauOpcode::LOP_MULK => {
                    // lvmexecute.cpp:2112
                    let insn = *pc;
                    pc = pc.add(1);
                    let ra = VM_REG!(LUAU_INSN_A!(insn), L, base) as *mut TValue;
                    let rb = VM_REG!(LUAU_INSN_B!(insn), L, base) as *mut TValue;
                    let kv = VM_KV!(LUAU_INSN_C!(insn), cl, k) as *mut TValue;

                    // fast-path
                    if ttisnumber!(rb as *const TValue) {
                        setnvalue!(
                            ra,
                            nvalue!(rb as *const TValue) * nvalue!(kv as *const TValue)
                        );
                        continue 'dispatch;
                    } else if ttisvector!(rb as *const TValue) {
                        let vb = vvalue!(rb as *const TValue).as_ptr();
                        let vc = nvalue!(kv as *const TValue) as f32;
                        setvvalue!(
                            ra,
                            *vb.add(0) * vc,
                            *vb.add(1) * vc,
                            *vb.add(2) * vc,
                            *vb.add(3) * vc
                        );
                        continue 'dispatch;
                    } else {
                        // fast-path for userdata with C functions
                        let mut fn_tm: *const TValue = core::ptr::null();
                        if ttisuserdata!(rb as *const TValue)
                            && {
                                fn_tm = lua_t_gettmbyobj(L, rb as *const TValue, TMS::TM_MUL);
                                !fn_tm.is_null()
                            }
                            && ttisfunction!(fn_tm)
                            && (*clvalue!(fn_tm)).isC != 0
                        {
                            // note: it's safe to push arguments past top for
                            // complicated reasons (see top of the file)
                            LUAU_ASSERT!((*L).top.add(3) < (*L).stack.add((*L).stacksize as usize));
                            let top = (*L).top;
                            setobj_2_s!(L, top.add(0), fn_tm);
                            setobj_2_s!(L, top.add(1), rb as *const TValue);
                            setobj_2_s!(L, top.add(2), kv as *const TValue);
                            (*L).top = top.add(3);

                            vm_protect!(L, pc, base, {
                                lua_v_call_tm(L, 2, LUAU_INSN_A!(insn) as i32);
                            });
                            continue 'dispatch;
                        } else {
                            // slow-path, may invoke C/Lua via metamethods
                            vm_protect!(L, pc, base, {
                                lua_v_doarithimpl(
                                    L,
                                    ra,
                                    rb as *const TValue,
                                    kv as *const TValue,
                                    TMS::TM_MUL,
                                );
                            });
                            continue 'dispatch;
                        }
                    }
                }
                LuauOpcode::LOP_DIVK => {
                    // lvmexecute.cpp:2158
                    let insn = *pc;
                    pc = pc.add(1);
                    let ra = VM_REG!(LUAU_INSN_A!(insn), L, base) as *mut TValue;
                    let rb = VM_REG!(LUAU_INSN_B!(insn), L, base) as *mut TValue;
                    let kv = VM_KV!(LUAU_INSN_C!(insn), cl, k) as *mut TValue;

                    // fast-path
                    if ttisnumber!(rb as *const TValue) {
                        setnvalue!(
                            ra,
                            nvalue!(rb as *const TValue) / nvalue!(kv as *const TValue)
                        );
                        continue 'dispatch;
                    } else if ttisvector!(rb as *const TValue) {
                        let vb = vvalue!(rb as *const TValue).as_ptr();
                        let vc = nvalue!(kv as *const TValue) as f32;
                        setvvalue!(
                            ra,
                            *vb.add(0) / vc,
                            *vb.add(1) / vc,
                            *vb.add(2) / vc,
                            *vb.add(3) / vc
                        );
                        continue 'dispatch;
                    } else {
                        // fast-path for userdata with C functions
                        let mut fn_tm: *const TValue = core::ptr::null();
                        if ttisuserdata!(rb as *const TValue)
                            && {
                                fn_tm = lua_t_gettmbyobj(L, rb as *const TValue, TMS::TM_DIV);
                                !fn_tm.is_null()
                            }
                            && ttisfunction!(fn_tm)
                            && (*clvalue!(fn_tm)).isC != 0
                        {
                            // note: it's safe to push arguments past top for
                            // complicated reasons (see top of the file)
                            LUAU_ASSERT!((*L).top.add(3) < (*L).stack.add((*L).stacksize as usize));
                            let top = (*L).top;
                            setobj_2_s!(L, top.add(0), fn_tm);
                            setobj_2_s!(L, top.add(1), rb as *const TValue);
                            setobj_2_s!(L, top.add(2), kv as *const TValue);
                            (*L).top = top.add(3);

                            vm_protect!(L, pc, base, {
                                lua_v_call_tm(L, 2, LUAU_INSN_A!(insn) as i32);
                            });
                            continue 'dispatch;
                        } else {
                            // slow-path, may invoke C/Lua via metamethods
                            vm_protect!(L, pc, base, {
                                lua_v_doarithimpl(
                                    L,
                                    ra,
                                    rb as *const TValue,
                                    kv as *const TValue,
                                    TMS::TM_DIV,
                                );
                            });
                            continue 'dispatch;
                        }
                    }
                }
                LuauOpcode::LOP_IDIVK => {
                    // lvmexecute.cpp:2204
                    let insn = *pc;
                    pc = pc.add(1);
                    let ra = VM_REG!(LUAU_INSN_A!(insn), L, base) as *mut TValue;
                    let rb = VM_REG!(LUAU_INSN_B!(insn), L, base) as *mut TValue;
                    let kv = VM_KV!(LUAU_INSN_C!(insn), cl, k) as *mut TValue;

                    // fast-path
                    if ttisnumber!(rb as *const TValue) {
                        setnvalue!(
                            ra,
                            luai_numidiv(
                                nvalue!(rb as *const TValue),
                                nvalue!(kv as *const TValue)
                            )
                        );
                        continue 'dispatch;
                    } else if ttisvector!(rb as *const TValue) {
                        let vb = vvalue!(rb as *const TValue).as_ptr();
                        let vc = nvalue!(kv as *const TValue) as f32;
                        setvvalue!(
                            ra,
                            luai_numidiv(*vb.add(0) as f64, vc as f64) as f32,
                            luai_numidiv(*vb.add(1) as f64, vc as f64) as f32,
                            luai_numidiv(*vb.add(2) as f64, vc as f64) as f32,
                            luai_numidiv(*vb.add(3) as f64, vc as f64) as f32
                        );
                        continue 'dispatch;
                    } else {
                        // fast-path for userdata with C functions
                        let mut fn_tm: *const TValue = core::ptr::null();
                        if ttisuserdata!(rb as *const TValue)
                            && {
                                fn_tm = lua_t_gettmbyobj(L, rb as *const TValue, TMS::TM_IDIV);
                                !fn_tm.is_null()
                            }
                            && ttisfunction!(fn_tm)
                            && (*clvalue!(fn_tm)).isC != 0
                        {
                            // note: it's safe to push arguments past top for
                            // complicated reasons (see top of the file)
                            LUAU_ASSERT!((*L).top.add(3) < (*L).stack.add((*L).stacksize as usize));
                            let top = (*L).top;
                            setobj_2_s!(L, top.add(0), fn_tm);
                            setobj_2_s!(L, top.add(1), rb as *const TValue);
                            setobj_2_s!(L, top.add(2), kv as *const TValue);
                            (*L).top = top.add(3);

                            vm_protect!(L, pc, base, {
                                lua_v_call_tm(L, 2, LUAU_INSN_A!(insn) as i32);
                            });
                            continue 'dispatch;
                        } else {
                            // slow-path, may invoke C/Lua via metamethods
                            vm_protect!(L, pc, base, {
                                lua_v_doarithimpl(
                                    L,
                                    ra,
                                    rb as *const TValue,
                                    kv as *const TValue,
                                    TMS::TM_IDIV,
                                );
                            });
                            continue 'dispatch;
                        }
                    }
                }
                LuauOpcode::LOP_MODK => {
                    // lvmexecute.cpp:2256
                    let insn = *pc;
                    pc = pc.add(1);
                    let ra = VM_REG!(LUAU_INSN_A!(insn), L, base) as *mut TValue;
                    let rb = VM_REG!(LUAU_INSN_B!(insn), L, base) as *mut TValue;
                    let kv = VM_KV!(LUAU_INSN_C!(insn), cl, k) as *mut TValue;

                    // fast-path
                    if ttisnumber!(rb as *const TValue) {
                        setnvalue!(
                            ra,
                            luai_nummod(nvalue!(rb as *const TValue), nvalue!(kv as *const TValue))
                        );
                        continue 'dispatch;
                    } else {
                        // slow-path, may invoke C/Lua via metamethods
                        vm_protect!(L, pc, base, {
                            lua_v_doarithimpl(
                                L,
                                ra,
                                rb as *const TValue,
                                kv as *const TValue,
                                TMS::TM_MOD,
                            );
                        });
                        continue 'dispatch;
                    }
                }
                LuauOpcode::LOP_POWK => {
                    // lvmexecute.cpp:2279
                    let insn = *pc;
                    pc = pc.add(1);
                    let ra = VM_REG!(LUAU_INSN_A!(insn), L, base) as *mut TValue;
                    let rb = VM_REG!(LUAU_INSN_B!(insn), L, base) as *mut TValue;
                    let kv = VM_KV!(LUAU_INSN_C!(insn), cl, k) as *mut TValue;

                    // fast-path
                    if ttisnumber!(rb as *const TValue) {
                        let nb = nvalue!(rb as *const TValue);
                        let nk = nvalue!(kv as *const TValue);

                        // pow is very slow so we specialize this for ^2, ^0.5 and ^3
                        let r = if nk == 2.0 {
                            nb * nb
                        } else if nk == 0.5 {
                            nb.sqrt()
                        } else if nk == 3.0 {
                            nb * nb * nb
                        } else {
                            nb.powf(nk)
                        };

                        setnvalue!(ra, r);
                        continue 'dispatch;
                    } else {
                        // slow-path, may invoke C/Lua via metamethods
                        vm_protect!(L, pc, base, {
                            lua_v_doarithimpl(
                                L,
                                ra,
                                rb as *const TValue,
                                kv as *const TValue,
                                TMS::TM_POW,
                            );
                        });
                        continue 'dispatch;
                    }
                }
                LuauOpcode::LOP_AND => {
                    // lvmexecute.cpp:2306
                    let insn = *pc;
                    pc = pc.add(1);
                    let ra = VM_REG!(LUAU_INSN_A!(insn), L, base) as *mut TValue;
                    let rb = VM_REG!(LUAU_INSN_B!(insn), L, base) as *mut TValue;
                    let rc = VM_REG!(LUAU_INSN_C!(insn), L, base) as *mut TValue;

                    setobj_2_s!(
                        L,
                        ra,
                        if l_isfalse!(rb as *const TValue) {
                            rb
                        } else {
                            rc
                        } as *const TValue
                    );
                    continue 'dispatch;
                }
                LuauOpcode::LOP_OR => {
                    // lvmexecute.cpp:2317
                    let insn = *pc;
                    pc = pc.add(1);
                    let ra = VM_REG!(LUAU_INSN_A!(insn), L, base) as *mut TValue;
                    let rb = VM_REG!(LUAU_INSN_B!(insn), L, base) as *mut TValue;
                    let rc = VM_REG!(LUAU_INSN_C!(insn), L, base) as *mut TValue;

                    setobj_2_s!(
                        L,
                        ra,
                        if l_isfalse!(rb as *const TValue) {
                            rc
                        } else {
                            rb
                        } as *const TValue
                    );
                    continue 'dispatch;
                }
                LuauOpcode::LOP_ANDK => {
                    // lvmexecute.cpp:2328
                    let insn = *pc;
                    pc = pc.add(1);
                    let ra = VM_REG!(LUAU_INSN_A!(insn), L, base) as *mut TValue;
                    let rb = VM_REG!(LUAU_INSN_B!(insn), L, base) as *mut TValue;
                    let kv = VM_KV!(LUAU_INSN_C!(insn), cl, k) as *mut TValue;

                    setobj_2_s!(
                        L,
                        ra,
                        if l_isfalse!(rb as *const TValue) {
                            rb
                        } else {
                            kv
                        } as *const TValue
                    );
                    continue 'dispatch;
                }
                LuauOpcode::LOP_ORK => {
                    // lvmexecute.cpp:2339
                    let insn = *pc;
                    pc = pc.add(1);
                    let ra = VM_REG!(LUAU_INSN_A!(insn), L, base) as *mut TValue;
                    let rb = VM_REG!(LUAU_INSN_B!(insn), L, base) as *mut TValue;
                    let kv = VM_KV!(LUAU_INSN_C!(insn), cl, k) as *mut TValue;

                    setobj_2_s!(
                        L,
                        ra,
                        if l_isfalse!(rb as *const TValue) {
                            kv
                        } else {
                            rb
                        } as *const TValue
                    );
                    continue 'dispatch;
                }
                LuauOpcode::LOP_CONCAT => {
                    // lvmexecute.cpp:2350
                    let insn = *pc;
                    pc = pc.add(1);
                    let b = LUAU_INSN_B!(insn) as i32;
                    let c = LUAU_INSN_C!(insn) as i32;

                    // This call may realloc the stack! So we need to query args further down
                    vm_protect!(L, pc, base, {
                        lua_v_concat(L, c - b + 1, c);
                    });

                    let ra = VM_REG!(LUAU_INSN_A!(insn), L, base) as *mut TValue;

                    setobj_2_s!(L, ra, base.add(b as usize) as *const TValue);
                    vm_protect!(L, pc, base, {
                        luaC_checkGC!(L);
                    });
                    continue 'dispatch;
                }
                LuauOpcode::LOP_NOT => {
                    // lvmexecute.cpp:2377
                    let insn = *pc;
                    pc = pc.add(1);
                    let ra = VM_REG!(LUAU_INSN_A!(insn), L, base) as *mut TValue;
                    let rb = VM_REG!(LUAU_INSN_B!(insn), L, base) as *mut TValue;

                    let res = l_isfalse!(rb as *const TValue) as i32;
                    setbvalue!(ra, res);
                    continue 'dispatch;
                }
                LuauOpcode::LOP_MINUS => {
                    // lvmexecute.cpp:2420
                    let insn = *pc;
                    pc = pc.add(1);
                    let ra = VM_REG!(LUAU_INSN_A!(insn), L, base) as *mut TValue;
                    let rb = VM_REG!(LUAU_INSN_B!(insn), L, base) as *mut TValue;

                    // fast-path
                    if ttisnumber!(rb as *const TValue) {
                        setnvalue!(ra, -nvalue!(rb as *const TValue));
                        continue 'dispatch;
                    } else if ttisvector!(rb as *const TValue) {
                        let vb = vvalue!(rb as *const TValue).as_ptr();
                        setvvalue!(ra, -*vb.add(0), -*vb.add(1), -*vb.add(2), -*vb.add(3));
                        continue 'dispatch;
                    } else {
                        // fast-path for userdata with C functions
                        let mut fn_tm: *const TValue = core::ptr::null();
                        if ttisuserdata!(rb as *const TValue)
                            && {
                                fn_tm = lua_t_gettmbyobj(L, rb as *const TValue, TMS::TM_UNM);
                                !fn_tm.is_null()
                            }
                            && ttisfunction!(fn_tm)
                            && (*clvalue!(fn_tm)).isC != 0
                        {
                            // note: it's safe to push arguments past top for
                            // complicated reasons (see top of the file)
                            LUAU_ASSERT!((*L).top.add(2) < (*L).stack.add((*L).stacksize as usize));
                            let top = (*L).top;
                            setobj_2_s!(L, top.add(0), fn_tm);
                            setobj_2_s!(L, top.add(1), rb as *const TValue);
                            (*L).top = top.add(2);

                            vm_protect!(L, pc, base, {
                                lua_v_call_tm(L, 1, LUAU_INSN_A!(insn) as i32);
                            });
                            continue 'dispatch;
                        } else {
                            // slow-path, may invoke C/Lua via metamethods
                            vm_protect!(L, pc, base, {
                                lua_v_doarithimpl(
                                    L,
                                    ra,
                                    rb as *const TValue,
                                    rb as *const TValue,
                                    TMS::TM_UNM,
                                );
                            });
                            continue 'dispatch;
                        }
                    }
                }
                LuauOpcode::LOP_LENGTH => {
                    // lvmexecute.cpp:2420 (LOP_LENGTH)
                    let insn = *pc;
                    pc = pc.add(1);
                    let ra = VM_REG!(LUAU_INSN_A!(insn), L, base) as *mut TValue;
                    let rb = VM_REG!(LUAU_INSN_B!(insn), L, base) as *mut TValue;

                    // fast-path #1: tables
                    if ttistable!(rb as *const TValue) {
                        let h = hvalue!(rb as *const TValue);

                        if fastnotm((*h).metatable, TMS::TM_LEN as i32) {
                            setnvalue!(ra, lua_h_getn(h) as f64);
                            continue 'dispatch;
                        } else {
                            // slow-path, may invoke C/Lua via metamethods
                            vm_protect!(L, pc, base, {
                                lua_v_dolen(L, ra, rb as *const TValue);
                            });
                            continue 'dispatch;
                        }
                    }
                    // fast-path #2: strings (not very important but easy to do)
                    else if ttisstring!(rb as *const TValue) {
                        let ts = tsvalue!(rb as *const TValue);
                        setnvalue!(ra, (*ts).len as f64);
                        continue 'dispatch;
                    } else {
                        // slow-path, may invoke C/Lua via metamethods
                        vm_protect!(L, pc, base, {
                            lua_v_dolen(L, ra, rb as *const TValue);
                        });
                        continue 'dispatch;
                    }
                }
                LuauOpcode::LOP_NEWTABLE => {
                    // lvmexecute.cpp:2458
                    let insn = *pc;
                    pc = pc.add(1);
                    let ra = VM_REG!(LUAU_INSN_A!(insn), L, base) as *mut TValue;
                    let b = LUAU_INSN_B!(insn) as i32;
                    let aux: u32 = *pc;
                    pc = pc.add(1);

                    (*(*L).ci).savedpc = pc; // VM_PROTECT_PC(): luaH_new may fail due to OOM

                    sethvalue!(
                        L,
                        ra,
                        lua_h_new(L, aux as i32, if b == 0 { 0 } else { 1 << (b - 1) })
                    );
                    vm_protect!(L, pc, base, {
                        luaC_checkGC!(L);
                    });
                    continue 'dispatch;
                }
                LuauOpcode::LOP_DUPTABLE => {
                    // lvmexecute.cpp:2472
                    let insn = *pc;
                    pc = pc.add(1);
                    let ra = VM_REG!(LUAU_INSN_A!(insn), L, base) as *mut TValue;
                    let kv = VM_KV!(LUAU_INSN_D!(insn), cl, k) as *mut TValue;

                    (*(*L).ci).savedpc = pc; // VM_PROTECT_PC(): luaH_clone may fail due to OOM

                    sethvalue!(L, ra, lua_h_clone(L, hvalue!(kv as *const TValue)));
                    vm_protect!(L, pc, base, {
                        luaC_checkGC!(L);
                    });
                    continue 'dispatch;
                }
                LuauOpcode::LOP_SETLIST => {
                    // lvmexecute.cpp:2485
                    let insn = *pc;
                    pc = pc.add(1);
                    let ra = VM_REG!(LUAU_INSN_A!(insn), L, base) as *mut TValue;
                    // note: this can point to L->top if c == LUA_MULTRET making VM_REG unsafe to use
                    let rb: StkId = base.add(LUAU_INSN_B!(insn) as usize);
                    let mut c = LUAU_INSN_C!(insn) as i32 - 1;
                    let index: u32 = *pc;
                    pc = pc.add(1);

                    if c == LUA_MULTRET {
                        c = (*L).top.offset_from(rb) as i32;
                        (*L).top = (*(*L).ci).top;
                    }

                    let h = hvalue!(ra as *const TValue);

                    // TODO: we really don't need this anymore
                    if !ttistable!(ra as *const TValue) {
                        // temporary workaround to weaken a rather powerful exploitation
                        // primitive in case of a MITM attack on bytecode
                        return;
                    }

                    let last = index as i32 + c - 1;
                    if last > (*h).sizearray {
                        (*(*L).ci).savedpc = pc; // VM_PROTECT_PC(): luaH_resizearray may fail due to OOM

                        lua_h_resizearray(L, h, last);
                    }

                    let array = (*h).array;

                    for i in 0..c {
                        setobj2t!(
                            L,
                            array.add((index as i32 + i - 1) as usize),
                            rb.add(i as usize) as *const TValue
                        );
                    }

                    luaC_barrierfast!(L, h);
                    continue 'dispatch;
                }
                LuauOpcode::LOP_FORNPREP => {
                    // lvmexecute.cpp:2546
                    let insn = *pc;
                    pc = pc.add(1);
                    let ra = VM_REG!(LUAU_INSN_A!(insn), L, base) as *mut TValue;

                    if !ttisnumber!(ra.add(0) as *const TValue)
                        || !ttisnumber!(ra.add(1) as *const TValue)
                        || !ttisnumber!(ra.add(2) as *const TValue)
                    {
                        // slow-path: can convert arguments to numbers and trigger Lua errors
                        // Note: this doesn't reallocate stack so we don't need to recompute ra/base
                        (*(*L).ci).savedpc = pc; // VM_PROTECT_PC()

                        lua_v_prepare_forn(L, ra.add(0), ra.add(1), ra.add(2));
                    }

                    let limit = nvalue!(ra.add(0) as *const TValue);
                    let step = nvalue!(ra.add(1) as *const TValue);
                    let idx = nvalue!(ra.add(2) as *const TValue);

                    // Note: make sure the loop condition is exactly the same between
                    // this and LOP_FORNLOOP so that we handle NaN/etc. consistently
                    pc = pc.offset(
                        if if step > 0.0 {
                            idx <= limit
                        } else {
                            limit <= idx
                        } {
                            0
                        } else {
                            LUAU_INSN_D!(insn) as isize
                        },
                    );
                    let p = {
                        let l = &(*cl).inner.l;
                        l.p
                    };
                    LUAU_ASSERT!((pc.offset_from((*p).code) as u32) < (*p).sizecode as u32);
                    continue 'dispatch;
                }
                LuauOpcode::LOP_FORNLOOP => {
                    // lvmexecute.cpp:2573
                    VM_INTERRUPT!(L, pc, base);
                    let insn = *pc;
                    pc = pc.add(1);
                    let ra = VM_REG!(LUAU_INSN_A!(insn), L, base) as *mut TValue;
                    LUAU_ASSERT!(
                        ttisnumber!(ra.add(0) as *const TValue)
                            && ttisnumber!(ra.add(1) as *const TValue)
                            && ttisnumber!(ra.add(2) as *const TValue)
                    );

                    let limit = nvalue!(ra.add(0) as *const TValue);
                    let step = nvalue!(ra.add(1) as *const TValue);
                    let idx = nvalue!(ra.add(2) as *const TValue) + step;

                    setnvalue!(ra.add(2), idx);

                    // Note: make sure the loop condition is exactly the same between
                    // this and LOP_FORNPREP so that we handle NaN/etc. consistently
                    if if step > 0.0 {
                        idx <= limit
                    } else {
                        limit <= idx
                    } {
                        pc = pc.offset(LUAU_INSN_D!(insn) as isize);
                        let p = {
                            let l = &(*cl).inner.l;
                            l.p
                        };
                        LUAU_ASSERT!((pc.offset_from((*p).code) as u32) < (*p).sizecode as u32);
                        continue 'dispatch;
                    } else {
                        // fallthrough to exit
                        continue 'dispatch;
                    }
                }
                LuauOpcode::LOP_FORGPREP => {
                    // lvmexecute.cpp:2695
                    let insn = *pc;
                    pc = pc.add(1);
                    let mut ra = VM_REG!(LUAU_INSN_A!(insn), L, base) as *mut TValue;

                    if luaur_common::FFlag::DebugLuauUserDefinedClassesRuntime.get() {
                        // If this is a function it will be called during FORGLOOP
                        if !ttisfunction!(ra as *const TValue) {
                            let mt = if ttistable!(ra as *const TValue) {
                                (*hvalue!(ra as *const TValue)).metatable
                            } else if ttisuserdata!(ra as *const TValue) {
                                (*uvalue!(ra as *const TValue)).metatable
                            } else {
                                core::ptr::null_mut()
                            };
                            let mut fn_tm = fasttm(L, mt, TMS::TM_ITER as i32);

                            if fn_tm.is_null() && ttisobject!(ra as *const TValue) {
                                fn_tm = lua_t_gettmbyobj(L, ra as *const TValue, TMS::TM_ITER);
                                // if the metamethod is not present, error.
                                if ttisnil!(fn_tm) {
                                    (*(*L).ci).savedpc = pc; // VM_PROTECT_PC(): next call always errors
                                    luaG_typeerrorL(
                                        L,
                                        ra as *const TValue,
                                        b"iterate over\0".as_ptr() as *const core::ffi::c_char,
                                    );
                                }
                            }

                            if !fn_tm.is_null() {
                                setobj_2_s!(L, ra.add(1), ra as *const TValue);
                                setobj_2_s!(L, ra, fn_tm);

                                (*L).top = ra.add(2); // func + self arg
                                LUAU_ASSERT!((*L).top <= (*L).stack_last);

                                vm_protect!(L, pc, base, {
                                    lua_d_call(L, ra, 3);
                                });
                                (*L).top = (*(*L).ci).top;

                                // recompute ra since stack might have been reallocated
                                ra = VM_REG!(LUAU_INSN_A!(insn), L, base) as *mut TValue;

                                // protect against __iter returning nil, since nil is used
                                // as a marker for builtin iteration in FORGLOOP
                                if ttisnil!(ra as *const TValue) {
                                    (*(*L).ci).savedpc = pc; // VM_PROTECT_PC(): next call always errors
                                    luaG_typeerrorL(
                                        L,
                                        ra as *const TValue,
                                        b"call\0".as_ptr() as *const core::ffi::c_char,
                                    );
                                }
                            } else if !fasttm(L, mt, TMS::TM_CALL as i32).is_null() {
                                // table or userdata with __call, will be called during FORGLOOP
                                // TODO: we might be able to stop supporting this depending
                                // on whether it's used in practice
                            } else if ttistable!(ra as *const TValue) {
                                // set up registers for builtin iteration
                                setobj_2_s!(L, ra.add(1), ra as *const TValue);
                                setpvalue!(
                                    ra.add(2),
                                    0usize as *mut core::ffi::c_void,
                                    LU_TAG_ITERATOR
                                );
                                setnilvalue!(ra);
                            } else {
                                (*(*L).ci).savedpc = pc; // VM_PROTECT_PC(): next call always errors
                                luaG_typeerrorL(
                                    L,
                                    ra as *const TValue,
                                    b"iterate over\0".as_ptr() as *const core::ffi::c_char,
                                );
                            }
                        }
                    } else {
                        if ttisfunction!(ra as *const TValue) {
                            // will be called during FORGLOOP
                        } else {
                            let mt = if ttistable!(ra as *const TValue) {
                                (*hvalue!(ra as *const TValue)).metatable
                            } else if ttisuserdata!(ra as *const TValue) {
                                (*uvalue!(ra as *const TValue)).metatable
                            } else {
                                core::ptr::null_mut()
                            };

                            let fn_tm = fasttm(L, mt, TMS::TM_ITER as i32);
                            if !fn_tm.is_null() {
                                setobj_2_s!(L, ra.add(1), ra as *const TValue);
                                setobj_2_s!(L, ra, fn_tm);

                                (*L).top = ra.add(2); // func + self arg
                                LUAU_ASSERT!((*L).top <= (*L).stack_last);

                                vm_protect!(L, pc, base, {
                                    lua_d_call(L, ra, 3);
                                });
                                (*L).top = (*(*L).ci).top;

                                // recompute ra since stack might have been reallocated
                                ra = VM_REG!(LUAU_INSN_A!(insn), L, base) as *mut TValue;

                                // protect against __iter returning nil, since nil is used
                                // as a marker for builtin iteration in FORGLOOP
                                if ttisnil!(ra as *const TValue) {
                                    (*(*L).ci).savedpc = pc; // VM_PROTECT_PC(): next call always errors
                                    luaG_typeerrorL(
                                        L,
                                        ra as *const TValue,
                                        b"call\0".as_ptr() as *const core::ffi::c_char,
                                    );
                                }
                            } else if !fasttm(L, mt, TMS::TM_CALL as i32).is_null() {
                                // table or userdata with __call, will be called during FORGLOOP
                                // TODO: we might be able to stop supporting this depending
                                // on whether it's used in practice
                            } else if ttistable!(ra as *const TValue) {
                                // set up registers for builtin iteration
                                setobj_2_s!(L, ra.add(1), ra as *const TValue);
                                setpvalue!(
                                    ra.add(2),
                                    0usize as *mut core::ffi::c_void,
                                    LU_TAG_ITERATOR
                                );
                                setnilvalue!(ra);
                            } else {
                                (*(*L).ci).savedpc = pc; // VM_PROTECT_PC(): next call always errors
                                luaG_typeerrorL(
                                    L,
                                    ra as *const TValue,
                                    b"iterate over\0".as_ptr() as *const core::ffi::c_char,
                                );
                            }
                        }
                    }

                    pc = pc.offset(LUAU_INSN_D!(insn) as isize);
                    let p = {
                        let l = &(*cl).inner.l;
                        l.p
                    };
                    LUAU_ASSERT!((pc.offset_from((*p).code) as u32) < (*p).sizecode as u32);
                    continue 'dispatch;
                }
                LuauOpcode::LOP_FORGLOOP => {
                    // lvmexecute.cpp:2807
                    VM_INTERRUPT!(L, pc, base);
                    let insn = *pc;
                    pc = pc.add(1);
                    let mut ra = VM_REG!(LUAU_INSN_A!(insn), L, base) as *mut TValue;
                    let aux: u32 = *pc;

                    // fast-path: builtin table iteration
                    // note: ra=nil guarantees ra+1=table and ra+2=userdata because of
                    // the setup by FORGPREP* opcodes
                    // TODO: remove the table check per guarantee above
                    if ttisnil!(ra as *const TValue) && ttistable!(ra.add(1) as *const TValue) {
                        let h = hvalue!(ra.add(1) as *const TValue);
                        let mut index = pvalue!(ra.add(2) as *const TValue) as usize as i32;

                        let sizearray = (*h).sizearray;

                        // clear extra variables since we might have more than two
                        // note: while aux encodes ipairs bit, when set we always use 2
                        // variables, so it's safe to check this via a signed comparison
                        if (aux as i32) > 2 {
                            for i in 2..aux as i32 {
                                setnilvalue!(ra.add(3 + i as usize));
                            }
                        }

                        // terminate ipairs-style traversal early when encountering nil
                        if (aux as i32) < 0
                            && (index as u32 >= sizearray as u32
                                || ttisnil!((*h).array.add(index as usize) as *const TValue))
                        {
                            pc = pc.add(1);
                            continue 'dispatch;
                        }

                        // first we advance index through the array portion
                        while (index as u32) < sizearray as u32 {
                            let e = (*h).array.add(index as usize);

                            if !ttisnil!(e as *const TValue) {
                                setpvalue!(
                                    ra.add(2),
                                    (index + 1) as usize as *mut core::ffi::c_void,
                                    LU_TAG_ITERATOR
                                );
                                setnvalue!(ra.add(3), (index + 1) as f64);
                                setobj_2_s!(L, ra.add(4), e as *const TValue);

                                pc = pc.offset(LUAU_INSN_D!(insn) as isize);
                                let p = {
                                    let l = &(*cl).inner.l;
                                    l.p
                                };
                                LUAU_ASSERT!(
                                    (pc.offset_from((*p).code) as u32) < (*p).sizecode as u32
                                );
                                continue 'dispatch;
                            }

                            index += 1;
                        }

                        let sizenode = 1i32 << (*h).lsizenode;

                        // then we advance index through the hash portion
                        while ((index - sizearray) as u32) < sizenode as u32 {
                            let n = (*h).node.add((index - sizearray) as usize);

                            if !ttisnil!(gval!(n) as *const TValue) {
                                setpvalue!(
                                    ra.add(2),
                                    (index + 1) as usize as *mut core::ffi::c_void,
                                    LU_TAG_ITERATOR
                                );
                                getnodekey!(L, ra.add(3), n);
                                setobj_2_s!(L, ra.add(4), gval!(n) as *const TValue);

                                pc = pc.offset(LUAU_INSN_D!(insn) as isize);
                                let p = {
                                    let l = &(*cl).inner.l;
                                    l.p
                                };
                                LUAU_ASSERT!(
                                    (pc.offset_from((*p).code) as u32) < (*p).sizecode as u32
                                );
                                continue 'dispatch;
                            }

                            index += 1;
                        }

                        // fallthrough to exit
                        pc = pc.add(1);
                        continue 'dispatch;
                    } else {
                        // note: it's safe to push arguments past top for complicated
                        // reasons (see top of the file)
                        setobj_2_s!(L, ra.add(3 + 2), ra.add(2) as *const TValue);
                        setobj_2_s!(L, ra.add(3 + 1), ra.add(1) as *const TValue);
                        setobj_2_s!(L, ra.add(3), ra as *const TValue);

                        (*L).top = ra.add(3 + 3); // func + 2 args (state and index)
                        LUAU_ASSERT!((*L).top <= (*L).stack_last);

                        if luaur_common::FFlag::LuauYieldIter2.get() {
                            let mut yielded = false;
                            vm_protect!(L, pc, base, {
                                yielded = lua_d_performcally(L, ra.add(3), aux as u8 as i32);
                            });

                            if yielded {
                                return; // goto exit
                            }
                        } else {
                            vm_protect!(L, pc, base, {
                                lua_d_call(L, ra.add(3), aux as u8 as i32);
                            });
                        }

                        (*L).top = (*(*L).ci).top;

                        // recompute ra since stack might have been reallocated
                        ra = VM_REG!(LUAU_INSN_A!(insn), L, base) as *mut TValue;

                        // copy first variable back into the iteration index
                        setobj_2_s!(L, ra.add(2), ra.add(3) as *const TValue);

                        // note that we need to increment pc by 1 to exit the loop since
                        // we need to skip over aux
                        pc = pc.offset(if ttisnil!(ra.add(3) as *const TValue) {
                            1
                        } else {
                            LUAU_INSN_D!(insn) as isize
                        });
                        let p = {
                            let l = &(*cl).inner.l;
                            l.p
                        };
                        LUAU_ASSERT!((pc.offset_from((*p).code) as u32) < (*p).sizecode as u32);
                        continue 'dispatch;
                    }
                }
                LuauOpcode::LOP_FORGPREP_INEXT => {
                    // lvmexecute.cpp:2830
                    let insn = *pc;
                    pc = pc.add(1);
                    let ra = VM_REG!(LUAU_INSN_A!(insn), L, base) as *mut TValue;

                    // fast-path: ipairs/inext
                    if (*(*cl).env).safeenv != 0
                        && ttistable!(ra.add(1) as *const TValue)
                        && ttisnumber!(ra.add(2) as *const TValue)
                        && nvalue!(ra.add(2) as *const TValue) == 0.0
                    {
                        setnilvalue!(ra);
                        // ra+1 is already the table
                        setpvalue!(ra.add(2), 0usize as *mut core::ffi::c_void, LU_TAG_ITERATOR);
                    } else if !ttisfunction!(ra as *const TValue) {
                        (*(*L).ci).savedpc = pc; // VM_PROTECT_PC(): next call always errors
                        luaG_typeerrorL(
                            L,
                            ra as *const TValue,
                            b"iterate over\0".as_ptr() as *const core::ffi::c_char,
                        );
                    }

                    pc = pc.offset(LUAU_INSN_D!(insn) as isize);
                    let p = {
                        let l = &(*cl).inner.l;
                        l.p
                    };
                    LUAU_ASSERT!((pc.offset_from((*p).code) as u32) < (*p).sizecode as u32);
                    continue 'dispatch;
                }
                LuauOpcode::LOP_FORGPREP_NEXT => {
                    // lvmexecute.cpp:2853
                    let insn = *pc;
                    pc = pc.add(1);
                    let ra = VM_REG!(LUAU_INSN_A!(insn), L, base) as *mut TValue;

                    // fast-path: pairs/next
                    if (*(*cl).env).safeenv != 0
                        && ttistable!(ra.add(1) as *const TValue)
                        && ttisnil!(ra.add(2) as *const TValue)
                    {
                        setnilvalue!(ra);
                        // ra+1 is already the table
                        setpvalue!(ra.add(2), 0usize as *mut core::ffi::c_void, LU_TAG_ITERATOR);
                    } else if !ttisfunction!(ra as *const TValue) {
                        (*(*L).ci).savedpc = pc; // VM_PROTECT_PC(): next call always errors
                        luaG_typeerrorL(
                            L,
                            ra as *const TValue,
                            b"iterate over\0".as_ptr() as *const core::ffi::c_char,
                        );
                    }

                    pc = pc.offset(LUAU_INSN_D!(insn) as isize);
                    let p = {
                        let l = &(*cl).inner.l;
                        l.p
                    };
                    LUAU_ASSERT!((pc.offset_from((*p).code) as u32) < (*p).sizecode as u32);
                    continue 'dispatch;
                }
                LuauOpcode::LOP_NATIVECALL => {
                    // lvmexecute.cpp:2873
                    let p = {
                        let l = &(*cl).inner.l;
                        l.p
                    };
                    LUAU_ASSERT!(!(*p).execdata.is_null());

                    let ci = (*L).ci;
                    (*ci).flags = LUA_CALLINFO_NATIVE as u32;
                    (*ci).savedpc = (*p).code;

                    // VM_HAS_NATIVE
                    if let Some(enter) = (*(*L).global).ecb.enter {
                        if enter(L, p) == 1 {
                            continue 'reentry; // goto reentry
                        } else {
                            return; // goto exit
                        }
                    }
                    // (no native entry callback installed)
                    return;
                }
                LuauOpcode::LOP_GETVARARGS => {
                    // lvmexecute.cpp:2902
                    let insn = *pc;
                    pc = pc.add(1);
                    let b = LUAU_INSN_B!(insn) as i32 - 1;
                    let n = {
                        let l = &(*cl).inner.l;
                        base.offset_from((*(*L).ci).func) as i32 - (*l.p).numparams as i32 - 1
                    };

                    if b == LUA_MULTRET {
                        vm_protect!(L, pc, base, {
                            luaD_checkstack!(L, n);
                        });
                        // previous call may change the stack
                        let ra = VM_REG!(LUAU_INSN_A!(insn), L, base) as *mut TValue;

                        for j in 0..n {
                            setobj_2_s!(
                                L,
                                ra.add(j as usize),
                                base.sub(n as usize).add(j as usize) as *const TValue
                            );
                        }

                        (*L).top = ra.add(n as usize);
                        continue 'dispatch;
                    } else {
                        let ra = VM_REG!(LUAU_INSN_A!(insn), L, base) as *mut TValue;

                        let mut j = 0;
                        while j < b && j < n {
                            setobj_2_s!(
                                L,
                                ra.add(j as usize),
                                base.sub(n as usize).add(j as usize) as *const TValue
                            );
                            j += 1;
                        }
                        let mut j = n;
                        while j < b {
                            setnilvalue!(ra.add(j as usize));
                            j += 1;
                        }
                        continue 'dispatch;
                    }
                }
                LuauOpcode::LOP_DUPCLOSURE => {
                    // lvmexecute.cpp:2959
                    let insn = *pc;
                    pc = pc.add(1);
                    let ra = VM_REG!(LUAU_INSN_A!(insn), L, base) as *mut TValue;
                    let kv = VM_KV!(LUAU_INSN_D!(insn), cl, k) as *mut TValue;

                    let kcl = clvalue!(kv as *const TValue);

                    (*(*L).ci).savedpc = pc; // VM_PROTECT_PC(): luaF_newLclosure may fail due to OOM

                    // clone closure if the environment is not shared
                    // note: we save closure to stack early in case the code below
                    // wants to capture it by value
                    let mut ncl = if (*kcl).env == (*cl).env {
                        kcl
                    } else {
                        let kp = {
                            let l = &(*kcl).inner.l;
                            l.p
                        };
                        lua_f_new_lclosure(L, (*kcl).nupvalues as i32, (*cl).env, kp)
                    };
                    setclvalue!(L, ra, ncl);

                    // this loop does three things:
                    // - if the closure was created anew, it just fills it with upvalues
                    // - if the closure from the constant table is used, it fills it with
                    //   upvalues so that it can be shared in the future
                    // - if the closure is reused, it checks if the reuse is safe via
                    //   rawequal, and falls back to duplicating the closure
                    // (C++ restarts the loop with ui = -1 on lazy clone)
                    let mut ui: i32 = 0;
                    while ui < (*kcl).nupvalues as i32 {
                        let uinsn = *pc.add(ui as usize);
                        LUAU_ASSERT!(LUAU_INSN_OP!(uinsn) == LuauOpcode::LOP_CAPTURE as u32);
                        LUAU_ASSERT!(
                            LUAU_INSN_A!(uinsn) == LuauCaptureType::LCT_VAL as u32
                                || LUAU_INSN_A!(uinsn) == LuauCaptureType::LCT_UPVAL as u32
                        );

                        let uv: *mut TValue =
                            if LUAU_INSN_A!(uinsn) == LuauCaptureType::LCT_VAL as u32 {
                                VM_REG!(LUAU_INSN_B!(uinsn), L, base) as *mut TValue
                            } else {
                                VM_UV!(LUAU_INSN_B!(uinsn), cl) as *mut TValue
                            };

                        let uref = {
                            let l = &mut (*ncl).inner.l;
                            l.uprefs.as_mut_ptr().add(ui as usize)
                        };

                        // check if the existing closure is safe to reuse
                        if ncl == kcl
                            && luaO_rawequalObj(uref as *const TValue, uv as *const TValue) != 0
                        {
                            ui += 1;
                            continue;
                        }

                        // lazily clone the closure and update the upvalues
                        if ncl == kcl && (*kcl).preload == 0 {
                            let kp = {
                                let l = &(*kcl).inner.l;
                                l.p
                            };
                            ncl = lua_f_new_lclosure(L, (*kcl).nupvalues as i32, (*cl).env, kp);
                            setclvalue!(L, ra, ncl);

                            ui = 0; // C++ `ui = -1; continue` — restart the loop to fill all upvalues
                            continue;
                        }

                        // this updates a newly created closure, or an existing closure
                        // created during preload, in which case we need a barrier
                        setobj!(L, uref, uv as *const TValue);
                        luaC_barrier!(L, ncl, uv as *const TValue);
                        ui += 1;
                    }

                    // this is a noop if ncl is newly created or shared successfully, but
                    // it has to run after the closure is preloaded for the first time
                    (*ncl).preload = 0;

                    if kcl != ncl {
                        vm_protect!(L, pc, base, {
                            luaC_checkGC!(L);
                        });
                    }

                    pc = pc.add((*kcl).nupvalues as usize);
                    continue 'dispatch;
                }
                LuauOpcode::LOP_PREPVARARGS => {
                    // lvmexecute.cpp:2988
                    let insn = *pc;
                    pc = pc.add(1);
                    let numparams = LUAU_INSN_A!(insn) as i32;

                    // all fixed parameters are copied after the top so we need more stack space
                    vm_protect!(L, pc, base, {
                        luaD_checkstack!(L, (*cl).stacksize as i32 + numparams);
                    });

                    // the caller must have filled extra fixed arguments with nil
                    LUAU_ASSERT!((*L).top.offset_from(base) as i32 >= numparams);

                    // move fixed parameters to final position
                    let fixed = base; // first fixed argument
                    base = (*L).top; // final position of first argument

                    for i in 0..numparams as usize {
                        setobj_2_s!(L, base.add(i), fixed.add(i) as *const TValue);
                        setnilvalue!(fixed.add(i));
                    }

                    // rewire our stack frame to point to the new base
                    (*(*L).ci).base = base;
                    (*(*L).ci).top = base.add((*cl).stacksize as usize);

                    (*L).base = base;
                    (*L).top = (*(*L).ci).top;
                    continue 'dispatch;
                }
                LuauOpcode::LOP_JUMPBACK => {
                    // lvmexecute.cpp:2988
                    VM_INTERRUPT!(L, pc, base);
                    let insn = *pc;
                    pc = pc.add(1);

                    pc = pc.offset(LUAU_INSN_D!(insn) as isize);
                    let p = {
                        let l = &(*cl).inner.l;
                        l.p
                    };
                    LUAU_ASSERT!((pc.offset_from((*p).code) as u32) < (*p).sizecode as u32);
                    continue 'dispatch;
                }

                LuauOpcode::LOP_LOADKX => {
                    // lvmexecute.cpp:2998
                    let insn = *pc;
                    pc = pc.add(1);
                    let ra = VM_REG!(LUAU_INSN_A!(insn), L, base);
                    let aux: u32 = *pc;
                    pc = pc.add(1);
                    let kv = VM_KV!(aux, cl, k);

                    setobj_2_s!(L, ra as *mut TValue, kv as *const TValue);
                    continue 'dispatch;
                }

                LuauOpcode::LOP_JUMPX => {
                    // lvmexecute.cpp:3009
                    VM_INTERRUPT!(L, pc, base);
                    let insn = *pc;
                    pc = pc.add(1);

                    pc = pc.offset(LUAU_INSN_E!(insn) as isize);
                    let p = {
                        let l = &(*cl).inner.l;
                        l.p
                    };
                    LUAU_ASSERT!((pc.offset_from((*p).code) as u32) < (*p).sizecode as u32);
                    continue 'dispatch;
                }
                LuauOpcode::LOP_FASTCALL => {
                    // lvmexecute.cpp:3068
                    let insn = *pc;
                    pc = pc.add(1);
                    let bfid = LUAU_INSN_A!(insn) as i32;
                    let skip = LUAU_INSN_C!(insn) as i32;
                    {
                        {
                            let p = {
                                {
                                    let l = &(*cl).inner.l;
                                    l.p
                                }
                            };
                            LUAU_ASSERT!(
                                ((pc.offset_from((*p).code) as i32 + skip) as u32)
                                    < (*p).sizecode as u32
                            );
                        }
                    }

                    let call: Instruction = *pc.add(skip as usize);
                    LUAU_ASSERT!(LUAU_INSN_OP!(call) == LuauOpcode::LOP_CALL as u32);

                    let ra = VM_REG!(LUAU_INSN_A!(call), L, base) as *mut TValue;

                    let mut nparams = LUAU_INSN_B!(call) as i32 - 1;
                    let nresults = LUAU_INSN_C!(call) as i32 - 1;

                    nparams = if nparams == LUA_MULTRET {
                        {
                            (*L).top.offset_from(ra.add(1)) as i32
                        }
                    } else {
                        {
                            nparams
                        }
                    };

                    let f = luauF_table[bfid as usize];
                    LUAU_ASSERT!(f.is_some());

                    if (*(*cl).env).safeenv != 0 {
                        (*(*L).ci).savedpc = pc; // VM_PROTECT_PC(): f may fail due to OOM

                        let n = f.unwrap()(L, ra, ra.add(1), nresults, ra.add(2), nparams);

                        if n >= 0 {
                            // when nresults != MULTRET, L->top might be pointing to the middle
                            // of stack frame if nparams is equal to MULTRET; restore
                            // unconditionally to skip an extra check
                            (*L).top = if nresults == LUA_MULTRET {
                                ra.add(n as usize)
                            } else {
                                (*(*L).ci).top
                            };

                            // skip instructions that compute function as well as CALL
                            pc = pc.add((skip + 1) as usize);
                            let p = {
                                let l = &(*cl).inner.l;
                                l.p
                            };
                            LUAU_ASSERT!((pc.offset_from((*p).code) as u32) < (*p).sizecode as u32);
                            continue 'dispatch;
                        } else {
                            // continue execution through the fallback code
                            continue 'dispatch;
                        }
                    } else {
                        // continue execution through the fallback code
                        continue 'dispatch;
                    }
                }
                LuauOpcode::LOP_COVERAGE => {
                    // lvmexecute.cpp:3080
                    let insn = *pc;
                    pc = pc.add(1);
                    let mut hits: i32 = LUAU_INSN_E!(insn);

                    // update hits with saturated add and patch the instruction in place
                    hits = if hits < (1 << 23) - 1 { hits + 1 } else { hits };
                    VM_PATCH_E(pc.sub(1), hits);

                    continue 'dispatch;
                }

                LuauOpcode::LOP_CAPTURE => {
                    // lvmexecute.cpp:3086
                    // C++ LUAU_ASSERT(!"CAPTURE is a pseudo-opcode and must be
                    // executed as part of NEWCLOSURE")
                    LUAU_ASSERT!(false);
                    unreachable!() // LUAU_UNREACHABLE()
                }
                LuauOpcode::LOP_SUBRK => {
                    // lvmexecute.cpp:3107
                    let insn = *pc;
                    pc = pc.add(1);
                    let ra = VM_REG!(LUAU_INSN_A!(insn), L, base) as *mut TValue;
                    let kv = VM_KV!(LUAU_INSN_B!(insn), cl, k) as *mut TValue;
                    let rc = VM_REG!(LUAU_INSN_C!(insn), L, base) as *mut TValue;

                    // fast-path
                    if ttisnumber!(rc as *const TValue) {
                        setnvalue!(
                            ra,
                            nvalue!(kv as *const TValue) - nvalue!(rc as *const TValue)
                        );
                        continue 'dispatch;
                    } else {
                        // slow-path, may invoke C/Lua via metamethods
                        vm_protect!(L, pc, base, {
                            lua_v_doarithimpl(
                                L,
                                ra,
                                kv as *const TValue,
                                rc as *const TValue,
                                TMS::TM_SUB,
                            );
                        });
                        continue 'dispatch;
                    }
                }
                LuauOpcode::LOP_DIVRK => {
                    // lvmexecute.cpp:3135
                    let insn = *pc;
                    pc = pc.add(1);
                    let ra = VM_REG!(LUAU_INSN_A!(insn), L, base) as *mut TValue;
                    let kv = VM_KV!(LUAU_INSN_B!(insn), cl, k) as *mut TValue;
                    let rc = VM_REG!(LUAU_INSN_C!(insn), L, base) as *mut TValue;

                    // fast-path
                    if ttisnumber!(rc as *const TValue) {
                        setnvalue!(
                            ra,
                            nvalue!(kv as *const TValue) / nvalue!(rc as *const TValue)
                        );
                        continue 'dispatch;
                    } else if ttisvector!(rc as *const TValue) {
                        let nb = nvalue!(kv as *const TValue) as f32;
                        let vc = vvalue!(rc as *const TValue).as_ptr();
                        setvvalue!(
                            ra,
                            nb / *vc.add(0),
                            nb / *vc.add(1),
                            nb / *vc.add(2),
                            nb / *vc.add(3)
                        );
                        continue 'dispatch;
                    } else {
                        // slow-path, may invoke C/Lua via metamethods
                        vm_protect!(L, pc, base, {
                            lua_v_doarithimpl(
                                L,
                                ra,
                                kv as *const TValue,
                                rc as *const TValue,
                                TMS::TM_DIV,
                            );
                        });
                        continue 'dispatch;
                    }
                }
                LuauOpcode::LOP_FASTCALL1 => {
                    // lvmexecute.cpp:3183
                    let insn = *pc;
                    pc = pc.add(1);
                    let bfid = LUAU_INSN_A!(insn) as i32;
                    let arg = VM_REG!(LUAU_INSN_B!(insn), L, base) as *mut TValue;
                    let skip = LUAU_INSN_C!(insn) as i32;
                    {
                        {
                            let p = {
                                {
                                    let l = &(*cl).inner.l;
                                    l.p
                                }
                            };
                            LUAU_ASSERT!(
                                ((pc.offset_from((*p).code) as i32 + skip) as u32)
                                    < (*p).sizecode as u32
                            );
                        }
                    }

                    let call: Instruction = *pc.add(skip as usize);
                    LUAU_ASSERT!(LUAU_INSN_OP!(call) == LuauOpcode::LOP_CALL as u32);

                    let ra = VM_REG!(LUAU_INSN_A!(call), L, base) as *mut TValue;

                    let nparams = 1i32;
                    let nresults = LUAU_INSN_C!(call) as i32 - 1;

                    let f = luauF_table[bfid as usize];
                    LUAU_ASSERT!(f.is_some());

                    if (*(*cl).env).safeenv != 0 {
                        (*(*L).ci).savedpc = pc; // VM_PROTECT_PC(): f may fail due to OOM

                        let n = f.unwrap()(L, ra, arg, nresults, core::ptr::null_mut(), nparams);

                        if n >= 0 {
                            if nresults == LUA_MULTRET {
                                (*L).top = ra.add(n as usize);
                            }

                            // skip instructions that compute function as well as CALL
                            pc = pc.add((skip + 1) as usize);
                            let p = {
                                let l = &(*cl).inner.l;
                                l.p
                            };
                            LUAU_ASSERT!((pc.offset_from((*p).code) as u32) < (*p).sizecode as u32);
                            continue 'dispatch;
                        } else {
                            // continue execution through the fallback code
                            continue 'dispatch;
                        }
                    } else {
                        // continue execution through the fallback code
                        continue 'dispatch;
                    }
                }
                LuauOpcode::LOP_FASTCALL2 => {
                    // lvmexecute.cpp:3233
                    let insn = *pc;
                    pc = pc.add(1);
                    let bfid = LUAU_INSN_A!(insn) as i32;
                    let skip = LUAU_INSN_C!(insn) as i32 - 1;
                    let aux: u32 = *pc;
                    pc = pc.add(1);
                    let arg1 = VM_REG!(LUAU_INSN_B!(insn), L, base) as *mut TValue;
                    let arg2 = VM_REG!(aux, L, base) as *mut TValue;
                    {
                        let p = {
                            let l = &(*cl).inner.l;
                            l.p
                        };
                        LUAU_ASSERT!(
                            ((pc.offset_from((*p).code) as i32 + skip) as u32)
                                < (*p).sizecode as u32
                        );
                    }

                    let call: Instruction = *pc.add(skip as usize);
                    LUAU_ASSERT!(LUAU_INSN_OP!(call) == LuauOpcode::LOP_CALL as u32);

                    let ra = VM_REG!(LUAU_INSN_A!(call), L, base) as *mut TValue;

                    let nparams = 2i32;
                    let nresults = LUAU_INSN_C!(call) as i32 - 1;

                    let f = luauF_table[bfid as usize];
                    LUAU_ASSERT!(f.is_some());

                    if (*(*cl).env).safeenv != 0 {
                        (*(*L).ci).savedpc = pc; // VM_PROTECT_PC(): f may fail due to OOM

                        let n = f.unwrap()(L, ra, arg1, nresults, arg2, nparams);

                        if n >= 0 {
                            if nresults == LUA_MULTRET {
                                (*L).top = ra.add(n as usize);
                            }

                            // skip instructions that compute function as well as CALL
                            pc = pc.add((skip + 1) as usize);
                            let p = {
                                let l = &(*cl).inner.l;
                                l.p
                            };
                            LUAU_ASSERT!((pc.offset_from((*p).code) as u32) < (*p).sizecode as u32);
                            continue 'dispatch;
                        } else {
                            // continue execution through the fallback code
                            continue 'dispatch;
                        }
                    } else {
                        // continue execution through the fallback code
                        continue 'dispatch;
                    }
                }
                LuauOpcode::LOP_FASTCALL2K => {
                    // lvmexecute.cpp:3283
                    let insn = *pc;
                    pc = pc.add(1);
                    let bfid = LUAU_INSN_A!(insn) as i32;
                    let skip = LUAU_INSN_C!(insn) as i32 - 1;
                    let aux: u32 = *pc;
                    pc = pc.add(1);
                    let arg1 = VM_REG!(LUAU_INSN_B!(insn), L, base) as *mut TValue;
                    let arg2 = VM_KV!(aux, cl, k) as *mut TValue;
                    {
                        let p = {
                            let l = &(*cl).inner.l;
                            l.p
                        };
                        LUAU_ASSERT!(
                            ((pc.offset_from((*p).code) as i32 + skip) as u32)
                                < (*p).sizecode as u32
                        );
                    }

                    let call: Instruction = *pc.add(skip as usize);
                    LUAU_ASSERT!(LUAU_INSN_OP!(call) == LuauOpcode::LOP_CALL as u32);

                    let ra = VM_REG!(LUAU_INSN_A!(call), L, base) as *mut TValue;

                    let nparams = 2i32;
                    let nresults = LUAU_INSN_C!(call) as i32 - 1;

                    let f = luauF_table[bfid as usize];
                    LUAU_ASSERT!(f.is_some());

                    if (*(*cl).env).safeenv != 0 {
                        (*(*L).ci).savedpc = pc; // VM_PROTECT_PC(): f may fail due to OOM

                        let n = f.unwrap()(L, ra, arg1, nresults, arg2, nparams);

                        if n >= 0 {
                            if nresults == LUA_MULTRET {
                                (*L).top = ra.add(n as usize);
                            }

                            // skip instructions that compute function as well as CALL
                            pc = pc.add((skip + 1) as usize);
                            let p = {
                                let l = &(*cl).inner.l;
                                l.p
                            };
                            LUAU_ASSERT!((pc.offset_from((*p).code) as u32) < (*p).sizecode as u32);
                            continue 'dispatch;
                        } else {
                            // continue execution through the fallback code
                            continue 'dispatch;
                        }
                    } else {
                        // continue execution through the fallback code
                        continue 'dispatch;
                    }
                }
                LuauOpcode::LOP_FASTCALL3 => {
                    // lvmexecute.cpp:3340
                    let insn = *pc;
                    pc = pc.add(1);
                    let bfid = LUAU_INSN_A!(insn) as i32;
                    let skip = LUAU_INSN_C!(insn) as i32 - 1;
                    let aux: u32 = *pc;
                    pc = pc.add(1);
                    let arg1 = VM_REG!(LUAU_INSN_B!(insn), L, base) as *mut TValue;
                    let arg2 = VM_REG!(LUAU_INSN_AUX_A(aux), L, base) as *mut TValue;
                    let arg3 = VM_REG!(LUAU_INSN_AUX_B(aux), L, base) as *mut TValue;
                    {
                        let p = {
                            let l = &(*cl).inner.l;
                            l.p
                        };
                        LUAU_ASSERT!(
                            ((pc.offset_from((*p).code) as i32 + skip) as u32)
                                < (*p).sizecode as u32
                        );
                    }

                    let call: Instruction = *pc.add(skip as usize);
                    LUAU_ASSERT!(LUAU_INSN_OP!(call) == LuauOpcode::LOP_CALL as u32);

                    let ra = VM_REG!(LUAU_INSN_A!(call), L, base) as *mut TValue;

                    let nparams = 3i32;
                    let nresults = LUAU_INSN_C!(call) as i32 - 1;

                    let f = luauF_table[bfid as usize];
                    LUAU_ASSERT!(f.is_some());

                    if (*(*cl).env).safeenv != 0 {
                        (*(*L).ci).savedpc = pc; // VM_PROTECT_PC(): f may fail due to OOM

                        // note: it's safe to push arguments past top for complicated reasons (see top of the file)
                        LUAU_ASSERT!((*L).top.add(2) < (*L).stack.add((*L).stacksize as usize));
                        let top = (*L).top;
                        setobj_2_s!(L, top, arg2 as *const TValue);
                        setobj_2_s!(L, top.add(1), arg3 as *const TValue);

                        let n = f.unwrap()(L, ra, arg1, nresults, top, nparams);

                        if n >= 0 {
                            if nresults == LUA_MULTRET {
                                (*L).top = ra.add(n as usize);
                            }

                            // skip instructions that compute function as well as CALL
                            pc = pc.add((skip + 1) as usize);
                            let p = {
                                let l = &(*cl).inner.l;
                                l.p
                            };
                            LUAU_ASSERT!((pc.offset_from((*p).code) as u32) < (*p).sizecode as u32);
                            continue 'dispatch;
                        } else {
                            // continue execution through the fallback code
                            continue 'dispatch;
                        }
                    } else {
                        // continue execution through the fallback code
                        continue 'dispatch;
                    }
                }
                LuauOpcode::LOP_BREAK => {
                    // lvmexecute.cpp:3359
                    let proto = {
                        let l = &(*cl).inner.l;
                        l.p
                    };
                    LUAU_ASSERT!(!(*proto).debuginsn.is_null());

                    let op = *(*proto)
                        .debuginsn
                        .add(pc.offset_from((*proto).code) as usize);
                    LUAU_ASSERT!(op != LuauOpcode::LOP_BREAK as u8);

                    if (*(*L).global).cb.debugbreak.is_some() {
                        let debugbreak = (*(*L).global).cb.debugbreak;
                        vm_protect!(L, pc, base, {
                            luau_callhook(L, debugbreak, core::ptr::null_mut());
                        });

                        // allow debugbreak hook to put thread into error/yield state
                        if (*L).status != 0 {
                            return; // goto exit
                        }
                    }

                    // VM_CONTINUE(op): re-dispatch the original opcode without refetching
                    continue_op = Some(op);
                    continue 'dispatch;
                }
                LuauOpcode::LOP_JUMPXEQKNIL => {
                    // lvmexecute.cpp:3372
                    let insn = *pc;
                    pc = pc.add(1);
                    let aux: u32 = *pc;
                    let ra = VM_REG!(LUAU_INSN_A!(insn), L, base) as *mut TValue;

                    // static_assert(LUA_TNIL == 0): type-1 is negative iff type is nil.
                    // condition is equivalent to: int(ttisnil(ra)) != LUAU_INSN_AUX_NOT(aux)
                    pc = pc.offset(
                        if (((ttype!(ra as *const TValue) - 1) as u32 ^ aux) as i32) < 0 {
                            LUAU_INSN_D!(insn) as isize
                        } else {
                            1
                        },
                    );
                    let p = {
                        let l = &(*cl).inner.l;
                        l.p
                    };
                    LUAU_ASSERT!((pc.offset_from((*p).code) as u32) < (*p).sizecode as u32);
                    continue 'dispatch;
                }
                LuauOpcode::LOP_JUMPXEQKB => {
                    // lvmexecute.cpp:3383
                    let insn = *pc;
                    pc = pc.add(1);
                    let aux: u32 = *pc;
                    let ra = VM_REG!(LUAU_INSN_A!(insn), L, base) as *mut TValue;

                    pc = pc.offset(
                        if (ttisboolean!(ra as *const TValue)
                            && bvalue!(ra as *const TValue) == LUAU_INSN_AUX_KB(aux) as i32)
                            as i32
                            != LUAU_INSN_AUX_NOT(aux) as i32
                        {
                            LUAU_INSN_D!(insn) as isize
                        } else {
                            1
                        },
                    );
                    let p = {
                        let l = &(*cl).inner.l;
                        l.p
                    };
                    LUAU_ASSERT!((pc.offset_from((*p).code) as u32) < (*p).sizecode as u32);
                    continue 'dispatch;
                }
                LuauOpcode::LOP_JUMPXEQKN => {
                    // lvmexecute.cpp:3405 (non-aarch64 flavor; the __aarch64__
                    // branch is a codegen-only variant with identical semantics)
                    let insn = *pc;
                    pc = pc.add(1);
                    let aux: u32 = *pc;
                    let ra = VM_REG!(LUAU_INSN_A!(insn), L, base) as *mut TValue;
                    let kv = VM_KV!(LUAU_INSN_AUX_KV(aux), cl, k) as *mut TValue;
                    LUAU_ASSERT!(ttisnumber!(kv as *const TValue));

                    pc = pc.offset(
                        if (ttisnumber!(ra as *const TValue)
                            && nvalue!(ra as *const TValue) == nvalue!(kv as *const TValue))
                            as i32
                            != LUAU_INSN_AUX_NOT(aux) as i32
                        {
                            LUAU_INSN_D!(insn) as isize
                        } else {
                            1
                        },
                    );
                    let p = {
                        let l = &(*cl).inner.l;
                        l.p
                    };
                    LUAU_ASSERT!((pc.offset_from((*p).code) as u32) < (*p).sizecode as u32);
                    continue 'dispatch;
                }
                LuauOpcode::LOP_JUMPXEQKS => {
                    // lvmexecute.cpp:3418
                    let insn = *pc;
                    pc = pc.add(1);
                    let aux: u32 = *pc;
                    let ra = VM_REG!(LUAU_INSN_A!(insn), L, base) as *mut TValue;
                    let kv = VM_KV!(LUAU_INSN_AUX_KV(aux), cl, k) as *mut TValue;
                    LUAU_ASSERT!(ttisstring!(kv as *const TValue));

                    pc = pc.offset(
                        if (ttisstring!(ra as *const TValue)
                            && gcvalue!(ra as *const TValue) == gcvalue!(kv as *const TValue))
                            as i32
                            != LUAU_INSN_AUX_NOT(aux) as i32
                        {
                            LUAU_INSN_D!(insn) as isize
                        } else {
                            1
                        },
                    );
                    let p = {
                        let l = &(*cl).inner.l;
                        l.p
                    };
                    LUAU_ASSERT!((pc.offset_from((*p).code) as u32) < (*p).sizecode as u32);
                    continue 'dispatch;
                }
                LuauOpcode::LOP_GETUDATAKS => {
                    // lvmexecute.cpp:3498
                    let insn = *pc;
                    pc = pc.add(1);
                    let mut ra = VM_REG!(LUAU_INSN_A!(insn), L, base) as *mut TValue;
                    let rb = VM_REG!(LUAU_INSN_B!(insn), L, base) as *mut TValue;
                    let aux: u32 = *pc;
                    pc = pc.add(1);
                    let kidx = LUAU_INSN_AUX_KV16(aux);
                    let kv = VM_KV!(kidx, cl, k) as *mut TValue;

                    'udata_fast: {
                        if ttisuserdata!(rb as *const TValue) {
                            let utag = (*uvalue!(rb as *const TValue)).tag as usize;
                            let udatadirect = &mut (*(*L).global).udatadirect[utag];
                            let onudataindex = udatadirect.index;
                            let tm = &mut udatadirect.indextm as *mut TValue;

                            if let Some(onudataindex) = onudataindex {
                                if !ttisnil!(tm as *const TValue) {
                                    let udata = {
                                        let u = uvalue!(rb as *const TValue);
                                        u.data.as_ptr() as *mut core::ffi::c_void
                                    };

                                    // note: it's safe to push arguments past top for
                                    // complicated reasons (see top of the file)
                                    LUAU_ASSERT!(
                                        (*L).top.add(3) < (*L).stack.add((*L).stacksize as usize)
                                    );
                                    let top = (*L).top;
                                    setobj_2_s!(L, top.add(0), tm as *const TValue);
                                    setobj_2_s!(L, top.add(1), rb as *const TValue);
                                    setobj_2_s!(L, top.add(2), kv as *const TValue);
                                    (*L).top = (*L).top.add(3);

                                    (*(*L).ci).savedpc = pc;

                                    (*L).nCcalls += 1;

                                    if ((*L).nCcalls as i32) >= LUAI_MAXCCALLS {
                                        luaD_checkCstack(L);
                                    }

                                    luau_setupcci(L, 1, top);

                                    let mut cachedslot: u16 = LUAU_INSN_AUX_SLOT!(aux) as u16;
                                    onudataindex(
                                        L,
                                        udata,
                                        (*tsvalue!(kv as *const TValue)).atom as i32,
                                        &mut cachedslot,
                                        utag as i32,
                                    );

                                    // update cached slot if instruction didn't deoptimize
                                    if cachedslot as u32 != LUAU_INSN_AUX_SLOT!(aux)
                                        && LUAU_INSN_OP!(*pc.sub(2))
                                            == LuauOpcode::LOP_GETUDATAKS as u32
                                    {
                                        VM_PATCH_AUX_SLOT(pc.sub(1), kidx, cachedslot as i32);
                                    }

                                    // ci is our callinfo, cip is our parent
                                    let ci = (*L).ci;
                                    let cip = ci.sub(1);

                                    if luaur_common::FFlag::LuauClosureUsageCounter.get() {
                                        let cicl = clvalue!((*ci).func);
                                        LUAU_ASSERT!((*cicl).usage > 0);
                                        (*cicl).usage -= 1;
                                    }

                                    (*L).ci = cip;
                                    (*L).base = (*cip).base;
                                    (*L).nCcalls -= 1;

                                    // stack may have been reallocated, so we need to refresh base ptr
                                    base = (*L).base;
                                    ra = VM_REG!(LUAU_INSN_A!(insn), L, base) as *mut TValue;

                                    // grab result while L->top is still pointed to the
                                    // previous function frame
                                    setobj_2_s!(L, ra, (*L).top.sub(1) as *const TValue);

                                    // then update top
                                    (*L).top = (*cip).top;

                                    continue 'dispatch;
                                }
                            }
                        }
                        break 'udata_fast;
                    }

                    // Slow path - backpatch and dispatch to regular table access
                    VM_PATCH_OP(pc.sub(2), LuauOpcode::LOP_GETTABLEKS as u8);
                    VM_PATCH_AUX_SLOT(pc.sub(1), kidx, 0);

                    pc = pc.sub(2);
                    continue_op = Some(LuauOpcode::LOP_GETTABLEKS as u8); // VM_CONTINUE
                    continue 'dispatch;
                }
                LuauOpcode::LOP_SETUDATAKS => {
                    // lvmexecute.cpp:3573
                    let insn = *pc;
                    pc = pc.add(1);
                    let ra = VM_REG!(LUAU_INSN_A!(insn), L, base) as *mut TValue;
                    let rb = VM_REG!(LUAU_INSN_B!(insn), L, base) as *mut TValue;
                    let aux: u32 = *pc;
                    pc = pc.add(1);
                    let kidx = LUAU_INSN_AUX_KV16(aux);
                    let kv = VM_KV!(kidx, cl, k) as *mut TValue;

                    'udata_fast: {
                        if ttisuserdata!(rb as *const TValue) {
                            let utag = (*uvalue!(rb as *const TValue)).tag as usize;
                            let udatadirect = &mut (*(*L).global).udatadirect[utag];
                            let onudatanewindex = udatadirect.newindex;
                            let tm = &mut udatadirect.newindextm as *mut TValue;

                            if let Some(onudatanewindex) = onudatanewindex {
                                if !ttisnil!(tm as *const TValue) {
                                    let udata = {
                                        let u = uvalue!(rb as *const TValue);
                                        u.data.as_ptr() as *mut core::ffi::c_void
                                    };

                                    // note: it's safe to push arguments past top for
                                    // complicated reasons (see top of the file)
                                    LUAU_ASSERT!(
                                        (*L).top.add(4) < (*L).stack.add((*L).stacksize as usize)
                                    );
                                    let top = (*L).top;
                                    setobj_2_s!(L, top.add(0), tm as *const TValue);
                                    setobj_2_s!(L, top.add(1), rb as *const TValue);
                                    setobj_2_s!(L, top.add(2), kv as *const TValue);
                                    setobj_2_s!(L, top.add(3), ra as *const TValue);
                                    (*L).top = (*L).top.add(4);

                                    (*(*L).ci).savedpc = pc;

                                    (*L).nCcalls += 1;

                                    if ((*L).nCcalls as i32) >= LUAI_MAXCCALLS {
                                        luaD_checkCstack(L);
                                    }

                                    luau_setupcci(L, 0, top);

                                    let mut cachedslot: u16 = LUAU_INSN_AUX_SLOT!(aux) as u16;
                                    onudatanewindex(
                                        L,
                                        udata,
                                        (*tsvalue!(kv as *const TValue)).atom as i32,
                                        &mut cachedslot,
                                        utag as i32,
                                    );

                                    // update cached slot if instruction didn't deoptimize
                                    if cachedslot as u32 != LUAU_INSN_AUX_SLOT!(aux)
                                        && LUAU_INSN_OP!(*pc.sub(2))
                                            == LuauOpcode::LOP_SETUDATAKS as u32
                                    {
                                        VM_PATCH_AUX_SLOT(pc.sub(1), kidx, cachedslot as i32);
                                    }

                                    // ci is our callinfo, cip is our parent
                                    let ci = (*L).ci;
                                    let cip = ci.sub(1);

                                    if luaur_common::FFlag::LuauClosureUsageCounter.get() {
                                        let cicl = clvalue!((*ci).func);
                                        LUAU_ASSERT!((*cicl).usage > 0);
                                        (*cicl).usage -= 1;
                                    }

                                    (*L).ci = cip;
                                    (*L).base = (*cip).base;
                                    (*L).top = (*cip).top;
                                    (*L).nCcalls -= 1;

                                    // stack may have been reallocated, so we need to refresh base ptr
                                    base = (*L).base;

                                    continue 'dispatch;
                                }
                            }
                        }
                        break 'udata_fast;
                    }

                    // Slow path - backpatch and dispatch to regular table access
                    VM_PATCH_OP(pc.sub(2), LuauOpcode::LOP_SETTABLEKS as u8);
                    VM_PATCH_AUX_SLOT(pc.sub(1), kidx, 0);

                    pc = pc.sub(2);
                    continue_op = Some(LuauOpcode::LOP_SETTABLEKS as u8); // VM_CONTINUE
                    continue 'dispatch;
                }
                LuauOpcode::LOP_NAMECALLUDATA => {
                    // lvmexecute.cpp:3670
                    let insn = *pc;
                    pc = pc.add(1);
                    let ra = VM_REG!(LUAU_INSN_A!(insn), L, base) as *mut TValue;
                    let rb = VM_REG!(LUAU_INSN_B!(insn), L, base) as *mut TValue;
                    let aux: u32 = *pc;
                    pc = pc.add(1);
                    let kidx = LUAU_INSN_AUX_KV16(aux);
                    let kv = VM_KV!(kidx, cl, k) as *mut TValue;

                    'udata_fast: {
                        if ttisuserdata!(rb as *const TValue) {
                            let utag = (*uvalue!(rb as *const TValue)).tag as usize;
                            let udatadirect = &mut (*(*L).global).udatadirect[utag];
                            let onudatanamecall = udatadirect.namecall;
                            let tm = &mut udatadirect.namecalltm as *mut TValue;

                            if let Some(onudatanamecall) = onudatanamecall {
                                if !ttisnil!(tm as *const TValue) {
                                    let udata = {
                                        let u = uvalue!(rb as *const TValue);
                                        u.data.as_ptr() as *mut core::ffi::c_void
                                    };

                                    // note: order of copies allows rb to alias ra+1 or ra
                                    setobj_2_s!(L, ra.add(1), rb as *const TValue);
                                    setobj_2_s!(L, ra, tm as *const TValue);
                                    let ncslot: *const Instruction = pc.sub(1);

                                    LUAU_ASSERT!(
                                        LUAU_INSN_OP!(*pc) == LuauOpcode::LOP_CALL as u32
                                            || LUAU_INSN_OP!(*pc) == LuauOpcode::LOP_CALLFB as u32
                                    );
                                    let call_insn = *pc;
                                    pc = pc.add(1);
                                    if luaur_common::FFlag::LuauCallFeedback.get()
                                        && LUAU_INSN_OP!(call_insn) == LuauOpcode::LOP_CALLFB as u32
                                    {
                                        pc = pc.add(1);
                                    }

                                    let call_ra =
                                        VM_REG!(LUAU_INSN_A!(call_insn), L, base) as *mut TValue;
                                    LUAU_ASSERT!(call_ra == ra);

                                    // first half of OP_CALL
                                    let nparams = LUAU_INSN_B!(call_insn) as i32 - 1;
                                    let nresults = LUAU_INSN_C!(call_insn) as i32 - 1;

                                    (*(*L).ci).savedpc = pc;
                                    (*L).namecall = tsvalue!(kv as *const TValue)
                                        as *mut crate::records::t_string::TString;
                                    (*L).top = if nparams == LUA_MULTRET {
                                        (*L).top
                                    } else {
                                        ra.add(1 + nparams as usize)
                                    };

                                    // note: namecalls do not increase C call number and allow yielding

                                    luau_setupcci(L, nresults, ra);

                                    LUAU_ASSERT!((*tsvalue!(kv as *const TValue)).atom >= 0);

                                    let mut cachedslot: u16 = LUAU_INSN_AUX_SLOT!(aux) as u16;
                                    let results = onudatanamecall(
                                        L,
                                        udata,
                                        (*tsvalue!(kv as *const TValue)).atom as i32,
                                        &mut cachedslot,
                                        utag as i32,
                                    );

                                    // update cached slot if instruction didn't deoptimize
                                    if cachedslot as u32 != LUAU_INSN_AUX_SLOT!(aux)
                                        && LUAU_INSN_OP!(*ncslot.sub(1))
                                            == LuauOpcode::LOP_NAMECALLUDATA as u32
                                    {
                                        VM_PATCH_AUX_SLOT(ncslot, kidx, cachedslot as i32);
                                    }

                                    // yield
                                    if results < 0 {
                                        return;
                                    }

                                    // ci is our callinfo, cip is our parent
                                    let ci = (*L).ci;
                                    let cip = ci.sub(1);

                                    if luaur_common::FFlag::LuauClosureUsageCounter.get() {
                                        let cicl = clvalue!((*ci).func);
                                        LUAU_ASSERT!((*cicl).usage > 0);
                                        (*cicl).usage -= 1;
                                    }

                                    let mut res = (*ci).func;
                                    let mut vali = (*L).top.sub(results as usize);
                                    let valend = (*L).top;

                                    let mut i = nresults;
                                    while i != 0 && vali < valend {
                                        setobj_2_s!(L, res, vali as *const TValue);
                                        res = res.add(1);
                                        vali = vali.add(1);
                                        i -= 1;
                                    }
                                    while i > 0 {
                                        setnilvalue!(res);
                                        res = res.add(1);
                                        i -= 1;
                                    }

                                    (*L).ci = cip;
                                    (*L).base = (*cip).base;
                                    (*L).top = if nresults == LUA_MULTRET {
                                        res
                                    } else {
                                        (*cip).top
                                    };

                                    // stack may have been reallocated, so we need to refresh base ptr
                                    base = (*L).base;

                                    continue 'dispatch;
                                }
                            }
                        }
                        break 'udata_fast;
                    }

                    // Slow path - backpatch and dispatch to regular namecall
                    VM_PATCH_OP(pc.sub(2), LuauOpcode::LOP_NAMECALL as u8);
                    VM_PATCH_AUX_SLOT(pc.sub(1), kidx, 0);

                    pc = pc.sub(2);
                    continue_op = Some(LuauOpcode::LOP_NAMECALL as u8); // VM_CONTINUE
                    continue 'dispatch;
                }
                LuauOpcode::LOP_NEWCLASSMEMBER => {
                    // lvmexecute.cpp:3670 (NEWCLASSMEMBER)
                    let insn = *pc;
                    pc = pc.add(1);
                    let aux: u32 = *pc;
                    pc = pc.add(1);
                    let ra = VM_REG!(LUAU_INSN_A!(insn), L, base) as *mut TValue;
                    let membername = VM_KV!(aux, cl, k) as *mut TValue;
                    LUAU_ASSERT!(ttisstring!(membername as *const TValue));
                    LUAU_ASSERT!(LUAU_INSN_B!(insn) == 0);
                    let rc = VM_REG!(LUAU_INSN_C!(insn), L, base) as *mut TValue;
                    (*(*L).ci).savedpc = pc; // VM_PROTECT_PC()
                    lua_r_addclassmember(
                        L,
                        &mut **classvalue!(ra as *const TValue) as *mut LuauClass,
                        tsvalue!(membername as *const TValue)
                            as *mut crate::records::t_string::TString,
                        rc,
                    );
                    continue 'dispatch;
                }
                LuauOpcode::LOP_CMPPROTO => {
                    // lvmexecute.cpp:3684
                    let insn = *pc;
                    pc = pc.add(1);
                    let funid: u32 = *pc;
                    pc = pc.add(1);
                    let ra = VM_REG!(LUAU_INSN_A!(insn), L, base) as *mut TValue;

                    if !ttisfunction!(ra as *const TValue) {
                        pc = pc.offset(LUAU_INSN_D!(insn) as isize - 1);
                        let p = {
                            let l = &(*cl).inner.l;
                            l.p
                        };
                        LUAU_ASSERT!((pc.offset_from((*p).code) as u32) < (*p).sizecode as u32);
                        continue 'dispatch;
                    }

                    let ccl = clvalue!(ra as *const TValue);
                    if (*ccl).isC != 0 || {
                        let l = &(*ccl).inner.l;
                        (*l.p).funid != funid
                    } {
                        pc = pc.offset(LUAU_INSN_D!(insn) as isize - 1);
                    }

                    let p = {
                        let l = &(*cl).inner.l;
                        l.p
                    };
                    LUAU_ASSERT!((pc.offset_from((*p).code) as u32) < (*p).sizecode as u32);
                    continue 'dispatch;
                }

                #[allow(unreachable_patterns)]
                _ => unreachable!("byte is not an executable opcode"),
            }
        }

        // C++ `exit:;` is the function end; 'reentry is only re-entered via
        // explicit `continue 'reentry` (native-call return paths).
        #[allow(unreachable_code)]
        {
            break 'reentry;
        }
    }
}
