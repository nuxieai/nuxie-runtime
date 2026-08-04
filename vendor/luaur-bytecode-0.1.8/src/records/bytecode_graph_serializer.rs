use crate::records::bc_function::{BcFunction, VmConst};
use crate::records::bytecode_builder::BytecodeBuilder;
use crate::records::jump_info::JumpInfo;
use crate::type_aliases::jumps::Jumps;
use alloc::vec::Vec;

#[derive(Debug)]
pub struct BytecodeGraphSerializer<'a> {
    pub(crate) bcb: &'a mut BytecodeBuilder,
    pub(crate) func: &'a mut BcFunction,
    pub(crate) jumps: Jumps,
    pub(crate) error: bool,
    pub(crate) consts: Option<Vec<u16>>,
}

impl<'a> BytecodeGraphSerializer<'a> {
    pub(crate) fn new(bcb: &'a mut BytecodeBuilder, func: &'a mut BcFunction) -> Self {
        Self {
            bcb,
            func,
            jumps: Vec::new(),
            error: false,
            consts: None,
        }
    }
}
