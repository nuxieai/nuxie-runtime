use crate::records::bc_function::BcFunction;
use crate::records::bc_inst::BcInst;
use crate::records::bc_ref::BcRef;

#[derive(Debug)]
pub struct BcInstHelper<'a> {
    pub(crate) graph: &'a mut BcFunction,
    pub(crate) inst: BcRef<'a, BcInst>,
}

impl<'a> BcInstHelper<'a> {
    pub(crate) fn new(graph: &'a mut BcFunction, inst: BcRef<'a, BcInst>) -> Self {
        Self { graph, inst }
    }
}
