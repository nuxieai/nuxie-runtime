use core::marker::PhantomData;

use luaur_common::enums::luau_opcode::LuauOpcode;

use crate::methods::bc_function_as::BcInstType;
use crate::methods::bc_inst_helper_create::BcInstHelperCreate;
use crate::records::bc_function::VmConst;
use crate::records::bc_inst_helper::BcInstHelper;
use crate::records::bc_op::BcOp;
use crate::type_aliases::reg::Reg;

#[derive(Debug)]
pub struct BcLoadNil<'a, T = VmConst> {
    pub(crate) base: BcInstHelper<'a>,
    _marker: PhantomData<T>,
}

impl<T> BcInstType for BcLoadNil<'_, T> {
    const OPCODE: i32 = LuauOpcode::LOP_LOADNIL as i32;
}

impl<T> BcInstHelperCreate for BcLoadNil<'_, T> {
    const OPCODE: LuauOpcode = LuauOpcode::LOP_LOADNIL;
}

impl<'a, T> BcLoadNil<'a, T> {
    pub fn create(graph: &'a mut crate::records::bc_function::BcFunction) -> Self {
        Self {
            base: BcInstHelper::create::<Self>(graph),
            _marker: PhantomData,
        }
    }

    pub fn set_out_reg(&mut self, out: Reg) {
        self.base.set_out_reg(out);
    }

    pub fn prepend_to(&mut self, block: BcOp) {
        self.base.prepend_to(block);
    }

    pub fn append_to(&mut self, block: BcOp) {
        self.base.append_to(block);
    }

    pub fn op(&self) -> BcOp {
        self.base.op()
    }
}
