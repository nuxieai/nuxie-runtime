use crate::records::bc_function::BcFunction;
use crate::records::sccp::Sccp;

impl<'func, 'ops> Sccp<'func, 'ops> {
    pub(crate) fn func_mut(&mut self) -> &mut BcFunction {
        unsafe { &mut *self.func }
    }
}
