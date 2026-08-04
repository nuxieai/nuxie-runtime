use crate::records::bc_function::BcFunction;

#[derive(Debug, Clone, Copy)]
pub struct BcVmConstImpl {
    pub func: *mut BcFunction,
}
