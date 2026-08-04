use core::marker::PhantomData;

use luaur_common::enums::luau_opcode::LuauOpcode;

use crate::methods::bc_function_as::BcInstType;
use crate::methods::bc_inst_helper_create::BcInstHelperCreate;
use crate::records::bc_function::{BcFunction, VmConst};
use crate::records::bc_inst_helper::BcInstHelper;
use crate::records::bc_op::BcOp;

#[derive(Debug)]
pub struct BcJump<'a, T = VmConst> {
    pub(crate) base: BcInstHelper<'a>,
    _marker: PhantomData<T>,
}

impl<'a, T> BcJump<'a, T> {
    pub fn create(graph: &'a mut BcFunction) -> Self {
        Self {
            base: BcInstHelper::create::<Self>(graph),
            _marker: PhantomData,
        }
    }

    pub fn set_target(&mut self, block: BcOp) {
        self.base.set_bc_op(0, block);
    }

    pub fn append_to(&mut self, block: BcOp) {
        self.base.append_to(block);
    }
}

impl<T> BcInstType for BcJump<'_, T> {
    const OPCODE: i32 = LuauOpcode::LOP_JUMP as i32;
}

impl<T> BcInstHelperCreate for BcJump<'_, T> {
    const OPCODE: LuauOpcode = LuauOpcode::LOP_JUMP;
}
