use core::marker::PhantomData;

use luaur_common::enums::luau_opcode::LuauOpcode;

use crate::methods::bc_function_as::BcInstType;
use crate::methods::bc_inst_helper_create::BcInstHelperCreate;
use crate::records::bc_function::{BcFunction, VmConst};
use crate::records::bc_inst::BcInst;
use crate::records::bc_inst_helper::BcInstHelper;
use crate::records::bc_op::BcOp;
use crate::records::bc_ref::BcRef;
use crate::type_aliases::reg::Reg;

#[derive(Debug)]
pub struct BcGetTableKS<'a, T = VmConst> {
    pub(crate) base: BcInstHelper<'a>,
    _marker: PhantomData<T>,
}

impl<'a, T> BcGetTableKS<'a, T> {
    pub fn from(graph: *mut BcFunction, inst: BcRef<'a, BcInst>) -> Self {
        Self {
            base: BcInstHelper::new(unsafe { &mut *graph }, inst),
            _marker: PhantomData,
        }
    }

    pub fn source(&mut self) -> BcOp {
        self.base.get_bc_op(0)
    }

    pub fn set_source(&mut self, value: BcOp) {
        self.base.set_bc_op(0, value);
    }

    pub fn set_hint(&mut self, value: u32) {
        self.base.set_imm_input(1, value as i32);
    }

    pub fn set_key(&mut self, value: u32) {
        self.base.set_vm_const(2, value);
    }

    pub fn create(graph: &'a mut BcFunction) -> Self {
        Self {
            base: BcInstHelper::create::<Self>(graph),
            _marker: PhantomData,
        }
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

impl<T> BcInstType for BcGetTableKS<'_, T> {
    const OPCODE: i32 = LuauOpcode::LOP_GETTABLEKS as i32;
}

impl<T> BcInstHelperCreate for BcGetTableKS<'_, T> {
    const OPCODE: LuauOpcode = LuauOpcode::LOP_GETTABLEKS;
}
