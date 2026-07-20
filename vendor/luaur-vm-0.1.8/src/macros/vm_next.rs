#[allow(non_snake_case)]
macro_rules! VM_NEXT {
    () => {
        $crate::macros::vm_continue::VM_CONTINUE!(LUAU_INSN_OP!(*pc))
    };
}

pub(crate) use VM_NEXT;
