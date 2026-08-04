use crate::records::bc_function::BcFunction;
use crate::records::sccp::Sccp;
use crate::records::vm_const_ops::VmConstOps;

pub fn fold_constants(func: &mut BcFunction, impl_: &dyn VmConstOps) {
    let mut sccp = Sccp::new(func, impl_);
    sccp.propagate();
    sccp.rewrite();
}
