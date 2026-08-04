use core::marker::PhantomData;

use alloc::vec::Vec;
use luaur_common::enums::luau_opcode::LuauOpcode;

use crate::methods::bc_function_as::BcInstType;
use crate::methods::bc_inst_helper_create::BcInstHelperCreate;
use crate::records::bc_function::{BcFunction, VmConst};
use crate::records::bc_inst::BcInst;
use crate::records::bc_inst_helper::BcInstHelper;
use crate::records::bc_op::BcOp;
use crate::records::bc_ref::BcRef;

#[derive(Debug)]
pub struct BcReturn<'a, T = VmConst> {
    pub(crate) base: BcInstHelper<'a>,
    _marker: PhantomData<T>,
}

impl<'a, T> BcReturn<'a, T> {
    pub const K_VALUES_START_INPUT: u32 = 1;

    pub fn from(graph: *mut BcFunction, inst: BcRef<'a, BcInst>) -> Self {
        Self {
            base: BcInstHelper::new(unsafe { &mut *graph }, inst),
            _marker: PhantomData,
        }
    }

    pub fn return_count(&mut self) -> i32 {
        self.base.int_imm_input(0)
    }

    pub fn set_return_count(&mut self, value: u32) {
        self.base.set_imm_input(0, value as i32);
    }

    pub fn values(&mut self) -> Vec<BcOp> {
        if self.return_count() == 0 {
            Vec::new()
        } else {
            self.base.slice_inputs(Self::K_VALUES_START_INPUT)
        }
    }
}

impl<T> BcInstType for BcReturn<'_, T> {
    const OPCODE: i32 = LuauOpcode::LOP_RETURN as i32;
}

impl<T> BcInstHelperCreate for BcReturn<'_, T> {
    const OPCODE: LuauOpcode = LuauOpcode::LOP_RETURN;
}
