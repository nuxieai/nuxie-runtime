use crate::enums::feedback_vector_slot_kind::FeedbackVectorSlotKind;
use crate::enums::lua_status::lua_Status;
use crate::functions::lua_a_toobject::luaA_toobject;
use crate::functions::lua_c_barrierback::lua_c_barrierback;
use crate::functions::lua_d_pcall::luaD_pcall;
use crate::functions::lua_f_new_lclosure::lua_f_new_lclosure;
use crate::functions::lua_f_newproto::lua_f_newproto;
use crate::functions::lua_gettop::lua_gettop;
use crate::functions::lua_h_new::lua_h_new;
use crate::functions::lua_h_set::luaH_set;
use crate::functions::lua_h_setstr::lua_h_setstr;
use crate::functions::lua_o_chunkid::lua_o_chunkid;
use crate::functions::lua_pushlstring::lua_pushlstring;
use crate::functions::lua_r_newclass::lua_r_newclass;
use crate::functions::lua_s_newlstr::luaS_newlstr;
use crate::functions::read::read;
use crate::functions::read_string::read_string;
use crate::functions::read_var_int::read_var_int;
use crate::functions::read_var_int_64::read_var_int_64;
use crate::functions::remap_userdata_types::remap_userdata_types;
use crate::macros::getstr::getstr;
use crate::macros::hvalue::hvalue;
use crate::macros::incr_top::incr_top;
use crate::macros::isblack::isblack;
use crate::macros::lua_c_barriert::luaC_barriert;
use crate::macros::lua_idsize::LUA_IDSIZE;
use crate::macros::lua_m_newarray::luaM_newarray;
use crate::macros::lua_s_new::luaS_new;
use crate::macros::lua_s_updateatom::luaS_updateatom;
use crate::macros::savestack::savestack;
use crate::macros::setbvalue::setbvalue;
use crate::macros::setclassvalue::setclassvalue;
use crate::macros::setclvalue::setclvalue;
use crate::macros::sethvalue::sethvalue;
use crate::macros::setlvalue::setlvalue;
use crate::macros::setnilvalue::setnilvalue;
use crate::macros::setnvalue::setnvalue;
use crate::macros::setobj::setobj;
use crate::macros::setobj_2_t::setobj2t;
use crate::macros::setsvalue::setsvalue;
use crate::macros::setvvalue::setvvalue;
use crate::macros::tsvalue::tsvalue;
use crate::macros::ttisnil::ttisnil;
use crate::macros::ttisstring::ttisstring;
use crate::records::feedback_vector_slot::FeedbackVectorSlot;
use crate::records::gc_object::GCObject;
use crate::records::loc_var::LocVar;
use crate::records::proto::Proto;
use crate::records::resolve_import::ResolveImport;
use crate::records::t_string::TString;
use crate::records::temp_buffer::TempBuffer;
use crate::type_aliases::instruction::Instruction;
use crate::type_aliases::lua_state::lua_State;
use crate::type_aliases::lua_table::LuaTable;
use crate::type_aliases::t_value::TValue;
use core::ffi::{c_char, c_int};
use luaur_common::enums::luau_bytecode_tag::{
    LBC_CONSTANT_BOOLEAN, LBC_CONSTANT_CLASS_SHAPE, LBC_CONSTANT_CLOSURE, LBC_CONSTANT_IMPORT,
    LBC_CONSTANT_INTEGER, LBC_CONSTANT_NIL, LBC_CONSTANT_NUMBER, LBC_CONSTANT_STRING,
    LBC_CONSTANT_TABLE, LBC_CONSTANT_TABLE_WITH_CONSTANTS, LBC_CONSTANT_VECTOR,
    LBC_TYPE_VERSION_MAX, LBC_TYPE_VERSION_MIN, LBC_VERSION_MAX, LBC_VERSION_MIN,
};
use luaur_common::enums::luau_bytecode_type::{
    LBC_TYPE_FUNCTION, LBC_TYPE_TAGGED_USERDATA_BASE, LBC_TYPE_TAGGED_USERDATA_END,
    LBC_TYPE_USERDATA,
};
use luaur_common::enums::luau_feedback_type::LuauFeedbackType;
use luaur_common::enums::luau_opcode::LuauOpcode;
use luaur_common::functions::get_op_length::getOpLength;
use luaur_common::macros::luau_assert::LUAU_ASSERT;
use luaur_common::macros::luau_insn_op::LUAU_INSN_OP;

