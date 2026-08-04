use crate::records::bc_function::BcFunction;
use crate::records::bc_op::BcOp;
use crate::records::call_inliner::CallInliner;

pub fn inline_call(
    caller: &mut BcFunction,
    target: &mut BcFunction,
    call_op: BcOp,
    target_proto_id: u32,
    caller_fb_vec_size: u32,
) -> bool {
    let mut inliner = CallInliner::call_inliner(caller, target, call_op, caller_fb_vec_size);
    inliner.inline_target(target_proto_id)
}
