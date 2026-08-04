use core::marker::PhantomData;

use luaur_common::enums::luau_opcode::LuauOpcode;

use crate::methods::bc_function_as::BcInstType;
use crate::methods::bc_inst_helper_create::BcInstHelperCreate;
use crate::records::bc_function::VmConst;
use crate::records::bc_inst_helper::BcInstHelper;
use crate::records::bc_op::BcOp;

#[derive(Debug)]
pub struct BcCmpProto<'a, T = VmConst> {
    pub(crate) base: BcInstHelper<'a>,
    _marker: PhantomData<T>,
}

impl<'a, T> BcCmpProto<'a, T> {
    pub fn create(graph: &'a mut crate::records::bc_function::BcFunction) -> Self {
        Self {
            base: BcInstHelper::create::<Self>(graph),
            _marker: PhantomData,
        }
    }

    pub fn set_closure(&mut self, value: BcOp) {
        self.base.set_bc_op(0, value);
    }

    pub fn set_proto_id(&mut self, value: u32) {
        self.base.set_imm_input(1, value as i32);
    }

    pub fn set_fallback(&mut self, value: BcOp) {
        self.base.set_bc_op(2, value);
    }

    pub fn append_to(&mut self, block: BcOp) {
        self.base.append_to(block);
    }
}

impl<T> BcInstType for BcCmpProto<'_, T> {
    const OPCODE: i32 = LuauOpcode::LOP_CMPPROTO as i32;
}

impl<T> BcInstHelperCreate for BcCmpProto<'_, T> {
    const OPCODE: LuauOpcode = LuauOpcode::LOP_CMPPROTO;
}
