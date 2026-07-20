//! Node: `cxx:Macro:Luau.VM:VM/src/lvmexecute.cpp:79:VM_INTERRUPT`
//! Source: `VM/src/lvmexecute.cpp:79-91` (hand-ported)
//!
//! C++ `goto exit` becomes `return` — the macro only expands inside
//! `luau_execute_impl`, whose `exit:` label is the function end.

#[allow(non_snake_case)]
#[macro_export]
macro_rules! VM_INTERRUPT {
    ($L:expr, $pc:expr, $base:expr) => {{
        let interrupt = unsafe { (*(*$L).global).cb.interrupt };
        if let Some(interrupt) = interrupt {
            // the interrupt hook is called right before we advance pc
            $crate::macros::vm_protect::vm_protect!($L, $pc, $base, {
                unsafe {
                    (*(*$L).ci).savedpc = (*(*$L).ci).savedpc.add(1);
                    interrupt($L, -1);
                }
            });
            if unsafe { (*$L).status } != 0 {
                unsafe {
                    (*(*$L).ci).savedpc = (*(*$L).ci).savedpc.sub(1);
                }
                return;
            }
        }
    }};
}

pub use VM_INTERRUPT;