const LBC_CONSTANT_NIL_U8: u8 = LBC_CONSTANT_NIL.0 as u8;
const LBC_CONSTANT_BOOLEAN_U8: u8 = LBC_CONSTANT_BOOLEAN.0 as u8;
const LBC_CONSTANT_NUMBER_U8: u8 = LBC_CONSTANT_NUMBER.0 as u8;
const LBC_CONSTANT_STRING_U8: u8 = LBC_CONSTANT_STRING.0 as u8;
const LBC_CONSTANT_IMPORT_U8: u8 = LBC_CONSTANT_IMPORT.0 as u8;
const LBC_CONSTANT_TABLE_U8: u8 = LBC_CONSTANT_TABLE.0 as u8;
const LBC_CONSTANT_CLOSURE_U8: u8 = LBC_CONSTANT_CLOSURE.0 as u8;
const LBC_CONSTANT_VECTOR_U8: u8 = LBC_CONSTANT_VECTOR.0 as u8;
const LBC_CONSTANT_TABLE_WITH_CONSTANTS_U8: u8 = LBC_CONSTANT_TABLE_WITH_CONSTANTS.0 as u8;
const LBC_CONSTANT_INTEGER_U8: u8 = LBC_CONSTANT_INTEGER.0 as u8;
const LBC_CONSTANT_CLASS_SHAPE_U8: u8 = LBC_CONSTANT_CLASS_SHAPE.0 as u8;
const USERDATA_TYPE_LIMIT: usize =
    (LBC_TYPE_TAGGED_USERDATA_END.0 - LBC_TYPE_TAGGED_USERDATA_BASE.0) as usize;

