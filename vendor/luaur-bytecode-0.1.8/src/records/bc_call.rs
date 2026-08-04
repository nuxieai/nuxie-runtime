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
pub struct BcCall<'a, T = VmConst> {
    pub(crate) base: BcInstHelper<'a>,
    _marker: PhantomData<T>,
}

impl<'a, T> BcCall<'a, T> {
    pub const K_PARAM_START_INPUT: u32 = 3;

    pub fn from(graph: *mut BcFunction, inst: BcRef<'a, BcInst>) -> Self {
        Self {
            base: BcInstHelper::new(unsafe { &mut *graph }, inst),
            _marker: PhantomData,
        }
    }

    pub fn params(&self) -> Vec<BcOp> {
        self.base.slice_inputs(Self::K_PARAM_START_INPUT)
    }

    pub fn param_count(&mut self) -> i32 {
        self.base.int_imm_input(0)
    }

    pub fn set_param_count(&mut self, value: u32) {
        self.base.set_imm_input(0, value as i32);
    }

    pub fn return_count(&mut self) -> i32 {
        self.base.int_imm_input(1)
    }

    pub fn set_return_count(&mut self, value: u32) {
        self.base.set_imm_input(1, value as i32);
    }

    pub fn target(&mut self) -> BcOp {
        self.base.get_bc_op(2)
    }

    pub fn set_target(&mut self, value: BcOp) {
        self.base.set_bc_op(2, value);
    }
}

impl<T> BcInstType for BcCall<'_, T> {
    const OPCODE: i32 = LuauOpcode::LOP_CALL as i32;
}

impl<T> BcInstHelperCreate for BcCall<'_, T> {
    const OPCODE: LuauOpcode = LuauOpcode::LOP_CALL;
}
