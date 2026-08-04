use crate::records::sccp_state::SccpState;
use crate::records::vm_const_ops::VmConstOps;

pub struct SccpInterpreter<'a> {
    pub(crate) impl_: &'a dyn VmConstOps,
    pub(crate) state: *mut SccpState,
}