#[allow(non_snake_case)]
pub unsafe fn loadsafe(
    L: *mut lua_State,
    strings: &mut TempBuffer<*mut TString>,
    protos: &mut TempBuffer<*mut Proto>,
    chunkname: *const c_char,
    data: *const c_char,
    size: usize,
    env: c_int,
) -> c_int {
    let mut offset: usize = 0;

    let version: u8 = read(data, size, &mut offset);

    // 0 means the rest of the bytecode is the error message
    if version == 0 {
        let mut chunkbuf = [0 as c_char; LUA_IDSIZE as usize];
        let chunkid = lua_o_chunkid(
            chunkbuf.as_mut_ptr(),
            chunkbuf.len(),
            chunkname,
            c_strlen(chunkname),
        );
        push_chunk_prefixed_slice(L, chunkid, data.add(offset), size - offset);
        return 1;
    }

    if version < LBC_VERSION_MIN.0 as u8 || version > LBC_VERSION_MAX.0 as u8 {
        let mut chunkbuf = [0 as c_char; LUA_IDSIZE as usize];
        let chunkid = lua_o_chunkid(
            chunkbuf.as_mut_ptr(),
            chunkbuf.len(),
            chunkname,
            c_strlen(chunkname),
        );
        let message = format!(
            "{}: bytecode version mismatch (expected [{}..{}], got {})",
            c_string_lossy(chunkid),
            LBC_VERSION_MIN.0,
            LBC_VERSION_MAX.0,
            version
        );
        push_rust_string(L, &message);
        return 1;
    }

    let mut typesversion: u8 = 0;

    if version >= 4 {
        typesversion = read(data, size, &mut offset);

        if typesversion < LBC_TYPE_VERSION_MIN.0 as u8
            || typesversion > LBC_TYPE_VERSION_MAX.0 as u8
        {
            let mut chunkbuf = [0 as c_char; LUA_IDSIZE as usize];
            let chunkid = lua_o_chunkid(
                chunkbuf.as_mut_ptr(),
                chunkbuf.len(),
                chunkname,
                c_strlen(chunkname),
            );
            let message = format!(
                "{}: bytecode type version mismatch (expected [{}..{}], got {})",
                c_string_lossy(chunkid),
                LBC_TYPE_VERSION_MIN.0,
                LBC_TYPE_VERSION_MAX.0,
                typesversion
            );
            push_rust_string(L, &message);
            return 1;
        }
    }

    // env is 0 for current environment and a stack index otherwise
    let envt: *mut LuaTable = if env == 0 {
        (*L).gt
    } else {
        hvalue!(luaA_toobject(L, env))
    };

    let source: *mut TString = luaS_new(L, chunkname);

    // string table
    let string_count = read_var_int(data, size, &mut offset);
    strings.allocate(L, string_count as usize);

    for i in 0..string_count {
        let length = read_var_int(data, size, &mut offset);

        *strings.data.add(i as usize) = luaS_newlstr(L, data.add(offset), length as usize);
        offset += length as usize;
    }

    // userdata type remapping table
    // for unknown userdata types, the entry will remap to common 'userdata' type
    let mut userdata_remapping = [LBC_TYPE_USERDATA.0 as u8; USERDATA_TYPE_LIMIT];

    if typesversion == 3 {
        let mut index: u8 = read(data, size, &mut offset);

        while index != 0 {
            let name = read_string(strings, data, size, &mut offset);

            if ((index - 1) as usize) < USERDATA_TYPE_LIMIT {
                if let Some(cb) = (*(*L).global).ecb.gettypemapping {
                    userdata_remapping[(index - 1) as usize] =
                        cb(L, getstr(name), (*name).len as usize);
                }
            }

            index = read(data, size, &mut offset);
        }
    }

    // proto table
    let proto_count = read_var_int(data, size, &mut offset);
    protos.allocate(L, proto_count as usize);

    for i in 0..proto_count {
        let p = lua_f_newproto(L);
        (*p).source = source;
        (*p).bytecodeid = i as c_int;
        (*p).funid = if (*(*L).global).lastprotoid == 0 {
            0
        } else {
            let id = (*(*L).global).lastprotoid;
            (*(*L).global).lastprotoid = (*(*L).global).lastprotoid.wrapping_add(1);
            id
        };

        (*p).maxstacksize = read(data, size, &mut offset);
        (*p).numparams = read(data, size, &mut offset);
        (*p).nups = read(data, size, &mut offset);
        (*p).is_vararg = read(data, size, &mut offset);

        if version >= 4 {
            (*p).flags = read(data, size, &mut offset);

            if typesversion == 1 {
                let typesize = read_var_int(data, size, &mut offset);

                if typesize != 0 {
                    let types = data.add(offset) as *mut u8;

                    LUAU_ASSERT!(typesize == 2 + (*p).numparams as u32);
                    LUAU_ASSERT!(*types.add(0) == LBC_TYPE_FUNCTION.0 as u8);
                    LUAU_ASSERT!(*types.add(1) == (*p).numparams);

                    // transform v1 into v2 format
                    let headersize = if typesize > 127 { 4usize } else { 3usize };

                    (*p).typeinfo =
                        luaM_newarray!(L, headersize + typesize as usize, u8, (*p).hdr.memcat);
                    (*p).sizetypeinfo = (headersize + typesize as usize) as c_int;

                    if headersize == 4 {
                        *(*p).typeinfo.add(0) = ((typesize & 127) | (1 << 7)) as u8;
                        *(*p).typeinfo.add(1) = (typesize >> 7) as u8;
                        *(*p).typeinfo.add(2) = 0;
                        *(*p).typeinfo.add(3) = 0;
                    } else {
                        *(*p).typeinfo.add(0) = typesize as u8;
                        *(*p).typeinfo.add(1) = 0;
                        *(*p).typeinfo.add(2) = 0;
                    }

                    core::ptr::copy_nonoverlapping(
                        types,
                        (*p).typeinfo.add(headersize),
                        typesize as usize,
                    );
                }

                offset += typesize as usize;
            } else if typesversion == 2 || typesversion == 3 {
                let typesize = read_var_int(data, size, &mut offset);

                if typesize != 0 {
                    let types = data.add(offset) as *mut u8;

                    (*p).typeinfo = luaM_newarray!(L, typesize as usize, u8, (*p).hdr.memcat);
                    (*p).sizetypeinfo = typesize as c_int;
                    core::ptr::copy_nonoverlapping(types, (*p).typeinfo, typesize as usize);
                    offset += typesize as usize;

                    if typesversion == 3 {
                        remap_userdata_types(
                            (*p).typeinfo as *mut c_char,
                            (*p).sizetypeinfo as usize,
                            userdata_remapping.as_mut_ptr(),
                            USERDATA_TYPE_LIMIT as u32,
                        );
                    }
                }
            }
        }

        let sizecode = read_var_int(data, size, &mut offset) as c_int;
        (*p).code = luaM_newarray!(L, sizecode as usize, Instruction, (*p).hdr.memcat);
        (*p).sizecode = sizecode;

        for j in 0..(*p).sizecode {
            *(*p).code.add(j as usize) = read::<u32>(data, size, &mut offset);
        }

        (*p).codeentry = (*p).code;

        let sizek = read_var_int(data, size, &mut offset) as c_int;
        (*p).k = luaM_newarray!(L, sizek as usize, TValue, (*p).hdr.memcat);
        (*p).sizek = sizek;

        // Initialize the constants to nil to ensure they have a valid state
        // in the event that some operation in the following loop fails with
        // an exception.
        for j in 0..(*p).sizek {
            setnilvalue!((*p).k.add(j as usize));
        }

        for j in 0..(*p).sizek {
            let k = (*p).k.add(j as usize);

            match read::<u8>(data, size, &mut offset) {
                LBC_CONSTANT_NIL_U8 => {
                    // All constants have already been pre-initialized to nil
                }

                LBC_CONSTANT_BOOLEAN_U8 => {
                    let v: u8 = read(data, size, &mut offset);
                    setbvalue!(k, v);
                }

                LBC_CONSTANT_NUMBER_U8 => {
                    let v: f64 = read(data, size, &mut offset);
                    setnvalue!(k, v);
                }

                LBC_CONSTANT_VECTOR_U8 => {
                    let x: f32 = read(data, size, &mut offset);
                    let y: f32 = read(data, size, &mut offset);
                    let z: f32 = read(data, size, &mut offset);
                    let w: f32 = read(data, size, &mut offset);
                    setvvalue!(k, x, y, z, w);
                }

                LBC_CONSTANT_STRING_U8 => {
                    let v = read_string(strings, data, size, &mut offset);
                    setsvalue!(L, k, v);
                }

                LBC_CONSTANT_IMPORT_U8 => {
                    let iid: u32 = read(data, size, &mut offset);
                    resolve_import_safe(L, envt, (*p).k, iid);
                    setobj!(L, k, (*L).top.sub(1));
                    (*L).top = (*L).top.sub(1);
                }

                LBC_CONSTANT_TABLE_U8 => {
                    let keys = read_var_int(data, size, &mut offset) as c_int;
                    let h = lua_h_new(L, 0, keys);
                    for _ in 0..keys {
                        let key = read_var_int(data, size, &mut offset) as c_int;
                        let val = luaH_set(L, h, (*p).k.add(key as usize) as *const TValue);
                        setnvalue!(val, 0.0);
                    }
                    sethvalue!(L, k, h);
                }

                LBC_CONSTANT_TABLE_WITH_CONSTANTS_U8 => {
                    let keys = read_var_int(data, size, &mut offset);
                    let h = lua_h_new(L, 0, keys as c_int);

                    let mut nil_keys: TempBuffer<i32> = TempBuffer::temp_buffer();
                    nil_keys.allocate(L, keys as usize);
                    let mut nil_keys_size: usize = 0;

                    for _ in 0..keys {
                        let key = read_var_int(data, size, &mut offset) as i32;
                        let val = luaH_set(L, h, (*p).k.add(key as usize) as *const TValue);
                        let constant_idx: i32 = read(data, size, &mut offset);
                        if constant_idx >= 0 {
                            let constant = (*p).k.add(constant_idx as usize);
                            if ttisnil!(constant) {
                                *nil_keys.data.add(nil_keys_size) = key;
                                nil_keys_size += 1;
                            } else {
                                setobj2t!(L, val, constant);
                                luaC_barriert!(L, h, constant);
                                continue;
                            }
                        }
                        setnvalue!(val, 0.0);
                    }

                    for idx in 0..nil_keys_size {
                        let key = *nil_keys.data.add(idx);
                        let val = luaH_set(L, h, (*p).k.add(key as usize) as *const TValue);
                        setnilvalue!(val);
                    }

                    sethvalue!(L, k, h);
                }

                LBC_CONSTANT_CLOSURE_U8 => {
                    let fid = read_var_int(data, size, &mut offset);
                    let proto = *protos.data.add(fid as usize);
                    let cl = lua_f_new_lclosure(L, (*proto).nups as c_int, envt, proto);
                    (*cl).preload = if (*cl).nupvalues > 0 { 1 } else { 0 };
                    setclvalue!(L, k, cl);
                }

                LBC_CONSTANT_CLASS_SHAPE_U8 => {
                    let cnid = read_var_int(data, size, &mut offset);
                    let classname = (*p).k.add(cnid as usize);
                    LUAU_ASSERT!(ttisstring!(classname));
                    let num_properties = read_var_int(data, size, &mut offset);
                    let num_methods = read_var_int(data, size, &mut offset);
                    let num_members = num_methods + num_properties;
                    let offset_to_member =
                        luaM_newarray!(L, num_members as usize, *mut TString, (*L).activememcat);
                    let members_to_offset = lua_h_new(L, 0, num_members as c_int);

                    for idx in 0..num_members {
                        let mid = read_var_int(data, size, &mut offset);
                        let member_name = (*p).k.add(mid as usize);
                        LUAU_ASSERT!(ttisstring!(member_name));
                        *offset_to_member.add(idx as usize) = tsvalue!(member_name) as *mut TString;
                        let val = lua_h_setstr(
                            L,
                            members_to_offset,
                            tsvalue!(member_name) as *mut TString,
                        );
                        setnvalue!(val, idx as f64);
                    }

                    (*members_to_offset).readonly = 1;

                    let lco = lua_r_newclass(
                        L,
                        tsvalue!(classname) as *mut TString,
                        members_to_offset,
                        offset_to_member,
                        num_properties as c_int,
                        num_methods as c_int,
                    );
                    setclassvalue!(L, k, lco);
                }

                LBC_CONSTANT_INTEGER_U8 => {
                    let is_negative: u8 = read(data, size, &mut offset);
                    let magnitude = read_var_int_64(data, size, &mut offset);
                    let value = if is_negative != 0 {
                        (!magnitude).wrapping_add(1) as i64
                    } else {
                        magnitude as i64
                    };
                    setlvalue!(k, value);
                }

                _ => {
                    LUAU_ASSERT!(false);
                }
            }
        }

        if luaur_common::FFlag::LuauUdataDirectAccess6.get() {
            let mut instruction = (*p).code;
            let end = (*p).code.add((*p).sizecode as usize);

            while (instruction as usize) < (end as usize) {
                let mut target_op = -1i32;

                match LuauOpcode::from(LUAU_INSN_OP(*instruction) as u8) {
                    LuauOpcode::LOP_GETTABLEKS => {
                        target_op = LuauOpcode::LOP_GETUDATAKS as u8 as i32;
                    }

                    LuauOpcode::LOP_SETTABLEKS => {
                        target_op = LuauOpcode::LOP_SETUDATAKS as u8 as i32;
                    }

                    LuauOpcode::LOP_NAMECALL => {
                        target_op = LuauOpcode::LOP_NAMECALLUDATA as u8 as i32;
                    }

                    _ => {}
                }

                if target_op != -1 {
                    LUAU_ASSERT!(*instruction.add(1) < sizek as u32);

                    // We take over the upper 16 bits of AUX - so no constants with big indices.
                    if *instruction.add(1) < 0x10000 {
                        let k = (*p).k.add(*instruction.add(1) as usize);
                        let s = tsvalue!(k) as *mut TString;

                        luaS_updateatom!(L, s);

                        if (*s).atom >= 0 {
                            *instruction = (*instruction & 0xffffff00) | target_op as u32;
                        }
                    }
                }

                instruction = instruction
                    .add(getOpLength(LuauOpcode::from(LUAU_INSN_OP(*instruction) as u8)) as usize);
            }
        }

        let sizep = read_var_int(data, size, &mut offset) as c_int;
        (*p).p = luaM_newarray!(L, sizep as usize, *mut Proto, (*p).hdr.memcat);
        (*p).sizep = sizep;

        for j in 0..(*p).sizep {
            let fid = read_var_int(data, size, &mut offset);
            *(*p).p.add(j as usize) = *protos.data.add(fid as usize);
        }

        (*p).linedefined = read_var_int(data, size, &mut offset) as c_int;
        (*p).debugname = read_string(strings, data, size, &mut offset);

        let lineinfo: u8 = read(data, size, &mut offset);

        if lineinfo != 0 {
            (*p).linegaplog2 = read::<u8>(data, size, &mut offset) as c_int;

            let intervals = (((*p).sizecode - 1) >> (*p).linegaplog2) + 1;
            let absoffset = ((*p).sizecode + 3) & !3;

            let sizelineinfo = absoffset + intervals * core::mem::size_of::<c_int>() as c_int;
            (*p).lineinfo = luaM_newarray!(L, sizelineinfo as usize, u8, (*p).hdr.memcat);
            (*p).sizelineinfo = sizelineinfo;

            (*p).abslineinfo = (*p).lineinfo.add(absoffset as usize) as *mut c_int;

            let mut lastoffset: u8 = 0;
            for j in 0..(*p).sizecode {
                lastoffset = lastoffset.wrapping_add(read::<u8>(data, size, &mut offset));
                *(*p).lineinfo.add(j as usize) = lastoffset;
            }

            let mut lastline: c_int = 0;
            for j in 0..intervals {
                lastline = lastline.wrapping_add(read::<i32>(data, size, &mut offset));
                *(*p).abslineinfo.add(j as usize) = lastline;
            }
        }

        let debuginfo: u8 = read(data, size, &mut offset);

        if debuginfo != 0 {
            let sizelocvars = read_var_int(data, size, &mut offset) as c_int;
            (*p).locvars = luaM_newarray!(L, sizelocvars as usize, LocVar, (*p).hdr.memcat);
            (*p).sizelocvars = sizelocvars;

            for j in 0..(*p).sizelocvars {
                let locvar = (*p).locvars.add(j as usize);
                (*locvar).varname = read_string(strings, data, size, &mut offset);
                (*locvar).startpc = read_var_int(data, size, &mut offset) as c_int;
                (*locvar).endpc = read_var_int(data, size, &mut offset) as c_int;
                (*locvar).reg = read(data, size, &mut offset);
            }

            let sizeupvalues = read_var_int(data, size, &mut offset) as c_int;
            LUAU_ASSERT!(sizeupvalues == (*p).nups as c_int);

            (*p).upvalues = luaM_newarray!(L, sizeupvalues as usize, *mut TString, (*p).hdr.memcat);
            (*p).sizeupvalues = sizeupvalues;

            for j in 0..(*p).sizeupvalues {
                *(*p).upvalues.add(j as usize) = read_string(strings, data, size, &mut offset);
            }
        }

        if version >= 11 {
            LUAU_ASSERT!(luaur_common::FFlag::LuauCallFeedback.get());
            (*p).feedbackvecsize = read_var_int(data, size, &mut offset);

            if (*p).feedbackvecsize > 0 {
                (*p).feedbackvec = luaM_newarray!(
                    L,
                    (*p).feedbackvecsize as usize,
                    FeedbackVectorSlot,
                    (*p).hdr.memcat
                );
            }
            for j in 0..(*p).feedbackvecsize {
                let slottype: u8 = read(data, size, &mut offset);
                LUAU_ASSERT!(slottype == LuauFeedbackType::LFT_CALLTARGET as u8);
                let slot = (*p).feedbackvec.add(j as usize);
                (*slot).kind = FeedbackVectorSlotKind::CALL_TARGET;
                (*slot).data.call_target.pc = read_var_int(data, size, &mut offset);
                (*slot).data.call_target.proto = 0;
                (*slot).data.call_target.hits = 0;
            }
        }

        *protos.data.add(i as usize) = p;
    }

    // "main" proto is pushed to Lua stack
    let mainid = read_var_int(data, size, &mut offset);
    let main = *protos.data.add(mainid as usize);

    let thread_obj = L as *mut GCObject;
    if isblack!(thread_obj) {
        lua_c_barrierback(L, thread_obj, &mut (*L).gclist);
    }

    let cl = lua_f_new_lclosure(L, 0, envt, main);
    setclvalue!(L, (*L).top, cl);
    incr_top!(L);

    0
}

