use core::marker::PhantomData;

use luaur_common::enums::luau_opcode::LuauOpcode;

use crate::methods::bc_inst_helper_create::BcInstHelperCreate;
use crate::records::bc_function::{BcFunction, VmConst};
use crate::records::bc_inst_helper::BcInstHelper;
use crate::records::bc_op::BcOp;
use crate::type_aliases::reg::Reg;

#[derive(Debug)]
pub struct BcMove<'a, T = VmConst> {
    pub(crate) base: BcInstHelper<'a>,
    _marker: PhantomData<T>,
}

impl<'a, T> BcMove<'a, T> {
    pub fn create(graph: &'a mut BcFunction) -> Self {
        Self {
            base: BcInstHelper::create::<Self>(graph),
            _marker: PhantomData,
        }
    }

    pub fn set_src(&mut self, value: BcOp) {
        self.base.set_bc_op(0, value);
    }

    pub fn set_out_reg(&mut self, out: Reg) {
        self.base.set_out_reg(out);
    }

    pub fn append_to(&mut self, block: BcOp) {
        self.base.append_to(block);
    }

    pub fn op(&self) -> BcOp {
        self.base.op()
    }
}

impl<T> BcInstHelperCreate for BcMove<'_, T> {
    const OPCODE: LuauOpcode = LuauOpcode::LOP_MOVE;
}
