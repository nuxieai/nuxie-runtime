use crate::records::sccp_interpreter::SccpInterpreter;

impl<'a> SccpInterpreter<'a> {
    pub(crate) unsafe fn new(
        impl_: &'a dyn crate::records::vm_const_ops::VmConstOps,
        state: *mut crate::records::sccp_state::SccpState,
    ) -> Self {
        Self { impl_, state }
    }
}
