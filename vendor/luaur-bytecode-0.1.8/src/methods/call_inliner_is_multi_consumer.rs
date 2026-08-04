use crate::records::bc_call::BcCall;
use crate::records::bc_call_fb::BcCallFB;
use crate::records::bc_function::{BcFunction, VmConst};
use crate::records::bc_inst::BcInst;
use crate::records::bc_ref::BcRef;
use crate::records::bc_return::BcReturn;
use crate::records::bc_set_list::BcSetList;
use crate::records::call_inliner::CallInliner;
use luaur_common::enums::luau_opcode::LuauOpcode;

impl<'a> CallInliner<'a> {
    pub fn is_multi_consumer(&self, inst: &BcRef<'a, BcInst>) -> bool {
        let op = unsafe { (*inst.operator_arrow()).op };
        let caller = self.caller as *const BcFunction as *mut BcFunction;
        match op {
            LuauOpcode::LOP_SETLIST => BcSetList::<VmConst>::from(caller, *inst).count() < 0,
            LuauOpcode::LOP_RETURN => BcReturn::<VmConst>::from(caller, *inst).return_count() < 0,
            LuauOpcode::LOP_CALLFB => BcCallFB::<VmConst>::from(caller, *inst).param_count() < 0,
            LuauOpcode::LOP_CALL => BcCall::<VmConst>::from(caller, *inst).param_count() < 0,
            _ => false,
        }
    }
}
