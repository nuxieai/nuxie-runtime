use crate::records::bc_function::VmConst;
use crate::records::bc_load_nil::BcLoadNil;
use crate::records::call_inliner::CallInliner;
use crate::type_aliases::reg::Reg;

impl<'a> CallInliner<'a> {
    pub fn fill_under_call_arguments(&mut self) {
        if self.call_params.len() as u8 >= self.target.numparams {
            return;
        }

        let inline_entry_block = self.map_block_op(self.target.entry_block);
        let call_param_size = self.call_params.len() as u8;
        self.call_params
            .resize(self.target.numparams as usize, Default::default());

        for param in (call_param_size..self.target.numparams).rev() {
            let mut load_nil = BcLoadNil::<VmConst>::create(self.caller);
            load_nil.set_out_reg(self.target_reg + 1 + param);
            load_nil.prepend_to(inline_entry_block);
            self.call_params[param as usize] = load_nil.op();
        }
    }
}
