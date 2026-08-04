use crate::records::bc_get_var_args::BcGetVarArgs;
use crate::records::bc_load_nil::BcLoadNil;
use crate::records::bc_move::BcMove;
use crate::records::bc_op::BcOp;
use crate::records::call_inliner::CallInliner;
use crate::type_aliases::reg::Reg;

impl<'a> CallInliner<'a> {
    pub fn replace_get_var_arg(&mut self, caller_block_op: BcOp, target_get_var_args_op: BcOp) {
        let target = self.target as *mut crate::records::bc_function::BcFunction;
        let target_get_var_args_ref = unsafe { (&*target).inst(target_get_var_args_op) };
        let mut get_var_args = BcGetVarArgs::<crate::records::bc_function::VmConst>::from(
            target,
            target_get_var_args_ref,
        );
        let count = get_var_args.values_count();
        let count = if count < 0 {
            std::cmp::max(
                0,
                self.call_params.len() as i32 - self.target.numparams as i32,
            ) as usize
        } else {
            count as usize
        };

        let mut moves = Vec::new();
        for i in 0..count {
            let target_reg = get_var_args.start_reg() as u32 + i as u32;
            let caller_reg = self.map_to_caller_reg(target_reg as Reg) as Reg;

            if (self.target.numparams as usize + i) < self.call_params.len() {
                let mut move_op =
                    BcMove::<crate::records::bc_function::VmConst>::create(self.caller);
                let src_op = self.call_params[self.target.numparams as usize + i];
                move_op.set_src(src_op);
                move_op.set_out_reg(caller_reg);
                move_op.append_to(caller_block_op);
                moves.push(move_op.op());
            } else {
                let mut load_nil =
                    BcLoadNil::<crate::records::bc_function::VmConst>::create(self.caller);
                load_nil.set_out_reg(caller_reg);
                load_nil.append_to(caller_block_op);
                moves.push(load_nil.op());
            }
        }

        self.var_arg_moves.try_insert(target_get_var_args_op, moves);
    }
}
