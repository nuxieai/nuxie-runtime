use core::marker::PhantomData;

use luaur_common::enums::luau_opcode::LuauOpcode;

use crate::methods::bc_function_as::BcInstType;
use crate::methods::bc_inst_helper_create::BcInstHelperCreate;
use crate::records::bc_function::{BcFunction, VmConst};
use crate::records::bc_inst::BcInst;
use crate::records::bc_inst_helper::BcInstHelper;
use crate::records::bc_ref::BcRef;
use crate::type_aliases::reg::Reg;

#[derive(Debug)]
pub struct BcGetVarArgs<'a, T = VmConst> {
    pub(crate) base: BcInstHelper<'a>,
    _marker: PhantomData<T>,
}

impl<'a, T> BcGetVarArgs<'a, T> {
    pub const K_START_REG_INPUT: u32 = 0;

    pub fn from(graph: *mut BcFunction, inst: BcRef<'a, BcInst>) -> Self {
        Self {
            base: BcInstHelper::new(unsafe { &mut *graph }, inst),
            _marker: PhantomData,
        }
    }

    pub fn values_count(&mut self) -> i32 {
        self.base.int_imm_input(1)
    }

    pub fn set_values_count(&mut self, value: u32) {
        self.base.set_imm_input(1, value as i32);
    }

    pub fn start_reg(&self) -> Reg {
        self.base.operator_deref().ops[Self::K_START_REG_INPUT as usize].index as Reg
    }
}

impl<T> BcInstType for BcGetVarArgs<'_, T> {
    const OPCODE: i32 = LuauOpcode::LOP_GETVARARGS as i32;
}

impl<T> BcInstHelperCreate for BcGetVarArgs<'_, T> {
    const OPCODE: LuauOpcode = LuauOpcode::LOP_GETVARARGS;
}