unsafe fn resolve_import_safe(L: *mut lua_State, _env: *mut LuaTable, k: *mut TValue, id: u32) {
    let mut ri = ResolveImport { k, id };

    if (*(*L).gt).safeenv != 0 {
        // luaD_pcall will make sure that if any C/Lua calls during import resolution fail, the thread state is restored back
        let old_top = lua_gettop(L);
        let status = luaD_pcall(
            L,
            Some(ResolveImport::run),
            &mut ri as *mut ResolveImport as *mut core::ffi::c_void,
            savestack!(L, (*L).top) as isize,
            0,
        );
        LUAU_ASSERT!(old_top + 1 == lua_gettop(L)); // if an error occurred, luaD_pcall saves it on stack

        if status != lua_Status::LUA_OK as c_int {
            // replace error object with nil
            setnilvalue!((*L).top.sub(1));
        }
    } else {
        setnilvalue!((*L).top);
        (*L).top = (*L).top.add(1);
    }
}

unsafe fn c_strlen(s: *const c_char) -> usize {
    let mut len = 0usize;
    while *s.add(len) != 0 {
        len += 1;
    }
    len
}

unsafe fn c_string_lossy(s: *const c_char) -> String {
    String::from_utf8_lossy(core::slice::from_raw_parts(s as *const u8, c_strlen(s))).into_owned()
}

unsafe fn push_chunk_prefixed_slice(
    L: *mut lua_State,
    chunkid: *const c_char,
    bytes: *const c_char,
    len: usize,
) {
    let prefix = core::slice::from_raw_parts(chunkid as *const u8, c_strlen(chunkid));
    let payload = core::slice::from_raw_parts(bytes as *const u8, len);
    let mut message = Vec::with_capacity(prefix.len() + payload.len());
    message.extend_from_slice(prefix);
    message.extend_from_slice(payload);
    lua_pushlstring(L, message.as_ptr() as *const c_char, message.len());
}

unsafe fn push_rust_string(L: *mut lua_State, message: &str) {
    lua_pushlstring(L, message.as_ptr() as *const c_char, message.len());
}
