use crate::records::bc_function::BcFunction;
use crate::records::sccp::Sccp;
use crate::records::sccp_interpreter::SccpInterpreter;
use crate::records::sccp_state::SccpState;
use crate::records::vm_const_ops::VmConstOps;
use alloc::boxed::Box;

impl<'func, 'ops> Sccp<'func, 'ops> {
    pub(crate) fn new(func: &'func mut BcFunction, impl_: &'ops dyn VmConstOps) -> Self {
        let mut state = Box::new(SccpState::default());
        let state_ptr = &mut *state as *mut SccpState;
        Self {
            func,
            impl_,
            state,
            interpreter: unsafe { SccpInterpreter::new(impl_, state_ptr) },
            block_uses: Default::default(),
            flow_worklist: Default::default(),
            flow_worklist_set: Default::default(),
            ssa_worklist: Default::default(),
            _func: core::marker::PhantomData,
        }
    }
}
