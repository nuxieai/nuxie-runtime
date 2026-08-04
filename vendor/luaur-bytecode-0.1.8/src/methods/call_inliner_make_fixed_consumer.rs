use crate::records::bc_call::BcCall;
use crate::records::bc_call_fb::BcCallFB;
use crate::records::bc_function::BcFunction;
use crate::records::bc_inst::BcInst;
use crate::records::bc_ref::BcRef;
use crate::records::bc_return::BcReturn;
use crate::records::bc_set_list::BcSetList;
use luaur_common::enums::luau_opcode::LuauOpcode;
use luaur_common::macros::luau_unreachable::LUAU_UNREACHABLE;

impl<'a> crate::records::call_inliner::CallInliner<'a> {
    pub fn make_fixed_consumer(&mut self, inst: &mut BcRef<'a, BcInst>) {
        match unsafe { (*inst.operator_arrow()).op } {
            LuauOpcode::LOP_SETLIST => {
                let mut set_list =
                    BcSetList::<crate::records::bc_op::BcOp>::from(self.caller, inst.clone());
                let count = set_list.params().len() as u32;
                set_list.set_count(count);
            }
            LuauOpcode::LOP_RETURN => {
                let mut ret =
                    BcReturn::<crate::records::bc_op::BcOp>::from(self.caller, inst.clone());
                let count = ret.values().len() as u32;
                ret.set_return_count(count);
            }
            LuauOpcode::LOP_CALLFB => {
                let mut call_fb =
                    BcCallFB::<crate::records::bc_op::BcOp>::from(self.caller, inst.clone());
                let count = call_fb.params().len() as u32;
                call_fb.set_param_count(count);
            }
            LuauOpcode::LOP_CALL => {
                let mut call =
                    BcCall::<crate::records::bc_op::BcOp>::from(self.caller, inst.clone());
                let count = call.params().len() as u32;
                call.set_param_count(count);
            }
            _ => {
                LUAU_UNREACHABLE!();
            }
        }
    }
}
