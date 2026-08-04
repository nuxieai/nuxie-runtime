use crate::records::bc_function::BcFunction;
use crate::records::bc_vm_const_impl::BcVmConstImpl;

impl BcVmConstImpl {
    pub fn new(func: &mut BcFunction) -> Self {
        Self { func }
    }
}
