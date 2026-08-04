use alloc::vec::Vec;
use core::marker::PhantomData;
use luaur_common::enums::luau_opcode::LuauOpcode;

use crate::methods::bc_function_as::BcInstType;
use crate::methods::bc_inst_helper_create::BcInstHelperCreate;
use crate::records::bc_function::{BcFunction, VmConst};
use crate::records::bc_inst::BcInst;
use crate::records::bc_inst_helper::BcInstHelper;
use crate::records::bc_op::BcOp;
use crate::records::bc_ref::BcRef;

#[derive(Debug)]
pub struct BcGetImport<'a, T = VmConst> {
    pub(crate) base: BcInstHelper<'a>,
    _marker: PhantomData<T>,
}

impl<'a, T> BcGetImport<'a, T> {
    pub const K_PATH_START_INPUT: u32 = 2;

    pub fn from(graph: *mut BcFunction, inst: BcRef<'a, BcInst>) -> Self {
        Self {
            base: BcInstHelper::new(unsafe { &mut *graph }, inst),
            _marker: PhantomData,
        }
    }

    pub fn import(&mut self) -> BcRef<'_, VmConst> {
        self.base.get_vm_const(0)
    }

    pub fn path_length(&mut self) -> i32 {
        self.base.int_imm_input(1)
    }

    pub fn import_path(&self) -> Vec<BcOp> {
        self.base.slice_inputs(Self::K_PATH_START_INPUT)
    }
}

impl<T> BcInstType for BcGetImport<'_, T> {
    const OPCODE: i32 = LuauOpcode::LOP_GETIMPORT as i32;
}

impl<T> BcInstHelperCreate for BcGetImport<'_, T> {
    const OPCODE: LuauOpcode = LuauOpcode::LOP_GETIMPORT;
}
