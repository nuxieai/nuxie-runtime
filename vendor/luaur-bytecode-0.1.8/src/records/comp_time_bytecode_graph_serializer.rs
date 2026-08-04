use crate::records::bc_inst::BcInst;
use crate::records::bc_vm_const::BcVmConst;
use crate::records::bytecode_builder::BytecodeBuilder;
use crate::records::bytecode_graph_serializer::BytecodeGraphSerializer;
use crate::type_aliases::comp_time_bc_function::CompTimeBcFunction;
use alloc::vec::Vec;

#[derive(Debug)]
pub struct CompTimeBytecodeGraphSerializer {
    pub(crate) base: BytecodeGraphSerializer<'static>,
    pub(crate) consts: Vec<u16>,
}

impl CompTimeBytecodeGraphSerializer {
    pub fn comp_time_bytecode_graph_serializer_comp_time_bytecode_graph_serializer(
        bcb: &mut BytecodeBuilder,
        fn_: &mut CompTimeBcFunction,
        consts: &mut Vec<u16>,
    ) -> Self {
        // BytecodeGraphSerializer is parameterized by the lifetimes of `bcb`/`fn_`.
        // This record requires `base` to be stored as `'static`, so we extend the
        // lifetimes via raw pointers (matching the original C++ serializer lifetime assumptions).
        let bcb_ptr: *mut BytecodeBuilder = bcb;
        let fn_ptr: *mut CompTimeBcFunction = fn_;

        let mut base =
            BytecodeGraphSerializer::new(unsafe { &mut *bcb_ptr }, unsafe { &mut *fn_ptr });
        base.consts = Some(core::mem::take(consts));

        Self {
            base,
            consts: Vec::new(),
        }
    }

    pub fn get_vm_const_input_raw(&mut self, insn: &mut BcInst, index: u8) -> u32 {
        let cid = BytecodeGraphSerializer::get_vm_const_input_raw(&mut self.base, insn, index);
        assert!((cid as usize) < self.consts.len());
        self.consts[cid as usize] as u32
    }

    pub fn emit_bytecode(&mut self) -> Vec<u32> {
        self.base.emit_bytecode()
    }

    pub fn error(&self) -> bool {
        self.base.error
    }
}
