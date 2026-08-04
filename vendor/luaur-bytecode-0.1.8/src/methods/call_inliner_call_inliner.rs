use crate::records::bc_call_fb::BcCallFB;
use crate::records::bc_function::BcFunction;
use crate::records::bc_op::BcOp;
use crate::records::call_inliner::CallInliner;
use crate::type_aliases::reg::Reg;
use luaur_common::records::dense_hash_map::DenseHashMap;
use luaur_common::records::dense_hash_set::DenseHashSet;

impl<'a> CallInliner<'a> {
    pub fn call_inliner(
        caller: &'a mut BcFunction,
        target: &'a mut BcFunction,
        call_op: BcOp,
    ) -> Self {
        let caller_ptr: *mut BcFunction = caller;
        let call_ref = unsafe { (&*caller_ptr).inst(call_op) };
        let call = BcCallFB::from(caller_ptr, call_ref);
        let call_params = call.params();
        let target_reg = call.base.get_out_reg();

        CallInliner {
            caller,
            target,
            call,
            call_params,
            target_reg,
            caller_blocks_size_before_inline: 0,
            caller_inst_size_before_inline: 0,
            caller_vm_const_size_before_inline: 0,
            caller_proto_size_before_inline: 0,
            caller_up_val_size_before_inline: 0,
            return_ops: Vec::new(),
            call_projections: DenseHashSet::new(BcOp::new()),
            var_arg_moves: DenseHashMap::new(BcOp::new()),
        }
    }
}
