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
pub struct BcCallFB<'a, T = VmConst> {
    pub(crate) base: BcInstHelper<'a>,
    _marker: PhantomData<T>,
}

impl<'a, T> BcCallFB<'a, T> {
    pub const K_PARAM_START_INPUT: u32 = 4;

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

    pub fn fb_slot(&mut self) -> i32 {
        self.base.int_imm_input(2)
    }

    pub fn set_fb_slot(&mut self, value: u32) {
        self.base.set_imm_input(2, value as i32);
    }

    pub fn target(&mut self) -> BcOp {
        self.base.get_bc_op(3)
    }

    pub fn set_target(&mut self, value: BcOp) {
        self.base.set_bc_op(3, value);
    }

    pub fn op(&self) -> BcOp {
        self.base.op()
    }
}

impl<T> BcInstType for BcCallFB<'_, T> {
    const OPCODE: i32 = LuauOpcode::LOP_CALLFB as i32;
}

impl<T> BcInstHelperCreate for BcCallFB<'_, T> {
    const OPCODE: LuauOpcode = LuauOpcode::LOP_CALLFB;
}

impl<'a, T> From<(&'a BcFunction, BcRef<'a, BcInst>)> for BcCallFB<'a, T> {
    fn from((graph, inst): (&'a BcFunction, BcRef<'a, BcInst>)) -> Self {
        Self::from(graph as *const BcFunction as *mut BcFunction, inst)
    }
}
